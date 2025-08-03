#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! fltk = { version = "1.4", features = ["fltk-bundled"] }
//! rfd = "0.15"
//! image = "0.25"
//! extractous = "0.3"
//! serde = { version = "1.0", features = ["derive"] }
//! serde_json = "1.0"
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

const WINDOW_WIDTH: i32 = 1200;
const WINDOW_HEIGHT: i32 = 800;
const TOP_BAR_HEIGHT: i32 = 60;
const LOG_HEIGHT: i32 = 100;

// Color scheme
const COLOR_TEAL: Color = Color::from_rgb(0x1A, 0xBC, 0x9C);
const COLOR_DARK_BG: Color = Color::from_rgb(0x52, 0x56, 0x59);
const COLOR_DARKER_BG: Color = Color::from_rgb(0x2D, 0x2F, 0x31);

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
            .with_label("Extract Text");
        extract_btn.set_color(Color::from_rgb(0x00, 0x8C, 0xBA)); // Blue color for distinction
        extract_btn.set_label_color(Color::White);
        extract_btn.set_frame(FrameType::UpBox);
        extract_btn.set_label_size(14);
        extract_btn.deactivate(); // Start disabled until PDF is loaded
        
        x_pos += 130;
        let mut structured_btn = Button::default()
            .with_pos(x_pos, y_pos)
            .with_size(140, 40)
            .with_label("Structured Data");
        structured_btn.set_color(Color::from_rgb(0x00, 0x8C, 0x3A)); // Green color for distinction
        structured_btn.set_label_color(Color::White);
        structured_btn.set_frame(FrameType::UpBox);
        structured_btn.set_label_size(14);
        structured_btn.deactivate(); // Start disabled until PDF is loaded
        
        x_pos += 160;
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
        if let Some(pdf_path) = &self.pdf_path {
            // Show text display and hide structured view
            self.structured_html_view.hide();
            self.extracted_text_display.show();
            
            // Use extractous directly to extract text
            let extractor = Extractor::new();
            
            // Call awake periodically to prevent beach ball
            app::awake();
            
            match extractor.extract_file_to_string(pdf_path.to_str().unwrap_or("")) {
                Ok((text, _metadata)) => {
                    if text.trim().is_empty() {
                        self.extracted_text_buffer.set_text("No text found in PDF.");
                    } else {
                        // Normalize spacing: replace multiple newlines with single newlines
                        let normalized_text = text
                            .lines()
                            .filter(|line| !line.trim().is_empty())
                            .collect::<Vec<&str>>()
                            .join("\n");
                        
                        self.extracted_text_buffer.set_text(&normalized_text);
                        self.log("✅ Text extracted with extractous");
                    }
                }
                Err(e) => {
                    self.extracted_text_buffer.set_text(&format!("Error extracting text: {}", e));
                    self.log(&format!("❌ Extractous error: {}", e));
                }
            }
            
            app::awake();
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
}


fn main() {
    let app_state = Chonker5App::new();
    Chonker5App::run(app_state);
}