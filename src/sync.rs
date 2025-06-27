use std::collections::HashMap;

/// Selection synchronization between PDF viewer and markdown editor
/// Handles bidirectional highlighting and text mapping
pub struct SelectionSync {
    pub pdf_to_markdown_map: HashMap<PdfSelection, MarkdownRange>,
    pub markdown_to_pdf_map: HashMap<MarkdownRange, PdfSelection>,
    pub active_highlights: Vec<SyncedHighlight>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct PdfSelection {
    pub page: usize,
    pub start_coords: (i32, i32),
    pub end_coords: (i32, i32),
    pub text: String,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct MarkdownRange {
    pub start_char: usize,
    pub end_char: usize,
    pub line_start: usize,
    pub line_end: usize,
}

#[derive(Debug, Clone)]
pub struct SyncedHighlight {
    pub id: usize,
    pub pdf_selection: PdfSelection,
    pub markdown_range: MarkdownRange,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub highlight_type: HighlightType,
}

#[derive(Debug, Clone)]
pub enum HighlightType {
    Selection,      // User selected text
    Correction,     // User corrected extraction
    Verification,   // User verified accuracy
    Error,          // Extraction error
}

impl Default for SelectionSync {
    fn default() -> Self {
        Self {
            pdf_to_markdown_map: HashMap::new(),
            markdown_to_pdf_map: HashMap::new(),
            active_highlights: Vec::new(),
        }
    }
}

impl SelectionSync {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn map_pdf_to_markdown(&mut self, pdf_selection: PdfSelection, markdown_range: MarkdownRange) {
        self.pdf_to_markdown_map.insert(pdf_selection.clone(), markdown_range.clone());
        self.markdown_to_pdf_map.insert(markdown_range, pdf_selection);
    }
    
    pub fn find_markdown_from_pdf(&self, pdf_selection: &PdfSelection) -> Option<&MarkdownRange> {
        self.pdf_to_markdown_map.get(pdf_selection)
    }
    
    pub fn find_pdf_from_markdown(&self, markdown_range: &MarkdownRange) -> Option<&PdfSelection> {
        self.markdown_to_pdf_map.get(markdown_range)
    }
    
    pub fn create_highlight(&mut self, pdf_selection: PdfSelection, markdown_range: MarkdownRange, highlight_type: HighlightType) -> usize {
        let id = self.active_highlights.len();
        
        let highlight = SyncedHighlight {
            id,
            pdf_selection: pdf_selection.clone(),
            markdown_range: markdown_range.clone(),
            timestamp: chrono::Utc::now(),
            highlight_type,
        };
        
        self.active_highlights.push(highlight);
        self.map_pdf_to_markdown(pdf_selection, markdown_range);
        
        id
    }
    
    pub fn remove_highlight(&mut self, highlight_id: usize) -> bool {
        if let Some(pos) = self.active_highlights.iter().position(|h| h.id == highlight_id) {
            let highlight = self.active_highlights.remove(pos);
            
            // Remove from maps
            self.pdf_to_markdown_map.remove(&highlight.pdf_selection);
            self.markdown_to_pdf_map.remove(&highlight.markdown_range);
            
            true
        } else {
            false
        }
    }
    
    pub fn clear_highlights(&mut self) {
        self.active_highlights.clear();
        self.pdf_to_markdown_map.clear();
        self.markdown_to_pdf_map.clear();
    }
    
    pub fn get_highlights_for_page(&self, page: usize) -> Vec<&SyncedHighlight> {
        self.active_highlights
            .iter()
            .filter(|h| h.pdf_selection.page == page)
            .collect()
    }
    
    pub fn get_highlights_for_markdown_range(&self, start: usize, end: usize) -> Vec<&SyncedHighlight> {
        self.active_highlights
            .iter()
            .filter(|h| {
                let range = &h.markdown_range;
                !(range.end_char < start || range.start_char > end)
            })
            .collect()
    }
    
    pub fn update_markdown_mapping(&mut self, old_range: MarkdownRange, new_range: MarkdownRange) {
        if let Some(pdf_selection) = self.markdown_to_pdf_map.remove(&old_range) {
            self.markdown_to_pdf_map.insert(new_range.clone(), pdf_selection.clone());
            self.pdf_to_markdown_map.insert(pdf_selection, new_range.clone());
            
            // Update active highlights
            for highlight in &mut self.active_highlights {
                if highlight.markdown_range == old_range {
                    highlight.markdown_range = new_range.clone();
                    break;
                }
            }
        }
    }
    
    pub fn rebuild_mappings_from_extraction(&mut self, _extraction_result: &crate::extractor::ExtractionResult) {
        // TODO: Implement intelligent mapping rebuild
        // This will analyze the extraction result and attempt to preserve
        // existing mappings while updating them for the new content
        
        self.clear_highlights();
        
        // For now, just clear everything
        // In the future, this will use fuzzy matching to preserve mappings
    }
}
