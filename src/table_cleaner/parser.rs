use std::collections::HashMap;
use anyhow::Result;

#[derive(Debug, Clone, PartialEq)]
pub enum LineType {
    TableRow,
    TableSeparator,
    Text,
    Empty,
    Header,
}

#[derive(Debug, Clone)]
pub struct TableCell {
    content: String,
    start_pos: usize,
    end_pos: usize,
}

#[derive(Debug, Clone)]
pub struct TableRow {
    cells: Vec<TableCell>,
    raw_line: String,
    line_number: usize,
}

#[derive(Debug, Clone)]
pub struct ParsedTable {
    rows: Vec<TableRow>,
    start_line: usize,
    end_line: usize,
    column_count: Option<usize>,
    has_header_separator: bool,
}

pub struct MarkdownParser {
    min_table_indicators: usize,
    context_window: usize,
}

impl MarkdownParser {
    pub fn new() -> Self {
        Self {
            min_table_indicators: 2,  // At least 2 pipes to be a table
            context_window: 3,        // Look ahead/behind for context
        }
    }
    
    pub fn parse_document(&self, content: &str) -> Vec<ParsedTable> {
        let lines: Vec<&str> = content.lines().collect();
        let line_types = self.classify_lines(&lines);
        
        self.extract_tables(&lines, &line_types)
    }
    
    fn classify_lines(&self, lines: &[&str]) -> Vec<LineType> {
        lines.iter().enumerate().map(|(i, line)| {
            self.classify_line(line, i, lines)
        }).collect()
    }
    
    fn classify_line(&self, line: &str, index: usize, all_lines: &[&str]) -> LineType {
        let trimmed = line.trim();
        
        // Empty line
        if trimmed.is_empty() {
            return LineType::Empty;
        }
        
        // Header detection (# at start)
        if trimmed.starts_with('#') {
            return LineType::Header;
        }
        
        // Table detection - look for patterns, not just pipes
        let indicators = self.count_table_indicators(line);
        
        if indicators >= self.min_table_indicators {
            // Check if it's a separator line (---, ===, etc)
            if self.is_separator_line(trimmed) {
                return LineType::TableSeparator;
            }
            
            // Use context to determine if this is really a table
            if self.looks_like_table_row(line, index, all_lines) {
                return LineType::TableRow;
            }
        }
        
        LineType::Text
    }
    
    fn count_table_indicators(&self, line: &str) -> usize {
        let mut count = 0;
        let chars: Vec<char> = line.chars().collect();
        
        for i in 0..chars.len() {
            match chars[i] {
                '|' => count += 1,
                // Also count patterns that suggest columns
                _ => {
                    // Look for multiple spaces that might indicate column breaks
                    if i + 2 < chars.len() && 
                       chars[i] == ' ' && 
                       chars[i+1] == ' ' && 
                       chars[i+2] == ' ' {
                        count += 1;
                    }
                }
            }
        }
        
        count
    }
    
    fn is_separator_line(&self, line: &str) -> bool {
        let chars: Vec<char> = line.chars().collect();
        let unique_chars: std::collections::HashSet<char> = chars.iter().cloned().collect();
        
        // Common separator patterns
        let separator_chars: std::collections::HashSet<char> = 
            ['-', '=', '_', '|', ' ', ':'].iter().cloned().collect();
        
        // If line contains mostly separator characters
        unique_chars.is_subset(&separator_chars) && 
        chars.iter().filter(|&&c| c == '-' || c == '=').count() > chars.len() / 3
    }
    
    fn looks_like_table_row(&self, line: &str, index: usize, all_lines: &[&str]) -> bool {
        // Check context - tables usually have multiple similar lines
        let mut similar_lines = 0;
        
        // Look at surrounding lines
        for offset in -self.context_window as i32..=self.context_window as i32 {
            let check_index = index as i32 + offset;
            if check_index >= 0 && (check_index as usize) < all_lines.len() && offset != 0 {
                let other_line = all_lines[check_index as usize];
                if self.lines_have_similar_structure(line, other_line) {
                    similar_lines += 1;
                }
            }
        }
        
        similar_lines >= 1
    }
    
    fn lines_have_similar_structure(&self, line1: &str, line2: &str) -> bool {
        // Compare structural elements, not exact content
        let struct1 = self.extract_structure(line1);
        let struct2 = self.extract_structure(line2);
        
        // Similar number of segments
        (struct1.segment_count as i32 - struct2.segment_count as i32).abs() <= 1 &&
        // Similar positions of delimiters
        self.similar_delimiter_positions(&struct1.delimiter_positions, &struct2.delimiter_positions)
    }
    
    fn extract_structure(&self, line: &str) -> LineStructure {
        let mut delimiter_positions = Vec::new();
        let mut segment_count = 1;
        let chars: Vec<char> = line.chars().collect();
        
        for (i, &ch) in chars.iter().enumerate() {
            if ch == '|' {
                delimiter_positions.push(i);
                segment_count += 1;
            } else if i + 2 < chars.len() && 
                      chars[i] == ' ' && 
                      chars[i+1] == ' ' && 
                      chars[i+2] == ' ' {
                // Multiple spaces might indicate column boundary
                delimiter_positions.push(i);
                segment_count += 1;
            }
        }
        
        LineStructure {
            segment_count,
            delimiter_positions,
        }
    }
    
    fn similar_delimiter_positions(&self, pos1: &[usize], pos2: &[usize]) -> bool {
        if pos1.len() != pos2.len() {
            return false;
        }
        
        for (p1, p2) in pos1.iter().zip(pos2.iter()) {
            if (*p1 as i32 - *p2 as i32).abs() > 5 {  // Allow 5 char variance
                return false;
            }
        }
        
        true
    }
    
    fn extract_tables(&self, lines: &[&str], line_types: &[LineType]) -> Vec<ParsedTable> {
        let mut tables = Vec::new();
        let mut current_table: Option<TableBuilder> = None;
        
        for (i, (line, line_type)) in lines.iter().zip(line_types).enumerate() {
            match line_type {
                LineType::TableRow | LineType::TableSeparator => {
                    if current_table.is_none() {
                        current_table = Some(TableBuilder::new(i));
                    }
                    
                    if let Some(ref mut table) = current_table {
                        table.add_line(i, line, line_type);
                    }
                }
                _ => {
                    // End of table
                    if let Some(mut table) = current_table.take() {
                        if let Some(parsed) = table.build() {
                            tables.push(parsed);
                        }
                    }
                }
            }
        }
        
        // Don't forget table at end of document
        if let Some(mut table) = current_table {
            if let Some(parsed) = table.build() {
                tables.push(parsed);
            }
        }
        
        tables
    }
}

#[derive(Debug)]
struct LineStructure {
    segment_count: usize,
    delimiter_positions: Vec<usize>,
}

struct TableBuilder {
    start_line: usize,
    rows: Vec<TableRow>,
    has_separator: bool,
}

impl TableBuilder {
    fn new(start_line: usize) -> Self {
        Self {
            start_line,
            rows: Vec::new(),
            has_separator: false,
        }
    }
    
    fn add_line(&mut self, line_num: usize, line: &str, line_type: &LineType) {
        match line_type {
            LineType::TableRow => {
                let cells = self.parse_cells(line);
                self.rows.push(TableRow {
                    cells,
                    raw_line: line.to_string(),
                    line_number: line_num,
                });
            }
            LineType::TableSeparator => {
                self.has_separator = true;
            }
            _ => {}
        }
    }
    
    fn parse_cells(&self, line: &str) -> Vec<TableCell> {
        let mut cells = Vec::new();
        let mut current_cell = String::new();
        let mut start_pos = 0;
        let mut in_cell = false;
        
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;
        
        while i < chars.len() {
            match chars[i] {
                '|' => {
                    // Cell boundary
                    if in_cell {
                        cells.push(TableCell {
                            content: current_cell.trim().to_string(),
                            start_pos,
                            end_pos: i,
                        });
                        current_cell.clear();
                    }
                    in_cell = true;
                    start_pos = i + 1;
                }
                _ => {
                    if in_cell {
                        current_cell.push(chars[i]);
                    }
                }
            }
            i += 1;
        }
        
        // Don't forget last cell
        if in_cell && !current_cell.trim().is_empty() {
            cells.push(TableCell {
                content: current_cell.trim().to_string(),
                start_pos,
                end_pos: chars.len(),
            });
        }
        
        cells
    }
    
    fn build(self) -> Option<ParsedTable> {
        if self.rows.is_empty() {
            return None;
        }
        
        let column_count = self.rows.iter()
            .map(|row| row.cells.len())
            .max();
        
        Some(ParsedTable {
            end_line: self.rows.last().unwrap().line_number,
            rows: self.rows,
            start_line: self.start_line,
            column_count,
            has_header_separator: self.has_separator,
        })
    }
}
