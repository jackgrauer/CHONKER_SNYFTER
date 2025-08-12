use std::path::PathBuf;
use std::fs;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect, Alignment, Margin},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, List, ListItem, Paragraph},
};

#[derive(Debug, Clone)]
pub struct FileSelector {
    pub current_path: PathBuf,
    pub entries: Vec<FileEntry>,
    pub selected_index: usize,
    pub active: bool,
    filter_extension: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub is_pdf: bool,
}

impl FileSelector {
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
        let mut selector = Self {
            current_path: home,
            entries: Vec::new(),
            selected_index: 0,
            active: false,
            filter_extension: Some("pdf".to_string()),
        };
        selector.refresh_entries();
        selector
    }
    
    pub fn activate(&mut self) {
        self.active = true;
        self.refresh_entries();
    }
    
    pub fn deactivate(&mut self) {
        self.active = false;
    }
    
    pub fn navigate_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }
    
    pub fn navigate_down(&mut self) {
        if self.selected_index < self.entries.len().saturating_sub(1) {
            self.selected_index += 1;
        }
    }
    
    pub fn enter_directory(&mut self) -> Option<PathBuf> {
        if let Some(entry) = self.entries.get(self.selected_index) {
            if entry.is_dir {
                self.current_path = entry.path.clone();
                self.selected_index = 0;
                self.refresh_entries();
                None
            } else if entry.is_pdf {
                Some(entry.path.clone())
            } else {
                None
            }
        } else {
            None
        }
    }
    
    pub fn handle_click(&mut self, row: usize, file_list_start: usize) -> Option<PathBuf> {
        // Convert screen row to list index
        let clicked_index = row.saturating_sub(file_list_start);
        
        if clicked_index < self.entries.len() {
            // Select the clicked item
            self.selected_index = clicked_index;
            
            // Try to open/enter the selected item
            return self.enter_directory();
        }
        
        None
    }
    
    pub fn go_up_directory(&mut self) {
        if let Some(parent) = self.current_path.parent() {
            self.current_path = parent.to_path_buf();
            self.selected_index = 0;
            self.refresh_entries();
        }
    }
    
    fn refresh_entries(&mut self) {
        self.entries.clear();
        
        // Add parent directory option
        if self.current_path.parent().is_some() {
            self.entries.push(FileEntry {
                name: "📁 ..".to_string(),
                path: self.current_path.parent().unwrap().to_path_buf(),
                is_dir: true,
                is_pdf: false,
            });
        }
        
        // Read directory entries
        if let Ok(read_dir) = fs::read_dir(&self.current_path) {
            let mut dirs = Vec::new();
            let mut files = Vec::new();
            
            for entry in read_dir.flatten() {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();
                
                // Skip hidden files
                if name.starts_with('.') {
                    continue;
                }
                
                let is_dir = path.is_dir();
                let is_pdf = !is_dir && path.extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext.eq_ignore_ascii_case("pdf"))
                    .unwrap_or(false);
                
                // Apply filter
                if !is_dir && self.filter_extension.is_some() && !is_pdf {
                    continue;
                }
                
                let display_name = if is_dir {
                    format!("📁 {}", name)
                } else if is_pdf {
                    // Get file size for PDFs
                    let size = entry.metadata().ok()
                        .map(|m| m.len())
                        .map(|bytes| {
                            if bytes < 1024 {
                                format!("{}B", bytes)
                            } else if bytes < 1024 * 1024 {
                                format!("{}KB", bytes / 1024)
                            } else {
                                format!("{}MB", bytes / (1024 * 1024))
                            }
                        })
                        .unwrap_or_else(|| "?".to_string());
                    format!("📄 {} ({})", name, size)
                } else {
                    format!("   {}", name)
                };
                
                let entry = FileEntry {
                    name: display_name,
                    path,
                    is_dir,
                    is_pdf,
                };
                
                if is_dir {
                    dirs.push(entry);
                } else {
                    files.push(entry);
                }
            }
            
            // Sort directories and files separately
            dirs.sort_by(|a, b| a.name.cmp(&b.name));
            files.sort_by(|a, b| a.name.cmp(&b.name));
            
            // Add sorted entries
            self.entries.extend(dirs);
            self.entries.extend(files);
        }
        
        // Reset selection if out of bounds
        if self.selected_index >= self.entries.len() && !self.entries.is_empty() {
            self.selected_index = self.entries.len() - 1;
        }
    }
    
    pub fn render(&self, frame: &mut Frame, area: Rect) -> (usize, usize) {
        // Return the start row of the file list for click handling
        if !self.active {
            return (0, 0);
        }
        
        // Use the entire screen for file selection with solid black background
        let dialog_bg = Color::Black;  // Solid black to prevent bleed-through
        let dialog_block = Block::default()
            .style(Style::default().bg(dialog_bg));
        frame.render_widget(dialog_block, area);
        
        // Split screen into header, content, and footer
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Header
                Constraint::Min(1),     // File list
                Constraint::Length(3),  // Footer with more space
            ])
            .split(area);
        
        // Render header with current path
        let header_bg = Color::Rgb(20, 20, 20);  // Darker header
        let header_fg = Color::Rgb(200, 200, 255);
        
        let path_display = self.current_path.display().to_string();
        let header_text = format!(" 📂 Select PDF File - {}", 
            if path_display.len() > area.width as usize - 25 {
                format!("...{}", &path_display[path_display.len() - (area.width as usize - 30)..])
            } else {
                path_display
            }
        );
        
        let header = Paragraph::new(header_text)
            .style(Style::default().bg(header_bg).fg(header_fg))
            .alignment(Alignment::Left);
        frame.render_widget(header, chunks[0]);
        
        // Render file list with custom highlighting
        let items: Vec<ListItem> = self.entries
            .iter()
            .enumerate()
            .map(|(i, entry)| {
                let (fg, bg, modifiers) = if i == self.selected_index {
                    // Selected item - bright highlight with solid background
                    (
                        Color::Rgb(255, 255, 200),
                        Color::Rgb(40, 40, 50),  // Darker, more opaque
                        Modifier::empty(),
                    )
                } else if entry.is_pdf {
                    // PDF files - slightly highlighted
                    (
                        Color::Rgb(150, 200, 255),
                        Color::Black,  // Use solid black
                        Modifier::empty(),
                    )
                } else if entry.is_dir {
                    // Directories - normal
                    (
                        Color::Rgb(180, 180, 200),
                        Color::Black,  // Use solid black
                        Modifier::empty(),
                    )
                } else {
                    // Other files - dimmed
                    (
                        Color::Rgb(120, 120, 140),
                        Color::Black,  // Use solid black
                        Modifier::empty(),
                    )
                };
                
                // Create line with full width background
                let content = format!(" {:<width$}", entry.name, width = area.width as usize - 4);
                ListItem::new(Line::from(vec![
                    Span::styled(content, Style::default().fg(fg).bg(bg).add_modifier(modifiers))
                ]))
            })
            .collect();
        
        let list = List::new(items)
            .style(Style::default().bg(dialog_bg));
        
        // Add padding to the list area
        let list_area = chunks[1].inner(Margin { vertical: 0, horizontal: 1 });
        frame.render_widget(list, list_area);
        
        // Return the start position of the file list for click handling
        let file_list_start = chunks[1].y as usize;
        let file_list_height = chunks[1].height as usize;
        
        // Render footer with controls - make it more prominent
        let footer_bg = Color::Rgb(25, 28, 34);
        let footer_fg = Color::Rgb(200, 200, 255);
        
        let footer_lines = vec![
            Line::from(vec![
                Span::styled("🎯 ", Style::default().fg(Color::Rgb(100, 150, 255))),
                Span::styled("PDF File Selector", Style::default().fg(footer_fg)),
            ]),
            Line::from(vec![
                Span::styled("⌨️  ", Style::default().fg(Color::Rgb(100, 200, 100))),
                Span::styled("↑↓ Navigate • Enter: Open • Backspace: Parent Dir • Esc: Cancel", Style::default().fg(footer_fg)),
            ]),
        ];
        
        let footer = Paragraph::new(footer_lines)
            .style(Style::default().bg(footer_bg))
            .alignment(Alignment::Center);
        frame.render_widget(footer, chunks[2]);
        
        // Return file list bounds for click detection
        (file_list_start, file_list_height)
    }
}