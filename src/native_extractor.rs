use anyhow::{Result, anyhow};
use pdfium_render::prelude::*;
use std::path::Path;
use serde::{Deserialize, Serialize};

/// Native Rust PDF extractor for fast path processing
/// Uses pdfium-render for high-quality text extraction
pub struct NativeExtractor {
    pdfium: Pdfium,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeExtractionResult {
    pub success: bool,
    pub tool: String,
    pub extractions: Vec<NativePageExtraction>,
    pub metadata: NativeExtractionMetadata,
    pub error: Option<String>,
    pub performance_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativePageExtraction {
    pub page_number: usize,
    pub text: String,
    pub word_count: usize,
    pub character_count: usize,
    pub has_images: bool,
    pub confidence: f64,
    pub tool: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeExtractionMetadata {
    pub total_pages: usize,
    pub total_words: usize,
    pub total_characters: usize,
    pub processing_time_ms: u64,
    pub pages_with_images: usize,
    pub extraction_method: String,
}

impl NativeExtractor {
    pub fn new() -> Result<Self> {
        let pdfium = Pdfium::new(
            Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./"))
                .or_else(|_| Pdfium::bind_to_system_library())
                .map_err(|e| anyhow!("Failed to initialize PDFium: {}", e))?
        );

        Ok(Self { pdfium })
    }

    /// Extract text from PDF file (fast path)
    pub fn extract_pdf<P: AsRef<Path>>(&self, pdf_path: P) -> Result<NativeExtractionResult> {
        let start_time = std::time::Instant::now();
        
        let document = self.pdfium
            .load_pdf_from_file(pdf_path, None)
            .map_err(|e| anyhow!("Failed to load PDF: {}", e))?;

        let total_pages = document.pages().len();
        let mut extractions = Vec::new();
        let mut total_words = 0;
        let mut total_characters = 0;
        let mut pages_with_images = 0;

        for page_index in 0..total_pages {
            let page = document.pages().get(page_index)
                .map_err(|e| anyhow!("Failed to get page {}: {}", page_index + 1, e))?;

            // Extract text using PDFium's text extraction
            let text = page.text()
                .map_err(|e| anyhow!("Failed to extract text from page {}: {}", page_index + 1, e))?
                .all();

            // Count words and characters
            let word_count = text.split_whitespace().count();
            let character_count = text.chars().count();
            
            total_words += word_count;
            total_characters += character_count;

            // Check for images (simplified detection)
            let has_images = self.page_has_images(&page);
            if has_images {
                pages_with_images += 1;
            }

            let page_extraction = NativePageExtraction {
                page_number: page_index + 1,
                text,
                word_count,
                character_count,
                has_images,
                confidence: 0.9, // PDFium is generally high confidence for text
                tool: "native-pdfium".to_string(),
            };

            extractions.push(page_extraction);
        }

        let processing_time_ms = start_time.elapsed().as_millis() as u64;

        Ok(NativeExtractionResult {
            success: true,
            tool: "native-pdfium".to_string(),
            extractions,
            metadata: NativeExtractionMetadata {
                total_pages,
                total_words,
                total_characters,
                processing_time_ms,
                pages_with_images,
                extraction_method: "PDFium native text extraction".to_string(),
            },
            error: None,
            performance_ms: processing_time_ms,
        })
    }

    /// Extract text from specific page
    pub fn extract_page<P: AsRef<Path>>(&self, pdf_path: P, page_num: usize) -> Result<NativeExtractionResult> {
        let start_time = std::time::Instant::now();
        
        let document = self.pdfium
            .load_pdf_from_file(pdf_path, None)
            .map_err(|e| anyhow!("Failed to load PDF: {}", e))?;

        let total_pages = document.pages().len();
        
        if page_num == 0 || page_num > total_pages {
            return Err(anyhow!("Invalid page number: {}. Document has {} pages.", page_num, total_pages));
        }

        let page_index = page_num - 1; // Convert to 0-based index
        let page = document.pages().get(page_index)
            .map_err(|e| anyhow!("Failed to get page {}: {}", page_num, e))?;

        let text = page.text()
            .map_err(|e| anyhow!("Failed to extract text from page {}: {}", page_num, e))?
            .all();

        let word_count = text.split_whitespace().count();
        let character_count = text.chars().count();
        let has_images = self.page_has_images(&page);

        let processing_time_ms = start_time.elapsed().as_millis() as u64;

        let page_extraction = NativePageExtraction {
            page_number: page_num,
            text,
            word_count,
            character_count,
            has_images,
            confidence: 0.9,
            tool: "native-pdfium".to_string(),
        };

        Ok(NativeExtractionResult {
            success: true,
            tool: "native-pdfium".to_string(),
            extractions: vec![page_extraction],
            metadata: NativeExtractionMetadata {
                total_pages,
                total_words: word_count,
                total_characters: character_count,
                processing_time_ms,
                pages_with_images: if has_images { 1 } else { 0 },
                extraction_method: "PDFium native text extraction (single page)".to_string(),
            },
            error: None,
            performance_ms: processing_time_ms,
        })
    }

    /// Extract text from PDF bytes
    pub fn extract_from_bytes(&self, pdf_bytes: &[u8]) -> Result<NativeExtractionResult> {
        let start_time = std::time::Instant::now();
        
        let document = self.pdfium
            .load_pdf_from_bytes(pdf_bytes, None)
            .map_err(|e| anyhow!("Failed to load PDF from bytes: {}", e))?;

        let total_pages = document.pages().len();
        let mut extractions = Vec::new();
        let mut total_words = 0;
        let mut total_characters = 0;
        let mut pages_with_images = 0;

        for page_index in 0..total_pages {
            let page = document.pages().get(page_index)
                .map_err(|e| anyhow!("Failed to get page {}: {}", page_index + 1, e))?;

            let text = page.text()
                .map_err(|e| anyhow!("Failed to extract text from page {}: {}", page_index + 1, e))?
                .all();

            let word_count = text.split_whitespace().count();
            let character_count = text.chars().count();
            
            total_words += word_count;
            total_characters += character_count;

            let has_images = self.page_has_images(&page);
            if has_images {
                pages_with_images += 1;
            }

            let page_extraction = NativePageExtraction {
                page_number: page_index + 1,
                text,
                word_count,
                character_count,
                has_images,
                confidence: 0.9,
                tool: "native-pdfium".to_string(),
            };

            extractions.push(page_extraction);
        }

        let processing_time_ms = start_time.elapsed().as_millis() as u64;

        Ok(NativeExtractionResult {
            success: true,
            tool: "native-pdfium".to_string(),
            extractions,
            metadata: NativeExtractionMetadata {
                total_pages,
                total_words,
                total_characters,
                processing_time_ms,
                pages_with_images,
                extraction_method: "PDFium native text extraction (from bytes)".to_string(),
            },
            error: None,
            performance_ms: processing_time_ms,
        })
    }

    /// Simple image detection on page
    fn page_has_images(&self, page: &PdfPage) -> bool {
        // Simple heuristic: check if page has objects that might be images
        // This is a simplified approach - full image detection would be more complex
        match page.objects() {
            Ok(objects) => {
                for object in objects.iter() {
                    if matches!(object.object_type(), PdfPageObjectType::Image) {
                        return true;
                    }
                }
                false
            }
            Err(_) => false, // If we can't check objects, assume no images
        }
    }

    /// Get text extraction quality score
    pub fn assess_extraction_quality(&self, result: &NativeExtractionResult) -> f64 {
        if result.extractions.is_empty() {
            return 0.0;
        }

        let total_chars = result.metadata.total_characters;
        let total_words = result.metadata.total_words;
        
        // Heuristics for quality assessment
        let avg_word_length = if total_words > 0 {
            total_chars as f64 / total_words as f64
        } else {
            0.0
        };

        // Good text should have reasonable word lengths (2-15 characters average)
        let word_length_score = if avg_word_length >= 2.0 && avg_word_length <= 15.0 {
            1.0
        } else if avg_word_length >= 1.0 && avg_word_length <= 25.0 {
            0.7
        } else {
            0.3
        };

        // Pages with more text content generally extract better
        let content_density_score = if total_chars > 100 {
            1.0
        } else if total_chars > 50 {
            0.8
        } else if total_chars > 10 {
            0.5
        } else {
            0.2
        };

        // Combine scores
        (word_length_score * 0.6 + content_density_score * 0.4).min(1.0)
    }

    /// Convert to compatible format with existing extractor
    pub fn to_legacy_format(&self, result: &NativeExtractionResult) -> crate::extractor::ExtractionResult {
        let extractions = result.extractions.iter().map(|page| {
            crate::extractor::PageExtraction {
                page_number: page.page_number,
                text: page.text.clone(),
                tables: vec![], // Native extractor doesn't detect tables
                figures: vec![], // Native extractor doesn't detect figures
                formulas: vec![], // Native extractor doesn't detect formulas
                confidence: page.confidence,
                layout_boxes: vec![], // Native extractor doesn't provide layout boxes
                tool: page.tool.clone(),
            }
        }).collect();

        crate::extractor::ExtractionResult {
            success: result.success,
            tool: result.tool.clone(),
            extractions,
            metadata: crate::extractor::ExtractionMetadata {
                total_pages: result.metadata.total_pages,
                processing_time: result.metadata.processing_time_ms,
            },
            error: result.error.clone(),
        }
    }
}

impl Default for NativeExtractor {
    fn default() -> Self {
        Self::new().expect("Failed to initialize native PDF extractor")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_extractor_initialization() {
        let extractor = NativeExtractor::new();
        assert!(extractor.is_ok(), "Should be able to initialize native extractor");
    }

    #[test]
    fn test_quality_assessment() {
        let extractor = NativeExtractor::new().unwrap();
        
        // Good quality result
        let good_result = NativeExtractionResult {
            success: true,
            tool: "test".to_string(),
            extractions: vec![],
            metadata: NativeExtractionMetadata {
                total_pages: 1,
                total_words: 100,
                total_characters: 500,
                processing_time_ms: 10,
                pages_with_images: 0,
                extraction_method: "test".to_string(),
            },
            error: None,
            performance_ms: 10,
        };
        
        let quality = extractor.assess_extraction_quality(&good_result);
        assert!(quality > 0.7, "Good extraction should have high quality score");

        // Poor quality result
        let poor_result = NativeExtractionResult {
            success: true,
            tool: "test".to_string(),
            extractions: vec![],
            metadata: NativeExtractionMetadata {
                total_pages: 1,
                total_words: 1,
                total_characters: 5,
                processing_time_ms: 10,
                pages_with_images: 0,
                extraction_method: "test".to_string(),
            },
            error: None,
            performance_ms: 10,
        };
        
        let quality = extractor.assess_extraction_quality(&poor_result);
        assert!(quality < 0.6, "Poor extraction should have low quality score");
    }
}
