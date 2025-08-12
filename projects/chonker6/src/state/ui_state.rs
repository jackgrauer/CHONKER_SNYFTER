use crate::actions::Panel;

#[derive(Debug, Clone)]
pub struct UiState {
    pub focused_panel: Panel,
    pub split_ratio: u8,  // Percentage for left panel
    pub show_line_numbers: bool,
    pub theme: Theme,
    pub terminal_panel: TerminalPanelState,
}

#[derive(Debug, Clone)]
pub struct TerminalPanelState {
    pub visible: bool,
    pub height: u16,  // Number of lines for terminal output
    pub content: Vec<String>,  // Terminal output history
    pub scroll_offset: usize,  // For scrolling through history
    pub selected_lines: Option<(usize, usize)>,  // For text selection in terminal
}

#[derive(Debug, Clone, Copy)]
pub enum Theme {
    Dark,
    Light,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            focused_panel: Panel::Pdf,
            split_ratio: 50,
            show_line_numbers: true,
            theme: Theme::Dark,
            terminal_panel: TerminalPanelState::default(),
        }
    }
}

impl Default for TerminalPanelState {
    fn default() -> Self {
        Self {
            visible: false,
            height: 8,  // Default 8 lines
            content: vec!["Terminal Output Ready".to_string()],
            scroll_offset: 0,
            selected_lines: None,
        }
    }
}

impl TerminalPanelState {
    pub fn add_output(&mut self, text: String) {
        self.content.push(text);
        // Keep only last 1000 lines
        if self.content.len() > 1000 {
            self.content.remove(0);
        }
        // Auto-scroll to latest ONLY if already at bottom
        let was_at_bottom = self.scroll_offset >= self.content.len().saturating_sub(self.height as usize + 1);
        if was_at_bottom {
            self.scroll_offset = self.content.len().saturating_sub(self.height as usize);
        }
    }
    
    pub fn clear(&mut self) {
        self.content.clear();
        self.content.push("Terminal Output Cleared".to_string());
        self.scroll_offset = 0;
    }
    
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }
    
    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }
    
    pub fn scroll_down(&mut self) {
        let max_scroll = self.content.len().saturating_sub(self.height as usize);
        if self.scroll_offset < max_scroll {
            self.scroll_offset += 1;
        }
    }
    
    pub fn get_selected_text(&self) -> Option<String> {
        if let Some((start, end)) = self.selected_lines {
            if self.content.is_empty() {
                return None;
            }
            
            // Ensure indices are within bounds
            let max_idx = self.content.len().saturating_sub(1);
            let start = start.min(max_idx);
            let end = end.min(max_idx);
            
            // Ensure start <= end for valid slice
            let (start, end) = if start > end {
                (end, start)
            } else {
                (start, end)
            };
            
            let lines: Vec<String> = self.content[start..=end].to_vec();
            Some(lines.join("\n"))
        } else {
            None
        }
    }
}