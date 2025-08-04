#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! fltk = { version = "1.4", features = ["fltk-bundled"] }
//! rfd = "0.15"
//! image = "0.25"
//! extractous = "0.3"
//! pdfium-render = { version = "0.8", features = ["thread_safe"] }
//! serde = { version = "1.0", features = ["derive"] }
//! serde_json = "1.0"
//! ordered-float = "4.2"
//! ```

use fltk::{
    app::{self, App, Scheme},
    button::Button,
    enums::{Color, Event, Font, FrameType, Key},
    frame::Frame,
    group::{Flex, Group, Scroll},
    misc::HelpView,
    prelude::*,
    text::{TextBuffer, TextDisplay},
    window::Window,
    image as fltk_image,
};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::process::Command;
use std::fs;
use extractous::Extractor;
use std::error::Error;
use serde::{Serialize, Deserialize};
use pdfium_render::prelude::*;
use std::collections::{HashMap, HashSet};
use ordered_float::OrderedFloat;

const WINDOW_WIDTH: i32 = 1200;
const WINDOW_HEIGHT: i32 = 800;
const TOP_BAR_HEIGHT: i32 = 60;
const LOG_HEIGHT: i32 = 100;

// Color scheme
const COLOR_TEAL: Color = Color::from_rgb(0x1A, 0xBC, 0x9C);
const COLOR_DARK_BG: Color = Color::from_rgb(0x52, 0x56, 0x59);
const COLOR_DARKER_BG: Color = Color::from_rgb(0x2D, 0x2F, 0x31);

// Table extraction structures
#[derive(Debug, Clone)]
struct TextFragment {
    text: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    font_size: f64,
    font_name: String,
    page_number: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Cell {
    content: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    row_span: usize,
    col_span: usize,
    row_index: usize,
    col_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Table {
    cells: Vec<Vec<Option<Cell>>>,
    rows: usize,
    cols: usize,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    page_number: u16,
}

// TextFragment and Line structs removed - not needed without pdfium functions

struct Chonker5App {
    app: App,
    window: Window,
    pdf_frame: Frame,
    status_label: Frame,
    zoom_label: Frame,
    page_label: Frame,
    log_display: TextDisplay,
    log_buffer: TextBuffer,
    prev_btn: Button,
    next_btn: Button,
    extract_btn: Button,
    structured_btn: Button,
    table_btn: Button,
    extracted_text_display: TextDisplay,
    extracted_text_buffer: TextBuffer,
    structured_html_view: HelpView,
    
    // PDF state
    pdf_path: Option<PathBuf>,
    current_page: usize,
    total_pages: usize,
    zoom_level: f32,
    
}

impl Chonker5App {
    fn new() -> Rc<RefCell<Self>> {
        let app = App::default().with_scheme(Scheme::Gtk);
        
        // Create main window
        let mut window = Window::new(100, 100, WINDOW_WIDTH, WINDOW_HEIGHT, "🐹 CHONKER 5 - PDF Viewer");
        window.set_color(COLOR_DARK_BG);
        window.make_resizable(true);
        
        // Create main vertical layout
        let mut main_flex = Flex::default()
            .with_size(WINDOW_WIDTH, WINDOW_HEIGHT)
            .column();
        
        // Top bar
        let mut top_bar = fltk::group::Group::default()
            .with_size(WINDOW_WIDTH, TOP_BAR_HEIGHT);
        top_bar.set_color(COLOR_TEAL);
        top_bar.set_frame(FrameType::FlatBox);
        
        // Position buttons manually with explicit positions
        let mut x_pos = 10;
        let y_pos = 10;
        
        let mut open_btn = Button::default()
            .with_pos(x_pos, y_pos)
            .with_size(100, 40)
            .with_label("Open");
        open_btn.set_color(Color::White);
        open_btn.set_label_color(Color::Black);
        open_btn.set_frame(FrameType::UpBox);
        open_btn.set_label_size(14);
        
        x_pos += 110;
        let mut prev_btn = Button::default()
            .with_pos(x_pos, y_pos)
            .with_size(80, 40)
            .with_label("◀ Prev");
        prev_btn.set_color(Color::from_rgb(0xDD, 0xDD, 0xDD));
        prev_btn.set_label_color(Color::from_rgb(0x66, 0x66, 0x66));
        prev_btn.set_frame(FrameType::UpBox);
        prev_btn.set_label_size(14);
        prev_btn.deactivate();
        
        x_pos += 90;
        let mut next_btn = Button::default()
            .with_pos(x_pos, y_pos)
            .with_size(80, 40)
            .with_label("Next ▶");
        next_btn.set_color(Color::from_rgb(0xDD, 0xDD, 0xDD));
        next_btn.set_label_color(Color::from_rgb(0x66, 0x66, 0x66));
        next_btn.set_frame(FrameType::UpBox);
        next_btn.set_label_size(14);
        next_btn.deactivate();
        
        x_pos += 90;
        let mut zoom_in_btn = Button::default()
            .with_pos(x_pos, y_pos)
            .with_size(100, 40)
            .with_label("Zoom In +");
        zoom_in_btn.set_color(Color::White);
        zoom_in_btn.set_label_color(Color::Black);
        zoom_in_btn.set_frame(FrameType::UpBox);
        zoom_in_btn.set_label_size(14);
        
        x_pos += 110;
        let mut zoom_out_btn = Button::default()
            .with_pos(x_pos, y_pos)
            .with_size(100, 40)
            .with_label("Zoom Out -");
        zoom_out_btn.set_color(Color::White);
        zoom_out_btn.set_label_color(Color::Black);
        zoom_out_btn.set_frame(FrameType::UpBox);
        zoom_out_btn.set_label_size(14);
        
        x_pos += 110;
        let mut fit_width_btn = Button::default()
            .with_pos(x_pos, y_pos)
            .with_size(100, 40)
            .with_label("Fit Width");
        fit_width_btn.set_color(Color::White);
        fit_width_btn.set_label_color(Color::Black);
        fit_width_btn.set_frame(FrameType::UpBox);
        fit_width_btn.set_label_size(14);
        
        x_pos += 110;
        let mut extract_btn = Button::default()
            .with_pos(x_pos, y_pos)
            .with_size(120, 40)
            .with_label("Extractous - plain text");
        extract_btn.set_color(Color::from_rgb(0x00, 0x8C, 0xBA)); // Blue color for distinction
        extract_btn.set_label_color(Color::White);
        extract_btn.set_frame(FrameType::UpBox);
        extract_btn.set_label_size(14);
        extract_btn.deactivate(); // Start disabled until PDF is loaded
        
        x_pos += 130;
        let mut structured_btn = Button::default()
            .with_pos(x_pos, y_pos)
            .with_size(140, 40)
            .with_label("Ferrules - HTML");
        structured_btn.set_color(Color::from_rgb(0x00, 0x8C, 0x3A)); // Green color for distinction
        structured_btn.set_label_color(Color::White);
        structured_btn.set_frame(FrameType::UpBox);
        structured_btn.set_label_size(14);
        structured_btn.deactivate(); // Start disabled until PDF is loaded
        
        x_pos += 150;
        let mut table_btn = Button::default()
            .with_pos(x_pos, y_pos)
            .with_size(160, 40)
            .with_label("Pdfium - All Content");
        table_btn.set_color(Color::from_rgb(0xE6, 0x7E, 0x22)); // Orange color for distinction
        table_btn.set_label_color(Color::White);
        table_btn.set_frame(FrameType::UpBox);
        table_btn.set_label_size(14);
        table_btn.deactivate(); // Start disabled until PDF is loaded
        
        x_pos += 170;
        let mut status_label = Frame::default()
            .with_pos(x_pos, y_pos)
            .with_size(300, 40)
            .with_label("Ready! Click 'Open' to load a PDF");
        status_label.set_label_color(Color::White);
        
        x_pos += 310;
        let mut zoom_label = Frame::default()
            .with_pos(x_pos, y_pos)
            .with_size(100, 40)
            .with_label("Zoom: 100%");
        zoom_label.set_label_color(Color::White);
        
        x_pos += 110;
        let mut page_label = Frame::default()
            .with_pos(x_pos, y_pos)
            .with_size(100, 40)
            .with_label("Page: 0/0");
        page_label.set_label_color(Color::White);
        
        top_bar.end();
        top_bar.redraw();
        main_flex.fixed(&mut top_bar, TOP_BAR_HEIGHT);
        
        // Create horizontal split for PDF and text panels
        let content_flex = Flex::default()
            .with_size(WINDOW_WIDTH, WINDOW_HEIGHT - TOP_BAR_HEIGHT - LOG_HEIGHT)
            .row();
        
        // Left pane: PDF viewing area with scroll
        let mut pdf_scroll = Scroll::default()
            .with_size(WINDOW_WIDTH / 2, WINDOW_HEIGHT - TOP_BAR_HEIGHT - LOG_HEIGHT);
        pdf_scroll.set_color(COLOR_DARK_BG);
        
        let mut pdf_frame = Frame::default()
            .with_size(WINDOW_WIDTH / 2 - 20, 1000);
        pdf_frame.set_frame(FrameType::FlatBox);
        pdf_frame.set_color(Color::White);
        pdf_frame.set_label("Click 'Open' to load a PDF");
        pdf_frame.set_label_color(Color::Black);
        
        pdf_scroll.end();
        
        // Right pane: Create a group to hold both text display and structured view
        let mut right_group = Group::default()
            .with_size(WINDOW_WIDTH / 2, WINDOW_HEIGHT - TOP_BAR_HEIGHT - LOG_HEIGHT);
        right_group.set_color(COLOR_DARKER_BG);
        
        // Text display for basic extraction
        let mut extracted_text_display = TextDisplay::default()
            .with_size(WINDOW_WIDTH / 2, WINDOW_HEIGHT - TOP_BAR_HEIGHT - LOG_HEIGHT);
        extracted_text_display.set_color(COLOR_DARKER_BG);
        extracted_text_display.set_text_color(Color::White);
        extracted_text_display.set_text_font(Font::Helvetica);
        extracted_text_display.set_text_size(14);
        extracted_text_display.set_frame(FrameType::FlatBox);
        extracted_text_display.wrap_mode(fltk::text::WrapMode::AtBounds, 0);
        
        let mut extracted_text_buffer = TextBuffer::default();
        extracted_text_buffer.set_text("PDF text will appear here after clicking 'Extract Text' button...");
        extracted_text_display.set_buffer(extracted_text_buffer.clone());
        
        // Structured view with HelpView for ferrules HTML rendering (has its own scrollbar)
        let mut structured_html_view = HelpView::default()
            .with_size(WINDOW_WIDTH / 2, WINDOW_HEIGHT - TOP_BAR_HEIGHT - LOG_HEIGHT);
        structured_html_view.set_frame(FrameType::FlatBox);
        structured_html_view.hide();
        
        right_group.end();
        
        content_flex.end();
        
        // Log area
        let mut log_display = TextDisplay::default()
            .with_size(WINDOW_WIDTH, LOG_HEIGHT);
        log_display.set_color(COLOR_DARKER_BG);
        log_display.set_text_color(COLOR_TEAL);
        log_display.set_text_font(Font::Courier);
        log_display.set_text_size(11);
        log_display.set_frame(FrameType::DownBox);
        
        let mut log_buffer = TextBuffer::default();
        log_buffer.append("🐹 CHONKER 5 Ready!\n");
        log_display.set_buffer(log_buffer.clone());
        
        main_flex.fixed(&mut log_display, LOG_HEIGHT);
        main_flex.end();
        
        window.resizable(&window);
        window.end();
        window.show();
        
        // Force redraw of all widgets
        window.redraw();
        app::redraw();
        
        log_buffer.append("🐹 CHONKER 5 Ready!\n");
        log_buffer.append("📌 Using MuPDF for PDF rendering + Extractous/Ferrules for text extraction\n");
        log_buffer.append("📌 Keyboard shortcuts: Cmd+O (Open), Cmd+P (Extract Text), ←/→ (Navigate), +/- (Zoom), F (Fit width)\n");
        log_buffer.append("📌 Extract Text: Basic text extraction | Structured Data: Perfect layout reconstruction\n");
        
        let app_state = Rc::new(RefCell::new(Self {
            app,
            window: window.clone(),
            pdf_frame,
            status_label,
            zoom_label,
            page_label,
            log_display,
            log_buffer,
            prev_btn: prev_btn.clone(),
            next_btn: next_btn.clone(),
            extract_btn: extract_btn.clone(),
            structured_btn: structured_btn.clone(),
            table_btn: table_btn.clone(),
            extracted_text_display: extracted_text_display.clone(),
            extracted_text_buffer,
            structured_html_view: structured_html_view.clone(),
            pdf_path: None,
            current_page: 0,
            total_pages: 0,
            zoom_level: 1.0,
        }));
        
        // Set up event handlers
        
        // Open button
        {
            let state = app_state.clone();
            open_btn.set_callback(move |_| {
                state.borrow_mut().open_file();
            });
        }
        
        // Navigation buttons
        {
            let state = app_state.clone();
            prev_btn.set_callback(move |_| {
                let mut state_ref = state.borrow_mut();
                
                // Always navigate PDF pages since structured view shows entire document
                state_ref.prev_page();
            });
        }
        
        {
            let state = app_state.clone();
            next_btn.set_callback(move |_| {
                let mut state_ref = state.borrow_mut();
                
                // Always navigate PDF pages since structured view shows entire document
                state_ref.next_page();
            });
        }
        
        // Zoom buttons
        {
            let state = app_state.clone();
            zoom_in_btn.set_callback(move |_| {
                state.borrow_mut().zoom_in();
            });
        }
        
        {
            let state = app_state.clone();
            zoom_out_btn.set_callback(move |_| {
                state.borrow_mut().zoom_out();
            });
        }
        
        {
            let state = app_state.clone();
            fit_width_btn.set_callback(move |_| {
                state.borrow_mut().fit_to_width();
            });
        }
        
        // Extract text button
        {
            let state = app_state.clone();
            extract_btn.set_callback(move |_| {
                state.borrow_mut().process_pdf();
            });
        }
        
        // Structured data button
        {
            let state = app_state.clone();
            structured_btn.set_callback(move |_| {
                state.borrow_mut().extract_structured_data();
            });
        }
        
        // Table extraction button
        {
            let state = app_state.clone();
            table_btn.set_callback(move |_| {
                state.borrow_mut().extract_tables();
            });
        }
        
        
        // Remove focus tracking event handlers to avoid borrow checker issues
        // Focus will be determined by mouse position when needed
        
        // Make window respond to close events
        window.set_callback(|_| {
            if app::event() == Event::Close {
                app::quit();
            }
        });
        
        // Keyboard shortcuts and window events
        {
            let state = app_state.clone();
            let mut win_clone = window.clone();
            window.handle(move |_, ev| match ev {
                Event::Show => {
                    win_clone.show();
                    win_clone.set_visible_focus();
                    true
                }
                Event::KeyDown => {
                    let key = app::event_key();
                    if app::is_event_command() && key == Key::from_char('o') {
                        state.borrow_mut().open_file();
                        true
                    } else if app::is_event_command() && key == Key::from_char('p') {
                        state.borrow_mut().process_pdf();
                        true
                    } else if key == Key::Left {
                        let mut state_ref = state.borrow_mut();
                        
                        // Always navigate PDF pages since structured view shows entire document
                        state_ref.prev_page();
                        true
                    } else if key == Key::Right {
                        let mut state_ref = state.borrow_mut();
                        
                        // Always navigate PDF pages since structured view shows entire document
                        state_ref.next_page();
                        true
                    } else if key == Key::from_char('+') || key == Key::from_char('=') {
                        state.borrow_mut().zoom_in();
                        true
                    } else if key == Key::from_char('-') {
                        state.borrow_mut().zoom_out();
                        true
                    } else if key == Key::from_char('f') {
                        state.borrow_mut().fit_to_width();
                        true
                    } else {
                        false
                    }
                }
                _ => false,
            });
        }
        
        app_state
    }
    
    fn log(&mut self, message: &str) {
        self.log_buffer.append(&format!("{}\n", message));
        // Scroll to bottom
        let len = self.log_buffer.length();
        self.log_display.scroll(len, 0);
    }
    
    fn open_file(&mut self) {
        self.log("📂 Opening file dialog...");
        
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("PDF", &["pdf"])
            .pick_file()
        {
            self.load_pdf(path);
        } else {
            self.log("❌ No file selected");
        }
    }
    
    fn process_pdf(&mut self) {
        if self.pdf_path.is_some() {
            self.log("🔄 Extracting text...");
            self.extract_current_page_text();
        } else {
            self.log("⚠️ No PDF loaded. Press Cmd+O to open a file first.");
        }
    }
    
    fn load_pdf(&mut self, path: PathBuf) {
        let filename = path.file_name().unwrap_or_default().to_string_lossy();
        self.log(&format!("📄 Loading: {}", filename));
        
        // Use mupdf info command to get page count with timeout
        match Command::new("timeout")
            .arg("5")  // 5 second timeout
            .arg("mutool")
            .arg("info")
            .arg(&path)
            .output()
        {
            Ok(output) => {
                let info = String::from_utf8_lossy(&output.stdout);
                
                // Parse page count from output
                let mut total_pages = 0;
                for line in info.lines() {
                    if line.contains("Pages:") {
                        if let Some(count_str) = line.split("Pages:").nth(1) {
                            if let Ok(count) = count_str.trim().parse::<usize>() {
                                total_pages = count;
                                break;
                            }
                        }
                    }
                }
                
                if total_pages > 0 {
                    self.pdf_path = Some(path);
                    self.total_pages = total_pages;
                    self.current_page = 0;
                    
                    self.log(&format!("✅ PDF loaded successfully: {} pages", self.total_pages));
                    self.update_status(&format!("Loaded! {} pages", self.total_pages));
                    
                    // Enable navigation buttons
                    if self.total_pages > 1 {
                        self.next_btn.activate();
                    }
                    
                    // Enable extract buttons
                    self.extract_btn.activate();
                    self.extract_btn.set_color(Color::from_rgb(0x00, 0x8C, 0xBA));
                    self.extract_btn.set_label_color(Color::White);
                    
                    self.structured_btn.activate();
                    self.structured_btn.set_color(Color::from_rgb(0x00, 0x8C, 0x3A));
                    self.structured_btn.set_label_color(Color::White);
                    
                    self.table_btn.activate();
                    self.table_btn.set_color(Color::from_rgb(0xE6, 0x7E, 0x22));
                    self.table_btn.set_label_color(Color::White);
                    
                    // Update UI
                    self.update_page_label();
                    
                    // Render the PDF page immediately
                    self.render_current_page();
                    
                    // But don't extract text yet - wait for Extract button
                    self.extracted_text_buffer.set_text("Click 'Extract Text' button or press Cmd+P to extract text from this PDF...");
                } else {
                    self.log("❌ Failed to parse PDF info");
                    self.update_status("Failed to parse PDF info");
                }
            }
            Err(e) => {
                let error_msg = format!("Failed to run mutool: {}", e);
                self.log(&format!("❌ {}", error_msg));
                self.update_status(&error_msg);
            }
        }
    }
    
    fn render_current_page(&mut self) {
        if let Some(pdf_path) = &self.pdf_path {
            // Create temp file for rendered page
            let temp_dir = std::env::temp_dir();
            let png_path = temp_dir.join(format!("chonker5_page_{}.png", self.current_page));
            
            // Calculate DPI based on zoom level
            let dpi = (150.0 * self.zoom_level) as i32;
            
            // Use mutool draw to render page to PNG with timeout
            let output = Command::new("timeout")
                .arg("5")  // 5 second timeout
                .arg("mutool")
                .arg("draw")
                .arg("-o")
                .arg(&png_path)
                .arg("-r")
                .arg(dpi.to_string())
                .arg("-F")
                .arg("png")
                .arg(&pdf_path)
                .arg((self.current_page + 1).to_string())
                .output();
            
            match output {
                Ok(_) => {
                    // Load the rendered PNG
                    if let Ok(img) = fltk_image::PngImage::load(&png_path) {
                        // Convert to RgbImage
                        let width = img.width();
                        let height = img.height();
                        
                        // Update the frame size and redraw
                        self.pdf_frame.set_size(width, height);
                        self.pdf_frame.set_image(Some(img));
                        self.pdf_frame.set_label("");
                        self.pdf_frame.redraw();
                        
                        self.log(&format!("✅ Page {} rendered", self.current_page + 1));
                    }
                    
                    // Clean up temp file
                    let _ = fs::remove_file(&png_path);
                }
                Err(e) => {
                    self.log(&format!("❌ Failed to render page: {}", e));
                }
            }
            
            // Don't extract text automatically - wait for Cmd+P
        }
    }
    
    fn extract_current_page_text(&mut self) {
        if let Some(pdf_path) = &self.pdf_path.clone() {
            // Show structured view and hide text display
            self.extracted_text_display.hide();
            self.structured_html_view.show();
            
            // Use extractous with XML output enabled
            let extractor = Extractor::new()
                .set_xml_output(true);
            
            // Call awake periodically to prevent beach ball
            app::awake();
            
            match extractor.extract_file_to_string(pdf_path.to_str().unwrap_or("")) {
                Ok((content, metadata)) => {
                    // Check if we got XML or text
                    if content.trim().starts_with("<?xml") || content.trim().starts_with("<") {
                        // We got XML content
                        let escaped_xml = content
                            .replace("&", "&amp;")
                            .replace("<", "&lt;")
                            .replace(">", "&gt;");
                        
                        let html = format!(r#"
                        <html>
                        <head>
                            <style>
                                body {{ font-family: Arial, sans-serif; margin: 20px; }}
                                h2 {{ color: #2c3e50; border-bottom: 2px solid #3498db; padding-bottom: 10px; }}
                                .metadata {{ background: #f8f9fa; padding: 10px; border-radius: 5px; margin-bottom: 20px; }}
                                .metadata dt {{ font-weight: bold; color: #495057; }}
                                .metadata dd {{ margin-left: 20px; margin-bottom: 5px; }}
                                pre {{ background: #f5f5f5; padding: 15px; border-radius: 5px; overflow-x: auto; }}
                                code {{ font-family: 'Courier New', monospace; font-size: 14px; }}
                            </style>
                        </head>
                        <body>
                            <h2>📄 Extractous XML Output</h2>
                            <div class="metadata">
                                <dl>
                                    <dt>Document:</dt>
                                    <dd>{}</dd>
                                    <dt>Method:</dt>
                                    <dd>Extractous XML extraction</dd>
                                </dl>
                            </div>
                            <pre><code>{}</code></pre>
                        </body>
                        </html>"#,
                        pdf_path.file_name().unwrap_or_default().to_string_lossy(),
                        escaped_xml);
                        
                        self.structured_html_view.set_value(&html);
                        self.log("✅ XML extracted with extractous");
                    } else {
                        // We got plain text, format it nicely
                        self.extract_text_fallback(pdf_path);
                    }
                }
                Err(e) => {
                    let html = format!(r#"
                    <html>
                    <head>
                        <style>
                            body {{ font-family: Arial, sans-serif; margin: 20px; }}
                            .error {{ color: #d9534f; background: #f2dede; padding: 15px; border-radius: 5px; }}
                        </style>
                    </head>
                    <body>
                        <h2>Extractous Error</h2>
                        <div class="error">
                            <p>Error extracting content: {}</p>
                        </div>
                    </body>
                    </html>"#, e);
                    self.structured_html_view.set_value(&html);
                    self.log(&format!("❌ Extractous error: {}", e));
                }
            }
            
            app::awake();
        }
    }
    
    fn extract_text_fallback(&mut self, pdf_path: &std::path::Path) {
        // Fallback to regular text extraction
        let extractor = Extractor::new();
        
        match extractor.extract_file_to_string(pdf_path.to_str().unwrap_or("")) {
            Ok((text, _metadata)) => {
                let escaped_text = text
                    .replace("&", "&amp;")
                    .replace("<", "&lt;")
                    .replace(">", "&gt;");
                
                let html = format!(r#"
                <html>
                <head>
                    <style>
                        body {{ font-family: Arial, sans-serif; margin: 20px; }}
                        h2 {{ color: #2c3e50; border-bottom: 2px solid #3498db; padding-bottom: 10px; }}
                        .note {{ background: #fff3cd; padding: 10px; border-radius: 5px; margin-bottom: 20px; }}
                        pre {{ background: #f5f5f5; padding: 15px; border-radius: 5px; overflow-x: auto; white-space: pre-wrap; }}
                    </style>
                </head>
                <body>
                    <h2>📄 Extractous Text Output</h2>
                    <div class="note">
                        <p>Note: XML extraction not available, showing plain text instead.</p>
                    </div>
                    <pre>{}</pre>
                </body>
                </html>"#, escaped_text);
                
                self.structured_html_view.set_value(&html);
                self.log("✅ Text extracted with extractous (fallback)");
            }
            Err(e) => {
                let html = format!(r#"
                <html>
                <head>
                    <style>
                        body {{ font-family: Arial, sans-serif; margin: 20px; }}
                        .error {{ color: #d9534f; background: #f2dede; padding: 15px; border-radius: 5px; }}
                    </style>
                </head>
                <body>
                    <h2>Extractous Error</h2>
                    <div class="error">
                        <p>Error extracting content: {}</p>
                    </div>
                </body>
                </html>"#, e);
                self.structured_html_view.set_value(&html);
                self.log(&format!("❌ Extractous error: {}", e));
            }
        }
    }
    
    fn extract_structured_data(&mut self) {
        if let Some(pdf_path) = &self.pdf_path.clone() {
            self.log("🔄 Extracting structured data with ferrules...");
            
            // Create temp dir for ferrules output
            let temp_dir = std::env::temp_dir();
            
            // Run ferrules command with HTML output
            let output = Command::new("timeout")
                .arg("30")  // 30 second timeout
                .arg("ferrules")
                .arg(pdf_path)
                .arg("-o")
                .arg(&temp_dir)
                .arg("--html")  // Request HTML output
                .output();
            
            match output {
                Ok(result) => {
                    if result.status.success() {
                        // Parse the output directory from stdout
                        let stdout = String::from_utf8_lossy(&result.stdout);
                        let results_dir = if let Some(line) = stdout.lines().find(|l| l.contains("Results saved in:")) {
                            if let Some(path_str) = line.split("Results saved in:").nth(1) {
                                PathBuf::from(path_str.trim())
                            } else {
                                // Fallback: try to guess the directory name
                                let stem = pdf_path.file_stem().unwrap_or_default().to_string_lossy();
                                temp_dir.join(format!("{}-results", stem))
                            }
                        } else {
                            // Fallback: try to guess the directory name
                            let stem = pdf_path.file_stem().unwrap_or_default().to_string_lossy();
                            temp_dir.join(format!("{}-results", stem))
                        };
                        
                        // Find the .html file in the results directory
                        let html_file = fs::read_dir(&results_dir)
                            .ok()
                            .and_then(|dir| {
                                dir.filter_map(|entry| entry.ok())
                                    .find(|entry| {
                                        entry.path().extension()
                                            .map(|ext| ext == "html")
                                            .unwrap_or(false)
                                    })
                                    .map(|entry| entry.path())
                            });
                        
                        // Read the HTML output from the results directory
                        match html_file {
                            Some(output_file) => {
                                match fs::read_to_string(&output_file) {
                                    Ok(content) => {
                                        // Display the HTML content
                                        self.structured_html_view.set_value(&content);
                                        
                                        // Hide text display, show structured HTML view
                                        self.extracted_text_display.hide();
                                        self.structured_html_view.show();
                                        
                                        self.log("✅ Structured data extracted with ferrules (HTML format)");
                                        
                                        // Clean up temp directory
                                        let _ = fs::remove_dir_all(&results_dir);
                                    }
                                    Err(e) => {
                                        self.extracted_text_buffer.set_text(&format!(
                                            "Error reading HTML file: {}", e
                                        ));
                                        self.log(&format!("❌ Failed to read HTML file: {}", e));
                                    }
                                }
                            }
                            None => {
                                // Try to show what's in the results directory
                                let entries = fs::read_dir(&results_dir)
                                    .map(|dir| {
                                        dir.filter_map(|e| e.ok())
                                            .map(|e| e.file_name().to_string_lossy().into_owned())
                                            .collect::<Vec<_>>()
                                            .join(", ")
                                    })
                                    .unwrap_or_else(|_| "Failed to read directory".to_string());
                                
                                self.extracted_text_buffer.set_text(&format!(
                                    "No HTML file found\nResults dir: {}\nFiles found: {}", 
                                    results_dir.display(), entries
                                ));
                                self.log("❌ No HTML file found in ferrules output");
                            }
                        }
                    } else {
                        let error_msg = String::from_utf8_lossy(&result.stderr);
                        self.extracted_text_buffer.set_text(&format!("Ferrules error: {}", error_msg));
                        self.log(&format!("❌ Ferrules failed: {}", error_msg));
                    }
                }
                Err(e) => {
                    self.extracted_text_buffer.set_text(&format!("Failed to run ferrules: {}", e));
                    self.log(&format!("❌ Failed to run ferrules: {}", e));
                }
            }
            
            app::awake();
        } else {
            self.log("⚠️ No PDF loaded. Press Cmd+O to open a file first.");
        }
    }
    
    fn extract_tables(&mut self) {
        if let Some(pdf_path) = &self.pdf_path.clone() {
            self.log("🔄 Extracting all content with pdfium-render...");
            
            // Show structured view and hide text display
            self.extracted_text_display.hide();
            self.structured_html_view.show();
            
            match self.pdfium_html_extraction(pdf_path) {
                Ok(html) => {
                    self.structured_html_view.set_value(&html);
                    self.log("✅ Content extraction completed");
                }
                Err(e) => {
                    let error_html = format!(r#"
                    <html>
                    <head>
                        <style>
                            body {{ font-family: Arial, sans-serif; margin: 20px; }}
                            h2 {{ color: #d9534f; }}
                            .error {{ background: #f2dede; padding: 15px; border-radius: 5px; color: #a94442; }}
                            .suggestions {{ margin-top: 20px; }}
                            .suggestions li {{ margin-bottom: 10px; }}
                        </style>
                    </head>
                    <body>
                        <h2>⚠️ Pdfium Extraction Error</h2>
                        <div class="error">
                            <p>Failed to extract content: {}</p>
                        </div>
                        <div class="suggestions">
                            <h3>This might be due to:</h3>
                            <ul>
                                <li>PDF format incompatibility</li>
                                <li>No structured content in the document</li>
                                <li>Missing pdfium library</li>
                            </ul>
                            <p>Try using 'Ferrules - HTML' for structured document layout instead.</p>
                        </div>
                    </body>
                    </html>"#, e);
                    self.structured_html_view.set_value(&error_html);
                    self.log(&format!("❌ Pdfium extraction error: {}", e));
                }
            }
            
            app::awake();
        } else {
            self.log("⚠️ No PDF loaded. Press Cmd+O to open a file first.");
        }
    }
    
    fn pdfium_html_extraction(&self, pdf_path: &std::path::Path) -> Result<String, Box<dyn Error>> {
        let mut html = String::from(r#"
        <html>
        <head>
            <style>
                body { font-family: Arial, sans-serif; margin: 20px; line-height: 1.6; }
                h2 { color: #2c3e50; border-bottom: 2px solid #e67e22; padding-bottom: 10px; }
                h3 { color: #34495e; margin-top: 20px; }
                .metadata { background: #f8f9fa; padding: 15px; border-radius: 5px; margin-bottom: 20px; }
                .metadata dt { font-weight: bold; color: #495057; display: inline-block; width: 120px; }
                .metadata dd { display: inline; margin-left: 10px; }
                .page { margin-bottom: 30px; border: 1px solid #dee2e6; padding: 20px; border-radius: 5px; }
                .page-header { background: #e9ecef; margin: -20px -20px 15px -20px; padding: 10px 20px; border-radius: 5px 5px 0 0; }
                .table-container { background: #f5f5f5; padding: 15px; border-radius: 5px; margin: 15px 0; overflow-x: auto; }
                pre { white-space: pre-wrap; word-wrap: break-word; }
                .no-tables { color: #6c757d; font-style: italic; }
                .stats { background: #e7f3ff; padding: 10px; border-radius: 5px; margin-top: 20px; }
            </style>
        </head>
        <body>
            <h2>📊 Pdfium Full Content Extraction</h2>
        "#);
        
        // Initialize pdfium
        let pdfium = Pdfium::new(
            Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./lib/"))?
        );
        
        // Load the PDF document
        let document = pdfium.load_pdf_from_file(pdf_path, None)?;
        let total_pages = document.pages().len();
        
        html.push_str(&format!(r#"
            <div class="metadata">
                <dl>
                    <dt>Document:</dt>
                    <dd>{}</dd>
                </dl>
                <dl>
                    <dt>Total Pages:</dt>
                    <dd>{}</dd>
                </dl>
                <dl>
                    <dt>Method:</dt>
                    <dd>Pdfium-render extraction</dd>
                </dl>
            </div>
        "#, 
        pdf_path.file_name().unwrap_or_default().to_string_lossy(),
        total_pages));
        
        let mut total_tables_found = 0;
        let mut total_text_length = 0;
        
        // Process each page
        for (page_index, page) in document.pages().iter().enumerate() {
            let page_number = page_index + 1;
            
            html.push_str(&format!(r#"
            <div class="page">
                <div class="page-header">
                    <h3>📄 Page {}</h3>
                </div>
            "#, page_number));
            
            // Extract all text from the page
            let text_page = page.text()?;
            let char_count = text_page.chars().len();
            
            let mut page_text = String::new();
            for index in 0..char_count {
                if let Ok(character) = text_page.chars().get(index) {
                    if let Some(ch) = character.unicode_char() {
                        page_text.push(ch);
                    }
                }
            }
            
            total_text_length += page_text.len();
            
            // Detect tables
            let detected_tables = self.detect_simple_tables(&page_text);
            
            if !detected_tables.is_empty() {
                html.push_str(&format!("<h4>Found {} table(s):</h4>", detected_tables.len()));
                
                for (table_idx, table_text) in detected_tables.iter().enumerate() {
                    total_tables_found += 1;
                    html.push_str(&format!(r#"
                    <div class="table-container">
                        <h5>Table {} (Page {}):</h5>
                        <pre>{}</pre>
                    </div>
                    "#, 
                    table_idx + 1, 
                    page_number,
                    table_text.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")));
                }
            } else {
                html.push_str(r#"<p class="no-tables">No tables detected on this page.</p>"#);
            }
            
            // Show full page text - no truncation
            if !page_text.trim().is_empty() {
                html.push_str(r#"<h4>Full Page Text:</h4><div style="background: #f8f9fa; padding: 15px; border-radius: 5px; margin-top: 10px;"><pre style="font-size: 12px; white-space: pre-wrap;">"#);
                html.push_str(&page_text.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;"));
                html.push_str("</pre></div>");
            }
            
            html.push_str("</div>");
        }
        
        // Add summary statistics
        html.push_str(&format!(r#"
        <div class="stats">
            <h3>📊 Extraction Summary:</h3>
            <ul>
                <li>Total tables found: {}</li>
                <li>Total text extracted: {} characters</li>
                <li>Pages processed: {}</li>
            </ul>
        </div>
        "#, total_tables_found, total_text_length, total_pages));
        
        html.push_str("</body></html>");
        
        Ok(html)
    }
    
    fn simple_table_extraction(&self, pdf_path: &std::path::Path) -> Result<String, Box<dyn Error>> {
        // Use pdfium-render with static linking
        self.pdfium_table_extraction(pdf_path)
    }
    
    fn pdfium_table_extraction(&self, pdf_path: &std::path::Path) -> Result<String, Box<dyn Error>> {
        let mut output = String::new();
        
        // Initialize pdfium - try to load from lib directory
        let pdfium = Pdfium::new(
            Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./lib/"))?
        );
        
        // Load the PDF document
        let document = pdfium.load_pdf_from_file(pdf_path, None)?;
        
        output.push_str(&format!("📊 PDFIUM TABLE EXTRACTION RESULTS\n"));
        output.push_str(&format!("Document: {}\n", pdf_path.file_name().unwrap_or_default().to_string_lossy()));
        output.push_str(&format!("Total Pages: {}\n", document.pages().len()));
        output.push_str(&format!("Method: Pdfium-render text extraction\n\n"));
        
        let mut tables_found = 0;
        
        // Process each page
        for (page_index, page) in document.pages().iter().enumerate() {
            let page_number = page_index + 1;
            
            // Extract all text from the page
            let text_page = page.text()?;
            let char_count = text_page.chars().len();
            
            let mut page_text = String::new();
            for index in 0..char_count {
                if let Ok(character) = text_page.chars().get(index) {
                    if let Some(ch) = character.unicode_char() {
                        page_text.push(ch);
                    }
                }
            }
            
            // Simple table detection: look for patterns that suggest tabular data
            let detected_tables = self.detect_simple_tables(&page_text);
            
            if !detected_tables.is_empty() {
                output.push_str(&format!("📄 PAGE {} TABLES:\n", page_number));
                output.push_str(&format!("Found {} potential table(s)\n\n", detected_tables.len()));
                
                for (table_idx, table_text) in detected_tables.iter().enumerate() {
                    tables_found += 1;
                    output.push_str(&format!("🔢 Table {} (Page {}):\n", table_idx + 1, page_number));
                    output.push_str("```\n");
                    output.push_str(table_text);
                    output.push_str("\n```\n\n");
                }
            }
        }
        
        if tables_found == 0 {
            output.push_str("ℹ️ No tables detected in this document.\n\n");
            output.push_str("This could mean:\n");
            output.push_str("• The document contains no tabular data\n");
            output.push_str("• Tables are embedded as images\n");
            output.push_str("• Tables don't follow recognizable patterns\n\n");
            output.push_str("Try using 'Ferrules - HTML' for better structure detection.\n");
        } else {
            output.push_str(&format!("✅ Successfully detected {} table(s) across all pages.\n", tables_found));
        }
        
        Ok(output)
    }
    
    fn extract_text_fragments(&self, page: &PdfPage, page_number: u16) -> Result<Vec<TextFragment>, Box<dyn Error>> {
        let mut fragments = Vec::new();
        let text_page = page.text()?;
        let chars = text_page.chars();
        let char_count = chars.len();
        
        let mut current_fragment = String::new();
        let mut fragment_start_x = 0.0;
        let mut fragment_start_y = 0.0;
        let mut fragment_font_size = 0.0;
        let mut fragment_font_name = String::new();
        let mut last_x = 0.0;
        let mut last_y = 0.0;
        
        let vertical_tolerance = 3.0;
        
        for index in 0..char_count {
            if let Ok(character) = chars.get(index) {
                // Get character position 
                if let Ok(bounds_result) = character.loose_bounds() {
                    let font_size = 10.0; // Default font size
                    let font_name = String::from("Unknown"); // Default font name
                    
                    // Convert PdfPoints to f64
                    let bounds_left = bounds_result.left().value as f64;
                    let bounds_top = bounds_result.top().value as f64;
                    let bounds_right = bounds_result.right().value as f64;
                    
                    let is_new_fragment = current_fragment.is_empty() ||
                        (bounds_top - last_y).abs() > vertical_tolerance ||
                        bounds_left - last_x > font_size * 0.3 ||
                        font_size != fragment_font_size ||
                        font_name != fragment_font_name;
                    
                    if is_new_fragment && !current_fragment.is_empty() {
                        fragments.push(TextFragment {
                            text: current_fragment.clone(),
                            x: fragment_start_x,
                            y: fragment_start_y,
                            width: last_x - fragment_start_x,
                            height: fragment_font_size,
                            font_size: fragment_font_size,
                            font_name: fragment_font_name.clone(),
                            page_number,
                        });
                        current_fragment.clear();
                    }
                    
                    if current_fragment.is_empty() {
                        fragment_start_x = bounds_left;
                        fragment_start_y = bounds_top;
                        fragment_font_size = font_size;
                        fragment_font_name = font_name.clone();
                    }
                    
                    if let Some(ch) = character.unicode_char() {
                        current_fragment.push(ch);
                    }
                    last_x = bounds_right;
                    last_y = bounds_top;
                }
            }
        }
        
        if !current_fragment.is_empty() {
            fragments.push(TextFragment {
                text: current_fragment,
                x: fragment_start_x,
                y: fragment_start_y,
                width: last_x - fragment_start_x,
                height: fragment_font_size,
                font_size: fragment_font_size,
                font_name: fragment_font_name,
                page_number,
            });
        }
        
        Ok(fragments)
    }
    
    fn detect_simple_tables(&self, text: &str) -> Vec<String> {
        let mut tables = Vec::new();
        let lines: Vec<&str> = text.lines().collect();
        
        let mut current_table = Vec::new();
        let mut in_table = false;
        
        for line in lines {
            // Simple heuristics for table detection
            let trimmed = line.trim();
            
            // Skip empty lines
            if trimmed.is_empty() {
                if in_table && !current_table.is_empty() {
                    // End current table
                    let table_text = current_table.join("\n");
                    if self.looks_like_table(&table_text) {
                        tables.push(table_text);
                    }
                    current_table.clear();
                    in_table = false;
                }
                continue;
            }
            
            // Check if line looks like it could be part of a table
            if self.line_looks_tabular(trimmed) {
                if !in_table {
                    in_table = true;
                    current_table.clear();
                }
                current_table.push(trimmed.to_string());
            } else if in_table {
                // End current table
                let table_text = current_table.join("\n");
                if self.looks_like_table(&table_text) {
                    tables.push(table_text);
                }
                current_table.clear();
                in_table = false;
            }
        }
        
        // Handle table at end of text
        if in_table && !current_table.is_empty() {
            let table_text = current_table.join("\n");
            if self.looks_like_table(&table_text) {
                tables.push(table_text);
            }
        }
        
        tables
    }
    
    fn line_looks_tabular(&self, line: &str) -> bool {
        // Very strict table detection - only actual data tables
        let tab_count = line.matches('\t').count();
        let pipe_count = line.matches('|').count();
        
        // Only consider lines with actual tab or pipe separators
        if tab_count >= 2 || pipe_count >= 2 {
            return true;
        }
        
        // Look for numeric data patterns that strongly suggest tables
        let words: Vec<&str> = line.split_whitespace().collect();
        if words.len() < 3 {
            return false;
        }
        
        // Count numeric tokens (numbers, currency, percentages)
        let numeric_count = words.iter().filter(|w| {
            w.chars().any(|c| c.is_numeric()) || 
            w.contains('$') || w.contains('%') || w.contains('.')
        }).count();
        
        // At least half the tokens should be numeric for it to be a data table
        if numeric_count >= words.len() / 2 && words.len() >= 4 {
            // Additional check: consistent spacing pattern
            let double_space_count = line.matches("  ").count();
            if double_space_count >= 2 {
                return true;
            }
        }
        
        false
    }
    
    fn looks_like_table(&self, text: &str) -> bool {
        let lines: Vec<&str> = text.lines().collect();
        
        // Must have at least 2 rows to be a table
        if lines.len() < 2 || lines.len() > 50 {
            return false;
        }
        
        // Check if most lines have similar structure
        let tabular_lines = lines.iter()
            .filter(|line| self.line_looks_tabular(line.trim()))
            .count();
        
        // At least 90% of lines should look tabular for very strict detection
        tabular_lines as f64 / lines.len() as f64 >= 0.9
    }
    
    fn prev_page(&mut self) {
        if self.current_page > 0 {
            self.current_page -= 1;
            self.update_page_label();
            self.update_nav_buttons();
            self.log(&format!("◀ Page {}", self.current_page + 1));
            
            // Render the new page
            self.render_current_page();
            
            // Clear extracted text - user needs to extract again
            self.extracted_text_buffer.set_text("Click 'Extract Text' button or press Cmd+P to extract text from this page...");
        }
    }
    
    fn next_page(&mut self) {
        if self.current_page < self.total_pages - 1 {
            self.current_page += 1;
            self.update_page_label();
            self.update_nav_buttons();
            self.log(&format!("▶ Page {}", self.current_page + 1));
            
            // Render the new page
            self.render_current_page();
            
            // Clear extracted text - user needs to extract again
            self.extracted_text_buffer.set_text("Click 'Extract Text' button or press Cmd+P to extract text from this page...");
        }
    }
    
    fn zoom_in(&mut self) {
        self.zoom_level = (self.zoom_level * 1.2).min(4.0);
        self.update_zoom_label();
        self.render_current_page();
        self.log(&format!("🔍+ Zoom: {}%", (self.zoom_level * 100.0) as i32));
    }
    
    fn zoom_out(&mut self) {
        self.zoom_level = (self.zoom_level / 1.2).max(0.25);
        self.update_zoom_label();
        self.render_current_page();
        self.log(&format!("🔍- Zoom: {}%", (self.zoom_level * 100.0) as i32));
    }
    
    fn fit_to_width(&mut self) {
        // Calculate zoom to fit width (now using half window width due to split pane)
        let viewport_width = self.window.width() / 2 - 40;
        let base_width = 800.0;
        
        self.zoom_level = (viewport_width as f32 / base_width / 2.0).clamp(0.25, 4.0);
        self.update_zoom_label();
        self.render_current_page();
        self.log(&format!("📐 Fit to width - Zoom: {}%", (self.zoom_level * 100.0) as i32));
    }
    
    fn update_status(&mut self, text: &str) {
        self.status_label.set_label(text);
    }
    
    fn update_zoom_label(&mut self) {
        self.zoom_label.set_label(&format!("Zoom: {}%", (self.zoom_level * 100.0) as i32));
    }
    
    fn update_page_label(&mut self) {
        if self.total_pages > 0 {
            self.page_label.set_label(&format!("Page: {}/{}", self.current_page + 1, self.total_pages));
        } else {
            self.page_label.set_label("Page: 0/0");
        }
    }
    
    fn update_nav_buttons(&mut self) {
        if self.current_page > 0 {
            self.prev_btn.activate();
            self.prev_btn.set_color(Color::White);
            self.prev_btn.set_label_color(Color::Black);
        } else {
            self.prev_btn.deactivate();
            self.prev_btn.set_color(Color::from_rgb(0xDD, 0xDD, 0xDD));
            self.prev_btn.set_label_color(Color::from_rgb(0x66, 0x66, 0x66));
        }
        
        if self.current_page < self.total_pages - 1 {
            self.next_btn.activate();
            self.next_btn.set_color(Color::White);
            self.next_btn.set_label_color(Color::Black);
        } else {
            self.next_btn.deactivate();
            self.next_btn.set_color(Color::from_rgb(0xDD, 0xDD, 0xDD));
            self.next_btn.set_label_color(Color::from_rgb(0x66, 0x66, 0x66));
        }
    }
    
    fn run(app_state: Rc<RefCell<Self>>) {
        let app = app_state.borrow().app.clone();
        app.run().unwrap();
    }
    
    // Advanced table detection methods
    fn detect_tables(&self, text_fragments: Vec<TextFragment>, page_number: u16) -> Vec<Table> {
        let mut tables = Vec::new();
        let row_groups = self.group_fragments_by_row(&text_fragments);
        let table_regions = self.find_table_regions(&row_groups);
        
        for region in table_regions {
            if let Some(table) = self.extract_table_from_region(region, page_number) {
                tables.push(table);
            }
        }
        
        tables
    }
    
    fn group_fragments_by_row(&self, fragments: &[TextFragment]) -> Vec<Vec<TextFragment>> {
        let vertical_tolerance = 3.0;
        let mut row_map: HashMap<OrderedFloat<f64>, Vec<TextFragment>> = HashMap::new();
        
        for fragment in fragments {
            let y_key = OrderedFloat((fragment.y / vertical_tolerance).round() * vertical_tolerance);
            row_map.entry(y_key).or_insert_with(Vec::new).push(fragment.clone());
        }
        
        let mut rows: Vec<(OrderedFloat<f64>, Vec<TextFragment>)> = row_map.into_iter().collect();
        rows.sort_by_key(|(y, _)| *y);
        
        rows.into_iter().map(|(_, mut fragments)| {
            fragments.sort_by(|a, b| OrderedFloat(a.x).cmp(&OrderedFloat(b.x)));
            fragments
        }).collect()
    }
    
    fn find_table_regions(&self, rows: &[Vec<TextFragment>]) -> Vec<Vec<Vec<TextFragment>>> {
        let mut regions = Vec::new();
        let mut current_region = Vec::new();
        let mut in_table = false;
        let min_table_rows = 2;
        let _min_table_cols = 2;
        
        for (i, row) in rows.iter().enumerate() {
            if self.is_table_row(row, i, rows) {
                if !in_table {
                    in_table = true;
                    current_region.clear();
                }
                current_region.push(row.clone());
            } else if in_table {
                if current_region.len() >= min_table_rows {
                    regions.push(current_region.clone());
                }
                current_region.clear();
                in_table = false;
            }
        }
        
        if in_table && current_region.len() >= min_table_rows {
            regions.push(current_region);
        }
        
        regions
    }
    
    fn is_table_row(&self, row: &[TextFragment], row_index: usize, all_rows: &[Vec<TextFragment>]) -> bool {
        let min_table_cols = 2;
        let horizontal_tolerance = 3.0;
        
        if row.len() < min_table_cols {
            return false;
        }
        
        // Check alignment with previous and next rows
        if row_index > 0 && row_index < all_rows.len() - 1 {
            let prev_row = &all_rows[row_index - 1];
            let next_row = &all_rows[row_index + 1];
            
            let mut aligned_count = 0;
            for fragment in row {
                let has_prev_aligned = prev_row.iter().any(|pf| 
                    (pf.x - fragment.x).abs() < horizontal_tolerance
                );
                let has_next_aligned = next_row.iter().any(|nf| 
                    (nf.x - fragment.x).abs() < horizontal_tolerance
                );
                
                if has_prev_aligned || has_next_aligned {
                    aligned_count += 1;
                }
            }
            
            // At least half the columns should be aligned
            if aligned_count < row.len() / 2 {
                return false;
            }
        }
        
        true
    }
    
    fn extract_table_from_region(&self, region: Vec<Vec<TextFragment>>, page_number: u16) -> Option<Table> {
        let min_table_rows = 2;
        let min_table_cols = 2;
        let horizontal_tolerance = 3.0;
        
        if region.len() < min_table_rows {
            return None;
        }
        
        // Find all unique column positions
        let mut all_x_positions = HashSet::new();
        for row in &region {
            for fragment in row {
                all_x_positions.insert(OrderedFloat(fragment.x));
            }
        }
        
        let mut column_positions: Vec<f64> = all_x_positions.into_iter()
            .map(|x| x.0)
            .collect();
        column_positions.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        // Merge close column positions
        let mut merged_positions = Vec::new();
        let mut last_pos = None;
        for pos in column_positions {
            if let Some(last) = last_pos {
                if pos - last > horizontal_tolerance {
                    merged_positions.push(pos);
                }
            } else {
                merged_positions.push(pos);
            }
            last_pos = Some(pos);
        }
        column_positions = merged_positions;
        
        if column_positions.len() < min_table_cols {
            return None;
        }
        
        let rows = region.len();
        let cols = column_positions.len();
        let mut cells: Vec<Vec<Option<Cell>>> = vec![vec![None; cols]; rows];
        
        // Fill cells based on fragment positions
        for (row_idx, row_fragments) in region.iter().enumerate() {
            for fragment in row_fragments {
                let col_idx = column_positions.iter()
                    .position(|&col_x| (fragment.x - col_x).abs() < horizontal_tolerance)
                    .unwrap_or_else(|| {
                        column_positions.iter()
                            .enumerate()
                            .min_by_key(|(_, &col_x)| OrderedFloat((fragment.x - col_x).abs()))
                            .map(|(idx, _)| idx)
                            .unwrap_or(0)
                    });
                
                if col_idx < cols {
                    if let Some(ref mut existing_cell) = cells[row_idx][col_idx] {
                        existing_cell.content.push(' ');
                        existing_cell.content.push_str(&fragment.text);
                        existing_cell.width = existing_cell.width.max(fragment.x + fragment.width - existing_cell.x);
                    } else {
                        cells[row_idx][col_idx] = Some(Cell {
                            content: fragment.text.clone(),
                            x: fragment.x,
                            y: fragment.y,
                            width: fragment.width,
                            height: fragment.height,
                            row_span: 1,
                            col_span: 1,
                            row_index: row_idx,
                            col_index: col_idx,
                        });
                    }
                }
            }
        }
        
        // Calculate table bounds
        let min_x = region.iter()
            .flat_map(|row| row.iter().map(|f| f.x))
            .min_by_key(|&x| OrderedFloat(x))
            .unwrap_or(0.0);
        
        let max_x = region.iter()
            .flat_map(|row| row.iter().map(|f| f.x + f.width))
            .max_by_key(|&x| OrderedFloat(x))
            .unwrap_or(0.0);
        
        let min_y = region.first()
            .and_then(|row| row.first())
            .map(|f| f.y)
            .unwrap_or(0.0);
        
        let max_y = region.last()
            .and_then(|row| row.first())
            .map(|f| f.y + f.height)
            .unwrap_or(0.0);
        
        Some(Table {
            cells, rows, cols,
            x: min_x,
            y: min_y,
            width: max_x - min_x,
            height: max_y - min_y,
            page_number,
        })
    }
    
    fn export_to_markdown(&self, table: &Table) -> String {
        let mut markdown = String::new();
        
        if !table.cells.is_empty() {
            // Header row
            markdown.push('|');
            for cell in &table.cells[0] {
                markdown.push(' ');
                markdown.push_str(&cell.as_ref().map(|c| c.content.as_str()).unwrap_or(""));
                markdown.push_str(" |");
            }
            markdown.push('\n');
            
            // Separator
            markdown.push('|');
            for _ in 0..table.cols {
                markdown.push_str(" --- |");
            }
            markdown.push('\n');
            
            // Data rows
            for row_idx in 1..table.rows {
                markdown.push('|');
                for cell in &table.cells[row_idx] {
                    markdown.push(' ');
                    markdown.push_str(&cell.as_ref().map(|c| c.content.as_str()).unwrap_or(""));
                    markdown.push_str(" |");
                }
                markdown.push('\n');
            }
        }
        
        markdown
    }
}


fn main() {
    let app_state = Chonker5App::new();
    Chonker5App::run(app_state);
}