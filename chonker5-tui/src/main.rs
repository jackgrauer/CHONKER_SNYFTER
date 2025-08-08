use ratatui::{prelude::*, widgets::*};
use crossterm::event::{self, Event, KeyCode, KeyModifiers, MouseEventKind};
use pdfium_render::prelude::*;
use std::path::PathBuf;
use std::time::Duration;
use anyhow::Result;
use std::io::Write;

// ============= CONSTANTS =============
const TERM_BG: Color = Color::Rgb(0, 0, 0);
const TERM_FG: Color = Color::Rgb(200, 200, 200);
const TERM_TEAL: Color = Color::Rgb(26, 188, 156);
const TERM_DIM: Color = Color::Rgb(80, 80, 80);
const TERM_ERROR: Color = Color::Rgb(231, 76, 60);

// ============= CORE TYPES =============
#[derive(Clone, Copy, PartialEq, Debug)]
enum PaneFocus {
    PdfPane,
    MatrixPane,
}

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Normal,
    Insert,
    Visual,
}

#[derive(Clone)]
struct Selection {
    start: Option<(usize, usize)>,
    end: Option<(usize, usize)>,
}

impl Selection {
    fn new() -> Self {
        Self { start: None, end: None }
    }

    fn is_selected(&self, row: usize, col: usize) -> bool {
        if let (Some(start), Some(end)) = (self.start, self.end) {
            let min_row = start.0.min(end.0);
            let max_row = start.0.max(end.0);
            let min_col = start.1.min(end.1);
            let max_col = start.1.max(end.1);
            row >= min_row && row <= max_row && col >= min_col && col <= max_col
        } else {
            false
        }
    }

    fn get_bounds(&self) -> Option<(usize, usize, usize, usize)> {
        if let (Some(start), Some(end)) = (self.start, self.end) {
            let min_row = start.0.min(end.0);
            let max_row = start.0.max(end.0);
            let min_col = start.1.min(end.1);
            let max_col = start.1.max(end.1);
            Some((min_row, max_row, min_col, max_col))
        } else {
            None
        }
    }
}

// Reuse from original - simplified
#[derive(Clone)]
struct CharacterMatrix {
    width: usize,
    height: usize,
    matrix: Vec<Vec<char>>,
}

impl CharacterMatrix {
    fn new(width: usize, height: usize) -> Self {
        let matrix = vec![vec![' '; width]; height];
        Self { width, height, matrix }
    }
}

// ============= MAIN TUI STRUCT =============
struct ChonkerTUI {
    // PDF State
    pdf_path: Option<PathBuf>,
    current_page: usize,
    total_pages: usize,
    pdf_render_cache: Option<String>, // ASCII art representation
    
    // Character Matrix
    character_matrix: Option<CharacterMatrix>,
    editable_matrix: Option<Vec<Vec<char>>>,
    
    // UI State
    focus: PaneFocus,
    split_ratio: u16,
    
    // Matrix Editor
    cursor: (usize, usize),
    selection: Selection,
    clipboard: Vec<Vec<char>>,
    
    // View mode
    mode: Mode,
    
    // Status/log messages
    status_message: String,
    
    // Scrolling
    pdf_scroll: (u16, u16),
    matrix_scroll: (u16, u16),
}

impl ChonkerTUI {
    fn new() -> Self {
        Self {
            pdf_path: None,
            current_page: 0,
            total_pages: 0,
            pdf_render_cache: None,
            character_matrix: None,
            editable_matrix: None,
            focus: PaneFocus::PdfPane,
            split_ratio: 50,
            cursor: (0, 0),
            selection: Selection::new(),
            clipboard: Vec::new(),
            mode: Mode::Normal,
            status_message: "Press 'o' to open PDF, '?' for help".to_string(),
            pdf_scroll: (0, 0),
            matrix_scroll: (0, 0),
        }
    }

    fn open_pdf(&mut self) -> Result<()> {
        // Simple file path input
        print!("\rEnter PDF path: ");
        std::io::stdout().flush()?;
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let path = PathBuf::from(input.trim());
        
        if path.exists() && path.extension().map_or(false, |ext| ext == "pdf") {
            self.pdf_path = Some(path.clone());
            
            // Initialize PDFium and load document to get page count
            let pdfium = Pdfium::new(
                Pdfium::bind_to_library(
                    Pdfium::pdfium_platform_library_name_at_path("./lib/")
                ).or_else(|_| Pdfium::bind_to_system_library())?
            );
            
            let document = pdfium.load_pdf_from_file(&path, None)?;
            self.total_pages = document.pages().len() as usize;
            self.current_page = 0;
            
            self.render_current_page()?;
            self.status_message = format!("Loaded: {} ({} pages)", path.display(), self.total_pages);
        } else {
            self.status_message = "Invalid PDF path".to_string();
        }
        
        Ok(())
    }

    fn render_current_page(&mut self) -> Result<()> {
        if let Some(_pdf_path) = &self.pdf_path {
            // For TUI, we'll create a simple ASCII representation
            // In a real implementation, you'd use ratatui-image or similar
            self.pdf_render_cache = Some(format!(
                "PDF Page {}/{}\n\n[PDF content would be rendered here]\n\nUse ← → to navigate pages",
                self.current_page + 1,
                self.total_pages
            ));
        }
        Ok(())
    }

    fn extract_matrix(&mut self) -> Result<()> {
        if let Some(pdf_path) = &self.pdf_path {
            // Reinitialize PDFium for extraction
            let pdfium = Pdfium::new(
                Pdfium::bind_to_library(
                    Pdfium::pdfium_platform_library_name_at_path("./lib/")
                ).or_else(|_| Pdfium::bind_to_system_library())?
            );
            
            // Simplified matrix extraction
            let document = pdfium.load_pdf_from_file(pdf_path, None)?;
            let page = document.pages().get(self.current_page as u16)?;
            
            // Get page dimensions
            let width = page.width().value as usize / 7;  // Approximate char width
            let height = page.height().value as usize / 12; // Approximate char height
            
            let mut matrix = CharacterMatrix::new(width.min(200), height.min(100));
            
            // Extract text objects
            let _text_page = page.text()?;
            for object in page.objects().iter() {
                if let Some(text_obj) = object.as_text_object() {
                    let bounds = text_obj.bounds()?;
                    let text = text_obj.text();
                    
                    // Convert PDF coordinates to matrix coordinates
                    let x = ((bounds.left().value / 7.0) as usize).min(width - 1);
                    let y = ((height as f32 - bounds.top().value / 12.0) as usize).min(height - 1);
                    
                    // Place text in matrix
                    for (i, ch) in text.chars().enumerate() {
                        if x + i < matrix.width && y < matrix.height {
                            matrix.matrix[y][x + i] = ch;
                        }
                    }
                }
            }
            
            self.editable_matrix = Some(matrix.matrix.clone());
            self.character_matrix = Some(matrix);
            self.status_message = format!("Extracted {}x{} character matrix", width, height);
        } else {
            self.status_message = "No PDF loaded".to_string();
        }
        Ok(())
    }

    fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            PaneFocus::PdfPane => PaneFocus::MatrixPane,
            PaneFocus::MatrixPane => PaneFocus::PdfPane,
        };
    }

    fn handle_event(&mut self, event: Event) -> Result<bool> {
        match event {
            Event::Key(key) => match (key.code, key.modifiers, self.mode) {
                // Global commands
                (KeyCode::Char('q'), KeyModifiers::CONTROL, _) => return Ok(true),
                (KeyCode::Tab, _, _) => {
                    self.toggle_focus();
                    self.status_message = format!("Focus: {:?}", self.focus);
                }
                (KeyCode::Char('?'), _, Mode::Normal) => {
                    self.status_message = "Commands: o=open m=extract i=insert v=visual ←→=pages hjkl=move Tab=switch Ctrl-Q=quit".to_string();
                }
                
                // Mode switches
                (KeyCode::Esc, _, _) => {
                    self.mode = Mode::Normal;
                    self.selection = Selection::new();
                    self.status_message = "NORMAL mode".to_string();
                }
                (KeyCode::Char('i'), _, Mode::Normal) if matches!(self.focus, PaneFocus::MatrixPane) => {
                    self.mode = Mode::Insert;
                    self.status_message = "INSERT mode".to_string();
                }
                (KeyCode::Char('v'), _, Mode::Normal) if matches!(self.focus, PaneFocus::MatrixPane) => {
                    self.mode = Mode::Visual;
                    self.selection.start = Some(self.cursor);
                    self.selection.end = Some(self.cursor);
                    self.status_message = "VISUAL mode".to_string();
                }
                
                // Normal mode commands
                (KeyCode::Char('o'), _, Mode::Normal) => self.open_pdf()?,
                (KeyCode::Char('m'), _, Mode::Normal) => self.extract_matrix()?,
                
                // Page navigation
                (KeyCode::Left, _, Mode::Normal) if matches!(self.focus, PaneFocus::PdfPane) => {
                    if self.current_page > 0 {
                        self.current_page -= 1;
                        self.render_current_page()?;
                    }
                }
                (KeyCode::Right, _, Mode::Normal) if matches!(self.focus, PaneFocus::PdfPane) => {
                    if self.current_page + 1 < self.total_pages {
                        self.current_page += 1;
                        self.render_current_page()?;
                    }
                }
                
                // Matrix navigation (vim-like)
                (KeyCode::Char('h'), _, Mode::Normal | Mode::Visual) if matches!(self.focus, PaneFocus::MatrixPane) => {
                    self.move_cursor(-1, 0);
                }
                (KeyCode::Char('j'), _, Mode::Normal | Mode::Visual) if matches!(self.focus, PaneFocus::MatrixPane) => {
                    self.move_cursor(0, 1);
                }
                (KeyCode::Char('k'), _, Mode::Normal | Mode::Visual) if matches!(self.focus, PaneFocus::MatrixPane) => {
                    self.move_cursor(0, -1);
                }
                (KeyCode::Char('l'), _, Mode::Normal | Mode::Visual) if matches!(self.focus, PaneFocus::MatrixPane) => {
                    self.move_cursor(1, 0);
                }
                
                // Insert mode - edit matrix
                (KeyCode::Char(c), _, Mode::Insert) if matches!(self.focus, PaneFocus::MatrixPane) => {
                    self.insert_char(c);
                }
                (KeyCode::Backspace, _, Mode::Insert) if matches!(self.focus, PaneFocus::MatrixPane) => {
                    if self.cursor.1 > 0 {
                        self.cursor.1 -= 1;
                        self.insert_char(' ');
                        self.cursor.1 -= 1;
                    }
                }
                
                // Visual mode - selection operations
                (KeyCode::Char('y'), _, Mode::Visual) => self.copy_selection(),
                (KeyCode::Char('d'), _, Mode::Visual) => self.cut_selection(),
                (KeyCode::Char('p'), _, Mode::Normal) if matches!(self.focus, PaneFocus::MatrixPane) => {
                    self.paste();
                }
                
                // Split ratio adjustment
                (KeyCode::Char('+'), KeyModifiers::CONTROL, _) => {
                    self.split_ratio = (self.split_ratio + 5).min(80);
                }
                (KeyCode::Char('-'), KeyModifiers::CONTROL, _) => {
                    self.split_ratio = (self.split_ratio.saturating_sub(5)).max(20);
                }
                
                _ => {}
            },
            
            Event::Mouse(mouse) => match mouse.kind {
                MouseEventKind::Down(_) => {
                    // Determine which pane was clicked
                    let x = mouse.column;
                    let term_width = crossterm::terminal::size()?.0;
                    let split_x = (term_width * self.split_ratio / 100) as u16;
                    
                    self.focus = if x < split_x {
                        PaneFocus::PdfPane
                    } else {
                        PaneFocus::MatrixPane
                    };
                }
                _ => {}
            },
            
            _ => {}
        }
        Ok(false)
    }

    fn move_cursor(&mut self, dx: i32, dy: i32) {
        if let Some(matrix) = &self.editable_matrix {
            let new_x = (self.cursor.1 as i32 + dx).max(0) as usize;
            let new_y = (self.cursor.0 as i32 + dy).max(0) as usize;
            
            if new_y < matrix.len() && new_x < matrix.get(new_y).map_or(0, |row| row.len()) {
                self.cursor = (new_y, new_x);
                
                if self.mode == Mode::Visual {
                    self.selection.end = Some(self.cursor);
                }
            }
        }
    }

    fn insert_char(&mut self, c: char) {
        if let Some(matrix) = &mut self.editable_matrix {
            if self.cursor.0 < matrix.len() && self.cursor.1 < matrix[self.cursor.0].len() {
                matrix[self.cursor.0][self.cursor.1] = c;
                self.move_cursor(1, 0);
            }
        }
    }

    fn copy_selection(&mut self) {
        if let (Some(matrix), Some(bounds)) = (&self.editable_matrix, self.selection.get_bounds()) {
            let (min_row, max_row, min_col, max_col) = bounds;
            self.clipboard.clear();
            
            for row in min_row..=max_row {
                if row < matrix.len() {
                    let mut row_chars = Vec::new();
                    for col in min_col..=max_col {
                        if col < matrix[row].len() {
                            row_chars.push(matrix[row][col]);
                        }
                    }
                    self.clipboard.push(row_chars);
                }
            }
            
            self.status_message = format!("Copied {}x{} block", max_row - min_row + 1, max_col - min_col + 1);
            self.mode = Mode::Normal;
            self.selection = Selection::new();
        }
    }

    fn cut_selection(&mut self) {
        if let (Some(matrix), Some(bounds)) = (&mut self.editable_matrix, self.selection.get_bounds()) {
            let (min_row, max_row, min_col, max_col) = bounds;
            self.clipboard.clear();
            
            for row in min_row..=max_row {
                if row < matrix.len() {
                    let mut row_chars = Vec::new();
                    for col in min_col..=max_col {
                        if col < matrix[row].len() {
                            row_chars.push(matrix[row][col]);
                            matrix[row][col] = ' ';
                        }
                    }
                    self.clipboard.push(row_chars);
                }
            }
            
            self.status_message = format!("Cut {}x{} block", max_row - min_row + 1, max_col - min_col + 1);
            self.mode = Mode::Normal;
            self.selection = Selection::new();
        }
    }

    fn paste(&mut self) {
        if let Some(matrix) = &mut self.editable_matrix {
            for (i, clipboard_row) in self.clipboard.iter().enumerate() {
                let target_row = self.cursor.0 + i;
                if target_row < matrix.len() {
                    for (j, &ch) in clipboard_row.iter().enumerate() {
                        let target_col = self.cursor.1 + j;
                        if target_col < matrix[target_row].len() {
                            matrix[target_row][target_col] = ch;
                        }
                    }
                }
            }
            
            if !self.clipboard.is_empty() {
                self.status_message = format!("Pasted {}x{} block at ({}, {})", 
                    self.clipboard.len(), 
                    self.clipboard.get(0).map_or(0, |r| r.len()),
                    self.cursor.0, self.cursor.1
                );
            }
        }
    }
}

// ============= RENDERING =============
impl Widget for &mut ChonkerTUI {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Create main layout
        let main_chunks = Layout::vertical([
            Constraint::Min(3),
            Constraint::Length(3),
        ]).split(area);
        
        // Split content area horizontally
        let content_chunks = Layout::horizontal([
            Constraint::Percentage(self.split_ratio),
            Constraint::Percentage(100 - self.split_ratio),
        ]).split(main_chunks[0]);
        
        // Render PDF pane
        self.render_pdf_pane(content_chunks[0], buf);
        
        // Render Matrix pane
        self.render_matrix_pane(content_chunks[1], buf);
        
        // Render status bar
        self.render_status_bar(main_chunks[1], buf);
    }
}

impl ChonkerTUI {
    fn render_pdf_pane(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(if matches!(self.focus, PaneFocus::PdfPane) {
                Style::default().fg(TERM_TEAL)
            } else {
                Style::default().fg(TERM_DIM)
            })
            .title(format!(" PDF [Page {}/{}] ", self.current_page + 1, self.total_pages.max(1)));
        
        let inner = block.inner(area);
        block.render(area, buf);
        
        // Render PDF content (simplified ASCII representation)
        if let Some(content) = &self.pdf_render_cache {
            let paragraph = Paragraph::new(content.as_str())
                .style(Style::default().fg(TERM_FG))
                .wrap(Wrap { trim: true })
                .scroll(self.pdf_scroll);
            paragraph.render(inner, buf);
        } else {
            let help = Paragraph::new("No PDF loaded\n\nPress 'o' to open a PDF file")
                .style(Style::default().fg(TERM_DIM))
                .alignment(Alignment::Center);
            help.render(inner, buf);
        }
    }
    
    fn render_matrix_pane(&mut self, area: Rect, buf: &mut Buffer) {
        let mode_str = match self.mode {
            Mode::Normal => "NORMAL",
            Mode::Insert => "INSERT",
            Mode::Visual => "VISUAL",
        };
        
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(if matches!(self.focus, PaneFocus::MatrixPane) {
                Style::default().fg(TERM_TEAL)
            } else {
                Style::default().fg(TERM_DIM)
            })
            .title(format!(" Matrix [{}] ", mode_str));
        
        let inner = block.inner(area);
        block.render(area, buf);
        
        // Render matrix content
        if let Some(matrix) = &self.editable_matrix {
            let viewport_height = inner.height as usize;
            let viewport_width = inner.width as usize;
            
            // Calculate visible area
            let start_row = self.matrix_scroll.0 as usize;
            let end_row = (start_row + viewport_height).min(matrix.len());
            
            for (view_y, row_idx) in (start_row..end_row).enumerate() {
                if let Some(row) = matrix.get(row_idx) {
                    let start_col = self.matrix_scroll.1 as usize;
                    let end_col = (start_col + viewport_width).min(row.len());
                    
                    for (view_x, col_idx) in (start_col..end_col).enumerate() {
                        if let Some(&ch) = row.get(col_idx) {
                            let x = inner.x + view_x as u16;
                            let y = inner.y + view_y as u16;
                            
                            let style = if (row_idx, col_idx) == self.cursor && matches!(self.focus, PaneFocus::MatrixPane) {
                                Style::default().bg(TERM_TEAL).fg(Color::Black)
                            } else if self.selection.is_selected(row_idx, col_idx) {
                                Style::default().bg(TERM_DIM)
                            } else {
                                Style::default().fg(TERM_FG)
                            };
                            
                            buf[(x, y)]
                                .set_char(ch)
                                .set_style(style);
                        }
                    }
                }
            }
            
            // Adjust scroll to keep cursor visible
            if matches!(self.focus, PaneFocus::MatrixPane) {
                // Vertical scrolling
                if self.cursor.0 < start_row {
                    self.matrix_scroll.0 = self.cursor.0 as u16;
                } else if self.cursor.0 >= start_row + viewport_height {
                    self.matrix_scroll.0 = (self.cursor.0 - viewport_height + 1) as u16;
                }
                
                // Horizontal scrolling
                let start_col = self.matrix_scroll.1 as usize;
                if self.cursor.1 < start_col {
                    self.matrix_scroll.1 = self.cursor.1 as u16;
                } else if self.cursor.1 >= start_col + viewport_width {
                    self.matrix_scroll.1 = (self.cursor.1 - viewport_width + 1) as u16;
                }
            }
        } else {
            let help = Paragraph::new("No matrix extracted\n\nPress 'm' to extract character matrix")
                .style(Style::default().fg(TERM_DIM))
                .alignment(Alignment::Center);
            help.render(inner, buf);
        }
    }
    
    fn render_status_bar(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(TERM_DIM));
        
        let inner = block.inner(area);
        block.render(area, buf);
        
        // Status message
        let status = Paragraph::new(self.status_message.as_str())
            .style(Style::default().fg(TERM_FG));
        status.render(inner, buf);
        
        // Right-aligned info
        let info = format!("Cursor: ({},{}) | Split: {}%", self.cursor.0, self.cursor.1, self.split_ratio);
        let info_width = info.len() as u16;
        if inner.width > info_width {
            let info_area = Rect {
                x: inner.x + inner.width - info_width,
                y: inner.y,
                width: info_width,
                height: 1,
            };
            Paragraph::new(info)
                .style(Style::default().fg(TERM_DIM))
                .render(info_area, buf);
        }
    }
}

// ============= MAIN =============
fn main() -> Result<()> {
    // Terminal setup
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(
        stdout,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )?;
    
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    
    // App state
    let mut app = ChonkerTUI::new();
    
    // Main loop
    let mut should_quit = false;
    while !should_quit {
        // Draw
        terminal.draw(|f| {
            f.render_widget(&mut app, f.area());
        })?;
        
        // Handle events with timeout for responsive UI
        if event::poll(Duration::from_millis(16))? {
            should_quit = app.handle_event(event::read()?)?;
        }
    }
    
    // Cleanup
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;
    
    Ok(())
}