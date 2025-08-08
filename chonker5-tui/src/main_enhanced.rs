use ratatui::{prelude::*, widgets::*};
use crossterm::event::{self, Event, KeyCode, KeyModifiers, MouseEventKind};
use pdfium_render::prelude::*;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use anyhow::Result;
use std::io::Write;

mod pdf_cache;
use pdf_cache::{PdfCache, ProgressiveLoader};

// ============= CONSTANTS =============
const TERM_BG: Color = Color::Rgb(0, 0, 0);
const TERM_FG: Color = Color::Rgb(200, 200, 200);
const TERM_TEAL: Color = Color::Rgb(26, 188, 156);
const TERM_DIM: Color = Color::Rgb(80, 80, 80);
const TERM_ERROR: Color = Color::Rgb(231, 76, 60);

// Re-use core types instead of include
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

// Simple character matrix for TUI
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

// Base TUI struct (simplified)
struct ChonkerTUI {
    pdf_path: Option<PathBuf>,
    current_page: usize,
    total_pages: usize,
    pdf_render_cache: Option<String>,
    character_matrix: Option<CharacterMatrix>,
    editable_matrix: Option<Vec<Vec<char>>>,
    focus: PaneFocus,
    split_ratio: u16,
    cursor: (usize, usize),
    mode: Mode,
    status_message: String,
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
            mode: Mode::Normal,
            status_message: "Press 'o' to open PDF, '?' for help".to_string(),
            pdf_scroll: (0, 0),
            matrix_scroll: (0, 0),
        }
    }
    
    fn open_pdf(&mut self) -> Result<()> {
        // Simplified - would implement full PDF loading
        self.status_message = "PDF loading...".to_string();
        Ok(())
    }
    
    fn render_current_page(&mut self) -> Result<()> {
        if let Some(_pdf_path) = &self.pdf_path {
            self.pdf_render_cache = Some(format!(
                "PDF Page {}/{}\n\n[PDF content]\n\nUse ← → to navigate",
                self.current_page + 1,
                self.total_pages
            ));
        }
        Ok(())
    }
    
    fn handle_event(&mut self, _event: Event) -> Result<bool> {
        // Base implementation
        Ok(false)
    }
    
    fn render(&mut self, area: Rect, buf: &mut Buffer) {
        // Split content area
        let chunks = Layout::horizontal([
            Constraint::Percentage(self.split_ratio),
            Constraint::Percentage(100 - self.split_ratio),
        ]).split(area);
        
        // PDF pane
        let pdf_block = Block::default()
            .borders(Borders::ALL)
            .title(" PDF ")
            .border_style(Style::default().fg(TERM_DIM));
        pdf_block.render(chunks[0], buf);
        
        // Matrix pane
        let matrix_block = Block::default()
            .borders(Borders::ALL)
            .title(" Matrix ")
            .border_style(Style::default().fg(TERM_DIM));
        matrix_block.render(chunks[1], buf);
    }
}

// ============= ENHANCED TUI STRUCT =============
struct ChonkerTUIEnhanced {
    // Base TUI
    base: ChonkerTUI,
    
    // Advanced caching
    pdf_cache: PdfCache,
    progressive_loader: ProgressiveLoader,
    
    // Performance metrics
    last_render_time: Duration,
    cache_hits: usize,
    cache_misses: usize,
    
    // Render mode
    render_mode: RenderMode,
}

#[derive(Clone, Copy, PartialEq)]
enum RenderMode {
    Fast,        // Low DPI, grayscale
    Quality,     // High DPI, color
    Progressive, // Start low, upgrade to high
}

impl ChonkerTUIEnhanced {
    fn new() -> Self {
        Self {
            base: ChonkerTUI::new(),
            pdf_cache: PdfCache::new(20), // Cache up to 20 pages
            progressive_loader: ProgressiveLoader::new(),
            last_render_time: Duration::ZERO,
            cache_hits: 0,
            cache_misses: 0,
            render_mode: RenderMode::Progressive,
        }
    }
    
    fn open_pdf(&mut self) -> Result<()> {
        // Use base implementation
        self.base.open_pdf()?;
        
        // Set up caching
        if let Some(pdf_path) = &self.base.pdf_path {
            self.pdf_cache.set_pdf_path(pdf_path.clone());
            
            // Pre-render first page
            self.change_page(0)?;
        }
        
        Ok(())
    }
    
    fn change_page(&mut self, new_page: usize) -> Result<()> {
        let start = Instant::now();
        
        // Use cache for fast page changes
        match self.pdf_cache.change_page(new_page, self.base.total_pages)? {
            Some(rendered_text) => {
                self.base.pdf_render_cache = Some(rendered_text);
                self.cache_hits += 1;
            }
            None => {
                // Fallback to base rendering
                self.base.render_current_page()?;
                self.cache_misses += 1;
            }
        }
        
        self.base.current_page = new_page;
        self.last_render_time = start.elapsed();
        
        // Update status with performance info
        let (cached, max_cache) = self.pdf_cache.get_cache_stats();
        self.base.status_message = format!(
            "Page {}/{} | Render: {:?} | Cache: {}/{} | Hits: {}",
            new_page + 1,
            self.base.total_pages,
            self.last_render_time,
            cached,
            max_cache,
            self.cache_hits
        );
        
        Ok(())
    }
    
    fn handle_event(&mut self, event: Event) -> Result<bool> {
        match event {
            Event::Key(key) => match (key.code, key.modifiers, self.base.mode) {
                // Performance mode switching
                (KeyCode::Char('1'), _, Mode::Normal) => {
                    self.render_mode = RenderMode::Fast;
                    self.base.status_message = "Switched to FAST rendering mode".to_string();
                }
                (KeyCode::Char('2'), _, Mode::Normal) => {
                    self.render_mode = RenderMode::Quality;
                    self.base.status_message = "Switched to QUALITY rendering mode".to_string();
                }
                (KeyCode::Char('3'), _, Mode::Normal) => {
                    self.render_mode = RenderMode::Progressive;
                    self.base.status_message = "Switched to PROGRESSIVE rendering mode".to_string();
                }
                
                // Enhanced page navigation with caching
                (KeyCode::Left, _, Mode::Normal) if matches!(self.base.focus, PaneFocus::PdfPane) => {
                    if self.base.current_page > 0 {
                        self.change_page(self.base.current_page - 1)?;
                    }
                }
                (KeyCode::Right, _, Mode::Normal) if matches!(self.base.focus, PaneFocus::PdfPane) => {
                    if self.base.current_page + 1 < self.base.total_pages {
                        self.change_page(self.base.current_page + 1)?;
                    }
                }
                
                // Page jump (10 pages)
                (KeyCode::PageUp, _, Mode::Normal) if matches!(self.base.focus, PaneFocus::PdfPane) => {
                    let new_page = self.base.current_page.saturating_sub(10);
                    self.change_page(new_page)?;
                }
                (KeyCode::PageDown, _, Mode::Normal) if matches!(self.base.focus, PaneFocus::PdfPane) => {
                    let new_page = (self.base.current_page + 10).min(self.base.total_pages - 1);
                    self.change_page(new_page)?;
                }
                
                // Clear cache
                (KeyCode::Char('c'), KeyModifiers::CONTROL, Mode::Normal) => {
                    self.pdf_cache = PdfCache::new(20);
                    self.cache_hits = 0;
                    self.cache_misses = 0;
                    self.base.status_message = "Cache cleared".to_string();
                }
                
                // Delegate other events to base
                _ => return self.base.handle_event(event),
            },
            _ => return self.base.handle_event(event),
        }
        Ok(false)
    }
}

// ============= TERMINAL-SPECIFIC OPTIMIZATIONS =============

#[cfg(target_os = "macos")]
fn detect_terminal() -> TerminalType {
    if std::env::var("TERM_PROGRAM").unwrap_or_default() == "iTerm.app" {
        TerminalType::ITerm2
    } else if std::env::var("KITTY_WINDOW_ID").is_ok() {
        TerminalType::Kitty
    } else {
        TerminalType::Generic
    }
}

#[cfg(not(target_os = "macos"))]
fn detect_terminal() -> TerminalType {
    if std::env::var("KITTY_WINDOW_ID").is_ok() {
        TerminalType::Kitty
    } else {
        TerminalType::Generic
    }
}

enum TerminalType {
    ITerm2,
    Kitty,
    Generic,
}

// ============= RENDERING =============
impl Widget for &mut ChonkerTUIEnhanced {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Use base rendering with performance overlay
        self.base.render(area, buf);
        
        // Add performance metrics overlay
        if self.cache_hits > 0 || self.cache_misses > 0 {
            let hit_rate = (self.cache_hits as f32 / (self.cache_hits + self.cache_misses) as f32) * 100.0;
            let perf_text = format!("Cache Hit Rate: {:.1}%", hit_rate);
            
            let perf_area = Rect {
                x: area.width - perf_text.len() as u16 - 2,
                y: 1,
                width: perf_text.len() as u16,
                height: 1,
            };
            
            Paragraph::new(perf_text)
                .style(Style::default().fg(TERM_TEAL))
                .render(perf_area, buf);
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
    
    // Detect terminal for optimizations
    let term_type = detect_terminal();
    println!("Detected terminal: {:?}", match term_type {
        TerminalType::ITerm2 => "iTerm2",
        TerminalType::Kitty => "Kitty",
        TerminalType::Generic => "Generic",
    });
    
    // App state
    let mut app = ChonkerTUIEnhanced::new();
    
    // Main loop with faster event polling
    let mut should_quit = false;
    while !should_quit {
        // Draw
        terminal.draw(|f| {
            f.render_widget(&mut app, f.area());
        })?;
        
        // Handle events with shorter timeout for snappier response
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
    
    // Print performance summary
    println!("\nPerformance Summary:");
    println!("Cache Hits: {}", app.cache_hits);
    println!("Cache Misses: {}", app.cache_misses);
    if app.cache_hits + app.cache_misses > 0 {
        let hit_rate = (app.cache_hits as f32 / (app.cache_hits + app.cache_misses) as f32) * 100.0;
        println!("Hit Rate: {:.1}%", hit_rate);
    }
    
    Ok(())
}