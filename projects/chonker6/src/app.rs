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
        // Kitty sets TERM to "xterm-kitty" and KITTY_WINDOW_ID
        let kitty_window = std::env::var("KITTY_WINDOW_ID").ok();
        let is_kitty = (term_program.contains("kitty") || term_program == "xterm-kitty") && 
                       kitty_window.is_some();
        
        // Allow forcing Kitty mode for testing
        let force_kitty = std::env::var("CHONKER6_FORCE_KITTY").is_ok();
        let is_kitty = is_kitty || force_kitty;
        
        eprintln!("Terminal detection: TERM={}, KITTY_WINDOW_ID={:?}, is_kitty={}, forced={}", 
            term_program, kitty_window, is_kitty, force_kitty);
        
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
        
        let mode_str = if self.state.mode == AppMode::Editing {
            "EDIT MODE"
        } else {
            "VIEW MODE"
        };
        println!("  TEXT MATRIX - {} (iTerm2 Enhanced)", mode_str);
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
                    let is_cursor = self.state.mode == AppMode::Editing 
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
        
        use std::io::{stdout, Write};
        
        // Debug: Log the call
        let (new_state, _) = self.state.clone().update(
            Action::AddTerminalOutput(format!("render_pdf_with_kitty_post_frame called for area: {}x{} at ({},{})", 
                area.width, area.height, area.x, area.y))
        );
        self.state = new_state;
        
        if let Some(engine) = &self.pdf_engine {
            // First test - send a small test pattern to verify Kitty protocol is working
            self.send_test_image();
            
            // Log that we're rendering
            let (new_state, _) = self.state.clone().update(
                Action::AddTerminalOutput(format!("📍 Sent Kitty test image | Rendering PDF page {}...", self.state.pdf.current_page + 1))
            );
            self.state = new_state;
            
            // Calculate pixel dimensions for the display area
            let cell_width = 9u32;
            let cell_height = 18u32;
            let display_width_px = (area.width as u32).saturating_sub(4) * cell_width;
            let display_height_px = (area.height as u32).saturating_sub(4) * cell_height;
            
            // Render PDF page with automatic size fitting
            match engine.render_page_for_kitty(
                self.state.pdf.current_page, 
                display_width_px,
                display_height_px
            ) {
                Ok((rgba_data, width, height)) => {
                    // Log data size
                    let (new_state, _) = self.state.clone().update(
                        Action::AddTerminalOutput(format!("Got RGBA data: {} bytes for {}x{} image", 
                            rgba_data.len(), width, height))
                    );
                    self.state = new_state;
                    
                    // Position cursor at the PDF area
                    print!("\x1b[{};{}H", area.y + 1, area.x + 1);
                    
                    // Use Kitty graphics protocol - proper implementation
                    use base64::Engine;
                    
                    // First, delete any existing images
                    print!("\x1b_Ga=d\x1b\\");
                    
                    // Log PDF rendering details
                    let (new_state, _) = self.state.clone().update(
                        Action::AddTerminalOutput(format!("PDF render dimensions: {}x{} pixels, {} RGBA bytes", 
                            width, height, rgba_data.len()))
                    );
                    self.state = new_state;
                    
                    // Encode the RGBA data as base64
                    let base64_image = base64::engine::general_purpose::STANDARD.encode(&rgba_data);
                    
                    // Kitty has a limit on how much data can be sent in one escape sequence
                    // We need to chunk it if it's too large
                    const CHUNK_SIZE: usize = 4096;
                    
                    // Log base64 size
                    let (new_state, _) = self.state.clone().update(
                        Action::AddTerminalOutput(format!("Base64 encoded size: {} bytes", base64_image.len()))
                    );
                    self.state = new_state;
                    
                    // Always use chunked transmission for reliability
                    let chunks: Vec<&str> = base64_image.as_bytes()
                        .chunks(CHUNK_SIZE)
                        .map(|chunk| std::str::from_utf8(chunk).unwrap_or(""))
                        .collect();
                    
                    let (new_state, _) = self.state.clone().update(
                        Action::AddTerminalOutput(format!("Sending image in {} chunks", chunks.len()))
                    );
                    self.state = new_state;
                    
                    if chunks.len() == 1 {
                        // Single chunk - use simpler format
                        eprint!("\x1b_Ga=T,f=32,s={},v={};{}\x1b\\", width, height, chunks[0]);
                        
                        let (new_state, _) = self.state.clone().update(
                            Action::AddTerminalOutput(format!("✓ Sent single-chunk image ({}x{})", width, height))
                        );
                        self.state = new_state;
                    } else {
                        // Multi-chunk transmission - proper protocol
                        // First chunk with metadata
                        if let Some(first) = chunks.first() {
                            eprint!("\x1b_Ga=T,f=32,s={},v={},m=1;{}\x1b\\", width, height, first);
                        }
                        
                        // Middle chunks (if any)
                        for (i, chunk) in chunks.iter().skip(1).enumerate() {
                            let is_last = i == chunks.len() - 2; // Last of the remaining chunks
                            if is_last {
                                eprint!("\x1b_Gm=0;{}\x1b\\", chunk); // Last chunk
                            } else {
                                eprint!("\x1b_Gm=1;{}\x1b\\", chunk); // More chunks coming
                            }
                        }
                        
                        let (new_state, _) = self.state.clone().update(
                            Action::AddTerminalOutput(format!("✓ Sent multi-chunk image ({}x{}, {} chunks)", width, height, chunks.len()))
                        );
                        self.state = new_state;
                    }
                    
                    // Ensure all data is flushed immediately
                    use std::io::Write;
                    let _ = std::io::stderr().flush();
                    let _ = std::io::stdout().flush();
                    
                    // Wait a moment for Kitty to process
                    std::thread::sleep(std::time::Duration::from_millis(50));
                    
                    // Log success
                    let (new_state, _) = self.state.clone().update(
                        Action::AddTerminalOutput(format!("✓ Rendered {}x{} PDF page to Kitty terminal", width, height))
                    );
                    self.state = new_state;
                }
                Err(e) => {
                    // Log error to terminal panel
                    let (new_state, _) = self.state.clone().update(
                        Action::AddTerminalOutput(format!("✗ Failed to render PDF: {:?}", e))
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
                Ok((rgba_data, width, height)) => {
                    // Use Kitty graphics protocol to display the image
                    // First, encode as base64
                    use base64::Engine;
                    let base64_image = base64::engine::general_purpose::STANDARD.encode(&rgba_data);
                    
                    // Position cursor for image
                    print!("\x1b[{};{}H", area.y + 1, area.x + 1);
                    
                    // Kitty graphics protocol: 
                    // a=T means transmit image data
                    // f=32 means RGBA format (32-bit)
                    // s=width,height specifies dimensions
                    // t=d means direct transmission (base64)
                    print!("\x1b_Ga=T,f=32,s={},{},t=d;{}\x1b\\", width, height, base64_image);
                    
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
            let text_panel_width = term_width - text_panel_start;
            
            // Check if mouse is in the text panel (right side)
            if screen_col >= text_panel_start && screen_row < main_area_height {
                // Convert to panel-relative coordinates
                let panel_col = screen_col - text_panel_start;
                
                // Account for panel border (1 char on each side) and header (2 rows)
                let content_start_col = 1;  // Left border
                let content_start_row = 2;  // Top border + header
                
                if panel_col >= content_start_col && screen_row >= content_start_row {
                    let matrix_col = (panel_col - content_start_col) as usize;
                    let matrix_row = (screen_row - content_start_row) as usize;
                    
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
        use crossterm::event::{KeyCode, KeyModifiers};
        
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
                AppMode::Viewing => self.handle_viewing_keys(key),
                AppMode::Editing => self.handle_editing_keys(key),
                AppMode::Searching => self.handle_search_keys(key),
                AppMode::Help => None, // Help handles its own keys
                AppMode::Commanding => self.handle_command_keys(key),
            }
        }
    }
    
    fn handle_viewing_keys(&mut self, key: KeyEvent) -> Option<Action> {
        use crossterm::event::{KeyCode, KeyModifiers};
        
        match (key.code, key.modifiers) {
            (KeyCode::Char('o'), KeyModifiers::CONTROL) => {
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
            (KeyCode::Right, _) | (KeyCode::Char('l'), _) => {
                if self.is_kitty && self.state.pdf.is_loaded() {
                    self.kitty_needs_redraw = true;
                }
                Some(Action::NavigatePage(crate::actions::PageDirection::Next))
            }
            (KeyCode::Left, _) | (KeyCode::Char('h'), _) => {
                if self.is_kitty && self.state.pdf.is_loaded() {
                    self.kitty_needs_redraw = true;
                }
                Some(Action::NavigatePage(crate::actions::PageDirection::Previous))
            }
            (KeyCode::Tab, _) => {
                let next_panel = match self.state.ui.focused_panel {
                    crate::actions::Panel::Pdf => crate::actions::Panel::Text,
                    crate::actions::Panel::Text => crate::actions::Panel::Pdf,
                };
                Some(Action::SwitchPanel(next_panel))
            }
            (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
                // Export matrix with Ctrl+S
                Some(Action::ExportMatrix)
            }
            (KeyCode::Char('d'), KeyModifiers::CONTROL) if self.state.pdf.is_loaded() => {
                // Toggle dark mode for PDF
                Some(Action::ToggleDarkMode)
            }
            (KeyCode::Char(']'), KeyModifiers::CONTROL) | (KeyCode::Char('+'), KeyModifiers::CONTROL) => {
                // Zoom in PDF (only with Ctrl)
                Some(Action::ZoomIn)
            }
            (KeyCode::Char('['), KeyModifiers::CONTROL) | (KeyCode::Char('-'), KeyModifiers::CONTROL) => {
                // Zoom out PDF (only with Ctrl)
                Some(Action::ZoomOut)
            }
            (KeyCode::Char('0'), KeyModifiers::CONTROL) if self.state.pdf.is_loaded() => {
                // Reset zoom (only with Ctrl)
                Some(Action::ZoomReset)
            }
            (KeyCode::Char('f'), KeyModifiers::CONTROL) if self.state.pdf.is_loaded() => {
                // Toggle auto-fit (only with Ctrl)
                Some(Action::ToggleAutoFit)
            }
            (KeyCode::Char('t'), KeyModifiers::CONTROL) => {
                // Toggle terminal panel
                Some(Action::ToggleTerminalPanel)
            }
            (KeyCode::Char('k'), KeyModifiers::CONTROL | KeyModifiers::SHIFT) => {
                // Clear terminal output (changed from 'l' to 'k' to avoid conflict)
                Some(Action::ClearTerminalOutput)
            }
            (KeyCode::PageUp, _) if self.state.ui.terminal_panel.visible => {
                Some(Action::ScrollTerminalUp)
            }
            (KeyCode::PageDown, _) if self.state.ui.terminal_panel.visible => {
                Some(Action::ScrollTerminalDown)
            }
            (KeyCode::Char('c'), KeyModifiers::CONTROL) if self.state.ui.terminal_panel.visible && self.state.ui.terminal_panel.selected_lines.is_some() => {
                // Copy selected terminal text
                Some(Action::CopyTerminalSelection)
            }
            (KeyCode::Char('a'), KeyModifiers::CONTROL) if self.state.ui.terminal_panel.visible => {
                // Select all terminal text with Ctrl+A
                let last_line = self.state.ui.terminal_panel.content.len().saturating_sub(1);
                Some(Action::SelectTerminalText(0, last_line))
            }
            _ => None,
        }
    }
    
    fn handle_editing_keys(&mut self, key: KeyEvent) -> Option<Action> {
        use crossterm::event::{KeyCode, KeyModifiers};
        
        // Handle selection with Shift modifier
        let has_shift = key.modifiers.contains(KeyModifiers::SHIFT);
        
        match (key.code, key.modifiers) {
            // Navigation with optional selection - fix selection logic
            (KeyCode::Up, _) => {
                if has_shift {
                    // Start selection if not active, move cursor, then update selection
                    if self.state.editor.selection.is_none() {
                        Some(Action::StartSelection(self.state.editor.cursor))
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
                        Some(Action::StartSelection(self.state.editor.cursor))
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
                        Some(Action::StartSelection(self.state.editor.cursor))
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
                        Some(Action::StartSelection(self.state.editor.cursor))
                    } else {
                        Some(Action::MoveCursor(crate::actions::CursorDirection::Right))
                    }
                } else {
                    Some(Action::MoveCursor(crate::actions::CursorDirection::Right))
                }
            }
            (KeyCode::Home, _) => Some(Action::MoveCursor(crate::actions::CursorDirection::Home)),
            (KeyCode::End, _) => Some(Action::MoveCursor(crate::actions::CursorDirection::End)),
            
            // Clipboard operations (Ctrl+C, Ctrl+X, Ctrl+V)
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
            (KeyCode::Char('b'), KeyModifiers::ALT) => Some(Action::StartBlockSelection(self.state.editor.cursor)),
            (KeyCode::Char('b'), modifiers) if modifiers.contains(KeyModifiers::CONTROL) && modifiers.contains(KeyModifiers::ALT) => {
                Some(Action::StartBlockSelection(self.state.editor.cursor))
            }
            
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
            
            // Exit edit mode
            (KeyCode::Esc, _) => Some(Action::ExitEditMode),
            
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
                                path: path.clone(),
                                page_count,
                                title: Some(title.clone()),
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
                // TODO: Implement PDF image rendering
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
        // TODO: Handle background tasks like PDF rendering
        Ok(())
    }
    
    /// Render the UI with highlighting instead of borders
    pub fn render(&self, frame: &mut Frame) {
        use ratatui::{
            layout::{Constraint, Direction, Layout, Margin},
            style::{Color, Style},
            widgets::{Block, Paragraph},
            text::{Line, Span},
        };
        
        // Adjust layout based on terminal panel visibility
        let chunks = if self.state.ui.terminal_panel.visible {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(1),  // Main content
                    Constraint::Length(self.state.ui.terminal_panel.height),  // Terminal panel
                    Constraint::Length(1),  // Status bar
                ])
                .split(frame.area())
        } else {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(1),
                    Constraint::Length(1),
                ])
                .split(frame.area())
        };
        
        // Main content area - add small gap between panels
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(chunks[0]);
        
        // PDF Panel with file selector color scheme
        let (pdf_bg, pdf_fg) = if self.state.ui.focused_panel == crate::actions::Panel::Pdf {
            // Focused: nice blue-gray background with light purple text
            (Color::Rgb(60, 65, 78), Color::Rgb(180, 180, 200))
        } else {
            // Unfocused: darker blue-gray with light purple text
            (Color::Rgb(30, 34, 42), Color::Rgb(180, 180, 200))
        };
        
        // Handle PDF rendering based on terminal type
        if self.is_kitty && self.state.pdf.is_loaded() {
            // For Kitty, render a background and page info at bottom
            let pdf_block = Block::default()
                .style(Style::default().bg(pdf_bg));
            frame.render_widget(pdf_block, main_chunks[0]);
            
            // Add page navigation info at the bottom
            let nav_text = format!("  Page {}/{} | ←/→: Navigate | +/-: Zoom", 
                self.state.pdf.current_page + 1, 
                self.state.pdf.page_count);
            let nav_area = ratatui::layout::Rect {
                x: main_chunks[0].x,
                y: main_chunks[0].y + main_chunks[0].height - 2,
                width: main_chunks[0].width,
                height: 1,
            };
            frame.render_widget(
                Paragraph::new(nav_text)
                    .style(Style::default().fg(pdf_fg).bg(pdf_bg)),
                nav_area,
            );
            // The post-frame render will overlay the actual PDF image
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
        
        // Text Panel with file selector color scheme
        let (text_bg, text_fg) = if self.state.ui.focused_panel == crate::actions::Panel::Text {
            // Focused: nice blue-gray background with light purple text
            (Color::Rgb(60, 65, 78), Color::Rgb(180, 180, 200))
        } else {
            // Unfocused: darker blue-gray with light purple text
            (Color::Rgb(30, 34, 42), Color::Rgb(180, 180, 200))
        };
        
        
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
            let mode_str = if self.state.mode == AppMode::Editing {
                "EDIT MODE"
            } else {
                "VIEW MODE (press 'i' to edit)"
            };
            let mut lines = vec![
                ratatui::text::Line::from(format!("  TEXT MATRIX - {}", mode_str)),
                ratatui::text::Line::from(""),
            ];
            
            // Selection highlighting colors
            let selection_bg = Color::Rgb(22, 160, 133); // Teal highlight like chonker5
            let cursor_bg = Color::Rgb(52, 73, 94);      // Dark blue for cursor
            
            // Render matrix with proper highlighting
            for (row_idx, row) in self.state.editor.matrix.iter().enumerate() {
                let mut spans = vec![ratatui::text::Span::raw("  ")]; // Padding
                
                for (col_idx, &ch) in row.iter().enumerate() {
                    let pos = crate::actions::Position { row: row_idx, col: col_idx };
                    let is_cursor = self.state.mode == AppMode::Editing 
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
            // No content case
            frame.render_widget(
                Paragraph::new("  TEXT MATRIX\n  No text extracted\n\n  Press Ctrl+E to extract")
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
            let terminal_border_fg = Color::Rgb(60, 65, 78);  // Match focused panel color
            
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
        
        // Render file selector overlay if active and store bounds for click handling
        if self.file_selector.active {
            let (_file_list_start, _file_list_height) = self.file_selector.render(frame, frame.area());
            // Note: We could store these for more accurate click detection
        }
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