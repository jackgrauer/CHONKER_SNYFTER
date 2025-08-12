use crate::actions::{Action, Panel};
use super::{PdfState, EditorState, UiState};

/// Core application state - single source of truth
#[derive(Debug, Clone)]
pub struct AppState {
    pub pdf: PdfState,
    pub editor: EditorState,
    pub ui: UiState,
    pub mode: AppMode,
    pub status_message: String,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppMode {
    Editing,      // Editing matrix
    Commanding,   // Command palette open
    Searching,    // Search mode active
    Help,         // Help overlay shown
}

/// Commands that need to be executed (side effects)
#[derive(Debug)]
pub enum Command {
    LoadPdf(std::path::PathBuf),
    ExtractText,
    RenderPdfPage,
    ExportMatrix,
    SaveFile(std::path::PathBuf, String),
    CopyToClipboard(String),
    PasteFromClipboard,
    ShowFileDialog,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            pdf: PdfState::default(),
            editor: EditorState::default(),
            ui: UiState::default(),
            mode: AppMode::Editing,
            status_message: "Welcome to Chonker6".to_string(),
            error_message: None,
        }
    }
}

impl AppState {
    /// Pure state transition function
    pub fn update(mut self, action: Action) -> (Self, Option<Command>) {
        // Clear error on any action
        self.error_message = None;
        
        match action {
            // PDF Actions
            Action::OpenPdf(path) => {
                self.status_message = format!("Loading {}...", path.display());
                (self, Some(Command::LoadPdf(path)))
            }
            Action::PdfLoaded(metadata) => {
                self.pdf.load(metadata);
                self.status_message = "PDF loaded successfully".to_string();
                (self, None)
            }
            Action::NavigatePage(direction) => {
                self.pdf.navigate(direction);
                self.status_message = format!("Page {}/{}", 
                    self.pdf.current_page + 1, 
                    self.pdf.page_count
                );
                // Trigger PDF re-render
                (self, Some(Command::RenderPdfPage))
            }
            Action::ZoomIn => {
                if self.pdf.zoom_in() {
                    self.status_message = format!("Zoom: {:.0}%", self.pdf.zoom_level * 100.0);
                    (self, Some(Command::RenderPdfPage))
                } else {
                    self.error_message = Some("Maximum zoom reached (120%)".to_string());
                    (self, None)
                }
            }
            Action::ZoomOut => {
                if self.pdf.zoom_out() {
                    self.status_message = format!("Zoom: {:.0}%", self.pdf.zoom_level * 100.0);
                    (self, Some(Command::RenderPdfPage))
                } else {
                    self.error_message = Some("Minimum zoom reached (90%)".to_string());
                    (self, None)
                }
            }
            Action::ZoomReset => {
                self.pdf.zoom_reset();
                self.status_message = "Zoom reset to 100%".to_string();
                (self, Some(Command::RenderPdfPage))
            }
            Action::ToggleAutoFit => {
                self.pdf.toggle_auto_fit();
                self.status_message = if self.pdf.auto_fit {
                    "PDF auto-fit enabled".to_string()
                } else {
                    format!("PDF auto-fit disabled (Zoom: {:.0}%)", self.pdf.zoom_level * 100.0)
                };
                (self, Some(Command::RenderPdfPage))
            }
            Action::ToggleDarkMode => {
                self.pdf.toggle_dark_mode();
                self.status_message = if self.pdf.dark_mode {
                    "PDF dark mode enabled".to_string()
                } else {
                    "PDF dark mode disabled".to_string()
                };
                (self, Some(Command::RenderPdfPage))
            }
            
            // Editor Actions
            Action::ExtractMatrix => {
                if self.pdf.is_loaded() {
                    self.status_message = "Extracting text...".to_string();
                    // Return command to trigger actual extraction
                    (self, Some(Command::ExtractText))
                } else {
                    self.error_message = Some("No PDF loaded".to_string());
                    (self, None)
                }
            }
            Action::MatrixExtracted(matrix) => {
                self.editor.set_matrix(matrix);
                self.mode = AppMode::Editing;  // Default to edit mode
                self.ui.focused_panel = Panel::Text;
                self.status_message = "Text extracted - ready to edit. ESC to exit edit mode.".to_string();
                (self, None)
            }
            Action::ExitEditMode => {
                // Since we only have edit mode now, this just resets focus
                self.ui.focused_panel = Panel::Pdf;
                self.status_message = "Focus returned to PDF panel.".to_string();
                (self, None)
            }
            Action::InsertChar(c) => {
                // Clear selection first if it exists, then insert character
                if self.editor.selection.is_some() {
                    self.editor.delete_selection();
                }
                
                // Handle newline specially
                if c == '\n' {
                    self.editor.insert_newline();
                } else {
                    self.editor.insert_char(c);
                }
                (self, None)
            }
            Action::MoveCursor(dir) => {
                self.editor.move_cursor(dir);
                // Update selection if active
                if self.editor.selection.is_some() {
                    self.editor.update_selection();
                }
                (self, None)
            }
            Action::DeleteChar => {
                self.editor.delete_char();
                (self, None)
            }
            Action::Copy => {
                // Debug: Check if we have a selection
                let has_selection = self.editor.selection.is_some();
                let has_mouse_selection = self.editor.mouse_selection.is_some();
                
                if let Some(text) = self.editor.get_selected_text() {
                    self.status_message = format!("Copied {} characters (selection: {}, mouse: {})", 
                        text.len(), has_selection, has_mouse_selection);
                    (self, Some(Command::CopyToClipboard(text)))
                } else {
                    self.error_message = Some(format!("No text selected (selection: {}, mouse: {})", 
                        has_selection, has_mouse_selection));
                    (self, None)
                }
            }
            Action::Cut => {
                // Debug: Check if we have a selection
                let has_selection = self.editor.selection.is_some();
                let has_mouse_selection = self.editor.mouse_selection.is_some();
                
                if let Some(text) = self.editor.get_selected_text() {
                    self.editor.delete_selection();
                    self.status_message = format!("Cut {} characters (selection: {}, mouse: {})", 
                        text.len(), has_selection, has_mouse_selection);
                    (self, Some(Command::CopyToClipboard(text)))
                } else {
                    self.error_message = Some(format!("No text selected for cut (selection: {}, mouse: {})", 
                        has_selection, has_mouse_selection));
                    (self, None)
                }
            }
            Action::Paste(text) => {
                // Use block mode if we have a block selection active
                let paste_mode = if let Some(ref selection) = self.editor.selection {
                    selection.mode
                } else {
                    crate::actions::SelectionMode::Block // Default to block mode
                };
                self.editor.paste_text_with_mode(text, paste_mode);
                self.status_message = "Text pasted".to_string();
                (self, None)
            }
            Action::PasteFromSystem => {
                // Always in edit mode
                (self, Some(Command::PasteFromClipboard))
            }
            Action::SelectAll => {
                // Always in edit mode
                self.editor.select_all();
                self.status_message = "All text selected".to_string();
                (self, None)
            }
            Action::DeleteSelection => {
                // Always in edit mode
                self.editor.delete_selection();
                self.status_message = "Selection deleted".to_string();
                (self, None)
            }
            Action::ExportMatrix => {
                if self.editor.has_content() {
                    (self, Some(Command::ExportMatrix))
                } else {
                    self.error_message = Some("No matrix content to export".to_string());
                    (self, None)
                }
            }
            
            // UI Actions
            Action::SwitchPanel(panel) => {
                self.ui.focused_panel = panel;
                self.status_message = format!("Focused: {:?} panel", panel);
                (self, None)
            }
            Action::ShowHelp => {
                self.mode = AppMode::Help;
                (self, None)
            }
            Action::HideHelp => {
                self.mode = AppMode::Editing;
                (self, None)
            }
            Action::SetStatus(msg) => {
                self.status_message = msg;
                (self, None)
            }
            Action::Error(msg) => {
                self.error_message = Some(msg);
                (self, None)
            }
            
            // Mouse actions
            Action::MouseDown(col, row, _button, modifiers) => {
                // Always in edit mode
                let pos = crate::actions::Position { row: row as usize, col: col as usize };
                
                // Move cursor to click position immediately
                self.editor.cursor = pos;
                
                // Clear any existing selection on new click
                self.editor.selection = None;
                
                // Determine selection mode based on modifiers - Block is now default
                let selection_mode = if modifiers.contains(crossterm::event::KeyModifiers::ALT) {
                    crate::actions::SelectionMode::Line  // Alt+click for line selection
                } else {
                    crate::actions::SelectionMode::Block // Default to block selection
                };
                
                // Prepare for potential drag selection
                self.editor.start_mouse_selection(pos, selection_mode);
                
                self.status_message = "Click to position cursor, drag to select".to_string();
                (self, None)
            }
            Action::MouseDrag(col, row) => {
                // Always in edit mode
                let pos = crate::actions::Position { row: row as usize, col: col as usize };
                self.editor.update_mouse_selection(pos);
                (self, None)
            }
            Action::MouseUp(col, row) => {
                // Always in edit mode
                let pos = crate::actions::Position { row: row as usize, col: col as usize };
                
                // Check if this was a click (no drag) or a selection
                if let Some(ref mouse_sel) = self.editor.mouse_selection {
                    if mouse_sel.start == pos {
                        // Just a click - clear selection and position cursor
                        self.editor.cursor = pos;
                        self.editor.selection = None;
                        self.editor.mouse_selection = None;
                        self.status_message = format!("Cursor at ({}, {})", pos.row + 1, pos.col + 1);
                    } else {
                        // Drag selection completed
                        self.editor.update_mouse_selection(pos);
                        self.editor.complete_mouse_selection();
                        self.status_message = "Selection completed".to_string();
                    }
                }
                (self, None)
            }
            
            // Terminal Panel actions
            Action::ToggleTerminalPanel => {
                self.ui.terminal_panel.toggle();
                self.status_message = if self.ui.terminal_panel.visible {
                    "Terminal panel shown (Ctrl+T to hide)".to_string()
                } else {
                    "Terminal panel hidden (Ctrl+T to show)".to_string()
                };
                (self, None)
            }
            Action::ClearTerminalOutput => {
                self.ui.terminal_panel.clear();
                self.status_message = "Terminal output cleared".to_string();
                (self, None)
            }
            Action::AddTerminalOutput(text) => {
                self.ui.terminal_panel.add_output(text);
                (self, None)
            }
            Action::ScrollTerminalUp => {
                self.ui.terminal_panel.scroll_up();
                (self, None)
            }
            Action::ScrollTerminalDown => {
                self.ui.terminal_panel.scroll_down();
                (self, None)
            }
            Action::SelectTerminalText(start, end) => {
                self.ui.terminal_panel.selected_lines = Some((start, end));
                self.status_message = format!("Selected {} lines in terminal", end - start + 1);
                (self, None)
            }
            Action::CopyTerminalSelection => {
                if let Some(text) = self.ui.terminal_panel.get_selected_text() {
                    let lines_count = text.lines().count();
                    self.status_message = format!("Copied {} line{} from terminal", 
                        lines_count, 
                        if lines_count == 1 { "" } else { "s" });
                    (self, Some(Command::CopyToClipboard(text)))
                } else {
                    // Auto-select all terminal content and copy
                    let all_text = self.ui.terminal_panel.content.join("\n");
                    if !all_text.is_empty() {
                        let lines_count = self.ui.terminal_panel.content.len();
                        self.status_message = format!("Copied all {} line{} from terminal", 
                            lines_count, 
                            if lines_count == 1 { "" } else { "s" });
                        (self, Some(Command::CopyToClipboard(all_text)))
                    } else {
                        self.error_message = Some("No terminal content to copy".to_string());
                        (self, None)
                    }
                }
            }
            
            // System
            Action::Quit => {
                // State doesn't change, but main loop will exit
                (self, None)
            }
            
            _ => (self, None), // Unhandled actions don't change state
        }
    }
    
    pub fn is_running(&self) -> bool {
        // Check if we should keep running
        true // Will be set to false by Quit action handler
    }
}