use crate::chonker_types::{DocumentChunk, TableData};
use serde_json::Value;

/// HTML renderer for formatted document output display
pub struct HtmlRenderer {
    theme: RenderTheme,
}

#[derive(Clone)]
pub struct RenderTheme {
    pub primary_color: String,
    pub secondary_color: String,
    pub background_color: String,
    pub text_color: String,
    pub table_border_color: String,
}

impl Default for RenderTheme {
    fn default() -> Self {
        Self {
            primary_color: "#00ff00".to_string(),
            secondary_color: "#ff1493".to_string(), 
            background_color: "#111111".to_string(),
            text_color: "#ffffff".to_string(),
            table_border_color: "#333333".to_string(),
        }
    }
}

impl HtmlRenderer {
    pub fn new() -> Self {
        Self {
            theme: RenderTheme::default(),
        }
    }

    #[allow(dead_code)]
    pub fn with_theme(theme: RenderTheme) -> Self {
        Self { theme }
    }

    /// Render processing results as formatted HTML
    pub fn render_processing_results(&self, data: &Value) -> String {
        let mut html = String::new();
        
        // Add CSS styling
        html.push_str(&self.generate_css());
        
        // Create stats section
        html.push_str(&self.render_stats_section(data));
        
        // Render tables if available
        if let Some(tables) = data["tables"].as_array() {
            for (index, table) in tables.iter().enumerate() {
                html.push_str(&self.render_table(table, index + 1));
            }
        }
        
        html
    }

    /// Generate CSS styles for the HTML output
    fn generate_css(&self) -> String {
        format!(r#"
<style>
    .chonker-output {{
        font-family: 'Hack', 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace;
        background: {background};
        color: {text};
        margin: 0;
        padding: 15px;
        line-height: 1.5;
    }}
    
    .stats-section {{
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
        gap: 15px;
        margin-bottom: 25px;
        padding: 20px;
        background: rgba(0, 255, 0, 0.05);
        border: 1px solid {primary};
        border-radius: 8px;
    }}
    
    .stat-card {{
        text-align: center;
        padding: 15px;
        background: rgba(0, 0, 0, 0.3);
        border: 1px solid {table_border};
        border-radius: 6px;
        transition: all 0.3s ease;
    }}
    
    .stat-card:hover {{
        border-color: {primary};
        box-shadow: 0 0 10px rgba(0, 255, 0, 0.3);
    }}
    
    .stat-value {{
        font-size: 24px;
        font-weight: bold;
        color: {primary};
        margin-bottom: 5px;
        text-shadow: 0 0 5px rgba(0, 255, 0, 0.5);
    }}
    
    .stat-label {{
        font-size: 12px;
        color: {secondary};
        text-transform: uppercase;
        letter-spacing: 1px;
    }}
    
    .table-container {{
        margin: 40px 0;
        background: rgba(17, 17, 17, 0.95);
        border: 1px solid {primary};
        border-radius: 10px;
        padding: 20px;
        box-shadow: 0 4px 15px rgba(0, 255, 0, 0.2);
        overflow: hidden;
    }}
    
    .table-header {{
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 15px;
        padding-bottom: 10px;
        border-bottom: 2px solid {secondary};
    }}
    
    .table-title {{
        font-size: 18px;
        font-weight: bold;
        color: {secondary};
        margin: 0;
        text-shadow: 0 0 5px rgba(255, 20, 147, 0.5);
    }}
    
    .table-info {{
        font-size: 12px;
        color: {text};
        opacity: 0.8;
    }}
    
    .chonker-table {{
        width: 100%;
        border-collapse: collapse;
        margin: 0;
        background: rgba(0, 0, 0, 0.8);
        border-radius: 6px;
        overflow: hidden;
    }}
    
    .chonker-table th {{
        background: linear-gradient(135deg, {primary} 0%, {secondary} 100%);
        color: #000000;
        padding: 12px 8px;
        text-align: left;
        font-weight: bold;
        font-size: 14px;
        border: 1px solid {primary};
        position: sticky;
        top: 0;
        z-index: 10;
    }}
    
    .chonker-table td {{
        padding: 10px 8px;
        border: 1px solid {table_border};
        background: rgba(17, 17, 17, 0.9);
        vertical-align: top;
        font-size: 13px;
    }}
    
    .chonker-table td.numeric {{
        text-align: right;
        font-family: 'Courier New', monospace;
        color: #00ffff;
        font-weight: 500;
    }}
    
    .chonker-table td.text {{
        color: {text};
    }}
    
    .chonker-table tr:nth-child(even) td {{
        background: rgba(0, 255, 0, 0.02);
    }}
    
    .chonker-table tr:hover td {{
        background: rgba(0, 255, 0, 0.1);
        box-shadow: inset 0 0 10px rgba(0, 255, 0, 0.2);
    }}
    
    .empty-cell {{
        color: #666666;
        font-style: italic;
        text-align: center;
    }}
    
    .processing-info {{
        margin-top: 20px;
        padding: 15px;
        background: rgba(255, 20, 147, 0.1);
        border: 1px solid {secondary};
        border-radius: 6px;
        font-size: 12px;
        color: {secondary};
    }}
    
    /* Content type styling */
    .content-container {{
        margin: 30px 0;
        padding: 15px;
        background: rgba(17, 17, 17, 0.8);
        border-radius: 6px;
        border-left: 4px solid {primary};
    }}
    
    .content-heading {{
        background: rgba(0, 255, 0, 0.1);
        border-left-color: {primary};
        font-size: 18px;
        font-weight: bold;
        color: {primary};
        text-shadow: 0 0 5px rgba(0, 255, 0, 0.5);
    }}
    
    .content-text {{
        background: rgba(17, 17, 17, 0.9);
        border-left-color: {text};
        color: {text};
        line-height: 1.6;
    }}
    
    .content-caption {{
        background: rgba(255, 20, 147, 0.1);
        border-left-color: {secondary};
        color: {secondary};
        font-style: italic;
        text-align: center;
        font-size: 14px;
    }}
    
    .content-list {{
        background: rgba(0, 255, 255, 0.05);
        border-left-color: #00ffff;
        color: {text};
    }}
    
    .content-list ul, .content-list ol {{
        margin: 10px 0;
        padding-left: 20px;
    }}
    
    .content-list li {{
        margin: 5px 0;
        color: #00ffff;
    }}
    
    .emoji {{
        font-size: 16px;
        margin-right: 5px;
    }}
    
    /* Scrollbar styling */
    .table-container::-webkit-scrollbar {{
        width: 8px;
        height: 8px;
    }}
    
    .table-container::-webkit-scrollbar-track {{
        background: {table_border};
        border-radius: 4px;
    }}
    
    .table-container::-webkit-scrollbar-thumb {{
        background: {primary};
        border-radius: 4px;
    }}
    
    .table-container::-webkit-scrollbar-thumb:hover {{
        background: {secondary};
    }}
    
    /* Bidirectional selection highlighting */
    .chunk-highlighted {{
        background: rgba(0, 255, 255, 0.2) !important;
        border: 2px solid #00ffff !important;
        box-shadow: 0 0 20px rgba(0, 255, 255, 0.5) !important;
        transform: scale(1.02);
        transition: all 0.3s ease;
    }}
    
    .content-container.chunk-highlighted {{
        background: rgba(0, 255, 255, 0.15) !important;
        border-left: 4px solid #00ffff !important;
    }}
    
    .chonker-table.chunk-highlighted {{
        border: 2px solid #00ffff !important;
    }}
    
    .chonker-table.chunk-highlighted td,
    .chonker-table.chunk-highlighted th {{
        background: rgba(0, 255, 255, 0.1) !important;
    }}
    
    [data-chunk-id] {{
        cursor: pointer;
        transition: all 0.2s ease;
    }}
    
    [data-chunk-id]:hover {{
        background: rgba(0, 255, 0, 0.1) !important;
        border-color: {primary} !important;
    }}
</style>
"#, 
            primary = self.theme.primary_color,
            secondary = self.theme.secondary_color,
            background = self.theme.background_color,
            text = self.theme.text_color,
            table_border = self.theme.table_border_color
        )
    }

    /// Render the stats section with processing metrics
    fn render_stats_section(&self, data: &Value) -> String {
        let stats = [
            ("tables_found", "Tables Found", "📊"),
            ("chunks_extracted", "Chunks", "📄"),
            ("formulas_detected", "Formulas", "📐"),
            ("pages_processed", "Pages", "📖"),
        ];

        let stats_html = stats.iter()
            .map(|(key, label, emoji)| {
                let value = data[key].as_u64().unwrap_or(0);
                format!(r#"
                    <div class="stat-card">
                        <div class="stat-value"><span class="emoji">{}</span>{}</div>
                        <div class="stat-label">{}</div>
                    </div>
                "#, emoji, value, label)
            })
            .collect::<Vec<_>>()
            .join("");

        format!(r#"
            <div class="chonker-output">
                <div class="stats-section">
                    {}
                </div>
        "#, stats_html)
    }

    /// Render an individual table with chunk highlighting support
    fn render_table(&self, table: &Value, table_index: usize) -> String {
        let empty_vec = vec![];
        let headers = table["headers"].as_array().unwrap_or(&empty_vec);
        let rows = table["rows"].as_array().unwrap_or(&empty_vec);
        
        let mut html = format!(r#"
            <div class="table-container">
                <div class="table-header">
                    <h3 class="table-title">🐹 Table {}</h3>
                </div>
                <div style="overflow-x: auto;">
                    <table class="chonker-table">
        "#, table_index);

        // Render headers
        if !headers.is_empty() {
            html.push_str("                        <thead>\n                            <tr>\n");
            for header in headers {
                let header_text = header.as_str().unwrap_or("");
                html.push_str(&format!(
                    "                                <th>{}</th>\n", 
                    self.html_escape(header_text)
                ));
            }
            html.push_str("                            </tr>\n                        </thead>\n");
        }

        // Render data rows
        html.push_str("                        <tbody>\n");
        for row in rows {
            if let Some(row_array) = row.as_array() {
                html.push_str("                            <tr>\n");
                for (_i, cell) in row_array.iter().enumerate() {
                    let cell_text = cell.as_str().unwrap_or("");
                    let cell_class = if self.is_numeric_content(cell_text) { "numeric" } else { "text" };
                    let display_text = if cell_text.trim().is_empty() { 
                        "<span class=\"empty-cell\">—</span>".to_string()
                    } else { 
                        self.html_escape(cell_text) 
                    };
                    
                    html.push_str(&format!(
                        "                                <td class=\"{}\">{}</td>\n",
                        cell_class, display_text
                    ));
                }
                html.push_str("                            </tr>\n");
            }
        }
        html.push_str("                        </tbody>\n");
        
        html.push_str("                    </table>\n");
        html.push_str("                </div>\n");
        html.push_str("            </div>\n");

        html
    }

    /// Check if content appears to be numeric
    fn is_numeric_content(&self, text: &str) -> bool {
        if text.trim().is_empty() {
            return false;
        }
        
        text.trim().chars().all(|c| {
            c.is_ascii_digit() || c == '.' || c == ',' || c == '-' || 
            c == '%' || c == '$' || c == '(' || c == ')' || c.is_whitespace()
        })
    }

    /// Escape HTML entities
    fn html_escape(&self, text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;")
    }
    
    /// Format text content with paragraph breaks
    fn format_text_content(&self, text: &str) -> String {
        let paragraphs: Vec<&str> = text.split("\n\n").collect();
        paragraphs.iter()
            .map(|para| {
                let escaped = self.html_escape(para.trim());
                if !escaped.is_empty() {
                    format!("<p>{}</p>", escaped)
                } else {
                    String::new()
                }
            })
            .filter(|p| !p.is_empty())
            .collect::<Vec<String>>()
            .join("\n")
    }
    
    /// Format list content as proper HTML lists
    fn format_list_content(&self, text: &str) -> String {
        let mut html = String::new();
        let lines: Vec<&str> = text.lines().collect();
        let mut in_ordered_list = false;
        let mut in_unordered_list = false;
        
        for line in lines {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            
            // Check for ordered list items (1. 2. 3. etc.)
            if trimmed.chars().next().map_or(false, |c| c.is_ascii_digit()) && 
               trimmed.contains('.') && trimmed.len() > 2 {
                if !in_ordered_list {
                    if in_unordered_list {
                        html.push_str("</ul>\n");
                        in_unordered_list = false;
                    }
                    html.push_str("<ol>\n");
                    in_ordered_list = true;
                }
                
                // Extract content after the number and dot
                if let Some(dot_pos) = trimmed.find('.') {
                    let content = trimmed[dot_pos + 1..].trim();
                    html.push_str(&format!("  <li>{}</li>\n", self.html_escape(content)));
                }
            }
            // Check for unordered list items (-, *, +, •)
            else if trimmed.starts_with('-') || trimmed.starts_with('*') || 
                    trimmed.starts_with('+') || trimmed.starts_with('•') {
                if !in_unordered_list {
                    if in_ordered_list {
                        html.push_str("</ol>\n");
                        in_ordered_list = false;
                    }
                    html.push_str("<ul>\n");
                    in_unordered_list = true;
                }
                
                let content = trimmed[1..].trim();
                html.push_str(&format!("  <li>{}</li>\n", self.html_escape(content)));
            }
            // Regular text line
            else {
                if in_ordered_list {
                    html.push_str("</ol>\n");
                    in_ordered_list = false;
                }
                if in_unordered_list {
                    html.push_str("</ul>\n");
                    in_unordered_list = false;
                }
                html.push_str(&format!("<p>{}</p>\n", self.html_escape(trimmed)));
            }
        }
        
        // Close any open lists
        if in_ordered_list {
            html.push_str("</ol>\n");
        }
        if in_unordered_list {
            html.push_str("</ul>\n");
        }
        
        html
    }

    /// Render chunks as formatted HTML (alternative method)
    pub fn render_document_chunks(&self, chunks: &[DocumentChunk]) -> String {
        let mut html = String::new();
        html.push_str(&self.generate_css());
        
        html.push_str(r#"<div class="chonker-output">"#);
        
        for (index, chunk) in chunks.iter().enumerate() {
            html.push_str(&self.render_chunk(chunk, index + 1));
        }
        
        html.push_str("</div>");
        html
    }

    /// Render individual document chunk
    #[allow(dead_code)]
    fn render_chunk(&self, chunk: &DocumentChunk, chunk_index: usize) -> String {
        let chunk_id = format!("chunk-{}", chunk_index);
        match chunk.content_type.as_str() {
        "table" => {
            if let Some(table_data) = &chunk.table_data {
                let mut table_html = self.render_table_data(table_data, chunk_index);
                
                // Add data-chunk-id for bidirectional selection
                table_html = table_html.replace("<table class=\"chonker-table\"", &format!("<table class=\"chonker-table\" data-chunk-id=\"{}\"", chunk_id));
                table_html
            } else {
                format!(r#"
                    <div class="table-container" data-chunk-id="{0}">
                        <div class="table-header">
                            <h3 class="table-title">🐹 Table {0} (Raw)</h3>
                        </div>
                        <pre style="color: #00ff00; background: rgba(0,0,0,0.5); padding: 15px; border-radius: 6px; overflow-x: auto;">{1}</pre>
                    </div>
                "#, chunk_index, self.html_escape(&chunk.content))
            }
        }
            "heading" => {
                format!(r#"
                    <div class="content-container content-heading" data-chunk-id="{}">
                        <h2>🏷️ {}</h2>
                    </div>
                "#, chunk_id, self.html_escape(&chunk.content))
            },
            "text" => {
                let formatted_text = self.format_text_content(&chunk.content);
                format!(r#"
                    <div class="content-container content-text" data-chunk-id="{}">
                        📄 {}
                    </div>
                "#, chunk_id, formatted_text)
            },
            "caption" => {
                format!(r#"
                    <div class="content-container content-caption" data-chunk-id="{}">
                        💬 {}
                    </div>
                "#, chunk_id, self.html_escape(&chunk.content))
            },
            "list" => {
                let formatted_list = self.format_list_content(&chunk.content);
                format!(r#"
                    <div class="content-container content-list" data-chunk-id="{}">
                        📋 {}
                    </div>
                "#, chunk_id, formatted_list)
            },
            _ => {
                format!(r#"
                    <div class="content-container" data-chunk-id="{}">
                        <strong>🐭 {} Content</strong>: {}
                    </div>
                "#, chunk_id, chunk.content_type, self.html_escape(&chunk.content))
            }
        }
    }

    /// Render TableData structure as HTML
    #[allow(dead_code)]
    fn render_table_data(&self, table_data: &TableData, table_index: usize) -> String {
        self.render_enhanced_table(table_data, table_index)
    }

    /// Enhanced table rendering with complex structure support
    fn render_enhanced_table(&self, table_data: &TableData, table_index: usize) -> String {
        eprintln!("🐹 DEBUG: Rendering enhanced table {} with {} rows, {} cols", table_index, table_data.num_rows, table_data.num_cols);
        eprintln!("🐹 DEBUG: Table data: {:?}", table_data.data.len());
        
        let mut html = format!(r#"
            <div class="table-container" data-table-index="{}">
                <div class="table-header">
                    <h3 class="table-title">📊 Table {} (Enhanced)</h3>
                    <div class="table-info">{}×{} cells</div>
                </div>
                <div class="table-wrapper" style="overflow-x: auto; max-height: 500px; overflow-y: auto;">
                    <table class="chonker-table enhanced-table" contenteditable="false">
        "#, table_index, table_index, table_data.num_rows, table_data.num_cols);

        // Render table data with enhanced structure
        for (row_idx, row) in table_data.data.iter().enumerate() {
            if row_idx == 0 && !row.is_empty() {
                // Header row with editing support
                html.push_str("                        <thead>\n                            <tr>\n");
                for (col_idx, cell) in row.iter().enumerate() {
                    html.push_str(&format!(
                        "                                <th contenteditable=\"true\" data-row=\"{}\" data-col=\"{}\">{}</th>\n",
                        row_idx, col_idx, self.html_escape(&cell.content)
                    ));
                }
                html.push_str("                            </tr>\n                        </thead>\n");
                html.push_str("                        <tbody>\n");
            } else {
                // Data rows with editing support
                html.push_str("                            <tr>\n");
                for (col_idx, cell) in row.iter().enumerate() {
                    let cell_class = if self.is_numeric_content(&cell.content) { "numeric" } else { "text" };
                    let display_text = if cell.content.trim().is_empty() { 
                        "<span class=\"empty-cell\">—</span>".to_string()
                    } else { 
                        self.html_escape(&cell.content) 
                    };
                    
                    // Add colspan/rowspan support
                    let mut cell_attrs = format!("class=\"{}\" contenteditable=\"true\" data-row=\"{}\" data-col=\"{}\"",
                        cell_class, row_idx, col_idx);
                    
                    if let Some(colspan) = cell.colspan {
                        if colspan > 1 {
                            cell_attrs.push_str(&format!(" colspan=\"{}\"", colspan));
                        }
                    }
                    
                    if let Some(rowspan) = cell.rowspan {
                        if rowspan > 1 {
                            cell_attrs.push_str(&format!(" rowspan=\"{}\"", rowspan));
                        }
                    }
                    
                    html.push_str(&format!(
                        "                                <td {}>{}</td>\n",
                        cell_attrs, display_text
                    ));
                }
                html.push_str("                            </tr>\n");
            }
        }
        
        html.push_str("                        </tbody>\n");
        html.push_str("                    </table>\n");
        html.push_str("                </div>\n");
        html.push_str("            </div>\n");

        html
    }
}

impl Default for HtmlRenderer {
    fn default() -> Self {
        Self::new()
    }
}
