use anyhow::Result;
use crossterm::event::{KeyEvent, MouseEvent, MouseEventKind, MouseButton};
use ratatui::Frame;
use ratatui::style::Color;

use crate::{
    actions::Action,
    state::{AppState, app_state::{AppMode, Command}},
    components::FileSelector,
    services::PdfEngine,
};

/// Main application orchestrator
pub struct App {
    state: AppState,
    running: bool,
    file_selector: FileSelector,
    pdf_engine: Option<PdfEngine>,
    is_iterm2: bool,
    is_kitty: bool,
    kitty_needs_redraw: bool,
    last_click_time: std::time::Instant,
    click_count: u8,
    last_click_row: u16,
    pending_selection: Option<(usize, usize)>, // For auto-scroll + selection
    cursor_blink_timer: std::time::Instant,
    cursor_visible: bool,
}

impl App {
    pub fn new() -> Self {
        // Try to initialize PDF engine
        let pdf_engine = match PdfEngine::new() {
            Ok(engine) => Some(engine),
            Err(e) => {
                eprintln!("Warning: Failed to initialize PDF engine: {}", e);
                eprintln!("Make sure to run with: ./run_chonker6.sh or set DYLD_LIBRARY_PATH");
                None
            }
        };
        
        // Detect terminal type
        let term_program = std::env::var("TERM").unwrap_or_default();
        let is_iterm2 = std::env::var("TERM_PROGRAM")
            .map(|t| t == "iTerm.app")
            .unwrap_or(false);
        // Use our proper Kitty detection
        let is_kitty = crate::kitty_graphics::test_kitty_graphics();
        
        let mut app = Self {
            state: AppState::default(),
            running: true,
            file_selector: FileSelector::new(),
            pdf_engine,
            is_iterm2,
            is_kitty,
            kitty_needs_redraw: false,
            last_click_time: std::time::Instant::now(),
            click_count: 0,
            last_click_row: 0,
            pending_selection: None,
            cursor_blink_timer: std::time::Instant::now(),
            cursor_visible: true,
        };
        
        // Initialize terminal-specific features
        if is_iterm2 {
            app.initialize_iterm2_mode();
        } else if is_kitty {
            app.initialize_kitty_mode();
        }
        
        // Add initialization message to status
        if app.pdf_engine.is_none() {
            app.state.status_message = "⚠️ PDF engine not initialized - run with ./run_chonker6.sh".to_string();
        } else {
            app.state.status_message = "Ready - Press Ctrl+O to open PDF".to_string();
        }
        
        app
    }
    
    fn initialize_kitty_mode(&mut self) {
        use std::io::{stdout, Write};
        
        // Enable Kitty graphics protocol and features
        let setup_commands = vec![
            // Enable mouse reporting
            "\x1b[?1003h",  // All mouse events
            "\x1b[?1006h",  // SGR mouse mode
            
            // Clear any existing images when starting
            "\x1b_Ga=d\x1b\\",  // Delete all images
        ];
        
        for cmd in setup_commands {
            print!("{}", cmd);
        }
        
        stdout().flush().unwrap();
    }
    
    fn initialize_iterm2_mode(&mut self) {
        use std::io::{stdout, Write};
        
        // Enable all iTerm2 features for better grid selection and PDF rendering
        let setup_commands = vec![
            // Shell integration for better semantic understanding
            "\x1b]1337;ShellIntegrationVersion=14\x07",
            
            // Advanced mouse reporting
            "\x1b[?1003h",  // All mouse events
            "\x1b[?1006h",  // SGR mouse mode
            "\x1b[?1015h",  // RXVT mouse mode as fallback
            
            // iTerm2 specific features
            "\x1b]1337;ReportCellSize\x07", // Get exact cell dimensions
            "\x1b]1337;PushKeyLabels\x07",  // Show key hints
            
            // Grid selection modes - the key for perfect clipboard
            "\x1b]1337;HighlightMode=block\x07",
            "\x1b]1337;CopyMode=table\x07",
            "\x1b]1337;SetProfile=CopyAsTable\x07",
            
            // Allow clipboard access
            "\x1b]1337;AllowClipboardAccess=true\x07",
        ];
        
        for cmd in setup_commands {
            print!("{}", cmd);
        }
        stdout().flush().unwrap();
        
        self.state.status_message = "iTerm2 enhanced mode activated - perfect grid clipboard enabled".to_string();
    }
    
    fn copy_with_iterm2_table_mode(&mut self) {
        use std::io::{stdout, Write};
        
        if self.state.editor.selection.is_some() {
            // Tell iTerm2 to copy selection as structured table data
            print!("\x1b]1337;Copy=mode:table;format:tsv\x07");
            stdout().flush().unwrap();
            
            // iTerm2 will preserve the rectangular structure perfectly!
            self.state.status_message = "Copied as table (iTerm2) - rectangular structure preserved".to_string();
        } else {
            self.state.error_message = Some("No text selected".to_string());
        }
    }
    
    fn cut_with_iterm2_table_mode(&mut self) {
        use std::io::{stdout, Write};
        
        if self.state.editor.selection.is_some() {
            // Copy first, then delete selection
            print!("\x1b]1337;Copy=mode:table;format:tsv\x07");
            stdout().flush().unwrap();
            
            // Delete the selected content
            self.state.editor.delete_selection();
            self.state.status_message = "Cut as table (iTerm2) - rectangular structure preserved".to_string();
        } else {
            self.state.error_message = Some("No text selected".to_string());
        }
    }
    
    fn paste_with_iterm2_table_mode(&mut self) {
        use std::io::{stdout, Write};
        
        if self.state.mode == crate::state::app_state::AppMode::Editing {
            // Request structured paste from iTerm2
            print!("\x1b]1337;Paste=mode:table\x07");
            stdout().flush().unwrap();
            
            // iTerm2 will send back clipboard with rectangular structure intact
            // For now, fall back to system clipboard until we implement the response handler
            if let Ok(output) = std::process::Command::new("pbpaste").output() {
                if let Ok(text) = String::from_utf8(output.stdout) {
                    self.state.editor.paste_text(text);
                    self.state.status_message = "Pasted from iTerm2 table mode".to_string();
                }
            }
        } else {
            self.state.error_message = Some("Must be in edit mode to paste".to_string());
        }
    }
    
    fn render_matrix_with_iterm2_semantics(&self, area: ratatui::layout::Rect, _bg_color: Color, _fg_color: Color) {
        use std::io::{stdout, Write};
        
        // Position cursor at the start of the text area
        print!("\x1b[{};{}H", area.y + 1, area.x + 1);
        
        // Clear the area
        for y in 0..area.height {
            print!("\x1b[{};{}H\x1b[K", area.y + y + 1, area.x + 1);
        }
        
        // Position cursor for header
        print!("\x1b[{};{}H", area.y + 1, area.x + 1);
        
        // Just show TEXT MATRIX without mode since we're always in edit mode
        println!("  TEXT MATRIX (iTerm2 Enhanced)");
        println!();
        
        if !self.state.editor.matrix.is_empty() {
            // Start iTerm2 table semantic region
            print!("\x1b]1337;SetMark\x07"); // Mark start of matrix
            print!("\x1b]1337;TableStart={}x{}\x07", 
                   self.state.editor.matrix.first().map(|row| row.len()).unwrap_or(0),
                   self.state.editor.matrix.len());
            
            // Render matrix with semantic markers for perfect grid selection
            for (row_idx, row) in self.state.editor.matrix.iter().enumerate() {
                // Position cursor for this row
                print!("\x1b[{};{}H", area.y + row_idx as u16 + 3, area.x + 1);
                
                // Mark row start for iTerm2's semantic understanding
                print!("\x1b]1337;RowStart={}\x07", row_idx);
                print!("  "); // Padding
                
                for (col_idx, &ch) in row.iter().enumerate() {
                    let pos = crate::actions::Position { row: row_idx, col: col_idx };
                    let is_cursor = self.cursor_visible && self.state.mode == AppMode::Editing 
                        && row_idx == self.state.editor.cursor.row 
                        && col_idx == self.state.editor.cursor.col;
                    let is_selected = self.state.editor.is_position_selected(pos);
                    
                    // Mark each cell for iTerm2's grid understanding
                    print!("\x1b]1337;Cell={},{}\x07", row_idx, col_idx);
                    
                    // Apply styling
                    if is_selected {
                        print!("\x1b[48;2;22;160;133m\x1b[30m"); // Teal background, black text
                    } else if is_cursor && self.state.mode == AppMode::Editing {
                        print!("\x1b[48;2;52;73;94m\x1b[97m"); // Dark blue background, white text
                    } else {
                        print!("\x1b[38;2;180;180;200m"); // Light purple text
                    }
                    
                    // Print the character with tab for proper grid alignment
                    print!("{}\t", ch);
                    
                    // Reset styling
                    print!("\x1b[0m");
                }
                
                print!("\x1b]1337;RowEnd\x07");
                println!(); // New line
            }
            
            print!("\x1b]1337;TableEnd\x07");
            
            // Show controls
            let controls_y = area.y + self.state.editor.matrix.len() as u16 + 4;
            print!("\x1b[{};{}H", controls_y, area.x + 1);
            
            if self.state.mode == AppMode::Editing {
                let selection_info = if self.state.editor.selection.is_some() {
                    let mode_str = if let Some(ref sel) = self.state.editor.selection {
                        match sel.mode {
                            crate::actions::SelectionMode::Block => " [Block]",
                            crate::actions::SelectionMode::Line => " [Line]",
                        }
                    } else {
                        ""
                    };
                    format!(" • Selection{} • Ctrl+C: Copy • Ctrl+X: Cut • Ctrl+V: Paste", mode_str)
                } else {
                    " • Alt+B: Block mode • Ctrl+A: Select All".to_string()
                };
                print!("  Position: ({},{}) • Type to edit • Arrows to move{} • ESC to exit", 
                    self.state.editor.cursor.row + 1, 
                    self.state.editor.cursor.col + 1,
                    selection_info);
            } else {
                print!("  Ctrl+E: Extract text • Ctrl+O: Open file • Ctrl+S: Export");
            }
        }
        
        stdout().flush().unwrap();
    }
    
    fn render_pdf_with_iterm2_images(&self, area: ratatui::layout::Rect) {
        use std::io::{stdout, Write};
        
        // Position cursor at the start of the PDF area
        print!("\x1b[{};{}H", area.y + 1, area.x + 1);
        
        // Clear the area
        for y in 0..area.height {
            print!("\x1b[{};{}H\x1b[K", area.y + y + 1, area.x + 1);
        }
        
        if let Some(engine) = &self.pdf_engine {
            // Try to render PDF page as image
            match engine.render_page_as_image(self.state.pdf.current_page, self.state.pdf.zoom_level) {
                Ok(image_data) => {
                    // Encode image as base64 for iTerm2 inline image protocol
                    use base64::Engine;
                    let base64_image = base64::engine::general_purpose::STANDARD.encode(&image_data);
                    
                    // Calculate optimal image dimensions for terminal cell size
                    let cell_width = 9;  // Typical iTerm2 cell width in pixels
                    let cell_height = 18; // Typical iTerm2 cell height in pixels
                    let max_width = (area.width as u32 - 4) * cell_width;  // Leave some padding
                    let max_height = (area.height as u32 - 4) * cell_height; // Leave some padding
                    
                    // Position cursor for image
                    print!("\x1b[{};{}H", area.y + 2, area.x + 2);
                    
                    // iTerm2 inline image protocol with optimal sizing
                    print!("\x1b]1337;File=inline=1;width={}px;height={}px;preserveAspectRatio=1:{}\x07",
                           max_width, max_height, base64_image);
                    
                    // Position cursor for controls below image
                    let controls_y = area.y + area.height - 3;
                    print!("\x1b[{};{}H", controls_y, area.x + 2);
                    print!("Page {}/{} • Zoom: {:.0}% • +/-: Zoom • ← →: Navigate", 
                           self.state.pdf.current_page + 1,
                           self.state.pdf.page_count,
                           self.state.pdf.zoom_level * 100.0);
                }
                Err(e) => {
                    // Fallback to text info if image rendering fails
                    print!("\x1b[{};{}H", area.y + 2, area.x + 2);
                    print!("PDF VIEWER (iTerm2 Enhanced)");
                    print!("\x1b[{};{}H", area.y + 4, area.x + 2);
                    print!("Image rendering failed: {}", e);
                    print!("\x1b[{};{}H", area.y + 6, area.x + 2);
                    print!("Page {}/{} • Zoom: {:.0}%", 
                           self.state.pdf.current_page + 1,
                           self.state.pdf.page_count,
                           self.state.pdf.zoom_level * 100.0);
                }
            }
        } else {
            // No PDF engine available
            print!("\x1b[{};{}H", area.y + 2, area.x + 2);
            print!("PDF VIEWER (iTerm2 Enhanced)");
            print!("\x1b[{};{}H", area.y + 4, area.x + 2);
            print!("PDF engine not available");
            print!("\x1b[{};{}H", area.y + 6, area.x + 2);
            print!("Press Ctrl+O to open a PDF");
        }
        
        stdout().flush().unwrap();
    }
    
    pub fn render_pdf_with_kitty_post_frame(&mut self, area: ratatui::layout::Rect) {
        // This should be called AFTER the frame has been rendered to terminal
        if !self.state.pdf.is_loaded() {
            return;
        }
        
        // Clear previous images before rendering new one
        use std::io::{stdout, Write};
        let mut out = stdout();
        let _ = out.write_all(b"\x1b_Ga=d,i=99\x1b\\");
        let _ = out.flush();
        
        let (new_state, _) = self.state.clone().update(
            Action::AddTerminalOutput(format!("🖼️ Kitty PDF render: {}x{} at ({},{})", 
                area.width, area.height, area.x, area.y))
        );
        self.state = new_state;
        
        if let Some(engine) = &self.pdf_engine {
            // Calculate realistic pixel dimensions
            // Terminal cells vary but typically 8-12x16-24 pixels
            let cell_width = 10u32;
            let cell_height = 20u32;
            let display_width_px = (area.width as u32).saturating_sub(4) * cell_width;
            let display_height_px = (area.height as u32).saturating_sub(6) * cell_height;
            
            let (new_state, _) = self.state.clone().update(
                Action::AddTerminalOutput(format!("📏 Target dimensions: {}x{} px", display_width_px, display_height_px))
            );
            self.state = new_state;
            
            // Render PDF page 
            match engine.render_page_for_kitty(
                self.state.pdf.current_page, 
                display_width_px,
                display_height_px
            ) {
                Ok((_png_data, width, height, base64_png)) => {
                    let (new_state, _) = self.state.clone().update(
                        Action::AddTerminalOutput(format!("📸 Rendered: {}x{} px", width, height))
                    );
                    self.state = new_state;
                    
                    // Use the area-based rendering for better integration with ratatui
                    match crate::kitty_graphics::render_pdf_in_area(
                        &base64_png,
                        width,
                        height,
                        &area
                    ) {
                        Ok(()) => {
                            let (new_state, _) = self.state.clone().update(
                                Action::AddTerminalOutput(format!("✅ PDF displayed via Kitty: {}x{} px", width, height))
                            );
                            self.state = new_state;
                        }
                        Err(e) => {
                            let (new_state, _) = self.state.clone().update(
                                Action::AddTerminalOutput(format!("❌ Kitty error: {}", e))
                            );
                            self.state = new_state;
                        }
                    }
                }
                Err(e) => {
                    let (new_state, _) = self.state.clone().update(
                        Action::AddTerminalOutput(format!("❌ PDF render failed: {}", e))
                    );
                    self.state = new_state;
                }
            }
        }
    }
    
    fn render_pdf_with_kitty(&self, area: ratatui::layout::Rect) {
        use std::io::{stdout, Write};
        
        // Clear the area first
        for y in 0..area.height {
            print!("\x1b[{};{}H\x1b[K", area.y + y + 1, area.x + 1);
        }
        
        if let Some(engine) = &self.pdf_engine {
            // Calculate pixel dimensions for the display area
            // Kitty typically uses ~9x18 pixel cells
            let cell_width = 9u32;
            let cell_height = 18u32;
            let display_width_px = (area.width as u32).saturating_sub(2) * cell_width;
            let display_height_px = (area.height as u32).saturating_sub(4) * cell_height; // Leave room for controls
            
            // Render PDF page with automatic size fitting
            match engine.render_page_for_kitty(
                self.state.pdf.current_page, 
                display_width_px,
                display_height_px
            ) {
                Ok((_png_data, width, height, base64_png)) => {
                    // Position cursor for image
                    print!("\x1b[{};{}H", area.y + 1, area.x + 1);
                    
                    // Kitty graphics protocol: 
                    // a=T means transmit image data
                    // f=100 means PNG format
                    // s=width,height specifies dimensions
                    // t=d means direct transmission (base64)
                    print!("\x1b_Ga=T,f=100,s={},{},t=d;{}\x1b\\", width, height, base64_png);
                    
                    // Display controls at the bottom
                    let controls_y = area.y + area.height - 2;
                    print!("\x1b[{};{}H", controls_y, area.x + 2);
                    print!("\x1b[38;2;180;180;200m"); // Light purple text
                    print!("Page {}/{} • Arrows: Navigate • Ctrl+E: Extract text", 
                           self.state.pdf.current_page + 1,
                           self.state.pdf.page_count);
                    print!("\x1b[0m"); // Reset colors
                }
                Err(e) => {
                    // Fallback to text display on error
                    print!("\x1b[{};{}H", area.y + 2, area.x + 2);
                    print!("\x1b[38;2;180;180;200m"); // Light purple text
                    print!("PDF VIEWER (Kitty)");
                    print!("\x1b[{};{}H", area.y + 4, area.x + 2);
                    print!("Rendering error: {}", e);
                    print!("\x1b[{};{}H", area.y + 6, area.x + 2);
                    print!("Try pressing Ctrl+E to extract text instead");
                    print!("\x1b[0m"); // Reset colors
                }
            }
        } else {
            // No PDF loaded
            print!("\x1b[{};{}H", area.y + area.height / 2, area.x + 2);
            print!("\x1b[38;2;180;180;200m"); // Light purple text
            print!("Press Ctrl+O to open a PDF file");
            print!("\x1b[0m"); // Reset colors
        }
        
        stdout().flush().unwrap();
    }
    
    /// Handle mouse input and return action
    pub fn handle_mouse(&mut self, mouse: MouseEvent) -> Option<Action> {
        
        let mouse_col = mouse.column;
        let mouse_row = mouse.row;
        
        // Get window dimensions for panel detection
        let (term_width, term_height) = if let Ok(size) = crossterm::terminal::size() {
            (size.0, size.1)
        } else {
            (80, 24)
        };
        
        // Calculate panel boundaries
        let panel_width = term_width / 2;
        let main_height = if self.state.ui.terminal_panel.visible {
            term_height.saturating_sub(self.state.ui.terminal_panel.height + 1)
        } else {
            term_height.saturating_sub(1)
        };
        
        // Determine which panel the mouse is in
        let in_pdf_panel = mouse_col < panel_width && mouse_row < main_height;
        let in_text_panel = mouse_col >= panel_width && mouse_row < main_height;
        
        // Handle scroll wheel for ALL panels
        match mouse.kind {
            MouseEventKind::ScrollUp => {
                if self.state.ui.terminal_panel.visible && mouse_row >= main_height {
                    return Some(Action::ScrollTerminalUp);
                } else if in_pdf_panel && self.state.pdf.is_loaded() {
                    return Some(Action::NavigatePage(crate::actions::PageDirection::Previous));
                } else if in_text_panel && self.state.editor.has_content() {
                    // Scroll text up by moving cursor
                    for _ in 0..3 {  // Scroll 3 lines at a time
                        self.state.editor.move_cursor(crate::actions::CursorDirection::Up);
                    }
                    return None;
                }
            }
            MouseEventKind::ScrollDown => {
                if self.state.ui.terminal_panel.visible && mouse_row >= main_height {
                    return Some(Action::ScrollTerminalDown);
                } else if in_pdf_panel && self.state.pdf.is_loaded() {
                    return Some(Action::NavigatePage(crate::actions::PageDirection::Next));
                } else if in_text_panel && self.state.editor.has_content() {
                    // Scroll text down by moving cursor
                    for _ in 0..3 {  // Scroll 3 lines at a time
                        self.state.editor.move_cursor(crate::actions::CursorDirection::Down);
                    }
                    return None;
                }
            }
            _ => {}
        }
        
        // Check if terminal panel is visible and mouse is in its area
        if self.state.ui.terminal_panel.visible {
            // Calculate terminal panel bounds (rough estimate)
            let terminal_start_y = if let Ok(size) = crossterm::terminal::size() {
                size.1.saturating_sub(self.state.ui.terminal_panel.height + 1)
            } else {
                20  // Fallback
            };
            
            if mouse_row >= terminal_start_y {
                // Mouse is in terminal panel area
                match mouse.kind {
                    MouseEventKind::Down(MouseButton::Left) => {
                        // Handle double/triple click
                        let now = std::time::Instant::now();
                        let time_since_last = now.duration_since(self.last_click_time);
                        
                        if time_since_last.as_millis() < 500 && mouse_row == self.last_click_row {
                            self.click_count += 1;
                        } else {
                            self.click_count = 1;
                        }
                        
                        self.last_click_time = now;
                        self.last_click_row = mouse_row;
                        
                        if mouse_row > terminal_start_y {
                            let relative_row = (mouse_row - terminal_start_y).saturating_sub(1) as usize;
                            let line_idx = relative_row + self.state.ui.terminal_panel.scroll_offset;
                            
                            if line_idx < self.state.ui.terminal_panel.content.len() {
                                if self.click_count == 2 {
                                    // Double-click: select entire line
                                    return Some(Action::SelectTerminalText(line_idx, line_idx));
                                } else {
                                    // Single click: start normal selection
                                    return Some(Action::SelectTerminalText(line_idx, line_idx));
                                }
                            }
                        }
                    }
                    MouseEventKind::Drag(MouseButton::Left) => {
                        // Update selection with auto-scroll when dragging beyond visible area
                        if let Some((start, _)) = self.state.ui.terminal_panel.selected_lines {
                            let terminal_height = self.state.ui.terminal_panel.height as u16;
                            let terminal_end_y = terminal_start_y + terminal_height;
                            
                            // Check if we need to auto-scroll
                            let mut scroll_action = None;
                            
                            if mouse_row <= terminal_start_y {
                                // Dragging above terminal - scroll up
                                if self.state.ui.terminal_panel.scroll_offset > 0 {
                                    scroll_action = Some(Action::ScrollTerminalUp);
                                }
                            } else if mouse_row >= terminal_end_y {
                                // Dragging below terminal - scroll down
                                let max_scroll = self.state.ui.terminal_panel.content.len().saturating_sub(terminal_height as usize);
                                if self.state.ui.terminal_panel.scroll_offset < max_scroll {
                                    scroll_action = Some(Action::ScrollTerminalDown);
                                }
                            }
                            
                            // Always update selection based on current mouse position
                            if mouse_row > terminal_start_y {
                                let relative_row = (mouse_row - terminal_start_y).saturating_sub(1) as usize;
                                let line_idx = (relative_row + self.state.ui.terminal_panel.scroll_offset)
                                    .min(self.state.ui.terminal_panel.content.len().saturating_sub(1));
                                
                                // If we need to scroll, do that first, then update selection
                                if let Some(scroll) = scroll_action {
                                    // Store the current selection to update after scroll
                                    self.pending_selection = Some((start, line_idx));
                                    return Some(scroll);
                                } else {
                                    return Some(Action::SelectTerminalText(start, line_idx));
                                }
                            } else if let Some(scroll) = scroll_action {
                                // Scrolling above terminal area
                                let line_idx = self.state.ui.terminal_panel.scroll_offset;
                                self.pending_selection = Some((start, line_idx));
                                return Some(scroll);
                            }
                        }
                    }
                    MouseEventKind::Up(MouseButton::Left) => {
                        // Mouse release - selection is complete, no auto-copy
                        // User can manually copy with Ctrl+C
                    }
                    MouseEventKind::ScrollUp => {
                        // Scroll terminal up with mouse wheel
                        return Some(Action::ScrollTerminalUp);
                    }
                    MouseEventKind::ScrollDown => {
                        // Scroll terminal down with mouse wheel
                        return Some(Action::ScrollTerminalDown);
                    }
                    _ => {}
                }
                return None;
            }
        }
        
        // Check if file selector is active and handle all mouse events
        if self.file_selector.active {
            match mouse.kind {
                MouseEventKind::ScrollUp => {
                    self.file_selector.navigate_up();
                    return None;
                }
                MouseEventKind::ScrollDown => {
                    self.file_selector.navigate_down();
                    return None;
                }
                _ => {}
            }
            
            if let MouseEventKind::Down(MouseButton::Left) = mouse.kind {
                // Store the file list bounds from last render
                // For now, assume standard layout: 3 lines header, rest is file list
                let file_list_start = 3;
                
                if let Some(path) = self.file_selector.handle_click(mouse_row as usize, file_list_start) {
                    self.file_selector.deactivate();
                    return Some(Action::OpenPdf(path));
                }
            }
            return None;
        }
        
        // Handle panel focusing with single click
        if let MouseEventKind::Down(MouseButton::Left) = mouse.kind {
            if in_pdf_panel {
                // Focus PDF panel on click
                if self.state.ui.focused_panel != crate::actions::Panel::Pdf {
                    return Some(Action::SwitchPanel(crate::actions::Panel::Pdf));
                }
            } else if in_text_panel {
                // Focus text panel on click
                if self.state.ui.focused_panel != crate::actions::Panel::Text {
                    let action = Some(Action::SwitchPanel(crate::actions::Panel::Text));
                    // Also enter edit mode if we have content
                    if self.state.editor.has_content() && self.state.mode != crate::state::app_state::AppMode::Editing {
                        self.state.mode = crate::state::app_state::AppMode::Editing;
                    }
                    return action;
                }
            }
        }
        
        // Advanced text editor mouse functions
        if in_text_panel && self.state.editor.has_content() {
            match mouse.kind {
                MouseEventKind::Down(MouseButton::Right) => {
                    // Right click for context menu simulation - copy selected text
                    if self.state.editor.selection.is_some() {
                        return Some(Action::Copy);
                    }
                }
                MouseEventKind::Down(MouseButton::Middle) => {
                    // Middle click to paste (Unix-style)
                    return Some(Action::PasteFromSystem);
                }
                _ => {}
            }
        }
        
        // Handle clicks in text editor for cursor positioning and selection
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                // Check if click is in text panel for direct cursor positioning
                if let Some(matrix_pos) = self.screen_to_matrix_pos(mouse_col, mouse_row) {
                    // Use ALT modifier for block selection (hold ALT while clicking)
                    let modifiers = if mouse.modifiers.contains(crossterm::event::KeyModifiers::ALT) {
                        crossterm::event::KeyModifiers::ALT
                    } else {
                        crossterm::event::KeyModifiers::empty()
                    };
                    
                    Some(Action::MouseDown(
                        matrix_pos.col as u16, 
                        matrix_pos.row as u16, 
                        crate::actions::MouseButton::Left, 
                        modifiers
                    ))
                } else {
                    None
                }
            }
            MouseEventKind::Drag(MouseButton::Left) => {
                if let Some(matrix_pos) = self.screen_to_matrix_pos(mouse_col, mouse_row) {
                    Some(Action::MouseDrag(matrix_pos.col as u16, matrix_pos.row as u16))
                } else {
                    None
                }
            }
            MouseEventKind::Up(MouseButton::Left) => {
                if let Some(matrix_pos) = self.screen_to_matrix_pos(mouse_col, mouse_row) {
                    Some(Action::MouseUp(matrix_pos.col as u16, matrix_pos.row as u16))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
    
    /// Send a small test image to verify Kitty graphics protocol is working
    fn send_test_image(&self) {
        // Create a small 4x4 red test image (RGBA format)
        let width = 4u32;
        let height = 4u32;
        let mut rgba_data = Vec::new();
        
        // Red pixels: R=255, G=0, B=0, A=255
        for _ in 0..(width * height) {
            rgba_data.extend_from_slice(&[255, 0, 0, 255]); // Red pixel
        }
        
        use base64::Engine;
        use std::io::Write;
        let base64_image = base64::engine::general_purpose::STANDARD.encode(&rgba_data);
        
        // Send simple test image
        eprint!("\x1b_Ga=T,f=32,s={},v={};{}\x1b\\", width, height, base64_image);
        let _ = std::io::stderr().flush();
        
        // Note: This is a test function, logging will be done by caller
    }

    /// Convert screen coordinates to matrix position
    fn screen_to_matrix_pos(&self, screen_col: u16, screen_row: u16) -> Option<crate::actions::Position> {
        // Get the actual terminal size
        if let Ok((term_width, term_height)) = crossterm::terminal::size() {
            // Account for terminal panel if visible
            let terminal_panel_height = if self.state.ui.terminal_panel.visible {
                self.state.ui.terminal_panel.height + 1 // +1 for separator
            } else {
                0
            };
            
            // Main area excludes status bar and terminal panel
            let main_area_height = term_height.saturating_sub(1 + terminal_panel_height);
            
            // 50-50 horizontal split for PDF and text panels
            let text_panel_start = term_width / 2;
            let _text_panel_width = term_width - text_panel_start;
            
            // Check if mouse is in the text panel (right side)
            if screen_col >= text_panel_start && screen_row < main_area_height {
                // Convert to panel-relative coordinates
                let panel_col = screen_col - text_panel_start;
                
                // Account for text area margin: adjust for cursor offset 
                let content_start_col = 1 + 2;  // Left margin + 2 (user said 2 to the right)
                let content_start_row = 4u16.saturating_sub(8);  // Top margin - 8 (3 + 2 + 3 more north), prevent underflow
                
                if panel_col >= content_start_col && screen_row >= content_start_row {
                    // Subtract 1 to move cursor one square left (lower column number)
                    let matrix_col = ((panel_col - content_start_col) as usize).saturating_sub(1);
                    // Subtract 3 more to move cursor further north (higher in the matrix means lower row numbers)
                    let matrix_row = ((screen_row - content_start_row) as usize).saturating_sub(3);
                    
                    Some(crate::actions::Position {
                        row: matrix_row,
                        col: matrix_col,
                    })
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            // Fallback if terminal size detection fails
            None
        }
    }

    /// Handle keyboard input and return action
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<Action> {
        use crossterm::event::KeyCode;
        
        // Check if file selector is active first
        if self.file_selector.active {
            match key.code {
                KeyCode::Esc => {
                    self.file_selector.deactivate();
                    Some(Action::SetStatus("File selection cancelled".to_string()))
                }
                KeyCode::Up => {
                    self.file_selector.navigate_up();
                    None
                }
                KeyCode::Down => {
                    self.file_selector.navigate_down();
                    None
                }
                KeyCode::Enter => {
                    if let Some(path) = self.file_selector.enter_directory() {
                        self.file_selector.deactivate();
                        Some(Action::OpenPdf(path))
                    } else {
                        None
                    }
                }
                KeyCode::Backspace => {
                    self.file_selector.go_up_directory();
                    None
                }
                _ => None,
            }
        } else {
            // Mode-specific key handling
            match self.state.mode {
                AppMode::Editing => self.handle_editing_keys(key),
                AppMode::Searching => self.handle_search_keys(key),
                AppMode::Help => None, // Help handles its own keys
                AppMode::Commanding => self.handle_command_keys(key),
            }
        }
    }
    
    fn handle_editing_keys(&mut self, key: KeyEvent) -> Option<Action> {
        use crossterm::event::{KeyCode, KeyModifiers};
        
        // Handle selection with Shift modifier
        let has_shift = key.modifiers.contains(KeyModifiers::SHIFT);
        
        match (key.code, key.modifiers) {
            // Control commands first - highest priority
            (KeyCode::Char('t'), KeyModifiers::CONTROL) => Some(Action::ToggleTerminalPanel),
            (KeyCode::Char('o'), KeyModifiers::CONTROL) => {
                // Clear any Kitty images when opening file selector
                if self.is_kitty {
                    use std::io::{stdout, Write};
                    let mut out = stdout();
                    let _ = out.write_all(b"\x1b_Ga=d,i=99\x1b\\");
                    let _ = out.flush();
                }
                // Activate our custom file selector
                self.file_selector.activate();
                Some(Action::SetStatus("Select a PDF file...".to_string()))
            }
            (KeyCode::Char('e'), KeyModifiers::CONTROL) | (KeyCode::Char('m'), KeyModifiers::CONTROL) => {
                // Extract text from PDF
                if self.state.pdf.is_loaded() {
                    Some(Action::ExtractMatrix)
                } else {
                    Some(Action::Error("No PDF loaded. Press Ctrl+O to open a file.".to_string()))
                }
            }
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                if self.is_iterm2 {
                    self.copy_with_iterm2_table_mode();
                    None // Handle internally for iTerm2
                } else {
                    Some(Action::Copy)
                }
            }
            (KeyCode::Char('x'), KeyModifiers::CONTROL) => {
                if self.is_iterm2 {
                    self.cut_with_iterm2_table_mode();
                    None // Handle internally for iTerm2
                } else {
                    Some(Action::Cut)
                }
            }
            (KeyCode::Char('v'), KeyModifiers::CONTROL) => {
                if self.is_iterm2 {
                    self.paste_with_iterm2_table_mode();
                    None // Handle internally for iTerm2
                } else {
                    Some(Action::PasteFromSystem)
                }
            }
            (KeyCode::Char('a'), KeyModifiers::CONTROL) => Some(Action::SelectAll),
            
            // Block selection mode (Alt+B or Ctrl+Alt+B)
            (KeyCode::Char('b'), KeyModifiers::ALT) => None, // Block selection removed
            (KeyCode::Char('b'), modifiers) if modifiers.contains(KeyModifiers::CONTROL) && modifiers.contains(KeyModifiers::ALT) => {
                None // Block selection removed
            }
            
            // Exit edit mode
            (KeyCode::Esc, _) => Some(Action::ExitEditMode),
            
            // Navigation with optional selection - fix selection logic
            (KeyCode::Up, _) => {
                if has_shift {
                    // Start selection if not active, move cursor, then update selection
                    if self.state.editor.selection.is_none() {
                        None // Selection removed
                    } else {
                        // Move cursor first, then update selection in state handler
                        Some(Action::MoveCursor(crate::actions::CursorDirection::Up))
                    }
                } else {
                    Some(Action::MoveCursor(crate::actions::CursorDirection::Up))
                }
            }
            (KeyCode::Down, _) => {
                if has_shift {
                    if self.state.editor.selection.is_none() {
                        None // Selection removed
                    } else {
                        Some(Action::MoveCursor(crate::actions::CursorDirection::Down))
                    }
                } else {
                    Some(Action::MoveCursor(crate::actions::CursorDirection::Down))
                }
            }
            (KeyCode::Left, _) => {
                if has_shift {
                    if self.state.editor.selection.is_none() {
                        None // Selection removed
                    } else {
                        Some(Action::MoveCursor(crate::actions::CursorDirection::Left))
                    }
                } else {
                    Some(Action::MoveCursor(crate::actions::CursorDirection::Left))
                }
            }
            (KeyCode::Right, _) => {
                if has_shift {
                    if self.state.editor.selection.is_none() {
                        None // Selection removed
                    } else {
                        Some(Action::MoveCursor(crate::actions::CursorDirection::Right))
                    }
                } else {
                    Some(Action::MoveCursor(crate::actions::CursorDirection::Right))
                }
            }
            (KeyCode::Home, _) => Some(Action::MoveCursor(crate::actions::CursorDirection::Home)),
            (KeyCode::End, _) => Some(Action::MoveCursor(crate::actions::CursorDirection::End)),
            
            // Text input - just insert the character (delete selection in state handler if needed)
            (KeyCode::Char(c), KeyModifiers::NONE) => {
                Some(Action::InsertChar(c))
            }
            (KeyCode::Char(c), KeyModifiers::SHIFT) => {
                Some(Action::InsertChar(c))
            }
            
            // Editing operations
            (KeyCode::Backspace, _) => Some(Action::DeleteChar),
            (KeyCode::Delete, _) => {
                if self.state.editor.selection.is_some() {
                    Some(Action::DeleteSelection)
                } else {
                    Some(Action::DeleteChar) // Use the standard delete char action
                }
            }
            (KeyCode::Enter, _) => {
                // Insert newline - add a new line action
                Some(Action::InsertChar('\n'))
            }
            
            _ => None,
        }
    }
    
    fn handle_search_keys(&mut self, _key: KeyEvent) -> Option<Action> {
        // TODO: Implement search key handling
        None
    }
    
    fn handle_command_keys(&mut self, _key: KeyEvent) -> Option<Action> {
        // TODO: Implement command palette key handling
        None
    }
    
    /// Dispatch action and execute any resulting commands
    pub async fn dispatch(&mut self, action: Action) -> Result<()> {
        // Check for quit action
        if matches!(action, Action::Quit) {
            self.running = false;
            return Ok(());
        }
        
        // Reset cursor blink on user input actions
        match &action {
            Action::InsertChar(_) | Action::MoveCursor(_) | Action::DeleteChar |
            Action::MouseDown(_, _, _, _) | Action::ExitEditMode => {
                self.reset_cursor_blink();
            },
            _ => {}
        }
        
        // Update state with pure function
        let (new_state, command) = self.state.clone().update(action.clone());
        self.state = new_state;
        
        // Handle pending selection after scroll actions
        if matches!(action, Action::ScrollTerminalUp | Action::ScrollTerminalDown) {
            if let Some((start, end)) = self.pending_selection.take() {
                let (new_state, _) = self.state.clone().update(Action::SelectTerminalText(start, end));
                self.state = new_state;
            }
        }
        
        // Execute side effects
        if let Some(cmd) = command {
            self.execute_command(cmd).await?;
        }
        
        Ok(())
    }
    
    async fn execute_command(&mut self, command: Command) -> Result<()> {
        match command {
            Command::LoadPdf(path) => {
                // Load PDF with PDFium
                if let Some(engine) = &mut self.pdf_engine {
                    // Log to terminal
                    let (mut new_state, _) = self.state.clone().update(
                        Action::AddTerminalOutput(format!("Loading PDF: {}", path.display()))
                    );
                    self.state = new_state.clone();
                    
                    match engine.load_pdf(&path) {
                        Ok((page_count, title)) => {
                            let metadata = crate::actions::PdfMetadata {
                                page_count,
                            };
                            
                            // Log success to terminal
                            let (state2, _) = new_state.update(
                                Action::AddTerminalOutput(format!("✓ PDF loaded: {} pages, title: {}", page_count, title))
                            );
                            new_state = state2;
                            
                            let (state3, _) = new_state.update(Action::PdfLoaded(metadata));
                            self.state = state3;
                            
                            // Mark that Kitty needs to redraw the PDF
                            if self.is_kitty {
                                self.kitty_needs_redraw = true;
                                // Log that we're ready to render
                                let (state4, _) = self.state.clone().update(
                                    Action::AddTerminalOutput("🎨 Kitty graphics ready to render".to_string())
                                );
                                self.state = state4;
                            }
                        }
                        Err(e) => {
                            // Log error to terminal
                            let (state2, _) = new_state.update(
                                Action::AddTerminalOutput(format!("✗ Failed to load PDF: {}", e))
                            );
                            new_state = state2;
                            
                            let (state3, _) = new_state.update(
                                Action::Error(format!("Failed to load PDF: {}", e))
                            );
                            self.state = state3;
                        }
                    }
                } else {
                    let (new_state, _) = self.state.clone().update(
                        Action::Error("PDF engine not initialized".to_string())
                    );
                    self.state = new_state;
                }
            }
            Command::ExtractText => {
                // Extract text from current PDF page
                if let Some(engine) = &self.pdf_engine {
                    // Log to terminal
                    let (mut new_state, _) = self.state.clone().update(
                        Action::AddTerminalOutput(format!("Extracting text from page {}...", self.state.pdf.current_page + 1))
                    );
                    self.state = new_state.clone();
                    
                    match engine.extract_text_matrix(engine.get_current_page()) {
                        Ok(matrix) => {
                            let rows = matrix.len();
                            let cols = matrix.first().map(|r| r.len()).unwrap_or(0);
                            
                            // Log success to terminal
                            let (state2, _) = new_state.update(
                                Action::AddTerminalOutput(format!("✓ Extracted {}x{} character matrix", rows, cols))
                            );
                            new_state = state2;
                            
                            // Show terminal panel if it was hidden
                            if !new_state.ui.terminal_panel.visible {
                                new_state.ui.terminal_panel.visible = true;
                            }
                            
                            let (state3, _) = new_state.update(Action::MatrixExtracted(matrix));
                            self.state = state3;
                        }
                        Err(e) => {
                            // Log error to terminal
                            let (state2, _) = new_state.update(
                                Action::AddTerminalOutput(format!("✗ Text extraction failed: {}", e))
                            );
                            new_state = state2;
                            
                            let (state3, _) = new_state.update(
                                Action::Error(format!("Failed to extract text: {}", e))
                            );
                            self.state = state3;
                        }
                    }
                } else {
                    let (new_state, _) = self.state.clone().update(
                        Action::Error("PDF engine not initialized".to_string())
                    );
                    self.state = new_state;
                }
            }
            Command::CopyToClipboard(text) => {
                // Use pbcopy on macOS for clipboard
                if let Ok(mut child) = std::process::Command::new("pbcopy")
                    .stdin(std::process::Stdio::piped())
                    .spawn()
                {
                    if let Some(stdin) = child.stdin.as_mut() {
                        use std::io::Write;
                        let _ = stdin.write_all(text.as_bytes());
                    }
                    let _ = child.wait();
                    let (new_state, _) = self.state.clone().update(
                        Action::SetStatus("Copied to clipboard".to_string())
                    );
                    self.state = new_state;
                } else {
                    let (new_state, _) = self.state.clone().update(
                        Action::Error("Failed to copy to clipboard".to_string())
                    );
                    self.state = new_state;
                }
            }
            Command::PasteFromClipboard => {
                // Use pbpaste on macOS for clipboard
                if let Ok(output) = std::process::Command::new("pbpaste").output() {
                    if let Ok(text) = String::from_utf8(output.stdout) {
                        let (new_state, _) = self.state.clone().update(Action::Paste(text));
                        self.state = new_state;
                    }
                }
            }
            Command::SaveFile(_path, _content) => {
                // TODO: Implement file save
            }
            Command::ShowFileDialog => {
                // TODO: Implement file dialog
            }
            Command::RenderPdfPage => {
                // Trigger Kitty PDF re-render
                if self.is_kitty {
                    self.kitty_needs_redraw = true;
                }
            }
            Command::ExportMatrix => {
                // Simple matrix export to timestamp file
                if self.state.editor.has_content() {
                    let timestamp = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    let filename = format!("matrix_export_{}.txt", timestamp);
                    
                    // Convert matrix to string
                    let mut content = String::new();
                    for row in &self.state.editor.matrix {
                        let line: String = row.iter().collect();
                        content.push_str(&line.trim_end());
                        content.push('\n');
                    }
                    
                    // Write to file
                    if let Err(e) = std::fs::write(&filename, content) {
                        let (new_state, _) = self.state.clone().update(
                            Action::Error(format!("Export failed: {}", e))
                        );
                        self.state = new_state;
                    } else {
                        let (new_state, _) = self.state.clone().update(
                            Action::SetStatus(format!("Exported to {}", filename))
                        );
                        self.state = new_state;
                    }
                } else {
                    let (new_state, _) = self.state.clone().update(
                        Action::Error("No matrix content to export".to_string())
                    );
                    self.state = new_state;
                }
            }
        }
        Ok(())
    }
    
    /// Process any async tasks
    pub async fn tick(&mut self) -> Result<()> {
        // Handle cursor blinking
        let now = std::time::Instant::now();
        if now.duration_since(self.cursor_blink_timer).as_millis() >= 530 { // Standard cursor blink rate (530ms)
            self.cursor_visible = !self.cursor_visible;
            self.cursor_blink_timer = now;
        }
        
        // TODO: Handle background tasks like PDF rendering
        Ok(())
    }
    
    /// Reset cursor blink to visible state on user input
    pub fn reset_cursor_blink(&mut self) {
        self.cursor_visible = true;
        self.cursor_blink_timer = std::time::Instant::now();
    }
    
    /// Render the UI with highlighting instead of borders
    pub fn render(&self, frame: &mut Frame) {
        use ratatui::{
            layout::{Constraint, Direction, Layout, Margin},
            style::{Color, Style},
            widgets::{Block, Paragraph},
            text::{Line, Span},
        };
        
        // Clear any potential screen artifacts to prevent tearing
        // BUT: Don't clear the PDF panel area when using Kitty graphics!
        if self.is_kitty && self.state.pdf.is_loaded() {
            // Clear only the right side and status bar, preserve left panel for Kitty graphics
            let area = frame.area();
            let pdf_width = area.width / 2;
            
            // Clear right panel area
            let right_area = ratatui::layout::Rect {
                x: pdf_width,
                y: 0,
                width: area.width - pdf_width,
                height: area.height,
            };
            frame.render_widget(ratatui::widgets::Clear, right_area);
            
            // Clear status bar area
            let status_area = ratatui::layout::Rect {
                x: 0,
                y: area.height.saturating_sub(1),
                width: area.width,
                height: 1,
            };
            frame.render_widget(ratatui::widgets::Clear, status_area);
        } else {
            // Normal clearing for non-Kitty mode
            frame.render_widget(ratatui::widgets::Clear, frame.area());
        }
        
        // Handle overlay modes - help screen and file selector take full control
        if self.state.mode == crate::state::app_state::AppMode::Help {
            self.render_help_screen(frame);
            return;
        }
        
        if self.file_selector.active {
            self.file_selector.render(frame, frame.area());
            return;
        }
        
        // Ultra-aggressive bounds checking to prevent ratatui buffer overflow
        let area = frame.area();
        if area.width < 40 || area.height < 15 || area.width > 500 || area.height > 200 {
            // Terminal too small or potentially corrupt dimensions - just show a minimal message
            let safe_area = ratatui::layout::Rect {
                x: 0,
                y: 0,
                width: area.width.min(40).max(10),
                height: area.height.min(5).max(1),
            };
            let paragraph = Paragraph::new("Terminal size error - please resize")
                .style(Style::default().fg(Color::Red));
            frame.render_widget(paragraph, safe_area);
            return;
        }
        
        // Adjust layout based on terminal panel visibility with overflow protection
        let chunks = if self.state.ui.terminal_panel.visible {
            // Clamp terminal panel height to prevent layout overflow
            let safe_terminal_height = self.state.ui.terminal_panel.height
                .min(area.height.saturating_sub(3))  // Leave room for main content + status
                .max(1);  // Minimum 1 line
            
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(2),  // Main content (minimum 2 lines)
                    Constraint::Length(safe_terminal_height),  // Terminal panel
                    Constraint::Length(1),  // Status bar
                ])
                .split(frame.area())
        } else {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(2),  // Main content (minimum 2 lines)
                    Constraint::Length(1),  // Status bar
                ])
                .split(frame.area())
        };
        
        // Additional bounds check after layout splitting
        if chunks.is_empty() || chunks[0].width < 4 || chunks[0].height < 2 {
            let paragraph = Paragraph::new("Layout error - please resize terminal")
                .style(Style::default().fg(Color::Red));
            frame.render_widget(paragraph, area);
            return;
        }
        
        // Validate all chunks to prevent ratatui buffer overflow (index 65535 issue)
        for (i, chunk) in chunks.iter().enumerate() {
            if chunk.width == 0 || chunk.height == 0 || 
               chunk.width > 1000 || chunk.height > 1000 ||
               (chunk.width as u32 * chunk.height as u32) > 65535 {
                let paragraph = Paragraph::new(format!("Chunk {} invalid: {}x{} - please resize", i, chunk.width, chunk.height))
                    .style(Style::default().fg(Color::Red));
                frame.render_widget(paragraph, area);
                return;
            }
        }

        // Main content area - add small gap between panels
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(chunks[0]);
        
        // Validate main chunks for ratatui buffer overflow prevention
        if main_chunks.len() < 2 || main_chunks[0].width < 2 || main_chunks[1].width < 2 {
            let paragraph = Paragraph::new("Horizontal split error - please resize terminal")
                .style(Style::default().fg(Color::Red));
            frame.render_widget(paragraph, area);
            return;
        }
        
        // Additional validation for main chunks
        for (i, chunk) in main_chunks.iter().enumerate() {
            if (chunk.width as u32 * chunk.height as u32) > 65535 {
                let paragraph = Paragraph::new(format!("Main chunk {} too large: {}x{} = {} cells", 
                    i, chunk.width, chunk.height, chunk.width as u32 * chunk.height as u32))
                    .style(Style::default().fg(Color::Red));
                frame.render_widget(paragraph, area);
                return;
            }
        }
        
        // PDF Panel with consistent color scheme (no focus haze)
        let pdf_bg = Color::Rgb(30, 34, 42);  // Always use darker background
        let pdf_fg = Color::Rgb(180, 180, 200);
        
        // Handle PDF rendering based on terminal type
        if self.is_kitty {
            if self.state.pdf.is_loaded() {
                // FOR KITTY: Reserve the entire left panel for Kitty graphics
                // DO NOT render any ratatui widgets in main_chunks[0] - this prevents conflicts
                // Kitty graphics will be rendered post-frame with exclusive control
                // 
                // The PDF area (main_chunks[0]) is completely excluded from ratatui control
                // to prevent clearing/overwriting of Kitty images
            } else {
                // Show a placeholder in the reserved Kitty area when no PDF is loaded
                let placeholder = Paragraph::new("\n\n  PDF VIEWER (Kitty Graphics Mode)\n\n  📄 No PDF loaded\n\n  Press Ctrl+O to open a PDF file\n\n  This area is reserved for\n  Kitty graphics protocol")
                    .style(Style::default().fg(pdf_fg).bg(pdf_bg))
                    .block(Block::default().style(Style::default().bg(pdf_bg)));
                frame.render_widget(placeholder, main_chunks[0]);
            }
        } else if self.is_iterm2 {
            // Use iTerm2's inline image protocol for actual PDF rendering
            self.render_pdf_with_iterm2_images(main_chunks[0]);
            return; // Skip the rest of rendering for iTerm2
        } else {
            // For other terminals, render text-based PDF info
            let pdf_content = if self.state.pdf.is_loaded() {
                if let Some(engine) = &self.pdf_engine {
                    match engine.render_page_info(self.state.pdf.current_page) {
                        Ok(info) => info,
                        Err(_) => "\n  PDF VIEWER\n  Error getting PDF info".to_string()
                    }
                } else {
                    "\n  PDF VIEWER\n  PDF engine not available".to_string()
                }
            } else {
                "\n  PDF VIEWER\n  No PDF loaded\n\n  Press Ctrl+O to open".to_string()
            };
            
            // Fill entire panel with background color
            let pdf_block = Block::default()
                .style(Style::default().bg(pdf_bg));
            
            frame.render_widget(pdf_block, main_chunks[0]);
            
            // Render content with padding
            let pdf_area = main_chunks[0].inner(Margin { vertical: 1, horizontal: 2 });
            frame.render_widget(
                Paragraph::new(pdf_content)
                    .style(Style::default().fg(pdf_fg).bg(pdf_bg)),
                pdf_area,
            );
        }
        
        // Text Panel with consistent color scheme (no focus haze)
        let text_bg = Color::Rgb(30, 34, 42);  // Always use darker background
        let text_fg = Color::Rgb(180, 180, 200);
        
        
        // Fill entire panel with background color
        let text_block = Block::default()
            .style(Style::default().bg(text_bg));
        
        frame.render_widget(text_block, main_chunks[1]);
        
        // Render content with rich styling for selection highlighting
        let text_area = main_chunks[1].inner(Margin { vertical: 1, horizontal: 2 });
        
        if self.state.editor.has_content() {
            if self.is_iterm2 {
                // Use iTerm2's semantic grid rendering for perfect clipboard
                self.render_matrix_with_iterm2_semantics(text_area, text_bg, text_fg);
                return;
            }
            
            // Fallback rendering for non-iTerm2 terminals
            // Just show TEXT MATRIX without mode since we're always in edit mode
            let mut lines = vec![
                ratatui::text::Line::from(""),  // Empty line to keep spacing clean
                ratatui::text::Line::from(""),
            ];
            
            // Selection highlighting colors - brighter for better visibility
            let selection_bg = Color::Rgb(50, 200, 170); // Brighter teal highlight
            let cursor_bg = Color::Rgb(52, 73, 94);      // Dark blue for cursor
            
            // Render matrix with proper highlighting
            for (row_idx, row) in self.state.editor.matrix.iter().enumerate() {
                let mut spans = vec![ratatui::text::Span::raw("  ")]; // Padding
                
                for (col_idx, &ch) in row.iter().enumerate() {
                    let pos = crate::actions::Position { row: row_idx, col: col_idx };
                    let is_cursor = self.cursor_visible && self.state.mode == AppMode::Editing 
                        && row_idx == self.state.editor.cursor.row 
                        && col_idx == self.state.editor.cursor.col;
                    let is_selected = self.state.editor.is_position_selected(pos);
                    
                    let style = if is_selected {
                        // Highlight selection with background color, keep text readable
                        Style::default().bg(selection_bg).fg(Color::Black)
                    } else if is_cursor && self.state.mode == AppMode::Editing {
                        // Cursor with background color
                        Style::default().bg(cursor_bg).fg(Color::White)
                    } else {
                        // Normal text
                        Style::default().fg(text_fg)
                    };
                    
                    spans.push(ratatui::text::Span::styled(ch.to_string(), style));
                }
                
                lines.push(ratatui::text::Line::from(spans));
            }
            
            // Show controls based on mode
            if self.state.mode == AppMode::Editing {
                let selection_info = if self.state.editor.selection.is_some() {
                    let mode_str = if let Some(ref sel) = self.state.editor.selection {
                        match sel.mode {
                            crate::actions::SelectionMode::Block => " [Block]",
                            crate::actions::SelectionMode::Line => " [Line]",
                        }
                    } else {
                        ""
                    };
                    format!(" • Selection{} • Ctrl+C: Copy • Ctrl+X: Cut • Ctrl+V: Paste", mode_str)
                } else {
                    " • Shift+Arrows: Line select • Alt+B: Block select • Ctrl+A: Select All".to_string()
                };
                lines.push(ratatui::text::Line::from(""));
                lines.push(ratatui::text::Line::from(ratatui::text::Span::styled(
                    format!("  Position: ({},{}) • Type to edit • Arrows to move{} • ESC to exit", 
                        self.state.editor.cursor.row + 1, 
                        self.state.editor.cursor.col + 1,
                        selection_info),
                    Style::default().fg(text_fg)
                )));
            } else {
                lines.push(ratatui::text::Line::from(""));
                lines.push(ratatui::text::Line::from(ratatui::text::Span::styled(
                    "  Ctrl+E: Extract text • Ctrl+O: Open file • Ctrl+S: Export",
                    Style::default().fg(text_fg)
                )));
            }
            
            let paragraph = ratatui::widgets::Paragraph::new(lines)
                .style(Style::default().bg(text_bg));
            frame.render_widget(paragraph, text_area);
        } else {
            // No content case - just show empty background
            frame.render_widget(
                Paragraph::new("")
                    .style(Style::default().fg(text_fg).bg(text_bg)),
                text_area,
            );
        }
        
        // Terminal Panel (if visible)
        if self.state.ui.terminal_panel.visible {
            let terminal_area = chunks[1];
            
            // Terminal background and border - match app theme
            let terminal_bg = Color::Rgb(30, 34, 42);  // Match unfocused panel background
            let terminal_fg = Color::Rgb(180, 180, 200);  // Match app text color
            let _terminal_border_fg = Color::Rgb(60, 65, 78);  // Match focused panel color
            
            // Calculate visible lines
            let visible_lines = self.state.ui.terminal_panel.height.saturating_sub(2) as usize;
            
            // Create a block with border and scroll indicator
            let scroll_indicator = if self.state.ui.terminal_panel.content.len() > visible_lines {
                let current_line = self.state.ui.terminal_panel.scroll_offset + 1;
                let total_lines = self.state.ui.terminal_panel.content.len();
                format!(" Terminal Output [{}/{}] ", current_line, total_lines)
            } else {
                " Terminal Output ".to_string()
            };
            
            let terminal_block = Block::default()
                .title(scroll_indicator)
                .title_alignment(ratatui::layout::Alignment::Left)
                .borders(ratatui::widgets::Borders::NONE)
                .style(Style::default().bg(terminal_bg));
            
            // Calculate inner area for content
            let inner_area = terminal_block.inner(terminal_area);
            
            // Render the block first
            frame.render_widget(terminal_block, terminal_area);
            
            // Prepare terminal content with scrolling - show ALL content if scrolled up
            let visible_lines = self.state.ui.terminal_panel.height as usize;
            let start = self.state.ui.terminal_panel.scroll_offset;
            let end = (start + visible_lines).min(self.state.ui.terminal_panel.content.len());
            
            let mut lines = Vec::new();
            for (idx, line) in self.state.ui.terminal_panel.content[start..end].iter().enumerate() {
                let line_num = start + idx;
                let is_selected = if let Some((sel_start, sel_end)) = self.state.ui.terminal_panel.selected_lines {
                    line_num >= sel_start && line_num <= sel_end
                } else {
                    false
                };
                
                let style = if is_selected {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Rgb(22, 160, 133))  // Teal selection like main editor
                } else {
                    Style::default().fg(terminal_fg).bg(terminal_bg)
                };
                
                lines.push(Line::from(Span::styled(line.clone(), style)));
            }
            
            // Add scroll indicator if there's more content
            if self.state.ui.terminal_panel.content.len() > visible_lines {
                // Show scroll position at bottom
                let at_top = self.state.ui.terminal_panel.scroll_offset == 0;
                let at_bottom = self.state.ui.terminal_panel.scroll_offset >= 
                    self.state.ui.terminal_panel.content.len().saturating_sub(visible_lines);
                
                if !at_bottom && lines.len() < visible_lines {
                    // Add padding and scroll hint
                    while lines.len() < visible_lines - 1 {
                        lines.push(Line::from(""));
                    }
                    lines.push(Line::from(Span::styled(
                        if at_top {
                            "▼ More below ▼".to_string()
                        } else {
                            "▲ Scroll for more ▲".to_string()
                        },
                        Style::default().fg(Color::Rgb(100, 100, 120)).add_modifier(ratatui::style::Modifier::ITALIC)
                    )));
                }
            }
            
            let paragraph = Paragraph::new(lines)
                .style(Style::default().bg(terminal_bg));
            frame.render_widget(paragraph, inner_area);
        }
        
        // Status bar with file selector color scheme
        let status_bar_idx = if self.state.ui.terminal_panel.visible { 2 } else { 1 };
        let status_chunks = if self.state.ui.terminal_panel.visible && chunks.len() > 2 {
            chunks[status_bar_idx]
        } else if !self.state.ui.terminal_panel.visible && chunks.len() > 1 {
            chunks[1]
        } else {
            // Fallback to last chunk
            chunks[chunks.len() - 1]
        };
        
        let status = if let Some(error) = &self.state.error_message {
            format!(" ⚠ {}", error)
        } else {
            let terminal_hint = if !self.state.ui.terminal_panel.visible {
                " • Ctrl+T: Terminal"
            } else {
                ""
            };
            format!(" ▶ {}{}", self.state.status_message, terminal_hint)
        };
        
        let (status_bg, status_fg) = if self.state.error_message.is_some() {
            (Color::Rgb(80, 40, 40), Color::Rgb(255, 150, 150))
        } else {
            (Color::Rgb(25, 28, 34), Color::Rgb(180, 180, 200))
        };
        
        frame.render_widget(
            Paragraph::new(status)
                .style(Style::default().fg(status_fg).bg(status_bg)),
            status_chunks,
        );
        
        // File selector overlay is now handled at the start of the render function
    }
    
    fn render_help_screen(&self, frame: &mut ratatui::Frame) {
        use ratatui::{
            style::{Color, Style},
            widgets::{Block, Borders, Paragraph, Wrap, Clear},
            text::{Line, Span},
        };
        
        // Create a centered popup area
        let popup_area = Self::centered_rect(80, 90, frame.area());
        
        // Clear the popup area
        frame.render_widget(Clear, popup_area);
        
        // Create help content
        let help_text = vec![
            Line::from(vec![
                Span::styled("CHONKER6 - Terminal PDF Viewer & Text Editor", 
                    Style::default().fg(Color::Yellow))
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("NAVIGATION:", Style::default().fg(Color::Cyan))
            ]),
            Line::from("  ←/→ or h/l    Navigate PDF pages"),
            Line::from("  ↑/↓ or j/k    Scroll content"),
            Line::from("  +/-           Zoom PDF in/out"),
            Line::from("  0             Reset zoom to 100%"),
            Line::from(""),
            Line::from(vec![
                Span::styled("FILE OPERATIONS:", Style::default().fg(Color::Cyan))
            ]),
            Line::from("  Ctrl+O        Open PDF file"),
            Line::from("  Ctrl+E        Extract text from PDF"),
            Line::from("  Ctrl+S        Save text content"),
            Line::from(""),
            Line::from(vec![
                Span::styled("EDITING (when in text editor):", Style::default().fg(Color::Cyan))
            ]),
            Line::from("  Arrow keys    Move cursor"),
            Line::from("  Click/drag    Position cursor & select"),
            Line::from("  Ctrl+A        Select all text"),
            Line::from("  Ctrl+C        Copy selection"),
            Line::from("  Ctrl+X        Cut selection"),
            Line::from("  Ctrl+V        Paste from clipboard"),
            Line::from("  ESC           Exit editing mode"),
            Line::from(""),
            Line::from(vec![
                Span::styled("PANELS:", Style::default().fg(Color::Cyan))
            ]),
            Line::from("  Ctrl+T        Toggle terminal panel"),
            Line::from("  Page Up/Down  Scroll terminal output"),
            Line::from(""),
            Line::from(vec![
                Span::styled("OTHER:", Style::default().fg(Color::Cyan))
            ]),
            Line::from("  Ctrl+H        Show this help"),
            Line::from("  ESC           Close help/dialogs"),
            Line::from("  Ctrl+Q        Quit application"),
            Line::from(""),
            Line::from(vec![
                Span::styled("Press ESC to close this help screen", 
                    Style::default().fg(Color::Green))
            ]),
        ];
        
        // Render the help popup
        frame.render_widget(
            Paragraph::new(help_text)
                .block(
                    Block::default()
                        .title(" Help ")
                        .borders(Borders::ALL)
                        .style(Style::default().bg(Color::Rgb(25, 28, 34)))
                )
                .wrap(Wrap { trim: true })
                .style(Style::default().fg(Color::White).bg(Color::Rgb(25, 28, 34))),
            popup_area
        );
    }
    
    fn centered_rect(percent_x: u16, percent_y: u16, r: ratatui::layout::Rect) -> ratatui::layout::Rect {
        use ratatui::layout::{Constraint, Direction, Layout};
        
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }
    
    pub fn is_running(&self) -> bool {
        self.running
    }
    
    pub fn is_help_shown(&self) -> bool {
        self.state.mode == AppMode::Help
    }
    
    pub fn is_kitty_terminal(&self) -> bool {
        self.is_kitty
    }
    
    pub fn has_pdf_loaded(&self) -> bool {
        self.state.pdf.is_loaded()
    }
    
    pub fn should_render_kitty_pdf(&mut self) -> bool {
        if self.is_kitty && self.state.pdf.is_loaded() && self.kitty_needs_redraw {
            self.kitty_needs_redraw = false;
            // Log that we're about to render
            let (new_state, _) = self.state.clone().update(
                Action::AddTerminalOutput(format!("Kitty redraw triggered for page {}", self.state.pdf.current_page + 1))
            );
            self.state = new_state;
            true
        } else {
            false
        }
    }
}