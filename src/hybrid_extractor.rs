use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use blake3;

use crate::complexity::{ComplexityScorer, ComplexityAnalysis, ExtractionPath};
use crate::native_extractor::{NativeExtractor, NativeExtractionResult};
use crate::extractor::{Extractor, ExtractionResult};

/// Hybrid PDF processor that routes between fast native and advanced Python extraction
pub struct HybridExtractor {
    native_extractor: NativeExtractor,
    python_extractor: Extractor,
    complexity_scorer: ComplexityScorer,
    cache: ExtractionCache,
    config: HybridConfig,
}

#[derive(Debug, Clone)]
pub struct HybridConfig {
    pub enable_caching: bool,
    pub force_native: bool,           // Force native extraction for testing
    pub force_python: bool,           // Force Python extraction for testing
    pub enable_progressive: bool,     // Show native results immediately, enhance with Python
    pub quality_threshold: f64,       // If native quality < threshold, try Python
    pub performance_threshold_ms: u64, // If native takes > threshold, prefer Python for similar docs
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridExtractionResult {
    pub primary_result: ExtractionResult,
    pub enhancement_result: Option<ExtractionResult>,
    pub complexity_analysis: ComplexityAnalysis,
    pub extraction_path_used: ExtractionPath,
    pub performance_metrics: PerformanceMetrics,
    pub cache_hit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub native_time_ms: Option<u64>,
    pub python_time_ms: Option<u64>,
    pub total_time_ms: u64,
    pub cache_lookup_ms: u64,
    pub complexity_analysis_ms: u64,
}

/// Simple in-memory cache for extraction results
#[derive(Debug)]
struct ExtractionCache {
    native_cache: HashMap<String, (NativeExtractionResult, std::time::Instant)>,
    python_cache: HashMap<String, (ExtractionResult, std::time::Instant)>,
    ttl_seconds: u64,
}

impl Default for HybridConfig {
    fn default() -> Self {
        Self {
            enable_caching: true,
            force_native: false,
            force_python: false,
            enable_progressive: true,
            quality_threshold: 0.6,
            performance_threshold_ms: 100,
        }
    }
}

impl ExtractionCache {
    fn new() -> Self {
        Self {
            native_cache: HashMap::new(),
            python_cache: HashMap::new(),
            ttl_seconds: 3600, // 1 hour TTL
        }
    }

    fn generate_cache_key(pdf_bytes: &[u8], page_num: Option<usize>) -> String {
        let hash = blake3::hash(pdf_bytes);
        match page_num {
            Some(page) => format!("{}:page:{}", hash.to_hex(), page),
            None => format!("{}:full", hash.to_hex()),
        }
    }

    fn is_expired(&self, timestamp: std::time::Instant) -> bool {
        timestamp.elapsed().as_secs() > self.ttl_seconds
    }

    fn get_native(&mut self, key: &str) -> Option<NativeExtractionResult> {
        if let Some((result, timestamp)) = self.native_cache.get(key) {
            if !self.is_expired(*timestamp) {
                return Some(result.clone());
            } else {
                self.native_cache.remove(key);
            }
        }
        None
    }

    fn get_python(&mut self, key: &str) -> Option<ExtractionResult> {
        if let Some((result, timestamp)) = self.python_cache.get(key) {
            if !self.is_expired(*timestamp) {
                return Some(result.clone());
            } else {
                self.python_cache.remove(key);
            }
        }
        None
    }

    fn put_native(&mut self, key: String, result: NativeExtractionResult) {
        self.native_cache.insert(key, (result, std::time::Instant::now()));
    }

    fn put_python(&mut self, key: String, result: ExtractionResult) {
        self.python_cache.insert(key, (result, std::time::Instant::now()));
    }

    fn clear_expired(&mut self) {
        let ttl_seconds = self.ttl_seconds;
        self.native_cache.retain(|_, (_, timestamp)| {
            timestamp.elapsed().as_secs() <= ttl_seconds
        });
        self.python_cache.retain(|_, (_, timestamp)| {
            timestamp.elapsed().as_secs() <= ttl_seconds
        });
    }

    fn stats(&self) -> (usize, usize) {
        (self.native_cache.len(), self.python_cache.len())
    }
}

impl HybridExtractor {
    pub fn new() -> Result<Self> {
        Ok(Self {
            native_extractor: NativeExtractor::new()?,
            python_extractor: Extractor::new(),
            complexity_scorer: ComplexityScorer::new(),
            cache: ExtractionCache::new(),
            config: HybridConfig::default(),
        })
    }

    pub fn with_config(config: HybridConfig) -> Result<Self> {
        Ok(Self {
            native_extractor: NativeExtractor::new()?,
            python_extractor: Extractor::new(),
            complexity_scorer: ComplexityScorer::new(),
            cache: ExtractionCache::new(),
            config,
        })
    }

    /// Extract PDF with smart routing between native and Python extractors
    pub async fn extract_pdf<P: AsRef<Path>>(&mut self, pdf_path: P) -> Result<HybridExtractionResult> {
        let start_time = std::time::Instant::now();
        let path = pdf_path.as_ref();

        // Read PDF for analysis and caching
        let pdf_bytes = tokio::fs::read(path).await
            .map_err(|e| anyhow!("Failed to read PDF file: {}", e))?;

        self.extract_from_bytes(&pdf_bytes, None).await
    }

    /// Extract specific page with smart routing
    pub async fn extract_page<P: AsRef<Path>>(&mut self, pdf_path: P, page_num: usize) -> Result<HybridExtractionResult> {
        let path = pdf_path.as_ref();
        let pdf_bytes = tokio::fs::read(path).await
            .map_err(|e| anyhow!("Failed to read PDF file: {}", e))?;

        self.extract_from_bytes(&pdf_bytes, Some(page_num)).await
    }

    /// Extract from PDF bytes (main extraction logic)
    pub async fn extract_from_bytes(&mut self, pdf_bytes: &[u8], page_num: Option<usize>) -> Result<HybridExtractionResult> {
        let start_time = std::time::Instant::now();
        
        // Generate cache key
        let cache_key = ExtractionCache::generate_cache_key(pdf_bytes, page_num);
        let cache_start = std::time::Instant::now();
        
        // Check cache first
        if self.config.enable_caching {
            self.cache.clear_expired();
            
            // Try native cache first (faster results)
            if let Some(native_result) = self.cache.get_native(&cache_key) {
                let legacy_result = self.native_extractor.to_legacy_format(&native_result);
                return Ok(HybridExtractionResult {
                    primary_result: legacy_result.clone(),
                    enhancement_result: self.cache.get_python(&format!("{}_enhanced", cache_key)),
                    complexity_analysis: ComplexityAnalysis {
                        score: 0.3, // Assume cached results were simple
                        recommended_path: ExtractionPath::Native,
                        file_size_mb: pdf_bytes.len() as f64 / (1024.0 * 1024.0),
                        estimated_page_count: native_result.metadata.total_pages,
                        text_to_image_ratio: None,
                        has_forms: false,
                        has_complex_layout: false,
                        confidence: 0.9,
                    },
                    extraction_path_used: ExtractionPath::Native,
                    performance_metrics: PerformanceMetrics {
                        native_time_ms: Some(native_result.performance_ms),
                        python_time_ms: None,
                        total_time_ms: start_time.elapsed().as_millis() as u64,
                        cache_lookup_ms: cache_start.elapsed().as_millis() as u64,
                        complexity_analysis_ms: 0,
                    },
                    cache_hit: true,
                });
            }
        }

        let cache_lookup_ms = cache_start.elapsed().as_millis() as u64;

        // Analyze document complexity
        let complexity_start = std::time::Instant::now();
        let complexity_analysis = self.complexity_scorer.analyze_content(pdf_bytes)?;
        let complexity_analysis_ms = complexity_start.elapsed().as_millis() as u64;

        // Determine extraction path
        let extraction_path = if self.config.force_native {
            ExtractionPath::Native
        } else if self.config.force_python {
            ExtractionPath::Python
        } else {
            complexity_analysis.recommended_path.clone()
        };

        // Route to appropriate extractor
        match extraction_path {
            ExtractionPath::Native => {
                self.extract_with_native_path(pdf_bytes, page_num, complexity_analysis, cache_key, 
                                            cache_lookup_ms, complexity_analysis_ms, start_time).await
            }
            ExtractionPath::Python => {
                self.extract_with_python_path(pdf_bytes, page_num, complexity_analysis, cache_key,
                                             cache_lookup_ms, complexity_analysis_ms, start_time).await
            }
        }
    }

    /// Extract using native path with optional Python enhancement
    async fn extract_with_native_path(
        &mut self,
        pdf_bytes: &[u8],
        page_num: Option<usize>,
        complexity_analysis: ComplexityAnalysis,
        cache_key: String,
        cache_lookup_ms: u64,
        complexity_analysis_ms: u64,
        start_time: std::time::Instant,
    ) -> Result<HybridExtractionResult> {
        let native_start = std::time::Instant::now();

        // Extract with native extractor
        let native_result = match page_num {
            Some(page) => {
                // For single page, create a temporary file (pdfium-render needs file path)
                let temp_path = self.write_temp_pdf(pdf_bytes).await?;
                let result = self.native_extractor.extract_page(&temp_path, page)?;
                let _ = tokio::fs::remove_file(temp_path).await; // Clean up
                result
            }
            None => self.native_extractor.extract_from_bytes(pdf_bytes)?,
        };

        let native_time_ms = native_start.elapsed().as_millis() as u64;
        let legacy_result = self.native_extractor.to_legacy_format(&native_result);

        // Cache native result
        if self.config.enable_caching {
            self.cache.put_native(cache_key.clone(), native_result.clone());
        }

        // Assess extraction quality
        let quality = self.native_extractor.assess_extraction_quality(&native_result);

        // Decide whether to enhance with Python
        let mut enhancement_result = None;
        let mut python_time_ms = None;

        if self.config.enable_progressive && quality < self.config.quality_threshold {
            // Quality is low, try Python enhancement
            match self.try_python_enhancement(pdf_bytes, page_num).await {
                Ok(python_result) => {
                    python_time_ms = Some(python_result.metadata.processing_time);
                    
                    // Cache Python enhancement
                    if self.config.enable_caching {
                        self.cache.put_python(format!("{}_enhanced", cache_key), python_result.clone());
                    }
                    
                    enhancement_result = Some(python_result);
                }
                Err(_) => {
                    // Python enhancement failed, stick with native result
                }
            }
        }

        Ok(HybridExtractionResult {
            primary_result: legacy_result,
            enhancement_result,
            complexity_analysis,
            extraction_path_used: ExtractionPath::Native,
            performance_metrics: PerformanceMetrics {
                native_time_ms: Some(native_time_ms),
                python_time_ms,
                total_time_ms: start_time.elapsed().as_millis() as u64,
                cache_lookup_ms,
                complexity_analysis_ms,
            },
            cache_hit: false,
        })
    }

    /// Extract using Python path
    async fn extract_with_python_path(
        &mut self,
        pdf_bytes: &[u8],
        page_num: Option<usize>,
        complexity_analysis: ComplexityAnalysis,
        cache_key: String,
        cache_lookup_ms: u64,
        complexity_analysis_ms: u64,
        start_time: std::time::Instant,
    ) -> Result<HybridExtractionResult> {
        let python_start = std::time::Instant::now();

        // Write temporary file for Python extractor
        let temp_path = self.write_temp_pdf(pdf_bytes).await?;

        let python_result = match page_num {
            Some(page) => self.python_extractor.extract_page(&temp_path, page).await?,
            None => self.python_extractor.extract_pdf(&temp_path).await?,
        };

        let python_time_ms = python_start.elapsed().as_millis() as u64;

        // Clean up temporary file
        let _ = tokio::fs::remove_file(temp_path).await;

        // Cache Python result
        if self.config.enable_caching {
            self.cache.put_python(cache_key, python_result.clone());
        }

        Ok(HybridExtractionResult {
            primary_result: python_result,
            enhancement_result: None,
            complexity_analysis,
            extraction_path_used: ExtractionPath::Python,
            performance_metrics: PerformanceMetrics {
                native_time_ms: None,
                python_time_ms: Some(python_time_ms),
                total_time_ms: start_time.elapsed().as_millis() as u64,
                cache_lookup_ms,
                complexity_analysis_ms,
            },
            cache_hit: false,
        })
    }

    /// Try Python enhancement for low-quality native results
    async fn try_python_enhancement(&mut self, pdf_bytes: &[u8], page_num: Option<usize>) -> Result<ExtractionResult> {
        let temp_path = self.write_temp_pdf(pdf_bytes).await?;

        let result = match page_num {
            Some(page) => self.python_extractor.extract_page(&temp_path, page).await,
            None => self.python_extractor.extract_pdf(&temp_path).await,
        };

        let _ = tokio::fs::remove_file(temp_path).await;
        result
    }

    /// Write PDF bytes to temporary file
    async fn write_temp_pdf(&self, pdf_bytes: &[u8]) -> Result<PathBuf> {
        use tokio::io::AsyncWriteExt;
        
        let temp_dir = std::env::temp_dir();
        let temp_filename = format!("chonker_temp_{}.pdf", uuid::Uuid::new_v4());
        let temp_path = temp_dir.join(temp_filename);

        let mut file = tokio::fs::File::create(&temp_path).await?;
        file.write_all(pdf_bytes).await?;
        file.flush().await?;

        Ok(temp_path)
    }

    /// Get performance and cache statistics
    pub fn get_stats(&self) -> HybridStats {
        let (native_cache_entries, python_cache_entries) = self.cache.stats();
        
        HybridStats {
            native_cache_entries,
            python_cache_entries,
            config: self.config.clone(),
        }
    }

    /// Clear all caches
    pub fn clear_cache(&mut self) {
        self.cache.native_cache.clear();
        self.cache.python_cache.clear();
    }

    /// Update configuration
    pub fn update_config(&mut self, config: HybridConfig) {
        self.config = config;
    }
}

#[derive(Debug, Clone)]
pub struct HybridStats {
    pub native_cache_entries: usize,
    pub python_cache_entries: usize,
    pub config: HybridConfig,
}

impl Default for HybridExtractor {
    fn default() -> Self {
        Self::new().expect("Failed to initialize hybrid extractor")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[tokio::test]
    async fn test_hybrid_extractor_initialization() {
        let extractor = HybridExtractor::new();
        assert!(extractor.is_ok(), "Should be able to initialize hybrid extractor");
    }

    #[tokio::test]
    async fn test_cache_functionality() {
        let mut cache = ExtractionCache::new();
        let test_key = "test_key".to_string();
        
        // Test empty cache
        assert!(cache.get_native(&test_key).is_none());
        
        // Test cache stats
        let (native_count, python_count) = cache.stats();
        assert_eq!(native_count, 0);
        assert_eq!(python_count, 0);
    }

    #[test]
    fn test_config_defaults() {
        let config = HybridConfig::default();
        assert!(config.enable_caching);
        assert!(!config.force_native);
        assert!(!config.force_python);
        assert!(config.enable_progressive);
    }
}
