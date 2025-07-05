use crate::database::{DocumentChunk, ProcessingOptions};
use crate::error::{ChonkerError, ChonkerResult};
// use crate::smart_chunker::SmartChunker;
#[cfg(feature = "advanced_pdf")]
use crate::analyzer::{ComplexityAnalyzer, ComplexityScore};
#[cfg(feature = "advanced_pdf")]
use crate::native_extractor::NativeExtractor;
// TODO: Integrate new FastPathProcessor and ComplexityAnalyzer
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
    #[cfg(feature = "advanced_pdf")]
    pub complexity_analyzer: Option<ComplexityAnalyzer>,
    #[cfg(not(feature = "advanced_pdf"))]
    pub complexity_analyzer: Option<()>,
    
    // Caching
    pub processing_cache: HashMap<String, ProcessingResult>,
    
    // Configuration
    pub enable_caching: bool,
    pub complexity_threshold: f64,
}

/// Rust native extractor for fast path
#[cfg(feature = "advanced_pdf")]
pub struct RustExtractor {
    native_extractor: NativeExtractor,
}

#[cfg(not(feature = "advanced_pdf"))]
pub struct RustExtractor {
    _placeholder: (),
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
        // Try to initialize native extractor, but don't fail if PDFium is unavailable
        let rust_extractor = match RustExtractor::new() {
            Ok(extractor) => {
                info!("✅ Native PDF extractor initialized");
                Some(extractor)
            }
            Err(e) => {
                warn!("⚠️  Native PDF extractor unavailable: {}", e);
                warn!("📋 Will use Python fallback for all documents");
                None
            }
        };
        
        Self {
            rust_extractor,
            python_bridge: Some(PythonBridge::new()),
            #[cfg(feature = "advanced_pdf")]
            complexity_analyzer: ComplexityAnalyzer::new().ok(),
            #[cfg(not(feature = "advanced_pdf"))]
            complexity_analyzer: None,
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
        let _start_time = Instant::now();
        
        info!("🐹 Processing document: {:?}", file_path);
        
        // Check cache first
        let cache_key = self.generate_cache_key(file_path, options);
        if self.enable_caching {
            if let Some(cached_result) = self.processing_cache.get(&cache_key) {
                info!("📋 Using cached result for: {:?}", file_path);
                return Ok(cached_result.clone());
            }
        }
        
        // Without advanced_pdf feature, always use complex path
        #[cfg(not(feature = "advanced_pdf"))]
        {
            warn!("⚠️  Advanced PDF features not available, using complex path");
            return self.process_complex_path(file_path, options).await;
        }
        
        // Analyze document complexity using the new analyzer
        #[cfg(feature = "advanced_pdf")]
        {
            let complexity_analysis = match &self.complexity_analyzer {
                Some(analyzer) => {
                    analyzer.analyze_simple(file_path)
                        .map_err(|e| ChonkerError::SystemResource {
                            resource: "complexity_analysis".to_string(),
                            source: Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())),
                        })?
                }
                None => {
                    // Fallback complexity analysis - create a simple score structure
                    ComplexityScore {
                        score: 5.0,
                        factors: crate::analyzer::complexity::ComplexityFactors {
                            page_count: 0,
                            has_images: false,
                            has_tables: false,
                            has_forms: false,
                            file_size_mb: 0.0,
                            has_multiple_columns: false,
                        },
                        reasoning: "Fallback analysis - analyzer unavailable".to_string(),
                        should_use_fast_path: false,
                    }
                }
            };
            
            info!("📊 Complexity analysis: {}", complexity_analysis.reasoning);
            
            // Route to appropriate processing path based on analysis
            let mut result = if complexity_analysis.should_use_fast_path {
            if self.rust_extractor.is_some() {
                // Fast path - Rust native
                info!("🚀 Using fast path (Rust native)");
                self.process_fast_path(file_path, options).await?
            } else {
                // Fallback to complex path if native extractor unavailable
                info!("🧠 Falling back to complex path (native unavailable)");
                self.process_complex_path(file_path, options).await?
            }
        } else {
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
                        complexity_score: complexity_analysis.score as f64,
                    },
                    processing_path: ProcessingPath::Progressive,
                }
            } else {
                // Direct complex path
                info!("🧠 Using complex path (Python ML)");
                self.process_complex_path(file_path, options).await?
            }
        };
        
            // Update metadata with complexity analysis
            result.metadata.complexity_score = complexity_analysis.score as f64;
            
            // Cache the result
            if self.enable_caching {
                self.processing_cache.insert(cache_key, result.clone());
            }
            
            let total_time = start_time.elapsed().as_millis() as u64;
            info!("✅ Processing completed in {}ms", total_time);
            
            Ok(result)
        }
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
        // OCR not yet implemented - return empty string instead of fake data
        Ok(String::new())
    }

    pub fn detect_formulas(&self, _text: &str) -> Vec<String> {
        // Formula detection not yet implemented - return empty vector instead of fake data
        vec![]
    }

    pub fn detect_tables(&self, _page_data: &[u8]) -> Vec<String> {
        // Table detection not yet implemented - return empty vector instead of fake data
        vec![]
    }

    pub fn apply_spatial_chunking(&self, _elements: &[String]) -> Vec<DocumentChunk> {
        // TODO: Implement spatial chunking algorithm
        vec![]
    }
}

#[cfg(feature = "advanced_pdf")]
impl RustExtractor {
    pub fn new() -> Result<Self, anyhow::Error> {
        let native_extractor = NativeExtractor::new()?;
        Ok(Self {
            native_extractor,
        })
    }
    
    /// Extract text using Rust native tools (pdfium-render)
    pub async fn extract_text(&self, file_path: &Path) -> ChonkerResult<Vec<DocumentChunk>> {
        debug!("🔍 Rust native extraction: {:?}", file_path);
        
        // Use the actual native extractor
        let result = self.native_extractor.extract_pdf(file_path)
            .map_err(|e| ChonkerError::SystemResource {
                resource: "native_extractor".to_string(),
                source: Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())),
            })?;
        
        // Convert NativeExtractor results to DocumentChunk format
        let mut chunks = Vec::new();
        for (idx, page_extraction) in result.extractions.iter().enumerate() {
            if !page_extraction.text.trim().is_empty() {
                chunks.push(DocumentChunk {
                    id: (idx + 1) as i64,
                    content: page_extraction.text.clone(),
                    page_range: page_extraction.page_number.to_string(),
                    element_types: vec!["text".to_string()],
                    spatial_bounds: None, // Native extractor doesn't provide detailed bounds
                    char_count: page_extraction.character_count as i64,
                    table_data: None,
                });
            }
        }
        
        info!("✅ Native extraction completed: {} pages, {} chunks", 
              result.metadata.total_pages, chunks.len());
        
        Ok(chunks)
    }
    
    /// Extract tables using heuristics
    pub fn extract_tables(&self, _page_data: &[u8]) -> ChonkerResult<Vec<String>> {
        // Table detection heuristics not yet implemented - return empty vector instead of fake data
        Ok(vec![])
    }
    
    /// Basic layout detection
    pub fn detect_layout(&self, _page_data: &[u8]) -> ChonkerResult<Vec<String>> {
        // Layout detection not yet implemented - return empty vector instead of fake data
        Ok(vec![])
    }
}

#[cfg(not(feature = "advanced_pdf"))]
impl RustExtractor {
    pub fn new() -> Result<Self, anyhow::Error> {
        Err(anyhow::anyhow!("Advanced PDF features not available"))
    }
    
    pub async fn extract_text(&self, _file_path: &Path) -> ChonkerResult<Vec<DocumentChunk>> {
        Err(ChonkerError::SystemResource {
            resource: "native_extractor".to_string(),
            source: Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Advanced PDF features not available"
            )),
        })
    }

    pub fn extract_tables(&self, _page_data: &[u8]) -> ChonkerResult<Vec<String>> {
        Err(ChonkerError::SystemResource {
            resource: "native_extractor".to_string(),
            source: Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Advanced PDF features not available"
            )),
        })
    }
    
    pub fn detect_layout(&self, _page_data: &[u8]) -> ChonkerResult<Vec<String>> {
        Err(ChonkerError::SystemResource {
            resource: "native_extractor".to_string(),
            source: Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Advanced PDF features not available"
            )),
        })
    }
}

impl PythonBridge {
    pub fn new() -> Self {
        // Use the unified environmental lab extraction bridge
        let script_path = if let Ok(current_dir) = std::env::current_dir() {
            let extraction_bridge = current_dir.join("python/extraction_bridge.py");
            if extraction_bridge.exists() {
                info!("🧪 Using Docling v2 environmental lab extraction bridge");
                extraction_bridge
            } else if let Ok(exe_path) = std::env::current_exe() {
                let exe_dir = exe_path.parent().unwrap_or_else(|| std::path::Path::new("."));
                let script_from_exe = exe_dir.join("../../../python/extraction_bridge.py");
                if script_from_exe.exists() {
                    info!("🧪 Using extraction bridge from exe path");
                    script_from_exe
                } else {
                    // Fallback to relative path
                    std::path::PathBuf::from("python/extraction_bridge.py")
                }
            } else {
                std::path::PathBuf::from("python/extraction_bridge.py")
            }
        } else {
            std::path::PathBuf::from("python/extraction_bridge.py")
        };
        
        Self {
            script_path,
        }
    }
    
    /// Advanced extraction using Python ML tools with smart chunking
    pub async fn extract_advanced(
        &self,
        file_path: &Path,
        _options: &ProcessingOptions,
    ) -> ChonkerResult<Vec<DocumentChunk>> {
        debug!("🧠 Python ML extraction with smart chunking: {:?}", file_path);
        
        // Use the actual Python extraction script
        let mut extractor = crate::extractor::Extractor::new();
        
        let extraction_result = extractor.extract_pdf(&file_path.to_path_buf()).await
            .map_err(|e| ChonkerError::PdfProcessing {
                message: format!("Python extraction failed: {}", e),
                source: None,
            })?;
        
        if !extraction_result.success {
            return Err(ChonkerError::PdfProcessing {
                message: extraction_result.error.unwrap_or_else(|| "Unknown error".to_string()),
                source: None,
            });
        }
        
        // TODO: Implement smart chunking here once the module is properly integrated
        info!("📋 Using structured table chunker (smart chunking temporarily disabled)");
        
        // Fallback: Legacy chunking for non-Docling results or if smart chunking fails
        info!("📋 Using legacy character-based chunking");
        
        // Convert Python extraction results to DocumentChunk format (legacy method)
        let mut chunks = Vec::new();
        for (idx, page_extraction) in extraction_result.extractions.iter().enumerate() {
            // Only create chunks if there's actual content
            if !page_extraction.text.trim().is_empty() {
                chunks.push(DocumentChunk {
                    id: (idx + 1) as i64,
                    content: page_extraction.text.clone(),
                    page_range: page_extraction.page_number.to_string(),
                    element_types: self.detect_element_types(&page_extraction.text),
                    spatial_bounds: if !page_extraction.layout_boxes.is_empty() {
                        Some(serde_json::to_string(&page_extraction.layout_boxes).unwrap_or_default())
                    } else {
                        None
                    },
                    char_count: page_extraction.text.chars().count() as i64,
                    table_data: None,
                });
            }
        }
        
        // If no text content was found, don't create fake chunks
        if chunks.is_empty() {
            info!("📄 No extractable text content found in PDF");
        } else {
            info!("✅ Legacy extraction completed: {} chunks from {} pages", 
                  chunks.len(), extraction_result.metadata.total_pages);
        }
        
        Ok(chunks)
    }
    
    /// Detect element types from text content
    fn detect_element_types(&self, text: &str) -> Vec<String> {
        let mut types = Vec::new();
        
        if text.lines().any(|line| line.trim().starts_with('#')) {
            types.push("heading".to_string());
        }
        if text.contains('|') && text.lines().any(|line| line.matches('|').count() > 1) {
            types.push("table".to_string());
        }
        if !text.trim().is_empty() {
            types.push("text".to_string());
        }
        
        if types.is_empty() {
            types.push("unknown".to_string());
        }
        
        types
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
