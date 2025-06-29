// Dashboard State Management
// Clean, unified state for the Document Surgery Dashboard

use std::path::PathBuf;
use std::time::Instant;

#[derive(Debug, Clone, PartialEq)]
pub enum ProcessingStep {
    Idle,
    Extracting,
    Processing,
    Exporting,
    Complete,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FocusArea {
    DocumentLibrary,
    WorkArea,
    PipelineStatus,
}

#[derive(Debug, Clone)]
pub struct DocumentInfo {
    pub id: String,
    pub filename: String,
    pub file_path: PathBuf,
    pub created_at: String,
    pub file_size: u64,
    pub processing_status: ProcessingStep,
    pub chunk_count: Option<usize>,
    pub last_processed: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ProcessingProgress {
    pub step: ProcessingStep,
    pub started_at: Option<Instant>,
    pub current_operation: String,
    pub estimated_completion: Option<u64>, // seconds
    pub logs: Vec<String>,
}

impl ProcessingProgress {
    pub fn new() -> Self {
        Self {
            step: ProcessingStep::Idle,
            started_at: None,
            current_operation: "Ready".to_string(),
            estimated_completion: None,
            logs: Vec::new(),
        }
    }
    
    pub fn start_step(&mut self, step: ProcessingStep, operation: String) {
        self.step = step;
        self.started_at = Some(Instant::now());
        self.current_operation = operation;
        self.estimated_completion = None;
    }
    
    pub fn add_log(&mut self, message: String) {
        self.logs.push(message);
        // Keep only last 10 logs to prevent memory bloat
        if self.logs.len() > 10 {
            self.logs.remove(0);
        }
    }
    
    pub fn elapsed_seconds(&self) -> u64 {
        self.started_at
            .map(|start| start.elapsed().as_secs())
            .unwrap_or(0)
    }
}

#[derive(Debug)]
pub struct DashboardState {
    // Focus and navigation
    pub current_focus: FocusArea,
    pub should_quit: bool,
    
    // Document library
    pub available_documents: Vec<DocumentInfo>,
    pub selected_document_index: Option<usize>,
    pub library_scroll_offset: usize,
    
    // Work area
    pub current_document: Option<DocumentInfo>,
    pub markdown_content: Option<String>,
    pub extraction_results: Option<String>,
    
    // Processing pipeline
    pub processing_progress: ProcessingProgress,
    
    // Global status
    pub status_message: String,
    pub error_message: Option<String>,
    
    // Export options
    pub export_formats: Vec<String>,
    pub selected_export_format: usize,
}

impl DashboardState {
    pub fn new() -> Self {
        Self {
            current_focus: FocusArea::DocumentLibrary,
            should_quit: false,
            
            available_documents: Vec::new(),
            selected_document_index: None,
            library_scroll_offset: 0,
            
            current_document: None,
            markdown_content: None,
            extraction_results: None,
            
            processing_progress: ProcessingProgress::new(),
            
            status_message: "Welcome to CHONKER Document Surgery Dashboard".to_string(),
            error_message: None,
            
            export_formats: vec!["CSV".to_string(), "JSON".to_string(), "Parquet".to_string()],
            selected_export_format: 0,
        }
    }
    
    pub fn selected_document(&self) -> Option<&DocumentInfo> {
        self.selected_document_index
            .and_then(|idx| self.available_documents.get(idx))
    }
    
    pub fn select_next_document(&mut self) {
        if !self.available_documents.is_empty() {
            self.selected_document_index = Some(match self.selected_document_index {
                Some(idx) if idx < self.available_documents.len() - 1 => idx + 1,
                Some(_) => 0,
                None => 0,
            });
        }
    }
    
    pub fn select_previous_document(&mut self) {
        if !self.available_documents.is_empty() {
            self.selected_document_index = Some(match self.selected_document_index {
                Some(idx) if idx > 0 => idx - 1,
                Some(_) => self.available_documents.len() - 1,
                None => 0,
            });
        }
    }
    
    pub fn set_current_document(&mut self, document: DocumentInfo) {
        self.current_document = Some(document);
        self.processing_progress = ProcessingProgress::new();
        self.markdown_content = None;
        self.extraction_results = None;
    }
    
    pub fn cycle_focus(&mut self) {
        self.current_focus = match self.current_focus {
            FocusArea::DocumentLibrary => FocusArea::WorkArea,
            FocusArea::WorkArea => FocusArea::PipelineStatus,
            FocusArea::PipelineStatus => FocusArea::DocumentLibrary,
        };
    }
    
    pub fn set_status(&mut self, message: String) {
        self.status_message = message;
        self.error_message = None;
    }
    
    pub fn set_error(&mut self, error: String) {
        self.error_message = Some(error);
    }
    
    pub fn clear_error(&mut self) {
        self.error_message = None;
    }
}
