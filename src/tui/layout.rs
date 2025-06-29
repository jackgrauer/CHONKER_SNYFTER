// Dashboard Layout System
// Professional three-pane layout with responsive design

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
};

use super::state::FocusArea;

/// Warp Terminal color scheme
pub struct WarpColors;

impl WarpColors {
    pub const BACKGROUND: Color = Color::Rgb(16, 16, 16);
    pub const SURFACE: Color = Color::Rgb(32, 32, 32);
    pub const BORDER: Color = Color::Rgb(64, 64, 64);
    pub const BORDER_FOCUSED: Color = Color::Rgb(58, 128, 200);
    pub const TEXT_PRIMARY: Color = Color::Rgb(240, 240, 240);
    pub const TEXT_SECONDARY: Color = Color::Rgb(180, 180, 180);
    pub const TEXT_MUTED: Color = Color::Rgb(120, 120, 120);
    pub const ACCENT_BLUE: Color = Color::Rgb(58, 128, 200);
    pub const ACCENT_GREEN: Color = Color::Rgb(120, 180, 120);
    pub const ACCENT_YELLOW: Color = Color::Rgb(200, 160, 58);
    pub const ACCENT_RED: Color = Color::Rgb(200, 80, 80);
    pub const STATUS_SUCCESS: Color = Color::Rgb(140, 200, 140);
    pub const STATUS_WARNING: Color = Color::Rgb(200, 180, 100);
    pub const STATUS_ERROR: Color = Color::Rgb(200, 100, 100);
}

/// Dashboard layout areas
#[derive(Debug)]
pub struct DashboardLayout {
    pub document_library: Rect,
    pub work_area: Rect,
    pub pipeline_status: Rect,
    pub status_bar: Rect,
    pub help_bar: Rect,
}

impl DashboardLayout {
    /// Create the main dashboard layout
    pub fn new(area: Rect) -> Self {
        // Main vertical split: content + status + help
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Min(0),    // Main content area
                Constraint::Length(3), // Status bar
                Constraint::Length(3), // Help bar
            ])
            .split(area);

        // Three-pane horizontal layout for main content
        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25), // Document Library
                Constraint::Percentage(50), // Work Area
                Constraint::Percentage(25), // Pipeline Status
            ])
            .split(main_chunks[0]);

        Self {
            document_library: content_chunks[0],
            work_area: content_chunks[1],
            pipeline_status: content_chunks[2],
            status_bar: main_chunks[1],
            help_bar: main_chunks[2],
        }
    }
    
    /// Get border style for a pane based on focus
    pub fn border_style(&self, pane: FocusArea, current_focus: FocusArea) -> Style {
        if pane == current_focus {
            Style::default().fg(WarpColors::BORDER_FOCUSED)
        } else {
            Style::default().fg(WarpColors::BORDER)
        }
    }
}

/// Work area sub-layouts for different content types
pub struct WorkAreaLayout {
    pub preview: Rect,
    pub controls: Rect,
    pub results: Rect,
}

impl WorkAreaLayout {
    pub fn new(area: Rect) -> Self {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Percentage(40), // Document preview
                Constraint::Length(6),      // Processing controls
                Constraint::Min(0),         // Results area
            ])
            .split(area);

        Self {
            preview: chunks[0],
            controls: chunks[1],
            results: chunks[2],
        }
    }
}

/// Pipeline status area layout
pub struct PipelineLayout {
    pub progress: Rect,
    pub logs: Rect,
    pub export_options: Rect,
}

impl PipelineLayout {
    pub fn new(area: Rect) -> Self {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(8),  // Progress indicator
                Constraint::Min(0),     // Processing logs
                Constraint::Length(6),  // Export options
            ])
            .split(area);

        Self {
            progress: chunks[0],
            logs: chunks[1],
            export_options: chunks[2],
        }
    }
}

/// Responsive layout utilities
pub struct ResponsiveLayout;

impl ResponsiveLayout {
    /// Check if terminal is too small for full dashboard
    pub fn is_compact_mode(area: Rect) -> bool {
        area.width < 120 || area.height < 30
    }
    
    /// Get compact layout for smaller terminals
    pub fn compact_layout(area: Rect) -> DashboardLayout {
        // Stack vertically instead of horizontally
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Min(0),    // Main content
                Constraint::Length(3), // Status
                Constraint::Length(3), // Help
            ])
            .split(area);

        let content_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(40), // Document Library
                Constraint::Percentage(60), // Work Area (Pipeline collapsed)
            ])
            .split(main_chunks[0]);

        DashboardLayout {
            document_library: content_chunks[0],
            work_area: content_chunks[1],
            pipeline_status: Rect::default(), // Hidden in compact mode
            status_bar: main_chunks[1],
            help_bar: main_chunks[2],
        }
    }
}
