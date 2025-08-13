use std::io::{stdout, Write};
use std::path::PathBuf;
use anyhow::Result;
use crossterm::{
    terminal,
    event::{self, Event, KeyCode, KeyModifiers, MouseButton},
    cursor,
    ExecutableCommand,
};
use pdfium_render::prelude::*;
use rfd::FileDialog;
use copypasta::{ClipboardContext, ClipboardProvider};

use crate::matrix::{SpatialTextMatrix, CharInfo};
use crate::services::pdf_engine::PdfEngine;
use crate::file_selector_matrix::FileSelectorMatrix;

#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    Navigate,  // Moving between pages
    Select,    // Selecting text in matrix
    Edit,      // Editing matrix content
}

pub struct ChonkerMatrix {
    // PDF side
    pdf_engine: Option<PdfEngine>,
    current_page: usize,
    total_pages: usize,
    pdf_path: Option<PathBuf>,
    pdf_zoom: f32,  // Zoom level for PDF rendering
    pdf_dark_mode: bool,  // Dark mode for PDF rendering
    
    // Matrix side
    matrix: SpatialTextMatrix,
    mode: Mode,
    
    // File selector
    file_selector: FileSelectorMatrix,
    
    // Terminal state
    term_width: u16,
    term_height: u16,
    
    // Display settings
    show_help: bool,
    status_message: String,
    
    // Mouse state
    last_mouse_x: u16,
    last_mouse_y: u16,
    
    // Clipboard
    clipboard: Option<ClipboardContext>,
    
    // Selection for cut/copy
    selection_start: Option<(usize, usize)>,
    selection_end: Option<(usize, usize)>,
}

impl ChonkerMatrix {
    pub fn new() -> Result<Self> {
        let (width, height) = terminal::size()?;
        
        // Create matrix with sample text for testing
        let mut matrix = SpatialTextMatrix::new();
        
        // Add sample text at various positions
        let sample_texts = vec![
            (5, 2, "INVOICE"),
            (5, 4, "Date: 2024-08-13"),
            (5, 6, "Bill To:"),
            (5, 7, "John Smith"),
            (5, 8, "123 Main St"),
            (5, 10, "Items:"),
            (5, 12, "Widget A     $123.45"),
            (5, 13, "Gadget B     $67.89"),
            (5, 14, "Service C    $200.00"),
            (5, 16, "Total:       $391.34"),
            (40, 4, "Test Edit Mode"),
            (40, 6, "Use arrow keys"),
            (40, 7, "to move cursor"),
            (40, 9, "Press 's' for"),
            (40, 10, "selection mode"),
            (40, 12, "Press 'e' for"),
            (40, 13, "edit mode"),
        ];
        
        for (x, y, text) in sample_texts {
            for (i, ch) in text.chars().enumerate() {
                matrix.set_char(x + i, y, ch);
            }
        }
        
        // Try to initialize clipboard
        let clipboard = ClipboardContext::new().ok();
        
        Ok(ChonkerMatrix {
            pdf_engine: None,
            current_page: 0,
            total_pages: 0,
            pdf_path: None,
            pdf_zoom: 1.0,  // Start at 100% zoom - fit-to-window is calculated automatically
            pdf_dark_mode: true,  // Start in dark mode by default
            matrix,
            mode: Mode::Edit,  // Start in Edit mode by default
            file_selector: FileSelectorMatrix::new(),
            term_width: width,
            term_height: height,
            show_help: false,
            status_message: String::from("ChonkerMatrix - Cmd+O: open PDF, Edit mode active"),
            last_mouse_x: 0,
            last_mouse_y: 0,
            clipboard,
            selection_start: None,
            selection_end: None,
        })
    }
    
    pub fn run(&mut self) -> Result<()> {
        terminal::enable_raw_mode()?;
        stdout().execute(terminal::EnterAlternateScreen)?;
        stdout().execute(cursor::Hide)?;
        stdout().execute(crossterm::event::EnableMouseCapture)?;
        
        let result = self.run_loop();
        
        // Cleanup
        stdout().execute(crossterm::event::DisableMouseCapture)?;
        stdout().execute(cursor::Show)?;
        stdout().execute(terminal::LeaveAlternateScreen)?;
        terminal::disable_raw_mode()?;
        
        result
    }
    
    fn run_loop(&mut self) -> Result<()> {
        // Initial render
        self.render()?;
        
        loop {
            // Handle input without constant rerendering
            match event::read()? {
                Event::Key(key) => {
                    let mut needs_render = true;
                    
                    // Check for file selector first
                    if self.file_selector.active {
                        self.handle_file_selector_input(key.code, key.modifiers)?;
                    } else {
                        match key.code {
                            KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::SUPER) => {
                                break;
                            }
                            // Don't render for certain keys that don't change anything
                            KeyCode::Null | KeyCode::Esc if self.mode == Mode::Navigate => {
                                needs_render = false;
                            }
                            _ => self.handle_input(key.code, key.modifiers)?,
                        }
                    }
                    
                    // Only render if needed
                    if needs_render {
                        self.render()?;
                    }
                }
                Event::Mouse(mouse) => {
                    // Only render if mouse event needs it
                    if self.handle_mouse_event(mouse)? {
                        self.render()?;
                    }
                }
                Event::Resize(width, height) => {
                    self.term_width = width;
                    self.term_height = height;
                    self.render()?;
                }
                _ => {}
            }
        }
        
        Ok(())
    }
    
    fn render(&mut self) -> Result<()> {
        // If file selector is active, render it instead
        if self.file_selector.active {
            self.file_selector.render(self.term_width, self.term_height)?;
            return Ok(());
        }
        
        // Clear screen
        print!("\x1b[2J\x1b[H");
        
        // Calculate split
        let pdf_width = self.term_width / 2;
        let matrix_width = self.term_width - pdf_width - 1;
        
        // Left side: Render PDF as image (or placeholder)
        self.render_pdf_side(0, 0, pdf_width, self.term_height - 2)?;
        
        // Draw divider
        self.draw_divider(pdf_width)?;
        
        // Right side: Render matrix view
        self.matrix.render_at(
            pdf_width + 1,
            0,
            matrix_width,
            self.term_height - 2
        )?;
        
        // Status line at bottom
        self.render_status()?;
        
        stdout().flush()?;
        Ok(())
    }
    
    fn render_pdf_side(&mut self, x: u16, y: u16, width: u16, height: u16) -> Result<()> {
        if let Some(engine) = &mut self.pdf_engine {
            // Get the fit-to-window zoom and render with dark mode
            let (image_data, actual_zoom) = engine.render_page_fit_to_window_with_mode(
                self.current_page, width, height, self.pdf_zoom, self.pdf_dark_mode
            )?;
            
            // Send image at the calculated display size
            self.send_kitty_image_with_fit(image_data, x, y, width, height, actual_zoom)?;
        } else {
            // No PDF loaded - show placeholder
            self.render_pdf_placeholder(x, y, width, height)?;
        }
        
        Ok(())
    }
    
    fn render_pdf_placeholder(&self, x: u16, y: u16, width: u16, height: u16) -> Result<()> {
        // Draw a box with instructions
        for row in 0..height {
            print!("\x1b[{};{}H", y + row + 1, x + 1);
            if row == 0 || row == height - 1 {
                for _ in 0..width {
                    print!("─");
                }
            } else if row == height / 2 {
                let msg = if self.pdf_path.is_some() {
                    format!("Page {}/{}", self.current_page + 1, self.total_pages)
                } else {
                    "Press 'o' to open PDF".to_string()
                };
                let padding = (width as usize).saturating_sub(msg.len()) / 2;
                print!("{:padding$}{}", "", msg, padding = padding);
            }
        }
        Ok(())
    }
    
    fn send_kitty_image_with_fit(&self, data: Vec<u8>, x: u16, y: u16, width: u16, height: u16, zoom: f32) -> Result<()> {
        // Kitty graphics protocol with proper fit-to-window
        let encoded = base64::encode(&data);
        
        // ALWAYS display at full window size regardless of internal render zoom
        // This ensures the PDF takes up the full space while being rendered sharply
        let display_width = width;
        let display_height = height;
        
        // Split into chunks if needed (Kitty has a max chunk size)
        const CHUNK_SIZE: usize = 4096;
        let chunks: Vec<&str> = encoded
            .as_bytes()
            .chunks(CHUNK_SIZE)
            .map(|chunk| std::str::from_utf8(chunk).unwrap())
            .collect();
        
        if chunks.len() == 1 {
            // Single chunk transmission
            print!("\x1b_Ga=T,f=100,X={},Y={},c={},r={};{}\x1b\\", 
                   x, y, display_width, display_height, chunks[0]);
        } else {
            // Multi-chunk transmission
            for (i, chunk) in chunks.iter().enumerate() {
                if i == 0 {
                    // First chunk
                    print!("\x1b_Ga=T,f=100,X={},Y={},c={},r={},m=1;{}\x1b\\",
                           x, y, display_width, display_height, chunk);
                } else if i == chunks.len() - 1 {
                    // Last chunk
                    print!("\x1b_Gm=0;{}\x1b\\", chunk);
                } else {
                    // Middle chunks
                    print!("\x1b_Gm=1;{}\x1b\\", chunk);
                }
            }
        }
        
        Ok(())
    }
    
    fn send_kitty_image(&self, data: Vec<u8>, x: u16, y: u16, width: u16, height: u16) -> Result<()> {
        // Simple Kitty image display for backwards compatibility
        self.send_kitty_image_with_fit(data, x, y, width, height, 1.0)
    }
    
    fn draw_divider(&self, x: u16) -> Result<()> {
        for y in 0..self.term_height - 2 {
            print!("\x1b[{};{}H│", y + 1, x + 1);
        }
        Ok(())
    }
    
    fn render_status(&self) -> Result<()> {
        let y = self.term_height - 1;
        print!("\x1b[{};1H", y);
        
        // Clear line
        print!("\x1b[K");
        
        // Status message
        let status = if self.term_width > 100 {
            format!(
                " {} | PDF: {} | Cursor: ({},{}) | Cmd+O: Open PDF | PageUp/Down: Navigate",
                self.status_message,
                if self.pdf_path.is_some() { 
                    format!("{}/{}", self.current_page + 1, self.total_pages)
                } else {
                    "None".to_string()
                },
                self.matrix.cursor.0,
                self.matrix.cursor.1
            )
        } else {
            format!(
                " {} | Page {}/{} | ({},{})",
                self.status_message,
                self.current_page + 1,
                self.total_pages,
                self.matrix.cursor.0,
                self.matrix.cursor.1
            )
        };
        
        print!("{}", status);
        Ok(())
    }
    
    fn handle_input(&mut self, code: KeyCode, modifiers: KeyModifiers) -> Result<()> {
        // Always use edit mode behavior - no mode switching
        self.handle_editing(code, modifiers)?;
        Ok(())
    }
    
    fn handle_navigation(&mut self, code: KeyCode, modifiers: KeyModifiers) -> Result<()> {
        match (code, modifiers) {
            // PDF navigation - use PageUp/PageDown only
            (KeyCode::PageUp, _) => {
                if self.current_page > 0 {
                    self.current_page -= 1;
                    self.load_current_page()?;
                }
            }
            (KeyCode::PageDown, _) => {
                if self.current_page < self.total_pages.saturating_sub(1) {
                    self.current_page += 1;
                    self.load_current_page()?;
                }
            }
            
            // Arrow keys in navigation mode - move cursor for consistency
            (KeyCode::Up, _) => self.matrix.move_cursor(0, -1),
            (KeyCode::Down, _) => self.matrix.move_cursor(0, 1),
            (KeyCode::Left, _) => self.matrix.move_cursor(-1, 0),
            (KeyCode::Right, _) => self.matrix.move_cursor(1, 0),
            
            // Mode switching with Tab
            (KeyCode::Tab, _) => {
                self.mode = match self.mode {
                    Mode::Navigate => {
                        self.status_message = "Selection mode - Click/drag or use Space to select".to_string();
                        Mode::Select
                    }
                    Mode::Select => {
                        self.status_message = "Edit mode - Type freely to edit text".to_string();
                        Mode::Edit
                    }
                    Mode::Edit => {
                        self.status_message = "Navigation mode - Use Cmd+O to open PDF".to_string();
                        Mode::Navigate
                    }
                };
            }
            
            // File operations with Cmd+O - native macOS file dialog
            (KeyCode::Char('o'), m) if m.contains(KeyModifiers::SUPER) => {
                self.open_native_file_dialog()?;
            }
            // Also support Ctrl+O as fallback (some terminals map Cmd to Ctrl)
            (KeyCode::Char('o'), m) if m.contains(KeyModifiers::CONTROL) => {
                self.open_native_file_dialog()?;
            }
            
            // View options with Cmd
            (KeyCode::Char('g'), m) if m.contains(KeyModifiers::SUPER) => {
                self.matrix.show_grid = !self.matrix.show_grid;
                self.status_message = format!("Grid {}", if self.matrix.show_grid { "enabled" } else { "disabled" });
            }
            (KeyCode::Char('+'), m) if m.contains(KeyModifiers::SUPER) => {
                self.pdf_zoom = (self.pdf_zoom * 1.1).min(2.0);  // Max 200% zoom, smaller increments
                self.status_message = format!("Zoom: {:.0}%", self.pdf_zoom * 100.0);
            }
            (KeyCode::Char('-'), m) if m.contains(KeyModifiers::SUPER) => {
                self.pdf_zoom = (self.pdf_zoom / 1.1).max(0.1);  // Min 10% zoom, smaller increments
                self.status_message = format!("Zoom: {:.0}%", self.pdf_zoom * 100.0);
            }
            (KeyCode::Char('d'), m) if m.contains(KeyModifiers::SUPER) => {
                self.pdf_dark_mode = !self.pdf_dark_mode;
                self.status_message = format!("PDF {} mode", if self.pdf_dark_mode { "dark" } else { "light" });
            }
            
            _ => {}
        }
        Ok(())
    }
    
    fn open_native_file_dialog(&mut self) -> Result<()> {
        // Open native macOS file dialog
        let file = FileDialog::new()
            .add_filter("PDF files", &["pdf"])
            .add_filter("All files", &["*"])
            .set_title("Select a PDF file to open")
            .pick_file();
        
        if let Some(path) = file {
            self.status_message = format!("Loading PDF: {:?}", path.file_name().unwrap_or_default());
            self.load_pdf_and_extract_text(path)?;
        } else {
            self.status_message = "No file selected".to_string();
        }
        
        Ok(())
    }
    
    fn load_pdf_and_extract_text(&mut self, path: PathBuf) -> Result<()> {
        // Load the PDF
        self.load_pdf(path.clone())?;
        
        // Clear existing matrix text
        self.matrix.clear();
        
        // Extract text from the current page if PDF is loaded
        if let Some(engine) = &mut self.pdf_engine {
            if let Ok(text_data) = engine.extract_text_with_positions(self.current_page) {
                // Add extracted text to the matrix
                for (x, y, ch) in text_data {
                    // Convert PDF coordinates to matrix grid coordinates
                    // PDF coordinates are in points, we need to scale to terminal characters
                    let grid_x = (x / 8.0) as usize;  // Approx 8 points per character width
                    let grid_y = (y / 12.0) as usize; // Approx 12 points per line height
                    
                    self.matrix.set_char(grid_x, grid_y, ch);
                }
                
                self.status_message = format!("Loaded PDF with {} characters extracted", self.matrix.chars.len());
            } else {
                self.status_message = "PDF loaded but text extraction failed".to_string();
            }
        }
        
        Ok(())
    }
    
    fn handle_selection(&mut self, code: KeyCode, modifiers: KeyModifiers) -> Result<()> {
        match code {
            KeyCode::Esc => {
                self.mode = Mode::Navigate;
                self.matrix.end_selection();
                self.status_message = "Navigation mode".to_string();
            }
            
            // Cursor movement
            KeyCode::Up => self.matrix.move_cursor(0, -1),
            KeyCode::Down => self.matrix.move_cursor(0, 1),
            KeyCode::Left => self.matrix.move_cursor(-1, 0),
            KeyCode::Right => self.matrix.move_cursor(1, 0),
            
            // Selection
            KeyCode::Char(' ') => {
                if !self.matrix.selecting {
                    self.matrix.start_selection();
                } else {
                    self.matrix.end_selection();
                }
            }
            
            // Copy
            KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                let text = self.matrix.copy_selection();
                // TODO: Copy to system clipboard
                self.status_message = format!("Copied {} characters", text.len());
            }
            
            _ => {}
        }
        Ok(())
    }
    
    fn handle_editing(&mut self, code: KeyCode, modifiers: KeyModifiers) -> Result<()> {
        match code {
            // PDF navigation with PageUp/PageDown
            KeyCode::PageUp => {
                if self.current_page > 0 {
                    self.current_page -= 1;
                    self.load_current_page()?;
                    self.status_message = format!("Page {}/{}", self.current_page + 1, self.total_pages);
                }
            }
            KeyCode::PageDown => {
                if self.current_page < self.total_pages.saturating_sub(1) {
                    self.current_page += 1;
                    self.load_current_page()?;
                    self.status_message = format!("Page {}/{}", self.current_page + 1, self.total_pages);
                }
            }
            
            // File operations
            KeyCode::Char('o') if modifiers.contains(KeyModifiers::SUPER) => {
                self.open_native_file_dialog()?;
            }
            
            // Cursor movement
            KeyCode::Up => self.matrix.move_cursor(0, -1),
            KeyCode::Down => self.matrix.move_cursor(0, 1),
            KeyCode::Left => self.matrix.move_cursor(-1, 0),
            KeyCode::Right => self.matrix.move_cursor(1, 0),
            
            // Home/End for line navigation
            KeyCode::Home => {
                // Move to beginning of current line
                let y = self.matrix.cursor.1;
                let mut x = 0;
                // Find first character in this line
                for test_x in 0..self.matrix.max_x {
                    if self.matrix.chars.contains_key(&(test_x, y)) {
                        x = test_x;
                        break;
                    }
                }
                self.matrix.cursor.0 = x;
            }
            KeyCode::End => {
                // Move to end of current line
                let y = self.matrix.cursor.1;
                let mut x = self.matrix.cursor.0;
                // Find last character in this line
                for test_x in (0..=self.matrix.max_x).rev() {
                    if self.matrix.chars.contains_key(&(test_x, y)) {
                        x = test_x + 1;  // Position after last char
                        break;
                    }
                }
                self.matrix.cursor.0 = x;
            }
            
            
            // Clipboard operations - MUST come before general Char match
            KeyCode::Char('x') if modifiers.contains(KeyModifiers::SUPER) => {
                // Cut - copy text and delete it
                self.cut_text();
            }
            KeyCode::Char('c') if modifiers.contains(KeyModifiers::SUPER) => {
                // Copy selected text or current character
                self.copy_text();
            }
            KeyCode::Char('v') if modifiers.contains(KeyModifiers::SUPER) => {
                // Paste from clipboard
                self.paste_text();
            }
            KeyCode::Char('a') if modifiers.contains(KeyModifiers::SUPER) => {
                // Select all text in current line
                self.select_line();
            }
            KeyCode::Char('s') if modifiers.contains(KeyModifiers::SUPER) => {
                self.status_message = "Save not yet implemented".to_string();
            }
            
            // Word navigation with Alt/Option - MUST come before general Char match
            KeyCode::Char('b') if modifiers.contains(KeyModifiers::ALT) => {
                // Move backward by word
                self.move_word_backward();
            }
            KeyCode::Char('f') if modifiers.contains(KeyModifiers::ALT) => {
                // Move forward by word
                self.move_word_forward();
            }
            
            // Line editing with Ctrl - MUST come before general Char match
            KeyCode::Char('k') if modifiers.contains(KeyModifiers::CONTROL) => {
                self.delete_to_end_of_line();
            }
            KeyCode::Char('a') if modifiers.contains(KeyModifiers::CONTROL) => {
                self.matrix.cursor.0 = 0;
            }
            KeyCode::Char('e') if modifiers.contains(KeyModifiers::CONTROL) => {
                // Find last character in line
                let y = self.matrix.cursor.1;
                let mut max_x = 0;
                for x in 0..200 {
                    if self.matrix.chars.contains_key(&(x, y)) {
                        max_x = x;
                    }
                }
                self.matrix.cursor.0 = max_x + 1;
            }
            
            // Character input - allow all typing in edit mode
            KeyCode::Char(ch) => {
                // Insert character at cursor position
                self.matrix.set_char(self.matrix.cursor.0, self.matrix.cursor.1, ch);
                self.matrix.move_cursor(1, 0);  // Move right after typing
                
                // Update status to show we're editing
                self.status_message = format!("Typed '{}'", ch);
            }
            
            // Special characters
            KeyCode::Enter => {
                // Move to next line, first column
                self.matrix.cursor.1 += 1;
                self.matrix.cursor.0 = 0;
            }
            KeyCode::Tab => {
                // Insert 4 spaces for tab
                for _ in 0..4 {
                    self.matrix.set_char(self.matrix.cursor.0, self.matrix.cursor.1, ' ');
                    self.matrix.move_cursor(1, 0);
                }
            }
            
            // Delete word with Alt+Backspace - MUST come before regular Backspace
            KeyCode::Backspace if modifiers.contains(KeyModifiers::ALT) => {
                self.delete_word_backward();
            }
            
            KeyCode::Backspace => {
                if self.matrix.cursor.0 > 0 {
                    self.matrix.move_cursor(-1, 0);
                    self.matrix.chars.remove(&(self.matrix.cursor.0, self.matrix.cursor.1));
                } else if self.matrix.cursor.1 > 0 {
                    // Move to end of previous line
                    self.matrix.cursor.1 -= 1;
                    self.matrix.cursor.0 = self.matrix.max_x;
                }
            }
            
            KeyCode::Delete => {
                self.matrix.chars.remove(&(self.matrix.cursor.0, self.matrix.cursor.1));
            }
            
            _ => {}
        }
        Ok(())
    }
    
    pub fn load_pdf_on_start(&mut self, path: PathBuf) -> Result<()> {
        self.load_pdf(path)
    }
    
    fn handle_file_selector_input(&mut self, code: KeyCode, _modifiers: KeyModifiers) -> Result<()> {
        match code {
            KeyCode::Esc => {
                self.file_selector.deactivate();
                self.status_message = "File selection cancelled".to_string();
            }
            KeyCode::Up => {
                self.file_selector.navigate_up();
            }
            KeyCode::Down => {
                self.file_selector.navigate_down();
            }
            KeyCode::Enter => {
                if let Some(pdf_path) = self.file_selector.enter_directory() {
                    self.file_selector.deactivate();
                    self.load_pdf(pdf_path)?;
                }
            }
            KeyCode::Backspace => {
                self.file_selector.go_up_directory();
            }
            _ => {}
        }
        Ok(())
    }
    
    fn handle_mouse_event(&mut self, mouse: crossterm::event::MouseEvent) -> Result<bool> {
        use crossterm::event::MouseEventKind;
        
        self.last_mouse_x = mouse.column;
        self.last_mouse_y = mouse.row;
        
        // Calculate split position
        let pdf_width = self.term_width / 2;
        
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                // Check if click is in matrix area
                if mouse.column > pdf_width {
                    // Convert to matrix coordinates
                    let matrix_x = (mouse.column - pdf_width - 1) as usize;
                    let matrix_y = mouse.row as usize;
                    
                    // Update cursor position
                    self.matrix.cursor = (
                        matrix_x + self.matrix.viewport_x,
                        matrix_y + self.matrix.viewport_y
                    );
                    
                    // Start selection if in select mode
                    if self.mode == Mode::Select {
                        self.matrix.start_selection();
                    }
                }
            }
            MouseEventKind::Drag(MouseButton::Left) => {
                if self.mode == Mode::Select && mouse.column > pdf_width {
                    // Update selection while dragging
                    let matrix_x = (mouse.column - pdf_width - 1) as usize;
                    let matrix_y = mouse.row as usize;
                    
                    self.matrix.cursor = (
                        matrix_x + self.matrix.viewport_x,
                        matrix_y + self.matrix.viewport_y
                    );
                    
                    self.matrix.update_selection();
                }
            }
            MouseEventKind::Up(MouseButton::Left) => {
                if self.mode == Mode::Select && self.matrix.selecting {
                    self.matrix.end_selection();
                    self.status_message = "Selection complete".to_string();
                }
            }
            MouseEventKind::ScrollDown => {
                // Scroll matrix viewport down
                self.matrix.viewport_y = self.matrix.viewport_y.saturating_add(3);
            }
            MouseEventKind::ScrollUp => {
                // Scroll matrix viewport up
                self.matrix.viewport_y = self.matrix.viewport_y.saturating_sub(3);
            }
            _ => {}
        }
        
        // Only return true if we need to rerender (button clicks, not just movement)
        Ok(matches!(mouse.kind, 
            MouseEventKind::Down(_) |
            MouseEventKind::Drag(_)
        ))
    }
    
    fn open_file_dialog(&mut self) -> Result<()> {
        // Try multiple locations for test PDFs
        let test_locations = vec![
            PathBuf::from("/Users/jack/chonker6/projects/chonker6/test.pdf"),
            PathBuf::from("/Users/jack/Downloads/test.pdf"),
            PathBuf::from("./test.pdf"),
            PathBuf::from("~/Desktop/test.pdf"),
        ];
        
        // Find first existing PDF
        for test_pdf in test_locations {
            let expanded = if test_pdf.starts_with("~") {
                let home = std::env::var("HOME").unwrap_or_else(|_| "/Users/jack".to_string());
                PathBuf::from(test_pdf.to_string_lossy().replace("~", &home))
            } else {
                test_pdf.clone()
            };
            
            if expanded.exists() {
                self.status_message = format!("Loading: {}", expanded.display());
                return self.load_pdf(expanded);
            }
        }
        
        // If no test PDF found, show instructions
        self.status_message = "No PDF found. Run: chonker6 /path/to/your.pdf".to_string();
        
        Ok(())
    }
    
    fn load_pdf(&mut self, path: PathBuf) -> Result<()> {
        // Clear any existing Kitty images before loading new PDF
        self.clear_kitty_images()?;
        
        // Clear the matrix for fresh start
        self.matrix.clear();
        
        // Drop the old engine to free resources
        self.pdf_engine = None;
        
        // Load the new PDF
        self.pdf_engine = Some(PdfEngine::new(&path)?);
        
        if let Some(engine) = &self.pdf_engine {
            self.total_pages = engine.page_count();
            self.current_page = 0;
            self.pdf_path = Some(path);
            
            self.load_current_page()?;
            self.status_message = format!("Loaded PDF with {} pages", self.total_pages);
        }
        
        Ok(())
    }
    
    fn load_current_page(&mut self) -> Result<()> {
        if let Some(engine) = &mut self.pdf_engine {
            // Extract text from current page
            let chars = engine.extract_text_with_positions(self.current_page)?;
            let page_height = engine.get_page_height(self.current_page)?;
            
            // Convert to CharInfo format
            let char_infos: Vec<CharInfo> = chars.into_iter()
                .map(|(x, y, ch)| CharInfo {
                    x,
                    y,
                    ch,
                    font_size: 12.0,  // Default for now
                })
                .collect();
            
            // Update matrix
            self.matrix = SpatialTextMatrix::from_pdf_coords(char_infos, page_height);
            self.status_message = format!("Page {}/{} - {} characters", 
                                        self.current_page + 1, 
                                        self.total_pages,
                                        self.matrix.chars.len());
        }
        
        Ok(())
    }
    
    // Clear all Kitty graphics from the terminal
    fn clear_kitty_images(&mut self) -> Result<()> {
        // Clear all images with action=d (delete) and d=A (all placements)
        print!("\x1b_Ga=d,d=A\x1b\\");
        stdout().flush()?;
        Ok(())
    }
    
    // Clipboard operations
    fn copy_text(&mut self) {
        // Get text first before borrowing clipboard
        let text = if let (Some(start), Some(end)) = (self.selection_start, self.selection_end) {
            self.get_selected_text(start, end)
        } else {
            // Copy current character
            self.matrix.chars.get(&self.matrix.cursor)
                .map(|c| c.to_string())
                .unwrap_or_default()
        };
        
        if let Some(ref mut clipboard) = self.clipboard {
            if !text.is_empty() {
                if let Err(e) = clipboard.set_contents(text.clone()) {
                    self.status_message = format!("Copy failed: {}", e);
                } else {
                    self.status_message = format!("Copied: {}", text);
                }
            }
        } else {
            self.status_message = "Clipboard not available".to_string();
        }
    }
    
    fn cut_text(&mut self) {
        // Get text and perform deletion first
        let text = if let (Some(start), Some(end)) = (self.selection_start, self.selection_end) {
            let text = self.get_selected_text(start, end);
            // Delete the selected text
            self.delete_selection(start, end);
            text
        } else {
            // Cut current character
            self.matrix.chars.remove(&self.matrix.cursor)
                .map(|c| c.to_string())
                .unwrap_or_default()
        };
        
        // Then handle clipboard
        if let Some(ref mut clipboard) = self.clipboard {
            if !text.is_empty() {
                if let Err(e) = clipboard.set_contents(text.clone()) {
                    self.status_message = format!("Cut failed: {}", e);
                } else {
                    self.status_message = format!("Cut: {}", text);
                }
            }
        } else {
            self.status_message = "Clipboard not available".to_string();
        }
    }
    
    fn paste_text(&mut self) {
        if let Some(ref mut clipboard) = self.clipboard {
            match clipboard.get_contents() {
                Ok(text) => {
                    // Insert text at current cursor position
                    for (i, ch) in text.chars().enumerate() {
                        if ch == '\n' {
                            // Move to next line
                            self.matrix.cursor.1 += 1;
                            self.matrix.cursor.0 = 0;
                        } else {
                            self.matrix.set_char(
                                self.matrix.cursor.0 + i,
                                self.matrix.cursor.1,
                                ch
                            );
                        }
                    }
                    self.status_message = format!("Pasted {} chars", text.len());
                }
                Err(e) => {
                    self.status_message = format!("Paste failed: {}", e);
                }
            }
        } else {
            self.status_message = "Clipboard not available".to_string();
        }
    }
    
    fn select_line(&mut self) {
        let y = self.matrix.cursor.1;
        let mut min_x = usize::MAX;
        let mut max_x = 0;
        
        // Find bounds of text on current line
        for x in 0..200 {  // Reasonable max width
            if self.matrix.chars.contains_key(&(x, y)) {
                min_x = min_x.min(x);
                max_x = max_x.max(x);
            }
        }
        
        if min_x != usize::MAX {
            self.selection_start = Some((min_x, y));
            self.selection_end = Some((max_x + 1, y));
            self.status_message = "Line selected".to_string();
        }
    }
    
    fn get_selected_text(&self, start: (usize, usize), end: (usize, usize)) -> String {
        let mut text = String::new();
        
        // Handle single line or multi-line selection
        if start.1 == end.1 {
            // Single line
            for x in start.0..end.0 {
                if let Some(ch) = self.matrix.chars.get(&(x, start.1)) {
                    text.push(*ch);
                }
            }
        } else {
            // Multi-line - simplified for now
            for y in start.1..=end.1 {
                for x in 0..200 {  // Reasonable max width
                    if let Some(ch) = self.matrix.chars.get(&(x, y)) {
                        text.push(*ch);
                    }
                }
                if y < end.1 {
                    text.push('\n');
                }
            }
        }
        
        text
    }
    
    fn delete_selection(&mut self, start: (usize, usize), end: (usize, usize)) {
        if start.1 == end.1 {
            // Single line deletion
            for x in start.0..end.0 {
                self.matrix.chars.remove(&(x, start.1));
            }
        } else {
            // Multi-line deletion
            for y in start.1..=end.1 {
                for x in 0..200 {  // Reasonable max width
                    self.matrix.chars.remove(&(x, y));
                }
            }
        }
        
        // Clear selection
        self.selection_start = None;
        self.selection_end = None;
    }
    
    // Text editing helper methods
    fn move_word_backward(&mut self) {
        let y = self.matrix.cursor.1;
        let mut x = self.matrix.cursor.0;
        
        // Skip spaces backward
        while x > 0 && !self.matrix.chars.contains_key(&(x - 1, y)) {
            x -= 1;
        }
        
        // Move to beginning of word
        while x > 0 && self.matrix.chars.contains_key(&(x - 1, y)) {
            let ch = self.matrix.chars.get(&(x - 1, y)).unwrap_or(&' ');
            if ch.is_whitespace() {
                break;
            }
            x -= 1;
        }
        
        self.matrix.cursor.0 = x;
    }
    
    fn move_word_forward(&mut self) {
        let y = self.matrix.cursor.1;
        let mut x = self.matrix.cursor.0;
        
        // Move to end of current word
        while x < 200 && self.matrix.chars.contains_key(&(x, y)) {
            let ch = self.matrix.chars.get(&(x, y)).unwrap_or(&' ');
            if ch.is_whitespace() {
                break;
            }
            x += 1;
        }
        
        // Skip spaces forward
        while x < 200 && self.matrix.chars.contains_key(&(x, y)) {
            let ch = self.matrix.chars.get(&(x, y)).unwrap_or(&' ');
            if !ch.is_whitespace() {
                break;
            }
            x += 1;
        }
        
        self.matrix.cursor.0 = x;
    }
    
    fn delete_word_backward(&mut self) {
        let y = self.matrix.cursor.1;
        let start_x = self.matrix.cursor.0;
        
        // Move backward by word
        self.move_word_backward();
        let end_x = self.matrix.cursor.0;
        
        // Delete characters from new position to old position
        for x in end_x..start_x {
            self.matrix.chars.remove(&(x, y));
        }
    }
    
    fn delete_to_end_of_line(&mut self) {
        let y = self.matrix.cursor.1;
        let start_x = self.matrix.cursor.0;
        
        // Delete all characters from cursor to end of line
        for x in start_x..200 {
            self.matrix.chars.remove(&(x, y));
        }
    }
}