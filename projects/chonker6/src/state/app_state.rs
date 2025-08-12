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
    Viewing,      // Just viewing PDF
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
            mode: AppMode::Viewing,
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
            Action::EnterEditMode => {
                if self.editor.has_content() {
                    self.mode = AppMode::Editing;
                    self.status_message = "Edit mode enabled. Use arrows to navigate, type to edit.".to_string();
                } else {
                    self.error_message = Some("No text to edit. Extract text first with Ctrl+E.".to_string());
                }
                (self, None)
            }
            Action::ExitEditMode => {
                self.mode = AppMode::Viewing;
                self.status_message = "Returned to view mode.".to_string();
                (self, None)
            }
            Action::InsertChar(c) if self.mode == AppMode::Editing => {
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
            Action::MoveCursor(dir) if self.mode == AppMode::Editing => {
                self.editor.move_cursor(dir);
                // Update selection if active
                if self.editor.selection.is_some() {
                    self.editor.update_selection();
                }
                (self, None)
            }
            Action::DeleteChar if self.mode == AppMode::Editing => {
                self.editor.delete_char();
                (self, None)
            }
            Action::StartSelection(_) if self.mode == AppMode::Editing => {
                self.editor.start_selection();
                (self, None)
            }
            Action::StartBlockSelection(_) if self.mode == AppMode::Editing => {
                self.editor.start_block_selection();
                self.status_message = "Block selection mode - use arrows to select rectangular region".to_string();
                (self, None)
            }
            Action::UpdateSelection(_) if self.mode == AppMode::Editing => {
                self.editor.update_selection();
                (self, None)
            }
            Action::EndSelection => {
                self.editor.end_selection();
                (self, None)
            }
            Action::Copy => {
                if let Some(text) = self.editor.get_selected_text() {
                    (self, Some(Command::CopyToClipboard(text)))
                } else {
                    self.error_message = Some("No text selected".to_string());
                    (self, None)
                }
            }
            Action::Cut => {
                if let Some(text) = self.editor.get_selected_text() {
                    self.editor.delete_selection();
                    (self, Some(Command::CopyToClipboard(text)))
                } else {
                    self.error_message = Some("No text selected".to_string());
                    (self, None)
                }
            }
            Action::Paste(text) => {
                if self.mode == AppMode::Editing {
                    self.editor.paste_text(text);
                    self.status_message = "Text pasted".to_string();
                } else {
                    self.error_message = Some("Must be in edit mode to paste".to_string());
                }
                (self, None)
            }
            Action::PasteFromSystem => {
                if self.mode == AppMode::Editing {
                    (self, Some(Command::PasteFromClipboard))
                } else {
                    self.error_message = Some("Must be in edit mode to paste".to_string());
                    (self, None)
                }
            }
            Action::SelectAll => {
                if self.mode == AppMode::Editing {
                    self.editor.select_all();
                    self.status_message = "All text selected".to_string();
                } else {
                    self.error_message = Some("Must be in edit mode to select".to_string());
                }
                (self, None)
            }
            Action::DeleteSelection => {
                if self.mode == AppMode::Editing {
                    self.editor.delete_selection();
                    self.status_message = "Selection deleted".to_string();
                } else {
                    self.error_message = Some("Must be in edit mode".to_string());
                }
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
                self.mode = AppMode::Viewing;
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
                if self.mode == AppMode::Editing {
                    let pos = crate::actions::Position { row: row as usize, col: col as usize };
                    
                    // Move cursor to click position immediately
                    self.editor.cursor = pos;
                    
                    // Clear any existing selection on new click
                    self.editor.selection = None;
                    
                    // Determine selection mode based on modifiers
                    let selection_mode = if modifiers.contains(crossterm::event::KeyModifiers::ALT) {
                        crate::actions::SelectionMode::Block
                    } else {
                        crate::actions::SelectionMode::Line
                    };
                    
                    // Prepare for potential drag selection
                    self.editor.start_mouse_selection(pos, selection_mode);
                    
                    self.status_message = "Click to position cursor, drag to select".to_string();
                }
                (self, None)
            }
            Action::MouseDrag(col, row) => {
                if self.mode == AppMode::Editing {
                    let pos = crate::actions::Position { row: row as usize, col: col as usize };
                    self.editor.update_mouse_selection(pos);
                }
                (self, None)
            }
            Action::MouseUp(col, row) => {
                if self.mode == AppMode::Editing {
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
            Action::ResizeTerminalPanel(delta) => {
                let new_height = if delta > 0 {
                    (self.ui.terminal_panel.height + delta.abs() as u16).min(20)
                } else {
                    self.ui.terminal_panel.height.saturating_sub(delta.abs() as u16).max(3)
                };
                self.ui.terminal_panel.height = new_height;
                self.status_message = format!("Terminal panel height: {} lines", new_height);
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
                    self.error_message = Some("No terminal text selected".to_string());
                    (self, None)
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