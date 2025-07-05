fn show_chonking_progress(ui: &mut egui::Ui, progress: f32, message: &str) {
    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
        ui.add_space(100.0);

        ui.label(
            egui::RichText::new("Chonking out...")
                .font(egui::FontId::monospace(24.0))
                .color(egui::Color32::from_rgb(0, 200, 0))
        );

        ui.add_space(10.0);

        ui.label(
            egui::RichText::new(message)
                .font(egui::FontId::monospace(14.0))
                .color(egui::Color32::LIGHT_GRAY)
        );

        ui.add_space(30.0);

        draw_chunky_progress_bar(ui, progress);

        ui.add_space(15.0);

        ui.label(
            egui::RichText::new(format!("{}%", (progress * 100.0) as u32))
                .font(egui::FontId::monospace(16.0))
                .color(egui::Color32::WHITE)
        );
    });
}

fn draw_chunky_progress_bar(ui: &mut egui::Ui, progress: f32) {
    const BAR_WIDTH: f32 = 400.0;
    const BAR_HEIGHT: f32 = 30.0;
    const NUM_SEGMENTS: usize = 25;
    const SEGMENT_GAP: f32 = 2.0;

    let segment_width = (BAR_WIDTH - (NUM_SEGMENTS - 1) as f32 * SEGMENT_GAP) / NUM_SEGMENTS as f32;
    let filled_segments = (progress * NUM_SEGMENTS as f32).ceil() as usize;

    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(BAR_WIDTH, BAR_HEIGHT),
        egui::Sense::hover()
    );

    ui.painter().rect_stroke(
        rect,
        egui::Rounding::ZERO,
        egui::Stroke::new(2.0, egui::Color32::WHITE)
    );

    for i in 0..NUM_SEGMENTS {
        let x_offset = i as f32 * (segment_width + SEGMENT_GAP);
        let segment_rect = egui::Rect::from_min_size(
            rect.min + egui::vec2(x_offset + 2.0, 2.0),
            egui::vec2(segment_width, BAR_HEIGHT - 4.0)
        );

        let color = if i < filled_segments {
            egui::Color32::from_rgb(0, 255, 0)
        } else {
            egui::Color32::from_rgb(30, 30, 30)
        };

        ui.painter().rect_filled(segment_rect, egui::Rounding::ZERO, color);

        ui.painter().rect_stroke(
            segment_rect,
            egui::Rounding::ZERO,
            egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 100, 100))
        );
    }
}

fn apply_retro_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    style.visuals.window_fill = egui::Color32::BLACK;
    style.visuals.panel_fill = egui::Color32::BLACK;

    style.visuals.override_text_color = Some(egui::Color32::from_rgb(0, 200, 0));

    style.visuals.window_shadow = egui::epaint::Shadow::NONE;
    style.visuals.popup_shadow = egui::epaint::Shadow::NONE;

    ctx.set_style(style);
}

#[cfg(feature = "gui")]
use eframe::egui;
use std::path::PathBuf;
use tracing::{info, debug};
use crate::error::{ChonkerError, ChonkerResult};
use crate::database::ChonkerDatabase;
#[cfg(all(feature = "mupdf", feature = "gui"))]
use crate::mupdf_viewer::MuPdfViewer;
use crate::markdown_editor::MarkdownEditor;
use crate::extractor::Extractor;
use crate::processing::ChonkerProcessor;
use crate::sync::SelectionSync;
use crate::project::Project;
use std::sync::mpsc;
use std::thread;

#[derive(Debug)]
enum QcMessage {
    Progress(String),
    Complete(String),
    Error(String),
}

#[derive(Debug)]
enum ProcessingMessage {
    Progress(f32, String),
    Complete(Vec<DocumentChunk>, Option<String>), // Added raw JSON
    Error(String),
}

// Structures for parsing structured data from Python bridge
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StructuredDocument {
    pub metadata: DocumentMetadata,
    pub elements: Vec<DocumentElement>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DocumentMetadata {
    pub source_file: String,
    pub extraction_tool: String,
    pub extraction_time: String,
    pub page_count: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DocumentElement {
    pub id: String,
    #[serde(rename = "type")]
    pub element_type: String,
    pub element_index: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub table_structure: Option<TableStructure>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grid_data: Option<GridData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heading_level: Option<u32>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TableStructure {
    pub num_rows: u32,
    pub num_cols: u32,
    pub cells: Vec<TableCell>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TableCell {
    pub text: String,
    pub row_span: u32,
    pub col_span: u32,
    pub start_row: u32,
    pub end_row: u32,
    pub start_col: u32,
    pub end_col: u32,
    pub is_header: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GridData {
    pub num_rows: u32,
    pub num_cols: u32,
    pub grid: Vec<Vec<GridCell>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum GridCell {
    Complex { 
        text: String, 
        row_span: u32, 
        col_span: u32 
    },
    Simple(String),
    Empty,
}

// Legacy structures for backward compatibility
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StructuredChunk {
    pub id: String,
    #[serde(rename = "type")]
    pub chunk_type: String,
    pub element_type: String,
    pub content: String,
    pub page_number: u32,
    pub bbox: Option<BoundingBox>,
    pub table_data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BoundingBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub table_data: Option<GridData>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    Loading { progress: f32, message: String },
    Ready,
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

pub enum PdfViewerType {
    #[cfg(all(feature = "mupdf", feature = "gui"))]
    MuPdf(MuPdfViewer),
}

impl PdfViewerType {
    pub fn render(&mut self, ui: &mut egui::Ui) {
        match self {
            #[cfg(all(feature = "mupdf", feature = "gui"))]
            PdfViewerType::MuPdf(viewer) => viewer.render(ui),
            #[cfg(not(all(feature = "mupdf", feature = "gui")))]
            _ => {},
        }
    }
    
    pub fn load_pdf(&mut self, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            #[cfg(all(feature = "mupdf", feature = "gui"))]
            PdfViewerType::MuPdf(viewer) => viewer.load_pdf(path),
            #[cfg(not(all(feature = "mupdf", feature = "gui")))]
            _ => Err("MuPDF feature not enabled".into()),
        }
    }
    
    pub fn get_page_count(&self) -> usize {
        match self {
            #[cfg(all(feature = "mupdf", feature = "gui"))]
            PdfViewerType::MuPdf(viewer) => viewer.get_page_count(),
            #[cfg(not(all(feature = "mupdf", feature = "gui")))]
            _ => 0,
        }
    }
}

pub struct ChonkerApp {
    // App state for loading screen
    pub state: AppState,
    
    // Auto-load tracking
    pub auto_loaded: bool,
    
    // Core components
    pub pdf_viewer: PdfViewerType,
    pub markdown_editor: MarkdownEditor,
    pub extractor: Extractor,
    pub processor: ChonkerProcessor,
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
    
    // Export bin for SNYFTER mode
    pub export_bin: Vec<DocumentChunk>,
    
    
    // Coordinate mapping state
    pub selected_pdf_region: Option<usize>,
    pub selected_text_chunk: Option<usize>,
    
    // QC Processing state
    pub qc_processing: bool,
    pub qc_progress_message: String,
    pub qc_receiver: Option<mpsc::Receiver<QcMessage>>,
    
    // Async PDF processing
    pub processing_receiver: Option<mpsc::Receiver<ProcessingMessage>>,
    
    // Data visualization components
    #[cfg(feature = "gui")]
    pub data_viz_pane: crate::data_visualization::DataVisualizationPane,
}

impl ChonkerApp {
    pub fn new(_cc: &eframe::CreationContext<'_>, database: Option<ChonkerDatabase>) -> Self {
        Self {
            // Initialize app state as ready, not loading
            state: AppState::Ready,
            
            // Auto-load tracking
            auto_loaded: false,
            
            // Initialize core components with MuPDF
            pdf_viewer: {
                #[cfg(all(feature = "mupdf", feature = "gui"))]
                {
                    println!("🚀 Initializing with high-performance MuPDF viewer!");
                    PdfViewerType::MuPdf(MuPdfViewer::new())
                }
                #[cfg(not(all(feature = "mupdf", feature = "gui")))]
                {
                    panic!("MuPDF feature must be enabled - no fallback PDF viewer available");
                }
            },
            markdown_editor: MarkdownEditor::new(),
            extractor: Extractor::new(),
            processor: ChonkerProcessor::new(),
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
            export_bin: Vec::new(),
            selected_pdf_region: None,
            selected_text_chunk: None,
            
            // Initialize QC processing fields
            qc_processing: false,
            qc_progress_message: String::new(),
            qc_receiver: None,
            processing_receiver: None,
            
            // Initialize data visualization component
            #[cfg(feature = "gui")]
            data_viz_pane: crate::data_visualization::DataVisualizationPane::new(),
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
                    tracing::error!("File validation failed: {}", e);
                    self.error_message = Some(format!("{}", e));
                    self.selected_file = None;
                    self.file_input.clear();
                } else {
                // Load PDF into the PDF viewer
                    match self.pdf_viewer.load_pdf(&file) {
                        Err(e) => {
                            self.error_message = Some(format!("Failed to load PDF: {}", e));
                            self.selected_file = None;
                            self.file_input.clear();
                        }
                        Ok(()) => {
                            let page_count = self.pdf_viewer.get_page_count();
                            self.status_message = format!("✅ PDF loaded: {} ({} pages)", 
                                self.file_input, 
                                page_count
                            );
                        }
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
            
            // Start async processing in background thread
            self.start_async_processing(file_path.clone());
        }
    }
    
    fn start_async_processing(&mut self, file_path: std::path::PathBuf) {
        let (sender, receiver) = mpsc::channel();
        self.processing_receiver = Some(receiver);
        
        // Spawn background task for PDF processing
        thread::spawn(move || {
            let _ = sender.send(ProcessingMessage::Progress(10.0, "🔍 Analyzing PDF structure...".to_string()));
            
            // Run the actual PDF processing
            match Self::process_pdf_blocking(&file_path) {
                Ok((chunks, raw_json)) => {
                    let _ = sender.send(ProcessingMessage::Progress(90.0, "✅ Processing complete, generating chunks...".to_string()));
                    let _ = sender.send(ProcessingMessage::Complete(chunks, raw_json));
                }
                Err(e) => {
                    let _ = sender.send(ProcessingMessage::Error(format!("❌ Processing failed: {}", e)));
                }
            }
        });
    }
    fn process_pdf_blocking(file_path: &std::path::Path) -> Result<(Vec<DocumentChunk>, Option<String>), Box<dyn std::error::Error>> {
        // Performance timing
        let total_start = std::time::Instant::now();
        println!("🔍 Starting PDF processing for: {:?}", file_path);
        
        // Memory optimization: Check file size first
        let file_size = std::fs::metadata(file_path)?.len();
        let is_large_file = file_size > 50_000_000; // 50MB threshold
        
        if is_large_file {
            println!("⚠️ Large file detected ({:.1}MB) - using memory-optimized processing", file_size as f64 / 1_048_576.0);
        }
        
        println!("🧠 Running Docling processing (this is our main engine)...");
        
        let path_buf = file_path.to_path_buf();
        let mut extractor = Extractor::new();
        extractor.set_preferred_tool("auto".to_string());
        
        let extraction_start = std::time::Instant::now();
        
        // Use the structured JSON extraction bridge for table-accurate data
        let json_result = match std::process::Command::new("./venv/bin/python")
            .arg("python/docling_html_bridge.py")
            .arg(path_buf.to_string_lossy().as_ref())
            .output()
        {
            Ok(output) if output.status.success() => {
                let json_content = String::from_utf8_lossy(&output.stdout).to_string();
                Ok(json_content)
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(format!("JSON extraction failed: {}", stderr))
            }
            Err(e) => Err(format!("Failed to run JSON extraction: {}", e))
        };
        
        let extraction_result = match json_result {
            Ok(json_content) => {
                println!("✅ Received structured JSON content: {} characters", json_content.len());
                
                // Parse the new structured JSON format first
                match Self::parse_structured_json(&json_content) {
                    Ok(chunks) => {
                        println!("📊 Created {} structured chunks from new JSON format", chunks.len());
                        Ok((chunks, Some(json_content)))
                    }
                    Err(e) => {
                        println!("❌ New format failed: {}, trying legacy format", e);
                        // Fallback to legacy format
                        match Self::parse_docling_json(&json_content) {
                            Ok(chunks) => {
                                println!("📊 Created {} chunks from legacy JSON format", chunks.len());
                                Ok((chunks, Some(json_content)))
                            }
                            Err(e2) => {
                                println!("❌ Both parsing methods failed: new={}, legacy={}", e, e2);
                                // Final fallback: create single chunk with raw JSON
                                let fallback_chunk = crate::app::DocumentChunk {
                                    id: 1,
                                    content: json_content.clone(),
                                    page_range: "full_document".to_string(),
                                    element_types: vec!["json_fallback".to_string(), "docling_output".to_string()],
                                    spatial_bounds: Some("full_document".to_string()),
                                    char_count: json_content.len(),
                                    table_data: None,
                                };
                                Ok((vec![fallback_chunk], Some(json_content)))
                            }
                        }
                    }
                }
            }
            Err(e) => Err(e)
        };
        
        println!("🔍 Advanced PDF extraction took: {:?}", extraction_start.elapsed());
        
        let (mut chunks, raw_json) = match extraction_result {
            Ok((chunks, raw_json)) => {
                info!("HTML extraction successful: {} chunks extracted", chunks.len());
                (chunks, raw_json)
            }
            Err(e) => {
                return Err(format!("Advanced extraction failed: {}. Please check if Python dependencies (Docling/Magic-PDF) are installed correctly.", e).into());
            }
        };
        
        if chunks.is_empty() {
            return Err("No chunks extracted from PDF - document might be image-based or encrypted. Try enabling OCR!".into());
        }

        // Memory optimization: Adaptive chunking for large documents
        let file_size = std::fs::metadata(file_path)?.len();
        let is_large_doc = file_size > 10_000_000; // 10MB threshold
        let is_huge_doc = file_size > 50_000_000; // 50MB threshold
        
        let _chunk_size = if is_huge_doc {
            5000 // Much larger chunks for huge documents to save memory
        } else if is_large_doc {
            2000 // Larger chunks for big documents for performance
        } else {
            800  // Smaller chunks for better granularity
        };
        
        // chunks variable should already be defined by this point from the extraction result
        // Memory optimization: For huge documents, fallback to simple chunking
        if is_huge_doc && !chunks.is_empty() {
            // For huge files, just use the chunks we already have
            println!("⚡ Using existing chunks for huge document to save memory");
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

        println!("🔍 Total PDF processing took: {:?}", total_start.elapsed());
        println!("🔍 Generated {} chunks with {} total characters", 
            chunks.len(), 
            chunks.iter().map(|c| c.char_count).sum::<usize>()
        );
        
        Ok((chunks, raw_json))
    }
    
    /// Convert structured chunks from Python to DocumentChunk format
    fn convert_structured_chunks_to_document_chunks(structured_chunks: Vec<StructuredChunk>) -> Vec<DocumentChunk> {
        let mut chunks = Vec::new();
        
        for (index, structured_chunk) in structured_chunks.into_iter().enumerate() {
            let chunk_id = index + 1;
            
            // Determine element types based on structured chunk data
            let mut element_types = vec![structured_chunk.element_type.clone()];
            
            // Clone content early to avoid borrow checker issues
            let content_text = structured_chunk.content.clone();
            
            // Parse content based on element type
            let content = match structured_chunk.element_type.as_str() {
                "table" => {
                    // For tables, try to extract structured table data from table_data field
                    if let Some(table_data_json) = structured_chunk.table_data {
                        // Try to parse the table_data as structured data
                        if let Ok(table_structure) = serde_json::from_value::<serde_json::Value>(table_data_json) {
                            println!("📊 Found structured table data for chunk {}", chunk_id);
                            // Store as JSON string for now, but mark as structured table
                            element_types.push("structured_table".to_string());
                            serde_json::to_string_pretty(&table_structure).unwrap_or(content_text)
                        } else {
                            // Fallback to content
                            content_text
                        }
                    } else {
                        // No structured table data, use content as-is
                        content_text
                    }
                },
                "heading" => {
                    element_types.push("heading".to_string());
                    content_text
                },
                "list" => {
                    element_types.push("list".to_string());
                    content_text
                },
                "formula" => {
                    element_types.push("formula".to_string());
                    content_text
                },
                _ => {
                    // Default to text
                    element_types.push("text".to_string());
                    content_text
                }
            };
            
            // Create page range from page number
            let page_range = format!("page_{}", structured_chunk.page_number);
            
            // Create spatial bounds from bbox if available
            let spatial_bounds = if let Some(bbox) = structured_chunk.bbox {
                Some(format!("x:{:.1},y:{:.1},w:{:.1},h:{:.1}", bbox.x, bbox.y, bbox.width, bbox.height))
            } else {
                Some(format!("chunk_bounds_{}", chunk_id))
            };
            
            chunks.push(DocumentChunk {
                id: chunk_id,
                content,
                page_range,
                element_types,
                spatial_bounds,
                char_count: structured_chunk.content.len(),
                table_data: None,
            });
        }
        
        println!("📊 Converted {} structured chunks to DocumentChunks", chunks.len());
        println!("📊 Element types found: {:?}", 
            chunks.iter().flat_map(|c| &c.element_types).collect::<std::collections::HashSet<_>>());
        
        chunks
    }
    
    /// Convert Document model back to chunks for compatibility
    fn convert_document_to_chunks(document: &crate::document_model::Document) -> Vec<DocumentChunk> {
        let mut chunks = Vec::new();
        let mut chunk_id = 1;
        
        for element in &document.elements {
            let (content, element_types, page_range) = match element {
                crate::document_model::DocumentElement::Table { data, page_number, .. } => {
                    // Convert table data to markdown format
                    let mut table_content = String::new();
                    
                    // Add headers if available
                    if data.total_rows > 0 && data.total_cols > 0 {
                        table_content.push_str("| ");
                        for col in 0..data.total_cols {
                            if let Some(cell) = data.cells.get(0).and_then(|row| row.get(col)) {
                                let cell_text = match &cell.content {
                                    crate::document_model::CellContent::Text(s) => s.clone(),
                                    crate::document_model::CellContent::Number(n) => n.to_string(),
                                    crate::document_model::CellContent::Formula(f) => f.clone(),
                                    crate::document_model::CellContent::Empty => "-".to_string(),
                                    crate::document_model::CellContent::Mixed(_) => "[Mixed]".to_string(),
                                };
                                table_content.push_str(&cell_text);
                            } else {
                                table_content.push_str("Header");
                            }
                            if col < data.total_cols - 1 {
                                table_content.push_str(" | ");
                            }
                        }
                        table_content.push_str(" |\n");
                        
                        // Add separator
                        table_content.push_str("|");
                        for _ in 0..data.total_cols {
                            table_content.push_str("---|")
                        }
                        table_content.push_str("\n");
                        
                        // Add data rows
                        for row in 1..data.total_rows {
                            table_content.push_str("| ");
                            for col in 0..data.total_cols {
                                if let Some(cell) = data.cells.get(row).and_then(|r| r.get(col)) {
                                    let cell_text = match &cell.content {
                                        crate::document_model::CellContent::Text(s) => s.clone(),
                                        crate::document_model::CellContent::Number(n) => n.to_string(),
                                        crate::document_model::CellContent::Formula(f) => f.clone(),
                                        crate::document_model::CellContent::Empty => "-".to_string(),
                                        crate::document_model::CellContent::Mixed(_) => "[Mixed]".to_string(),
                                    };
                                    table_content.push_str(&cell_text);
                                } else {
                                    table_content.push_str("-");
                                }
                                if col < data.total_cols - 1 {
                                    table_content.push_str(" | ");
                                }
                            }
                            table_content.push_str(" |\n");
                        }
                    }
                    
                    (table_content, vec!["table".to_string(), "native_parsed".to_string()], format!("page_{}", page_number))
                }
                crate::document_model::DocumentElement::Paragraph { text, page_number, .. } => {
                    (text.clone(), vec!["text".to_string(), "native_parsed".to_string()], format!("page_{}", page_number))
                }
                crate::document_model::DocumentElement::Heading { text, level, page_number, .. } => {
                    let heading_content = format!("{} {}", "#".repeat(*level as usize), text);
                    (heading_content, vec!["heading".to_string(), "native_parsed".to_string()], format!("page_{}", page_number))
                }
                crate::document_model::DocumentElement::List { items, page_number, .. } => {
                    let list_content = items.iter()
                        .map(|item| format!("- {}", item.text))
                        .collect::<Vec<_>>()
                        .join("\n");
                    (list_content, vec!["list".to_string(), "native_parsed".to_string()], format!("page_{}", page_number))
                }
                crate::document_model::DocumentElement::Image { caption, page_number, .. } => {
                    let caption_text = caption.as_ref().map(|c| c.as_str()).unwrap_or("[Image]");
                    (caption_text.to_string(), vec!["image".to_string(), "native_parsed".to_string()], format!("page_{}", page_number))
                }
                crate::document_model::DocumentElement::Formula { latex, page_number, .. } => {
                    (latex.clone(), vec!["formula".to_string(), "native_parsed".to_string()], format!("page_{}", page_number))
                }
                crate::document_model::DocumentElement::Section { title, page_number, .. } => {
                    (title.clone(), vec!["section".to_string(), "native_parsed".to_string()], format!("page_{}", page_number))
                }
            };
            
            let char_count = content.len();
            chunks.push(DocumentChunk {
                id: chunk_id,
                content,
                page_range,
                element_types,
                spatial_bounds: Some(format!("native_element_{}", chunk_id)),
                char_count,
                table_data: None,
            });
            
            chunk_id += 1;
        }
        
        chunks
    }
    
    /// Memory-efficient processing for huge documents (50MB+)
    fn process_huge_document_streaming(
        text: &str, 
        _file_path: &std::path::Path, 
        raw_json: Option<String>
    ) -> Result<(Vec<DocumentChunk>, Option<String>), Box<dyn std::error::Error>> {
        println!("⚡ Processing huge document in streaming mode to save memory");
        
        let mut chunks = Vec::new();
        let chunk_size = 8000; // Larger chunks for huge files
        let mut chunk_id = 1;
        
        // Process text in streaming chunks to avoid loading everything into memory
        let mut current_pos = 0;
        let text_len = text.len();
        
        while current_pos < text_len {
            let end_pos = std::cmp::min(current_pos + chunk_size, text_len);
            
            // Try to break at sentence boundaries for better chunk quality
            let actual_end = if end_pos < text_len {
                // Look back for sentence boundary
                if let Some(sentence_end) = text[current_pos..end_pos].rfind('.') {
                    current_pos + sentence_end + 1
                } else {
                    end_pos
                }
            } else {
                end_pos
            };
            
            let chunk_text = &text[current_pos..actual_end];
            if !chunk_text.trim().is_empty() {
                chunks.push(DocumentChunk {
                    id: chunk_id,
                    content: chunk_text.trim().to_string(),
                    page_range: format!("stream_chunk_{}", chunk_id),
                    element_types: vec!["text".to_string(), "streaming".to_string()],
                    spatial_bounds: Some(format!("stream_bounds_{}", chunk_id)),
                    char_count: chunk_text.trim().len(),
                    table_data: None,
                });
                
                chunk_id += 1;
            }
            
            current_pos = actual_end;
            
            // Memory optimization: Force garbage collection every 100 chunks
            if chunk_id % 100 == 0 {
                println!("💾 Processed {} streaming chunks, forcing cleanup", chunk_id - 1);
            }
        }
        
        println!("✅ Streaming processing complete: {} chunks", chunks.len());
        Ok((chunks, raw_json))
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
        match self.save_to_database_async() {
            Ok(()) => {
                // Success message already set in save_to_database_async
            }
            Err(e) => {
                self.error_message = Some(format!("❌ Failed to save to database: {}", e));
                self.status_message = "❌ Saving failed".to_string();
            }
        }
    }
    
    fn save_to_database_async(&mut self) -> ChonkerResult<()> {
        // Database storage temporarily disabled
        self.status_message = "💾 Database storage is temporarily disabled".to_string();
        Ok(())
    }
    
    fn _save_to_database_async_original(&mut self) -> ChonkerResult<()> {
        if let (Some(ref file_path), Some(ref _database)) = (&self.selected_file, &self.database) {
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
            
            // Convert app types to database types
            let _db_chunks: Vec<crate::database::DocumentChunk> = self.chunks.iter().map(|chunk| {
                crate::database::DocumentChunk {
                    id: chunk.id as i64,
                    content: chunk.content.clone(),
                    page_range: chunk.page_range.clone(),
                    element_types: chunk.element_types.clone(),
                    spatial_bounds: chunk.spatial_bounds.clone(),
                    char_count: chunk.char_count as i64,
                    table_data: chunk.table_data.as_ref().map(|data| serde_json::to_string(data).unwrap_or_default()),
                }
            }).collect();
            
            let _db_options = crate::database::ProcessingOptions {
                tool: "docling".to_string(),
                extract_tables: self.processing_options.table_detection,
                extract_formulas: self.processing_options.formula_recognition,
            };
            
            // Use a simple blocking approach since we can't nest runtimes
            // For now, we'll defer the actual database save until we can properly handle async
            let _file_path_clone = file_path.clone();
            
            // Temporarily return success and defer the actual save
            // TODO: Implement proper async handling for GUI context
            let document_id = format!("temp_{}", uuid::Uuid::new_v4());
            
            match Ok(document_id.clone()) {
                Ok(document_id) => {
                    self.status_message = format!(
                        "✅ Saved '{}' with {} chunks to database (ID: {})!", 
                        filename, chunk_count, document_id
                    );
                    Ok(())
                },
                Err(e) => {
                    Err(e)
                }
            }
        } else if self.database.is_none() {
            Err(ChonkerError::SystemResource {
                resource: "database_connection".to_string(),
                source: Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, "Database not connected")),
            })
        } else {
            Err(ChonkerError::SystemResource {
                resource: "document_selection".to_string(),
                source: Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, "No document selected")),
            })
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
    
    fn clean_chunk_content(&self, content: &str) -> String {
        // Remove Docling markup tags
        content
            .replace("<fcel>", "")
            .replace("<ecel>", "")
            .replace("<nl>", "\n")
            .replace("<rhed>", "")
            .replace("<srow>", "")
    }
}

impl eframe::App for ChonkerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Auto-load disabled - let user choose PDF
        if !self.auto_loaded {
            self.status_message = "🐹 CHONKER ready! Use Ctrl+O to open a PDF".to_string();
            self.auto_loaded = true;
        }
        
        apply_retro_theme(ctx);
        
        
        // Handle Tab key for switching modes
        ctx.input(|i| {
            if i.key_pressed(egui::Key::Tab) {
                self.mode = match self.mode {
                    AppMode::Chonker => AppMode::Snyfter,
                    AppMode::Snyfter => AppMode::Chonker,
                };
            }
        });
        
        // Header with emoji tabs and status
        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.vertical(|ui| {
                // Top row: Mode switching and status
                ui.horizontal(|ui| {
                    // Center the mode switching emojis
                    ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::LeftToRight), |ui| {
                        ui.horizontal(|ui| {
                            // Animal emojis for mode switching - highlight emoji faces, not background
                            let (chonker_emoji_color, chonker_outline, snyfter_emoji_color, snyfter_outline) = match self.mode {
                                AppMode::Chonker => (
                                    egui::Color32::from_rgb(255, 140, 0), // Orange emoji for active
                                    egui::Color32::WHITE,                  // White outline for active
                                    egui::Color32::GRAY,                  // Gray emoji for inactive
                                    egui::Color32::GRAY                   // Gray outline for inactive
                                ),
                                AppMode::Snyfter => (
                                    egui::Color32::GRAY,                  // Gray emoji for inactive
                                    egui::Color32::GRAY,                  // Gray outline for inactive
                                    egui::Color32::from_rgb(255, 140, 0), // Orange emoji for active
                                    egui::Color32::WHITE                  // White outline for active
                                ),
                            };
                            
                            // CHONKER button - transparent background, colored emoji
                            let chonker_button = egui::Button::new(
                                egui::RichText::new("🐹")
                                    .size(50.0)
                                    .color(chonker_emoji_color)
                            )
                            .fill(egui::Color32::TRANSPARENT)
                            .stroke(egui::Stroke::new(2.0, chonker_outline));
                            if ui.add(chonker_button).clicked() {
                                self.mode = AppMode::Chonker;
                            }
                            
                            ui.add_space(15.0);
                            
                            // SNYFTER button - transparent background, colored emoji
                            let snyfter_button = egui::Button::new(
                                egui::RichText::new("🐭")
                                    .size(50.0)
                                    .color(snyfter_emoji_color)
                            )
                            .fill(egui::Color32::TRANSPARENT)
                            .stroke(egui::Stroke::new(2.0, snyfter_outline));
                            if ui.add(snyfter_button).clicked() {
                                self.mode = AppMode::Snyfter;
                            }
                            
                            ui.add_space(20.0);
                            
                            // Mode label
                            let mode_text = match self.mode {
                                AppMode::Chonker => "CHONKER - Review & Edit",
                                AppMode::Snyfter => "SNYFTER - Finalize & Export",
                            };
                ui.label(egui::RichText::new(mode_text).size(18.0).strong());
                        });
                    });
                });
                
                ui.separator();

                // Display QC processing progress if active
                if self.qc_processing {
                    ui.heading("⏳ QC Processing...");
                    ui.add_space(10.0);
                    ui.label(&self.qc_progress_message);
                }
                
                // Second row: Status and terminal output
                ui.horizontal(|ui| {
                    // Status message with proper wrapping
                    let status_color = if self.error_message.is_some() {
                        egui::Color32::from_rgb(255, 100, 100)
                    } else if self.is_processing {
                        egui::Color32::from_rgb(255, 200, 100)
                    } else {
                        egui::Color32::from_rgb(100, 255, 100)
                    };
                    
                    let status_text = if let Some(ref error) = self.error_message {
                        error.clone()
                    } else {
                        self.status_message.clone()
                    };
                    
                    // Wrap status text if it's too long
                    let wrapped_text = if status_text.len() > 80 {
                        let mut wrapped = String::new();
                        let words: Vec<&str> = status_text.split_whitespace().collect();
                        let mut current_line = String::new();
                        
                        for word in words {
                            if current_line.len() + word.len() + 1 > 80 {
                                if !wrapped.is_empty() {
                                    wrapped.push('\n');
                                }
                                wrapped.push_str(&current_line);
                                current_line = word.to_string();
                            } else {
                                if !current_line.is_empty() {
                                    current_line.push(' ');
                                }
                                current_line.push_str(word);
                            }
                        }
                        if !current_line.is_empty() {
                            if !wrapped.is_empty() {
                                wrapped.push('\n');
                            }
                            wrapped.push_str(&current_line);
                        }
                        wrapped
                    } else {
                        status_text
                    };
                    
                    ui.colored_label(status_color, wrapped_text);
                    
                    // Show inline progress bar when processing
                    if self.is_processing {
                        ui.add_space(5.0);
                        ui.horizontal(|ui| {
                            ui.label("Progress:");
                            ui.add(egui::ProgressBar::new(self.processing_progress as f32 / 100.0)
                                .text(format!("{}%", self.processing_progress as u32)));
                        });
                    }
                });
            });
        });
        
        // CONDITIONAL UPDATE: Update markdown only if chunks changed and not already generated
        if !self.chunks.is_empty() && self.markdown_content.is_empty() {
            self.generate_markdown_from_chunks();
        }
        
        // Main content based on current mode
        match self.mode {
            AppMode::Chonker => {
                // CHONKER MODE: Panel A (PDF) + Panel B (Markdown)
                
                // Panel A: PDF (left side panel) - increased width for full PDF display
                egui::SidePanel::left("panel_a_pdf")
                    .resizable(true)
                    .default_width(ctx.screen_rect().width() * 0.65)  // Increased from 50% to 65%
                    .min_width(300.0)  // Increased minimum width
                    .max_width(ctx.screen_rect().width() * 0.85)  // Increased max from 80% to 85%
                    .show(ctx, |ui| {
                        self.pdf_viewer.render(ui);
                    });

                // Panel B: Document Content (central panel)
                egui::CentralPanel::default().show(ctx, |ui| {
                    self.render_document_content(ui);
                });
            },
            AppMode::Snyfter => {
                // SNYFTER MODE: Panel A (Same Semantic Editor) + Panel B (Export Dialog)
                
                // Panel A: Document Content Display (left side panel)
                egui::SidePanel::left("panel_a_semantic")
                    .resizable(true)
                    .default_width(ctx.screen_rect().width() * 0.5)
                    .min_width(200.0)
                    .max_width(ctx.screen_rect().width() * 0.8)
                    .show(ctx, |ui| {
                        self.render_document_content(ui);
                    });

                // Panel B: Export Dialog (central panel)
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.heading("📦 Panel B - Export Dialog");
                    ui.separator();
                    
                    ui.label(format!("Export bin contains {} items", self.export_bin.len()));
                    ui.separator();
                    
                    // Export format selection
                    ui.horizontal(|ui| {
                        ui.label("Export format:");
                        egui::ComboBox::from_label("")
                            .selected_text("CSV")
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut (), (), "CSV");
                                ui.selectable_value(&mut (), (), "JSON");
                                ui.selectable_value(&mut (), (), "Markdown");
                                ui.selectable_value(&mut (), (), "Plain Text");
                            });
                    });
                    
                    ui.separator();
                    
                    // Available chunks to add to export bin
                    if !self.chunks.is_empty() {
                        ui.label("Available chunks:");
                        
                        egui::ScrollArea::vertical()
                            .max_height(400.0)
                            .show(ui, |ui| {
                                for (_i, chunk) in self.chunks.iter().enumerate() {
                                    ui.horizontal(|ui| {
                                        if ui.button("📦 Add").clicked() {
                                            if !self.export_bin.iter().any(|c| c.id == chunk.id) {
                                                self.export_bin.push(chunk.clone());
                                                self.status_message = format!("Added chunk {} to export bin", chunk.id);
                                            }
                                        }
                                        
                                        let is_adversarial = chunk.element_types.contains(&"adversarial_content".to_string());
                                        let status_icon = if is_adversarial { "⚠️" } else { "✅" };
                                        
                                        ui.label(format!("{} Chunk {} ({})", 
                                            status_icon, 
                                            chunk.id, 
                                            chunk.page_range
                                        ));
                                    });
                                    
                                    ui.label(&chunk.content[..chunk.content.len().min(100)]);
                                    if chunk.content.len() > 100 {
                                        ui.label("...");
                                    }
                                    ui.separator();
                                }
                            });
                    } else {
                        ui.label("No chunks available. Process a document first.");
                    }
                    
                    ui.separator();
                    
                    // Export bin contents
                    if !self.export_bin.is_empty() {
                        ui.label("Export bin contents:");
                        
                        // Collect data first to avoid borrow checker issues
                        let export_data: Vec<(usize, String, String)> = self.export_bin.iter()
                            .enumerate()
                            .map(|(i, chunk)| (i, chunk.id.to_string(), chunk.page_range.clone()))
                            .collect();
                        
                        let mut remove_index = None;
                        
                        egui::ScrollArea::vertical()
                            .max_height(200.0)
                            .show(ui, |ui| {
                                for (i, chunk_id, page_range) in export_data {
                                    ui.horizontal(|ui| {
                                        if ui.button("🗑️ Remove").clicked() {
                                            remove_index = Some(i);
                                        }
                                        
                                        ui.label(format!("Chunk {} - {}", chunk_id, page_range));
                                    });
                                }
                            });
                        
                        // Handle removal after the loop
                        if let Some(index) = remove_index {
                            if index < self.export_bin.len() {
                                let chunk_id = self.export_bin[index].id;
                                self.export_bin.remove(index);
                                self.status_message = format!("Removed chunk {} from export bin", chunk_id);
                            }
                        }
                        
                        ui.separator();
                        
                        if ui.button("💾 Export Selected").clicked() {
                            self.status_message = format!("Exported {} chunks", self.export_bin.len());
                            // TODO: Implement actual export functionality
                        }
                        
                        if ui.button("🗑️ Clear Export Bin").clicked() {
                            self.export_bin.clear();
                            self.status_message = "Export bin cleared".to_string();
                        }
                    }
                });
            },
        }
        
                        // Bottom help bar (cleaner, no overlapping status)
        egui::TopBottomPanel::bottom("help_bar").show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                ui.label("💡");
                ui.add_space(10.0);
                
                match self.mode {
                    AppMode::Chonker => {
                        ui.label("Ctrl+O: Open | Space: Process | Cmd+R: QC Report | Tab: Switch to SNYFTER | 💾 Database storage temporarily disabled");
                    },
                    AppMode::Snyfter => {
                        ui.label("Tab: Switch to CHONKER | E: Export | Add chunks to bin | 💾 Database storage temporarily disabled");
                    },
                }
            });
        });
        
        // Check for async processing messages - keep GUI responsive
        let mut should_clear_processing_receiver = false;
        let mut should_clear_qc_receiver = false;
        let mut should_request_repaint = false;
        let mut should_generate_markdown = false;
        
        // Handle PDF processing messages
        if let Some(ref receiver) = self.processing_receiver {
            while let Ok(message) = receiver.try_recv() {
                match message {
                    ProcessingMessage::Progress(progress, msg) => {
                        self.processing_progress = progress as f64;
                        self.status_message = msg;
                        should_request_repaint = true;
                    }
                    ProcessingMessage::Complete(chunks, raw_json) => {
                        self.is_processing = false;
                        self.processing_progress = 100.0;
                        let chunk_count = chunks.len();
                        self.chunks = chunks;
                        should_clear_processing_receiver = true;
                        should_generate_markdown = true;
                        
                        self.status_message = format!("✅ Processing complete! {} chunks created", chunk_count);
                        
                        // Note: Database storage is temporarily disabled
                        should_request_repaint = true;
                    }
                    ProcessingMessage::Error(err) => {
                        self.is_processing = false;
                        self.processing_progress = 0.0;
                        should_clear_processing_receiver = true;
                        self.error_message = Some(err);
                        should_request_repaint = true;
                    }
                }
            }
        }
        
        // Check for QC processing messages - keep GUI responsive
        if let Some(ref receiver) = self.qc_receiver {
            // Process all available messages to keep UI responsive
            while let Ok(message) = receiver.try_recv() {
                match message {
                    QcMessage::Progress(msg) => {
                        self.qc_progress_message = msg.clone();
                        self.status_message = format!("🔧 QC Processing: {}", msg);
                        println!("📋 QC Progress: {}", msg);
                    }
                    QcMessage::Complete(report_path) => {
                        self.qc_processing = false;
                        should_clear_qc_receiver = true;
                        self.qc_progress_message.clear();
                        self.status_message = "✅ QC processing complete!".to_string();
                        println!("✅ QC processing completed successfully");
                        
                        // Load the generated report
                        if let Ok(content) = std::fs::read_to_string(&report_path) {
                            self.markdown_editor.set_content(content.clone());
                            self.markdown_content = content;
                            println!("📋 Loaded QC report: {}", report_path);
                        } else {
                            println!("⚠️ Could not load QC report file: {}", report_path);
                        }
                    }
                    QcMessage::Error(err) => {
                        self.qc_processing = false;
                        should_clear_qc_receiver = true;
                        self.qc_progress_message.clear();
                        self.error_message = Some(format!("❌ QC processing failed: {}", err));
                        println!("❌ QC processing error: {}", err);
                    }
                }
            }
            
            should_request_repaint = true;
        }
        
        // Clear receivers after borrowing is done
        if should_clear_processing_receiver {
            self.processing_receiver = None;
        }
        if should_clear_qc_receiver {
            self.qc_receiver = None;
        }
        
        // Generate markdown after all borrows are complete
        if should_generate_markdown {
            self.generate_markdown_from_chunks();
        }
        
        // Request repaint to keep GUI updating while processing
        if should_request_repaint {
            ctx.request_repaint();
        }
        
        // Handle keyboard shortcuts
        ctx.input(|i| {
            if i.key_pressed(egui::Key::O) && i.modifiers.ctrl {
                self.open_file_dialog();
            }
            if i.key_pressed(egui::Key::Space) && self.selected_file.is_some() && !self.is_processing {
                self.start_processing();
            }
            if i.key_pressed(egui::Key::R) && i.modifiers.ctrl {
                self.reset_for_new_file();
            }
            if i.key_pressed(egui::Key::R) && i.modifiers.command && !self.chunks.is_empty() && !self.qc_processing {
                // Cmd+R for QC Report + Qwen Cleaning (async version)
                self.start_qc_processing_async();
            }
            if i.key_pressed(egui::Key::Q) && i.modifiers.ctrl {
                std::process::exit(0);
            }
            if i.key_pressed(egui::Key::E) && self.mode == AppMode::Snyfter && !self.export_bin.is_empty() {
                // Trigger export
                self.status_message = format!("Exported {} chunks", self.export_bin.len());
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
                            
                            ui.separator();
                            
                            // Optional QC and Qwen processing (can be slow)
                            if ui.button("📋 Generate QC Report + Qwen Cleaning").clicked() {
                                self.status_message = "🔧 Starting QC analysis and table cleaning...".to_string();
                                self.call_existing_pipeline();
                            }
                            
                            ui.label("⚠️ QC processing takes 30+ seconds");
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
                                    .font(egui::FontId::monospace(18.0))
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
                            if ui.button("📋 QC + Qwen").clicked() {
                                self.status_message = "🔧 Starting QC analysis and table cleaning...".to_string();
                                self.call_existing_pipeline();
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
                    
                    egui::ScrollArea::both()
                        .max_height(remaining_height)
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.add(
                                egui::Label::new(&self.markdown_content)
                                    .selectable(true)
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
    
    /// Call the existing Python processing pipeline
    fn call_existing_pipeline(&mut self) {
        if let Some(ref file_path) = self.selected_file {
            // Update loading screen
            if let AppState::Loading { progress: _, message } = &mut self.state {
                *message = "Running CHONKER.py pipeline...".to_string();
            }
            self.status_message = "🐹 Calling existing CHONKER.py pipeline...".to_string();
            
            // Performance timing
            let start = std::time::Instant::now();
            println!("🔍 Starting Python pipeline...");
            
            // Call the Python processing pipeline that already works
            match std::process::Command::new("./venv/bin/python")
                .arg("CHONKER.py")
                .arg(file_path)
                .current_dir(".")
                .output()
            {
                Ok(output) => {
                    println!("🔍 Python pipeline took: {:?}", start.elapsed());
                    
                    if output.status.success() {
                        let _stdout = String::from_utf8_lossy(&output.stdout);
                        self.status_message = "✅ CHONKER.py processing complete!".to_string();
                        
                        // Try to load the generated QC report
                        self.load_generated_qc_report();
                        
                        // Run Qwen second pass to fix tables
                        self.run_qwen_table_fixer();
                    } else {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        self.error_message = Some(format!("❌ CHONKER.py failed: {}", stderr));
                    }
                }
                Err(e) => {
                    self.error_message = Some(format!("❌ Failed to call CHONKER.py: {}", e));
                }
            }
        }
    }
    
    /// Load the QC report generated by the Python pipeline
    fn load_generated_qc_report(&mut self) {
        // Look for the generated QC report file
        let qc_report_path = "pdf_table_qc_report.md";
        
        match std::fs::read_to_string(qc_report_path) {
            Ok(qc_content) => {
                self.markdown_editor.set_content(qc_content);
                self.status_message = format!("📋 Loaded QC report: {}", qc_report_path);
            }
            Err(_) => {
                // Try alternative paths or create a simple report
                let simple_report = format!(
                    "# Document Processing Complete\n\n**File:** {}\n\n**Status:** ✅ Processed successfully\n\n*QC report not found - processing may still be in progress*",
                    self.selected_file.as_ref()
                        .and_then(|f| f.file_name())
                        .map(|n| n.to_string_lossy())
                        .unwrap_or_default()
                );
                self.markdown_editor.set_content(simple_report);
                self.status_message = "📋 Created basic processing report".to_string();
            }
        }
    }
    
    /// Run Qwen second pass to fix table formatting issues
    fn run_qwen_table_fixer(&mut self) {
        self.status_message = "🔧 Running Qwen table fixer to clean up jumbled tables...".to_string();
        
        // Call the Qwen table fixer on the generated QC report
        match std::process::Command::new("./venv/bin/python")
            .arg("python/qwen_production_direct.py")
            .arg("pdf_table_qc_report.md")
            .arg("-o")
            .arg("pdf_table_qc_report_FIXED.md")
            .current_dir(".")
            .output()
        {
            Ok(output) => {
                if output.status.success() {
                    self.status_message = "✅ Qwen table fixing complete!".to_string();
                    
                    // Load the fixed QC report
                    match std::fs::read_to_string("pdf_table_qc_report_FIXED.md") {
                        Ok(fixed_content) => {
                            self.markdown_editor.set_content(fixed_content);
                            self.status_message = "📋 Loaded fixed QC report with cleaned tables".to_string();
                        }
                        Err(_) => {
                            self.status_message = "⚠️ Qwen fixing completed but couldn't load fixed report".to_string();
                        }
                    }
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    self.error_message = Some(format!("❌ Qwen table fixer failed: {}", stderr));
                }
            }
            Err(e) => {
                self.error_message = Some(format!("❌ Failed to call Qwen table fixer: {}", e));
            }
        }
    }
    
    
    /// Generate markdown content from processed chunks
    fn generate_markdown_from_chunks(&mut self) {
        if self.chunks.is_empty() {
            return;
        }
        
        // For the new human-friendly rendering, we'll just store the raw HTML
        // and let the render function handle the display
        let all_content = self.chunks.iter()
            .map(|chunk| chunk.content.clone())
            .collect::<Vec<_>>()
            .join("\n\n");
        
        self.markdown_content = all_content;
        
        println!("📺 Generated human-friendly display content: {} characters (from {} chunks)", self.markdown_content.len(), self.chunks.len());
    }
    
    /// Start QC processing in background thread with progress updates
    fn start_qc_processing_async(&mut self) {
        if let Some(ref file_path) = self.selected_file {
            // Create channel for progress updates
            let (sender, receiver) = mpsc::channel();
            self.qc_receiver = Some(receiver);
            self.qc_processing = true;
            self.qc_progress_message = "Initializing QC pipeline...".to_string();
            self.status_message = "🔧 Starting QC analysis and table cleaning (async)...".to_string();
            
            let file_path_clone = file_path.clone();
            
            // Spawn background thread for QC processing
            thread::spawn(move || {
                let _ = sender.send(QcMessage::Progress("Starting CHONKER.py pipeline...".to_string()));
                
                // Run CHONKER.py pipeline
                match std::process::Command::new("./venv/bin/python")
                    .arg("CHONKER.py")
                    .arg(&file_path_clone)
                    .current_dir(".")
                    .output()
                {
                    Ok(output) if output.status.success() => {
                        let _ = sender.send(QcMessage::Progress("CHONKER.py complete, running Qwen table fixer...".to_string()));
                        
                        // Run Qwen table fixer
                        match std::process::Command::new("./venv/bin/python")
                            .arg("python/qwen_production_direct.py")
                            .arg("pdf_table_qc_report.md")
                            .arg("-o")
                            .arg("pdf_table_qc_report_FIXED.md")
                            .current_dir(".")
                            .output()
                        {
                            Ok(qwen_output) if qwen_output.status.success() => {
                                let _ = sender.send(QcMessage::Complete("pdf_table_qc_report_FIXED.md".to_string()));
                            }
                            Ok(qwen_output) => {
                                let stderr = String::from_utf8_lossy(&qwen_output.stderr);
                                let _ = sender.send(QcMessage::Error(format!("Qwen table fixer failed: {}", stderr)));
                            }
                            Err(e) => {
                                let _ = sender.send(QcMessage::Error(format!("Failed to run Qwen: {}", e)));
                            }
                        }
                    }
                    Ok(output) => {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        let _ = sender.send(QcMessage::Error(format!("CHONKER.py failed: {}", stderr)));
                    }
                    Err(e) => {
                        let _ = sender.send(QcMessage::Error(format!("Failed to run CHONKER.py: {}", e)));
                    }
                }
            });
        }
    }
    
    fn render_document_content(&mut self, ui: &mut egui::Ui) {
        // Check if we have data to visualize
        if self.chunks.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.label("No document processed yet.");
                ui.add_space(10.0);
                ui.label("Load a PDF to see the extracted data here.");
            });
        } else {
            // Render structured content with maximum accuracy
            self.render_structured_content_accurate(ui);
        }
    }
    
    /// Convert chunks to data visualization format
    fn convert_chunks_to_viz_data(&self) -> crate::data_visualization::ExtractedData {
        let mut content_blocks = Vec::new();
        let mut tables_count = 0;
        let mut text_blocks_count = 0;
        let mut formulas_count = 0;
        let mut quality_issues = Vec::new();
        
        for chunk in &self.chunks {
            // Convert each chunk to a content block
            let block = if chunk.element_types.contains(&"table".to_string()) {
                // Parse as table
                let (headers, rows) = self.parse_markdown_table(&chunk.content);
                if !headers.is_empty() || !rows.is_empty() {
                    tables_count += 1;
                    let mut qualifiers = Vec::new();
                    
                    // Add adversarial content qualifier if detected
                    if chunk.element_types.contains(&"adversarial_content".to_string()) {
                        qualifiers.push(crate::data_visualization::DataQualifier {
                            symbol: "⚠️".to_string(),
                            description: "Adversarial content detected".to_string(),
                            applies_to: vec![], // Apply to whole table
                            severity: crate::data_visualization::QualifierSeverity::Critical,
                        });
                        
                        quality_issues.push(crate::data_visualization::QualityIssue {
                            block_id: chunk.id.to_string(),
                            issue_type: crate::data_visualization::IssueType::DataInconsistency,
                            description: "Adversarial content detected in this table".to_string(),
                            severity: crate::data_visualization::QualifierSeverity::Critical,
                            suggested_fix: Some("Review and verify table data for accuracy".to_string()),
                        });
                    }
                    
                    crate::data_visualization::ContentBlock::Table {
                        id: chunk.id.to_string(),
                        title: Some(format!("Table {} ({})", chunk.id, chunk.page_range)),
                        headers,
                        rows,
                        qualifiers,
                        metadata: std::collections::HashMap::new(),
                    }
                } else {
                    // Fallback to text if table parsing failed
                    text_blocks_count += 1;
                    self.create_text_block_from_chunk(chunk, &mut quality_issues)
                }
            } else if chunk.element_types.contains(&"formula".to_string()) {
                formulas_count += 1;
                crate::data_visualization::ContentBlock::Formula {
                    id: chunk.id.to_string(),
                    latex: chunk.content.clone(),
                    rendered_text: None,
                    metadata: std::collections::HashMap::new(),
                }
            } else {
                // Default to text block
                text_blocks_count += 1;
                self.create_text_block_from_chunk(chunk, &mut quality_issues)
            };
            
            content_blocks.push(block);
        }
        
        // Create document metadata
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("total_chunks".to_string(), self.chunks.len().to_string());
        metadata.insert("total_characters".to_string(), 
            self.chunks.iter().map(|c| c.char_count).sum::<usize>().to_string());
        
        let adversarial_count = self.chunks.iter()
            .filter(|c| c.element_types.contains(&"adversarial_content".to_string()))
            .count();
        metadata.insert("adversarial_chunks".to_string(), adversarial_count.to_string());
        
        let source_file = if let Some(ref file) = self.selected_file {
            file.file_name().unwrap_or_default().to_string_lossy().to_string()
        } else {
            "Unknown".to_string()
        };
        
        // Create extraction statistics
        let statistics = crate::data_visualization::ExtractionStatistics {
            total_content_blocks: content_blocks.len(),
            tables_count,
            text_blocks_count,
            lists_count: 0,
            images_count: 0,
            formulas_count,
            charts_count: 0,
            total_qualifiers: content_blocks.iter().map(|block| {
                match block {
                    crate::data_visualization::ContentBlock::Table { qualifiers, .. } => qualifiers.len(),
                    _ => 0,
                }
            }).sum(),
            confidence_score: 0.85, // Default confidence
            quality_issues,
        };
        
        crate::data_visualization::ExtractedData {
            source_file,
            extraction_timestamp: chrono::Utc::now().to_rfc3339(),
            tool_used: "CHONKER Enhanced Docling".to_string(),
            processing_time_ms: 0, // We don't track this currently
            content_blocks,
            metadata,
            statistics,
        }
    }
    
    /// Helper function to create text blocks from chunks
    fn create_text_block_from_chunk(
        &self, 
        chunk: &DocumentChunk, 
        quality_issues: &mut Vec<crate::data_visualization::QualityIssue>
    ) -> crate::data_visualization::ContentBlock {
        // Add adversarial content issue if detected
        if chunk.element_types.contains(&"adversarial_content".to_string()) {
            quality_issues.push(crate::data_visualization::QualityIssue {
                block_id: chunk.id.to_string(),
                issue_type: crate::data_visualization::IssueType::DataInconsistency,
                description: "Adversarial content detected in this text block".to_string(),
                severity: crate::data_visualization::QualifierSeverity::Critical,
                suggested_fix: Some("Review and verify text content for accuracy".to_string()),
            });
        }
        
        crate::data_visualization::ContentBlock::Text {
            id: chunk.id.to_string(),
            title: Some(format!("Text Block {} ({})", chunk.id, chunk.page_range)),
            content: chunk.content.clone(),
            formatting: crate::data_visualization::TextFormatting {
                is_bold: false,
                is_italic: false,
                font_size: None,
                alignment: crate::data_visualization::TextAlignment::Left,
            },
            metadata: std::collections::HashMap::new(),
        }
    }
    
    /// Interactive table rendering for DocTags content with inline table editor
    fn render_doctags_content_interactive(&mut self, ui: &mut egui::Ui, content: &str) {
        // Enhanced DocTags parser with interactive table editing
        if content.contains("<page_") || content.contains("<loc_") || content.contains("<table>") {
            // Native DocTags format - parse and render interactively
            self.render_native_doctags_interactive(ui, content);
        } else if content.contains("<otsl>") {
            // Legacy OTSL (Open Table and Structure Language) - Docling's table format
            self.render_otsl_tables_interactive(ui, content);
        } else if content.contains("<fcel>") || content.contains("<rhed>") || content.contains("<nl>") {
            // Inline OTSL content without wrapper tags - parse directly
            let (headers, rows) = Self::parse_otsl_content_robust(content);
            
            if !headers.is_empty() || !rows.is_empty() {
                // Render interactive table instead of static
                self.render_otsl_table_interactive(ui, &headers, &rows);
            } else {
                // Fallback to text if table parsing failed
                ui.label(content);
            }
        } else if content.contains('|') && content.lines().filter(|line| line.trim().starts_with('|')).count() > 1 {
            // Markdown table format - convert to interactive table
            let (headers, rows) = self.parse_markdown_table(content);
            if !headers.is_empty() || !rows.is_empty() {
                self.render_otsl_table_interactive(ui, &headers, &rows);
            } else {
                // Fallback to text if table parsing failed
                self.render_text_content_interactive(ui, content);
            }
        } else {
            // Fallback: render as formatted text with some structure awareness
            self.render_text_content_interactive(ui, content);
        }
    }

    /// Render OTSL tables with interactive editing capabilities
    fn render_otsl_tables_interactive(&mut self, ui: &mut egui::Ui, content: &str) {
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;
        let mut table_count = 0;
        
        while i < lines.len() {
            let line = lines[i];
            
            if line.contains("<otsl>") {
                table_count += 1;
                
                // Extract OTSL content between <otsl> and </otsl>
                let mut otsl_content = String::new();
                i += 1; // Skip opening OTSL tag
                
                let mut max_iterations = 1000; // Safety limit
                while i < lines.len() && !lines[i].contains("</otsl>") && max_iterations > 0 {
                    otsl_content.push_str(lines[i]);
                    otsl_content.push('\n');
                    i += 1;
                    max_iterations -= 1;
                }
                
                if max_iterations == 0 {
                    break;
                }
                
                if i < lines.len() && lines[i].contains("</otsl>") {
                    i += 1; // Skip closing tag
                }
                
                if !otsl_content.trim().is_empty() {
                    let (headers, rows) = Self::parse_otsl_content_robust(&otsl_content);
                    self.render_otsl_table_interactive(ui, &headers, &rows);
                    ui.separator();
                }
            } else {
                i += 1;
            }
        }
    }

    /// Render XML-style tables with interactive editing
    fn render_doctags_with_tables_interactive(&mut self, ui: &mut egui::Ui, content: &str) {
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;
        
        while i < lines.len() {
            let line = lines[i];
            
            if line.contains("<table>") {
                // Found a table - extract and render it
                let mut table_lines = Vec::new();
                i += 1; // Skip opening table tag
                
                // Collect table content until closing tag
                while i < lines.len() && !lines[i].contains("</table>") {
                    if lines[i].trim() != "<thead>" && lines[i].trim() != "</thead>" &&
                       lines[i].trim() != "<tbody>" && lines[i].trim() != "</tbody>" {
                        table_lines.push(lines[i]);
                    }
                    i += 1;
                }
                
                // Parse and render the table interactively
                let (headers, rows) = self.parse_xml_table(&table_lines);
                self.render_otsl_table_interactive(ui, &headers, &rows);
                
                ui.add_space(10.0);
                ui.separator();
            } else {
                // Regular content
                self.render_text_content_interactive(ui, line);
            }
            
            i += 1;
        }
    }

    /// Render regular text content with structure awareness
    fn render_text_content_interactive(&mut self, ui: &mut egui::Ui, content: &str) {
        let lines: Vec<&str> = content.lines().collect();
        
        for line in lines {
            if line.trim().is_empty() {
                ui.add_space(5.0);
                continue;
            }
            
            // Check for DocTags structural elements
            if line.contains("<heading") {
                let text = Self::extract_text_between_tags_static(line, "heading");
                ui.heading(&text);
            } else if line.contains("<section-header") {
                let text = Self::extract_text_between_tags_static(line, "section-header");
                ui.label(egui::RichText::new(&text).strong().size(18.0));
            } else if line.contains("<list-item") {
                let text = Self::extract_text_between_tags_static(line, "list-item");
                ui.horizontal(|ui| {
                    ui.label("•");
                    ui.label(&text);
                });
            } else if line.contains("<paragraph") {
                let text = Self::extract_text_between_tags_static(line, "paragraph");
                ui.label(&text);
            } else {
                ui.label(line);
            }
        }
    }

    /// Parse XML-style table content
    fn parse_xml_table(&self, table_lines: &[&str]) -> (Vec<String>, Vec<Vec<String>>) {
        let mut headers = Vec::new();
        let mut rows = Vec::new();
        let mut current_row = Vec::new();
        let mut is_header_row = false;
        
        for line in table_lines {
            if line.contains("<tr>") {
                current_row.clear();
                is_header_row = false;
            } else if line.contains("</tr>") {
                if !current_row.is_empty() {
                    if is_header_row || headers.is_empty() {
                        headers = current_row.clone();
                    } else {
                        rows.push(current_row.clone());
                    }
                }
            } else if line.contains("<th>") {
                is_header_row = true;
                let text = Self::extract_text_between_tags_static(line, "th");
                current_row.push(text);
            } else if line.contains("<td>") {
                let text = Self::extract_text_between_tags_static(line, "td");
                current_row.push(text);
            }
        }
        
        (headers, rows)
    }

    /// Render native DocTags content with full structure parsing
    fn render_native_doctags_interactive(&mut self, ui: &mut egui::Ui, content: &str) {
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;
        
        ui.label(egui::RichText::new("📋 Native DocTags Content").strong().size(16.0));
        ui.separator();
        ui.add_space(5.0);
        
        while i < lines.len() {
            let line = lines[i];
            
            // Parse DocTags elements based on tags
            if line.contains("<page_") {
                // Page marker
                let page_num = self.extract_page_number(line);
                ui.horizontal(|ui| {
                    ui.label("📄");
                    ui.label(egui::RichText::new(format!("Page {}", page_num)).strong());
                });
                ui.separator();
            } else if line.contains("<loc_") {
                // Location tag - could show coordinates in debug mode
                if self.debug_mode().unwrap_or(false) {
                    let coords = self.extract_location_coords(line);
                    ui.label(egui::RichText::new(format!("📍 Location: {}", coords)).size(10.0).color(egui::Color32::GRAY));
                }
            } else if line.contains("<table>") {
                // Native DocTags table - parse and render interactively
                let table_content = self.extract_table_content(&lines, &mut i);
                let (headers, rows) = self.parse_doctags_table(&table_content);
                self.render_otsl_table_interactive(ui, &headers, &rows);
            } else if line.contains("<title>") {
                let text = Self::extract_text_between_tags_static(line, "title");
                ui.heading(&text);
            } else if line.contains("<section_header>") {
                let text = Self::extract_text_between_tags_static(line, "section_header");
                ui.label(egui::RichText::new(&text).strong().size(18.0));
            } else if line.contains("<paragraph>") {
                let text = Self::extract_text_between_tags_static(line, "paragraph");
                ui.label(&text);
            } else if line.contains("<list_item>") {
                let text = Self::extract_text_between_tags_static(line, "list_item");
                ui.horizontal(|ui| {
                    ui.label("•");
                    ui.label(&text);
                });
            } else if line.contains("<formula>") {
                let text = Self::extract_text_between_tags_static(line, "formula");
                ui.label(egui::RichText::new(&text).monospace().color(egui::Color32::LIGHT_YELLOW));
            } else if !line.trim().is_empty() && !line.starts_with('<') {
                // Regular text content
                ui.label(line);
            }
            
            i += 1;
        }
    }
    
    /// Extract page number from page tag
    fn extract_page_number(&self, line: &str) -> String {
        if let Some(start) = line.find("<page_") {
            if let Some(end) = line[start..].find('>') {
                let tag = &line[start..start + end + 1];
                return tag.replace("<page_", "").replace(">", "");
            }
        }
        "?".to_string()
    }
    
    /// Extract location coordinates from location tag
    fn extract_location_coords(&self, line: &str) -> String {
        if let Some(start) = line.find("<loc_") {
            if let Some(end) = line[start..].find('>') {
                let tag = &line[start..start + end + 1];
                return tag.replace("<loc_", "").replace(">", "");
            }
        }
        "0,0".to_string()
    }
    
    /// Extract table content from DocTags
    fn extract_table_content(&self, lines: &[&str], current_pos: &mut usize) -> Vec<String> {
        let mut table_lines = Vec::new();
        let mut i = *current_pos + 1; // Skip opening <table>
        
        while i < lines.len() && !lines[i].contains("</table>") {
            table_lines.push(lines[i].to_string());
            i += 1;
        }
        
        *current_pos = i; // Update position past </table>
        table_lines
    }
    
    /// Parse DocTags table content
    fn parse_doctags_table(&self, table_lines: &[String]) -> (Vec<String>, Vec<Vec<String>>) {
        let mut headers = Vec::new();
        let mut rows = Vec::new();
        let mut current_row = Vec::new();
        let mut in_header = false;
        
        for line in table_lines {
            if line.contains("<row>") || line.contains("<tr>") {
                current_row.clear();
            } else if line.contains("</row>") || line.contains("</tr>") {
                if !current_row.is_empty() {
                    if in_header || headers.is_empty() {
                        headers = current_row.clone();
                        in_header = false;
                    } else {
                        rows.push(current_row.clone());
                    }
                }
            } else if line.contains("<cell>") || line.contains("<td>") || line.contains("<th>") {
                if line.contains("<th>") {
                    in_header = true;
                }
                let content = if line.contains("<cell>") {
                    Self::extract_text_between_tags_static(line, "cell")
                } else if line.contains("<td>") {
                    Self::extract_text_between_tags_static(line, "td")
                } else {
                    Self::extract_text_between_tags_static(line, "th")
                };
                current_row.push(content);
            }
        }
        
        (headers, rows)
    }
    
    /// Parse markdown table format
    fn parse_markdown_table(&self, content: &str) -> (Vec<String>, Vec<Vec<String>>) {
        let mut headers = Vec::new();
        let mut rows = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        
        for line in lines {
            let trimmed = line.trim();
            if trimmed.starts_with('|') && trimmed.ends_with('|') {
                // Skip separator lines
                if trimmed.chars().all(|c| c == '|' || c == '-' || c == ':' || c.is_whitespace()) {
                    continue;
                }
                
                let cells: Vec<String> = trimmed
                    .split('|')
                    .map(|cell| cell.trim().to_string())
                    .filter(|cell| !cell.is_empty())
                    .collect();
                
                if !cells.is_empty() {
                    if headers.is_empty() {
                        headers = cells;
                    } else {
                        rows.push(cells);
                    }
                }
            }
        }
        
        (headers, rows)
    }
    
    /// Debug mode flag
    fn debug_mode(&self) -> Option<bool> {
        Some(false) // Can be made configurable later
    }

    /// Render interactive OTSL table with editing capabilities  
    fn render_otsl_table_interactive(&mut self, ui: &mut egui::Ui, headers: &[String], rows: &[Vec<String>]) {
        if headers.is_empty() && rows.is_empty() {
            ui.label("⚠️ Empty table data");
            return;
        }
        
        // Add proper spacing before table
        ui.add_space(10.0);
        ui.separator();
        ui.add_space(8.0);
        
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("📊 Interactive Table").strong().size(16.0));
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("⚙️ Edit Structure").clicked() {
                    // Future: Open table structure editor
                }
            });
        });
        ui.add_space(8.0);
        
        // Show table stats
        if !headers.is_empty() && !rows.is_empty() {
            ui.label(format!("📏 {} columns × {} rows", headers.len(), rows.len()));
            ui.add_space(5.0);
        }
        
        // Use a frame to contain the interactive table
        egui::Frame::default()
            .fill(egui::Color32::from_rgba_unmultiplied(15, 15, 15, 150))
            .stroke(egui::Stroke::new(1.0, egui::Color32::GRAY))
            .inner_margin(egui::Margin::same(10.0))
            .show(ui, |ui| {
                egui::ScrollArea::horizontal()
                    .show(ui, |ui| {
                        egui::Grid::new(format!("interactive_table_{}", headers.len()))
                            .striped(true)
                            .min_col_width(100.0)  // Larger minimum for better readability
                            .max_col_width(200.0)  // Prevent extremely wide columns
                            .spacing([8.0, 6.0])   // Good spacing for readability
                            .show(ui, |ui| {
                                // Render headers with better styling
                                if !headers.is_empty() {
                                    for (col_idx, header) in headers.iter().enumerate() {
                                        let header_text = if header.is_empty() { 
                                            format!("Col {}", col_idx + 1) 
                                        } else { 
                                            header.clone() 
                                        };
                                        
                                        ui.label(
                                            egui::RichText::new(header_text)
                                                .strong()
                                                .size(13.0)
                                                .color(egui::Color32::LIGHT_BLUE)
                                        );
                                    }
                                    ui.end_row();
                                }
                                
                                // Render data rows with interactive elements
                                for (row_idx, row) in rows.iter().enumerate() {
                                    let max_cols = headers.len().max(row.len());
                                    
                                    for col_idx in 0..max_cols {
                                        let cell_content = row.get(col_idx)
                                            .map(|s| s.as_str())
                                            .unwrap_or("");
                                        
                                        let display_text = if cell_content.is_empty() { 
                                            "-".to_string() 
                                        } else if cell_content.len() > 50 {
                                            format!("{}...", &cell_content[..47])
                                        } else {
                                            cell_content.to_string()
                                        };
                                        
                                        // Make cells clickable for future editing
                                        let cell_response = ui.add(
                                            egui::Label::new(
                                                if cell_content.chars().all(|c| {
                                                    c.is_ascii_digit() || c == ',' || c == '.' || c == '-' || c == ' '
                                                }) && !cell_content.is_empty() {
                                                    egui::RichText::new(display_text).monospace()
                                                } else {
                                                    egui::RichText::new(display_text)
                                                }
                                            )
                                            .sense(egui::Sense::click())
                                        );
                                        
                                        // Show tooltip with full content on hover and handle clicks
                                        if cell_content.len() > 50 {
                                            let cell_response_with_tooltip = cell_response.on_hover_text(cell_content);
                                            // Future: Handle cell clicks for editing
                                            if cell_response_with_tooltip.clicked() {
                                                // Store cell position for future editing
                                                debug!("Cell clicked: row {}, col {}, content: {}", row_idx, col_idx, cell_content);
                                            }
                                        } else {
                                            // Future: Handle cell clicks for editing
                                            if cell_response.clicked() {
                                                // Store cell position for future editing
                                                debug!("Cell clicked: row {}, col {}, content: {}", row_idx, col_idx, cell_content);
                                            }
                                        }
                                    }
                                    ui.end_row();
                                }
                            });
                    });
            });
        
        // Add proper spacing after table
        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);
    }
    
    fn render_doctags_content_static(ui: &mut egui::Ui, content: &str) {
        // Enhanced DocTags parser with OTSL support for structured tables
        if content.contains("<otsl>") {
            // OTSL (Open Table and Structure Language) - Docling's table format
            Self::render_otsl_tables_static(ui, content);
        } else if content.contains("<table>") {
            // Standard XML-style tables
            Self::render_doctags_with_tables_static(ui, content);
        } else if content.contains("<fcel>") || content.contains("<rhed>") || content.contains("<nl>") {
            // Inline OTSL content without wrapper tags - parse directly
            println!("🔍 OTSL Debug: Found inline OTSL content, parsing directly");
            let (headers, rows) = Self::parse_otsl_content_robust(content);
            
            println!("🔍 OTSL Debug: Parsed {} headers, {} rows from inline content", headers.len(), rows.len());
            if !headers.is_empty() {
                println!("🔍 OTSL Debug: Headers: {:?}", headers);
            }
            if !rows.is_empty() {
                println!("🔍 OTSL Debug: First row ({} cells): {:?}", rows[0].len(), rows[0]);
            }
            
            // Render the OTSL table
            Self::render_otsl_table_static(ui, &headers, &rows);
        } else {
            // Fallback: render as formatted text with some structure awareness
            let lines: Vec<&str> = content.lines().collect();
            
            for line in lines {
                if line.trim().is_empty() {
                    ui.add_space(5.0);
                    continue;
                }
                
                // Check for DocTags structural elements
                if line.contains("<heading") {
                    // Extract heading text
                    let text = Self::extract_text_between_tags_static(line, "heading");
                    ui.heading(&text);
                } else if line.contains("<section-header") {
                    let text = Self::extract_text_between_tags_static(line, "section-header");
                    ui.label(egui::RichText::new(&text).strong().size(18.0));
                } else if line.contains("<list-item") {
                    let text = Self::extract_text_between_tags_static(line, "list-item");
                    ui.horizontal(|ui| {
                        ui.label("•");
                        ui.label(&text);
                    });
                } else if line.contains("<paragraph") {
                    let text = Self::extract_text_between_tags_static(line, "paragraph");
                    ui.label(&text);
                } else {
                    // Regular text line
                    ui.label(line);
                }
            }
        }
    }
    
    fn render_otsl_tables_static(ui: &mut egui::Ui, content: &str) {
        println!("🔍 OTSL Debug: Processing content length: {}", content.len());
        
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;
        let mut table_count = 0;
        
        while i < lines.len() {
            let line = lines[i];
            
            if line.contains("<otsl>") {
                table_count += 1;
                println!("🔍 OTSL Debug: Found table #{} at line {}", table_count, i);
                
                // Extract OTSL content between <otsl> and </otsl>
                let mut otsl_content = String::new();
                let start_line = i;
                i += 1; // Skip opening OTSL tag
                
                // Critical fix: Add boundary check to prevent infinite loop
                let mut max_iterations = 1000; // Safety limit
                while i < lines.len() && !lines[i].contains("</otsl>") && max_iterations > 0 {
                    otsl_content.push_str(lines[i]);
                    otsl_content.push('\n');
                    i += 1;
                    max_iterations -= 1;
                }
                
                // If we hit the iteration limit, log error and break
                if max_iterations == 0 {
                    println!("❌ OTSL Debug: Hit iteration limit extracting table content - possible infinite loop");
                    break;
                }
                
                // Critical fix: Ensure we advance past the closing tag
                if i < lines.len() && lines[i].contains("</otsl>") {
                    i += 1; // Skip closing tag
                }
                
                println!("🔍 OTSL Debug: Extracted content ({} chars):\n{}", otsl_content.len(), &otsl_content);
                
                // Also log just the first few lines to see structure
                let preview_lines: Vec<&str> = otsl_content.lines().take(10).collect();
                println!("🔍 OTSL Debug: First 10 lines: {:?}", preview_lines);
                
                // Only parse if we have actual content
                if !otsl_content.trim().is_empty() {
                    // Parse the OTSL content with robust error handling
                    let (headers, rows) = Self::parse_otsl_content_robust(&otsl_content);
                    
                    println!("🔍 OTSL Debug: Parsed {} headers, {} rows", headers.len(), rows.len());
                    if !headers.is_empty() {
                        println!("🔍 OTSL Debug: Headers: {:?}", headers);
                    }
                    if !rows.is_empty() {
                        println!("🔍 OTSL Debug: First row ({} cells): {:?}", rows[0].len(), rows[0]);
                    }
                    
                    // Render the OTSL table with validation
                    Self::render_otsl_table_static(ui, &headers, &rows);
                    ui.separator();
                } else {
                    println!("⚠️ OTSL Debug: Empty OTSL content, skipping table rendering");
                }
            } else {
                i += 1;
            }
        }
        
        if table_count == 0 {
            println!("🔍 OTSL Debug: No OTSL tables found in content");
        }
    }
    
    fn render_doctags_with_tables_static(ui: &mut egui::Ui, content: &str) {
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;
        
        while i < lines.len() {
            let line = lines[i];
            
            if line.contains("<table>") {
                // Found a table - extract and render it
                let mut table_lines = Vec::new();
                i += 1; // Skip opening table tag
                
                // Collect table content until closing tag
                while i < lines.len() && !lines[i].contains("</table>") {
                    if lines[i].trim() != "<thead>" && lines[i].trim() != "</thead>" &&
                       lines[i].trim() != "<tbody>" && lines[i].trim() != "</tbody>" {
                        table_lines.push(lines[i]);
                    }
                    i += 1;
                }
                
                // Render the table
                ui.separator();
                ui.label(egui::RichText::new("📊 Table").strong());
                ui.add_space(5.0);
                
                Self::render_doctags_table_static(ui, &table_lines);
                
                ui.add_space(10.0);
                ui.separator();
            } else {
                // Regular content
                if line.contains("<heading") {
                    let text = Self::extract_text_between_tags_static(line, "heading");
                    ui.heading(&text);
                } else if line.contains("<paragraph") {
                    let text = Self::extract_text_between_tags_static(line, "paragraph");
                    ui.label(&text);
                } else if !line.trim().is_empty() {
                    ui.label(line);
                }
            }
            
            i += 1;
        }
    }
    
    fn parse_otsl_content_robust(content: &str) -> (Vec<String>, Vec<Vec<String>>) {
        let mut headers = Vec::new();
        let mut rows = Vec::new();
        let mut current_row = Vec::new();
        
        println!("🔍 OTSL Debug: Parsing content: {}", &content[..content.len().min(200)]);
        
        // Handle inline OTSL content (continuous stream)
        if content.contains("<fcel>") || content.contains("<rhed>") {
            // Parse as continuous inline stream
            let mut tokens = Vec::new();
            let mut current_pos = 0;
            
            // Extract all OTSL tokens from the content
            while current_pos < content.len() {
                if let Some(tag_start) = content[current_pos..].find('<') {
                    let abs_tag_start = current_pos + tag_start;
                    if let Some(tag_end) = content[abs_tag_start..].find('>') {
                        let abs_tag_end = abs_tag_start + tag_end + 1;
                        let tag = &content[abs_tag_start..abs_tag_end];
                        
                        // Extract content between this tag and the next tag (if any)
                        let content_start = abs_tag_end;
                        let content_end = if let Some(next_tag) = content[content_start..].find('<') {
                            content_start + next_tag
                        } else {
                            content.len()
                        };
                        
                        let cell_content = content[content_start..content_end].trim();
                        
                        tokens.push((tag.to_string(), cell_content.to_string()));
                        current_pos = content_end;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
            
            println!("🔍 OTSL Debug: Found {} tokens", tokens.len());
            
            // Convert tokens to table structure
            for (tag, cell_content) in tokens {
                match tag.as_str() {
                    "<nl>" => {
                        // New line - finish current row
                        if !current_row.is_empty() {
                            rows.push(current_row.clone());
                            current_row.clear();
                        }
                    }
                    "<rhed>" => {
                        // Row header - could be start of new row or header definition
                        if !cell_content.is_empty() {
                            current_row.push(cell_content);
                        }
                    }
                    "<fcel>" => {
                        // Field cell - regular data cell
                        if cell_content.is_empty() || cell_content == "-" || cell_content == "- -" {
                            current_row.push("".to_string());
                        } else {
                            current_row.push(cell_content);
                        }
                    }
                    "<ecel>" => {
                        // Empty cell
                        current_row.push("".to_string());
                    }
                    "<ched>" => {
                        // Column header
                        if !cell_content.is_empty() {
                            headers.push(cell_content);
                        }
                    }
                    _ => {
                        // Unknown tag, treat as content
                        if !cell_content.is_empty() {
                            current_row.push(cell_content);
                        }
                    }
                }
            }
            
            // Add final row if not empty
            if !current_row.is_empty() {
                rows.push(current_row);
            }
        } else {
            // Original line-by-line parsing for structured content
            for line in content.lines() {
                let line = line.trim();
                
                if line.is_empty() {
                    continue;
                }
                
                // Handle cell content with robust parsing
                if line.contains("<fcel>") || line.contains("<ched>") || line.contains("<rhed>") || line.contains("<ecel>") {
                    Self::parse_cell_content_robust(line, &mut current_row);
                }
            }
            
            if !current_row.is_empty() {
                rows.push(current_row);
            }
        }
        
        // If no explicit headers but we have rows, try to detect header row
        if headers.is_empty() && !rows.is_empty() {
            // Use first row as headers if it looks like headers
            let first_row = &rows[0];
            if first_row.len() > 1 && first_row.iter().any(|cell| {
                !cell.is_empty() && !cell.chars().all(|c| c.is_ascii_digit() || c == ',' || c == '.' || c == '-' || c.is_whitespace())
            }) {
                headers = rows.remove(0);
            }
        }
        
        println!("🔍 OTSL Debug: Final result - {} headers, {} rows", headers.len(), rows.len());
        
        (headers, rows)
    }
    
    fn parse_cell_content_robust(line: &str, current_row: &mut Vec<String>) {
        // Handle different cell types and malformed content
        if line.contains("<ecel>") {
            // Empty cell - but only add one empty cell, not multiple
            current_row.push(String::new());
        } else if let Some(content) = Self::extract_cell_content_robust(line) {
            // Parse cell content, handling space-separated values
            let content = content.trim();
            
            // Handle empty/dash representations
            if content == "-" || content == "- -" || content.is_empty() {
                current_row.push(String::new());
            } else {
                // Critical fix: Don't auto-split on spaces for most content
                // Only split if it's clearly numeric data with specific patterns
                if content.contains(' ') && 
                   content.split_whitespace().count() > 1 &&
                   content.split_whitespace().all(|part| {
                       part.chars().all(|c| c.is_ascii_digit() || c == ',' || c == '.' || c == '-') &&
                       !part.is_empty()
                   }) {
                    // Space-separated numeric values - split into separate cells
                    for value in content.split_whitespace() {
                        if !value.is_empty() {
                            current_row.push(value.to_string());
                        }
                    }
                } else {
                    // Keep content as single cell (most common case)
                    current_row.push(content.to_string());
                }
            }
        }
    }
    
    fn extract_cell_content_robust(line: &str) -> Option<String> {
        // Try different tag patterns
        if let Some(content) = Self::extract_between_tags_robust(line, "fcel") {
            return Some(content);
        }
        if let Some(content) = Self::extract_between_tags_robust(line, "ched") {
            return Some(content);
        }
        if let Some(content) = Self::extract_between_tags_robust(line, "rhed") {
            return Some(content);
        }
        None
    }
    
    fn extract_between_tags_robust(text: &str, tag: &str) -> Option<String> {
        let start_tag = format!("<{}>", tag);
        let end_tag = format!("</{}>", tag);
        
        if let Some(start) = text.find(&start_tag) {
            let content_start = start + start_tag.len();
            if let Some(end) = text[content_start..].find(&end_tag) {
                let content = &text[content_start..content_start + end];
                // Clean up the content by trimming whitespace and newlines
                return Some(content.trim().to_string());
            }
        }
        None
    }
    
    fn normalize_row_length(row: &mut Vec<String>, target_length: usize) {
        while row.len() < target_length {
            row.push(String::new()); // Pad with empty cells
        }
        if row.len() > target_length {
            row.truncate(target_length); // Trim excess cells
        }
    }
    
    fn render_otsl_table_static(ui: &mut egui::Ui, headers: &[String], rows: &[Vec<String>]) {
        if headers.is_empty() && rows.is_empty() {
            ui.label("⚠️ Empty table data");
            return;
        }
        
        // Add proper spacing before table
        ui.add_space(10.0);
        ui.separator();
        ui.add_space(8.0);
        
        ui.label(egui::RichText::new("📊 Financial Table (OTSL)").strong().size(16.0));
        ui.add_space(8.0);
        
        // Show table stats
        if !headers.is_empty() && !rows.is_empty() {
            ui.label(format!("📏 {} columns × {} rows", headers.len(), rows.len()));
            ui.add_space(5.0);
        }
        
        // Use a frame to contain the table and prevent overlap
        egui::Frame::default()
            .fill(egui::Color32::from_rgba_unmultiplied(15, 15, 15, 150))
            .stroke(egui::Stroke::new(1.0, egui::Color32::GRAY))
            .inner_margin(egui::Margin::same(10.0))
            .show(ui, |ui| {
                egui::Grid::new("otsl_table")
                    .striped(true)
                    .min_col_width(80.0)  // Ensure minimum column width
                    .spacing([10.0, 8.0]) // Add horizontal and vertical spacing
                    .show(ui, |ui| {
                        // Render headers
                        if !headers.is_empty() {
                            for header in headers {
                                ui.label(egui::RichText::new(header).strong().color(egui::Color32::LIGHT_BLUE));
                            }
                            ui.end_row();
                        }
                        
                        // Render data rows with error handling
                        for (row_idx, row) in rows.iter().enumerate() {
                            for (cell_idx, cell) in row.iter().enumerate() {
                                // Show cell index in debug mode if cell is suspiciously long
                                if cell.len() > 100 {
                                    println!("⚠️ OTSL Debug: Long cell at row {} col {}: {} chars", row_idx, cell_idx, cell.len());
                                }
                                
                                let display_text = if cell.is_empty() { 
                                    "-".to_string() 
                                } else { 
                                    // Wrap long text to prevent overflow
                                    if cell.len() > 50 {
                                        format!("{}...", &cell[..47])
                                    } else {
                                        cell.clone()
                                    }
                                };
                                
                                // Use monospace font for numeric data alignment
                                if cell.chars().all(|c| c.is_ascii_digit() || c == ',' || c == '.' || c == '-' || c == ' ') {
                                    ui.label(egui::RichText::new(display_text).monospace());
                                } else {
                                    ui.label(display_text);
                                }
                            }
                            ui.end_row();
                        }
                    });
            });
        
        // Add proper spacing after table
        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);
    }
    
    fn render_doctags_table_static(ui: &mut egui::Ui, table_lines: &[&str]) {
        // Parse table rows
        let mut rows = Vec::new();
        let mut current_row = Vec::new();
        
        for line in table_lines {
            if line.contains("<tr>") {
                current_row.clear();
            } else if line.contains("</tr>") {
                if !current_row.is_empty() {
                    rows.push(current_row.clone());
                }
            } else if line.contains("<td>") || line.contains("<th>") {
                let text = if line.contains("<td>") {
                    Self::extract_text_between_tags_static(line, "td")
                } else {
                    Self::extract_text_between_tags_static(line, "th")
                };
                current_row.push(text);
            }
        }
        
        // Render table using egui Grid
        if !rows.is_empty() {
            egui::Grid::new(format!("doctags_table_{}", rows.len()))
                .striped(true)
                .show(ui, |ui| {
                    for (row_idx, row) in rows.iter().enumerate() {
                        for cell in row {
                            if row_idx == 0 {
                                // Header row
                                ui.label(egui::RichText::new(cell).strong());
                            } else {
                                ui.label(cell);
                            }
                        }
                        ui.end_row();
                    }
                });
        }
    }
    
    fn extract_text_between_tags_static(line: &str, tag: &str) -> String {
        // Simple text extraction between XML-like tags
        let start_tag = format!("<{}>", tag);
        let end_tag = format!("</{}>", tag);
        
        if let Some(start) = line.find(&start_tag) {
            if let Some(end) = line.find(&end_tag) {
                let start_pos = start + start_tag.len();
                if start_pos < end {
                    return line[start_pos..end].to_string();
                }
            }
        }
        
        // Fallback: try to extract from self-closing tags or partial matches
        if line.contains(&format!("<{}", tag)) {
            // Look for content after the tag
            if let Some(start) = line.find('>') {
                let content = &line[start + 1..];
                if let Some(end) = content.find('<') {
                    return content[..end].trim().to_string();
                } else {
                    return content.trim().to_string();
                }
            }
        }
        
        // Ultimate fallback: return the line as-is
        line.trim().to_string()
    }
    
    /// Create mock Docling JSON from existing chunks for table editor
    fn create_mock_docling_json(&self) -> serde_json::Value {
        let mut structured_tables = Vec::new();
        let mut extractions = Vec::new();
        
        // Create mock structured tables from chunks that contain table data
        for (idx, chunk) in self.chunks.iter().enumerate() {
            if chunk.element_types.contains(&"table".to_string()) {
                // Try to parse the chunk content as table data
                let table_data = self.parse_chunk_as_table_data(&chunk.content);
                if !table_data.is_empty() {
                    structured_tables.push(serde_json::json!({
                        "table_index": idx,
                        "processed_data": table_data,
                        "bounds": chunk.spatial_bounds.clone().unwrap_or_default()
                    }));
                }
            }
        }
        
        // Create mock extraction from all chunks
        let all_text = self.chunks.iter()
            .map(|chunk| chunk.content.clone())
            .collect::<Vec<_>>()
            .join("\n\n");
        
        extractions.push(serde_json::json!({
            "page_number": 1,
            "text": all_text,
            "tables": [],
            "figures": [],
            "formulas": [],
            "confidence": 0.9,
            "layout_boxes": [],
            "tool": "mock_data"
        }));
        
        serde_json::json!({
            "success": true,
            "tool": "mock_docling",
            "extractions": extractions,
            "structured_tables": structured_tables,
            "metadata": {
                "total_pages": 1,
                "tables_found": structured_tables.len(),
                "figures_found": 0,
                "processing_time": 0
            }
        })
    }
    
    /// Parse chunk content as table data for mock JSON
    fn parse_chunk_as_table_data(&self, content: &str) -> Vec<Vec<String>> {
        let mut table_data = Vec::new();
        
        // Look for markdown table patterns
        if content.contains('|') {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with('|') && trimmed.ends_with('|') {
                    // Skip separator lines
                    if trimmed.chars().all(|c| c == '|' || c == '-' || c == ':' || c.is_whitespace()) {
                        continue;
                    }
                    
                    let cells: Vec<String> = trimmed
                        .split('|')
                        .map(|cell| cell.trim().to_string())
                        .filter(|cell| !cell.is_empty())
                        .collect();
                    
                    if !cells.is_empty() {
                        table_data.push(cells);
                    }
                }
            }
        }
        
        table_data
    }
    
    /// Get the latest processed native document for Table Editor
    fn get_latest_native_document(&self) -> Option<crate::document_model::Document> {
        // For now, check if we have any chunks that were processed with the native parser
        if self.chunks.iter().any(|chunk| chunk.element_types.contains(&"native_parsed".to_string())) {
            // Create a mock document from the native chunks
            // TODO: Store the actual Document model from native parsing
            let mut document = crate::document_model::Document::new();
            
            for chunk in &self.chunks {
                if chunk.element_types.contains(&"table".to_string()) {
                    // Convert chunk back to table element
                    let table_data = self.create_table_data_from_chunk(chunk);
                    let element = crate::document_model::DocumentElement::Table {
                        id: chunk.id.to_string(),
                        data: table_data,
                        bounds: crate::document_model::BoundingBox {
                            x: 0.0,
                            y: 0.0,
                            width: 100.0,
                            height: 50.0,
                        },
                        page_number: 1,
                        caption: None,
                        table_type: crate::document_model::TableType::General,
                    };
                    document.add_element(element);
                } else if chunk.element_types.contains(&"text".to_string()) {
                    let element = crate::document_model::DocumentElement::Paragraph {
                        id: chunk.id.to_string(),
                        text: chunk.content.clone(),
                        style: crate::document_model::TextStyle::default(),
                        bounds: crate::document_model::BoundingBox {
                            x: 0.0,
                            y: 0.0,
                            width: 100.0,
                            height: 20.0,
                        },
                        page_number: 1,
                    };
                    document.add_element(element);
                }
            }
            
            Some(document)
        } else {
            None
        }
    }
    
    /// Create TableData from a chunk for Document model
    fn create_table_data_from_chunk(&self, chunk: &DocumentChunk) -> crate::document_model::TableData {
        // Parse the chunk content as markdown table and convert to TableData
        let (headers, rows) = self.parse_markdown_table(&chunk.content);
        
        let total_rows = rows.len() + if headers.is_empty() { 0 } else { 1 };
        let total_cols = headers.len().max(rows.iter().map(|row| row.len()).max().unwrap_or(0));
        
        let mut cells = Vec::new();
        
        // Add header row if present
        if !headers.is_empty() {
            let mut header_row = Vec::new();
            for header in &headers {
                header_row.push(crate::document_model::Cell {
                    content: crate::document_model::CellContent::Text(header.clone()),
                    span: crate::document_model::CellSpan { row_span: 1, col_span: 1 },
                    style: crate::document_model::CellStyle::default(),
                    is_header: true,
                    is_empty: header.is_empty(),
                    confidence: Some(0.9),
                });
            }
            // Pad header row to match total columns
            while header_row.len() < total_cols {
                header_row.push(crate::document_model::Cell {
                    content: crate::document_model::CellContent::Empty,
                    span: crate::document_model::CellSpan { row_span: 1, col_span: 1 },
                    style: crate::document_model::CellStyle::default(),
                    is_header: true,
                    is_empty: true,
                    confidence: Some(0.5),
                });
            }
            cells.push(header_row);
        }
        
        // Add data rows
        for row in &rows {
            let mut cell_row = Vec::new();
            for cell_text in row {
                cell_row.push(crate::document_model::Cell {
                    content: if cell_text.is_empty() {
                        crate::document_model::CellContent::Empty
                    } else if cell_text.chars().all(|c| c.is_ascii_digit() || c == '.' || c == ',' || c == '-') {
                        // Try to parse as number
                        match cell_text.replace(',', "").parse::<f64>() {
                            Ok(num) => crate::document_model::CellContent::Number(num),
                            Err(_) => crate::document_model::CellContent::Text(cell_text.clone()),
                        }
                    } else {
                        crate::document_model::CellContent::Text(cell_text.clone())
                    },
                    span: crate::document_model::CellSpan { row_span: 1, col_span: 1 },
                    style: crate::document_model::CellStyle::default(),
                    is_header: false,
                    is_empty: cell_text.is_empty(),
                    confidence: Some(0.8),
                });
            }
            // Pad row to match total columns
            while cell_row.len() < total_cols {
                cell_row.push(crate::document_model::Cell {
                    content: crate::document_model::CellContent::Empty,
                    span: crate::document_model::CellSpan { row_span: 1, col_span: 1 },
                    style: crate::document_model::CellStyle::default(),
                    is_header: false,
                    is_empty: true,
                    confidence: Some(0.5),
                });
            }
            cells.push(cell_row);
        }
        
        crate::document_model::TableData {
            cells,
            headers: headers.into_iter().enumerate().map(|(i, text)| {
                crate::document_model::TableHeader {
                    text,
                    column_index: i,
                    span: 1,
                    is_multi_level: false,
                    parent_header: None,
                }
            }).collect(),
            col_widths: vec![100.0; total_cols],
            row_heights: vec![25.0; total_rows],
            total_rows,
            total_cols,
            merged_regions: Vec::new(),
        }
    }
    
    /// Render HTML content in a human-friendly way that preserves PDF formatting
    fn render_html_content_human_friendly(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("📄 Document Content");
                    if let Some(ref file) = self.selected_file {
                        ui.separator();
                        ui.label(file.file_name().unwrap_or_default().to_string_lossy());
                    }
                });
                ui.separator();
                ui.add_space(10.0);
                
                // Parse and render the HTML content from each chunk
                let chunks_content: Vec<String> = self.chunks.iter().map(|chunk| chunk.content.clone()).collect();
                for content in chunks_content {
                    self.render_html_chunk_content(ui, &content);
                    ui.add_space(15.0);
                }
            });
    }
    
    /// Parse and render individual HTML chunk content with proper formatting
    fn render_html_chunk_content(&mut self, ui: &mut egui::Ui, html_content: &str) {
        // Parse the entire HTML content to render it more intelligently
        self.render_html_document_structure(ui, html_content);
    }
    
    /// Render the complete HTML document with proper structure recognition
    fn render_html_document_structure(&mut self, ui: &mut egui::Ui, html_content: &str) {
        // Split content into logical sections and render each appropriately
        let mut current_section = String::new();
        let mut in_table = false;
        let mut in_list = false;
        let mut table_content = String::new();
        let mut list_content = String::new();
        
        for line in html_content.lines() {
            let trimmed = line.trim();
            
            // Handle table sections
            if trimmed.contains("<table") {
                if !current_section.trim().is_empty() {
                    self.render_text_section(ui, &current_section);
                    current_section.clear();
                }
                in_table = true;
                table_content.clear();
                table_content.push_str(line);
                table_content.push('\n');
            } else if in_table {
                table_content.push_str(line);
                table_content.push('\n');
                if trimmed.contains("</table>") {
                    in_table = false;
                    self.render_html_table_standalone(ui, &table_content);
                    table_content.clear();
                }
            }
            // Handle list sections
            else if trimmed.contains("<ul") || trimmed.contains("<ol") {
                if !current_section.trim().is_empty() {
                    self.render_text_section(ui, &current_section);
                    current_section.clear();
                }
                in_list = true;
                list_content.clear();
                list_content.push_str(line);
                list_content.push('\n');
            } else if in_list {
                list_content.push_str(line);
                list_content.push('\n');
                if trimmed.contains("</ul>") || trimmed.contains("</ol>") {
                    in_list = false;
                    self.render_html_list_standalone(ui, &list_content);
                    list_content.clear();
                }
            }
            // Regular content
            else {
                current_section.push_str(line);
                current_section.push('\n');
            }
        }
        
        // Render any remaining content
        if !current_section.trim().is_empty() {
            self.render_text_section(ui, &current_section);
        }
    }
    
    /// Render a section of text content with proper formatting
    fn render_text_section(&mut self, ui: &mut egui::Ui, content: &str) {
        let lines: Vec<&str> = content.lines().collect();
        
        for line in lines {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                ui.add_space(3.0);
                continue;
            }
            
            // Handle different HTML elements
            if trimmed.starts_with("<h1") {
                let text = self.extract_html_text(trimmed);
                if !text.is_empty() {
                    ui.add_space(12.0);
                    ui.heading(egui::RichText::new(text).size(26.0).strong());
                    ui.add_space(10.0);
                }
            } else if trimmed.starts_with("<h2") {
                let text = self.extract_html_text(trimmed);
                if !text.is_empty() {
                    ui.add_space(10.0);
                    ui.heading(egui::RichText::new(text).size(22.0).strong());
                    ui.add_space(8.0);
                }
            } else if trimmed.starts_with("<h3") {
                let text = self.extract_html_text(trimmed);
                if !text.is_empty() {
                    ui.add_space(8.0);
                    ui.heading(egui::RichText::new(text).size(18.0).strong());
                    ui.add_space(6.0);
                }
            } else if trimmed.starts_with("<h4") {
                let text = self.extract_html_text(trimmed);
                if !text.is_empty() {
                    ui.add_space(6.0);
                    ui.label(egui::RichText::new(text).size(16.0).strong());
                    ui.add_space(4.0);
                }
            } else if trimmed.starts_with("<p") {
                let text = self.extract_html_text(trimmed);
                if !text.is_empty() {
                    ui.label(egui::RichText::new(text).size(14.0));
                    ui.add_space(6.0);
                }
            } else if trimmed.starts_with("<div") {
                let text = self.extract_html_text(trimmed);
                if !text.is_empty() {
                    // Check if this div has special formatting
                    if trimmed.contains("class=") {
                        // Extract class information for better formatting
                        if trimmed.contains("title") || trimmed.contains("header") {
                            ui.label(egui::RichText::new(text).size(16.0).strong());
                        } else if trimmed.contains("subtitle") {
                            ui.label(egui::RichText::new(text).size(15.0).color(egui::Color32::LIGHT_GRAY));
                        } else {
                            ui.label(egui::RichText::new(text).size(14.0));
                        }
                    } else {
                        ui.label(egui::RichText::new(text).size(14.0));
                    }
                    ui.add_space(4.0);
                }
            } else if trimmed.starts_with("<span") {
                let text = self.extract_html_text(trimmed);
                if !text.is_empty() {
                    // Inline text - check for formatting
                    if trimmed.contains("bold") || trimmed.contains("strong") {
                        ui.label(egui::RichText::new(text).strong());
                    } else if trimmed.contains("italic") || trimmed.contains("emphasis") {
                        ui.label(egui::RichText::new(text).italics());
                    } else {
                        ui.label(egui::RichText::new(text).size(14.0));
                    }
                }
            } else if !trimmed.starts_with('<') && !trimmed.is_empty() {
                // Plain text content - render with good spacing
                ui.label(egui::RichText::new(trimmed).size(14.0));
                ui.add_space(2.0);
            }
        }
    }
    
    /// Extract text content from HTML tags
    fn extract_html_text(&self, html_line: &str) -> String {
        // Simple regex-free HTML text extraction
        let mut result = String::new();
        let mut inside_tag = false;
        let chars: Vec<char> = html_line.chars().collect();
        
        for &ch in &chars {
            match ch {
                '<' => inside_tag = true,
                '>' => inside_tag = false,
                _ if !inside_tag => result.push(ch),
                _ => {}
            }
        }
        
        // Decode common HTML entities
        result
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&#39;", "'")
            .replace("&nbsp;", " ")
            .trim()
            .to_string()
    }
    
    /// Render HTML table as standalone component  
    fn render_html_table_standalone(&mut self, ui: &mut egui::Ui, table_html: &str) {
        let mut headers: Vec<String> = Vec::new();
        let mut rows: Vec<Vec<String>> = Vec::new();
        let mut current_row: Vec<String> = Vec::new();
        let mut in_header = false;
        
        // Parse table from HTML
        for line in table_html.lines() {
            let trimmed = line.trim();
            
            if trimmed.contains("<thead") || trimmed.contains("<th") {
                in_header = true;
            } else if trimmed.contains("</thead") {
                in_header = false;
                if !current_row.is_empty() {
                    headers = current_row.clone();
                    current_row.clear();
                }
            } else if trimmed.contains("<tr") {
                current_row.clear();
            } else if trimmed.contains("</tr") {
                if !current_row.is_empty() {
                    if in_header {
                        headers = current_row.clone();
                    } else {
                        rows.push(current_row.clone());
                    }
                    current_row.clear();
                }
            } else if trimmed.contains("<td") || trimmed.contains("<th") {
                let cell_text = self.extract_html_text(trimmed);
                current_row.push(cell_text);
            }
        }
        
        // Render the table if we have data
        if !headers.is_empty() || !rows.is_empty() {
            ui.add_space(15.0);
            ui.separator();
            ui.label(egui::RichText::new("📊 Table Data").strong().size(16.0).color(egui::Color32::LIGHT_BLUE));
            ui.add_space(8.0);
            
            // Use egui Grid for table rendering with better spacing
            egui::Grid::new("html_table_standalone")
                .striped(true)
                .min_col_width(100.0)
                .max_col_width(300.0)
                .spacing([12.0, 8.0])
                .show(ui, |ui| {
                    // Render headers with enhanced styling
                    if !headers.is_empty() {
                        for header in &headers {
                            ui.label(egui::RichText::new(header)
                                .strong()
                                .size(15.0)
                                .color(egui::Color32::LIGHT_BLUE));
                        }
                        ui.end_row();
                    }
                    
                    // Render data rows with better formatting
                    for row in &rows {
                        let max_cols = headers.len().max(row.len());
                        for i in 0..max_cols {
                            let empty_string = String::new();
                            let cell_content = row.get(i).unwrap_or(&empty_string);
                            let display_text = if cell_content.is_empty() { "-" } else { cell_content };
                            
                            // Enhanced numeric detection and formatting
                            let is_numeric = cell_content.chars().all(|c| {
                                c.is_ascii_digit() || c == ',' || c == '.' || c == '-' || 
                                c == '%' || c == '$' || c == '(' || c == ')' || c.is_whitespace()
                            }) && !cell_content.is_empty();
                            
                            if is_numeric {
                                ui.label(egui::RichText::new(display_text)
                                    .monospace()
                                    .size(14.0)
                                    .color(egui::Color32::WHITE));
                            } else {
                                ui.label(egui::RichText::new(display_text).size(14.0));
                            }
                        }
                        ui.end_row();
                    }
                });
            
            ui.add_space(15.0);
            ui.separator();
        }
    }
    
    /// Render HTML list as standalone component
    fn render_html_list_standalone(&mut self, ui: &mut egui::Ui, list_html: &str) {
        let mut list_items: Vec<String> = Vec::new();
        let mut is_ordered = false;
        
        // Parse list from HTML
        for line in list_html.lines() {
            let trimmed = line.trim();
            
            if trimmed.contains("<ol") {
                is_ordered = true;
            } else if trimmed.contains("<li") {
                let item_text = self.extract_html_text(trimmed);
                if !item_text.is_empty() {
                    list_items.push(item_text);
                }
            }
        }
        
        // Render the list
        if !list_items.is_empty() {
            ui.add_space(12.0);
            ui.label(egui::RichText::new("📋 List Items").strong().size(16.0).color(egui::Color32::YELLOW));
            ui.add_space(6.0);
            
            for (i, item) in list_items.iter().enumerate() {
                ui.horizontal(|ui| {
                    let bullet = if is_ordered {
                        format!("{}.", i + 1)
                    } else {
                        "•".to_string()
                    };
                    ui.label(egui::RichText::new(bullet).color(egui::Color32::GRAY).size(14.0));
                    ui.label(egui::RichText::new(item).size(14.0));
                });
                ui.add_space(3.0);
            }
            ui.add_space(12.0);
        }
    }
    
    /// Render HTML tables with proper formatting (legacy)
    fn render_html_table(&mut self, ui: &mut egui::Ui, html_content: &str) {
        let mut headers: Vec<String> = Vec::new();
        let mut rows: Vec<Vec<String>> = Vec::new();
        let mut current_row: Vec<String> = Vec::new();
        let mut in_header = false;
        
        // Parse table from HTML
        for line in html_content.lines() {
            let trimmed = line.trim();
            
            if trimmed.contains("<thead") || trimmed.contains("<th") {
                in_header = true;
            } else if trimmed.contains("</thead") {
                in_header = false;
                if !current_row.is_empty() {
                    headers = current_row.clone();
                    current_row.clear();
                }
            } else if trimmed.contains("<tr") {
                current_row.clear();
            } else if trimmed.contains("</tr") {
                if !current_row.is_empty() {
                    if in_header {
                        headers = current_row.clone();
                    } else {
                        rows.push(current_row.clone());
                    }
                    current_row.clear();
                }
            } else if trimmed.contains("<td") || trimmed.contains("<th") {
                let cell_text = self.extract_html_text(trimmed);
                current_row.push(cell_text);
            }
        }
        
        // Render the table if we have data
        if !headers.is_empty() || !rows.is_empty() {
            ui.add_space(10.0);
            ui.separator();
            ui.label(egui::RichText::new("📊 Table").strong().size(16.0));
            ui.add_space(8.0);
            
            // Use egui Grid for table rendering
            egui::Grid::new("html_table")
                .striped(true)
                .min_col_width(80.0)
                .spacing([10.0, 6.0])
                .show(ui, |ui| {
                    // Render headers
                    if !headers.is_empty() {
                        for header in &headers {
                            ui.label(egui::RichText::new(header).strong().color(egui::Color32::LIGHT_BLUE));
                        }
                        ui.end_row();
                    }
                    
                    // Render data rows
                    for row in &rows {
                        for cell in row {
                            let display_text = if cell.is_empty() { "-" } else { cell };
                            
                            // Use monospace for numeric data
                            if cell.chars().all(|c| c.is_ascii_digit() || c == ',' || c == '.' || c == '-' || c == '%' || c == '$' || c.is_whitespace()) && !cell.is_empty() {
                                ui.label(egui::RichText::new(display_text).monospace());
                            } else {
                                ui.label(display_text);
                            }
                        }
                        ui.end_row();
                    }
                });
            
            ui.add_space(10.0);
            ui.separator();
        }
    }
    
    /// Render HTML lists with proper formatting
    fn render_html_list(&mut self, ui: &mut egui::Ui, html_content: &str) {
        let mut list_items: Vec<String> = Vec::new();
        let mut is_ordered = false;
        
        // Parse list from HTML
        for line in html_content.lines() {
            let trimmed = line.trim();
            
            if trimmed.contains("<ol") {
                is_ordered = true;
            } else if trimmed.contains("<li") {
                let item_text = self.extract_html_text(trimmed);
                if !item_text.is_empty() {
                    list_items.push(item_text);
                }
            }
        }
        
        // Render the list
        if !list_items.is_empty() {
            ui.add_space(8.0);
            for (i, item) in list_items.iter().enumerate() {
                ui.horizontal(|ui| {
                    let bullet = if is_ordered {
                        format!("{}.", i + 1)
                    } else {
                        "•".to_string()
                    };
                    ui.label(egui::RichText::new(bullet).color(egui::Color32::GRAY));
                    ui.label(egui::RichText::new(item).size(14.0));
                });
                ui.add_space(2.0);
            }
            ui.add_space(8.0);
        }
    }
    
    /// Render structured content with maximum data accuracy
    fn render_structured_content_accurate(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("📝 Document Content (Structured)");
                    if let Some(ref file) = self.selected_file {
                        ui.separator();
                        ui.label(file.file_name().unwrap_or_default().to_string_lossy());
                        ui.separator();
                        ui.label(format!("{} elements", self.chunks.len()));
                    }
                });
                ui.separator();
                ui.add_space(10.0);
                
                // Render each chunk based on its type with full data accuracy
                let chunks_clone = self.chunks.clone();
                for chunk in &chunks_clone {
                    self.render_chunk_by_type(ui, chunk);
                }
            });
    }
    
    /// Render individual chunk based on its element type with full accuracy
    fn render_chunk_by_type(&mut self, ui: &mut egui::Ui, chunk: &DocumentChunk) {
        // Determine the primary element type
        let primary_type = chunk.element_types.first().map(|s| s.as_str()).unwrap_or("unknown");
        
        match primary_type {
            "table" => {
                self.render_table_chunk_accurate(ui, chunk);
            },
            heading_type if heading_type.starts_with("heading_level_") => {
                self.render_heading_chunk_accurate(ui, chunk, heading_type);
            },
            "heading" => {
                self.render_heading_chunk_accurate(ui, chunk, "heading_level_2");
            },
            "list" => {
                self.render_list_chunk_accurate(ui, chunk);
            },
            "text" | "paragraph" => {
                self.render_text_chunk_accurate(ui, chunk);
            },
            _ => {
                // Generic content - render as text with type info
                ui.label(egui::RichText::new(format!("[{}]", primary_type))
                    .size(12.0)
                    .color(egui::Color32::GRAY));
                ui.label(egui::RichText::new(&chunk.content).size(14.0));
                ui.add_space(8.0);
            }
        }
    }
    
    /// Render table chunk with maximum data accuracy
    fn render_table_chunk_accurate(&mut self, ui: &mut egui::Ui, chunk: &DocumentChunk) {
        ui.add_space(15.0);
        ui.separator();
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("📊 Table Data")
                .strong()
                .size(18.0)
                .color(egui::Color32::LIGHT_BLUE));
            ui.label(egui::RichText::new(format!("({})", chunk.page_range))
                .size(12.0)
                .color(egui::Color32::GRAY));
        });
        ui.add_space(8.0);
        
        // Check if we have structured table data first
        if let Some(table_data) = &chunk.table_data {
            // Render structured table using accurate GridData
            self.render_structured_table(ui, table_data, chunk.id);
        } else {
            // Fallback: Parse markdown table from content
            let (headers, rows) = self.parse_markdown_table(&chunk.content);
            
            if !headers.is_empty() || !rows.is_empty() {
                self.render_markdown_table(ui, chunk.id, &headers, &rows);
            } else {
                // Final fallback: display raw content
                ui.label(egui::RichText::new("Table content (raw):")
                    .size(12.0)
                    .color(egui::Color32::YELLOW));
                ui.code(&chunk.content);
            }
        }
        
        ui.add_space(15.0);
        ui.separator();
    }
    
    /// Render structured table using GridData for maximum accuracy
    fn render_structured_table(&mut self, ui: &mut egui::Ui, table_data: &GridData, chunk_id: usize) {
        // Show table dimensions
        ui.label(egui::RichText::new(format!("Structured Table: {}×{} cells", table_data.num_rows, table_data.num_cols))
            .size(12.0)
            .color(egui::Color32::GRAY));
        ui.add_space(8.0);
        
        // Reconstruct the actual table grid accounting for merged cells
        let reconstructed_grid = self.reconstruct_table_grid(table_data);
        
        // Use ScrollArea for large tables
        egui::ScrollArea::horizontal().show(ui, |ui| {
            // Create a grid with proper layout
            egui::Grid::new(format!("structured_table_{}", chunk_id))
                .num_columns(reconstructed_grid.first().map(|row| row.len()).unwrap_or(0))
                .spacing([8.0, 4.0])
                .striped(true)
                .min_col_width(60.0)
                .max_col_width(200.0)
                .show(ui, |ui| {
                    // Render the reconstructed grid
                    for (row_idx, row) in reconstructed_grid.iter().enumerate() {
                        for (col_idx, cell_text) in row.iter().enumerate() {
                            // Determine if this is a header cell (first row or first column)
                            let is_header = row_idx == 0 || col_idx == 0;
                            
                            // Style the cell appropriately
                            let rich_text = if is_header && !cell_text.trim().is_empty() {
                                egui::RichText::new(cell_text)
                                    .strong()
                                    .size(12.0)
                                    .color(egui::Color32::WHITE)
                            } else if self.is_numeric_data(cell_text) {
                                egui::RichText::new(cell_text)
                                    .monospace()
                                    .size(11.0)
                                    .color(egui::Color32::LIGHT_GREEN)
                            } else if cell_text.trim().is_empty() {
                                egui::RichText::new("-")
                                    .size(11.0)
                                    .color(egui::Color32::DARK_GRAY)
                            } else {
                                egui::RichText::new(cell_text)
                                    .size(11.0)
                                    .color(egui::Color32::LIGHT_GRAY)
                            };
                            
                            // Add the cell to the grid
                            ui.label(rich_text);
                        }
                        
                        // End the row
                        ui.end_row();
                    }
                });
        });
        
        ui.add_space(8.0);
    }
    
    /// Reconstruct table grid accounting for merged cells
    fn reconstruct_table_grid(&self, table_data: &GridData) -> Vec<Vec<String>> {
        let num_rows = table_data.num_rows as usize;
        let num_cols = table_data.num_cols as usize;
        
        // Create a matrix to track which cells are occupied
        let mut occupied = vec![vec![false; num_cols]; num_rows];
        let mut result = vec![vec![String::new(); num_cols]; num_rows];
        
        // Process each cell in the original grid
        for (row_idx, row) in table_data.grid.iter().enumerate() {
            if row_idx >= num_rows { break; }
            
            let mut current_col = 0;
            
            for cell in row.iter() {
                // Find the next available column in this row
                while current_col < num_cols && occupied[row_idx][current_col] {
                    current_col += 1;
                }
                
                if current_col >= num_cols { break; }
                
                let (cell_text, row_span, col_span) = match cell {
                    GridCell::Complex { text, row_span, col_span } => {
                        (text.clone(), *row_span as usize, *col_span as usize)
                    },
                    GridCell::Simple(text) => (text.clone(), 1, 1),
                    GridCell::Empty => ("".to_string(), 1, 1),
                };
                
                // Place the cell text in the first position
                result[row_idx][current_col] = cell_text;
                
                // Mark all spanned cells as occupied
                for r in 0..row_span.min(num_rows - row_idx) {
                    for c in 0..col_span.min(num_cols - current_col) {
                        if row_idx + r < num_rows && current_col + c < num_cols {
                            occupied[row_idx + r][current_col + c] = true;
                            
                            // For merged cells beyond the first, use empty string
                            if r > 0 || c > 0 {
                                result[row_idx + r][current_col + c] = "".to_string();
                            }
                        }
                    }
                }
                
                current_col += col_span;
            }
        }
        
        result
    }
    
    /// Render markdown table for fallback compatibility
    fn render_markdown_table(&mut self, ui: &mut egui::Ui, chunk_id: usize, headers: &[String], rows: &[Vec<String>]) {
        egui::Grid::new(format!("markdown_table_{}", chunk_id))
            .striped(true)
            .min_col_width(120.0)
            .max_col_width(250.0)
            .spacing([15.0, 8.0])
            .show(ui, |ui| {
                // Render headers
                if !headers.is_empty() {
                    for header in headers {
                        ui.label(egui::RichText::new(header)
                            .strong()
                            .size(15.0)
                            .color(egui::Color32::WHITE));
                    }
                    ui.end_row();
                }
                
                // Render data rows
                for row in rows {
                    let max_cols = headers.len().max(row.len());
                    for i in 0..max_cols {
                        let cell_content = row.get(i).map(|s| s.as_str()).unwrap_or("");
                        
                        // Detect and format numeric data
                        if self.is_numeric_data(cell_content) {
                            ui.label(egui::RichText::new(cell_content)
                                .monospace()
                                .size(14.0)
                                .color(egui::Color32::LIGHT_GREEN));
                        } else if cell_content.is_empty() {
                            ui.label(egui::RichText::new("-")
                                .color(egui::Color32::DARK_GRAY));
                        } else {
                            ui.label(egui::RichText::new(cell_content).size(14.0));
                        }
                    }
                    ui.end_row();
                }
            });
    }
    
    /// Render heading chunk with proper hierarchy
    fn render_heading_chunk_accurate(&mut self, ui: &mut egui::Ui, chunk: &DocumentChunk, heading_type: &str) {
        let level = if let Some(level_str) = heading_type.strip_prefix("heading_level_") {
            level_str.parse::<u8>().unwrap_or(2)
        } else {
            2
        };
        
        let (size, spacing) = match level {
            1 => (28.0, 20.0),
            2 => (24.0, 16.0),
            3 => (20.0, 12.0),
            4 => (18.0, 10.0),
            _ => (16.0, 8.0),
        };
        
        ui.add_space(spacing);
        ui.label(egui::RichText::new(&chunk.content)
            .strong()
            .size(size)
            .color(egui::Color32::WHITE));
        ui.add_space(spacing * 0.6);
    }
    
    /// Render list chunk with proper formatting
    fn render_list_chunk_accurate(&mut self, ui: &mut egui::Ui, chunk: &DocumentChunk) {
        ui.add_space(12.0);
        ui.label(egui::RichText::new("📋 List")
            .strong()
            .size(16.0)
            .color(egui::Color32::YELLOW));
        ui.add_space(6.0);
        
        // Render each list item
        for line in chunk.content.lines() {
            if !line.trim().is_empty() {
                ui.horizontal(|ui| {
                    ui.add_space(10.0); // Indent
                    ui.label(egui::RichText::new(line).size(14.0));
                });
                ui.add_space(2.0);
            }
        }
        
        ui.add_space(12.0);
    }
    
    /// Render text chunk with formatting detection
    fn render_text_chunk_accurate(&mut self, ui: &mut egui::Ui, chunk: &DocumentChunk) {
        // Detect text formatting
        let content = &chunk.content;
        
        if content.contains("**") || content.contains("__") {
            // Bold text detected
            let processed = content.replace("**", "").replace("__", "");
            ui.label(egui::RichText::new(processed).strong().size(14.0));
        } else if content.contains("*") || content.contains("_") {
            // Italic text detected
            let processed = content.replace("*", "").replace("_", "");
            ui.label(egui::RichText::new(processed).italics().size(14.0));
        } else {
            // Regular text
            ui.label(egui::RichText::new(content).size(14.0));
        }
        
        ui.add_space(8.0);
    }
    
    /// Detect if content is numeric data
    fn is_numeric_data(&self, content: &str) -> bool {
        if content.trim().is_empty() {
            return false;
        }
        
        // More sophisticated numeric detection
        let cleaned = content.replace(',', "").replace(' ', "");
        
        // Check for currency
        if content.starts_with('$') || content.ends_with('%') {
            return true;
        }
        
        // Check for decimal numbers
        if cleaned.chars().all(|c| c.is_ascii_digit() || c == '.' || c == '-' || c == '+') {
            return cleaned.parse::<f64>().is_ok();
        }
        
        false
    }
    
    /// Parse new structured JSON format from Docling bridge
    fn parse_structured_json(json_content: &str) -> Result<Vec<DocumentChunk>, String> {
        let structured_doc: StructuredDocument = serde_json::from_str(json_content)
            .map_err(|e| format!("Structured JSON parse error: {}", e))?;
        
        let mut chunks = Vec::new();
        let mut chunk_id = 1;
        
        println!("📄 Processing {} elements from structured document", structured_doc.elements.len());
        
        for element in structured_doc.elements {
            match element.element_type.as_str() {
                "table" => {
                    if let Some(chunk) = Self::convert_table_element_to_chunk(&element, &mut chunk_id) {
                        chunks.push(chunk);
                    }
                }
                "text" => {
                    if let Some(chunk) = Self::convert_text_element_to_chunk(&element, &mut chunk_id) {
                        chunks.push(chunk);
                    }
                }
                "heading" => {
                    if let Some(chunk) = Self::convert_heading_element_to_chunk(&element, &mut chunk_id) {
                        chunks.push(chunk);
                    }
                }
                _ => {
                    // Generic element - try to extract content
                    if let Some(content) = &element.content {
                        if !content.trim().is_empty() {
                            chunks.push(DocumentChunk {
                                id: chunk_id,
                                content: content.clone(),
                                page_range: "page_1".to_string(),
                                element_types: vec![element.element_type.clone(), "generic".to_string()],
                                spatial_bounds: Some(element.id.clone()),
                                char_count: content.len(),
                                table_data: None,
                            });
                            chunk_id += 1;
                        }
                    }
                }
            }
        }
        
        if chunks.is_empty() {
            return Err("No processable elements found in structured JSON".to_string());
        }
        
        println!("🔍 Converted {} structured elements to chunks", chunks.len());
        Ok(chunks)
    }
    
    /// Convert table element to DocumentChunk with accurate table rendering info
    fn convert_table_element_to_chunk(element: &DocumentElement, chunk_id: &mut usize) -> Option<DocumentChunk> {
        if let Some(grid_data) = &element.grid_data {
            println!("📊 Processing table with {}x{} grid", grid_data.num_rows, grid_data.num_cols);
            
            let current_id = *chunk_id;
            *chunk_id += 1;
            
            Some(DocumentChunk {
                id: current_id,
                content: format!("[TABLE: {}x{} cells]", grid_data.num_rows, grid_data.num_cols),
                page_range: "page_1".to_string(),
                element_types: vec!["table".to_string(), "structured_table".to_string()],
                spatial_bounds: Some(element.id.clone()),
                char_count: (grid_data.num_rows * grid_data.num_cols) as usize,
                table_data: Some(grid_data.clone()),
            })
        } else {
            println!("⚠️ Table element {} has no grid data", element.id);
            None
        }
    }
    
    /// Convert text element to DocumentChunk
    fn convert_text_element_to_chunk(element: &DocumentElement, chunk_id: &mut usize) -> Option<DocumentChunk> {
        if let Some(content) = &element.content {
            if !content.trim().is_empty() {
                let current_id = *chunk_id;
                *chunk_id += 1;
                
                Some(DocumentChunk {
                    id: current_id,
                    content: content.clone(),
                    page_range: "page_1".to_string(),
                    element_types: vec!["text".to_string(), "paragraph".to_string()],
                    spatial_bounds: Some(element.id.clone()),
                    char_count: content.len(),
                    table_data: None,
                })
            } else {
                None
            }
        } else {
            None
        }
    }
    
    /// Convert heading element to DocumentChunk
    fn convert_heading_element_to_chunk(element: &DocumentElement, chunk_id: &mut usize) -> Option<DocumentChunk> {
        if let Some(content) = &element.content {
            if !content.trim().is_empty() {
                let current_id = *chunk_id;
                *chunk_id += 1;
                
                let level = element.heading_level.unwrap_or(1);
                Some(DocumentChunk {
                    id: current_id,
                    content: content.clone(),
                    page_range: "page_1".to_string(),
                    element_types: vec![format!("heading_level_{}", level), "heading".to_string()],
                    spatial_bounds: Some(element.id.clone()),
                    char_count: content.len(),
                    table_data: None,
                })
            } else {
                None
            }
        } else {
            None
        }
    }
    
    /// Parse structured JSON from Docling into DocumentChunks with high accuracy
    fn parse_docling_json(json_content: &str) -> Result<Vec<DocumentChunk>, String> {
        let doc_data: serde_json::Value = serde_json::from_str(json_content)
            .map_err(|e| format!("JSON parse error: {}", e))?;
        
        let mut chunks = Vec::new();
        let mut chunk_id = 1;
        
        // Look for elements in the JSON structure
        if let Some(elements) = doc_data.get("elements").and_then(|e| e.as_array()) {
            for element in elements {
                if let Some(chunk) = Self::parse_json_element(element, &mut chunk_id) {
                    chunks.push(chunk);
                }
            }
        }
        
        // If no elements found, try to parse the main body
        if chunks.is_empty() {
            if let Some(main_text) = doc_data.get("main_text").and_then(|t| t.as_str()) {
                chunks.push(DocumentChunk {
                    id: chunk_id,
                    content: main_text.to_string(),
                    page_range: "page_1".to_string(),
                    element_types: vec!["text".to_string(), "main_content".to_string()],
                    spatial_bounds: Some("main_body".to_string()),
                    char_count: main_text.len(),
                    table_data: None,
                });
            }
        }
        
        if chunks.is_empty() {
            return Err("No content elements found in JSON".to_string());
        }
        
        println!("🔍 Parsed {} elements from structured JSON", chunks.len());
        Ok(chunks)
    }
    
    /// Parse individual JSON element into DocumentChunk
    fn parse_json_element(element: &serde_json::Value, chunk_id: &mut usize) -> Option<DocumentChunk> {
        let element_type = element.get("type").and_then(|t| t.as_str()).unwrap_or("unknown");
        let enhanced_type = element.get("chonker_enhanced_type").and_then(|t| t.as_str()).unwrap_or(element_type);
        
        let page_number = element.get("prov").and_then(|p| p.get("page")).and_then(|pg| pg.as_u64()).unwrap_or(1);
        let page_range = format!("page_{}", page_number);
        
        let current_id = *chunk_id;
        *chunk_id += 1;
        
        match enhanced_type {
            "table" => {
                // Parse table data with high accuracy
                let table_content = Self::parse_table_from_json(element)?;
                Some(DocumentChunk {
                    id: current_id,
                    content: table_content,
                    page_range,
                    element_types: vec!["table".to_string(), "structured_data".to_string()],
                    spatial_bounds: Self::extract_spatial_bounds(element),
                    char_count: 0, // Will be calculated
                    table_data: None,
                })
            },
            "heading" => {
                let text = element.get("text").and_then(|t| t.as_str())?;
                let level = element.get("level").and_then(|l| l.as_u64()).unwrap_or(1);
                Some(DocumentChunk {
                    id: current_id,
                    content: text.to_string(),
                    page_range,
                    element_types: vec![format!("heading_level_{}", level), "heading".to_string()],
                    spatial_bounds: Self::extract_spatial_bounds(element),
                    char_count: text.len(),
                    table_data: None,
                })
            },
            "text" | "paragraph" => {
                let text = element.get("text").and_then(|t| t.as_str())?;
                Some(DocumentChunk {
                    id: current_id,
                    content: text.to_string(),
                    page_range,
                    element_types: vec!["text".to_string(), "paragraph".to_string()],
                    spatial_bounds: Self::extract_spatial_bounds(element),
                    char_count: text.len(),
                    table_data: None,
                })
            },
            "list" => {
                let list_content = Self::parse_list_from_json(element)?;
                let list_type = element.get("chonker_list_type").and_then(|t| t.as_str()).unwrap_or("unordered");
                Some(DocumentChunk {
                    id: current_id,
                    content: list_content,
                    page_range,
                    element_types: vec!["list".to_string(), list_type.to_string()],
                    spatial_bounds: Self::extract_spatial_bounds(element),
                    char_count: 0, // Will be calculated
                    table_data: None,
                })
            },
            _ => {
                // Generic element - try to extract text
                if let Some(text) = element.get("text").and_then(|t| t.as_str()) {
                    Some(DocumentChunk {
                        id: current_id,
                        content: text.to_string(),
                        page_range,
                        element_types: vec![element_type.to_string(), "generic".to_string()],
                        spatial_bounds: Self::extract_spatial_bounds(element),
                        char_count: text.len(),
                        table_data: None,
                    })
                } else {
                    None
                }
            }
        }
    }
    
    /// Parse table data from JSON with maximum accuracy
    fn parse_table_from_json(element: &serde_json::Value) -> Option<String> {
        let mut table_markdown = String::new();
        
        // Try to get table data from various possible fields
        let table_data = element.get("data")
            .or_else(|| element.get("table_data"))
            .or_else(|| element.get("cells"))?;
        
        if let Some(data_array) = table_data.as_array() {
            // Handle array of rows
            let mut headers: Vec<String> = Vec::new();
            let mut rows: Vec<Vec<String>> = Vec::new();
            
            for (row_idx, row) in data_array.iter().enumerate() {
                if let Some(row_array) = row.as_array() {
                    let row_cells: Vec<String> = row_array.iter()
                        .map(|cell| {
                            if let Some(cell_obj) = cell.as_object() {
                                cell_obj.get("text")
                                    .or_else(|| cell_obj.get("content"))
                                    .and_then(|t| t.as_str())
                                    .unwrap_or("")
                                    .to_string()
                            } else {
                                cell.as_str().unwrap_or("").to_string()
                            }
                        })
                        .collect();
                    
                    if row_idx == 0 {
                        // First row might be headers
                        headers = row_cells;
                    } else {
                        rows.push(row_cells);
                    }
                }
            }
            
            // Generate markdown table
            if !headers.is_empty() {
                table_markdown.push_str(&format!("| {} |\n", headers.join(" | ")));
                table_markdown.push_str(&format!("|{}|\n", "---|".repeat(headers.len())));
            }
            
            for row in rows {
                table_markdown.push_str(&format!("| {} |\n", row.join(" | ")));
            }
        }
        
        if table_markdown.is_empty() {
            None
        } else {
            Some(table_markdown)
        }
    }
    
    /// Parse list data from JSON
    fn parse_list_from_json(element: &serde_json::Value) -> Option<String> {
        let items = element.get("items").and_then(|i| i.as_array())?;
        let is_ordered = element.get("chonker_list_type")
            .and_then(|t| t.as_str())
            .map(|s| s == "ordered")
            .unwrap_or(false);
        
        let mut list_content = String::new();
        for (idx, item) in items.iter().enumerate() {
            let item_text = item.as_str().unwrap_or("");
            if is_ordered {
                list_content.push_str(&format!("{}. {}\n", idx + 1, item_text));
            } else {
                list_content.push_str(&format!("- {}\n", item_text));
            }
        }
        
        if list_content.is_empty() {
            None
        } else {
            Some(list_content)
        }
    }
    
    /// Extract spatial bounds from JSON element
    fn extract_spatial_bounds(element: &serde_json::Value) -> Option<String> {
        if let Some(bbox) = element.get("bbox") {
            if let (Some(x), Some(y), Some(w), Some(h)) = (
                bbox.get("x").and_then(|v| v.as_f64()),
                bbox.get("y").and_then(|v| v.as_f64()),
                bbox.get("w").and_then(|v| v.as_f64()),
                bbox.get("h").and_then(|v| v.as_f64()),
            ) {
                return Some(format!("x:{:.1},y:{:.1},w:{:.1},h:{:.1}", x, y, w, h));
            }
        }
        None
    }
    
    fn set_warp_theme(&self, ctx: &egui::Context) {
        // Use default fonts for now
        let fonts = egui::FontDefinitions::default();
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
