mod actions;
mod state;
mod app;
mod components;
mod services;
mod kitty_graphics;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers, EnableMouseCapture, DisableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;
use log::info;

use crate::{
    app::App,
    actions::Action,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();
    info!("Starting Chonker6...");
    
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    // Create app
    let mut app = App::new();
    
    // Run app
    let result = run_app(&mut terminal, &mut app).await;
    
    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    
    if let Err(err) = result {
        eprintln!("Error: {}", err);
    }
    
    Ok(())
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<()> {
    loop {
        // Draw UI
        terminal.draw(|f| app.render(f))?;
        
        // After frame is drawn, render Kitty PDF only when needed (not every frame!)
        if app.should_render_kitty_pdf() {
            // Calculate the PDF panel area based on terminal size
            let size = terminal.size()?;
            let main_height = size.height.saturating_sub(1); // Remove status bar
            let pdf_width = size.width / 2; // 50% split
            let pdf_area = ratatui::layout::Rect {
                x: 0,
                y: 0,
                width: pdf_width,
                height: main_height,
            };
            app.render_pdf_with_kitty_post_frame(pdf_area);
        }
        
        // Handle events
        if event::poll(std::time::Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => {
                    // Clean event handling with pattern matching
                    let action = if key.code == KeyCode::Char('q') && key.modifiers == KeyModifiers::CONTROL {
                        Some(Action::Quit)
                    } else if key.code == KeyCode::Char('h') && key.modifiers == KeyModifiers::CONTROL {
                        Some(Action::ShowHelp)
                    } else if key.code == KeyCode::Esc && app.is_help_shown() {
                        Some(Action::HideHelp)
                    } else {
                        app.handle_key(key)
                    };
                    
                    if let Some(action) = action {
                        app.dispatch(action).await?;
                        
                        if !app.is_running() {
                            break;
                        }
                    }
                }
                Event::Mouse(mouse) => {
                    // Handle mouse events
                    if let Some(action) = app.handle_mouse(mouse) {
                        app.dispatch(action).await?;
                    }
                }
                _ => {} // Ignore other events like resize
            }
        }
        
        // Process any async tasks
        app.tick().await?;
    }
    
    Ok(())
}