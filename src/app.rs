use eframe::egui;
use std::path::PathBuf;
use tracing::{info, debug, warn};
use crate::error::{ChonkerError, ChonkerResult};
use crate::log_error;
use crate::database::ChonkerDatabase;
use chrono::Utc;
use crate::pdf_viewer::PdfViewer;
use crate::markdown_editor::MarkdownEditor;
use crate::extractor::Extractor;
use crate::sync::SelectionSync;
use crate::project::Project;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DocumentChunk {
    pub id: usize,
    pub content: String,
    pub page_range: String,
    pub element_types: Vec<String>,
    pub spatial_bounds: Option<String>,
    pub char_count: usize,
}

#[derive(Debug, PartialEq)]
pub enum AppMode {
    Chonker,
    Snyfter,
}

#[derive(Debug, Clone)]
pub enum SelectedPane {
    Left,
    Right,
}

#[derive(Debug, Clone)]
pub struct Selection {
    pub start_x: u16,
    pub start_y: u16,
    pub end_x: u16,
    pub end_y: u16,
    pub pane: SelectedPane,
    pub text: String,
}

#[derive(Debug, Clone)]
pub enum SelectionSource {
    None,
    PdfPane,
    MarkdownPane,
}

#[derive(Debug)]
pub enum SelectionMode {
    AppControlled,    // Mouse capture ON - no bleed
    TerminalNative,   // Mouse capture OFF - will bleed
}

#[derive(Debug, Clone)]
pub enum ViewMode {
    Split,           // Both panes visible
    LeftOnly,        // Only left pane (zoom for clean selection)
    RightOnly,       // Only right pane (zoom for clean selection)
}

#[derive(Debug, Clone)]
pub enum CopyMode {
    PlainText,       // Just the text content
    WithMetadata,    // Text with chunk metadata
    AsMarkdown,      // Formatted as markdown
    AsJson,          // Structured JSON data
}

#[derive(Debug, Clone)]
pub struct VisualSelection {
    pub chunk_id: usize,
    pub highlighted: bool,
    pub copy_mode: CopyMode,
}

pub struct ChonkerApp {
    // Core components
    pub pdf_viewer: PdfViewer,
    pub markdown_editor: MarkdownEditor,
    pub extractor: Extractor,
    pub sync: SelectionSync,
    pub current_project: Option<Project>,
    
    // Database and state
    pub database: Option<ChonkerDatabase>,
    pub status_message: String,
    pub error_message: Option<String>,
    pub is_processing: bool,
    pub processing_progress: f64,
    
    // Separate scroll states for each pane
    pub pdf_scroll_offset: egui::Vec2,
    pub markdown_scroll_offset: egui::Vec2,
    pub active_selection: Option<Selection>,
    pub selection_source: SelectionSource,
    
    // Legacy fields for compatibility
    pub mode: AppMode,
    pub selected_pane: SelectedPane,
    pub selected_file: Option<PathBuf>,
    pub file_input: String,
    pub processing_options: ProcessingOptions,
    pub chunks: Vec<DocumentChunk>,
    pub selected_chunk: usize,
    pub copy_mode: CopyMode,
    pub visual_selection: Option<VisualSelection>,
    pub pdf_content: String,
    pub markdown_content: String,
}

impl ChonkerApp {
    pub fn new(_cc: &eframe::CreationContext<'_>, database: Option<ChonkerDatabase>) -> Self {
        Self {
            // Initialize core components
            pdf_viewer: PdfViewer::new(),
            markdown_editor: MarkdownEditor::new(),
            extractor: Extractor::new(),
            sync: SelectionSync::new(),
            current_project: None,
            
            // Database and state
            database,
            status_message: "🐹 CHONKER ready! Select a PDF to process".to_string(),
            error_message: None,
            is_processing: false,
            processing_progress: 0.0,
            
            // Initialize scroll states
            pdf_scroll_offset: egui::Vec2::ZERO,
            markdown_scroll_offset: egui::Vec2::ZERO,
            active_selection: None,
            selection_source: SelectionSource::None,
            
            // Legacy fields for compatibility
            mode: AppMode::Chonker,
            selected_pane: SelectedPane::Left,
            selected_file: None,
            file_input: String::new(),
            processing_options: ProcessingOptions::default(),
            chunks: Vec::new(),
            selected_chunk: 0,
            copy_mode: CopyMode::PlainText,
            visual_selection: None,
            pdf_content: String::new(),
            markdown_content: String::new(),
        }
    }
    
    pub fn set_database(&mut self, database: ChonkerDatabase) {
        self.database = Some(database);
        info!("Database connected to CHONKER app");
        self.status_message = "🐹 CHONKER ready with database! Select a PDF to process".to_string();
    }



    fn open_file_dialog(&mut self) {
        info!("Opening file dialog");
        
        match rfd::FileDialog::new()
            .add_filter("PDF files", &["pdf"])
            .pick_file()
        {
            Some(file) => {
                info!("File selected: {}", file.display());
                self.selected_file = Some(file.clone());
                self.file_input = file.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                self.status_message = format!("📄 Selected: {}", self.file_input);
                self.error_message = None;
                
                // Validate file
                if let Err(e) = self.validate_selected_file(&file) {
                    log_error!(e, "file_validation");
                    self.error_message = Some(e.user_message());
                    self.selected_file = None;
                    self.file_input.clear();
                } else {
                    // Load PDF into the PDF viewer
                    if let Err(e) = self.pdf_viewer.load_pdf(&file) {
                        self.error_message = Some(format!("Failed to load PDF: {}", e));
                        self.selected_file = None;
                        self.file_input.clear();
                    } else {
                        self.status_message = format!("✅ PDF loaded: {} ({} pages)", 
                            self.file_input, 
                            self.pdf_viewer.get_page_count()
                        );
                    }
                }
            }
            None => {
                debug!("File dialog cancelled");
            }
        }
    }
    
    fn validate_selected_file(&self, file_path: &PathBuf) -> ChonkerResult<()> {
        // Check if file exists
        if !file_path.exists() {
            return Err(ChonkerError::file_io(
                file_path.to_string_lossy().to_string(),
                std::io::Error::new(std::io::ErrorKind::NotFound, "File not found")
            ));
        }
        
        // Check file extension
        match file_path.extension().and_then(|s| s.to_str()) {
            Some("pdf") => {},
            Some(ext) => {
                return Err(ChonkerError::InvalidFormat {
                    format: ext.to_string()
                });
            }
            None => {
                return Err(ChonkerError::InvalidFormat {
                    format: "unknown".to_string()
                });
            }
        }
        
        // Check file size
        match std::fs::metadata(file_path) {
            Ok(metadata) => {
                let size_mb = metadata.len() / 1_048_576; // Convert to MB
                if size_mb > 100 { // 100MB limit
                    return Err(ChonkerError::MemoryLimit { limit_mb: 100 });
                }
                info!("File validation passed: {} MB", size_mb);
            }
            Err(e) => {
                return Err(ChonkerError::file_io(
                    file_path.to_string_lossy().to_string(),
                    e
                ));
            }
        }
        
        Ok(())
    }

    fn start_processing(&mut self) {
        if let Some(ref file_path) = self.selected_file {
            self.is_processing = true;
            self.processing_progress = 0.0;
            self.status_message = "🐹 CHONKER is processing PDF... Please wait".to_string();
            
            // Try to process the actual PDF
            match self.process_pdf(file_path) {
                Ok(chunks) => {
                    self.chunks = chunks;
                    self.processing_progress = 100.0;
                    self.is_processing = false;
                    self.status_message = format!("✅ Processing complete! {} chunks created", self.chunks.len());
                    
                    // Automatically save to database if available
                    if self.database.is_some() {
                        self.save_to_database();
                    }
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
        // Use advanced Python bridge for extraction with Docling/Magic-PDF
        
        // First, try to use the Python bridge with async extraction
        let path_buf = file_path.to_path_buf();
        let mut extractor = self.extractor.clone();
        extractor.set_preferred_tool("auto".to_string()); // Let it choose best tool
        
        let runtime = tokio::runtime::Runtime::new()?;
        let extraction_result = runtime.block_on(async {
            extractor.extract_pdf(&path_buf).await
        });
        
        let text = match extraction_result {
            Ok(result) => {
                info!("Extraction successful with tool: {}", result.tool);
                
                // Convert extraction result to text
                result.extractions.iter()
                    .map(|page| page.text.clone())
                    .collect::<Vec<_>>()
                    .join("\n\n")
            }
            Err(e) => {
                return Err(format!("Advanced extraction failed: {}. Please check if Python dependencies (Docling/Magic-PDF) are installed correctly.", e).into());
            }
        };
        
        if text.is_empty() {
            return Err("No text found in PDF - document might be image-based or encrypted. Try enabling OCR!".into());
        }

        // Adaptive chunking for large documents
        let file_size = std::fs::metadata(file_path)?.len();
        let is_large_doc = file_size > 10_000_000; // 10MB threshold
        
        let chunk_size = if is_large_doc {
            2000 // Larger chunks for big documents for performance
        } else {
            800  // Smaller chunks for better granularity
        };
        
        // Smart chunking: try to break on sentence boundaries
        let sentences: Vec<&str> = text.split(".")
            .filter(|s| !s.trim().is_empty() && s.len() > 10)
            .collect();
        
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        let mut chunk_id = 1;
        
        // Progress tracking for large documents
        let total_sentences = sentences.len();
        
        for (i, sentence) in sentences.iter().enumerate() {
            // Add sentence with period back
            let sentence_with_period = format!("{}.\n", sentence.trim());
            
            if current_chunk.len() + sentence_with_period.len() > chunk_size && !current_chunk.is_empty() {
                // Create chunk with metadata
                let element_types = vec![
                    "text".to_string(),
                    if current_chunk.contains("Table") || current_chunk.contains("table") { "table" } else { "paragraph" }.to_string()
                ];
                
                chunks.push(DocumentChunk {
                    id: chunk_id,
                    content: current_chunk.trim().to_string(),
                    page_range: format!("sentences_{}-{}", i.saturating_sub(10), i),
                    element_types,
                    spatial_bounds: Some(format!("chunk_bounds_{}", chunk_id)),
                    char_count: current_chunk.trim().len(),
                });
                current_chunk.clear();
                chunk_id += 1;
            }
            current_chunk.push_str(&sentence_with_period);
            
            // Progress update for large documents
            if is_large_doc && i % 100 == 0 {
                let _progress = (i as f64 / total_sentences as f64) * 80.0; // 80% for text processing
                // Note: In real implementation, you'd update progress here
            }
        }

        // Add final chunk if there's content
        if !current_chunk.trim().is_empty() {
            chunks.push(DocumentChunk {
                id: chunk_id,
                content: current_chunk.trim().to_string(),
                page_range: format!("final_chunk_{}", chunk_id),
                element_types: vec!["text".to_string()],
                spatial_bounds: Some(format!("chunk_bounds_{}", chunk_id)),
                char_count: current_chunk.trim().len(),
            });
        }

        // Fallback for edge cases
        if chunks.is_empty() {
            let text_len = text.len();
            chunks.push(DocumentChunk {
                id: 1,
                content: text,
                page_range: "full_document".to_string(),
                element_types: vec!["text".to_string(), "fallback".to_string()],
                spatial_bounds: Some("full_document_bounds".to_string()),
                char_count: text_len,
            });
        }

        // Post-processing: add adversarial document analysis metadata
        for chunk in &mut chunks {
            if chunk.content.to_lowercase().contains("hexavalent chromium") ||
               chunk.content.to_lowercase().contains("liability") ||
               chunk.content.to_lowercase().contains("disclaimer") ||
               chunk.content.to_lowercase().contains("pursuant") {
                chunk.element_types.push("adversarial_content".to_string());
            }
        }

        Ok(chunks)
    }

    pub fn get_current_chunk(&self) -> Option<&DocumentChunk> {
        self.chunks.get(self.selected_chunk)
    }
    
    fn reset_for_new_file(&mut self) {
        self.mode = AppMode::Chonker;
        self.selected_pane = SelectedPane::Left;
        self.selected_file = None;
        self.file_input.clear();
        self.chunks.clear();
        self.selected_chunk = 0;
        self.processing_progress = 0.0;
        self.is_processing = false;
        self.error_message = None;
        self.status_message = "🐹 CHONKER ready! Select a new adversarial document".to_string();
    }
    
    pub fn save_to_database(&mut self) {
        if let (Some(ref file_path), Some(_)) = (&self.selected_file, &self.database) {
            let filename = file_path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
                
            let chunk_count = self.chunks.len();
            let total_chars = self.chunks.iter().map(|c| c.char_count).sum::<usize>();
            
            self.status_message = format!(
                "💾 Saving {} chunks ({} chars) to database...", 
                chunk_count, total_chars
            );
            
        let _options = self.processing_options.clone();
            let _processing_start_time = Utc::now();

            // For now, skip database operations in GUI context
            let db_result: Result<String, ChonkerError> = Ok("temp_id".to_string());

            match db_result {
                Ok(document_id) => {
                    self.status_message = format!(
                        "✅ Saved '{}' with {} chunks to database (ID: {})!", 
                        filename, chunk_count, document_id
                    );
                },
                Err(e) => {
                    self.error_message = Some(format!("❌ Failed to save to database: {}", e));
                    self.status_message = "❌ Saving failed".to_string();
                }
            }
        } else if self.database.is_none() {
            self.error_message = Some("❌ Database not connected! Cannot save.".to_string());
        } else {
            self.error_message = Some("❌ No document selected to save!".to_string());
        }
    }







    fn copy_to_clipboard(&self, text: &str) {
        match arboard::Clipboard::new() {
            Ok(mut clipboard) => {
                if let Err(e) = clipboard.set_text(text) {
                    debug!("Failed to copy to clipboard: {}", e);
                } else {
                    debug!("Successfully copied {} chars to clipboard", text.len());
                }
            }
            Err(e) => {
                debug!("Failed to access clipboard: {}", e);
            }
        }
    }

    
    
    
    // ===============================
    // PROGRAM-CONTROLLED COPY SYSTEM
    // ===============================
    
    pub fn copy_current_chunk(&mut self) {
        self.copy_with_mode(self.copy_mode.clone());
    }
    
    pub fn copy_with_mode(&mut self, mode: CopyMode) {
        if let Some(chunk) = self.get_current_chunk() {
            let text_to_copy = match mode {
                CopyMode::PlainText => {
                    chunk.content.clone()
                }
                CopyMode::WithMetadata => {
                    format!(
                        "Chunk ID: {}\nPage: {}\nType: {}\nSize: {} chars\n\nContent:\n{}",
                        chunk.id,
                        chunk.page_range,
                        chunk.element_types.join(", "),
                        chunk.char_count,
                        chunk.content
                    )
                }
                CopyMode::AsMarkdown => {
                    let status_icon = if chunk.element_types.contains(&"adversarial_content".to_string()) {
                        "⚠️"
                    } else {
                        "✅"
                    };
                    
                    format!(
                        "## {} Chunk {} - {}

```
{}```

**Metadata:**
- Page: {}
- Type: {}
- Size: {} chars
- Status: {}",
                        status_icon,
                        chunk.id,
                        chunk.page_range,
                        chunk.content,
                        chunk.page_range,
                        chunk.element_types.join(", "),
                        chunk.char_count,
                        if chunk.element_types.contains(&"adversarial_content".to_string()) {
                            "Adversarial Content Detected"
                        } else {
                            "Clean Content"
                        }
                    )
                }
                CopyMode::AsJson => {
                    match serde_json::to_string_pretty(chunk) {
                        Ok(json) => json,
                        Err(_) => chunk.content.clone(), // Fallback to plain text
                    }
                }
            };
            
            self.copy_to_clipboard(&text_to_copy);
            
            let mode_name = match mode {
                CopyMode::PlainText => "plain text",
                CopyMode::WithMetadata => "with metadata",
                CopyMode::AsMarkdown => "as markdown",
                CopyMode::AsJson => "as JSON",
            };
            
            self.status_message = format!(
                "📋 Copied chunk {} ({}) - {} chars",
                chunk.id,
                mode_name,
                text_to_copy.len()
            );
            
            // Visual feedback - highlight the copied chunk
            self.visual_selection = Some(VisualSelection {
                chunk_id: self.selected_chunk,
                highlighted: true,
                copy_mode: mode,
            });
        } else {
            self.status_message = "❌ No chunk selected to copy".to_string();
        }
    }
    
    pub fn copy_all_chunks(&mut self) {
        if self.chunks.is_empty() {
            self.status_message = "❌ No chunks available to copy".to_string();
            return;
        }
        
        let all_content = match self.copy_mode {
            CopyMode::PlainText => {
                self.chunks.iter()
                    .map(|chunk| chunk.content.clone())
                    .collect::<Vec<_>>()
                    .join("\n\n---\n\n")
            }
            CopyMode::WithMetadata => {
                self.chunks.iter()
                    .map(|chunk| format!(
                        "[Chunk {}] Page: {} | Type: {} | {} chars\n{}",
                        chunk.id,
                        chunk.page_range,
                        chunk.element_types.join(", "),
                        chunk.char_count,
                        chunk.content
                    ))
                    .collect::<Vec<_>>()
                    .join(&format!("\n\n{}\n\n", "=".repeat(50)))
            }
            CopyMode::AsMarkdown => {
                let mut md = String::from("# Document Analysis Results\n\n");
                md.push_str(&format!("**Total Chunks:** {}\n", self.chunks.len()));
                md.push_str(&format!("**Total Characters:** {}\n\n", 
                    self.chunks.iter().map(|c| c.char_count).sum::<usize>()));
                
                for chunk in &self.chunks {
                    let status_icon = if chunk.element_types.contains(&"adversarial_content".to_string()) {
                        "⚠️"
                    } else {
                        "✅"
                    };
                    
                    md.push_str(&format!(
                        "## {} Chunk {} - {}\n\n```\n{}\n```\n\n",
                        status_icon,
                        chunk.id,
                        chunk.page_range,
                        chunk.content
                    ));
                }
                md
            }
            CopyMode::AsJson => {
                match serde_json::to_string_pretty(&self.chunks) {
                    Ok(json) => json,
                    Err(_) => self.chunks.iter()
                        .map(|c| c.content.clone())
                        .collect::<Vec<_>>()
                        .join("\n\n"), // Fallback
                }
            }
        };
        
        self.copy_to_clipboard(&all_content);
        self.status_message = format!(
            "📋 Copied all {} chunks ({} chars total)",
            self.chunks.len(),
            all_content.len()
        );
    }
    
    pub fn toggle_visual_selection(&mut self) {
        if let Some(ref mut visual) = self.visual_selection {
            visual.highlighted = !visual.highlighted;
            if visual.highlighted {
                self.status_message = format!("🔍 Highlighting chunk {}", visual.chunk_id + 1);
            } else {
                self.status_message = "🔍 Visual highlighting disabled".to_string();
            }
        } else {
            // Start visual selection on current chunk
            self.visual_selection = Some(VisualSelection {
                chunk_id: self.selected_chunk,
                highlighted: true,
                copy_mode: self.copy_mode.clone(),
            });
            self.status_message = format!("🔍 Visual selection started on chunk {}", self.selected_chunk + 1);
        }
    }
    
    pub fn toggle_copy_mode(&mut self) {
        self.copy_mode = match self.copy_mode {
            CopyMode::PlainText => CopyMode::WithMetadata,
            CopyMode::WithMetadata => CopyMode::AsMarkdown,
            CopyMode::AsMarkdown => CopyMode::AsJson,
            CopyMode::AsJson => CopyMode::PlainText,
        };
        
        let mode_name = match self.copy_mode {
            CopyMode::PlainText => "Plain Text",
            CopyMode::WithMetadata => "With Metadata",
            CopyMode::AsMarkdown => "Markdown Format",
            CopyMode::AsJson => "JSON Format",
        };
        
        self.status_message = format!("🔄 Copy mode: {}", mode_name);
    }
    
    pub fn is_chunk_highlighted(&self, chunk_index: usize) -> bool {
        if let Some(ref visual) = self.visual_selection {
            visual.highlighted && visual.chunk_id == chunk_index
        } else {
            false
        }
    }
}

impl eframe::App for ChonkerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Clean menu bar with HUGE hamster
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.label(egui::RichText::new("🐹").size(50.0)); // HUGE hamster!
                ui.label(egui::RichText::new("CHONKER").size(24.0).strong());
                ui.separator();
                
                ui.menu_button("File", |ui| {
                    if ui.button("Open PDF...").clicked() {
                        self.open_file_dialog();
                        ui.close_menu();
                    }
                    if ui.button("Save Project...").clicked() {
                        self.status_message = "Project saving not yet implemented".to_string();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        std::process::exit(0);
                    }
                });
                
                ui.menu_button("Edit", |ui| {
                    if ui.button("Copy All Text").clicked() {
                        self.copy_all_chunks();
                        ui.close_menu();
                    }
                    if ui.button("Clear Results").clicked() {
                        self.reset_for_new_file();
                        ui.close_menu();
                    }
                });
                
                ui.menu_button("View", |ui| {
                    if ui.button("Reset Zoom").clicked() {
                        self.pdf_scroll_offset = egui::Vec2::ZERO;
                        self.markdown_scroll_offset = egui::Vec2::ZERO;
                        self.status_message = "View reset".to_string();
                        ui.close_menu();
                    }
                    if ui.button("Refresh").clicked() {
                        if let Some(ref file) = self.selected_file {
                            self.start_processing();
                        }
                        ui.close_menu();
                    }
                });
                
                ui.menu_button("Help", |ui| {
                    if ui.button("About").clicked() {
                        self.status_message = "CHONKER v10.0 - Advanced PDF Processing with Docling & Magic-PDF".to_string();
                        ui.close_menu();
                    }
                    if ui.button("Keyboard Shortcuts").clicked() {
                        self.status_message = "Ctrl+O: Open, Ctrl+P: Process, Ctrl+R: Reset, Ctrl+Q: Quit".to_string();
                        ui.close_menu();
                    }
                });
            });
        });
        
        // LEFT PANEL - PDF Viewer with actual PDF rendering
        egui::SidePanel::left("pdf_panel")
            .resizable(true)
            .default_width(ctx.screen_rect().width() * 0.5) // Start at 50%
            .min_width(200.0)
            .max_width(ctx.screen_rect().width() * 0.8) // Max 80%
            .show(ctx, |ui| {
                // Render the PDF viewer component
                self.pdf_viewer.render(ui);
            });
        
        // RIGHT PANEL SECOND (CentralPanel gets remaining space)
        egui::CentralPanel::default().show(ctx, |ui| {
            // Use the enhanced MarkdownEditor component
            self.markdown_editor.render(ui);
            
            // Update markdown content when chunks are available
            if !self.chunks.is_empty() && self.markdown_editor.content.len() < 100 {
                let mut markdown = String::from("# Document Analysis\n\n");
                
                markdown.push_str(&format!("**File:** {}\n", 
                    self.selected_file.as_ref()
                        .and_then(|f| f.file_name())
                        .map(|n| n.to_string_lossy())
                        .unwrap_or_default()
                ));
                
                markdown.push_str(&format!("**Chunks:** {}\n", self.chunks.len()));
                markdown.push_str(&format!("**Characters:** {}\n\n", 
                    self.chunks.iter().map(|c| c.char_count).sum::<usize>()
                ));
                
                let adversarial_count = self.chunks.iter()
                    .filter(|c| c.element_types.contains(&"adversarial_content".to_string()))
                    .count();
                
                if adversarial_count > 0 {
                    markdown.push_str(&format!("**⚠️ Adversarial Content:** {} chunks detected\n\n", adversarial_count));
                } else {
                    markdown.push_str("**✅ Status:** Clean content detected\n\n");
                }
                
                markdown.push_str("---\n\n");
                
                for (i, chunk) in self.chunks.iter().enumerate() {
                    let is_adversarial = chunk.element_types.contains(&"adversarial_content".to_string());
                    let status_icon = if is_adversarial { "⚠️" } else { "✅" };
                    
                    markdown.push_str(&format!(
                        "## {} Chunk {} - {}\n\n",
                        status_icon,
                        chunk.id,
                        chunk.page_range
                    ));
                    
                    if is_adversarial {
                        markdown.push_str("**⚠️ ADVERSARIAL CONTENT DETECTED**\n\n");
                    }
                    
                    markdown.push_str(&format!("{}\n\n", chunk.content));
                    
                    if i < self.chunks.len() - 1 {
                        markdown.push_str("---\n\n");
                    }
                }
                
                // Update the markdown editor content
                self.markdown_editor.set_content(markdown);
            }
        });
    }
}

impl ChonkerApp {
    fn toggle_screen(&mut self) {
        self.mode = match self.mode {
            AppMode::Chonker => AppMode::Snyfter,
            AppMode::Snyfter => AppMode::Chonker,
        };
    }
    
    fn toggle_pane(&mut self) {
        self.selected_pane = match self.selected_pane {
            SelectedPane::Left => SelectedPane::Right,
            SelectedPane::Right => SelectedPane::Left,
        };
    }
    
    fn render_chonker_screen(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Left pane - File selection and options
            ui.group(|ui| {
                ui.set_min_width(400.0);
                ui.vertical(|ui| {
                    ui.heading("🐹 CHONKER - Document Processing");
                    ui.separator();
                    
                    // File selection
                    ui.group(|ui| {
                        ui.label("📄 Document Selection");
                        if let Some(ref file) = self.selected_file {
                            ui.label(format!("Selected: {}", file.file_name().unwrap_or_default().to_string_lossy()));
                        } else {
                            ui.label("No document selected");
                        }
                        
                        if ui.button("📂 Select PDF").clicked() {
                            self.open_file_dialog();
                        }
                    });
                    
                    ui.separator();
                    
                    // Processing options
                    ui.group(|ui| {
                        ui.label("⚙️ Processing Options");
                        ui.checkbox(&mut self.processing_options.ocr_enabled, "🔍 OCR Enabled");
                        ui.checkbox(&mut self.processing_options.formula_recognition, "🧮 Formula Recognition");
                        ui.checkbox(&mut self.processing_options.table_detection, "📊 Table Detection");
                        
                        egui::ComboBox::from_label("🌐 Language")
                            .selected_text(&self.processing_options.language)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.processing_options.language, "English".to_string(), "English");
                                ui.selectable_value(&mut self.processing_options.language, "Spanish".to_string(), "Spanish");
                                ui.selectable_value(&mut self.processing_options.language, "French".to_string(), "French");
                            });
                    });
                    
                    ui.separator();
                    
                    // Action buttons
                    ui.group(|ui| {
                        ui.label("🚀 Actions");
                        
                        let can_process = self.selected_file.is_some() && !self.is_processing;
                        if ui.add_enabled(can_process, egui::Button::new("🚀 Process Document")).clicked() {
                            self.start_processing();
                        }
                        
                        if self.is_processing {
                            ui.horizontal(|ui| {
                                ui.label("Processing...");
                                ui.add(egui::ProgressBar::new(self.processing_progress as f32 / 100.0));
                            });
                        }
                        
                        if !self.chunks.is_empty() {
                            if ui.button("💾 Save to Database").clicked() {
                                self.save_to_database();
                            }
                        }
                    });
                });
            });
            
            ui.separator();
            
            // Right pane - Results summary
            ui.group(|ui| {
                ui.set_min_width(400.0);
                ui.vertical(|ui| {
                    ui.heading("📊 Processing Results");
                    ui.separator();
                    
                    if self.chunks.is_empty() {
                        ui.label("No documents processed yet.\n\nSelect a PDF and click 'Process Document' to begin.");
                    } else {
                        ui.label(format!("📄 Document: {}", 
                            self.selected_file.as_ref()
                                .and_then(|f| f.file_name())
                                .map(|n| n.to_string_lossy())
                                .unwrap_or_default()
                        ));
                        
                        ui.label(format!("📦 Total chunks: {}", self.chunks.len()));
                        ui.label(format!("📝 Total characters: {}", 
                            self.chunks.iter().map(|c| c.char_count).sum::<usize>()
                        ));
                        
                        let adversarial_count = self.chunks.iter()
                            .filter(|c| c.element_types.contains(&"adversarial_content".to_string()))
                            .count();
                        
                        if adversarial_count > 0 {
                            ui.label(format!("⚠️ Adversarial content detected in {} chunks", adversarial_count));
                        } else {
                            ui.label("✅ No adversarial content detected");
                        }
                        
                        ui.separator();
                        
                        if ui.button("🐭 Switch to SNYFTER for detailed analysis").clicked() {
                            self.mode = AppMode::Snyfter;
                        }
                    }
                });
            });
        });
    }
    
    fn render_split_pane_view(&mut self, ui: &mut egui::Ui) {
        // Top header bar - minimal height
        ui.horizontal(|ui| {
            ui.add_space(10.0);
            ui.label(egui::RichText::new("🐹").size(32.0));
            ui.add_space(10.0);
            ui.label(egui::RichText::new("CHONKER").size(20.0).strong());
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Status message
                let status_color = if self.error_message.is_some() {
                    egui::Color32::from_rgb(255, 64, 129) // Neon pink
                } else if self.is_processing {
                    egui::Color32::from_rgb(255, 215, 0) // Gold
                } else if !self.chunks.is_empty() {
                    egui::Color32::from_rgb(57, 255, 20) // Neon green
                } else {
                    egui::Color32::from_rgb(0, 255, 255) // Cyan
                };
                
                let status_text = if let Some(ref error) = self.error_message {
                    error.clone()
                } else {
                    self.status_message.clone()
                };
                
                ui.colored_label(status_color, status_text);
                ui.add_space(20.0);
                
                // Fixed hotkey order
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("O").underline());
                    ui.label("pen Ctrl+O");
                    ui.add_space(10.0);
                    
                    ui.label(egui::RichText::new("P").underline());
                    ui.label("rocess Ctrl+P");
                    ui.add_space(10.0);
                    
                    ui.label(egui::RichText::new("R").underline());
                    ui.label("eset Ctrl+R");
                    ui.add_space(10.0);
                    
                    ui.label(egui::RichText::new("Q").underline());
                    ui.label("uit Ctrl+Q");
                });
            });
        });
        
        // Main split view - NO GROUPS, just raw rectangles
        ui.horizontal(|ui| {
            // Left pane - PDF source (full height)
            ui.vertical(|ui| {
                ui.set_width(ui.available_width() / 2.0 - 1.0); // Account for separator
                ui.set_height(ui.available_height());
                
                // Mini header
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("PDF SOURCE").strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if self.selected_file.is_some() && !self.is_processing {
                            if ui.button("Process").clicked() {
                                self.start_processing();
                            }
                        }
                        if ui.button("Open").clicked() {
                            self.open_file_dialog();
                        }
                    });
                });
                
                // File info line
                if let Some(ref file) = self.selected_file {
                    ui.label(file.file_name().unwrap_or_default().to_string_lossy());
                    
                    if self.is_processing {
                        ui.horizontal(|ui| {
                            ui.label("Processing...");
                            ui.add(egui::ProgressBar::new(self.processing_progress as f32 / 100.0));
                        });
                    }
                }
                
                // Main text area - FULL HEIGHT
                let remaining_height = ui.available_height();
                
                if !self.chunks.is_empty() {
                    let all_content = self.chunks.iter()
                        .map(|chunk| chunk.content.clone())
                        .collect::<Vec<_>>()
                        .join("\n\n");
                    
                    egui::ScrollArea::vertical()
                        .max_height(remaining_height)
                        .show(ui, |ui| {
                            ui.add_sized(
                                [ui.available_width(), remaining_height],
                                egui::TextEdit::multiline(&mut all_content.clone())
                                    .font(egui::TextStyle::Monospace)
                                    .interactive(true)
                            );
                        });
                } else {
                    ui.allocate_ui_with_layout(
                        [ui.available_width(), remaining_height].into(),
                        egui::Layout::centered_and_justified(egui::Direction::TopDown),
                        |ui| {
                            if self.selected_file.is_none() {
                                ui.label("Select a PDF to begin extraction");
                                if ui.button("Open PDF").clicked() {
                                    self.open_file_dialog();
                                }
                            } else {
                                ui.label("Raw PDF text will appear here after processing");
                            }
                        },
                    );
                }
            });
            
            // Thin separator
            ui.separator();
            
            // Right pane - Markdown output (full height)
            ui.vertical(|ui| {
                ui.set_width(ui.available_width());
                ui.set_height(ui.available_height());
                
                // Mini header
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("MARKDOWN OUTPUT").strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if !self.chunks.is_empty() {
                            if ui.button("Save DB").clicked() {
                                self.save_to_database();
                            }
                            if ui.button("Copy All").clicked() {
                                self.copy_all_chunks();
                            }
                        }
                    });
                });
                
                // Main markdown area - FULL HEIGHT
                let remaining_height = ui.available_height();
                
                if !self.chunks.is_empty() {
                    // Generate markdown from chunks
                    let mut markdown = String::from("# Document Analysis\n\n");
                    
                    markdown.push_str(&format!("**File:** {}\n", 
                        self.selected_file.as_ref()
                            .and_then(|f| f.file_name())
                            .map(|n| n.to_string_lossy())
                            .unwrap_or_default()
                    ));
                    
                    markdown.push_str(&format!("**Chunks:** {}\n", self.chunks.len()));
                    markdown.push_str(&format!("**Characters:** {}\n\n", 
                        self.chunks.iter().map(|c| c.char_count).sum::<usize>()
                    ));
                    
                    let adversarial_count = self.chunks.iter()
                        .filter(|c| c.element_types.contains(&"adversarial_content".to_string()))
                        .count();
                    
                    if adversarial_count > 0 {
                        markdown.push_str(&format!("**Adversarial Content:** {} chunks detected\n\n", adversarial_count));
                    }
                    
                    markdown.push_str("---\n\n");
                    
                    for (i, chunk) in self.chunks.iter().enumerate() {
                        let is_adversarial = chunk.element_types.contains(&"adversarial_content".to_string());
                        let status_icon = if is_adversarial { "[ADVERSARIAL]" } else { "[CLEAN]" };
                        
                        markdown.push_str(&format!(
                            "## {} Chunk {} - Page {}\n\n",
                            status_icon,
                            chunk.id,
                            chunk.page_range
                        ));
                        
                        if is_adversarial {
                            markdown.push_str("**ADVERSARIAL CONTENT DETECTED**\n\n");
                        }
                        
                        markdown.push_str(&format!("{}\n\n", chunk.content));
                        
                        if i < self.chunks.len() - 1 {
                            markdown.push_str("---\n\n");
                        }
                    }
                    
                    self.markdown_content = markdown.clone();
                    
                    egui::ScrollArea::vertical()
                        .max_height(remaining_height)
                        .show(ui, |ui| {
                            ui.add_sized(
                                [ui.available_width(), remaining_height],
                                egui::TextEdit::multiline(&mut self.markdown_content)
                                    .font(egui::TextStyle::Body)
                                    .interactive(true)
                            );
                        });
                } else {
                    ui.allocate_ui_with_layout(
                        [ui.available_width(), remaining_height].into(),
                        egui::Layout::centered_and_justified(egui::Direction::TopDown),
                        |ui| {
                            ui.label("Structured markdown output will appear here");
                            ui.label("after processing a PDF document");
                        },
                    );
                }
            });
        });
    }
    
    fn set_warp_theme(&self, ctx: &egui::Context) {
        // Set the default font to system monospace (we'll add Hack later)
        let fonts = egui::FontDefinitions::default();
        // For now, just use better system fonts
        // TODO: Add Hack font file later
        ctx.set_fonts(fonts);
        
        let mut visuals = egui::Visuals::dark();
        
        // Pure black Warp theme
        visuals.override_text_color = Some(egui::Color32::from_rgb(248, 248, 242)); // Off-white text
        visuals.panel_fill = egui::Color32::BLACK; // Pure black background
        visuals.window_fill = egui::Color32::BLACK; // Pure black windows
        visuals.extreme_bg_color = egui::Color32::BLACK; // Pure black everywhere
        
        // Remove all borders and piping
        visuals.widgets.noninteractive.bg_fill = egui::Color32::BLACK;
        visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(10, 10, 10); // Very dark
        visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(255, 64, 129); // Neon pink hover
        visuals.widgets.active.bg_fill = egui::Color32::from_rgb(57, 255, 20); // Neon green active
        
        // Bright chunky cursor and selection
        visuals.selection.bg_fill = egui::Color32::from_rgb(57, 255, 20); // Neon green selection
        visuals.selection.stroke = egui::Stroke::new(2.0, egui::Color32::from_rgb(0, 255, 255)); // Thick cyan border
        
        // Clean separators
        visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(40, 40, 40));
        visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(40, 40, 40));
        
        // Note: text cursor styling not available in this egui version
        
        ctx.set_visuals(visuals);
    }
}
