// Document Surgery Dashboard - Main TUI Module
// Modern, professional interface for document processing

pub mod app;
pub mod components;
pub mod layout;
pub mod state;
pub mod events;

use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::io;
use tracing::{info, error};

use crate::database::ChonkerDatabase;
use app::DashboardApp;
use events::EventHandler;

/// Entry point for the Document Surgery Dashboard
pub async fn run_dashboard(database: ChonkerDatabase) -> Result<()> {
    info!("Starting CHONKER Document Surgery Dashboard");
    
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    // Create dashboard app
    let mut app = DashboardApp::new(database).await?;
    let mut event_handler = EventHandler::new();
    
    // Run the dashboard
    let result = run_dashboard_loop(&mut terminal, &mut app, &mut event_handler).await;
    
    // Cleanup
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    
    if let Err(err) = result {
        error!("Dashboard error: {:?}", err);
        return Err(err);
    }
    
    info!("Dashboard shut down successfully");
    Ok(())
}

async fn run_dashboard_loop<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut DashboardApp,
    event_handler: &mut EventHandler,
) -> Result<()> {
    loop {
        // Render the dashboard
        terminal.draw(|frame| {
            app.render(frame);
        })?;
        
        // Handle events
        if let Some(action) = event_handler.handle_events().await? {
            if app.handle_action(action).await? {
                break; // App requested exit
            }
        }
        
        // Update any ongoing operations
        app.update().await?;
    }
    
    Ok(())
}
