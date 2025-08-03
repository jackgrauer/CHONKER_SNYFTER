#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! fltk = { version = "1.4", features = ["fltk-bundled"] }
//! mupdf = "0.4"
//! rfd = "0.15"
//! image = "0.25"
//! ```

use fltk::{
    app::{self, App, Scheme},
    button::Button,
    enums::{Color, ColorDepth, Event, Font, FrameType, Key},
    frame::Frame,
    group::{Flex, Pack, PackType, Scroll},
    prelude::*,
    text::{TextBuffer, TextDisplay},
    window::Window,
    image as fltk_image,
};
use mupdf::{Document, Matrix, Pixmap};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

const WINDOW_WIDTH: i32 = 1200;
const WINDOW_HEIGHT: i32 = 800;
const TOP_BAR_HEIGHT: i32 = 60;
const LOG_HEIGHT: i32 = 100;

// Color scheme
const COLOR_TEAL: Color = Color::from_rgb(0x1A, 0xBC, 0x9C);
const COLOR_DARK_BG: Color = Color::from_rgb(0x52, 0x56, 0x59);
const COLOR_DARKER_BG: Color = Color::from_rgb(0x2D, 0x2F, 0x31);
const COLOR_BUTTON_BG: Color = Color::from_rgb(0x3A, 0x3C, 0x3E);

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
    extracted_text_display: TextDisplay,
    extracted_text_buffer: TextBuffer,
    
    // PDF state
    pdf_document: Option<Document>,
    current_page: usize,
    total_pages: usize,
    zoom_level: f32,
    pdf_image: Option<fltk_image::RgbImage>,
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
        let mut top_bar = Pack::default()
            .with_size(WINDOW_WIDTH, TOP_BAR_HEIGHT);
        top_bar.set_type(PackType::Horizontal);
        top_bar.set_color(COLOR_TEAL);
        top_bar.set_spacing(10);
        top_bar.set_frame(FrameType::FlatBox);
        
        Frame::default().with_size(10, 0); // Padding
        
        let mut open_btn = Button::default()
            .with_size(100, 40)
            .with_label("Open");
        open_btn.set_color(COLOR_BUTTON_BG);
        open_btn.set_label_color(Color::White);
        open_btn.set_frame(FrameType::UpBox);
        
        let mut prev_btn = Button::default()
            .with_size(80, 40)
            .with_label("Prev");
        prev_btn.set_color(COLOR_BUTTON_BG);
        prev_btn.set_label_color(Color::White);
        prev_btn.set_frame(FrameType::UpBox);
        prev_btn.deactivate();
        
        let mut next_btn = Button::default()
            .with_size(80, 40)
            .with_label("Next");
        next_btn.set_color(COLOR_BUTTON_BG);
        next_btn.set_label_color(Color::White);
        next_btn.set_frame(FrameType::UpBox);
        next_btn.deactivate();
        
        let mut zoom_in_btn = Button::default()
            .with_size(100, 40)
            .with_label("Zoom In");
        zoom_in_btn.set_color(COLOR_BUTTON_BG);
        zoom_in_btn.set_label_color(Color::White);
        zoom_in_btn.set_frame(FrameType::UpBox);
        
        let mut zoom_out_btn = Button::default()
            .with_size(100, 40)
            .with_label("Zoom Out");
        zoom_out_btn.set_color(COLOR_BUTTON_BG);
        zoom_out_btn.set_label_color(Color::White);
        zoom_out_btn.set_frame(FrameType::UpBox);
        
        let mut fit_width_btn = Button::default()
            .with_size(100, 40)
            .with_label("Fit Width");
        fit_width_btn.set_color(COLOR_BUTTON_BG);
        fit_width_btn.set_label_color(Color::White);
        fit_width_btn.set_frame(FrameType::UpBox);
        
        Frame::default().with_size(20, 0); // Spacer
        
        let mut status_label = Frame::default()
            .with_size(300, 40)
            .with_label("Ready! Click 'Open' to load a PDF");
        status_label.set_label_color(Color::White);
        
        let mut zoom_label = Frame::default()
            .with_size(100, 40)
            .with_label("Zoom: 100%");
        zoom_label.set_label_color(Color::White);
        
        let mut page_label = Frame::default()
            .with_size(100, 40)
            .with_label("Page: 0/0");
        page_label.set_label_color(Color::White);
        
        top_bar.end();
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
        
        // Right pane: Extracted text display
        let mut text_scroll = Scroll::default()
            .with_size(WINDOW_WIDTH / 2, WINDOW_HEIGHT - TOP_BAR_HEIGHT - LOG_HEIGHT);
        text_scroll.set_color(COLOR_DARKER_BG);
        
        let mut extracted_text_display = TextDisplay::default()
            .with_size(WINDOW_WIDTH / 2 - 20, WINDOW_HEIGHT - TOP_BAR_HEIGHT - LOG_HEIGHT - 20);
        extracted_text_display.set_color(COLOR_DARKER_BG);
        extracted_text_display.set_text_color(Color::White);
        extracted_text_display.set_text_font(Font::Helvetica);
        extracted_text_display.set_text_size(14);
        extracted_text_display.set_frame(FrameType::FlatBox);
        extracted_text_display.wrap_mode(fltk::text::WrapMode::AtBounds, 0);
        
        let mut extracted_text_buffer = TextBuffer::default();
        extracted_text_buffer.set_text("Extracted text will appear here...");
        extracted_text_display.set_buffer(extracted_text_buffer.clone());
        
        text_scroll.end();
        
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
        
        log_buffer.append("🐹 CHONKER 5 Ready!\n");
        log_buffer.append("📌 Using MuPDF for PDF rendering\n");
        log_buffer.append("📌 Keyboard shortcuts: Cmd+O (Open), ←/→ (Navigate), +/- (Zoom), F (Fit width)\n");
        
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
            extracted_text_display,
            extracted_text_buffer,
            pdf_document: None,
            current_page: 0,
            total_pages: 0,
            zoom_level: 1.0,
            pdf_image: None,
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
        
        // Keyboard shortcuts
        {
            let state = app_state.clone();
            window.handle(move |_, ev| match ev {
                Event::KeyDown => {
                    let key = app::event_key();
                    if app::is_event_command() && key == Key::from_char('o') {
                        state.borrow_mut().open_file();
                        true
                    } else if key == Key::Left {
                        state.borrow_mut().prev_page();
                        true
                    } else if key == Key::Right {
                        state.borrow_mut().next_page();
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
    
    fn load_pdf(&mut self, path: PathBuf) {
        let filename = path.file_name().unwrap_or_default().to_string_lossy();
        self.log(&format!("📄 Loading: {}", filename));
        
        match Document::open(&path.to_string_lossy()) {
            Ok(document) => {
                let total_pages = document.page_count() as usize;
                
                self.pdf_document = Some(document);
                self.total_pages = total_pages;
                self.current_page = 0;
                
                self.log(&format!("✅ PDF loaded successfully: {} pages", self.total_pages));
                self.update_status(&format!("Loaded! {} pages", self.total_pages));
                
                // Enable navigation buttons
                if self.total_pages > 1 {
                    self.next_btn.activate();
                }
                
                // Update UI
                self.update_page_label();
                self.render_current_page();
            }
            Err(e) => {
                let error_msg = format!("Failed to load PDF: {}", e);
                self.log(&format!("❌ {}", error_msg));
                self.update_status(&error_msg);
            }
        }
    }
    
    fn render_current_page(&mut self) {
        if let Some(document) = &self.pdf_document {
            if let Ok(page) = document.load_page(self.current_page as i32) {
                // Calculate render size
                let scale = self.zoom_level * 2.0;
                let matrix = Matrix::new_scale(scale, scale);
                
                // Get page bounds
                let bounds = page.bounds().unwrap();
                let width = ((bounds.x1 - bounds.x0) * scale) as i32;
                let height = ((bounds.y1 - bounds.y0) * scale) as i32;
                
                // Create pixmap
                let pixmap = Pixmap::new(width, height, mupdf::ColorSpace::device_rgb(), 0).unwrap();
                
                // Clear to white
                pixmap.clear(0xFF);
                
                // Render page
                page.run_contents(&pixmap, &matrix, &mupdf::default_cookie()).unwrap();
                
                // Convert to FLTK image
                let data = pixmap.pixels().unwrap();
                let mut rgb_data = Vec::with_capacity((width * height * 3) as usize);
                
                // MuPDF uses RGB format
                for pixel in data.chunks(3) {
                    rgb_data.push(pixel[0]); // R
                    rgb_data.push(pixel[1]); // G
                    rgb_data.push(pixel[2]); // B
                }
                
                if let Ok(img) = fltk_image::RgbImage::new(&rgb_data, width, height, ColorDepth::Rgb8) {
                    self.pdf_image = Some(img.clone());
                    
                    // Update the frame size and redraw
                    self.pdf_frame.set_size(width, height);
                    self.pdf_frame.set_image(Some(img));
                    self.pdf_frame.set_label("");
                    self.pdf_frame.redraw();
                }
                
                // Extract text from current page
                self.extract_current_page_text();
            }
        }
    }
    
    fn extract_current_page_text(&mut self) {
        if let Some(document) = &self.pdf_document {
            if let Ok(page) = document.load_page(self.current_page as i32) {
                if let Ok(text) = page.to_text() {
                    if text.is_empty() {
                        self.extracted_text_buffer.set_text("No text found on this page.");
                    } else {
                        self.extracted_text_buffer.set_text(&text);
                    }
                } else {
                    self.extracted_text_buffer.set_text("Error extracting text from page.");
                }
            }
        }
    }
    
    fn prev_page(&mut self) {
        if self.current_page > 0 {
            self.current_page -= 1;
            self.render_current_page();
            self.update_page_label();
            self.update_nav_buttons();
            self.log(&format!("◀ Page {}", self.current_page + 1));
        }
    }
    
    fn next_page(&mut self) {
        if self.current_page < self.total_pages - 1 {
            self.current_page += 1;
            self.render_current_page();
            self.update_page_label();
            self.update_nav_buttons();
            self.log(&format!("▶ Page {}", self.current_page + 1));
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
        } else {
            self.prev_btn.deactivate();
        }
        
        if self.current_page < self.total_pages - 1 {
            self.next_btn.activate();
        } else {
            self.next_btn.deactivate();
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