// Ultra-Minimal TUI for Chonker - Clean blocks only
// Only the hamster emoji, no borders, just solid color blocks
//
// Add to Cargo.toml:
// [[bin]]
// name = "minimal_tui"
// path = "minimal_tui.rs"

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::Line,
    widgets::{List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::{error::Error, io};

#[derive(PartialEq)]
enum View {
    Files,
    Process,
    Data,
}

struct App {
    view: View,
    selected: usize,
    processing: usize, // 0-4 stages
    document_loaded: bool,
    files: Vec<(&'static str, &'static str, &'static str)>, // (name, size, status)
}

impl App {
    fn new() -> App {
        App {
            view: View::Files,
            selected: 0,
            processing: 0,
            document_loaded: false,
            files: vec![
                ("report.pdf", "2.3M", "DONE"),
                ("analysis.docx", "890K", "PROCESSING"),
                ("data.xlsx", "1.2M", "NEW"),
                ("notes.pdf", "567K", "ERROR"),
            ],
        }
    }

    fn handle_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('1') => self.view = View::Files,
            KeyCode::Char('2') => self.view = View::Process,
            KeyCode::Char('3') if self.document_loaded => self.view = View::Data,
            KeyCode::Up if self.view == View::Files && self.selected > 0 => {
                self.selected -= 1;
            }
            KeyCode::Down if self.view == View::Files && self.selected < self.files.len() - 1 => {
                self.selected += 1;
            }
            KeyCode::Enter if self.view == View::Files => {
                self.view = View::Process;
                self.processing = 1;
            }
            KeyCode::Char(' ') if self.view == View::Process => {
                if self.processing < 4 {
                    self.processing += 1;
                    if self.processing == 4 {
                        self.document_loaded = true;
                    }
                }
            }
            KeyCode::Char('q') => std::process::exit(0),
            _ => {}
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0), Constraint::Length(1)])
        .split(f.size());

    // Header - just title and tabs
    let header_text = format!(
        "🐹 CHONKER    Files{}Process{}Data{}",
        if app.view == View::Files { " [X] " } else { "  " },
        if app.view == View::Process { " [X] " } else { "  " },
        if app.view == View::Data && app.document_loaded { " [X]" } else { "" },
    );
    let header = Paragraph::new(header_text).style(Style::default().fg(Color::White));
    f.render_widget(header, chunks[0]);

    // Main content
    match app.view {
        View::Files => render_files(f, chunks[1], app),
        View::Process => render_process(f, chunks[1], app),
        View::Data => render_data(f, chunks[1], app),
    }

    // Status
    let status_text = match app.view {
        View::Files => "Up/Down: Navigate  Enter: Process  1/2/3: Switch  q: Quit",
        View::Process => "Space: Advance  1/2/3: Switch  q: Quit",
        View::Data => "1/2/3: Switch  q: Quit",
    };
    let status = Paragraph::new(status_text).style(Style::default().fg(Color::Gray));
    f.render_widget(status, chunks[2]);
}

fn render_files(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(area);

    // File list - pure solid color blocks
    let items: Vec<ListItem> = app.files
        .iter()
        .enumerate()
        .map(|(i, (name, size, status))| {
            let line = format!("{} {} {}", status, name, size);
            let style = if i == app.selected {
                Style::default().bg(Color::Blue).fg(Color::White)
            } else {
                match *status {
                    "ERROR" => Style::default().fg(Color::Red).bg(Color::Black),
                    "PROCESSING" => Style::default().fg(Color::Yellow).bg(Color::Black),
                    "DONE" => Style::default().fg(Color::Green).bg(Color::Black),
                    _ => Style::default().fg(Color::Gray).bg(Color::Black),
                }
            };
            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items).style(Style::default().bg(Color::Black));
    f.render_widget(list, chunks[0]);

    // Details - solid color block
    let details = if app.selected < app.files.len() {
        match app.files[app.selected].2 {
            "NEW" => "Ready to process\n\nPress Enter to start",
            "PROCESSING" => "Currently processing\n\nPlease wait...",
            "DONE" => "Processing complete\n\nPress Enter to view",
            "ERROR" => "Processing failed\n\nTry again later",
            _ => "Unknown status",
        }
    } else {
        "No file selected"
    };

    let panel = Paragraph::new(details).style(Style::default().fg(Color::White).bg(Color::DarkGray));
    f.render_widget(panel, chunks[1]);
}

fn render_process(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let lines = vec![
        Line::from("Pipeline"),
        Line::from(""),
        Line::from(if app.processing >= 1 { "✓ EXTRACT" } else { "○ extract" }),
        Line::from(if app.processing >= 2 { "✓ PROCESS" } else { "○ process" }),
        Line::from(if app.processing >= 3 { "✓ ANALYZE" } else { "○ analyze" }),
        Line::from(if app.processing >= 4 { "✓ EXPORT" } else { "○ export" }),
        Line::from(""),
        Line::from(format!("Progress: {}/4", app.processing)),
        Line::from(""),
        Line::from("Press SPACE to advance"),
    ];

    let panel = Paragraph::new(lines).style(Style::default().fg(Color::White).bg(Color::Black));
    f.render_widget(panel, area);
}

fn render_data(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    if !app.document_loaded {
        let panel = Paragraph::new("Process a document first")
            .style(Style::default().fg(Color::Gray).bg(Color::Black));
        f.render_widget(panel, area);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Original - solid blue block
    let original = Paragraph::new("ENVIRONMENTAL REPORT\n\nStation A pH: 7.2 Normal\nStation B pH: 6.8 Warning\nStation C pH: 7.1 Normal\n\nContact: admin@test.gov")
        .style(Style::default().fg(Color::White).bg(Color::Blue));
    f.render_widget(original, chunks[0]);

    // Extracted - solid dark block with colored text
    let extracted_lines = vec![
        Line::from("# Environmental Report"),
        Line::from(""),
        Line::from("Station A: pH 7.2 Normal"),
        Line::from("Station B: pH 6.8 Warning"),  
        Line::from("Station C: pH 7.1 Normal"),
        Line::from(""),
        Line::from("Contact: admin@test.gov"),
    ];

    let extracted = Paragraph::new(extracted_lines)
        .style(Style::default().fg(Color::White).bg(Color::DarkGray));
    f.render_widget(extracted, chunks[1]);
}

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    loop {
        terminal.draw(|f| ui(f, &app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == crossterm::event::KeyEventKind::Press {
                app.handle_key(key.code);
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
