use crossterm::event::{KeyCode, KeyEvent};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ProcessingOptions {
    pub ocr_enabled: bool,
    pub formula_recognition: bool,
    pub table_detection: bool,
    pub language: String,
    pub page_start: Option<u32>,
    pub page_end: Option<u32>,
}

impl Default for ProcessingOptions {
    fn default() -> Self {
        Self {
            ocr_enabled: true,
            formula_recognition: true,
            table_detection: true,
            language: "English".to_string(),
            page_start: None,
            page_end: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DocumentChunk {
    pub id: usize,
    pub content: String,
    pub page_range: String,
    pub element_types: Vec<String>,
    pub spatial_bounds: Option<String>,
    pub char_count: usize,
}

#[derive(Debug)]
pub enum AppMode {
    FileSelection,
    Processing,
    Results,
}

#[derive(Debug)]
pub enum SelectedPane {
    FileSelection,
    ProcessingOptions,
    Action,
    PdfViewer,
    ChunkPreview,
}

pub struct App {
    pub mode: AppMode,
    pub selected_pane: SelectedPane,
    pub selected_file: Option<PathBuf>,
    pub file_input: String,
    pub processing_options: ProcessingOptions,
    pub chunks: Vec<DocumentChunk>,
    pub selected_chunk: usize,
    pub processing_progress: f64,
    pub is_processing: bool,
    pub status_message: String,
    pub error_message: Option<String>,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            mode: AppMode::FileSelection,
            selected_pane: SelectedPane::FileSelection,
            selected_file: None,
            file_input: String::new(),
            processing_options: ProcessingOptions::default(),
            chunks: Vec::new(),
            selected_chunk: 0,
            processing_progress: 0.0,
            is_processing: false,
            status_message: "🐹 CHONKER ready! Select a PDF to process".to_string(),
            error_message: None,
            should_quit: false,
        }
    }

    pub fn handle_input(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.should_quit = true;
            }
            KeyCode::Tab => {
                self.next_pane();
            }
            KeyCode::BackTab => {
                self.prev_pane();
            }
            KeyCode::Enter => {
                self.handle_enter();
            }
            KeyCode::Char(' ') => {
                self.handle_space();
            }
            KeyCode::Up => {
                self.handle_up();
            }
            KeyCode::Down => {
                self.handle_down();
            }
            KeyCode::Left => {
                self.handle_left();
            }
            KeyCode::Right => {
                self.handle_right();
            }
            KeyCode::Char(c) => {
                self.handle_char(c);
            }
            KeyCode::Backspace => {
                self.handle_backspace();
            }
            _ => {}
        }
    }

    fn next_pane(&mut self) {
        self.selected_pane = match self.selected_pane {
            SelectedPane::FileSelection => SelectedPane::ProcessingOptions,
            SelectedPane::ProcessingOptions => SelectedPane::Action,
            SelectedPane::Action => match self.mode {
                AppMode::Results => SelectedPane::PdfViewer,
                _ => SelectedPane::FileSelection,
            },
            SelectedPane::PdfViewer => SelectedPane::ChunkPreview,
            SelectedPane::ChunkPreview => SelectedPane::FileSelection,
        };
    }

    fn prev_pane(&mut self) {
        self.selected_pane = match self.selected_pane {
            SelectedPane::FileSelection => match self.mode {
                AppMode::Results => SelectedPane::ChunkPreview,
                _ => SelectedPane::Action,
            },
            SelectedPane::ProcessingOptions => SelectedPane::FileSelection,
            SelectedPane::Action => SelectedPane::ProcessingOptions,
            SelectedPane::PdfViewer => SelectedPane::Action,
            SelectedPane::ChunkPreview => SelectedPane::PdfViewer,
        };
    }

    fn handle_enter(&mut self) {
        match self.selected_pane {
            SelectedPane::FileSelection => {
                // Open file dialog
                self.open_file_dialog();
            }
            SelectedPane::Action => {
                if self.selected_file.is_some() && !self.is_processing {
                    self.start_processing();
                }
            }
            _ => {}
        }
    }

    fn handle_space(&mut self) {
        match self.selected_pane {
            SelectedPane::ProcessingOptions => {
                // Toggle processing options (we'll implement this based on selected option)
                self.processing_options.ocr_enabled = !self.processing_options.ocr_enabled;
                self.status_message = format!("🔧 OCR: {}", if self.processing_options.ocr_enabled { "ON" } else { "OFF" });
            }
            _ => {}
        }
    }

    fn handle_up(&mut self) {
        match self.selected_pane {
            SelectedPane::ChunkPreview => {
                if self.selected_chunk > 0 {
                    self.selected_chunk -= 1;
                }
            }
            _ => {}
        }
    }

    fn handle_down(&mut self) {
        match self.selected_pane {
            SelectedPane::ChunkPreview => {
                if self.selected_chunk < self.chunks.len().saturating_sub(1) {
                    self.selected_chunk += 1;
                }
            }
            _ => {}
        }
    }

    fn handle_left(&mut self) {
        // Navigate between options in processing pane
    }

    fn handle_right(&mut self) {
        // Navigate between options in processing pane
    }

    fn handle_char(&mut self, c: char) {
        match self.selected_pane {
            SelectedPane::FileSelection => {
                self.file_input.push(c);
            }
            _ => {}
        }
    }

    fn handle_backspace(&mut self) {
        match self.selected_pane {
            SelectedPane::FileSelection => {
                self.file_input.pop();
            }
            _ => {}
        }
    }

    fn open_file_dialog(&mut self) {
        if let Some(file) = rfd::FileDialog::new()
            .add_filter("PDF files", &["pdf"])
            .pick_file()
        {
            self.selected_file = Some(file.clone());
            self.file_input = file.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            self.status_message = format!("📄 Selected: {}", self.file_input);
            self.error_message = None;
        }
    }

    fn start_processing(&mut self) {
        if let Some(ref file_path) = self.selected_file {
            self.is_processing = true;
            self.processing_progress = 0.0;
            self.status_message = "🐹 CHONKER is processing PDF... Please wait".to_string();
            self.mode = AppMode::Processing;
            
            // Try to process the actual PDF
            match self.process_pdf(file_path) {
                Ok(chunks) => {
                    self.chunks = chunks;
                    self.processing_progress = 100.0;
                    self.is_processing = false;
                    self.mode = AppMode::Results;
                    self.status_message = format!("✅ Processing complete! {} chunks created", self.chunks.len());
                }
                Err(e) => {
                    self.is_processing = false;
                    self.error_message = Some(format!("❌ Processing failed: {}", e));
                    self.status_message = "❌ Processing failed".to_string();
                }
            }
        }
    }

    fn process_pdf(&self, file_path: &std::path::Path) -> Result<Vec<DocumentChunk>, Box<dyn std::error::Error>> {
        // Extract text from PDF
        let text = pdf_extract::extract_text(file_path)?;
        
        if text.is_empty() {
            return Err("No text found in PDF".into());
        }

        // Simple chunking by paragraphs for now
        let paragraphs: Vec<&str> = text.split("\n\n")
            .filter(|p| !p.trim().is_empty())
            .collect();

        let mut chunks = Vec::new();
        let chunk_size = 1000; // Characters per chunk
        let mut current_chunk = String::new();
        let mut chunk_id = 1;

        for paragraph in paragraphs {
            if current_chunk.len() + paragraph.len() > chunk_size && !current_chunk.is_empty() {
                // Create chunk
                chunks.push(DocumentChunk {
                    id: chunk_id,
                    content: current_chunk.trim().to_string(),
                    page_range: format!("chunk_{}", chunk_id),
                    element_types: vec!["text".to_string()],
                    spatial_bounds: None,
                    char_count: current_chunk.len(),
                });
                current_chunk.clear();
                chunk_id += 1;
            }
            current_chunk.push_str(paragraph);
            current_chunk.push_str("\n\n");
        }

        // Add final chunk if there's content
        if !current_chunk.trim().is_empty() {
            chunks.push(DocumentChunk {
                id: chunk_id,
                content: current_chunk.trim().to_string(),
                page_range: format!("chunk_{}", chunk_id),
                element_types: vec!["text".to_string()],
                spatial_bounds: None,
                char_count: current_chunk.len(),
            });
        }

        if chunks.is_empty() {
            let text_len = text.len();
            chunks.push(DocumentChunk {
                id: 1,
                content: text,
                page_range: "full_document".to_string(),
                element_types: vec!["text".to_string()],
                spatial_bounds: None,
                char_count: text_len,
            });
        }

        Ok(chunks)
    }

    pub fn get_current_chunk(&self) -> Option<&DocumentChunk> {
        self.chunks.get(self.selected_chunk)
    }
}
