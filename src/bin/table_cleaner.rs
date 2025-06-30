use std::path::PathBuf;
use clap::Parser;
use anyhow::Result;

#[derive(Debug, Default)]
struct ProcessingChanges {
    total_tables: usize,
    tables_fixed: usize,
    tables_skipped: usize,
    empty_cells_removed: usize,
    columns_normalized: usize,
}

impl ProcessingChanges {
    fn summary(&self) -> String {
        format!(
            "Processed {} tables: {} fixed, {} skipped. Removed {} empty cells, normalized {} columns.",
            self.total_tables, self.tables_fixed, self.tables_skipped,
            self.empty_cells_removed, self.columns_normalized
        )
    }
}

#[derive(Parser)]
#[command(name = "table_cleaner")]
#[command(about = "Clean and format markdown tables extracted from PDFs")]
struct Cli {
    /// Input markdown file
    input: PathBuf,
    
    /// Output markdown file (optional, defaults to input_cleaned.md)
    #[arg(short, long)]
    output: Option<PathBuf>,
    
    /// Render output as HTML for viewing
    #[arg(short, long)]
    render: bool,
}

use cleaner::TableCleaner;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Read the input file
    let content = tokio::fs::read_to_string(&cli.input).await?;
    println!("Read {} characters from {:?}", content.len(), cli.input);
    
    // Process the markdown with table cleaner
    let cleaner = TableCleaner::new();
    let (cleaned_content, changes) = cleaner.process_markdown(&content)?;
    
    println!("{}", changes.summary());
    
    // Determine output path
    let output_path = cli.output.unwrap_or_else(|| {
        let mut path = cli.input.clone();
        let stem = path.file_stem().unwrap().to_string_lossy();
        path.set_file_name(format!("{}_cleaned.md", stem));
        path
    });
    
    // Write cleaned content
    tokio::fs::write(&output_path, &cleaned_content).await?;
    println!("Wrote cleaned markdown to {:?}", output_path);
    
    // Optionally render as HTML
    if cli.render {
        let html_path = {
            let mut path = output_path.clone();
            path.set_extension("html");
            path
        };
        
        let html_content = render_markdown_to_html(&cleaned_content);
        tokio::fs::write(&html_path, html_content).await?;
        println!("Rendered HTML to {:?}", html_path);
        
        // Try to open in browser
        if cfg!(target_os = "macos") {
            let _ = std::process::Command::new("open")
                .arg(&html_path)
                .spawn();
        }
    }
    
    Ok(())
}

fn render_markdown_to_html(markdown: &str) -> String {
    let parser = pulldown_cmark::Parser::new(markdown);
    let mut html_output = String::new();
    pulldown_cmark::html::push_html(&mut html_output, parser);
    
    // Wrap in full HTML document with CSS for table styling
    format!(r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Cleaned Tables</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            max-width: 1200px;
            margin: 0 auto;
            padding: 20px;
            line-height: 1.6;
        }}
        table {{
            border-collapse: collapse;
            width: 100%;
            margin: 20px 0;
            font-size: 14px;
        }}
        th, td {{
            border: 1px solid #ddd;
            padding: 8px 12px;
            text-align: left;
        }}
        th {{
            background-color: #f2f2f2;
            font-weight: bold;
        }}
        tr:nth-child(even) {{
            background-color: #f9f9f9;
        }}
        tr:hover {{
            background-color: #f5f5f5;
        }}
        h1, h2, h3 {{
            color: #333;
            margin-top: 30px;
        }}
        .table-container {{
            overflow-x: auto;
            margin: 20px 0;
        }}
    </style>
</head>
<body>
{}
</body>
</html>"#, html_output)
}

// Include the table cleaner modules inline since we can't use mod paths easily in a bin
mod parser {
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
        pub content: String,
        pub start_pos: usize,
        pub end_pos: usize,
    }

    #[derive(Debug, Clone)]
    pub struct TableRow {
        pub cells: Vec<TableCell>,
        pub raw_line: String,
        pub line_number: usize,
    }

    #[derive(Debug, Clone)]
    pub struct ParsedTable {
        pub rows: Vec<TableRow>,
        pub start_line: usize,
        pub end_line: usize,
        pub column_count: Option<usize>,
        pub has_header_separator: bool,
    }

    pub struct MarkdownParser {
        min_table_indicators: usize,
        context_window: usize,
    }

    impl MarkdownParser {
        pub fn new() -> Self {
            Self {
                min_table_indicators: 2,
                context_window: 3,
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
            
            if trimmed.is_empty() {
                return LineType::Empty;
            }
            
            if trimmed.starts_with('#') {
                return LineType::Header;
            }
            
            let indicators = self.count_table_indicators(line);
            
            if indicators >= self.min_table_indicators {
                if self.is_separator_line(trimmed) {
                    return LineType::TableSeparator;
                }
                
                if self.looks_like_table_row(line, index, all_lines) {
                    return LineType::TableRow;
                }
            }
            
            LineType::Text
        }
        
        fn count_table_indicators(&self, line: &str) -> usize {
            line.chars().filter(|&c| c == '|').count()
        }
        
        fn is_separator_line(&self, line: &str) -> bool {
            let chars: Vec<char> = line.chars().collect();
            let unique_chars: std::collections::HashSet<char> = chars.iter().cloned().collect();
            
            let separator_chars: std::collections::HashSet<char> = 
                ['-', '=', '_', '|', ' ', ':'].iter().cloned().collect();
            
            unique_chars.is_subset(&separator_chars) && 
            chars.iter().filter(|&&c| c == '-' || c == '=').count() > chars.len() / 3
        }
        
        fn looks_like_table_row(&self, line: &str, index: usize, all_lines: &[&str]) -> bool {
            let mut similar_lines = 0;
            
            for offset in -(self.context_window as i32)..=(self.context_window as i32) {
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
            let count1 = line1.chars().filter(|&c| c == '|').count();
            let count2 = line2.chars().filter(|&c| c == '|').count();
            (count1 as i32 - count2 as i32).abs() <= 1
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
                        if let Some(mut table) = current_table.take() {
                            if let Some(parsed) = table.build() {
                                tables.push(parsed);
                            }
                        }
                    }
                }
            }
            
            if let Some(mut table) = current_table {
                if let Some(parsed) = table.build() {
                    tables.push(parsed);
                }
            }
            
            tables
        }
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
}

mod formatter {
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
            
            let column_count = self.determine_column_count(table);
            let column_widths = self.calculate_column_widths(table, column_count);
            
            let mut formatted = Vec::new();
            
            for (i, row) in table.rows.iter().enumerate() {
                let formatted_row = self.format_row(row, &column_widths, column_count);
                formatted.push(formatted_row);
                
                if i == 0 && !table.has_header_separator {
                    let separator = self.create_separator(&column_widths);
                    formatted.push(separator);
                }
            }
            
            formatted.join("\n")
        }
        
        fn determine_column_count(&self, table: &ParsedTable) -> usize {
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
            format!("{:<width$}", content, width = width)
        }
        
        fn create_separator(&self, widths: &[usize]) -> String {
            let separators: Vec<String> = widths.iter()
                .map(|&w| "-".repeat(w))
                .collect();
            
            format!("| {} |", separators.join(" | "))
        }
    }
}

mod detector {
    pub struct SmartTableDetector {
        confidence_threshold: f32,
    }

    impl SmartTableDetector {
        pub fn new() -> Self {
            Self {
                confidence_threshold: 0.5, // Lower threshold for more detection
            }
        }
        
        pub fn detect_tables_smart(&self, content: &str) -> Vec<TableRegion> {
            let lines: Vec<&str> = content.lines().collect();
            let mut regions = Vec::new();
            let mut current_region: Option<TableRegionBuilder> = None;
            
            for (i, line) in lines.iter().enumerate() {
                let confidence = self.calculate_table_confidence(line, i, &lines);
                
                if confidence > self.confidence_threshold {
                    if current_region.is_none() {
                        current_region = Some(TableRegionBuilder::new(i));
                    }
                    
                    if let Some(ref mut region) = current_region {
                        region.add_line(i, line, confidence);
                    }
                } else if let Some(region) = current_region.take() {
                    if let Some(table) = region.build() {
                        regions.push(table);
                    }
                }
            }
            
            if let Some(region) = current_region {
                if let Some(table) = region.build() {
                    regions.push(table);
                }
            }
            
            regions
        }
        
        fn calculate_table_confidence(&self, line: &str, _index: usize, _all_lines: &[&str]) -> f32 {
            let pipe_count = line.chars().filter(|&c| c == '|').count();
            
            if pipe_count >= 3 {
                1.0
            } else if pipe_count >= 2 {
                0.7
            } else {
                0.0
            }
        }
    }

    pub struct TableRegion {
        pub start_line: usize,
        pub end_line: usize,
        pub lines: Vec<String>,
        pub confidence: f32,
    }

    struct TableRegionBuilder {
        start_line: usize,
        lines: Vec<(String, f32)>,
    }

    impl TableRegionBuilder {
        fn new(start_line: usize) -> Self {
            Self {
                start_line,
                lines: Vec::new(),
            }
        }
        
        fn add_line(&mut self, _line_num: usize, line: &str, confidence: f32) {
            self.lines.push((line.to_string(), confidence));
        }
        
        fn build(self) -> Option<TableRegion> {
            if self.lines.is_empty() {
                return None;
            }
            
            let avg_confidence = self.lines.iter()
                .map(|(_, conf)| conf)
                .sum::<f32>() / self.lines.len() as f32;
            
            Some(TableRegion {
                start_line: self.start_line,
                end_line: self.start_line + self.lines.len() - 1,
                lines: self.lines.into_iter().map(|(line, _)| line).collect(),
                confidence: avg_confidence,
            })
        }
    }
}

mod cleaner {
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
            
            let regions = self.detector.detect_tables_smart(content);
            
            for region in regions {
                let table_content = region.lines.join("\n");
                let parsed_tables = self.parser.parse_document(&table_content);
                
                for parsed_table in parsed_tables {
                    changes.total_tables += 1;
                    
                    if self.table_needs_fixing(&parsed_table) {
                        let formatted = self.formatter.format_table(&parsed_table);
                        result = result.replace(&table_content, &formatted);
                        changes.tables_fixed += 1;
                        changes.columns_normalized += 1;
                    } else {
                        changes.tables_skipped += 1;
                    }
                }
            }
            
            Ok((result, changes))
        }
        
        fn table_needs_fixing(&self, table: &ParsedTable) -> bool {
            // Always try to fix tables for now
            !table.rows.is_empty()
        }
    }
}
