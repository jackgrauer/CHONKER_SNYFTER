use crate::chonker_types::{DocumentChunk, TableData, TableCell};
use crate::error::{ChonkerError, ChonkerResult};
use crate::extractor::{Extractor, ExtractionResult};
use std::path::Path;
use std::collections::HashMap;
use std::time::Instant;
use tracing::{info};
use serde::{Serialize, Deserialize};
use std::hash::{Hash, Hasher};
use uuid::Uuid;
use chrono::Utc;

/// Structure to hold mixed content (text, tables, etc.)
#[derive(Debug, Clone)]
struct ContentChunk {
    content: String,
    content_type: String,
    metadata: String,
    table_data: Option<TableData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingOptions {
    pub tool: String,
    pub extract_tables: bool,
    pub extract_formulas: bool,
}

#[derive(Debug, Clone)]
pub struct ProcessingResult {
    pub chunks: Vec<DocumentChunk>,
    pub metadata: ProcessingMetadata,
    pub _processing_path: ProcessingPath,
}

#[derive(Debug, Clone)]
pub struct ProcessingMetadata {
    pub total_pages: usize,
    pub processing_time_ms: u64,
    pub tool_used: String,
    pub _complexity_score: f64,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum ProcessingPath {
    FastPath,     // Rust native processing
    ComplexPath,  // Python ML processing
    Progressive,  // Fast path with ML enhancement queued
}

/// Simplified PDF processor for Tauri integration
pub struct ChonkerProcessor {
    // Python extraction bridge
    pub extractor: Extractor,
    
    // Caching
    pub processing_cache: HashMap<String, ProcessingResult>,
    
    // Configuration
    pub enable_caching: bool,
}

impl Default for ChonkerProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl ChonkerProcessor {
    pub fn new() -> Self {
        Self {
            extractor: Extractor::new(),
            processing_cache: HashMap::new(),
            enable_caching: true,
        }
    }
    
    /// Main entry point for document processing
    pub async fn process_document(
        &mut self,
        file_path: &Path,
        options: &ProcessingOptions,
    ) -> ChonkerResult<ProcessingResult> {
        let start_time = Instant::now();
        
        info!("🐹 Processing document: {:?}", file_path);
        
        // Check cache first
        let cache_key = self.generate_cache_key(file_path, options);
        if self.enable_caching {
            if let Some(cached_result) = self.processing_cache.get(&cache_key) {
                info!("📋 Using cached result for: {:?}", file_path);
                return Ok(cached_result.clone());
            }
        }
        
        // Use Python extraction bridge for processing
        match self.extractor.extract_pdf(&file_path.to_path_buf()).await {
            Ok(extraction_result) => {
                let chunks = self.convert_extraction_to_chunks(extraction_result.clone());
                
                let result = ProcessingResult {
                    chunks,
                    metadata: ProcessingMetadata {
                        total_pages: extraction_result.metadata.total_pages,
                        processing_time_ms: start_time.elapsed().as_millis() as u64,
                        tool_used: extraction_result.tool,
                        _complexity_score: 0.5, // Default complexity
                    },
                    _processing_path: ProcessingPath::ComplexPath,
                };
                
                // Cache the result
                if self.enable_caching {
                    self.processing_cache.insert(cache_key, result.clone());
                }
                
                Ok(result)
            }
            Err(e) => {
                Err(ChonkerError::PdfProcessing {
                    message: format!("Extraction failed: {}", e),
                    source: None,
                })
            }
        }
    }
    
    /// Convert extraction result to document chunks
    fn convert_extraction_to_chunks(&self, extraction: ExtractionResult) -> Vec<DocumentChunk> {
        let mut chunks = Vec::new();
        let mut chunk_index = 0;
        let document_id = Uuid::new_v4();
        let now = Utc::now();
        
        for page_extraction in extraction.extractions {
            // Process ALL content from each page - both tables and text
            if !page_extraction.text.trim().is_empty() {
                // Parse the entire page content into mixed content chunks
                let content_chunks = self.parse_mixed_content(&page_extraction.text, page_extraction.page_number);
                
                for content_chunk in content_chunks {
                    chunks.push(DocumentChunk {
                        id: Uuid::new_v4(),
                        document_id,
                        chunk_index,
                        content: content_chunk.content,
                        content_type: content_chunk.content_type,
                        metadata: Some(content_chunk.metadata),
                        table_data: content_chunk.table_data,
                        created_at: now,
                    });
                    chunk_index += 1;
                }
            }
            
            // Skip legacy table chunks since the extraction bridge leaves tables empty
            // All the good table data is already in the markdown text above
            
            // Create formula chunks
            for (formula_idx, formula) in page_extraction.formulas.iter().enumerate() {
                chunks.push(DocumentChunk {
                    id: Uuid::new_v4(),
                    document_id,
                    chunk_index,
                    content: serde_json::to_string_pretty(formula).unwrap_or_default(),
                    content_type: "formula".to_string(),
                    metadata: Some(format!("page_{}_formula_{}", page_extraction.page_number, formula_idx)),
                    table_data: None,
                    created_at: now,
                });
                chunk_index += 1;
            }
        }
        
        chunks
    }
    
    /// Parse mixed content from text - captures EVERYTHING (tables, text, headers, etc.)
    fn parse_mixed_content(&self, text: &str, page_number: usize) -> Vec<ContentChunk> {
        let mut content_chunks = Vec::new();
        let lines: Vec<&str> = text.lines().collect();
        let mut current_pos = 0;
        let mut content_index = 0;
        
        while current_pos < lines.len() {
            // Try to parse a table at this position
            if let Some((table_data, end_pos)) = self.try_parse_table_at_position(&lines, current_pos) {
                // Found a table - create a table chunk
                content_chunks.push(ContentChunk {
                    content: self.extract_table_content(&lines[current_pos..end_pos]),
                    content_type: "table".to_string(),
                    metadata: format!("page_{}_table_{}", page_number, content_chunks.len() + 1),
                    table_data: Some(table_data),
                });
                current_pos = end_pos;
                content_index += 1;
            } else {
                // Not a table - collect consecutive non-table lines as text content
                let text_start = current_pos;
                let mut text_lines = Vec::new();
                
                // Collect lines until we hit a table or end of content
                while current_pos < lines.len() {
                    // Check if this line starts a table
                    if lines[current_pos].trim().contains('|') && 
                       current_pos + 1 < lines.len() && 
                       (lines[current_pos + 1].trim().contains('|') || 
                        lines[current_pos + 1].chars().all(|c| c == '|' || c == '-' || c == ':' || c.is_whitespace())) {
                        // This looks like the start of a table, stop collecting text
                        break;
                    }
                    
                    text_lines.push(lines[current_pos]);
                    current_pos += 1;
                }
                
                // If we collected any text lines, create a text chunk
                if !text_lines.is_empty() {
                    let text_content = text_lines.join("\n").trim().to_string();
                    if !text_content.is_empty() {
                        // Determine content type based on content
                        let content_type = self.classify_text_content(&text_content);
                        
                        content_chunks.push(ContentChunk {
                            content: text_content,
                            content_type,
                            metadata: format!("page_{}_content_{}", page_number, content_index + 1),
                            table_data: None,
                        });
                        content_index += 1;
                    }
                }
                
                // If we didn't advance (edge case), force advance to avoid infinite loop
                if current_pos == text_start {
                    current_pos += 1;
                }
            }
        }
        
        content_chunks
    }
    
    /// Extract the raw table content for display
    fn extract_table_content(&self, table_lines: &[&str]) -> String {
        table_lines.join("\n")
    }
    
    /// Classify text content to determine its type
    fn classify_text_content(&self, text: &str) -> String {
        let trimmed = text.trim();
        
        // Check for headers (short lines, often capitalized)
        if trimmed.lines().count() == 1 && trimmed.len() < 100 && 
           trimmed.chars().filter(|c| c.is_uppercase()).count() > trimmed.len() / 3 {
            return "heading".to_string();
        }
        
        // Check for lists (lines starting with bullets, numbers, etc.)
        let list_markers = ["-", "*", "+", "1.", "2.", "3.", "•"];
        if trimmed.lines().any(|line| {
            let line_trimmed = line.trim();
            list_markers.iter().any(|marker| line_trimmed.starts_with(marker))
        }) {
            return "list".to_string();
        }
        
        // Check for very short content (might be captions, labels, etc.)
        if trimmed.len() < 50 {
            return "caption".to_string();
        }
        
        // Default to paragraph text
        "text".to_string()
    }
    
    /// Parse markdown tables from text content
    #[allow(dead_code)]
    fn parse_markdown_tables(&self, text: &str) -> Vec<TableData> {
        let mut tables = Vec::new();
        let lines: Vec<&str> = text.lines().collect();
        let mut i = 0;
        
        while i < lines.len() {
            // Look for table patterns
            if let Some(table_data) = self.try_parse_table_at_position(&lines, i) {
                tables.push(table_data.0);
                i = table_data.1; // Skip to end of table
            } else {
                i += 1;
            }
        }
        
        tables
    }
    
    /// Try to parse a table starting at the given line position
    fn try_parse_table_at_position(&self, lines: &[&str], start_pos: usize) -> Option<(TableData, usize)> {
        if start_pos >= lines.len() {
            return None;
        }
        
        let first_line = lines[start_pos].trim();
        
        // Check if this looks like a table header (contains | characters)
        if !first_line.contains('|') {
            return None;
        }
        
        let mut table_lines = Vec::new();
        let mut current_pos = start_pos;
        
        // Collect all consecutive lines that look like table rows
        while current_pos < lines.len() {
            let line = lines[current_pos].trim();
            
            // Skip empty lines
            if line.is_empty() {
                current_pos += 1;
                continue;
            }
            
            // If line contains pipes, it's likely a table row
            if line.contains('|') {
                table_lines.push(line);
                current_pos += 1;
            } else {
                // No more table content
                break;
            }
        }
        
        // Need at least 2 lines to have a table (header + separator or header + data)
        if table_lines.len() < 2 {
            return None;
        }
        
        // Parse the table lines into structured data
        let parsed_table = self.parse_table_lines(&table_lines);
        if parsed_table.data.is_empty() {
            return None;
        }
        
        Some((parsed_table, current_pos))
    }
    
    /// Parse table lines into TableData structure
    fn parse_table_lines(&self, lines: &[&str]) -> TableData {
        let mut rows = Vec::new();
        let mut max_cols = 0;
        
        for line in lines {
            // Skip markdown separator lines (like |---|---|---|
            if line.chars().all(|c| c == '|' || c == '-' || c == ':' || c.is_whitespace()) {
                continue;
            }
            
            // Split by | and clean up cells
            let cells: Vec<String> = line
                .split('|')
                .map(|cell| cell.trim().to_string())
                .filter(|cell| !cell.is_empty()) // Remove empty cells from start/end
                .collect();
            
            if !cells.is_empty() {
                max_cols = max_cols.max(cells.len());
                let table_cells: Vec<TableCell> = cells
                    .into_iter()
                    .map(|content| TableCell {
                        content,
                        rowspan: None,
                        colspan: None,
                    })
                    .collect();
                rows.push(table_cells);
            }
        }
        
        TableData {
            num_rows: rows.len(),
            num_cols: max_cols,
            data: rows,
        }
    }
    
    /// Convert raw table data to structured TableData
    #[allow(dead_code)]
    fn convert_table_to_table_data(&self, table: &serde_json::Value) -> TableData {
        // Try to parse the table data structure
        if let Some(rows_array) = table.as_array() {
            let mut data = Vec::new();
            let mut max_cols = 0;
            
            for row in rows_array {
                if let Some(row_array) = row.as_array() {
                    let mut row_cells = Vec::new();
                    for cell in row_array {
                        let cell_content = cell.as_str().unwrap_or("").to_string();
                        row_cells.push(TableCell {
                            content: cell_content,
                            rowspan: None,
                            colspan: None,
                        });
                    }
                    max_cols = max_cols.max(row_cells.len());
                    data.push(row_cells);
                }
            }
            
            TableData {
                num_rows: data.len(),
                num_cols: max_cols,
                data,
            }
        } else {
            // Fallback: create a single cell with the JSON content
            TableData {
                num_rows: 1,
                num_cols: 1,
                data: vec![vec![TableCell {
                    content: table.to_string(),
                    rowspan: None,
                    colspan: None,
                }]],
            }
        }
    }
    
    /// Generate cache key for result caching
    fn generate_cache_key(&self, file_path: &Path, options: &ProcessingOptions) -> String {
        use std::collections::hash_map::DefaultHasher;
        
        let mut hasher = DefaultHasher::new();
        file_path.hash(&mut hasher);
        options.tool.hash(&mut hasher);
        options.extract_tables.hash(&mut hasher);
        options.extract_formulas.hash(&mut hasher);
        
        format!("{:x}", hasher.finish())
    }
    
    /// Clear processing cache
    #[allow(dead_code)]
    pub fn clear_cache(&mut self) {
        self.processing_cache.clear();
        info!("🧹 Processing cache cleared");
    }
}
