use pdfium_render::prelude::*;
use std::path::Path;

/// Handles fast path PDF processing using pdfium-render
pub struct FastPathProcessor {
    pdfium: Pdfium,
}

impl FastPathProcessor {
    pub fn new() -> Result<Self, String> {
        // Initialize PDFium
        let pdfium = Pdfium::new(
            Pdfium::bind_to_system_library().map_err(|e| e.to_string())?
        );
        Ok(Self { pdfium })
    }

    pub fn extract_text_from_pdf<P: AsRef<Path>>(&self, path: P) -> Result<String, String> {
        let document = self.pdfium.load_pdf_from_file(path.as_ref(), None).map_err(|e| e.to_string())?;
        let mut full_text = String::new();
        for (index, page) in document.pages().iter().enumerate() {
            if index >= 10 { break; } // Skip if more than 10 pages
            let text = page.text().map_err(|e| e.to_string())?.all();
            full_text.push_str(&text);
            full_text.push_str("\n");
        }
        Ok(full_text)
    }
}

impl Default for FastPathProcessor {
    fn default() -> Self {
        Self::new().expect("Failed to initialize FastPathProcessor")
    }
}
