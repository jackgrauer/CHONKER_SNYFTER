use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, List, ListItem, ListState, Paragraph, Tabs,
    },
    Frame, Terminal,
};
use std::io::{self, Write};
use tracing::{info, error};
use crate::database::ChonkerDatabase;

/// TUI Application state
struct App {
    should_quit: bool,
    current_tab: usize,
    database: ChonkerDatabase,
    documents: Vec<DocumentItem>,
    #[allow(dead_code)]
    selected_document: Option<usize>,
    list_state: ListState,
    status_message: String,
}

#[derive(Clone)]
struct DocumentItem {
    id: String,
    filename: String,
    created_at: String,
    chunk_count: usize,
}

impl App {
    fn new(database: ChonkerDatabase) -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        
        Self {
            should_quit: false,
            current_tab: 0,
            database,
            documents: Vec::new(),
            selected_document: None,
            list_state,
            status_message: "Welcome to CHONKER TUI! Press 'q' to quit.".to_string(),
        }
    }
    
    async fn load_documents(&mut self) -> Result<()> {
        let docs = self.database.get_all_documents().await?;
        self.documents = docs.into_iter().map(|doc| DocumentItem {
            id: doc.id,
            filename: doc.filename,
            created_at: doc.created_at,
            chunk_count: 0, // We'll load this separately if needed
        }).collect();
        
        if !self.documents.is_empty() && self.list_state.selected().is_none() {
            self.list_state.select(Some(0));
        }
        
        Ok(())
    }
    
    fn next_tab(&mut self) {
        self.current_tab = (self.current_tab + 1) % 4;
    }
    
    fn previous_tab(&mut self) {
        if self.current_tab > 0 {
            self.current_tab -= 1;
        } else {
            self.current_tab = 3;
        }
    }
    
    fn next_document(&mut self) {
        if !self.documents.is_empty() {
            let i = match self.list_state.selected() {
                Some(i) => {
                    if i >= self.documents.len() - 1 {
                        0
                    } else {
                        i + 1
                    }
                }
                None => 0,
            };
            self.list_state.select(Some(i));
        }
    }
    
    fn previous_document(&mut self) {
        if !self.documents.is_empty() {
            let i = match self.list_state.selected() {
                Some(i) => {
                    if i == 0 {
                        self.documents.len() - 1
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            self.list_state.select(Some(i));
        }
    }
    
    async fn delete_selected_document(&mut self) -> Result<()> {
        if let Some(selected) = self.list_state.selected() {
            if selected < self.documents.len() {
                let doc = &self.documents[selected];
                self.database.delete_document(&doc.id).await?;
                self.status_message = format!("Deleted document: {}", doc.filename);
                self.load_documents().await?;
                
                // Adjust selection
                if self.documents.is_empty() {
                    self.list_state.select(None);
                } else if selected >= self.documents.len() {
                    self.list_state.select(Some(self.documents.len() - 1));
                }
            }
        }
        Ok(())
    }
}

/// Restore terminal to normal state
fn restore_terminal() {
    let _ = disable_raw_mode();
    let _ = execute!(
        io::stdout(),
        LeaveAlternateScreen,
        DisableMouseCapture,
        crossterm::cursor::Show,
        crossterm::style::ResetColor
    );
    // Force flush to ensure all commands are executed
    let _ = io::stdout().flush();
}

/// Run the TUI application with simple, safe cleanup
pub async fn run_tui(database: ChonkerDatabase) -> Result<()> {
    info!("Starting CHONKER TUI");
    
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    // Create app state
    let mut app = App::new(database);
    app.load_documents().await?;
    
    // Run the main loop
    let result = run_app(&mut terminal, &mut app).await;
    
    // Simple cleanup - no fancy signal handling
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    
    // Handle any errors
    if let Err(err) = result {
        error!("TUI error: {:?}", err);
        return Err(err);
    }
    
    info!("TUI shut down successfully");
    Ok(())
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;
        
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => {
                    app.should_quit = true;
                }
                KeyCode::Tab => {
                    app.next_tab();
                }
                KeyCode::BackTab => {
                    app.previous_tab();
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    app.next_document();
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    app.previous_document();
                }
                KeyCode::Char('r') => {
                    app.load_documents().await?;
                    app.status_message = "Documents refreshed".to_string();
                }
                KeyCode::Char('d') => {
                    app.delete_selected_document().await?;
                }
                KeyCode::Char('e') | KeyCode::Char('E') => {
                    if app.current_tab == 1 { // Processing tab
                        app.status_message = "🔄 Extracting current page... (Feature coming soon!)".to_string();
                    }
                }
                KeyCode::Char('p') | KeyCode::Char('P') => {
                    if app.current_tab == 1 { // Processing tab
                        app.status_message = "◀️ Previous page (Feature coming soon!)".to_string();
                    }
                }
                KeyCode::Char('n') | KeyCode::Char('N') => {
                    if app.current_tab == 1 { // Processing tab
                        app.status_message = "▶️ Next page (Feature coming soon!)".to_string();
                    }
                }
                _ => {}
            }
        }
        
        if app.should_quit {
            break;
        }
    }
    Ok(())
}

fn ui(f: &mut Frame, app: &App) {
    let size = f.size();
    
    // Create layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Tabs
            Constraint::Min(0),    // Content
            Constraint::Length(3), // Status
        ])
        .split(size);
    
    // Render tabs with black borders and simple mascot
    let tab_titles = vec!["■ Documents ■", "■ Processing ■", "■ Export ■", "■ Settings ■"];
    let mascot_title = "<\\___/> [o-·-o] (\")~(\") CHONKER TUI";
    let tabs = Tabs::new(tab_titles)
        .block(Block::default().borders(Borders::ALL).title(mascot_title).border_style(Style::default().fg(Color::Black)))
        .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
        .highlight_style(Style::default().fg(Color::Magenta).bg(Color::White).add_modifier(Modifier::BOLD))
        .select(app.current_tab);
    f.render_widget(tabs, chunks[0]);
    
    // Render content based on current tab
    match app.current_tab {
        0 => render_documents_tab(f, app, chunks[1]),
        1 => render_processing_tab(f, app, chunks[1]),
        2 => render_export_tab(f, app, chunks[1]),
        3 => render_settings_tab(f, app, chunks[1]),
        _ => {}
    }
    
    // Render status bar with black borders
    let status = Paragraph::new(app.status_message.as_str())
        .block(Block::default().borders(Borders::ALL).title("■ Status ■").border_style(Style::default().fg(Color::Black)))
        .style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD));
    f.render_widget(status, chunks[2]);
}

fn render_documents_tab(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);
    
    // Document list
    let items: Vec<ListItem> = app
        .documents
        .iter()
        .map(|doc| {
            let content = Line::from(vec![
                Span::styled(&doc.filename, Style::default().fg(Color::White)),
                Span::styled(format!(" ({})", doc.created_at), Style::default().fg(Color::Gray)),
            ]);
            ListItem::new(content)
        })
        .collect();
    
    let documents_list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("■ Documents ■").border_style(Style::default().fg(Color::Black)))
        .highlight_style(Style::default().bg(Color::Magenta).fg(Color::Black).add_modifier(Modifier::BOLD))
        .highlight_symbol("▶ ");
    
    f.render_stateful_widget(documents_list, chunks[0], &mut app.list_state.clone());
    
    // Document details
    let details_text = if let Some(selected) = app.list_state.selected() {
        if selected < app.documents.len() {
            let doc = &app.documents[selected];
            format!(
                "ID: {}\nFilename: {}\nCreated: {}\nChunks: {}",
                doc.id, doc.filename, doc.created_at, doc.chunk_count
            )
        } else {
            "No document selected".to_string()
        }
    } else {
        "No documents available".to_string()
    };
    
    let details = Paragraph::new(details_text)
        .block(Block::default().borders(Borders::ALL).title("■ Details ■").border_style(Style::default().fg(Color::Black)))
        .style(Style::default().fg(Color::Green));
    f.render_widget(details, chunks[1]);
}

fn render_processing_tab(f: &mut Frame, _app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6),  // PDF Info
            Constraint::Length(4),  // Controls
            Constraint::Min(0),     // Processing log
        ])
        .split(area);
    
    // PDF Info Section
    let pdf_info = Paragraph::new("■ PDF Information:\n\n▶ Current PDF: /Users/jack/Documents/1.pdf\n▶ Page: 1 of 1\n▶ Status: Ready for extraction")
        .block(Block::default().borders(Borders::ALL).title("■ PDF Viewer ■").border_style(Style::default().fg(Color::Black)))
        .style(Style::default().fg(Color::Magenta));
    f.render_widget(pdf_info, chunks[0]);
    
    // Controls Section
    let controls = Paragraph::new("▶ Controls: [E] Extract Current Page  [P] Previous  [N] Next  [R] Reload")
        .block(Block::default().borders(Borders::ALL).title("■ Controls ■").border_style(Style::default().fg(Color::Black)))
        .style(Style::default().fg(Color::Green));
    f.render_widget(controls, chunks[1]);
    
    // Processing Log
    let log = Paragraph::new("■ Processing Log:\n\n▶ Ready to extract PDF content\n▶ Python bridge: Available\n▶ Press 'E' to extract current page")
        .block(Block::default().borders(Borders::ALL).title("■ Processing Log ■").border_style(Style::default().fg(Color::Black)))
        .style(Style::default().fg(Color::Magenta));
    f.render_widget(log, chunks[2]);
}

fn render_export_tab(f: &mut Frame, _app: &App, area: Rect) {
    let content = Paragraph::new("■ Export functionality will be implemented here.\n\n▶ Formats:\n- CSV\n- JSON\n- Parquet\n\n▶ Filters and queries coming soon!")
        .block(Block::default().borders(Borders::ALL).title("■ Export ■").border_style(Style::default().fg(Color::Black)))
        .style(Style::default().fg(Color::Green));
    f.render_widget(content, area);
}

fn render_settings_tab(f: &mut Frame, _app: &App, area: Rect) {
    let content = Paragraph::new("■ Settings and configuration.\n\n▶ Keys:\n- q: Quit\n- Tab: Next tab\n- ↑/↓: Navigate\n- r: Refresh\n- d: Delete")
        .block(Block::default().borders(Borders::ALL).title("■ Settings & Help ■").border_style(Style::default().fg(Color::Black)))
        .style(Style::default().fg(Color::Magenta));
    f.render_widget(content, area);
}
