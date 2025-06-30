use std::path::PathBuf;
use clap::Parser;
use anyhow::Result;

#[derive(Parser)]
#[command(name = "clean_tables")]
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
    
    /// Output format: markdown, csv, or tsv
    #[arg(short, long, default_value = "markdown")]
    format: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Read the input file
    let content = tokio::fs::read_to_string(&cli.input).await?;
    println!("Read {} characters from {:?}", content.len(), cli.input);
    
    // Process the markdown with a simple table cleaner
    let (cleaned_content, changes) = clean_markdown_tables(&content);
    
    println!("Processed {} tables: {} fixed", changes.0, changes.1);
    
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

fn clean_markdown_tables(content: &str) -> (String, (usize, usize)) {
    let lines: Vec<&str> = content.lines().collect();
    let mut result_lines = Vec::new();
    let mut i = 0;
    let mut tables_found = 0;
    let mut tables_cleaned = 0;
    
    while i < lines.len() {
        let line = lines[i];
        
        // Check if this line looks like a table
        if line.contains("|") && line.chars().filter(|&c| c == '|').count() >= 3 {
            // Found potential table start
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
                .filter(|line| line.contains("|"))
                .copied()
                .collect();
            
            if !table_lines.is_empty() {
                tables_found += 1;
                
                // Clean and format the table
                let cleaned_table = format_table(&table_lines);
                result_lines.extend(cleaned_table.split('\n').map(|s| s.to_string()));
                tables_cleaned += 1;
            }
            
            i = table_end;
        } else {
            result_lines.push(line.to_string());
            i += 1;
        }
    }
    
    (result_lines.join("\n"), (tables_found, tables_cleaned))
}

fn format_table(table_lines: &[&str]) -> String {
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
        return String::new();
    }
    
    // Find the maximum number of columns
    let max_cols = parsed_rows.iter().map(|row| row.len()).max().unwrap_or(0);
    
    // Calculate column widths
    let mut col_widths = vec![0; max_cols];
    for row in &parsed_rows {
        for (i, cell) in row.iter().enumerate() {
            if i < max_cols {
                col_widths[i] = col_widths[i].max(cell.len());
            }
        }
    }
    
    // Format the table
    let mut formatted_lines = Vec::new();
    
    for (row_idx, row) in parsed_rows.iter().enumerate() {
        let mut formatted_cells = Vec::new();
        
        for i in 0..max_cols {
            let cell_content = row.get(i).map(|s| s.as_str()).unwrap_or("");
            let formatted_cell = format!(" {:<width$} ", cell_content, width = col_widths[i]);
            formatted_cells.push(formatted_cell);
        }
        
        let formatted_row = format!("| {} |", formatted_cells.join(" | "));
        formatted_lines.push(formatted_row);
        
        // Add separator after first row
        if row_idx == 0 {
            let separator_cells: Vec<String> = col_widths
                .iter()
                .map(|&width| "-".repeat(width + 2))
                .collect();
            let separator = format!("| {} |", separator_cells.join(" | "));
            formatted_lines.push(separator);
        }
    }
    
    formatted_lines.join("\n")
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
