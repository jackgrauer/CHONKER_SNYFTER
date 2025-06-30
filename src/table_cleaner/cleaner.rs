use super::parser::{MarkdownParser, ParsedTable};
use super::formatter::TableFormatter;
use super::detector::SmartTableDetector;
use super::ProcessingChanges;
use anyhow::Result;

pub struct TableCleaner {
    parser: MarkdownParser,
    formatter: TableFormatter,
    detector: SmartTableDetector,
}

impl TableCleaner {
    pub fn new() -> Self {
        Self {
            parser: MarkdownParser::new(),
            formatter: TableFormatter::new(),
            detector: SmartTableDetector::new(),
        }
    }
    
    pub fn process_markdown(&self, content: &str) -> Result<(String, ProcessingChanges)> {
        let mut result = content.to_string();
        let mut changes = ProcessingChanges::default();
        
        // Use smart detection first
        let regions = self.detector.detect_tables_smart(content);
        
        // Process each table region
        for region in regions {
            // Extract just the table region
            let table_content = region.lines.join("\n");
            
            // Parse the table structure
            let parsed_tables = self.parser.parse_document(&table_content);
            
            for parsed_table in parsed_tables {
                changes.total_tables += 1;
                
                // Check if needs fixing
                if self.table_needs_fixing(&parsed_table) {
                    // Format the table
                    let formatted = self.formatter.format_table(&parsed_table);
                    
                    // Replace the original table content with the formatted version
                    result = result.replace(&table_content, &formatted);
                    changes.tables_fixed += 1;
                    
                    // Count improvements
                    changes.empty_cells_removed += self.count_empty_cells_removed(&parsed_table);
                    changes.columns_normalized += 1;
                } else {
                    changes.tables_skipped += 1;
                }
            }
        }
        
        Ok((result, changes))
    }
    
    fn table_needs_fixing(&self, table: &ParsedTable) -> bool {
        // Check various indicators that suggest the table needs fixing
        let indicators = vec![
            // Inconsistent column count
            table.column_count.is_none(),
            
            // Missing header separator
            !table.has_header_separator && table.rows.len() > 1,
            
            // Uneven cell counts
            table.rows.iter().any(|row| {
                table.column_count.map_or(false, |count| row.cells.len() != count)
            }),
            
            // Misaligned cells (check cell positions)
            self.has_alignment_issues(table),
            
            // Too many empty cells
            self.has_excessive_empty_cells(table),
        ];
        
        indicators.into_iter().any(|x| x)
    }
    
    fn has_alignment_issues(&self, table: &ParsedTable) -> bool {
        if table.rows.len() < 2 {
            return false;
        }
        
        // Check if cell positions are consistent across rows
        let first_row_positions: Vec<usize> = table.rows[0].cells.iter()
            .map(|cell| cell.start_pos)
            .collect();
        
        for row in &table.rows[1..] {
            let positions: Vec<usize> = row.cells.iter()
                .map(|cell| cell.start_pos)
                .collect();
            
            if positions.len() != first_row_positions.len() {
                return true;
            }
            
            for (p1, p2) in first_row_positions.iter().zip(&positions) {
                if (*p1 as i32 - *p2 as i32).abs() > 5 {
                    return true;
                }
            }
        }
        
        false
    }
    
    fn has_excessive_empty_cells(&self, table: &ParsedTable) -> bool {
        let total_cells: usize = table.rows.iter().map(|row| row.cells.len()).sum();
        let empty_cells: usize = table.rows.iter()
            .map(|row| row.cells.iter().filter(|cell| cell.content.trim().is_empty()).count())
            .sum();
        
        if total_cells == 0 {
            return false;
        }
        
        // If more than 30% of cells are empty, consider it problematic
        (empty_cells as f32 / total_cells as f32) > 0.3
    }
    
    fn count_empty_cells_removed(&self, table: &ParsedTable) -> usize {
        table.rows.iter()
            .map(|row| row.cells.iter().filter(|cell| cell.content.trim().is_empty()).count())
            .sum()
    }
}

impl Default for TableCleaner {
    fn default() -> Self {
        Self::new()
    }
}
