use super::parser::{ParsedTable, TableRow, TableCell};

pub struct TableFormatter {
    padding: usize,
    alignment: ColumnAlignment,
}

#[derive(Clone, Copy)]
pub enum ColumnAlignment {
    Left,
    Center,
    Right,
}

impl TableFormatter {
    pub fn new() -> Self {
        Self {
            padding: 1,
            alignment: ColumnAlignment::Left,
        }
    }
    
    pub fn format_table(&self, table: &ParsedTable) -> String {
        if table.rows.is_empty() {
            return String::new();
        }
        
        // Analyze table structure
        let column_count = self.determine_column_count(table);
        let column_widths = self.calculate_column_widths(table, column_count);
        
        // Rebuild table with proper formatting
        let mut formatted = Vec::new();
        
        for (i, row) in table.rows.iter().enumerate() {
            let formatted_row = self.format_row(row, &column_widths, column_count);
            formatted.push(formatted_row);
            
            // Add separator after header (first row)
            if i == 0 && !table.has_header_separator {
                let separator = self.create_separator(&column_widths);
                formatted.push(separator);
            }
        }
        
        formatted.join("\n")
    }
    
    fn determine_column_count(&self, table: &ParsedTable) -> usize {
        // Use mode (most common) column count, not max
        let mut count_frequency = std::collections::HashMap::new();
        
        for row in &table.rows {
            *count_frequency.entry(row.cells.len()).or_insert(0) += 1;
        }
        
        count_frequency.into_iter()
            .max_by_key(|(_, freq)| *freq)
            .map(|(count, _)| count)
            .unwrap_or(0)
    }
    
    fn calculate_column_widths(&self, table: &ParsedTable, column_count: usize) -> Vec<usize> {
        let mut widths = vec![0; column_count];
        
        for row in &table.rows {
            for (i, cell) in row.cells.iter().enumerate() {
                if i < column_count {
                    widths[i] = widths[i].max(cell.content.len());
                }
            }
        }
        
        // Add padding
        widths.iter_mut().for_each(|w| *w += self.padding * 2);
        
        widths
    }
    
    fn format_row(&self, row: &TableRow, widths: &[usize], column_count: usize) -> String {
        let mut formatted_cells = Vec::new();
        
        for i in 0..column_count {
            let content = row.cells.get(i)
                .map(|c| &c.content[..])
                .unwrap_or("");
            
            let formatted = self.pad_content(content, widths[i]);
            formatted_cells.push(formatted);
        }
        
        format!("| {} |", formatted_cells.join(" | "))
    }
    
    fn pad_content(&self, content: &str, width: usize) -> String {
        match self.alignment {
            ColumnAlignment::Left => format!("{:<width$}", content, width = width),
            ColumnAlignment::Right => format!("{:>width$}", content, width = width),
            ColumnAlignment::Center => format!("{:^width$}", content, width = width),
        }
    }
    
    fn create_separator(&self, widths: &[usize]) -> String {
        let separators: Vec<String> = widths.iter()
            .map(|&w| "-".repeat(w))
            .collect();
        
        format!("| {} |", separators.join(" | "))
    }
}
