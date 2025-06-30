use std::env;
use std::fs;
use std::io;
use std::process::Command;
use std::path::Path;

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
        
        // Calculate column widths for proper alignment
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
        
        // Minimum width of 8 for readability in Inlyne
        for width in &mut col_widths {
            *width = (*width).max(8);
        }

        result.push_str(&format!("\n## Table {} - Environmental Lab Data\n\n", self.table_number));
        
        // Add data quality notes
        result.push_str("> **Data Quality Notes:**\n");
        result.push_str("> - U = Undetected (below detection limit)\n");
        result.push_str("> - J = Estimated value (detected but below reporting limit)\n");
        result.push_str("> - Verify qualifiers are in correct columns\n\n");

        // Headers with proper alignment
        if !self.headers.is_empty() {
            result.push('|');
            for (i, header) in self.headers.iter().enumerate() {
                if i < col_widths.len() {
                    result.push_str(&format!(" {:^width$} |", header, width = col_widths[i]));
                } else {
                    result.push_str(&format!(" {} |", header));
                }
            }
            result.push('\n');
            
            // Separator row with proper alignment
            result.push('|');
            for (i, _) in self.headers.iter().enumerate() {
                if i < col_widths.len() {
                    result.push_str(&format!("{:-^width$}|", "", width = col_widths[i] + 2));
                } else {
                    result.push_str(":---:|");
                }
            }
            result.push('\n');
        }

        // Data rows with proper alignment
        for row in &self.rows {
            result.push('|');
            for (i, cell) in row.iter().enumerate() {
                // Detect if this might be a misplaced qualifier
                let formatted_cell = if cell.trim().len() == 1 && cell.trim().chars().all(|c| c.is_alphabetic()) {
                    format!("**{}**", cell) // Bold qualifiers for visibility
                } else {
                    cell.clone()
                };
                
                if i < col_widths.len() {
                    result.push_str(&format!(" {:^width$} |", formatted_cell, width = col_widths[i]));
                } else {
                    result.push_str(&format!(" {} |", formatted_cell));
                }
            }
            // Fill empty columns if row is shorter than headers
            for i in row.len()..col_widths.len() {
                result.push_str(&format!(" {:^width$} |", "", width = col_widths[i]));
            }
            result.push('\n');
        }
        
        result.push('\n');
        result.push_str("---\n\n"); // Section divider
        result
    }

    fn to_html(&self) -> String {
        let mut result = String::new();
        
        result.push_str(&format!("<div style=\"margin: 20px 0; page-break-inside: avoid;\">\n"));
        result.push_str(&format!("<h3 style=\"color: #333; border-bottom: 2px solid #007acc; padding-bottom: 5px;\">Table {}</h3>\n", self.table_number));
        result.push_str("<table style=\"border-collapse: collapse; font-family: 'Monaco', 'Menlo', monospace; font-size: 12px; width: 100%; margin: 10px 0;\">\n");
        
        // Headers with distinct styling
        if !self.headers.is_empty() {
            result.push_str("  <thead>\n");
            result.push_str("    <tr style=\"background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white;\">\n");
            for header in &self.headers {
                result.push_str(&format!("      <th style=\"border: 2px solid #333; padding: 8px; text-align: left; font-weight: bold;\">{}</th>\n", html_escape(header)));
            }
            result.push_str("    </tr>\n");
            result.push_str("  </thead>\n");
        }
        
        // Data rows with alternating colors for easy scanning
        if !self.rows.is_empty() {
            result.push_str("  <tbody>\n");
            for (row_idx, row) in self.rows.iter().enumerate() {
                let bg_color = if row_idx % 2 == 0 { "#f8f9fa" } else { "#ffffff" };
                result.push_str(&format!("    <tr style=\"background-color: {};\">\n", bg_color));
                
                for (col_idx, cell) in row.iter().enumerate() {
                    // Highlight potential qualifiers (U, J, etc.)
                    let cell_style = if cell.trim().len() == 1 && cell.trim().chars().all(|c| c.is_alphabetic()) {
                        "border: 2px solid #333; padding: 8px; background-color: #fff3cd; font-weight: bold; color: #856404;"
                    } else {
                        "border: 2px solid #333; padding: 8px;"
                    };
                    
                    result.push_str(&format!("      <td style=\"{}\">{}</td>\n", cell_style, html_escape(cell)));
                }
                
                // Fill empty columns if row is shorter than headers
                for _ in row.len()..self.headers.len() {
                    result.push_str("      <td style=\"border: 2px solid #333; padding: 8px; background-color: #f1f3f4;\"></td>\n");
                }
                result.push_str("    </tr>\n");
            }
            result.push_str("  </tbody>\n");
        }
        
        result.push_str("</table>\n");
        result.push_str("</div>\n");
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
    
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    
    while i < lines.len() {
        let line = lines[i].trim();
        
        if line.contains('|') && !line.is_empty() {
            let mut table_lines = Vec::new();
            
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
            
            if table_lines.len() >= 2 {
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

fn generate_markdown_report(tables: &[Table], pdf_file: &str) -> String {
    let mut markdown = String::new();
    
    markdown.push_str("# PDF Table Extraction - Quality Control Report\n\n");
    markdown.push_str(&format!("**Source:** `{}`\n", pdf_file));
    markdown.push_str(&format!("**Tables Found:** {}\n\n", tables.len()));
    
    markdown.push_str("## 🔍 Document Analysis Issues\n\n");
    markdown.push_str("> **Key Problem:** The extractor is processing tables without understanding document conventions.\n\n");
    
    markdown.push_str("### Missing Context:\n\n");
    markdown.push_str("- **Data Qualifiers:** U (Undetected), J (Estimated) should be in separate columns\n");
    markdown.push_str("- **Column Patterns:** Conc | Q | RL | MDL structure repeats throughout\n");
    markdown.push_str("- **Document Legend:** Header/footer likely explains data conventions\n\n");
    
    markdown.push_str("### Extraction Improvements Needed:\n\n");
    markdown.push_str("1. **Document-aware processing:** Pre-scan for data conventions/legends\n");
    markdown.push_str("2. **Pattern recognition:** Identify repeating column structures\n");
    markdown.push_str("3. **Context integration:** Use document context to guide table interpretation\n\n");
    
    markdown.push_str("---\n\n");
    
    for table in tables {
        markdown.push_str(&table.to_markdown());
    }
    
    markdown.push_str("## Quality Control Summary\n\n");
    markdown.push_str("### Validation Checklist:\n\n");
    markdown.push_str("- [ ] Headers match original PDF structure\n");
    markdown.push_str("- [ ] Qualifiers (U, J) are in correct columns (look for **bold** text)\n");
    markdown.push_str("- [ ] Numeric values didn't shift columns\n");
    markdown.push_str("- [ ] No missing data in critical fields\n");
    markdown.push_str("- [ ] Column patterns (Conc/Q/RL/MDL) are preserved\n\n");
    
    markdown
}

fn generate_html_report(tables: &[Table], pdf_file: &str) -> String {
    let mut html = String::new();
    
    html.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
    html.push_str("<meta charset=\"UTF-8\">\n");
    html.push_str("<title>PDF Table Extraction - Quality Control</title>\n");
    html.push_str("<style>\n");
    html.push_str("body { \n");
    html.push_str("  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;\n");
    html.push_str("  margin: 20px; \n");
    html.push_str("  background: linear-gradient(135deg, #f5f7fa 0%, #c3cfe2 100%);\n");
    html.push_str("  min-height: 100vh;\n");
    html.push_str("}\n");
    html.push_str(".header { \n");
    html.push_str("  background: white;\n");
    html.push_str("  padding: 20px;\n");
    html.push_str("  border-radius: 10px;\n");
    html.push_str("  box-shadow: 0 4px 6px rgba(0,0,0,0.1);\n");
    html.push_str("  margin-bottom: 20px;\n");
    html.push_str("}\n");
    html.push_str(".qc-note {\n");
    html.push_str("  background: #fff3cd;\n");
    html.push_str("  border: 1px solid #ffeaa7;\n");
    html.push_str("  padding: 15px;\n");
    html.push_str("  border-radius: 5px;\n");
    html.push_str("  margin: 20px 0;\n");
    html.push_str("}\n");
    html.push_str(".table-container {\n");
    html.push_str("  background: white;\n");
    html.push_str("  padding: 15px;\n");
    html.push_str("  border-radius: 10px;\n");
    html.push_str("  box-shadow: 0 2px 4px rgba(0,0,0,0.1);\n");
    html.push_str("  margin: 15px 0;\n");
    html.push_str("  overflow-x: auto;\n");
    html.push_str("}\n");
    html.push_str("</style>\n");
    html.push_str("</head>\n<body>\n");
    
    html.push_str("<div class=\"header\">\n");
    html.push_str("<h1 style=\"color: #2c3e50; margin: 0;\">📊 PDF Table Extraction - Quality Control</h1>\n");
    html.push_str(&format!("<p style=\"color: #7f8c8d; margin: 10px 0 0 0;\"><strong>Source:</strong> {}</p>\n", pdf_file));
    html.push_str(&format!("<p style=\"color: #7f8c8d; margin: 5px 0 0 0;\"><strong>Tables Found:</strong> {}</p>\n", tables.len()));
    html.push_str("</div>\n");
    
    html.push_str("<div class=\"qc-note\">\n");
    html.push_str("<h3 style=\"margin-top: 0; color: #856404;\">🔍 Quality Control Instructions</h3>\n");
    html.push_str("<ul style=\"margin: 10px 0;\">\n");
    html.push_str("<li><strong>Compare each cell</strong> against the original PDF</li>\n");
    html.push_str("<li><strong>Verify qualifiers</strong> (U, J, etc.) are in correct columns - highlighted in yellow</li>\n");
    html.push_str("<li><strong>Check numeric alignment</strong> - ensure values didn't shift columns</li>\n");
    html.push_str("<li><strong>Validate headers</strong> match the original table structure</li>\n");
    html.push_str("<li><strong>Look for missing data</strong> - empty cells are highlighted in gray</li>\n");
    html.push_str("</ul>\n");
    html.push_str("</div>\n");
    
    for table in tables {
        html.push_str("<div class=\"table-container\">\n");
        html.push_str(&table.to_html());
        html.push_str("</div>\n");
    }
    
    html.push_str("</body>\n</html>");
    html
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <input.pdf>", args[0]);
        eprintln!("This will extract tables from the PDF and show them in a quality control window.");
        std::process::exit(1);
    }
    
    let pdf_file = &args[1];
    
    if !Path::new(pdf_file).exists() {
        eprintln!("Error: PDF file '{}' does not exist", pdf_file);
        std::process::exit(1);
    }
    
    println!("🔄 Extracting tables from: {}", pdf_file);
    
    // Run the chonker extraction
    let output = Command::new("./target/release/chonker")
        .arg("extract")
        .arg(pdf_file)
        .output();
    
    let markdown_content = match output {
        Ok(result) => {
            if !result.status.success() {
                eprintln!("Error running chonker extractor:");
                eprintln!("{}", String::from_utf8_lossy(&result.stderr));
                std::process::exit(1);
            }
            
            // Look for the generated markdown file
            let base_name = pdf_file.trim_end_matches(".pdf");
            let possible_outputs = vec![
                format!("{}.md", base_name),
                "safe_test.md".to_string(),
                "output.md".to_string(),
            ];
            
            let mut markdown_content = String::new();
            for output_file in possible_outputs {
                if Path::new(&output_file).exists() {
                    markdown_content = fs::read_to_string(&output_file)?;
                    println!("✅ Found extraction output: {}", output_file);
                    break;
                }
            }
            
            if markdown_content.is_empty() {
                eprintln!("Error: Could not find markdown output file");
                std::process::exit(1);
            }
            
            markdown_content
        }
        Err(e) => {
            eprintln!("Error executing chonker: {}", e);
            eprintln!("Make sure you have built the chonker executable:");
            eprintln!("  cargo build --release");
            std::process::exit(1);
        }
    };
    
    // Extract tables
    let tables = extract_tables_from_markdown(&markdown_content);
    println!("📋 Extracted {} tables", tables.len());
    
    if tables.is_empty() {
        println!("ℹ️  No tables found in the extracted content.");
        return Ok(());
    }
    
    // Generate Markdown report for Inlyne
    let markdown_content = generate_markdown_report(&tables, pdf_file);
    let markdown_file = "table_qc_report.md";
    fs::write(markdown_file, markdown_content)?;
    
    println!("📝 Generated QC report: {}", markdown_file);
    
    // Check if Inlyne is available
    let inlyne_check = Command::new("inlyne")
        .arg("--version")
        .output();
    
    match inlyne_check {
        Ok(_) => {
            // Open with Inlyne for proper markdown table rendering
            let open_result = Command::new("inlyne")
                .arg(markdown_file)
                .status();
            
            match open_result {
                Ok(_) => {
                    println!("🚀 Opening quality control window with Inlyne...");
                    println!("💡 Use this window to compare tables against the original PDF");
                    println!("📋 Look for bold qualifiers (U, J) - they may be misplaced!");
                }
                Err(e) => {
                    eprintln!("Could not open with Inlyne: {}", e);
                    fallback_open(markdown_file);
                }
            }
        }
        Err(_) => {
            println!("ℹ️  Inlyne not found. Install with: cargo install inlyne");
            fallback_open(markdown_file);
        }
    }
}

fn fallback_open(markdown_file: &str) {
    // Fallback to system default
    let open_result = Command::new("open")
        .arg(markdown_file)
        .status();
    
    match open_result {
        Ok(_) => {
            println!("🚀 Opening quality control window...");
            println!("💡 Use this window to compare tables against the original PDF");
        }
        Err(e) => {
            eprintln!("Could not open automatically: {}", e);
            eprintln!("Please open {} manually", markdown_file);
        }
    }
}
