// Event Handling for Document Surgery Dashboard
// Clean, action-based event system

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum DashboardAction {
    // Navigation
    Quit,
    CycleFocus,
    MoveUp,
    MoveDown,
    
    // Document operations
    SelectDocument,
    RefreshDocuments,
    DeleteDocument,
    
    // Processing operations
    StartExtraction,
    StartProcessing,
    StartExport,
    
    // Special operations
    ShowHelp,
    ToggleCommandPalette,
    
    // Internal
    Update, // For background updates
}

pub struct EventHandler {
    // Future: Could add command palette state, key sequence tracking, etc.
}

impl EventHandler {
    pub fn new() -> Self {
        Self {}
    }
    
    pub async fn handle_events(&mut self) -> Result<Option<DashboardAction>> {
        // Non-blocking event polling with small timeout
        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key_event) => {
                    return Ok(self.handle_key_event(key_event));
                }
                Event::Resize(_, _) => {
                    // Handle terminal resize - could trigger layout recalculation
                    return Ok(Some(DashboardAction::Update));
                }
                _ => {}
            }
        }
        
        // Return update action for background processing
        Ok(Some(DashboardAction::Update))
    }
    
    fn handle_key_event(&self, key_event: crossterm::event::KeyEvent) -> Option<DashboardAction> {
        match key_event.code {
            // Global shortcuts
            KeyCode::Char('q') => Some(DashboardAction::Quit),
            KeyCode::Tab => Some(DashboardAction::CycleFocus),
            KeyCode::Char('?') => Some(DashboardAction::ShowHelp),
            
            // Navigation
            KeyCode::Up | KeyCode::Char('k') => Some(DashboardAction::MoveUp),
            KeyCode::Down | KeyCode::Char('j') => Some(DashboardAction::MoveDown),
            KeyCode::Enter => Some(DashboardAction::SelectDocument),
            
            // Document operations
            KeyCode::Char('r') => Some(DashboardAction::RefreshDocuments),
            KeyCode::Char('d') => Some(DashboardAction::DeleteDocument),
            
            // Processing operations
            KeyCode::Char('e') => Some(DashboardAction::StartExtraction),
            KeyCode::Char('p') => Some(DashboardAction::StartProcessing),
            KeyCode::Char('x') => Some(DashboardAction::StartExport),
            
            // Command palette (future feature)
            KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                Some(DashboardAction::ToggleCommandPalette)
            }
            
            _ => None,
        }
    }
}
