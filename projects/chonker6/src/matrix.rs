use std::collections::HashMap;
use anyhow::Result;

/// A single character with its position in the spatial grid
#[derive(Debug, Clone)]
pub struct CharInfo {
    pub x: f32,
    pub y: f32,
    pub ch: char,
    pub font_size: f32,
}

/// Rectangle for selections
#[derive(Debug, Clone)]
pub struct Rectangle {
    pub x1: usize,
    pub y1: usize,
    pub x2: usize,
    pub y2: usize,
}

impl Rectangle {
    pub fn from_points(p1: (usize, usize), p2: (usize, usize)) -> Self {
        Rectangle {
            x1: p1.0.min(p2.0),
            y1: p1.1.min(p2.1),
            x2: p1.0.max(p2.0),
            y2: p1.1.max(p2.1),
        }
    }
    
    pub fn contains(&self, x: usize, y: usize) -> bool {
        x >= self.x1 && x <= self.x2 && y >= self.y1 && y <= self.y2
    }
}

/// Spatial text matrix - the core of our extraction system
#[derive(Clone)]
pub struct SpatialTextMatrix {
    // Sparse storage - only positions with characters
    pub chars: HashMap<(usize, usize), char>,
    
    // Bounding box
    pub min_x: usize,
    pub min_y: usize,
    pub max_x: usize,
    pub max_y: usize,
    
    // Selection
    pub selections: Vec<Rectangle>,
    pub cursor: (usize, usize),
    pub selecting: bool,
    pub selection_start: Option<(usize, usize)>,
    
    // View settings
    pub zoom: f32,
    pub show_grid: bool,
    pub viewport_x: usize,
    pub viewport_y: usize,
}

impl SpatialTextMatrix {
    pub fn new() -> Self {
        SpatialTextMatrix {
            chars: HashMap::new(),
            min_x: 0,
            min_y: 0,
            max_x: 0,
            max_y: 0,
            selections: Vec::new(),
            cursor: (0, 0),
            selecting: false,
            selection_start: None,
            zoom: 1.0,
            show_grid: false,
            viewport_x: 0,
            viewport_y: 0,
        }
    }
    
    /// Convert PDF coordinates to grid coordinates
    pub fn from_pdf_coords(chars: Vec<CharInfo>, page_height: f32) -> Self {
        let mut matrix = Self::new();
        
        // Constants for coordinate transformation
        const CHAR_WIDTH: f32 = 7.0;  // Approximate terminal char width in PDF units
        const LINE_HEIGHT: f32 = 12.0;  // Approximate line height in PDF units
        
        for char_info in chars {
            // PDF coords are bottom-up, terminal is top-down
            let grid_x = (char_info.x / CHAR_WIDTH) as usize;
            let grid_y = ((page_height - char_info.y) / LINE_HEIGHT) as usize;
            
            matrix.set_char(grid_x, grid_y, char_info.ch);
        }
        
        matrix
    }
    
    pub fn set_char(&mut self, x: usize, y: usize, ch: char) {
        self.chars.insert((x, y), ch);
        self.update_bounds(x, y);
    }
    
    pub fn clear(&mut self) {
        self.chars.clear();
        self.selections.clear();
        self.min_x = 0;
        self.min_y = 0;
        self.max_x = 0;
        self.max_y = 0;
        self.cursor = (0, 0);
        self.selecting = false;
        self.selection_start = None;
    }
    
    pub fn update_bounds(&mut self, x: usize, y: usize) {
        if self.chars.len() == 1 {
            // First character
            self.min_x = x;
            self.max_x = x;
            self.min_y = y;
            self.max_y = y;
        } else {
            self.min_x = self.min_x.min(x);
            self.max_x = self.max_x.max(x);
            self.min_y = self.min_y.min(y);
            self.max_y = self.max_y.max(y);
        }
    }
    
    pub fn is_selected(&self, x: usize, y: usize) -> bool {
        for rect in &self.selections {
            if rect.contains(x, y) {
                return true;
            }
        }
        false
    }
    
    pub fn start_selection(&mut self) {
        self.selecting = true;
        self.selection_start = Some(self.cursor);
        self.selections.clear();  // Clear old selections when starting new
    }
    
    pub fn update_selection(&mut self) {
        if let Some(start) = self.selection_start {
            let rect = Rectangle::from_points(start, self.cursor);
            
            if self.selecting {
                if self.selections.is_empty() {
                    self.selections.push(rect);
                } else {
                    // Update the current selection
                    if let Some(last) = self.selections.last_mut() {
                        *last = rect;
                    }
                }
            }
        }
    }
    
    pub fn end_selection(&mut self) {
        self.selecting = false;
        self.selection_start = None;
    }
    
    pub fn copy_selection(&self) -> String {
        let mut result = String::new();
        
        for rect in &self.selections {
            // Copy preserving spatial layout
            for y in rect.y1..=rect.y2 {
                for x in rect.x1..=rect.x2 {
                    if let Some(&ch) = self.chars.get(&(x, y)) {
                        result.push(ch);
                    } else {
                        result.push(' ');  // Preserve spacing!
                    }
                }
                result.push('\n');
            }
            result.push('\n');
        }
        
        result
    }
    
    pub fn move_cursor(&mut self, dx: i32, dy: i32) {
        let new_x = (self.cursor.0 as i32 + dx).max(0) as usize;
        let new_y = (self.cursor.1 as i32 + dy).max(0) as usize;
        
        self.cursor = (new_x.min(self.max_x), new_y.min(self.max_y));
        
        if self.selecting {
            self.update_selection();
        }
    }
    
    pub fn zoom_in(&mut self) {
        self.zoom = (self.zoom * 1.2).min(3.0);
    }
    
    pub fn zoom_out(&mut self) {
        self.zoom = (self.zoom * 0.8).max(0.5);
    }
    
    /// Render the matrix at a specific terminal position
    pub fn render_at(&self, start_x: u16, start_y: u16, width: u16, height: u16) -> Result<()> {
        use std::io::{stdout, Write};
        
        // First pass: Draw dot matrix grid pattern
        for y in 0..height {
            print!("\x1b[{};{}H", start_y + y, start_x);
            for x in 0..width {
                // Draw dots at regular intervals for spatial reference
                if x % 5 == 0 && y % 2 == 0 {
                    print!("·");
                } else {
                    print!(" ");
                }
            }
        }
        
        // Draw grid lines if enabled (overlay on dots)
        if self.show_grid {
            // Vertical lines every 10 chars
            for x in (0..width).step_by(10) {
                for y in 0..height {
                    print!("\x1b[{};{}H│", start_y + y, start_x + x);
                }
            }
            // Horizontal lines every 5 rows
            for y in (0..height).step_by(5) {
                print!("\x1b[{};{}H", start_y + y, start_x);
                for x in 0..width {
                    if x % 10 == 0 {
                        print!("┼");  // Intersection
                    } else {
                        print!("─");
                    }
                }
            }
        }
        
        // Render each character
        for ((x, y), ch) in &self.chars {
            // Apply viewport offset
            if *x < self.viewport_x || *y < self.viewport_y {
                continue;
            }
            
            let screen_x = (*x - self.viewport_x) as u16;
            let screen_y = (*y - self.viewport_y) as u16;
            
            // Check bounds
            if screen_x >= width || screen_y >= height {
                continue;
            }
            
            let abs_x = start_x + screen_x;
            let abs_y = start_y + screen_y;
            
            // Move cursor and print
            print!("\x1b[{};{}H", abs_y, abs_x);
            
            if self.is_selected(*x, *y) {
                // Inverted colors for selection
                print!("\x1b[7m{}\x1b[0m", ch);
            } else {
                print!("{}", ch);
            }
        }
        
        // Show cursor
        if self.cursor.0 >= self.viewport_x && self.cursor.1 >= self.viewport_y {
            let cursor_x = (self.cursor.0 - self.viewport_x) as u16;
            let cursor_y = (self.cursor.1 - self.viewport_y) as u16;
            
            if cursor_x < width && cursor_y < height {
                print!("\x1b[{};{}H", start_y + cursor_y, start_x + cursor_x);
                print!("\x1b[7m \x1b[0m");  // Show cursor as inverted space
            }
        }
        
        stdout().flush()?;
        Ok(())
    }
}