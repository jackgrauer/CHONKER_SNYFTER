use std::path::PathBuf;
use std::fs;
use anyhow::Result;
use std::io::{stdout, Write};

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub is_pdf: bool,
}

pub struct FileSelectorMatrix {
    pub current_path: PathBuf,
    pub entries: Vec<FileEntry>,
    pub selected_index: usize,
    pub active: bool,
    filter_extension: Option<String>,
}

impl FileSelectorMatrix {
    pub fn new() -> Self {
        let home = std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/Users/jack"));
        
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
    
    pub fn render(&self, width: u16, height: u16) -> Result<()> {
        if !self.active {
            return Ok(());
        }
        
        // Clear screen
        print!("\x1b[2J\x1b[H");
        
        // Draw border
        print!("┌");
        for _ in 0..width-2 {
            print!("─");
        }
        println!("┐");
        
        // Draw header
        let path_display = self.current_path.display().to_string();
        let truncated_path = if path_display.len() > width as usize - 10 {
            format!("...{}", &path_display[path_display.len() - (width as usize - 15)..])
        } else {
            path_display
        };
        
        println!("│ 📂 Select PDF: {} │", truncated_path);
        
        print!("├");
        for _ in 0..width-2 {
            print!("─");
        }
        println!("┤");
        
        // Calculate visible range
        let list_height = height.saturating_sub(6) as usize;
        let start_index = if self.selected_index >= list_height {
            self.selected_index - list_height + 1
        } else {
            0
        };
        let end_index = (start_index + list_height).min(self.entries.len());
        
        // Draw file list
        for i in start_index..end_index {
            print!("│ ");
            
            if i == self.selected_index {
                // Highlight selected item
                print!("\x1b[7m");
            }
            
            let entry = &self.entries[i];
            let mut display_name = entry.name.clone();
            
            // Truncate if too long
            let max_width = width as usize - 4;
            if display_name.len() > max_width {
                display_name.truncate(max_width - 3);
                display_name.push_str("...");
            }
            
            // Pad to full width
            print!("{:<width$}", display_name, width = max_width);
            
            if i == self.selected_index {
                print!("\x1b[0m");
            }
            
            println!(" │");
        }
        
        // Fill remaining space
        for _ in end_index - start_index..list_height {
            print!("│");
            for _ in 0..width-2 {
                print!(" ");
            }
            println!("│");
        }
        
        // Draw footer
        print!("├");
        for _ in 0..width-2 {
            print!("─");
        }
        println!("┤");
        
        println!("│ ↑↓ Navigate • Enter: Open • Backspace: Up • Esc: Cancel │");
        
        print!("└");
        for _ in 0..width-2 {
            print!("─");
        }
        println!("┘");
        
        stdout().flush()?;
        Ok(())
    }
}