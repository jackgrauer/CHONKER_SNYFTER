#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! fltk = { version = "1.4", features = ["fltk-bundled"] }
//! rfd = "0.15"
//! image = "0.25"
//! pdfium-render = { version = "0.8", features = ["thread_safe"] }
//! ```

use fltk::{
    app::{self, App, Scheme},
    button::Button,
    enums::{Color, Event, Font, FrameType, Key},
    frame::Frame,
    group::{Flex, Group, Scroll},
    input::Input,
    misc::HelpView,
    text::{TextBuffer, TextDisplay},
    window::Window,
    image as fltk_image,
    prelude::*,
};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::process::Command;
use std::fs;
use std::error::Error;
use pdfium_render::prelude::*;

const WINDOW_WIDTH: i32 = 1200;
const WINDOW_HEIGHT: i32 = 800;
const TOP_BAR_HEIGHT: i32 = 60;
const LOG_HEIGHT: i32 = 100;

// Color scheme
const COLOR_TEAL: Color = Color::from_rgb(0x1A, 0xBC, 0x9C);
const COLOR_DARK_BG: Color = Color::from_rgb(0x52, 0x56, 0x59);
const COLOR_DARKER_BG: Color = Color::from_rgb(0x2D, 0x2F, 0x31);
const COLOR_LIGHT_CHROME: Color = Color::from_rgb(0xE8, 0xE8, 0xE8); // Light chrome for backgrounds

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
    structured_btn: Button,
    page_range_input: Input,
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
        let mut structured_btn = Button::default()
            .with_pos(x_pos, y_pos)
            .with_size(120, 40)
            .with_label("Ferrules");
        structured_btn.set_color(Color::from_rgb(0x00, 0x8C, 0x3A)); // Green color for distinction
        structured_btn.set_label_color(Color::White);
        structured_btn.set_frame(FrameType::UpBox);
        structured_btn.set_label_size(14);
        structured_btn.deactivate(); // Start disabled until PDF is loaded
        
        x_pos += 130;
        let mut page_range_input = Input::default()
            .with_pos(x_pos, y_pos)
            .with_size(80, 40);
        page_range_input.set_value("1-10");
        page_range_input.set_tooltip("Page range (e.g., 1-10, 1,3,5-7)");
        page_range_input.deactivate(); // Start disabled until PDF is loaded
        
        x_pos += 90;
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
        pdf_frame.set_color(COLOR_LIGHT_CHROME);
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
        extracted_text_buffer.set_text("PDF content will appear here after clicking 'Ferrules - HTML' or 'Pdfium - All Content'...");
        extracted_text_display.set_buffer(extracted_text_buffer.clone());
        
        // Structured view with HelpView for ferrules HTML rendering (has its own scrollbar)
        let mut structured_html_view = HelpView::default()
            .with_size(WINDOW_WIDTH / 2, WINDOW_HEIGHT - TOP_BAR_HEIGHT - LOG_HEIGHT);
        structured_html_view.set_frame(FrameType::FlatBox);
        structured_html_view.hide();
        
        right_group.end();
        
        // Ensure text is shown initially and structured view is hidden
        extracted_text_display.show();
        structured_html_view.hide();
        
        content_flex.end();
        
        // Log area
        let mut log_display = TextDisplay::default()
            .with_size(WINDOW_WIDTH, LOG_HEIGHT);
        log_display.set_color(COLOR_DARKER_BG);
        log_display.set_text_color(Color::White);
        log_display.set_text_font(Font::Courier);
        log_display.set_text_size(11);
        log_display.set_frame(FrameType::FlatBox);
        log_display.wrap_mode(fltk::text::WrapMode::AtBounds, 0);
        
        let mut log_buffer = TextBuffer::default();
        log_display.set_buffer(log_buffer.clone());
        main_flex.fixed(&mut log_display, LOG_HEIGHT);
        
        main_flex.end();
        window.end();
        window.show();
        
        // Force redraw of all widgets
        window.redraw();
        app::redraw();
        
        log_buffer.append("🐹 CHONKER 5 Ready!\n");
        log_buffer.append("📌 Using MuPDF for PDF rendering + Ferrules/Pdfium for structured data extraction\n");
        log_buffer.append("📌 Keyboard shortcuts: Cmd+O (Open), ←/→ (Navigate), +/- (Zoom), F (Fit width)\n");
        log_buffer.append("📌 Ferrules: Perfect layout reconstruction | Pdfium: Complete content extraction\n");
        
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
            structured_btn: structured_btn.clone(),
            page_range_input: page_range_input.clone(),
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
                state.borrow_mut().prev_page();
            });
        }
        
        {
            let state = app_state.clone();
            next_btn.set_callback(move |_| {
                state.borrow_mut().next_page();
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
        
        // Structured data button
        {
            let state = app_state.clone();
            structured_btn.set_callback(move |_| {
                state.borrow_mut().extract_structured_data();
            });
        }
        
        // Pdfium content extraction button
        {
            let state = app_state.clone();
            table_btn.set_callback(move |_| {
                state.borrow_mut().extract_pdfium_content();
            });
        }
        
        // Make window respond to close events
        window.set_callback(|_| {
            if app::event() == Event::Close {
                app::quit();
            }
        });
        
        // Add keyboard shortcuts
        {
            let state = app_state.clone();
            window.handle(move |win, event| match event {
                Event::Focus => {
                    win.set_visible_focus();
                    true
                }
                Event::KeyDown => {
                    let key = app::event_key();
                    if app::is_event_command() && key == Key::from_char('o') {
                        state.borrow_mut().open_file();
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
                    } else if key == Key::from_char('f') || key == Key::from_char('F') {
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
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                
                if !output.status.success() {
                    self.log(&format!("❌ mutool error: {}", stderr));
                    self.update_status("Failed to load PDF");
                    return;
                }
                
                // Parse page count from mutool info output
                let mut total_pages = 0;
                for line in stdout.lines() {
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
                    
                    // Enable extraction buttons
                    self.structured_btn.activate();
                    self.structured_btn.set_color(Color::from_rgb(0x00, 0x8C, 0x3A));
                    self.structured_btn.set_label_color(Color::White);
                    
                    // Enable page range input
                    self.page_range_input.activate();
                    
                    // Update button label and default page range for large files
                    if self.total_pages > 20 {
                        self.page_range_input.set_value("1-10");
                        self.log("📄 Large PDF detected - Default page range set to 1-10");
                    } else {
                        // For smaller PDFs, clear the input to process all pages
                        self.page_range_input.set_value("");
                    }
                    
                    self.table_btn.activate();
                    self.table_btn.set_color(Color::from_rgb(0xE6, 0x7E, 0x22));
                    self.table_btn.set_label_color(Color::White);
                    
                    // Update UI
                    self.update_page_label();
                    
                    // Render the PDF page immediately
                    self.render_current_page();
                    
                    // Show instructions for extraction
                    self.extracted_text_buffer.set_text("Click 'Ferrules - HTML' for structured layout or 'Pdfium - All Content' for complete extraction...");
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
            let png_path = temp_dir.join(format!("chonker_page_{}.png", self.current_page));
            
            // Calculate dimensions
            let dpi = 72.0 * self.zoom_level;
            
            // Render current page to PNG using mutool with timeout
            match Command::new("timeout")
                .arg("10")  // 10 second timeout
                .arg("mutool")
                .arg("draw")
                .arg("-r")
                .arg(dpi.to_string())
                .arg("-o")
                .arg(&png_path)
                .arg(&pdf_path)
                .arg((self.current_page + 1).to_string())
                .output()
            {
                Ok(output) => {
                    if output.status.success() && png_path.exists() {
                        // Load the rendered image
                        if let Ok(img) = fltk_image::PngImage::load(&png_path) {
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
                }
                Err(e) => {
                    self.log(&format!("❌ Failed to render page: {}", e));
                }
            }
            
            // Don't extract content automatically
        }
    }
    
    fn extract_structured_data(&mut self) {
        // Get page range from input field
        let page_range_str = self.page_range_input.value();
        let page_range = if !page_range_str.is_empty() {
            Some(page_range_str)
        } else if self.total_pages > 20 {
            // Default to first 10 pages for large PDFs
            Some("1-10".to_string())
        } else {
            None
        };
        self.extract_structured_data_with_options(page_range);
    }
    
    fn extract_structured_data_with_options(&mut self, page_range: Option<String>) {
        if let Some(pdf_path) = &self.pdf_path.clone() {
            let range_msg = if let Some(ref range) = page_range {
                format!(" (pages {})", range)
            } else {
                " (all pages)".to_string()
            };
            self.log(&format!("🔄 Extracting structured data with ferrules{}...", range_msg));
            
            // Create temp dir for ferrules output
            let temp_dir = std::env::temp_dir();
            
            // Run ferrules command with HTML output
            let mut cmd = Command::new("timeout");
            cmd.arg("120")  // Increased to 120 second timeout for large files
                .arg("ferrules")
                .arg(pdf_path)
                .arg("-o")
                .arg(&temp_dir)
                .arg("--html");  // Request HTML output
            
            // Add page range if specified
            if let Some(ref range) = page_range {
                cmd.arg("-r").arg(range);
            }
            
            // Add debug info for large files
            cmd.arg("--debug");
            
            self.log(&format!("🚀 Running ferrules with 120s timeout..."));
            if let Some(ref range) = page_range {
                self.log(&format!("📄 Processing pages: {}", range));
            }
            
            let start_time = std::time::Instant::now();
            let output = cmd.output();
            let elapsed = start_time.elapsed();
            
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
                        
                        if let Some(html_path) = html_file {
                            // Load the HTML content
                            match fs::read_to_string(&html_path) {
                                Ok(content) => {
                                    // Display the HTML content
                                    self.structured_html_view.set_value(&content);
                                    
                                    // Hide text display, show structured HTML view
                                    self.extracted_text_display.hide();
                                    self.structured_html_view.show();
                                    
                                    self.log(&format!("✅ Structured data extracted with ferrules in {:.1}s", elapsed.as_secs_f32()));
                                    
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
                        } else {
                            // Try to show what's in the results directory
                            let entries = fs::read_dir(&results_dir)
                                .map(|dir| {
                                    dir.filter_map(|e| e.ok())
                                        .map(|e| e.file_name().to_string_lossy().to_string())
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
                    } else {
                        let error_msg = String::from_utf8_lossy(&result.stderr);
                        let exit_code = result.status.code().unwrap_or(-1);
                        
                        // Check if it was a timeout
                        if exit_code == 124 {
                            self.log("⏱️ Ferrules timed out after 120 seconds");
                            self.log("💡 Try processing fewer pages or a smaller file");
                            
                            // Offer to retry with page range
                            let html = r#"
                            <html>
                            <head>
                                <style>
                                    body { font-family: Arial, sans-serif; margin: 20px; background-color: #e8e8e8; }
                                    .error { background: #fff3cd; padding: 20px; border-radius: 5px; }
                                    .suggestions { margin-top: 20px; }
                                    button { padding: 10px 20px; margin: 5px; cursor: pointer; }
                                </style>
                            </head>
                            <body>
                                <h2>⏱️ Ferrules Processing Timeout</h2>
                                <div class="error">
                                    <p>The document is too large to process in the time limit.</p>
                                    <div class="suggestions">
                                        <h3>Suggestions:</h3>
                                        <ul>
                                            <li>Try processing specific pages (e.g., pages 1-10)</li>
                                            <li>Use a smaller PDF file</li>
                                            <li>Try the Pdfium extraction method instead</li>
                                        </ul>
                                        <p>To process specific pages, you would need to modify the code to add page range support.</p>
                                    </div>
                                </div>
                            </body>
                            </html>"#;
                            self.structured_html_view.set_value(html);
                            self.extracted_text_display.hide();
                            self.structured_html_view.show();
                        } else {
                            // Check for memory-related errors
                            if error_msg.contains("memory") || error_msg.contains("killed") || exit_code == 137 {
                                self.log("💾 Ferrules crashed - likely out of memory");
                                self.log("💡 This PDF may be too large or complex");
                                
                                let html = r#"
                                <html>
                                <head>
                                    <style>
                                        body { font-family: Arial, sans-serif; margin: 20px; background-color: #e8e8e8; }
                                        .error { background: #f8d7da; padding: 20px; border-radius: 5px; }
                                    </style>
                                </head>
                                <body>
                                    <h2>💾 Memory Error</h2>
                                    <div class="error">
                                        <p>Ferrules ran out of memory processing this document.</p>
                                        <p>Try using the Pdfium extraction method for large files.</p>
                                    </div>
                                </body>
                                </html>"#;
                                self.structured_html_view.set_value(html);
                                self.extracted_text_display.hide();
                                self.structured_html_view.show();
                            } else {
                                self.extracted_text_buffer.set_text(&format!("Ferrules error: {}", error_msg));
                                self.log(&format!("❌ Ferrules failed with exit code {}: {}", exit_code, error_msg));
                            }
                        }
                    }
                }
                Err(e) => {
                    self.extracted_text_buffer.set_text(&format!("Failed to run ferrules: {}", e));
                    self.log(&format!("❌ Failed to run ferrules: {}", e));
                    self.log("💡 Make sure ferrules is installed: pip install ferrules");
                }
            }
            
            app::awake();
        } else {
            self.log("⚠️ No PDF loaded. Press Cmd+O to open a file first.");
        }
    }
    
    fn extract_pdfium_content(&mut self) {
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
                            body {{ font-family: Arial, sans-serif; margin: 20px; background-color: #e8e8e8; }}
                            h2 {{ color: #d9534f; }}
                            .error {{ background: #f2dede; padding: 15px; border-radius: 5px; color: #a94442; }}
                            .suggestions {{ margin-top: 20px; }}
                            .suggestions li {{ margin-bottom: 10px; }}
                        </style>
                    </head>
                    <body>
                        <h2>⚠️ Pdfium Extraction Error</h2>
                        <div class="error">
                            <h4>Error Details:</h4>
                            <p>{}</p>
                            
                            <div class="suggestions">
                                <h4>Possible causes:</h4>
                                <ul>
                                    <li>PDF format incompatibility</li>
                                    <li>No structured content in the document</li>
                                    <li>Missing pdfium library</li>
                                </ul>
                                <p>Try using 'Ferrules - HTML' for structured document layout instead.</p>
                            </div>
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
                body { font-family: Arial, sans-serif; margin: 20px; line-height: 1.6; background-color: #e8e8e8; }
                h2 { color: #2c3e50; border-bottom: 2px solid #e67e22; padding-bottom: 10px; }
                h3 { color: #34495e; margin-top: 20px; }
                .metadata { background: #f8f9fa; padding: 15px; border-radius: 5px; margin-bottom: 20px; }
                .metadata dt { font-weight: bold; color: #495057; display: inline-block; width: 120px; }
                .metadata dd { display: inline; margin-left: 10px; }
                .page { margin-bottom: 30px; border: 1px solid #dee2e6; padding: 20px; border-radius: 5px; background-color: #f5f5f5; }
                .page-header { background: #e9ecef; margin: -20px -20px 15px -20px; padding: 10px 20px; border-radius: 5px 5px 0 0; }
                .table-container { background: #ffffff; padding: 15px; border-radius: 5px; margin: 15px 0; overflow-x: auto; }
                pre { white-space: pre-wrap; word-wrap: break-word; }
                .no-tables { color: #6c757d; font-style: italic; }
                .stats { background: #e7f3ff; padding: 10px; border-radius: 5px; margin-top: 20px; }
            </style>
        </head>
        <body>
            <h2>📊 Pdfium Full Content Extraction</h2>
        "#);
        
        // Initialize pdfium - try dynamic library first, then system library
        let pdfium = Pdfium::new(
            Pdfium::bind_to_library("./lib/libpdfium.dylib").or_else(|_| {
                Pdfium::bind_to_library("/usr/local/lib/libpdfium.dylib")
            }).or_else(|_| {
                Pdfium::bind_to_system_library()
            }).map_err(|e| format!("Failed to bind to pdfium library: {}", e))?
        );
        
        // Load the PDF
        let document = pdfium.load_pdf_from_file(pdf_path, None)
            .map_err(|e| format!("Failed to load PDF: {}", e))?;
        
        // Add metadata
        html.push_str(r#"<div class="metadata">"#);
        html.push_str(&format!("<dl><dt>Document:</dt><dd>{}</dd>", pdf_path.file_name().unwrap_or_default().to_string_lossy()));
        html.push_str(&format!("<dt>Total Pages:</dt><dd>{}</dd>", document.pages().len()));
        html.push_str("</dl></div>");
        
        let total_pages = document.pages().len();
        let mut total_text_length = 0;
        let mut total_tables_found = 0;
        
        // Process each page
        for (page_index, page) in document.pages().iter().enumerate() {
            let page_number = page_index + 1;
            html.push_str(&format!(r#"<div class="page">"#));
            html.push_str(&format!(r#"<div class="page-header"><h3>Page {}</h3></div>"#, page_number));
            
            // Extract all text
            let text_page = page.text().map_err(|e| format!("Failed to get text: {}", e))?;
            let page_text = text_page.all();
            
            if !page_text.trim().is_empty() {
                html.push_str("<h4>Page Content:</h4>");
                html.push_str(&format!("<pre>{}</pre>", 
                    page_text
                        .replace("&", "&amp;")
                        .replace("<", "&lt;")
                        .replace(">", "&gt;")
                ));
                total_text_length += page_text.len();
                
                // Simple table detection based on alignment patterns
                let lines: Vec<&str> = page_text.lines().collect();
                let mut potential_table_lines = Vec::new();
                
                for line in &lines {
                    // Count spaces/tabs that might indicate columns
                    let segments: Vec<&str> = line.split_whitespace().collect();
                    if segments.len() >= 2 {
                        potential_table_lines.push(line);
                    }
                }
                
                // If we have multiple lines that look like table rows
                if potential_table_lines.len() >= 3 {
                    html.push_str(&format!(r#"<div class="table-container">"#));
                    html.push_str(&format!("<h5>Table {} (Page {}):</h5>", total_tables_found + 1, page_number));
                    html.push_str("<table border='1' style='border-collapse: collapse; width: 100%;'>");
                    
                    for (i, line) in potential_table_lines.iter().enumerate() {
                        html.push_str("<tr>");
                        let cells: Vec<&str> = line.split_whitespace().collect();
                        let tag = if i == 0 { "th" } else { "td" };
                        for cell in cells {
                            html.push_str(&format!("<{} style='padding: 8px; text-align: left;'>{}</{}>", 
                                tag,
                                cell.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;"),
                                tag
                            ));
                        }
                        html.push_str("</tr>");
                    }
                    
                    html.push_str("</table>");
                    html.push_str("</div>");
                    total_tables_found += 1;
                }
            } else {
                html.push_str(r#"<p class="no-tables">No text content detected on this page.</p>"#);
            }
            
            html.push_str("</div>");
        }
        
        // Add summary statistics
        html.push_str(&format!(r#"
        <div class="stats">
            <h3>Extraction Summary</h3>
            <ul>
                <li>Total pages processed: {}</li>
                <li>Total text extracted: {} characters</li>
                <li>Tables detected: {}</li>
            </ul>
        </div>
        "#, total_pages, total_text_length, total_tables_found));
        
        html.push_str("</body></html>");
        
        Ok(html)
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
            self.extracted_text_buffer.set_text("Click 'Ferrules - HTML' or 'Pdfium - All Content' to extract content from this page...");
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
            self.extracted_text_buffer.set_text("Click 'Ferrules - HTML' or 'Pdfium - All Content' to extract content from this page...");
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
        // Get the viewport width
        let viewport_width = (self.pdf_frame.parent().unwrap().width() - 40) as f32;
        
        // For simplicity, assume a standard page width of 612 points (8.5 inches)
        let page_width_points = 612.0;
        
        // Calculate zoom level to fit width
        self.zoom_level = viewport_width / page_width_points;
        self.zoom_level = self.zoom_level.clamp(0.25, 4.0);
        
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
