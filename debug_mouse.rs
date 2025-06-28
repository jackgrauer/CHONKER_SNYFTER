use crossterm::{
    event::{self, Event, KeyCode, MouseEventKind, MouseButton},
    execute,
    terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use crossterm::event::{EnableMouseCapture, DisableMouseCapture};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph},
    Terminal, Frame,
};
use std::io;

struct DebugApp {
    mouse_position: Option<(u16, u16)>,
    click_position: Option<(u16, u16)>,
    drag_position: Option<(u16, u16)>,
    is_dragging: bool,
    event_log: Vec<String>,
    clicked_pane: Option<String>,
}

impl DebugApp {
    fn new() -> Self {
        Self {
            mouse_position: None,
            click_position: None,
            drag_position: None,
            is_dragging: false,
            event_log: Vec::new(),
            clicked_pane: None,
        }
    }

    fn handle_mouse(&mut self, mouse: crossterm::event::MouseEvent) {
        self.mouse_position = Some((mouse.column, mouse.row));
        
        // Log the event
        let event_msg = format!("Mouse: {:?} at ({}, {})", mouse.kind, mouse.column, mouse.row);
        self.event_log.push(event_msg);
        
        // Keep only last 10 events
        if self.event_log.len() > 10 {
            self.event_log.remove(0);
        }
        
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                self.click_position = Some((mouse.column, mouse.row));
                self.is_dragging = true;
                
                // Determine which pane was clicked
                self.clicked_pane = Some(self.get_pane_at_coordinates(mouse.column, mouse.row));
            }
            MouseEventKind::Drag(MouseButton::Left) => {
                if self.is_dragging {
                    self.drag_position = Some((mouse.column, mouse.row));
                }
            }
            MouseEventKind::Up(MouseButton::Left) => {
                self.is_dragging = false;
                self.drag_position = None;
            }
            _ => {}
        }
    }

    fn get_pane_at_coordinates(&self, x: u16, y: u16) -> String {
        // Assuming 50% split at column 40 (for typical 80-col terminal)
        if x < 40 {
            "LEFT PANE (PDF)".to_string()
        } else {
            "RIGHT PANE (MARKDOWN)".to_string()
        }
    }

    fn render(&self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(frame.size());

        // Left pane
        let left_content = vec![
            "🐹 CHONKER MOUSE DEBUG".to_string(),
            "".to_string(),
            "This is the LEFT PANE".to_string(),
            "Try clicking and dragging here!".to_string(),
            "".to_string(),
            format!("Mouse at: {:?}", self.mouse_position),
            format!("Last click: {:?}", self.click_position),
            format!("Dragging: {}", self.is_dragging),
            format!("Drag pos: {:?}", self.drag_position),
            format!("Clicked pane: {:?}", self.clicked_pane),
        ];

        let left_paragraph = Paragraph::new(left_content.join("\n"))
            .block(Block::default().borders(Borders::ALL).title("LEFT PANE"));
        frame.render_widget(left_paragraph, chunks[0]);

        // Right pane with event log
        let mut right_content = vec![
            "📝 EVENT LOG".to_string(),
            "".to_string(),
        ];
        right_content.extend(self.event_log.clone());
        
        let right_paragraph = Paragraph::new(right_content.join("\n"))
            .block(Block::default().borders(Borders::ALL).title("RIGHT PANE"));
        frame.render_widget(right_paragraph, chunks[1]);

        // Bottom instructions
        let instructions = Paragraph::new("Press 'q' to quit | Try clicking and dragging in either pane")
            .block(Block::default().borders(Borders::ALL).title("Instructions"));
        
        let bottom_area = Rect {
            x: 0,
            y: frame.size().height.saturating_sub(3),
            width: frame.size().width,
            height: 3,
        };
        frame.render_widget(instructions, bottom_area);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture
    )?;
    
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    let mut app = DebugApp::new();
    
    // Main loop
    loop {
        terminal.draw(|f| app.render(f))?;
        
        match event::read()? {
            Event::Mouse(mouse) => {
                app.handle_mouse(mouse);
            }
            Event::Key(key) => {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
            _ => {}
        }
    }
    
    // Cleanup
    execute!(
        terminal.backend_mut(),
        DisableMouseCapture,
        LeaveAlternateScreen,
    )?;
    disable_raw_mode()?;
    
    Ok(())
}
