#[cfg(feature = "advanced_pdf")]
use pdfium_render::prelude::*;
use std::path::Path;
use anyhow::{Result, anyhow};

/// Document complexity analyzer for routing decisions
pub struct ComplexityAnalyzer {
    pdfium: Pdfium,
    pub complexity_threshold: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ComplexityScore {
    pub score: f32,           // 0.0 - 10.0 scale
    pub factors: ComplexityFactors,
    pub reasoning: String,
    pub should_use_fast_path: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ComplexityFactors {
    pub page_count: usize,
    pub has_images: bool,
    pub has_tables: bool,
    pub has_forms: bool,
    pub file_size_mb: f32,
    pub has_multiple_columns: bool,
}

impl ComplexityAnalyzer {
    pub fn new() -> Result<Self> {
        let pdfium = Pdfium::new(
            Pdfium::bind_to_system_library()
                .map_err(|e| anyhow!("Failed to initialize PDFium: {}", e))?
        );
        
        Ok(Self {
            pdfium,
            complexity_threshold: 3.0, // Default threshold
        })
    }

    pub fn analyze_pdf<P: AsRef<Path>>(&self, path: P) -> Result<ComplexityScore> {
        let document = self.pdfium.load_pdf_from_file(path.as_ref(), None)
            .map_err(|e| anyhow!("Failed to load PDF: {}", e))?;

        let page_count = document.pages().len();
        
        // Quick complexity scoring
        let mut has_tables = false;
        let mut has_images = false;
        let has_forms = false; // TODO: Implement form detection
        let mut total_text_length = 0;
        let mut total_objects = 0;

        // Analyze first few pages for characteristics
        let pages_to_analyze = std::cmp::min(page_count, 3);
        
        for page_index in 0..pages_to_analyze {
            let page = document.pages().get(page_index)
                .map_err(|e| anyhow!("Failed to get page {}: {}", page_index, e))?;

            // Check text density
            let page_text = page.text()
                .map_err(|e| anyhow!("Failed to extract text: {}", e))?
                .all();
            total_text_length += page_text.len();

            // Check for images and complex objects
            let objects = page.objects();
            total_objects += objects.len();
            
            for object in objects.iter() {
                match object.object_type() {
                    PdfPageObjectType::Image => has_images = true,
                    // Note: Form detection would require more complex analysis
                    _ => {}
                }
            }

            // Simple heuristic for tables: look for grid-like text patterns
            if self.detect_tables_in_text(&page_text) {
                has_tables = true;
            }
        }

        // Calculate complexity score (0-10 scale)
        
        // Factor 1: Page count (0-3 points)
        let page_score = match page_count {
            0..=3 => 0.0,
            4..=10 => 1.0,
            11..=20 => 2.0,
            _ => 3.0,
        };

        // Factor 2: Object density (0-2 points)
        let objects_per_page = total_objects as f64 / pages_to_analyze as f64;
        let object_score = if objects_per_page > 50.0 { 2.0 } else if objects_per_page > 20.0 { 1.0 } else { 0.0 };

        // Factor 3: Special content (0-3 points)
        let content_score = if has_forms { 3.0 } else if has_tables { 2.0 } else if has_images { 1.0 } else { 0.0 };

        // Factor 4: Text density (0-2 points)
        let avg_text_length = total_text_length as f64 / pages_to_analyze as f64;
        let text_density = if avg_text_length < 100.0 { 2.0 } else if avg_text_length < 500.0 { 1.0 } else { 0.0 };

        let score = page_score + object_score + content_score + text_density;
        
        let should_use_fast_path = score < self.complexity_threshold;

        // Get file size
        let file_size_mb = std::fs::metadata(path.as_ref())
            .map(|metadata| metadata.len() as f32 / 1_048_576.0)
            .unwrap_or(0.0);
        
        let factors = ComplexityFactors {
            page_count: page_count as usize,
            has_images,
            has_tables,
            has_forms,
            file_size_mb,
            has_multiple_columns: false, // TODO: Implement column detection
        };
        
        let reasoning = format!(
            "Page count: {}, File size: {:.1}MB, Images: {}, Tables: {}",
            factors.page_count, factors.file_size_mb, factors.has_images, factors.has_tables
        );
        
        Ok(ComplexityScore {
            score: score as f32,
            factors,
            reasoning,
            should_use_fast_path,
        })
    }

    fn detect_tables_in_text(&self, text: &str) -> bool {
        // Simple heuristic: look for multiple tab characters or aligned text patterns
        let tab_count = text.matches('\t').count();
        let pipe_count = text.matches('|').count();
        let lines_with_multiple_spaces = text.lines()
            .filter(|line| line.matches("  ").count() > 3)
            .count();

        tab_count > 10 || pipe_count > 5 || lines_with_multiple_spaces > 3
    }

    /// Simple file-based complexity analysis (doesn't require PDFium)
    pub fn analyze_simple<P: AsRef<Path>>(&self, path: P) -> Result<ComplexityScore> {
        let path = path.as_ref();
        let metadata = std::fs::metadata(path)
            .map_err(|e| anyhow!("Failed to read file metadata: {}", e))?;
        
        let file_size_mb = metadata.len() as f32 / 1_048_576.0;
        
        // Simple heuristics based on file size only
        let score = if file_size_mb < 2.0 {
            2.0  // < 2MB = likely simple
        } else if file_size_mb < 10.0 {
            5.0  // 2-10MB = medium complexity
        } else {
            8.0  // > 10MB = likely complex
        };
        
        let should_use_fast_path = score <= self.complexity_threshold as f32;
        
        let factors = ComplexityFactors {
            page_count: 0, // Unknown without PDF analysis
            has_images: false, // Unknown
            has_tables: false, // Unknown  
            has_forms: false, // Unknown
            file_size_mb,
            has_multiple_columns: false, // Unknown
        };
        
        let reasoning = format!(
            "File size: {:.1}MB - {} complexity (simple heuristic)",
            file_size_mb,
            if score <= 3.0 { "Low" } else if score <= 6.0 { "Medium" } else { "High" }
        );
        
        Ok(ComplexityScore {
            score,
            factors,
            reasoning,
            should_use_fast_path,
        })
    }
    
    pub fn set_threshold(&mut self, threshold: f64) {
        self.complexity_threshold = threshold;
    }
}

impl Default for ComplexityAnalyzer {
    fn default() -> Self {
        Self::new().expect("Failed to initialize ComplexityAnalyzer")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complexity_analyzer_creation() {
        let analyzer = ComplexityAnalyzer::new();
        assert!(analyzer.is_ok());
    }

    #[test]
    fn test_table_detection() {
        let analyzer = ComplexityAnalyzer::new().unwrap();
        
        // Text with tabs (table-like)
        let table_text = "Name\tAge\tCity\nJohn\t25\tNY\nJane\t30\tLA";
        assert!(analyzer.detect_tables_in_text(table_text));
        
        // Regular text
        let normal_text = "This is just normal paragraph text without any table structure.";
        assert!(!analyzer.detect_tables_in_text(normal_text));
    }
}
