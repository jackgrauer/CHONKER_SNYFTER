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
    pub processing_path: ProcessingPath,
}

#[derive(Debug, Clone)]
pub struct ProcessingMetadata {
    pub total_pages: usize,
    pub processing_time_ms: u64,
    pub tool_used: String,
    pub complexity_score: f64,
}

#[derive(Debug, Clone, PartialEq)]
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
                        complexity_score: 0.5, // Default complexity
                    },
                    processing_path: ProcessingPath::ComplexPath,
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
            // Create text chunk if there's content
            if !page_extraction.text.trim().is_empty() {
                chunks.push(DocumentChunk {
                    id: Uuid::new_v4(),
                    document_id,
                    chunk_index,
                    content: page_extraction.text.clone(),
                    content_type: "text".to_string(),
                    metadata: Some(format!("page_{}", page_extraction.page_number)),
                    table_data: None,
                    created_at: now,
                });
                chunk_index += 1;
            }
            
            // Create table chunks
            for (table_idx, table) in page_extraction.tables.iter().enumerate() {
                let table_data = self.convert_table_to_table_data(table);
                chunks.push(DocumentChunk {
                    id: Uuid::new_v4(),
                    document_id,
                    chunk_index,
                    content: serde_json::to_string_pretty(table).unwrap_or_default(),
                    content_type: "table".to_string(),
                    metadata: Some(format!("page_{}_table_{}", page_extraction.page_number, table_idx)),
                    table_data: Some(table_data),
                    created_at: now,
                });
                chunk_index += 1;
            }
            
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
    
    /// Convert raw table data to structured TableData
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
    pub fn clear_cache(&mut self) {
        self.processing_cache.clear();
        info!("🧹 Processing cache cleared");
    }
}
