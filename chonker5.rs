#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! eframe = "0.24"
//! egui = "0.24"
//! rfd = "0.15"
//! image = "0.25"
//! pdfium-render = { version = "0.8", features = ["thread_safe"] }
//! tokio = { version = "1.38", features = ["full", "rt-multi-thread"] }
//! anyhow = "1.0"
//! tracing = "0.1"
//! tracing-subscriber = { version = "0.3", features = ["env-filter"] }
//! serde = { version = "1.0", features = ["derive"] }
//! serde_json = "1.0"
//! ```

use eframe::egui;
use egui::{Color32, RichText, FontId, Stroke, Rounding};
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use tokio::sync::mpsc;
use anyhow::Result;

// We'll use the ferrules binary instead of direct integration due to compilation issues
use serde::{Deserialize, Serialize};

// Teal and chrome color scheme
const TERM_BG: Color32 = Color32::from_rgb(10, 15, 20); // Dark blue-black
const TERM_FG: Color32 = Color32::from_rgb(26, 188, 156); // Teal (#1ABC9C)
const TERM_BORDER: Color32 = Color32::from_rgb(192, 192, 192); // Chrome/silver
const TERM_HIGHLIGHT: Color32 = Color32::from_rgb(22, 160, 133); // Darker teal (#16A085)
const TERM_ERROR: Color32 = Color32::from_rgb(255, 80, 80); // Soft red
const TERM_DIM: Color32 = Color32::from_rgb(80, 100, 100); // Muted teal-gray
const TERM_YELLOW: Color32 = Color32::from_rgb(255, 200, 0); // Gold accent
const CHROME: Color32 = Color32::from_rgb(82, 86, 89); // Chrome (#525659)

// Box drawing characters
const BOX_TL: &str = "╔";
const BOX_TR: &str = "╗";
const BOX_BL: &str = "╚";
const BOX_BR: &str = "╝";
const BOX_H: &str = "═";
const BOX_V: &str = "║";
const BOX_T: &str = "╦";
const BOX_B: &str = "╩";
const BOX_L: &str = "╠";
const BOX_R: &str = "╣";
const BOX_CROSS: &str = "╬";

#[derive(Default)]
struct ExtractionResult {
    content: String,
    parsed_doc: Option<ParsedDocument>,
    is_loading: bool,
    error: Option<String>,
}

// Simplified ferrules entities to avoid compilation issues
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ParsedDocument {
    pages: Vec<Page>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Page {
    page_id: PageId,
    elements: Vec<Element>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PageId(usize);

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Element {
    text: String,
    bbox: BBox,
    element_type: ElementType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BBox {
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
enum ElementType {
    Title { level: TitleLevel },
    Text,
    List { items: Vec<String> },
    Table,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TitleLevel(usize);

#[derive(Clone)]
struct BoundingBox {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    label: String,
    confidence: f32,
    color: Color32,
}

struct Chonker5App {
    // PDF state
    pdf_path: Option<PathBuf>,
    current_page: usize,
    total_pages: usize,
    zoom_level: f32,
    pdf_texture: Option<egui::TextureHandle>,
    needs_render: bool,
    
    // UI assets
    hamster_texture: Option<egui::TextureHandle>,
    
    // Extraction state
    page_range: String,
    vision_result: ExtractionResult,
    data_result: ExtractionResult,
    active_tab: ExtractionTab,
    
    // Ferrules binary path
    ferrules_binary: Option<PathBuf>,
    
    // Async runtime
    runtime: Arc<tokio::runtime::Runtime>,
    
    // Channel for async results
    vision_receiver: Option<mpsc::Receiver<Result<ParsedDocument, String>>>,
    
    // Log messages
    log_messages: Vec<String>,
    
    // UI state
    show_bounding_boxes: bool,
    selected_page: usize,
    split_ratio: f32,
}

#[derive(PartialEq, Clone)]
enum ExtractionTab {
    Pdf,
    Vision,
    Data,
}

impl Chonker5App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        
        // Initialize async runtime
        let runtime = Arc::new(
            tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime")
        );
        
        // Initialize tracing
        tracing_subscriber::fmt::init();
        
        // Load hamster image
        let hamster_texture = if let Ok(image_data) = std::fs::read("./assets/emojis/chonker.png") {
            if let Ok(image) = image::load_from_memory(&image_data) {
                let size = [image.width() as _, image.height() as _];
                let image_buffer = image.to_rgba8();
                let pixels = image_buffer.as_flat_samples();
                let color_image = egui::ColorImage::from_rgba_unmultiplied(
                    size,
                    pixels.as_slice(),
                );
                Some(cc.egui_ctx.load_texture(
                    "hamster",
                    color_image,
                    Default::default()
                ))
            } else {
                None
            }
        } else {
            None
        };
        
        let mut app = Self {
            pdf_path: None,
            current_page: 0,
            total_pages: 0,
            zoom_level: 1.0,
            pdf_texture: None,
            needs_render: false,
            hamster_texture,
            page_range: "1-10".to_string(),
            vision_result: Default::default(),
            data_result: Default::default(),
            active_tab: ExtractionTab::Pdf,
            ferrules_binary: None,
            runtime,
            vision_receiver: None,
            log_messages: vec![
                "🐹 CHONKER 5 Ready!".to_string(),
                "📌 Using MuPDF for PDF rendering + Ferrules/Pdfium for structured data extraction".to_string(),
                "📌 Vision Extraction: AI-powered layout analysis | Data Extraction: Complete text content".to_string(),
            ],
            show_bounding_boxes: true,
            selected_page: 0,
            split_ratio: 0.7,
        };
        
        // Initialize ferrules binary path
        app.init_ferrules_binary();
        
        app
    }
    
    fn init_ferrules_binary(&mut self) {
        self.log("🔄 Looking for Ferrules binary...");
        
        // Check common locations for ferrules binary
        let possible_paths = vec![
            PathBuf::from("./ferrules/target/release/ferrules"),
            PathBuf::from("./ferrules/target/debug/ferrules"),
            PathBuf::from("./ferrules"),
            PathBuf::from("/usr/local/bin/ferrules"),
        ];
        
        for path in &possible_paths {
            if path.exists() {
                self.ferrules_binary = Some(path.clone());
                self.log(&format!("✅ Found Ferrules binary at: {}", path.display()));
                return;
            }
        }
        
        // Try to find it in PATH
        if let Ok(output) = Command::new("which")
            .arg("ferrules")
            .output()
        {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                self.ferrules_binary = Some(PathBuf::from(path.clone()));
                self.log(&format!("✅ Found Ferrules binary in PATH: {}", path));
                return;
            }
        }
        
        self.log("⚠️ Ferrules binary not found. Vision extraction will use fallback.");
    }
    
    fn log(&mut self, message: &str) {
        self.log_messages.push(message.to_string());
        if self.log_messages.len() > 100 {
            self.log_messages.remove(0);
        }
    }
    
    fn open_file(&mut self, ctx: &egui::Context) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("PDF files", &["pdf"])
            .pick_file()
        {
            self.pdf_path = Some(path.clone());
            self.current_page = 0;
            self.pdf_texture = None;
            
            // Get PDF info
            match self.get_pdf_info(&path) {
                Ok(pages) => {
                    self.total_pages = pages;
                    self.log(&format!("📄 Loaded PDF: {} ({} pages)", path.display(), pages));
                    
                    // Set default page range for large PDFs
                    if pages > 20 {
                        self.page_range = "1-10".to_string();
                        self.log("📄 Large PDF detected - Default page range set to 1-10");
                    } else {
                        self.page_range.clear();
                    }
                    
                    // Render the first page
                    self.render_current_page(ctx);
                }
                Err(e) => {
                    self.log(&format!("❌ Failed to load PDF: {}", e));
                }
            }
        }
    }
    
    fn get_pdf_info(&self, path: &PathBuf) -> Result<usize> {
        let output = Command::new("mutool")
            .arg("info")
            .arg(path)
            .output()?;
        
        let info = String::from_utf8_lossy(&output.stdout);
        for line in info.lines() {
            if line.contains("Pages:") {
                if let Some(pages_str) = line.split(':').nth(1) {
                    return pages_str.trim().parse().map_err(|e| anyhow::anyhow!("Parse error: {}", e));
                }
            }
        }
        
        Err(anyhow::anyhow!("Could not determine page count"))
    }
    
    fn render_current_page(&mut self, ctx: &egui::Context) {
        if let Some(pdf_path) = &self.pdf_path {
            // Use mutool to render the current page to a PNG
            let temp_png = std::env::temp_dir().join(format!("chonker5_page_{}.png", self.current_page));
            
            let dpi = 150.0 * self.zoom_level;
            let result = Command::new("mutool")
                .arg("draw")
                .arg("-o")
                .arg(&temp_png)
                .arg("-r")
                .arg(dpi.to_string())
                .arg("-F")
                .arg("png")
                .arg(&pdf_path)
                .arg(format!("{}", self.current_page + 1))
                .output();
                
            match result {
                Ok(output) => {
                    if output.status.success() {
                        // Load the PNG as a texture
                        if let Ok(image_data) = std::fs::read(&temp_png) {
                            if let Ok(image) = image::load_from_memory(&image_data) {
                                let size = [image.width() as _, image.height() as _];
                                let image_buffer = image.to_rgba8();
                                let pixels = image_buffer.as_flat_samples();
                                
                                let image = egui::ColorImage::from_rgba_unmultiplied(
                                    size,
                                    pixels.as_slice(),
                                );
                                
                                self.pdf_texture = Some(ctx.load_texture(
                                    format!("pdf_page_{}", self.current_page),
                                    image,
                                    Default::default()
                                ));
                                
                                self.log(&format!("📄 Rendered page {}", self.current_page + 1));
                            }
                        }
                        
                        // Clean up temp file
                        let _ = std::fs::remove_file(&temp_png);
                    } else {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        self.log(&format!("❌ Failed to render page: {}", stderr));
                    }
                }
                Err(e) => {
                    self.log(&format!("❌ Failed to run mutool: {}", e));
                }
            }
        }
    }
    
    fn extract_vision_data(&mut self, ctx: &egui::Context) {
        if self.pdf_path.is_none() {
            self.log("⚠️ No PDF loaded. Open a file first.");
            return;
        }
        
        if self.ferrules_binary.is_none() {
            self.log("❌ Ferrules binary not found. Using fallback extraction.");
            // Could implement a fallback here
            self.vision_result.error = Some("Ferrules binary not found".to_string());
            return;
        }
        
        let pdf_path = self.pdf_path.clone().unwrap();
        let page_range = self.page_range.clone();
        let ferrules_path = self.ferrules_binary.clone().unwrap();
        let runtime = self.runtime.clone();
        let ctx = ctx.clone();
        
        self.vision_result.is_loading = true;
        self.vision_result.error = None;
        self.log(&format!("🔄 Extracting structured data with Vision AI (pages {})...", 
            if page_range.is_empty() { "all" } else { &page_range }));
        
        // Parse page range
        let parsed_range = if !page_range.is_empty() {
            match parse_page_range(&page_range) {
                Ok(r) => Some(r),
                Err(e) => {
                    self.vision_result.error = Some(format!("Invalid page range: {}", e));
                    self.vision_result.is_loading = false;
                    return;
                }
            }
        } else {
            None
        };
        
        // Create channel for results
        let (tx, rx) = mpsc::channel(1);
        self.vision_receiver = Some(rx);
        
        // Spawn async task
        runtime.spawn(async move {
            let result = async {
                // Create temporary directory for ferrules output
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis();
                let temp_dir = std::env::temp_dir().join(format!("chonker5_ferrules_{}", timestamp));
                std::fs::create_dir_all(&temp_dir)?;
                
                // Build ferrules command
                let mut cmd = Command::new(&ferrules_path);
                cmd.arg(&pdf_path);
                cmd.arg("--output-dir").arg(&temp_dir);
                
                // Add page range if specified  
                if let Some(range) = parsed_range {
                    cmd.arg("--page-range").arg(format!("{}-{}", range.start + 1, range.end));
                }
                
                // Use markdown output
                cmd.arg("--md");
                
                // Execute ferrules
                let output = cmd.output()?;
                
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    return Err(anyhow::anyhow!("Ferrules failed with status {}: STDERR: {} STDOUT: {}", 
                        output.status, stderr, stdout));
                }
                
                // Read the markdown file that ferrules created
                let mut markdown_content = String::new();
                
                // Log the temp directory
                eprintln!("Looking for ferrules output in: {:?}", temp_dir);
                
                // Look for files in the output directory and subdirectories
                let mut dirs_to_check = vec![temp_dir.clone()];
                
                // Also check subdirectories - ferrules creates subdirectories for each page
                if let Ok(entries) = std::fs::read_dir(&temp_dir) {
                    for entry in entries {
                        if let Ok(entry) = entry {
                            let path = entry.path();
                            if path.is_dir() {
                                eprintln!("Found subdirectory: {:?}", path);
                                dirs_to_check.push(path);
                            }
                        }
                    }
                }
                
                // Collect all markdown content from all directories
                let mut all_markdown_content = Vec::new();
                
                for dir in dirs_to_check {
                    eprintln!("Checking directory: {:?}", dir);
                    
                    // First check for markdown files directly in the directory
                    if let Ok(entries) = std::fs::read_dir(&dir) {
                        for entry in entries {
                            if let Ok(entry) = entry {
                                let path = entry.path();
                                
                                if path.is_file() {
                                    eprintln!("Checking file: {:?}", path);
                                    
                                    // Check for .md extension
                                    if path.extension().and_then(|s| s.to_str()) == Some("md") {
                                        if let Ok(content) = std::fs::read_to_string(&path) {
                                            eprintln!("Read markdown file {} with {} bytes", path.display(), content.len());
                                            if !content.is_empty() {
                                                all_markdown_content.push(content);
                                            }
                                        }
                                    }
                                } else if path.is_dir() {
                                    // Check nested directories
                                    eprintln!("Checking nested directory: {:?}", path);
                                    if let Ok(nested_entries) = std::fs::read_dir(&path) {
                                        for nested_entry in nested_entries {
                                            if let Ok(nested_entry) = nested_entry {
                                                let nested_path = nested_entry.path();
                                                if nested_path.is_file() && nested_path.extension().and_then(|s| s.to_str()) == Some("md") {
                                                    if let Ok(content) = std::fs::read_to_string(&nested_path) {
                                                        eprintln!("Read nested markdown file {} with {} bytes", nested_path.display(), content.len());
                                                        if !content.is_empty() {
                                                            all_markdown_content.push(content);
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                
                // Combine all markdown content
                if !all_markdown_content.is_empty() {
                    markdown_content = all_markdown_content.join("\n\n---\n\n");
                    eprintln!("Combined {} markdown files into {} total bytes", all_markdown_content.len(), markdown_content.len());
                } else {
                    // If no markdown file found, check stdout
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    eprintln!("No markdown files found. STDOUT: {}, STDERR: {}", stdout, stderr);
                    
                    if !stdout.trim().is_empty() {
                        markdown_content = stdout.to_string();
                    } else {
                        markdown_content = format!("Extraction completed. Files saved to:\n{}\n\nPlease check the directory for the extracted content.", temp_dir.display());
                    }
                }
                
                // Parse the markdown content
                let mut pages = Vec::new();
                let mut current_page = Page {
                    page_id: PageId(0),
                    elements: Vec::new(),
                };
                
                // Parse markdown into elements
                for line in markdown_content.lines() {
                    if line.trim().is_empty() {
                        continue;
                    }
                    
                    let element_type = if line.starts_with('#') {
                        let level = line.chars().take_while(|&c| c == '#').count();
                        ElementType::Title { level: TitleLevel(level) }
                    } else if line.starts_with('|') {
                        ElementType::Table
                    } else if line.starts_with('-') || line.starts_with('*') {
                        ElementType::List { items: vec![line.trim_start_matches(|c| c == '-' || c == '*').trim().to_string()] }
                    } else {
                        ElementType::Text
                    };
                    
                    current_page.elements.push(Element {
                        text: line.to_string(),
                        bbox: BBox { x0: 0.0, y0: 0.0, x1: 100.0, y1: 100.0 },
                        element_type,
                    });
                }
                
                if !current_page.elements.is_empty() {
                    pages.push(current_page);
                }
                
                // If no elements found, show the raw content
                if pages.is_empty() {
                    pages.push(Page {
                        page_id: PageId(0),
                        elements: vec![Element {
                            text: markdown_content,
                            bbox: BBox { x0: 0.0, y0: 0.0, x1: 100.0, y1: 100.0 },
                            element_type: ElementType::Text,
                        }],
                    });
                }
                
                let parsed_doc = ParsedDocument { pages };
                
                // Clean up temp directory
                let _ = std::fs::remove_dir_all(&temp_dir);
                
                Ok::<_, anyhow::Error>(parsed_doc)
            }.await;
            
            // Send result through channel
            let _ = tx.send(result.map_err(|e| e.to_string())).await;
            
            // Update UI on main thread
            ctx.request_repaint();
        });
    }
    
    fn extract_data_content(&mut self) {
        if let Some(pdf_path) = self.pdf_path.clone() {
            self.data_result.is_loading = true;
            self.log("🔄 Extracting all content with pdfium-render...");
            
            match self.pdfium_text_extraction(&pdf_path) {
                Ok(content) => {
                    self.data_result.content = content;
                    self.data_result.is_loading = false;
                    self.log("✅ Content extraction completed");
                }
                Err(e) => {
                    self.data_result.error = Some(format!("Extraction failed: {}", e));
                    self.data_result.is_loading = false;
                    self.log(&format!("❌ Failed to extract content: {}", e));
                }
            }
        } else {
            self.log("⚠️ No PDF loaded. Open a file first.");
        }
    }
    
    fn pdfium_text_extraction(&self, pdf_path: &std::path::Path) -> Result<String, Box<dyn std::error::Error>> {
        let mut result = String::new();
        result.push_str("╔═══ DATA EXTRACTION RESULTS ═══╗\n\n");
        
        // Initialize pdfium
        let pdfium = pdfium_render::prelude::Pdfium::new(
            pdfium_render::prelude::Pdfium::bind_to_library("./lib/libpdfium.dylib").or_else(|_| {
                pdfium_render::prelude::Pdfium::bind_to_library("/usr/local/lib/libpdfium.dylib")
            }).or_else(|_| {
                pdfium_render::prelude::Pdfium::bind_to_system_library()
            }).map_err(|e| format!("Failed to bind to pdfium library: {}", e))?
        );
        
        // Load the PDF
        let document = pdfium.load_pdf_from_file(pdf_path, None)
            .map_err(|e| format!("Failed to load PDF: {}", e))?;
        
        result.push_str(&format!("Document: {}\n", pdf_path.file_name().unwrap_or_default().to_string_lossy()));
        result.push_str(&format!("Total Pages: {}\n\n", document.pages().len()));
        
        // Process each page
        for (page_index, page) in document.pages().iter().enumerate() {
            let page_number = page_index + 1;
            result.push_str(&format!("╔═══ PAGE {} ═══╗\n", page_number));
            
            // Extract all text
            let text_page = page.text().map_err(|e| format!("Failed to get text: {}", e))?;
            let page_text = text_page.all();
            
            if !page_text.trim().is_empty() {
                result.push_str(&page_text);
                result.push_str("\n");
            } else {
                result.push_str("║ [No text content detected] ║\n");
            }
            
            result.push_str("╚═════════════════════════════╝\n\n");
        }
        
        Ok(result)
    }
    
    fn draw_bounding_boxes(&self, ui: &mut egui::Ui, image_response: &egui::Response, 
                          parsed_doc: &ParsedDocument, display_size: egui::Vec2, original_size: egui::Vec2) {
        let painter = ui.painter();
        let image_rect = image_response.rect;
        
        // Calculate scaling factors
        let scale_x = display_size.x / original_size.x;
        let scale_y = display_size.y / original_size.y;
        
        // Draw bounding boxes for current page
        if let Some(page) = parsed_doc.pages.get(self.current_page) {
            for element in &page.elements {
                // Transform bounding box coordinates
                let x1 = image_rect.left() + element.bbox.x0 * scale_x;
                let y1 = image_rect.top() + element.bbox.y0 * scale_y;
                let x2 = image_rect.left() + element.bbox.x1 * scale_x;
                let y2 = image_rect.top() + element.bbox.y1 * scale_y;
                
                let rect = egui::Rect::from_min_max(
                    egui::pos2(x1, y1),
                    egui::pos2(x2, y2)
                );
                
                // Choose color based on element type
                let color = match element.element_type {
                    ElementType::Title { .. } => Color32::from_rgb(255, 100, 100), // Red for titles
                    ElementType::Text => TERM_HIGHLIGHT, // Teal for text
                    ElementType::List { .. } => Color32::from_rgb(100, 255, 100), // Green for lists
                    ElementType::Table => Color32::from_rgb(255, 255, 100), // Yellow for tables
                    ElementType::Other => TERM_DIM, // Gray for other
                };
                
                // Draw bounding box
                painter.rect_stroke(rect, 0.0, egui::Stroke::new(2.0, color));
                
                // Draw label if there's space
                if rect.width() > 50.0 && rect.height() > 20.0 {
                    let label_pos = rect.min + egui::vec2(2.0, 2.0);
                    painter.text(
                        label_pos,
                        egui::Align2::LEFT_TOP,
                        match element.element_type {
                            ElementType::Title { .. } => "T",
                            ElementType::Text => "txt",
                            ElementType::List { .. } => "L",
                            ElementType::Table => "tbl",
                            ElementType::Other => "?",
                        },
                        FontId::monospace(10.0),
                        color,
                    );
                }
            }
        }
    }
}

// Helper to draw terminal-style box with chrome borders
fn draw_terminal_box(ui: &mut egui::Ui, title: &str, add_contents: impl FnOnce(&mut egui::Ui)) {
    let frame = egui::Frame::none()
        .fill(TERM_BG)
        .stroke(Stroke::new(1.0, CHROME))
        .inner_margin(egui::Margin::same(5.0))
        .outer_margin(egui::Margin::same(1.0))
        .rounding(Rounding::same(2.0));
        
    frame.show(ui, |ui| {
        // Draw title with chrome accent
        ui.horizontal(|ui| {
            ui.label(RichText::new("▸").color(TERM_HIGHLIGHT).monospace());
            ui.label(RichText::new(title).color(CHROME).monospace().strong());
        });
        
        ui.add_space(5.0);
        add_contents(ui);
    });
}

impl eframe::App for Chonker5App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle deferred rendering
        if self.needs_render {
            self.needs_render = false;
            self.render_current_page(ctx);
        }
        
        // Set up terminal style
        let mut style = (*ctx.style()).clone();
        style.visuals.dark_mode = true;
        style.visuals.override_text_color = Some(TERM_FG);
        style.visuals.window_fill = TERM_BG;
        style.visuals.panel_fill = TERM_BG;
        style.visuals.extreme_bg_color = TERM_BG;
        style.visuals.widgets.noninteractive.bg_fill = TERM_BG;
        style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TERM_FG);
        style.visuals.widgets.inactive.bg_fill = TERM_BG;
        style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, CHROME);
        style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(30, 40, 50);
        style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, TERM_HIGHLIGHT);
        style.visuals.widgets.active.bg_fill = Color32::from_rgb(40, 50, 60);
        style.visuals.widgets.active.bg_stroke = Stroke::new(1.0, TERM_HIGHLIGHT);
        style.visuals.widgets.noninteractive.weak_bg_fill = TERM_BG;
        style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(20, 25, 30);
        style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, TERM_DIM);
        style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(30, 40, 45);
        style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, TERM_HIGHLIGHT);
        style.visuals.widgets.active.bg_fill = Color32::from_rgb(40, 50, 55);
        style.visuals.widgets.active.fg_stroke = Stroke::new(1.0, TERM_YELLOW);
        style.visuals.selection.bg_fill = Color32::from_rgb(0, 150, 140);
        style.visuals.selection.stroke = Stroke::new(1.0, TERM_HIGHLIGHT);
        ctx.set_style(style);
        
        // Check for async results
        if let Some(mut receiver) = self.vision_receiver.take() {
            if let Ok(result) = receiver.try_recv() {
                match result {
                    Ok(parsed_doc) => {
                        self.vision_result.parsed_doc = Some(parsed_doc.clone());
                        self.vision_result.content = render_vision_results(&parsed_doc);
                        self.vision_result.is_loading = false;
                        self.log("✅ Vision extraction completed");
                    }
                    Err(e) => {
                        self.vision_result.error = Some(e);
                        self.vision_result.is_loading = false;
                    }
                }
            } else {
                // Put the receiver back if no message yet
                self.vision_receiver = Some(receiver);
            }
        }
        
        // Main panel with terminal background
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(TERM_BG))
            .show(ctx, |ui| {
                // Compact header and controls
                ui.horizontal(|ui| {
                    // Display hamster image if available, otherwise emoji
                    if let Some(hamster) = &self.hamster_texture {
                        ui.image(egui::load::SizedTexture::new(hamster.id(), egui::vec2(32.0, 32.0)));
                    } else {
                        ui.label(
                            RichText::new("🐹")
                                .size(24.0)
                        );
                    }
                    
                    ui.label(
                        RichText::new("CHONKER 5")
                            .color(TERM_HIGHLIGHT)
                            .monospace()
                            .size(16.0)
                            .strong()
                    );
                    
                    ui.label(RichText::new("│").color(CHROME).monospace());
                    
                    if ui.button(RichText::new("[O] Open").color(TERM_FG).monospace().size(12.0)).clicked() {
                        self.open_file(ctx);
                    }
                    
                    ui.label(RichText::new("│").color(CHROME).monospace());
                    
                    // Navigation
                    ui.add_enabled_ui(self.pdf_path.is_some() && self.current_page > 0, |ui| {
                        if ui.button(RichText::new("←").color(TERM_FG).monospace().size(12.0)).clicked() {
                            self.current_page = self.current_page.saturating_sub(1);
                            self.render_current_page(ctx);
                        }
                    });
                    
                    if let Some(_) = &self.pdf_path {
                        ui.label(RichText::new(format!("{}/{}", self.current_page + 1, self.total_pages))
                            .color(TERM_FG)
                            .monospace()
                            .size(12.0));
                    }
                    
                    ui.add_enabled_ui(self.pdf_path.is_some() && self.current_page < self.total_pages - 1, |ui| {
                        if ui.button(RichText::new("→").color(TERM_FG).monospace().size(12.0)).clicked() {
                            self.current_page += 1;
                            self.render_current_page(ctx);
                        }
                    });
                    
                    ui.label(RichText::new("│").color(CHROME).monospace());
                    
                    // Zoom controls
                    ui.add_enabled_ui(self.pdf_path.is_some(), |ui| {
                        if ui.button(RichText::new("-").color(TERM_FG).monospace().size(12.0)).clicked() {
                            self.zoom_level = (self.zoom_level - 0.25).max(0.5);
                            self.render_current_page(ctx);
                        }
                        
                        ui.label(RichText::new(format!("{}%", (self.zoom_level * 100.0) as i32))
                            .color(TERM_FG)
                            .monospace()
                            .size(12.0));
                        
                        if ui.button(RichText::new("+").color(TERM_FG).monospace().size(12.0)).clicked() {
                            self.zoom_level = (self.zoom_level + 0.25).min(3.0);
                            self.render_current_page(ctx);
                        }
                    });
                    
                    ui.label(RichText::new("│").color(CHROME).monospace());
                    
                    // Page range
                    ui.label(RichText::new("R:").color(TERM_FG).monospace().size(12.0));
                    ui.add(egui::TextEdit::singleline(&mut self.page_range)
                        .desired_width(50.0)
                        .font(FontId::monospace(12.0)));
                    
                    ui.label(RichText::new("│").color(CHROME).monospace());
                    
                    // Extraction buttons
                    ui.add_enabled_ui(self.pdf_path.is_some(), |ui| {
                        if ui.button(RichText::new("[V]").color(TERM_FG).monospace().size(12.0)).clicked() {
                            self.extract_vision_data(ctx);
                            self.active_tab = ExtractionTab::Vision;
                        }
                        
                        if ui.button(RichText::new("[D]").color(TERM_FG).monospace().size(12.0)).clicked() {
                            self.extract_data_content();
                            self.active_tab = ExtractionTab::Data;
                        }
                        
                        ui.label(RichText::new("│").color(CHROME).monospace());
                        
                        // Bounding box toggle
                        let bbox_text = if self.show_bounding_boxes { "[B]✓" } else { "[B]" };
                        if ui.button(RichText::new(bbox_text).color(TERM_FG).monospace().size(12.0)).clicked() {
                            self.show_bounding_boxes = !self.show_bounding_boxes;
                        }
                    });
                });
                
                ui.add_space(2.0);
                
                // Main content area - Split pane view
                if self.pdf_path.is_some() {
                    let available_size = ui.available_size();
                    let available_width = available_size.x;
                    let available_height = available_size.y;
                    let separator_width = 8.0;
                    let padding = 20.0; // Increased padding to ensure right border is visible
                    let usable_width = available_width - (padding * 2.0);
                    let left_width = (usable_width - separator_width) * self.split_ratio;
                    let right_width = (usable_width - separator_width) * (1.0 - self.split_ratio);
                    
                    ui.horizontal_top(|ui| {
                        ui.add_space(padding); // Add left padding
                        // Left pane - PDF View
                        ui.allocate_ui_with_layout(
                            egui::vec2(left_width, available_height),
                            egui::Layout::left_to_right(egui::Align::TOP),
                            |ui| {
                                draw_terminal_box(ui, "PDF VIEW", |ui| {
                                    egui::ScrollArea::both()
                                        .auto_shrink([false; 2])
                                        .show(ui, |ui| {
                                            if let Some(texture) = &self.pdf_texture {
                                                let size = texture.size_vec2();
                                                let available_size = ui.available_size();
                                                
                                                // Calculate scaling to fit with zoom
                                                let base_scale = (available_size.x / size.x).min(available_size.y / size.y).min(1.0);
                                                let scale = base_scale * self.zoom_level;
                                                let display_size = size * scale;
                                                
                                                // Extract values needed for the closure
                                                let texture_id = texture.id();
                                                let current_zoom = self.zoom_level;
                                                let current_page = self.current_page;
                                                let total_pages = self.total_pages;
                                                
                                                // Center the image with trackpad support
                                                ui.vertical_centered(|ui| {
                                                    let response = ui.image(egui::load::SizedTexture::new(texture_id, display_size));
                                                    
                                                    // Handle trackpad zoom (pinch gesture)
                                                    if response.hovered() {
                                                        let zoom_delta = ui.input(|i| i.zoom_delta());
                                                        if zoom_delta != 1.0 {
                                                            self.zoom_level = (current_zoom * zoom_delta).clamp(0.5, 3.0);
                                                            self.needs_render = true;
                                                        }
                                                        
                                                        // Handle scroll for navigation
                                                        let scroll_delta = ui.input(|i| i.scroll_delta);
                                                        if scroll_delta.y.abs() > 10.0 {
                                                            if scroll_delta.y > 0.0 && current_page > 0 {
                                                                self.current_page = current_page - 1;
                                                                self.needs_render = true;
                                                            } else if scroll_delta.y < 0.0 && current_page < total_pages - 1 {
                                                                self.current_page = current_page + 1;
                                                                self.needs_render = true;
                                                            }
                                                        }
                                                    }
                                                });
                                            } else {
                                                ui.centered_and_justified(|ui| {
                                                    ui.label(RichText::new("Loading page...")
                                                        .color(TERM_DIM)
                                                        .monospace());
                                                });
                                            }
                                        });
                                });
                            }
                        );
                        
                        // Draggable separator with visual indicator
                        let separator_rect = ui.available_rect_before_wrap();
                        let separator_rect = egui::Rect::from_min_size(
                            separator_rect.min,
                            egui::vec2(separator_width, available_height)
                        );
                        let separator_response = ui.allocate_rect(separator_rect, egui::Sense::drag());
                        
                        // Visual feedback
                        let separator_color = if separator_response.hovered() {
                            TERM_HIGHLIGHT
                        } else {
                            CHROME
                        };
                        ui.painter().rect_filled(separator_response.rect, 0.0, separator_color);
                        
                        // Draw grip dots
                        let center = separator_response.rect.center();
                        for i in -2..=2 {
                            ui.painter().circle_filled(
                                egui::pos2(center.x, center.y + i as f32 * 10.0),
                                1.5,
                                TERM_DIM
                            );
                        }
                        
                        // Change cursor on hover
                        if separator_response.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
                        }
                        
                        if separator_response.dragged() {
                            let delta = separator_response.drag_delta().x;
                            self.split_ratio = (self.split_ratio + delta / available_width).clamp(0.2, 0.8);
                        }
                        
                        // Right pane - Extraction Results
                        ui.allocate_ui_with_layout(
                            egui::vec2(right_width, available_height),
                            egui::Layout::top_down(egui::Align::LEFT),
                            |ui| {
                                draw_terminal_box(ui, "EXTRACTION RESULTS", |ui| {
                                    // Tab buttons
                                    ui.horizontal(|ui| {
                                        let vision_label = if self.active_tab == ExtractionTab::Vision {
                                            RichText::new("[VISION]").color(TERM_HIGHLIGHT).monospace()
                                        } else {
                                            RichText::new(" Vision ").color(TERM_DIM).monospace()
                                        };
                                        if ui.button(vision_label).clicked() {
                                            self.active_tab = ExtractionTab::Vision;
                                        }
                                        
                                        ui.label(RichText::new("│").color(CHROME).monospace());
                                        
                                        let data_label = if self.active_tab == ExtractionTab::Data {
                                            RichText::new("[DATA]").color(TERM_HIGHLIGHT).monospace()
                                        } else {
                                            RichText::new(" Data ").color(TERM_DIM).monospace()
                                        };
                                        if ui.button(data_label).clicked() {
                                            self.active_tab = ExtractionTab::Data;
                                        }
                                    });
                                    
                                    ui.separator();
                                    
                                    // Content area
                                    egui::ScrollArea::both()
                                        .auto_shrink([false; 2])
                                        .show(ui, |ui| {
                                            match self.active_tab {
                                                ExtractionTab::Vision => {
                                                    if self.vision_result.is_loading {
                                                        ui.centered_and_justified(|ui| {
                                                            ui.spinner();
                                                            ui.label(RichText::new("\nExtracting with Vision AI...")
                                                                .color(TERM_FG)
                                                                .monospace());
                                                        });
                                                    } else if let Some(error) = &self.vision_result.error {
                                                        ui.label(RichText::new(error).color(TERM_ERROR).monospace());
                                                    } else if !self.vision_result.content.is_empty() {
                                                        ui.label(RichText::new(&self.vision_result.content)
                                                            .color(TERM_FG)
                                                            .monospace());
                                                    } else {
                                                        ui.centered_and_justified(|ui| {
                                                            ui.label(RichText::new("No vision extraction results yet\n\nPress [V] to extract")
                                                                .color(TERM_DIM)
                                                                .monospace());
                                                        });
                                                    }
                                                }
                                                ExtractionTab::Data => {
                                                    if self.data_result.is_loading {
                                                        ui.centered_and_justified(|ui| {
                                                            ui.spinner();
                                                            ui.label(RichText::new("\nExtracting data...")
                                                                .color(TERM_FG)
                                                                .monospace());
                                                        });
                                                    } else if let Some(error) = &self.data_result.error {
                                                        ui.label(RichText::new(error).color(TERM_ERROR).monospace());
                                                    } else if !self.data_result.content.is_empty() {
                                                        ui.label(RichText::new(&self.data_result.content)
                                                            .color(TERM_FG)
                                                            .monospace());
                                                    } else {
                                                        ui.centered_and_justified(|ui| {
                                                            ui.label(RichText::new("No data extraction results yet\n\nPress [D] to extract")
                                                                .color(TERM_DIM)
                                                                .monospace());
                                                        });
                                                    }
                                                }
                                                _ => {}
                                            }
                                        });
                                });
                            }
                        );
                        
                        ui.add_space(padding); // Add right padding
                    });
                } else {
                    // No PDF loaded - show welcome screen
                    draw_terminal_box(ui, "WELCOME", |ui| {
                        ui.centered_and_justified(|ui| {
                            ui.label(RichText::new("🐹 CHONKER 5\n\nPDF Viewer with AI Vision\n\nPress [O] to open a PDF file")
                                .color(TERM_FG)
                                .monospace()
                                .size(16.0));
                        });
                    });
                }
                
                // Collapsible log panel at bottom
                ui.add_space(5.0);
                egui::CollapsingHeader::new(RichText::new("▼ LOG").color(CHROME).monospace())
                    .default_open(false)
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical()
                            .max_height(60.0)
                            .auto_shrink([false; 2])
                            .show(ui, |ui| {
                                for message in &self.log_messages {
                                    ui.label(RichText::new(message).color(TERM_FG).monospace().size(10.0));
                                }
                            });
                    });
            });
    }
}

// Helper functions
fn parse_page_range(range_str: &str) -> Result<std::ops::Range<usize>, String> {
    if range_str.trim().is_empty() {
        return Err("Empty page range".to_string());
    }
    
    let parts: Vec<&str> = range_str.split('-').collect();
    if parts.len() != 2 {
        return Err("Invalid range format. Use format like '1-10'".to_string());
    }
    
    let start = parts[0].trim().parse::<usize>()
        .map_err(|_| format!("Invalid start page: {}", parts[0]))?;
    let end = parts[1].trim().parse::<usize>()
        .map_err(|_| format!("Invalid end page: {}", parts[1]))?;
    
    if start == 0 {
        return Err("Page numbers start at 1, not 0".to_string());
    }
    
    if start > end {
        return Err(format!("Start page {} is greater than end page {}", start, end));
    }
    
    // Convert to 0-based indexing
    Ok((start - 1)..end)
}

fn render_vision_results(doc: &ParsedDocument) -> String {
    let mut result = String::new();
    
    result.push_str("╔═══ VISION AI EXTRACTION RESULTS ═══╗\n\n");
    result.push_str(&format!("Total Pages: {}\n", doc.pages.len()));
    result.push_str(&format!("Total Elements: {}\n\n", 
        doc.pages.iter().map(|p| p.elements.len()).sum::<usize>()));
    
    for page in &doc.pages {
        result.push_str(&format!("╔═══ PAGE {} ═══╗\n", page.page_id.0 + 1));
        
        for element in &page.elements {
            // Show bounding box coordinates
            result.push_str(&format!("[{:>6.1},{:>6.1} | {:>6.1}x{:>6.1}] ",
                element.bbox.x0, element.bbox.y0,
                element.bbox.x1 - element.bbox.x0,
                element.bbox.y1 - element.bbox.y0));
            
            match &element.element_type {
                ElementType::Title { level } => {
                    let prefix = "═".repeat(3 - level.0.min(2));
                    result.push_str(&format!("{} {} {}\n", prefix, element.text, prefix));
                }
                ElementType::Text => {
                    result.push_str(&format!("│ {}\n", element.text));
                }
                ElementType::List { items } => {
                    result.push_str("┌─ LIST:\n");
                    for item in items {
                        result.push_str(&format!("│  • {}\n", item));
                    }
                    result.push_str("└─\n");
                }
                ElementType::Table { .. } => {
                    result.push_str("╔═ TABLE ═╗\n");
                    result.push_str(&format!("║ {} ║\n", element.text));
                    result.push_str("╚═════════╝\n");
                }
                ElementType::Other => {
                    result.push_str(&format!("  {}\n", element.text));
                }
            }
        }
        result.push_str("\n");
    }
    
    result
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "🐹 CHONKER 5 - PDF Viewer",
        options,
        Box::new(|cc| Box::new(Chonker5App::new(cc))),
    )
}