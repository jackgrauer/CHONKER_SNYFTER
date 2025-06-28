use crate::database::{DocumentChunk, ProcessingOptions};
use crate::error::{ChonkerError, ChonkerResult};
use crate::complexity::{ComplexityScorer, ExtractionPath};
use std::path::Path;
use std::collections::HashMap;
use std::time::Instant;
use tracing::{info, debug, warn};
use serde::{Serialize, Deserialize};
use anyhow::Result;

// Complexity threshold for routing decisions
const COMPLEXITY_THRESHOLD: f64 = 0.6;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentComplexity {
    pub file_size_score: f64,
    pub page_count_score: f64,
    pub image_ratio_score: f64,
    pub font_variety_score: f64,
    pub layout_score: f64,
    pub total_score: f64,
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

/// Hybrid PDF processor with fast Rust path and complex Python ML path
pub struct ChonkerProcessor {
    // Fast path components
    pub rust_extractor: Option<RustExtractor>,
    
    // Complex path bridge
    pub python_bridge: Option<PythonBridge>,
    
    // Complexity analysis
    pub complexity_scorer: ComplexityScorer,
    
    // Caching
    pub processing_cache: HashMap<String, ProcessingResult>,
    
    // Configuration
    pub enable_caching: bool,
    pub complexity_threshold: f64,
}

/// Rust native extractor for fast path
pub struct RustExtractor {
    // Will hold pdfium components
}

/// Python bridge for complex processing
pub struct PythonBridge {
    pub script_path: std::path::PathBuf,
}

impl Default for ChonkerProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl ChonkerProcessor {
    pub fn new() -> Self {
        Self {
            rust_extractor: Some(RustExtractor::new()),
            python_bridge: Some(PythonBridge::new()),
            complexity_scorer: ComplexityScorer::new(),
            processing_cache: HashMap::new(),
            enable_caching: true,
            complexity_threshold: COMPLEXITY_THRESHOLD,
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
        
        // Analyze document complexity using the proper analyzer
        let complexity_analysis = self.complexity_scorer.analyze_metadata(file_path)
            .map_err(|e| ChonkerError::SystemResource {
                resource: "complexity_analysis".to_string(),
                source: Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())),
            })?;
        
        info!("📊 {}", self.complexity_scorer.describe_analysis(&complexity_analysis));
        
        // Route to appropriate processing path based on analysis
        let mut result = match complexity_analysis.recommended_path {
            ExtractionPath::Native => {
                // Fast path - Rust native
                info!("🚀 Using fast path (Rust native)");
                self.process_fast_path(file_path, options).await?
            },
            ExtractionPath::Python => {
                // Complex path decision
                if options.tool == "fast" {
                    // Progressive: Fast path + background ML enhancement
                    info!("⚡ Using progressive path (Fast + ML queued)");
                    let fast_result = self.process_fast_path(file_path, options).await?;
                    self.queue_ml_enhancement(file_path, options).await;
                    
                    ProcessingResult {
                        chunks: fast_result.chunks,
                        metadata: ProcessingMetadata {
                            total_pages: fast_result.metadata.total_pages,
                            processing_time_ms: start_time.elapsed().as_millis() as u64,
                            tool_used: "rust_progressive".to_string(),
                            complexity_score: complexity_analysis.score,
                        },
                        processing_path: ProcessingPath::Progressive,
                    }
                } else {
                    // Direct complex path
                    info!("🧠 Using complex path (Python ML)");
                    self.process_complex_path(file_path, options).await?
                }
            }
        };
        
        // Update metadata with complexity analysis
        result.metadata.complexity_score = complexity_analysis.score;
        
        // Cache the result
        if self.enable_caching {
            self.processing_cache.insert(cache_key, result.clone());
        }
        
        let total_time = start_time.elapsed().as_millis() as u64;
        info!("✅ Processing completed in {}ms", total_time);
        
        Ok(result)
    }
    
    /// Analyze document complexity for routing decisions
    pub async fn analyze_complexity(&self, file_path: &Path) -> ChonkerResult<DocumentComplexity> {
        debug!("🔍 Analyzing document complexity: {:?}", file_path);
        
        // Get file metadata
        let metadata = std::fs::metadata(file_path)
            .map_err(|e| ChonkerError::file_io(file_path.to_string_lossy().to_string(), e))?;
        
        let file_size = metadata.len();
        
        // Basic complexity scoring
        let file_size_score = match file_size {
            0..=1_000_000 => 0.1,      // < 1MB
            1_000_001..=5_000_000 => 0.3,  // 1-5MB
            5_000_001..=20_000_000 => 0.6, // 5-20MB
            _ => 0.9,                       // > 20MB
        };
        
        // TODO: Implement actual PDF analysis for:
        // - Page count
        // - Image ratio
        // - Font variety
        // - Layout complexity
        
        let page_count_score = 0.3; // Placeholder
        let image_ratio_score = 0.2; // Placeholder
        let font_variety_score = 0.1; // Placeholder
        let layout_score = 0.2; // Placeholder
        
        let total_score = (file_size_score + page_count_score + image_ratio_score + 
                          font_variety_score + layout_score) / 5.0;
        
        Ok(DocumentComplexity {
            file_size_score,
            page_count_score,
            image_ratio_score,
            font_variety_score,
            layout_score,
            total_score,
        })
    }
    
    /// Fast path processing using Rust native tools
    async fn process_fast_path(
        &self,
        file_path: &Path,
        _options: &ProcessingOptions,
    ) -> ChonkerResult<ProcessingResult> {
        let start_time = Instant::now();
        
        if let Some(extractor) = &self.rust_extractor {
            let chunks = extractor.extract_text(file_path).await?;
            
            Ok(ProcessingResult {
                chunks,
                metadata: ProcessingMetadata {
                    total_pages: 1, // TODO: Get actual page count
                    processing_time_ms: start_time.elapsed().as_millis() as u64,
                    tool_used: "rust_native".to_string(),
                    complexity_score: 0.0, // Will be filled by caller
                },
                processing_path: ProcessingPath::FastPath,
            })
        } else {
            Err(ChonkerError::SystemResource {
                resource: "rust_extractor".to_string(),
                source: Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Rust extractor not initialized"
                )),
            })
        }
    }
    
    /// Complex path processing using Python ML tools
    async fn process_complex_path(
        &self,
        file_path: &Path,
        options: &ProcessingOptions,
    ) -> ChonkerResult<ProcessingResult> {
        let start_time = Instant::now();
        
        if let Some(bridge) = &self.python_bridge {
            let chunks = bridge.extract_advanced(file_path, options).await?;
            
            Ok(ProcessingResult {
                chunks,
                metadata: ProcessingMetadata {
                    total_pages: 1, // TODO: Get actual page count
                    processing_time_ms: start_time.elapsed().as_millis() as u64,
                    tool_used: "python_ml".to_string(),
                    complexity_score: 0.0, // Will be filled by caller
                },
                processing_path: ProcessingPath::ComplexPath,
            })
        } else {
            warn!("Python bridge not available, falling back to fast path");
            self.process_fast_path(file_path, options).await
        }
    }
    
    /// Queue ML enhancement for background processing
    async fn queue_ml_enhancement(&self, file_path: &Path, _options: &ProcessingOptions) {
        info!("📋 Queuing ML enhancement for: {:?}", file_path);
        // TODO: Implement actual background queue
        // This would typically use a job queue like Redis or in-memory queue
    }
    
    /// Generate cache key for result caching
    fn generate_cache_key(&self, file_path: &Path, options: &ProcessingOptions) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
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
    
    /// Get cache statistics
    pub fn get_cache_stats(&self) -> (usize, usize) {
        let entries = self.processing_cache.len();
        let memory_estimate = self.processing_cache.iter()
            .map(|(k, _)| k.len() + 1000) // Rough estimate per result
            .sum::<usize>();
        
        (entries, memory_estimate)
    }

    pub fn extract_text_with_ocr(&self, _page_data: &[u8]) -> Result<String, Box<dyn std::error::Error>> {
        // TODO: Implement OCR with MLX acceleration
        Ok("OCR extracted text".to_string())
    }

    pub fn detect_formulas(&self, _text: &str) -> Vec<String> {
        // TODO: Implement formula detection
        vec!["E=mc²".to_string()]
    }

    pub fn detect_tables(&self, _page_data: &[u8]) -> Vec<String> {
        // TODO: Implement table detection
        vec!["Table structure detected".to_string()]
    }

    pub fn apply_spatial_chunking(&self, _elements: &[String]) -> Vec<DocumentChunk> {
        // TODO: Implement spatial chunking algorithm
        vec![]
    }
}

impl RustExtractor {
    pub fn new() -> Self {
        Self {}
    }
    
    /// Extract text using Rust native tools (pdfium-render)
    pub async fn extract_text(&self, file_path: &Path) -> ChonkerResult<Vec<DocumentChunk>> {
        debug!("🔍 Rust native extraction: {:?}", file_path);
        
        // TODO: Implement pdfium-render integration
        // For now, return mock data
        let chunks = vec![
            DocumentChunk {
                id: 1,
                content: format!("🐹 CHONKER native processed: {:?}", file_path.file_name()),
                page_range: "1".to_string(),
                element_types: vec!["paragraph".to_string()],
                spatial_bounds: Some("{\"x\": 0, \"y\": 0, \"width\": 100, \"height\": 20}".to_string()),
                char_count: 50,
            },
        ];
        
        Ok(chunks)
    }
    
    /// Extract tables using heuristics
    pub fn extract_tables(&self, _page_data: &[u8]) -> ChonkerResult<Vec<String>> {
        // TODO: Implement table detection heuristics
        Ok(vec!["Table detected".to_string()])
    }
    
    /// Basic layout detection
    pub fn detect_layout(&self, _page_data: &[u8]) -> ChonkerResult<Vec<String>> {
        // TODO: Implement layout detection
        Ok(vec!["Single column".to_string()])
    }
}

impl PythonBridge {
    pub fn new() -> Self {
        Self {
            script_path: std::path::PathBuf::from("python/extraction_bridge.py"),
        }
    }
    
    /// Advanced extraction using Python ML tools
    pub async fn extract_advanced(
        &self,
        file_path: &Path,
        _options: &ProcessingOptions,
    ) -> ChonkerResult<Vec<DocumentChunk>> {
        debug!("🧠 Python ML extraction: {:?}", file_path);
        
        // TODO: Implement actual Python bridge using PyO3
        // For now, return mock data that simulates ML extraction
        let chunks = vec![
            DocumentChunk {
                id: 1,
                content: format!("🧠 ML enhanced extraction: {:?}", file_path.file_name()),
                page_range: "1".to_string(),
                element_types: vec!["heading".to_string(), "paragraph".to_string(), "table".to_string()],
                spatial_bounds: Some("{\"x\": 0, \"y\": 0, \"width\": 200, \"height\": 300}".to_string()),
                char_count: 150,
            },
            DocumentChunk {
                id: 2,
                content: "Advanced table structure detected with ML".to_string(),
                page_range: "1".to_string(),
                element_types: vec!["table".to_string()],
                spatial_bounds: Some("{\"x\": 0, \"y\": 320, \"width\": 200, \"height\": 100}".to_string()),
                char_count: 45,
            },
        ];
        
        Ok(chunks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs::File;
    use std::io::Write;
    
    #[tokio::test]
    async fn test_processor_creation() {
        let processor = ChonkerProcessor::new();
        assert!(processor.rust_extractor.is_some());
        assert!(processor.python_bridge.is_some());
    }
    
    #[tokio::test]
    async fn test_complexity_analysis() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.pdf");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "test content").unwrap();
        
        let processor = ChonkerProcessor::new();
        let complexity = processor.analyze_complexity(&file_path).await.unwrap();
        
        assert!(complexity.total_score >= 0.0);
        assert!(complexity.total_score <= 1.0);
    }
    
    #[tokio::test]
    async fn test_processing_paths() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.pdf");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "test content").unwrap();
        
        let mut processor = ChonkerProcessor::new();
        let options = ProcessingOptions {
            tool: "auto".to_string(),
            extract_tables: true,
            extract_formulas: false,
        };
        
        let result = processor.process_document(&file_path, &options).await.unwrap();
        assert!(!result.chunks.is_empty());
        assert!(result.metadata.processing_time_ms > 0);
    }
}
