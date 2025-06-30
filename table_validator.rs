use std::env;
use std::fs;
use std::io;

#[derive(Debug, Clone)]
struct Table {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    table_number: usize,
}

impl Table {
    fn to_markdown(&self) -> String {
        if self.headers.is_empty() && self.rows.is_empty() {
            return String::new();
        }

        let mut result = String::new();
        
        // Calculate column widths for alignment
        let mut col_widths = vec![0; self.headers.len().max(
            self.rows.iter().map(|r| r.len()).max().unwrap_or(0)
        )];
        
        // Check header widths
        for (i, header) in self.headers.iter().enumerate() {
            if i < col_widths.len() {
                col_widths[i] = col_widths[i].max(header.len());
            }
        }
        
        // Check row widths
        for row in &self.rows {
            for (i, cell) in row.iter().enumerate() {
                if i < col_widths.len() {
                    col_widths[i] = col_widths[i].max(cell.len());
                }
            }
        }
        
        // Minimum width of 3 for readability
        for width in &mut col_widths {
            *width = (*width).max(3);
        }

        result.push_str(&format!("\n### Table {}\n\n", self.table_number));

        // Headers
        if !self.headers.is_empty() {
            result.push('|');
            for (i, header) in self.headers.iter().enumerate() {
                if i < col_widths.len() {
                    result.push_str(&format!(" {:width$} |", header, width = col_widths[i]));
                } else {
                    result.push_str(&format!(" {} |", header));
                }
            }
            result.push('\n');
            
            // Separator row
            result.push('|');
            for (i, _) in self.headers.iter().enumerate() {
                if i < col_widths.len() {
                    result.push_str(&format!("{:-<width$}|", "", width = col_widths[i] + 2));
                } else {
                    result.push_str("---|");
                }
            }
            result.push('\n');
        }

        // Data rows
        for row in &self.rows {
            result.push('|');
            for (i, cell) in row.iter().enumerate() {
                if i < col_widths.len() {
                    result.push_str(&format!(" {:width$} |", cell, width = col_widths[i]));
                } else {
                    result.push_str(&format!(" {} |", cell));
                }
            }
            // Fill empty columns if row is shorter than headers
            for i in row.len()..col_widths.len() {
                result.push_str(&format!(" {:width$} |", "", width = col_widths[i]));
            }
            result.push('\n');
        }
        
        result.push('\n');
        result
    }

    fn to_html(&self) -> String {
        let mut result = String::new();
        
        result.push_str(&format!("<h3>Table {}</h3>\n", self.table_number));
        result.push_str("<table border=\"1\" cellpadding=\"5\" cellspacing=\"0\" style=\"border-collapse: collapse; font-family: monospace;\">\n");
        
        // Headers
        if !self.headers.is_empty() {
            result.push_str("  <thead>\n    <tr style=\"background-color: #f0f0f0; font-weight: bold;\">\n");
            for header in &self.headers {
                result.push_str(&format!("      <td>{}</td>\n", html_escape(header)));
            }
            result.push_str("    </tr>\n  </thead>\n");
        }
        
        // Data rows
        if !self.rows.is_empty() {
            result.push_str("  <tbody>\n");
            for row in &self.rows {
                result.push_str("    <tr>\n");
                for cell in row {
                    result.push_str(&format!("      <td>{}</td>\n", html_escape(cell)));
                }
                // Fill empty columns if row is shorter than headers
                for _ in row.len()..self.headers.len() {
                    result.push_str("      <td></td>\n");
                }
                result.push_str("    </tr>\n");
            }
            result.push_str("  </tbody>\n");
        }
        
        result.push_str("</table>\n<br>\n");
        result
    }

    fn to_ascii_table(&self) -> String {
        if self.headers.is_empty() && self.rows.is_empty() {
            return String::new();
        }

        let mut result = String::new();
        
        // Calculate column widths
        let mut col_widths = vec![0; self.headers.len().max(
            self.rows.iter().map(|r| r.len()).max().unwrap_or(0)
        )];
        
        // Check header widths
        for (i, header) in self.headers.iter().enumerate() {
            if i < col_widths.len() {
                col_widths[i] = col_widths[i].max(header.len());
            }
        }
        
        // Check row widths
        for row in &self.rows {
            for (i, cell) in row.iter().enumerate() {
                if i < col_widths.len() {
                    col_widths[i] = col_widths[i].max(cell.len());
                }
            }
        }
        
        // Minimum width of 3
        for width in &mut col_widths {
            *width = (*width).max(3);
        }

        result.push_str(&format!("\nTable {}\n", self.table_number));
        
        // Top border
        result.push('┌');
        for (i, &width) in col_widths.iter().enumerate() {
            result.push_str(&"─".repeat(width + 2));
            if i < col_widths.len() - 1 {
                result.push('┬');
            }
        }
        result.push_str("┐\n");

        // Headers
        if !self.headers.is_empty() {
            result.push('│');
            for (i, header) in self.headers.iter().enumerate() {
                if i < col_widths.len() {
                    result.push_str(&format!(" {:width$} │", header, width = col_widths[i]));
                } else {
                    result.push_str(&format!(" {} │", header));
                }
            }
            result.push('\n');
            
            // Header separator
            result.push('├');
            for (i, &width) in col_widths.iter().enumerate() {
                result.push_str(&"─".repeat(width + 2));
                if i < col_widths.len() - 1 {
                    result.push('┼');
                }
            }
            result.push_str("┤\n");
        }

        // Data rows
        for row in &self.rows {
            result.push('│');
            for (i, cell) in row.iter().enumerate() {
                if i < col_widths.len() {
                    result.push_str(&format!(" {:width$} │", cell, width = col_widths[i]));
                } else {
                    result.push_str(&format!(" {} │", cell));
                }
            }
            // Fill empty columns
            for i in row.len()..col_widths.len() {
                result.push_str(&format!(" {:width$} │", "", width = col_widths[i]));
            }
            result.push('\n');
        }
        
        // Bottom border
        result.push('└');
        for (i, &width) in col_widths.iter().enumerate() {
            result.push_str(&"─".repeat(width + 2));
            if i < col_widths.len() - 1 {
                result.push('┴');
            }
        }
        result.push_str("┘\n\n");
        
        result
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace('"', "&quot;")
     .replace('\'', "&#39;")
}

fn extract_tables_from_markdown(content: &str) -> Vec<Table> {
    let mut tables = Vec::new();
    let mut table_number = 1;
    
    // Split content into lines
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    
    while i < lines.len() {
        let line = lines[i].trim();
        
        // Look for markdown table start (line with pipes)
        if line.contains('|') && !line.is_empty() {
            let mut table_lines = Vec::new();
            let _start_index = i;
            
            // Collect all consecutive table lines
            while i < lines.len() {
                let current_line = lines[i].trim();
                if current_line.contains('|') && !current_line.is_empty() {
                    table_lines.push(current_line);
                    i += 1;
                } else if current_line.is_empty() {
                    i += 1;
                    continue;
                } else {
                    break;
                }
            }
            
            if table_lines.len() >= 2 { // Must have at least header and separator
                if let Some(table) = parse_markdown_table(&table_lines, table_number) {
                    tables.push(table);
                    table_number += 1;
                }
            }
        } else {
            i += 1;
        }
    }
    
    tables
}

fn parse_markdown_table(lines: &[&str], table_number: usize) -> Option<Table> {
    if lines.len() < 2 {
        return None;
    }
    
    // Parse header
    let header_line = lines[0];
    let headers: Vec<String> = header_line
        .split('|')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();
    
    if headers.is_empty() {
        return None;
    }
    
    // Skip separator line (line 1) and parse data rows
    let mut rows = Vec::new();
    for line in &lines[2..] {
        let row: Vec<String> = line
            .split('|')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();
        
        if !row.is_empty() {
            rows.push(row);
        }
    }
    
    Some(Table {
        headers,
        rows,
        table_number,
    })
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <input.md> [format]", args[0]);
        eprintln!("Formats: markdown (default), html, ascii, all");
        std::process::exit(1);
    }
    
    let input_file = &args[1];
    let format = args.get(2).map(|s| s.as_str()).unwrap_or("all");
    
    // Read input file
    let content = fs::read_to_string(input_file)?;
    println!("Reading markdown file: {}", input_file);
    
    // Extract tables
    let tables = extract_tables_from_markdown(&content);
    println!("Extracted {} tables", tables.len());
    
    if tables.is_empty() {
        println!("No tables found in the markdown file.");
        return Ok(());
    }
    
    let base_name = input_file.trim_end_matches(".md");
    
    match format {
        "markdown" => {
            let output_file = format!("{}_bordered.md", base_name);
            let mut output = String::new();
            output.push_str("# Bordered Tables for Quality Control\n\n");
            for table in &tables {
                output.push_str(&table.to_markdown());
            }
            fs::write(&output_file, output)?;
            println!("Markdown tables saved to: {}", output_file);
        },
        
        "html" => {
            let output_file = format!("{}_bordered.html", base_name);
            let mut output = String::new();
            output.push_str("<!DOCTYPE html>\n<html><head><title>Bordered Tables for QC</title></head><body>\n");
            output.push_str("<h1>Bordered Tables for Quality Control</h1>\n");
            for table in &tables {
                output.push_str(&table.to_html());
            }
            output.push_str("</body></html>");
            fs::write(&output_file, output)?;
            println!("HTML tables saved to: {}", output_file);
        },
        
        "ascii" => {
            let output_file = format!("{}_bordered.txt", base_name);
            let mut output = String::new();
            output.push_str("BORDERED TABLES FOR QUALITY CONTROL\n");
            output.push_str("====================================\n");
            for table in &tables {
                output.push_str(&table.to_ascii_table());
            }
            fs::write(&output_file, output)?;
            println!("ASCII tables saved to: {}", output_file);
        },
        
        "all" | _ => {
            // Markdown
            let md_file = format!("{}_bordered.md", base_name);
            let mut md_output = String::new();
            md_output.push_str("# Bordered Tables for Quality Control\n\n");
            for table in &tables {
                md_output.push_str(&table.to_markdown());
            }
            fs::write(&md_file, md_output)?;
            println!("Markdown tables saved to: {}", md_file);
            
            // HTML
            let html_file = format!("{}_bordered.html", base_name);
            let mut html_output = String::new();
            html_output.push_str("<!DOCTYPE html>\n<html><head>");
            html_output.push_str("<title>Bordered Tables for QC</title>");
            html_output.push_str("<style>body { font-family: Arial, sans-serif; margin: 20px; }</style>");
            html_output.push_str("</head><body>\n");
            html_output.push_str("<h1>Bordered Tables for Quality Control</h1>\n");
            for table in &tables {
                html_output.push_str(&table.to_html());
            }
            html_output.push_str("</body></html>");
            fs::write(&html_file, html_output)?;
            println!("HTML tables saved to: {}", html_file);
            
            // ASCII
            let ascii_file = format!("{}_bordered.txt", base_name);
            let mut ascii_output = String::new();
            ascii_output.push_str("BORDERED TABLES FOR QUALITY CONTROL\n");
            ascii_output.push_str("====================================\n");
            for table in &tables {
                ascii_output.push_str(&table.to_ascii_table());
            }
            fs::write(&ascii_file, ascii_output)?;
            println!("ASCII tables saved to: {}", ascii_file);
        }
    }
    
    println!("\nQuality Control Notes:");
    println!("- Compare each cell against the original PDF");
    println!("- Verify qualifiers (U, J, etc.) are in correct columns");
    println!("- Check that numeric values align properly");
    println!("- Ensure headers match the original table structure");
    println!("- Look for any shifted or missing data");
    
    Ok(())
}
