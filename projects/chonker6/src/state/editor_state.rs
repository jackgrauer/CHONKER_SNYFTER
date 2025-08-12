use crate::actions::{CursorDirection, Position, SelectionMode};

#[derive(Debug, Clone)]
pub struct EditorState {
    pub matrix: Vec<Vec<char>>,
    pub cursor: Position,
    pub selection: Option<Selection>,
    pub modified: bool,
    pub mouse_selection: Option<MouseSelection>,
}

#[derive(Debug, Clone)]
pub struct Selection {
    pub start: Position,
    pub end: Position,
    pub mode: SelectionMode,
}

#[derive(Debug, Clone)]
pub struct MouseSelection {
    pub start: Position,
    pub end: Position,
    pub mode: SelectionMode,
    pub active: bool,
}

impl Selection {
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end, mode: SelectionMode::Block }
    }
    
    pub fn new_block(start: Position, end: Position) -> Self {
        Self { start, end, mode: SelectionMode::Block }
    }
    
    pub fn get_bounds(&self) -> (Position, Position) {
        let min_row = self.start.row.min(self.end.row);
        let max_row = self.start.row.max(self.end.row);
        let min_col = self.start.col.min(self.end.col);
        let max_col = self.start.col.max(self.end.col);
        
        (Position { row: min_row, col: min_col }, Position { row: max_row, col: max_col })
    }
    
    pub fn contains(&self, pos: Position) -> bool {
        let (min_pos, max_pos) = self.get_bounds();
        match self.mode {
            SelectionMode::Block => {
                // Block selection: exact rectangular region
                pos.row >= min_pos.row && pos.row <= max_pos.row &&
                pos.col >= min_pos.col && pos.col <= max_pos.col
            }
            SelectionMode::Line => {
                // Line selection: traditional terminal selection
                if pos.row < min_pos.row || pos.row > max_pos.row {
                    false
                } else if pos.row == min_pos.row && pos.row == max_pos.row {
                    // Single line selection
                    pos.col >= min_pos.col && pos.col <= max_pos.col
                } else if pos.row == min_pos.row {
                    // First line - from start column to end
                    pos.col >= min_pos.col
                } else if pos.row == max_pos.row {
                    // Last line - from beginning to end column
                    pos.col <= max_pos.col
                } else {
                    // Middle lines - entire line selected
                    true
                }
            }
        }
    }
    
    pub fn get_selected_text(&self, matrix: &[Vec<char>]) -> String {
        let (min_pos, max_pos) = self.get_bounds();
        let mut result = String::new();
        
        match self.mode {
            SelectionMode::Block => {
                // Block selection: extract rectangular region
                for row in min_pos.row..=max_pos.row {
                    if row < matrix.len() {
                        for col in min_pos.col..=max_pos.col {
                            if col < matrix[row].len() {
                                result.push(matrix[row][col]);
                            } else {
                                result.push(' '); // Fill missing chars with spaces
                            }
                        }
                        if row < max_pos.row {
                            result.push('\n');
                        }
                    }
                }
            }
            SelectionMode::Line => {
                // Line selection: traditional terminal selection
                for row in min_pos.row..=max_pos.row {
                    if row < matrix.len() {
                        let start_col = if row == min_pos.row { min_pos.col } else { 0 };
                        let end_col = if row == max_pos.row { max_pos.col.min(matrix[row].len()) } else { matrix[row].len() };
                        
                        let line: String = matrix[row][start_col..end_col].iter().collect();
                        result.push_str(&line);
                        
                        if row < max_pos.row {
                            result.push('\n');
                        }
                    }
                }
            }
        }
        
        result
    }
}

impl MouseSelection {
    pub fn new(start: Position, mode: SelectionMode) -> Self {
        Self {
            start,
            end: start,
            mode,
            active: true,
        }
    }
    
    pub fn update_end(&mut self, end: Position) {
        self.end = end;
    }
    
    pub fn get_selection(&self) -> Selection {
        Selection {
            start: self.start,
            end: self.end,
            mode: self.mode,
        }
    }
}

impl Default for EditorState {
    fn default() -> Self {
        // Start with empty matrix - will be populated when text is extracted
        Self {
            matrix: Vec::new(),
            cursor: Position { row: 0, col: 0 },
            selection: None,
            modified: false,
            mouse_selection: None,
        }
    }
}

impl EditorState {
    pub fn has_content(&self) -> bool {
        !self.matrix.is_empty()
    }
    
    /// Ensure the matrix is large enough to accommodate the given position
    pub fn ensure_matrix_size(&mut self, pos: Position) {
        // Expand rows if needed
        while pos.row >= self.matrix.len() {
            let current_width = if self.matrix.is_empty() { 80 } else { self.matrix[0].len() };
            let mut new_row = vec![' '; current_width];
            
            // Fill new rows with dots
            for col in 0..current_width {
                new_row[col] = '.';
            }
            self.matrix.push(new_row);
        }
        
        // Expand columns if needed
        if pos.col >= self.matrix.get(0).map_or(0, |row| row.len()) {
            let target_width = pos.col + 1;
            for row in self.matrix.iter_mut() {
                while row.len() < target_width {
                    row.push('.');
                }
            }
        }
    }
    
    pub fn set_matrix(&mut self, matrix: Vec<Vec<char>>) {
        self.matrix = matrix;
        self.cursor = Position { row: 0, col: 0 };
        self.selection = None;
        self.modified = false;
    }
    
    pub fn insert_char(&mut self, c: char) {
        // Ensure we have rows up to cursor position
        while self.cursor.row >= self.matrix.len() {
            self.matrix.push(vec![' '; 80]); // Start with reasonable width
        }
        
        // Ensure current row has columns up to cursor position + 1
        if self.cursor.col >= self.matrix[self.cursor.row].len() {
            self.matrix[self.cursor.row].resize(self.cursor.col + 1, ' ');
        }
        
        // Insert character at cursor position
        self.matrix[self.cursor.row][self.cursor.col] = c;
        
        // Move cursor right
        self.cursor.col += 1;
        self.modified = true;
    }
    
    pub fn insert_newline(&mut self) {
        // Move cursor to next line at beginning
        self.cursor.row += 1;
        self.cursor.col = 0;
        
        // Ensure we have enough rows
        while self.cursor.row >= self.matrix.len() {
            self.matrix.push(vec![' '; 80]);
        }
        
        self.modified = true;
    }
    
    pub fn delete_char(&mut self) {
        if self.cursor.col > 0 && self.cursor.row < self.matrix.len() {
            self.cursor.col -= 1;
            if self.cursor.col < self.matrix[self.cursor.row].len() {
                self.matrix[self.cursor.row][self.cursor.col] = ' ';
                self.modified = true;
            }
        }
    }
    
    pub fn delete_at_cursor(&mut self) {
        if self.cursor.row < self.matrix.len() && self.cursor.col < self.matrix[self.cursor.row].len() {
            self.matrix[self.cursor.row][self.cursor.col] = ' ';
            self.modified = true;
        }
    }
    
    pub fn start_selection(&mut self) {
        self.selection = Some(Selection::new(self.cursor, self.cursor));
    }
    
    pub fn start_block_selection(&mut self) {
        self.selection = Some(Selection::new_block(self.cursor, self.cursor));
    }
    
    pub fn start_mouse_selection(&mut self, pos: Position, mode: SelectionMode) {
        // Ensure matrix can accommodate this position
        self.ensure_matrix_size(pos);
        
        self.mouse_selection = Some(MouseSelection::new(pos, mode));
        // Also set keyboard selection for consistency
        match mode {
            SelectionMode::Block => self.selection = Some(Selection::new_block(pos, pos)),
            SelectionMode::Line => self.selection = Some(Selection::new(pos, pos)),
        }
    }
    
    pub fn update_mouse_selection(&mut self, pos: Position) {
        // Ensure matrix can accommodate this position
        self.ensure_matrix_size(pos);
        
        if let Some(ref mut mouse_sel) = self.mouse_selection {
            mouse_sel.update_end(pos);
            // Update keyboard selection too
            if let Some(ref mut sel) = self.selection {
                sel.end = pos;
            }
        }
    }
    
    pub fn complete_mouse_selection(&mut self) {
        if let Some(mouse_sel) = &self.mouse_selection {
            // Keep the selection but remove mouse tracking
            self.selection = Some(mouse_sel.get_selection());
        }
        self.mouse_selection = None;
    }
    
    pub fn update_selection(&mut self) {
        if let Some(ref mut selection) = self.selection {
            selection.end = self.cursor;
        }
    }
    
    pub fn end_selection(&mut self) {
        self.selection = None;
    }
    
    pub fn select_all(&mut self) {
        if !self.matrix.is_empty() {
            let start = Position { row: 0, col: 0 };
            let end = Position { 
                row: self.matrix.len() - 1, 
                col: self.matrix.last().map(|row| row.len()).unwrap_or(0)
            };
            self.selection = Some(Selection::new(start, end));
        }
    }
    
    pub fn get_selected_text(&self) -> Option<String> {
        if let Some(ref selection) = self.selection {
            Some(selection.get_selected_text(&self.matrix))
        } else {
            None
        }
    }
    
    
    pub fn delete_selection(&mut self) {
        if let Some(ref selection) = self.selection {
            let (min_pos, max_pos) = selection.get_bounds();
            
            match selection.mode {
                SelectionMode::Block => {
                    // Block mode: only delete the rectangular region
                    for row in min_pos.row..=max_pos.row {
                        if row < self.matrix.len() {
                            for col in min_pos.col..=max_pos.col {
                                if col < self.matrix[row].len() {
                                    self.matrix[row][col] = ' ';
                                }
                            }
                        }
                    }
                },
                SelectionMode::Line => {
                    // Line mode: traditional terminal-style deletion
                    for row in min_pos.row..=max_pos.row {
                        if row < self.matrix.len() {
                            let start_col = if row == min_pos.row { min_pos.col } else { 0 };
                            let end_col = if row == max_pos.row { max_pos.col.min(self.matrix[row].len()) } else { self.matrix[row].len() };
                            
                            for col in start_col..end_col {
                                if col < self.matrix[row].len() {
                                    self.matrix[row][col] = ' ';
                                }
                            }
                        }
                    }
                }
            }
            
            self.cursor = min_pos;
            self.selection = None;
            self.modified = true;
        }
    }
    
    pub fn paste_text(&mut self, text: String) {
        self.paste_text_with_mode(text, SelectionMode::Line);
    }
    
    pub fn paste_text_with_mode(&mut self, text: String, mode: SelectionMode) {
        let lines: Vec<&str> = text.lines().collect();
        if lines.is_empty() {
            return;
        }
        
        let start_row = self.cursor.row;
        let start_col = self.cursor.col;
        
        // Ensure matrix can accommodate paste
        let needed_rows = start_row + lines.len();
        while self.matrix.len() < needed_rows {
            self.matrix.push(vec![' '; 80]);
        }
        
        match mode {
            SelectionMode::Block => {
                // Block mode: maintain rectangular structure
                for (row_offset, line) in lines.iter().enumerate() {
                    let target_row = start_row + row_offset;
                    let needed_cols = start_col + line.len();
                    
                    if target_row < self.matrix.len() {
                        // Extend row if needed
                        if self.matrix[target_row].len() < needed_cols {
                            self.matrix[target_row].resize(needed_cols, ' ');
                        }
                        
                        // Paste characters maintaining block structure
                        for (col_offset, ch) in line.chars().enumerate() {
                            let target_col = start_col + col_offset;
                            if target_col < self.matrix[target_row].len() {
                                self.matrix[target_row][target_col] = ch;
                            }
                        }
                    }
                }
                
                // For block mode, keep cursor at start position
                // User can see the rectangular paste result
                self.cursor = Position { row: start_row, col: start_col };
            },
            SelectionMode::Line => {
                // Line mode: traditional paste behavior
                for (row_offset, line) in lines.iter().enumerate() {
                    let target_row = start_row + row_offset;
                    let needed_cols = start_col + line.len();
                    
                    if target_row < self.matrix.len() {
                        // Extend row if needed
                        if self.matrix[target_row].len() < needed_cols {
                            self.matrix[target_row].resize(needed_cols, ' ');
                        }
                        
                        // Paste characters
                        for (col_offset, ch) in line.chars().enumerate() {
                            let target_col = start_col + col_offset;
                            if target_col < self.matrix[target_row].len() {
                                self.matrix[target_row][target_col] = ch;
                            }
                        }
                    }
                }
                
                // Move cursor to end of pasted text
                if let Some(last_line) = lines.last() {
                    if lines.len() == 1 {
                        self.cursor.col = start_col + last_line.len();
                    } else {
                        self.cursor.row = start_row + lines.len() - 1;
                        self.cursor.col = last_line.len();
                    }
                }
            }
        }
        
        self.modified = true;
    }
    
    pub fn is_position_selected(&self, pos: Position) -> bool {
        // Check keyboard selection first
        if let Some(ref selection) = self.selection {
            selection.contains(pos)
        } else if let Some(ref mouse_sel) = self.mouse_selection {
            // Check mouse selection if no keyboard selection
            mouse_sel.get_selection().contains(pos)
        } else {
            false
        }
    }
    
    pub fn move_cursor(&mut self, direction: CursorDirection) {
        match direction {
            CursorDirection::Up => {
                if self.cursor.row > 0 {
                    self.cursor.row -= 1;
                    // Keep column position, but ensure it's reasonable
                    if self.cursor.row < self.matrix.len() {
                        // Allow cursor beyond current row length for typing
                        let max_reasonable = self.matrix[self.cursor.row].len().max(80);
                        if self.cursor.col > max_reasonable {
                            self.cursor.col = max_reasonable;
                        }
                    }
                }
            }
            CursorDirection::Down => {
                // Always allow moving down to add new content
                self.cursor.row += 1;
                // Ensure matrix can accommodate new cursor position
                self.ensure_matrix_size(self.cursor);
            }
            CursorDirection::Left => {
                if self.cursor.col > 0 {
                    self.cursor.col -= 1;
                } else if self.cursor.row > 0 {
                    // Move to end of previous line
                    self.cursor.row -= 1;
                    if self.cursor.row < self.matrix.len() {
                        self.cursor.col = self.matrix[self.cursor.row].len();
                    } else {
                        self.cursor.col = 0;
                    }
                }
            }
            CursorDirection::Right => {
                // Always allow moving right to add new content
                self.cursor.col += 1;
                
                // Ensure matrix can accommodate new cursor position
                self.ensure_matrix_size(self.cursor);
            }
            CursorDirection::Home => {
                self.cursor.col = 0;
            }
            CursorDirection::End => {
                if self.cursor.row < self.matrix.len() {
                    self.cursor.col = self.matrix[self.cursor.row].len();
                } else {
                    self.cursor.col = 0;
                }
            }
            _ => {}
        }
    }
}