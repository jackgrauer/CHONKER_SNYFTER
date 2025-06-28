use anyhow::{Result, anyhow};
use std::path::Path;

/// Document complexity analysis for routing between fast/complex extraction paths
#[derive(Debug, Clone)]
pub struct ComplexityScorer {
    pub size_threshold_mb: f64,
    pub page_threshold: usize,
    pub text_ratio_threshold: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExtractionPath {
    Native,    // Fast Rust path for simple documents
    Python,    // Complex ML path for advanced documents
}

#[derive(Debug, Clone)]
pub struct ComplexityAnalysis {
    pub score: f64,               // 0.0 = simple, 1.0 = very complex
    pub recommended_path: ExtractionPath,
    pub file_size_mb: f64,
    pub estimated_page_count: usize,
    pub text_to_image_ratio: Option<f64>,
    pub has_forms: bool,
    pub has_complex_layout: bool,
    pub confidence: f64,          // How confident we are in this analysis
}

impl Default for ComplexityScorer {
    fn default() -> Self {
        Self {
            size_threshold_mb: 5.0,      // Files > 5MB likely complex
            page_threshold: 50,          // Documents > 50 pages likely complex
            text_ratio_threshold: 0.7,   // Text ratio < 70% suggests scanned/image-heavy
        }
    }
}

impl ComplexityScorer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Quick analysis based on file metadata only (ultra-fast)
    pub fn analyze_metadata<P: AsRef<Path>>(&self, pdf_path: P) -> Result<ComplexityAnalysis> {
        let path = pdf_path.as_ref();
        let metadata = std::fs::metadata(path)?;
        let file_size_bytes = metadata.len();
        let file_size_mb = file_size_bytes as f64 / (1024.0 * 1024.0);

        // Quick heuristics based on file size
        let size_score = if file_size_mb > self.size_threshold_mb {
            0.8 // Large files are likely complex
        } else if file_size_mb > 1.0 {
            0.3 // Medium files might be complex
        } else {
            0.1 // Small files are usually simple
        };

        let recommended_path = if size_score > 0.5 {
            ExtractionPath::Python
        } else {
            ExtractionPath::Native
        };

        Ok(ComplexityAnalysis {
            score: size_score,
            recommended_path,
            file_size_mb,
            estimated_page_count: (file_size_mb * 20.0) as usize, // Rough estimate: ~50KB per page
            text_to_image_ratio: None, // Would need full analysis
            has_forms: false,          // Would need full analysis
            has_complex_layout: false, // Would need full analysis
            confidence: 0.6,           // Metadata-only analysis has medium confidence
        })
    }

    /// Deep analysis using PDF content (slower but more accurate)
    pub fn analyze_content(&self, pdf_bytes: &[u8]) -> Result<ComplexityAnalysis> {
        let file_size_mb = pdf_bytes.len() as f64 / (1024.0 * 1024.0);
        
        // Try to do basic PDF parsing to get more insights
        let analysis = match self.parse_pdf_structure(pdf_bytes) {
            Ok(parsed) => parsed,
            Err(_) => {
                // If parsing fails, fall back to size-based heuristics
                return self.analyze_from_size(file_size_mb);
            }
        };

        Ok(analysis)
    }

    /// Analyze complexity from file size only (fallback method)
    fn analyze_from_size(&self, file_size_mb: f64) -> Result<ComplexityAnalysis> {
        let score = if file_size_mb > 10.0 {
            0.9
        } else if file_size_mb > self.size_threshold_mb {
            0.7
        } else if file_size_mb > 1.0 {
            0.4
        } else {
            0.2
        };

        let recommended_path = if score > 0.5 {
            ExtractionPath::Python
        } else {
            ExtractionPath::Native
        };

        Ok(ComplexityAnalysis {
            score,
            recommended_path,
            file_size_mb,
            estimated_page_count: (file_size_mb * 15.0) as usize,
            text_to_image_ratio: None,
            has_forms: false,
            has_complex_layout: false,
            confidence: 0.4, // Low confidence without content analysis
        })
    }

    /// Parse PDF structure to determine complexity
    fn parse_pdf_structure(&self, pdf_bytes: &[u8]) -> Result<ComplexityAnalysis> {
        use lopdf::Document;
        
        let doc = Document::load_mem(pdf_bytes)
            .map_err(|e| anyhow!("Failed to parse PDF: {}", e))?;

        let page_count = doc.get_pages().len();
        let file_size_mb = pdf_bytes.len() as f64 / (1024.0 * 1024.0);

        // Analyze document structure
        let mut complexity_factors = Vec::new();
        let mut total_score: f64 = 0.0;

        // Factor 1: Page count
        let page_score = if page_count > self.page_threshold {
            complexity_factors.push("High page count".to_string());
            0.8
        } else if page_count > 20 {
            0.4
        } else {
            0.1
        };
        total_score += page_score * 0.2; // 20% weight

        // Factor 2: File size
        let size_score = if file_size_mb > self.size_threshold_mb {
            complexity_factors.push("Large file size".to_string());
            0.8
        } else if file_size_mb > 1.0 {
            0.4
        } else {
            0.1
        };
        total_score += size_score * 0.3; // 30% weight

        // Factor 3: Object complexity (simplified analysis)
        let object_count = doc.objects.len();
        let objects_per_page = object_count as f64 / page_count as f64;
        let object_score = if objects_per_page > 100.0 {
            complexity_factors.push("High object density".to_string());
            0.9
        } else if objects_per_page > 50.0 {
            0.6
        } else {
            0.2
        };
        total_score += object_score * 0.3; // 30% weight

        // Factor 4: Check for forms and interactive elements
        let has_forms = self.detect_forms(&doc);
        if has_forms {
            complexity_factors.push("Interactive forms detected".to_string());
            total_score += 0.4; // 20% bonus for forms
        }

        // Normalize score to 0-1 range
        total_score = total_score.min(1.0);

        let recommended_path = if total_score > 0.5 || has_forms {
            ExtractionPath::Python
        } else {
            ExtractionPath::Native
        };

        Ok(ComplexityAnalysis {
            score: total_score,
            recommended_path,
            file_size_mb,
            estimated_page_count: page_count,
            text_to_image_ratio: None, // Would need deeper analysis
            has_forms,
            has_complex_layout: objects_per_page > 75.0,
            confidence: 0.8, // High confidence with PDF structure analysis
        })
    }

    /// Detect forms in PDF (simplified)
    fn detect_forms(&self, doc: &lopdf::Document) -> bool {
        // Look for AcroForm dictionary or form fields
        if let Ok(trailer) = doc.trailer.get(b"Root") {
            if let Ok(root) = doc.get_object(trailer.as_reference().unwrap()) {
                if let Ok(root_dict) = root.as_dict() {
                    if root_dict.has(b"AcroForm") {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Get string representation of complexity analysis
    pub fn describe_analysis(&self, analysis: &ComplexityAnalysis) -> String {
        let path_desc = match analysis.recommended_path {
            ExtractionPath::Native => "Fast native extraction",
            ExtractionPath::Python => "Advanced ML extraction",
        };

        let complexity_desc = if analysis.score > 0.7 {
            "High complexity"
        } else if analysis.score > 0.4 {
            "Medium complexity"
        } else {
            "Low complexity"
        };

        format!(
            "{} ({:.1}% complex) → {} | {:.1}MB, ~{} pages, {:.0}% confidence",
            complexity_desc,
            analysis.score * 100.0,
            path_desc,
            analysis.file_size_mb,
            analysis.estimated_page_count,
            analysis.confidence * 100.0
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_metadata_analysis() {
        let scorer = ComplexityScorer::new();
        
        // Create a temporary file
        let mut temp_file = NamedTempFile::new().unwrap();
        let large_content = vec![0u8; 10 * 1024 * 1024]; // 10MB
        temp_file.write_all(&large_content).unwrap();
        
        let analysis = scorer.analyze_metadata(temp_file.path()).unwrap();
        
        assert!(analysis.file_size_mb > 5.0);
        assert_eq!(analysis.recommended_path, ExtractionPath::Python);
        assert!(analysis.score > 0.5);
    }

    #[test]
    fn test_complexity_scoring() {
        let scorer = ComplexityScorer::new();
        
        // Small file should use native path
        let small_analysis = scorer.analyze_from_size(0.5).unwrap();
        assert_eq!(small_analysis.recommended_path, ExtractionPath::Native);
        assert!(small_analysis.score < 0.5);
        
        // Large file should use Python path
        let large_analysis = scorer.analyze_from_size(10.0).unwrap();
        assert_eq!(large_analysis.recommended_path, ExtractionPath::Python);
        assert!(large_analysis.score > 0.5);
    }
}
