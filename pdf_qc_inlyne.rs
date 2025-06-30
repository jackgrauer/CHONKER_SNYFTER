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
        
        // Calculate column widths for proper alignment in Inlyne
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
        
        // Minimum width of 10 for clear grid lines in Inlyne
        for width in &mut col_widths {
            *width = (*width).max(10);
        }

        result.push_str(&format!("\n## 📋 Table {} - Environmental Lab Data\n\n", self.table_number));
        
        // Add context-aware notes
        result.push_str("> **⚠️ Document Convention Analysis:**\n");
        result.push_str("> - **U** = Undetected (below detection limit) - should be in separate qualifier column\n");
        result.push_str("> - **J** = Estimated value (detected but below reporting limit) - should be in separate qualifier column\n");
        result.push_str("> - **Expected Pattern:** Concentration | Qualifier | Reporting Limit | Method Detection Limit\n");
        result.push_str("> - **Look for:** Values like '0.046 U' that should be split into '0.046' and 'U'\n\n");

        // Headers with proper alignment for Inlyne grid rendering
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
            
            // Separator row with centered alignment for better grid appearance
            result.push('|');
            for (i, _) in self.headers.iter().enumerate() {
                if i < col_widths.len() {
                    result.push_str(&format!(":{:-^width$}:|", "", width = col_widths[i]));
                } else {
                    result.push_str(":---:|");
                }
            }
            result.push('\n');
        }

        // Data rows with qualifier highlighting
        for (row_idx, row) in self.rows.iter().enumerate() {
            result.push('|');
            for (i, cell) in row.iter().enumerate() {
                // Detect potential misplaced qualifiers and highlight them
                let formatted_cell = if self.is_likely_misplaced_qualifier(cell) {
                    format!("🔴 **{}** 🔴", cell) // Red circles to highlight problems
                } else if self.is_single_qualifier(cell) {
                    format!("⚠️ **{}**", cell) // Warning for standalone qualifiers
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
                result.push_str(&format!(" {:^width$} |", "❌ *EMPTY*", width = col_widths[i]));
            }
            result.push('\n');
        }
        
        result.push('\n');
        
        // Add pattern analysis for this table
        self.analyze_table_patterns(&mut result);
        
        result.push_str("---\n\n"); // Section divider for Inlyne
        result
    }
    
    fn is_likely_misplaced_qualifier(&self, cell: &str) -> bool {
        // Check for patterns like "0.046 U", "170 J", etc.
        let parts: Vec<&str> = cell.split_whitespace().collect();
        if parts.len() == 2 {
            if let Ok(_) = parts[0].parse::<f64>() {
                return parts[1].len() == 1 && parts[1].chars().all(|c| c.is_alphabetic());
            }
        }
        false
    }
    
    fn is_single_qualifier(&self, cell: &str) -> bool {
        cell.trim().len() == 1 && cell.trim().chars().all(|c| c.is_alphabetic())
    }
    
    fn analyze_table_patterns(&self, result: &mut String) {
        result.push_str("### 🔍 Pattern Analysis for this Table:\n\n");
        
        let mut issues_found = Vec::new();
        let mut total_misplaced = 0;
        let mut total_single_qualifiers = 0;
        
        for row in &self.rows {
            for cell in row {
                if self.is_likely_misplaced_qualifier(cell) {
                    total_misplaced += 1;
                    issues_found.push(format!("- 🔴 **Misplaced qualifier detected:** `{}` (should be split)", cell));
                } else if self.is_single_qualifier(cell) {
                    total_single_qualifiers += 1;
                }
            }
        }
        
        if total_misplaced > 0 {
            result.push_str(&format!("**❌ {} misplaced qualifiers found!**\n", total_misplaced));
            for issue in &issues_found[..issues_found.len().min(5)] { // Show max 5 examples
                result.push_str(&format!("{}\n", issue));
            }
            if issues_found.len() > 5 {
                result.push_str(&format!("- ... and {} more\n", issues_found.len() - 5));
            }
        } else {
            result.push_str("✅ **No obviously misplaced qualifiers detected**\n");
        }
        
        if total_single_qualifiers > 0 {
            result.push_str(&format!("⚠️ **{} standalone qualifiers found** (verify they're in correct columns)\n", total_single_qualifiers));
        }
        
        result.push('\n');
    }
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

fn generate_qc_report(tables: &[Table], pdf_file: &str) -> String {
    let mut markdown = String::new();
    
    markdown.push_str("# 📊 PDF Table Extraction - Quality Control Report\n\n");
    markdown.push_str(&format!("**📄 Source:** `{}`  \n", pdf_file));
    markdown.push_str(&format!("**📋 Tables Found:** {}  \n", tables.len()));
    markdown.push_str("**🕒 Generated:** Just now  \n\n");
    
    markdown.push_str("## 🚨 Critical Document Understanding Issue\n\n");
    markdown.push_str("> **Root Problem:** The extractor processes tables without understanding environmental lab document conventions.\n\n");
    
    markdown.push_str("### 📖 Missing Document Context:\n\n");
    markdown.push_str("1. **Data Qualifiers Convention:**\n");
    markdown.push_str("   - `U` = Undetected (below detection limit)\n");
    markdown.push_str("   - `J` = Estimated value (detected but below reporting limit)\n");
    markdown.push_str("   - These should ALWAYS be in separate columns from numeric values\n\n");
    
    markdown.push_str("2. **Expected Column Structure Pattern:**\n");
    markdown.push_str("   - `Concentration | Qualifier | Reporting Limit | Method Detection Limit`\n");
    markdown.push_str("   - This pattern repeats for each analyte group\n\n");
    
    markdown.push_str("3. **Document Legend/Header:**\n");
    markdown.push_str("   - Document likely contains qualifier definitions\n");
    markdown.push_str("   - Pre-scanning could inform extraction logic\n\n");
    
    markdown.push_str("### 🔧 Suggested Extraction Improvements:\n\n");
    markdown.push_str("1. **Document-aware preprocessing:** Scan for data conventions and legends\n");
    markdown.push_str("2. **Pattern recognition:** Detect repeating column structures (Conc/Q/RL/MDL)\n");
    markdown.push_str("3. **Context integration:** Use document metadata to guide table interpretation\n");
    markdown.push_str("4. **Qualifier separation:** Automatically split combined values like '0.046 U'\n\n");
    
    markdown.push_str("---\n\n");
    
    // Overall statistics
    let mut total_misplaced = 0;
    let mut total_qualifiers = 0;
    
    for table in tables {
        for row in &table.rows {
            for cell in row {
                if table.is_likely_misplaced_qualifier(cell) {
                    total_misplaced += 1;
                }
                if table.is_single_qualifier(cell) {
                    total_qualifiers += 1;
                }
            }
        }
    }
    
    markdown.push_str("## 📈 Extraction Quality Summary\n\n");
    if total_misplaced > 0 {
        markdown.push_str(&format!("🔴 **{} likely misplaced qualifiers detected across all tables**\n\n", total_misplaced));
    } else {
        markdown.push_str("✅ **No obviously misplaced qualifiers detected**\n\n");
    }
    
    if total_qualifiers > 0 {
        markdown.push_str(&format!("⚠️ **{} standalone qualifiers found** - verify column placement\n\n", total_qualifiers));
    }
    
    markdown.push_str("---\n\n");
    
    // Individual tables
    for table in tables {
        markdown.push_str(&table.to_markdown());
    }
    
    markdown.push_str("## ✅ Quality Control Validation Checklist\n\n");
    markdown.push_str("Use this checklist while comparing against the original PDF:\n\n");
    markdown.push_str("- [ ] **Headers:** Match original PDF table structure exactly\n");
    markdown.push_str("- [ ] **Qualifiers:** All 🔴 marked items are extraction errors needing fixing\n");
    markdown.push_str("- [ ] **Numeric alignment:** Values didn't shift between columns\n");
    markdown.push_str("- [ ] **Missing data:** No ❌ *EMPTY* cells where data should exist\n");
    markdown.push_str("- [ ] **Column patterns:** Conc/Q/RL/MDL structure preserved where expected\n");
    markdown.push_str("- [ ] **Data integrity:** All lab qualifiers (U, J, etc.) in proper qualifier columns\n\n");
    
    markdown.push_str("---\n\n");
    markdown.push_str("*This report was generated automatically. For compliance reporting, manual validation against the original PDF is required.*\n");
    
    markdown
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <input.pdf>", args[0]);
        eprintln!("Extracts tables from PDF and opens quality control viewer with Inlyne.");
        eprintln!("Install Inlyne: cargo install inlyne");
        std::process::exit(1);
    }
    
    let pdf_file = &args[1];
    
    if !Path::new(pdf_file).exists() {
        eprintln!("❌ Error: PDF file '{}' does not exist", pdf_file);
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
                eprintln!("❌ Error running chonker extractor:");
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
                eprintln!("❌ Error: Could not find markdown output file");
                std::process::exit(1);
            }
            
            markdown_content
        }
        Err(e) => {
            eprintln!("❌ Error executing chonker: {}", e);
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
    
    // Generate QC report optimized for Inlyne
    let report_content = generate_qc_report(&tables, pdf_file);
    let report_file = "pdf_table_qc_report.md";
    fs::write(report_file, report_content)?;
    
    println!("📝 Generated QC report: {}", report_file);
    
    // Try to open with Inlyne
    let inlyne_result = Command::new("inlyne")
        .arg(report_file)
        .spawn();
    
    match inlyne_result {
        Ok(_) => {
            println!("🚀 Opening quality control report with Inlyne...");
            println!("💡 Look for 🔴 markers - these indicate extraction problems!");
            println!("📋 Use the grid view to compare against your original PDF");
            println!("⚠️ Pay special attention to qualifiers (U, J) placement");
        }
        Err(_) => {
            println!("ℹ️  Inlyne not found. Installing...");
            let install_result = Command::new("cargo")
                .args(&["install", "inlyne"])
                .status();
            
            match install_result {
                Ok(status) if status.success() => {
                    println!("✅ Inlyne installed successfully!");
                    println!("🚀 Opening report...");
                    let _ = Command::new("inlyne").arg(report_file).spawn();
                }
                _ => {
                    println!("❌ Could not install Inlyne automatically.");
                    println!("Please install manually: cargo install inlyne");
                    println!("Then open: inlyne {}", report_file);
                    
                    // Fallback to system default
                    let _ = Command::new("open").arg(report_file).status();
                }
            }
        }
    }
    
    Ok(())
}
