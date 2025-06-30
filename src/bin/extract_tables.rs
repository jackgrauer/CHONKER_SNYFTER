use std::path::PathBuf;
use clap::Parser;
use anyhow::Result;

#[derive(Parser)]
#[command(name = "extract_tables")]
#[command(about = "Extract and format tables from PDF markdown with multiple output formats")]
struct Cli {
    /// Input markdown file
    input: PathBuf,
    
    /// Output file (optional, defaults to input_tables.{format})
    #[arg(short, long)]
    output: Option<PathBuf>,
    
    /// Output format: markdown, csv, tsv, or xlsx
    #[arg(short, long, default_value = "csv")]
    format: String,
    
    /// Open output file after creation
    #[arg(long)]
    open: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Read the input file
    let content = tokio::fs::read_to_string(&cli.input).await?;
    println!("Read {} characters from {:?}", content.len(), cli.input);
    
    // Extract tables
    let tables = extract_tables_from_markdown(&content);
    println!("Found {} tables", tables.len());
    
    // Process based on format
    match cli.format.as_str() {
        "csv" => {
            let output_path = cli.output.unwrap_or_else(|| {
                let mut path = cli.input.clone();
                let stem = path.file_stem().unwrap().to_string_lossy();
                path.set_file_name(format!("{}_tables.csv", stem));
                path
            });
            
            let csv_content = tables_to_csv(&tables);
            tokio::fs::write(&output_path, csv_content).await?;
            println!("Wrote {} tables to CSV: {:?}", tables.len(), output_path);
            
            if cli.open {
                open_file(&output_path);
            }
        }
        "tsv" => {
            let output_path = cli.output.unwrap_or_else(|| {
                let mut path = cli.input.clone();
                let stem = path.file_stem().unwrap().to_string_lossy();
                path.set_file_name(format!("{}_tables.tsv", stem));
                path
            });
            
            let tsv_content = tables_to_tsv(&tables);
            tokio::fs::write(&output_path, tsv_content).await?;
            println!("Wrote {} tables to TSV: {:?}", tables.len(), output_path);
            
            if cli.open {
                open_file(&output_path);
            }
        }
        "markdown" => {
            let output_path = cli.output.unwrap_or_else(|| {
                let mut path = cli.input.clone();
                let stem = path.file_stem().unwrap().to_string_lossy();
                path.set_file_name(format!("{}_tables.md", stem));
                path
            });
            
            let md_content = tables_to_clean_markdown(&tables);
            tokio::fs::write(&output_path, md_content).await?;
            println!("Wrote {} tables to Markdown: {:?}", tables.len(), output_path);
            
            if cli.open {
                open_file(&output_path);
            }
        }
        _ => {
            println!("Unsupported format: {}. Use csv, tsv, or markdown", cli.format);
            return Ok(());
        }
    }
    
    Ok(())
}

#[derive(Debug, Clone)]
struct Table {
    title: String,
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
}

fn extract_tables_from_markdown(content: &str) -> Vec<Table> {
    let lines: Vec<&str> = content.lines().collect();
    let mut tables = Vec::new();
    let mut i = 0;
    
    while i < lines.len() {
        let line = lines[i];
        
        // Check if this line looks like a table
        if line.contains("|") && line.chars().filter(|&c| c == '|').count() >= 3 {
            // Look for table title in previous lines
            let mut title = String::new();
            if i > 0 {
                let prev_line = lines[i-1].trim();
                if prev_line.starts_with("##") || prev_line.starts_with("Table") || prev_line.starts_with("#") {
                    title = prev_line.replace("#", "").trim().to_string();
                }
            }
            
            // Extract table data
            let table_start = i;
            let mut table_end = i;
            
            // Find the end of this table
            while table_end < lines.len() && 
                  (lines[table_end].contains("|") || lines[table_end].trim().is_empty()) {
                table_end += 1;
            }
            
            // Extract table lines
            let table_lines: Vec<&str> = lines[table_start..table_end]
                .iter()
                .filter(|line| line.contains("|") && !line.trim_start().starts_with("|-"))
                .copied()
                .collect();
            
            if !table_lines.is_empty() {
                let table = parse_table_lines(&table_lines, title);
                if !table.rows.is_empty() {
                    tables.push(table);
                }
            }
            
            i = table_end;
        } else {
            i += 1;
        }
    }
    
    tables
}

fn parse_table_lines(table_lines: &[&str], title: String) -> Table {
    let mut parsed_rows = Vec::new();
    
    // Parse each row into cells
    for line in table_lines {
        let cells: Vec<String> = line
            .split('|')
            .map(|cell| cell.trim().to_string())
            .filter(|cell| !cell.is_empty())
            .collect();
        
        if !cells.is_empty() {
            parsed_rows.push(cells);
        }
    }
    
    if parsed_rows.is_empty() {
        return Table {
            title,
            headers: Vec::new(),
            rows: Vec::new(),
        };
    }
    
    // First row is headers, rest are data
    let headers = parsed_rows[0].clone();
    let rows = if parsed_rows.len() > 1 {
        parsed_rows[1..].to_vec()
    } else {
        Vec::new()
    };
    
    Table {
        title,
        headers,
        rows,
    }
}

fn tables_to_csv(tables: &[Table]) -> String {
    let mut csv_content = String::new();
    
    for (table_idx, table) in tables.iter().enumerate() {
        // Add table title as a comment
        csv_content.push_str(&format!("# Table {}: {}\n", table_idx + 1, table.title));
        
        // Add headers
        let headers_csv = table.headers.iter()
            .map(|h| escape_csv_field(h))
            .collect::<Vec<_>>()
            .join(",");
        csv_content.push_str(&headers_csv);
        csv_content.push('\n');
        
        // Add rows
        for row in &table.rows {
            let row_csv = row.iter()
                .map(|cell| escape_csv_field(cell))
                .collect::<Vec<_>>()
                .join(",");
            csv_content.push_str(&row_csv);
            csv_content.push('\n');
        }
        
        // Add blank line between tables
        csv_content.push('\n');
    }
    
    csv_content
}

fn tables_to_tsv(tables: &[Table]) -> String {
    let mut tsv_content = String::new();
    
    for (table_idx, table) in tables.iter().enumerate() {
        // Add table title as a comment
        tsv_content.push_str(&format!("# Table {}: {}\n", table_idx + 1, table.title));
        
        // Add headers
        let headers_tsv = table.headers.join("\t");
        tsv_content.push_str(&headers_tsv);
        tsv_content.push('\n');
        
        // Add rows
        for row in &table.rows {
            let row_tsv = row.join("\t");
            tsv_content.push_str(&row_tsv);
            tsv_content.push('\n');
        }
        
        // Add blank line between tables
        tsv_content.push('\n');
    }
    
    tsv_content
}

fn tables_to_clean_markdown(tables: &[Table]) -> String {
    let mut md_content = String::new();
    
    for (table_idx, table) in tables.iter().enumerate() {
        md_content.push_str(&format!("## Table {}: {}\n\n", table_idx + 1, table.title));
        
        if table.headers.is_empty() || table.rows.is_empty() {
            md_content.push_str("*No data available*\n\n");
            continue;
        }
        
        // Calculate column widths
        let max_cols = table.headers.len().max(
            table.rows.iter().map(|row| row.len()).max().unwrap_or(0)
        );
        
        let mut col_widths = vec![0; max_cols];
        
        // Check header widths
        for (i, header) in table.headers.iter().enumerate() {
            if i < max_cols {
                col_widths[i] = col_widths[i].max(header.len());
            }
        }
        
        // Check row widths
        for row in &table.rows {
            for (i, cell) in row.iter().enumerate() {
                if i < max_cols {
                    col_widths[i] = col_widths[i].max(cell.len());
                }
            }
        }
        
        // Format headers
        let mut formatted_headers = Vec::new();
        for i in 0..max_cols {
            let header = table.headers.get(i).map(|s| s.as_str()).unwrap_or("");
            formatted_headers.push(format!(" {:<width$} ", header, width = col_widths[i]));
        }
        md_content.push_str(&format!("|{}|\n", formatted_headers.join("|")));
        
        // Format separator
        let separator_cells: Vec<String> = col_widths
            .iter()
            .map(|&width| "-".repeat(width + 2))
            .collect();
        md_content.push_str(&format!("|{}|\n", separator_cells.join("|")));
        
        // Format rows
        for row in &table.rows {
            let mut formatted_cells = Vec::new();
            for i in 0..max_cols {
                let cell = row.get(i).map(|s| s.as_str()).unwrap_or("");
                formatted_cells.push(format!(" {:<width$} ", cell, width = col_widths[i]));
            }
            md_content.push_str(&format!("|{}|\n", formatted_cells.join("|")));
        }
        
        md_content.push('\n');
    }
    
    md_content
}

fn escape_csv_field(field: &str) -> String {
    if field.contains(',') || field.contains('"') || field.contains('\n') {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}

fn open_file(path: &PathBuf) {
    if cfg!(target_os = "macos") {
        let _ = std::process::Command::new("open")
            .arg(path)
            .spawn();
    }
}
