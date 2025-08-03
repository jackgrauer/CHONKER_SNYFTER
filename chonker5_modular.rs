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

// ==================== IMPORTS ====================
use fltk::{
    app::{self, App, Scheme},
    button::Button,
    draw,
    enums::{Color, Event, Font, FrameType, Key},
    frame::Frame,
    group::{Flex, Group, Scroll},
    input::MultilineInput,
    prelude::*,
    text::{TextBuffer, TextDisplay},
    window::Window,
    widget::Widget,
    widget_extends,
    image as fltk_image,
};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::process::Command;
use std::fs;
use extractous::Extractor;
use serde::{Deserialize, Serialize};
use serde_json;

// ==================== CONSTANTS ====================
mod constants {
    use fltk::enums::Color;
    
    pub const WINDOW_WIDTH: i32 = 1200;
    pub const WINDOW_HEIGHT: i32 = 800;
    pub const TOP_BAR_HEIGHT: i32 = 60;
    pub const LOG_HEIGHT: i32 = 100;
    
    // Color scheme
    pub const COLOR_TEAL: Color = Color::from_rgb(0x1A, 0xBC, 0x9C);
    pub const COLOR_DARK_BG: Color = Color::from_rgb(0x52, 0x56, 0x59);
    pub const COLOR_DARKER_BG: Color = Color::from_rgb(0x2D, 0x2F, 0x31);
}

// ==================== FERRULES DATA STRUCTURES ====================
mod ferrules {
    use serde::{Deserialize, Serialize};
    use serde_json;
    
    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Document {
        pub pages: Vec<Page>,
        pub blocks: Vec<Block>,
    }
    
    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Page {
        pub id: i32,
        pub width: f64,
        pub height: f64,
    }
    
    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Block {
        pub id: i32,
        pub pages_id: Vec<i32>,
        pub bbox: BoundingBox,
        pub kind: Kind,
    }
    
    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct BoundingBox {
        pub x0: f64,
        pub y0: f64,
        pub x1: f64,
        pub y1: f64,
    }
    
    #[derive(Debug, Deserialize, Serialize, Clone)]
    #[serde(untagged)]
    pub enum Kind {
        Structured { block_type: String, text: String },
        Text { text: String },
        Other(serde_json::Value),
    }
    
    impl Kind {
        pub fn get_text(&self) -> Option<&str> {
            match self {
                Kind::Structured { text, .. } => Some(text),
                Kind::Text { text } => Some(text),
                Kind::Other(_) => None,
            }
        }
        
        pub fn get_block_type(&self) -> Option<&str> {
            match self {
                Kind::Structured { block_type, .. } => Some(block_type),
                _ => None,
            }
        }
    }
}

// ==================== HELPER FUNCTIONS ====================
mod helpers {
    use fltk::text::TextBuffer;
    
    pub fn add_status(buffer: &mut TextBuffer, message: &str) {
        let timestamp = chrono::Local::now().format("[%H:%M:%S]");
        buffer.append(&format!("{} {}\n", timestamp, message));
        
        // Auto-scroll to bottom
        if let Some(mut display) = buffer.text_display() {
            display.scroll(buffer.length(), 0);
        }
    }
    
    pub fn is_html_content(content: &str) -> bool {
        content.trim().starts_with('<') || 
        content.contains("<html") || 
        content.contains("<body") ||
        content.contains("<div")
    }
}

// ==================== CUSTOM WIDGET ====================
mod custom_widget {
    use super::*;
    use super::ferrules;
    use super::constants::*;
    
    #[derive(Debug, Clone)]
    pub struct StructuredTextWidget {
        inner: Widget,
        document: Rc<RefCell<Option<ferrules::Document>>>,
        selected_block: Rc<RefCell<Option<usize>>>,
        scroll_offset: Rc<RefCell<(f64, f64)>>,
        zoom: Rc<RefCell<f32>>,
        dragging: Rc<RefCell<Option<(usize, f64, f64)>>>,
    }
    
    widget_extends!(StructuredTextWidget, Widget, inner);
    
    impl StructuredTextWidget {
        pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
            let mut inner = Widget::default().with_pos(x, y).with_size(w, h);
            inner.set_frame(FrameType::FlatBox);
            inner.set_color(Color::White);
            
            let widget = Self {
                inner,
                document: Rc::new(RefCell::new(None)),
                selected_block: Rc::new(RefCell::new(None)),
                scroll_offset: Rc::new(RefCell::new((0.0, 0.0))),
                zoom: Rc::new(RefCell::new(1.0)),
                dragging: Rc::new(RefCell::new(None)),
            };
            
            // Clone references for the draw callback
            let doc_clone = widget.document.clone();
            let selected_clone = widget.selected_block.clone();
            let scroll_clone = widget.scroll_offset.clone();
            let zoom_clone = widget.zoom.clone();
            
            widget.inner.draw(move |w| {
                draw::draw_rect_fill(w.x(), w.y(), w.w(), w.h(), Color::White);
                
                // Status indicator
                draw::set_draw_color(Color::from_rgb(0, 150, 0));
                draw::set_font(Font::Helvetica, 10);
                draw::draw_text("🔧 Custom Renderer Active", w.x() + 5, w.y() + 15);
                
                if let Some(doc) = doc_clone.borrow().as_ref() {
                    let scroll = scroll_clone.borrow();
                    let zoom = *zoom_clone.borrow();
                    let selected = *selected_clone.borrow();
                    
                    draw::push_clip(w.x(), w.y(), w.w(), w.h());
                    
                    let mut current_y = 20.0;
                    let page_gap = 20.0;
                    
                    for (page_idx, page) in doc.pages.iter().enumerate() {
                        let page_x = w.x() as f64 + 10.0 - scroll.0;
                        let page_y = w.y() as f64 + current_y - scroll.1;
                        let page_width = page.width * zoom as f64;
                        let page_height = page.height * zoom as f64;
                        
                        // Draw page background
                        draw::set_draw_color(Color::from_rgb(250, 250, 250));
                        draw::draw_rect_fill(
                            page_x as i32,
                            page_y as i32,
                            page_width as i32,
                            page_height as i32,
                            Color::from_rgb(250, 250, 250)
                        );
                        
                        // Draw page border
                        draw::set_draw_color(Color::from_rgb(200, 200, 200));
                        draw::draw_rect(
                            page_x as i32,
                            page_y as i32,
                            page_width as i32,
                            page_height as i32
                        );
                        
                        // Page info
                        draw::set_draw_color(Color::from_rgb(100, 100, 100));
                        draw::set_font(Font::Helvetica, 12);
                        draw::draw_text(
                            &format!("Page {} ({:.0}x{:.0})", page_idx + 1, page.width, page.height),
                            page_x as i32 + 5,
                            page_y as i32 - 5
                        );
                        
                        // Draw blocks for this page
                        for (block_idx, block) in doc.blocks.iter().enumerate() {
                            if block.pages_id.contains(&page.id) {
                                // Validate coordinates
                                if block.bbox.x0 < 0.0 || block.bbox.y0 < 0.0 || 
                                   block.bbox.x1 > page.width || block.bbox.y1 > page.height {
                                    draw::set_draw_color(Color::Red);
                                    draw::draw_text(
                                        &format!("⚠️ Block {} out of bounds", block_idx), 
                                        10, 
                                        current_y as i32 + 20
                                    );
                                    continue;
                                }
                                
                                let block_x = page_x + block.bbox.x0 * zoom as f64;
                                let block_y = page_y + block.bbox.y0 * zoom as f64;
                                let block_width = (block.bbox.x1 - block.bbox.x0) * zoom as f64;
                                let block_height = (block.bbox.y1 - block.bbox.y0) * zoom as f64;
                                
                                // Draw block background
                                if Some(block_idx) == selected {
                                    draw::set_draw_color(Color::from_rgb(255, 255, 200));
                                } else {
                                    draw::set_draw_color(Color::from_rgb(255, 255, 255));
                                }
                                draw::draw_rect_fill(
                                    block_x as i32,
                                    block_y as i32,
                                    block_width as i32,
                                    block_height as i32,
                                    draw::color()
                                );
                                
                                // Draw bounding box
                                if Some(block_idx) == selected {
                                    draw::set_draw_color(Color::from_rgb(0, 0, 255));
                                    draw::set_line_style(draw::LineStyle::Solid, 2);
                                } else {
                                    draw::set_draw_color(Color::from_rgb(200, 200, 200));
                                    draw::set_line_style(draw::LineStyle::Solid, 1);
                                }
                                draw::draw_rect(
                                    block_x as i32,
                                    block_y as i32,
                                    block_width as i32,
                                    block_height as i32
                                );
                                
                                // Draw block label
                                draw::set_draw_color(Color::from_rgb(100, 100, 255));
                                draw::set_font(Font::Helvetica, 10);
                                let label = match block.kind.get_block_type() {
                                    Some(t) => format!("#{} {}", block_idx, t),
                                    None => format!("#{} Text", block_idx),
                                };
                                draw::draw_text(&label, block_x as i32 + 2, block_y as i32 - 2);
                                
                                // Draw text content
                                draw::set_draw_color(Color::Black);
                                draw::set_font(Font::Helvetica, 12);
                                
                                if let Some(text) = block.kind.get_text() {
                                    // Debug: Show first 100 chars
                                    let preview = if text.len() > 100 {
                                        format!("{}...", &text[..100])
                                    } else {
                                        text.to_string()
                                    };
                                    
                                    draw::draw_text(
                                        &preview,
                                        block_x as i32 + 5,
                                        block_y as i32 + 15
                                    );
                                } else {
                                    draw::set_draw_color(Color::Red);
                                    draw::draw_text(
                                        "NO TEXT DATA",
                                        block_x as i32 + 5,
                                        block_y as i32 + 15
                                    );
                                }
                            }
                        }
                        
                        current_y += page_height + page_gap;
                    }
                    
                    draw::pop_clip();
                    
                    // Debug overlay
                    draw::set_draw_color(Color::from_rgb(0, 0, 0));
                    draw::set_font(Font::Helvetica, 10);
                    draw::draw_text(
                        &format!("Blocks: {} | Zoom: {:.0}%", doc.blocks.len(), zoom * 100.0),
                        w.x() + 5,
                        w.y() + w.h() - 5
                    );
                } else {
                    // No document loaded
                    draw::set_draw_color(Color::from_rgb(100, 100, 100));
                    draw::set_font(Font::Helvetica, 14);
                    draw::draw_text("No structured data loaded", w.x() + w.w()/2 - 80, w.y() + w.h()/2);
                }
            });
            
            // Clone references for event handling
            let doc_clone = widget.document.clone();
            let selected_clone = widget.selected_block.clone();
            let scroll_clone = widget.scroll_offset.clone();
            let zoom_clone = widget.zoom.clone();
            let dragging_clone = widget.dragging.clone();
            
            widget.inner.handle(move |_, event| {
                match event {
                    Event::Push => {
                        if let Some(_doc) = doc_clone.borrow().as_ref() {
                            let coords = app::event_coords();
                            // Commented out click-to-edit for now
                            /*
                            if let Some(idx) = find_block_at_coords(doc, coords, &scroll_clone, &zoom_clone) {
                                *selected_clone.borrow_mut() = Some(idx);
                                return true;
                            }
                            */
                        }
                        false
                    }
                    Event::MouseWheel => {
                        let dy = match app::event_dy() {
                            app::MouseWheel::Up => -20.0,
                            app::MouseWheel::Down => 20.0,
                            _ => 0.0,
                        };
                        
                        let mut scroll = scroll_clone.borrow_mut();
                        scroll.1 += dy;
                        if scroll.1 < 0.0 {
                            scroll.1 = 0.0;
                        }
                        true
                    }
                    Event::KeyDown => {
                        let key = app::event_key();
                        if key == Key::from_char('+') {
                            let mut zoom = zoom_clone.borrow_mut();
                            *zoom = (*zoom * 1.1).min(3.0);
                            true
                        } else if key == Key::from_char('-') {
                            let mut zoom = zoom_clone.borrow_mut();
                            *zoom = (*zoom * 0.9).max(0.5);
                            true
                        } else {
                            false
                        }
                    }
                    _ => false,
                }
            });
            
            widget
        }
        
        pub fn set_document(&mut self, doc: ferrules::Document) {
            *self.document.borrow_mut() = Some(doc);
            self.redraw();
        }
        
        pub fn clear(&mut self) {
            *self.document.borrow_mut() = None;
            *self.selected_block.borrow_mut() = None;
            *self.scroll_offset.borrow_mut() = (0.0, 0.0);
            *self.zoom.borrow_mut() = 1.0;
            self.redraw();
        }
    }
}

// ==================== POST PROCESSING ====================
mod post_processing {
    pub fn clean_text(text: &str) -> String {
        let mut cleaned = text.to_string();
        
        // Remove multiple spaces
        while cleaned.contains("  ") {
            cleaned = cleaned.replace("  ", " ");
        }
        
        // Fix line breaks
        cleaned = cleaned.replace("\r\n", "\n");
        cleaned = cleaned.replace("\r", "\n");
        
        // Remove common PDF artifacts
        cleaned = cleaned.replace("", "fi");
        cleaned = cleaned.replace("", "fl");
        cleaned = cleaned.replace("", "ff");
        cleaned = cleaned.replace("", "ffi");
        cleaned = cleaned.replace("", "ffl");
        
        // Fix common OCR errors
        cleaned = cleaned.replace("l\\/", "IV");
        cleaned = cleaned.replace("l\\/l", "M");
        
        cleaned.trim().to_string()
    }
    
    pub fn detect_tables(text: &str) -> Vec<String> {
        let lines: Vec<&str> = text.lines().collect();
        let mut tables = Vec::new();
        let mut current_table = Vec::new();
        let mut in_table = false;
        
        for line in lines {
            let pipe_count = line.matches('|').count();
            let tab_count = line.matches('\t').count();
            
            if pipe_count >= 2 || tab_count >= 2 {
                in_table = true;
                current_table.push(line.to_string());
            } else if in_table && line.trim().is_empty() {
                if !current_table.is_empty() {
                    tables.push(current_table.join("\n"));
                    current_table.clear();
                }
                in_table = false;
            } else if in_table {
                current_table.push(line.to_string());
            }
        }
        
        if !current_table.is_empty() {
            tables.push(current_table.join("\n"));
        }
        
        tables
    }
    
    pub fn analyze_layout(text: &str) -> String {
        let lines: Vec<&str> = text.lines().collect();
        let mut result = String::new();
        let mut current_section = String::new();
        
        for line in lines {
            if line.trim().is_empty() && !current_section.is_empty() {
                result.push_str(&format!("\n[SECTION]\n{}\n", current_section.trim()));
                current_section.clear();
            } else {
                current_section.push_str(line);
                current_section.push('\n');
            }
        }
        
        if !current_section.is_empty() {
            result.push_str(&format!("\n[SECTION]\n{}\n", current_section.trim()));
        }
        
        result
    }
    
    pub fn process_extracted_text(text: &str) -> String {
        let cleaned = clean_text(text);
        let with_layout = analyze_layout(&cleaned);
        
        let tables = detect_tables(&cleaned);
        let mut result = with_layout;
        
        if !tables.is_empty() {
            result.push_str("\n\n[DETECTED TABLES]\n");
            for (i, table) in tables.iter().enumerate() {
                result.push_str(&format!("\nTable {}:\n{}\n", i + 1, table));
            }
        }
        
        result
    }
}

// ==================== MAIN APPLICATION ====================
use constants::*;
use helpers::*;
use custom_widget::StructuredTextWidget;
use post_processing::*;

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
    compare_btn: Button,
    extracted_text_display: TextDisplay,
    extracted_text_buffer: TextBuffer,
    structured_view: StructuredTextWidget,
    structured_html_content: String,
    structured_json_data: Option<ferrules::Document>,
    compare_mode: bool,
    
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
        
        // Create UI components (simplified for refactoring demo)
        let pdf_frame = Frame::default();
        let status_label = Frame::default();
        let zoom_label = Frame::default();
        let page_label = Frame::default();
        let log_display = TextDisplay::default();
        let log_buffer = TextBuffer::default();
        let prev_btn = Button::default();
        let next_btn = Button::default();
        let extract_btn = Button::default();
        let structured_btn = Button::default();
        let compare_btn = Button::default();
        let extracted_text_display = TextDisplay::default();
        let extracted_text_buffer = TextBuffer::default();
        let structured_view = StructuredTextWidget::new(0, 0, 100, 100);
        
        window.end();
        window.show();
        
        let app_instance = Rc::new(RefCell::new(Self {
            app,
            window,
            pdf_frame,
            status_label,
            zoom_label,
            page_label,
            log_display,
            log_buffer,
            prev_btn,
            next_btn,
            extract_btn,
            structured_btn,
            compare_btn,
            extracted_text_display,
            extracted_text_buffer,
            structured_view,
            structured_html_content: String::new(),
            structured_json_data: None,
            compare_mode: false,
            pdf_path: None,
            current_page: 1,
            total_pages: 0,
            zoom_level: 1.0,
        }));
        
        app_instance
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Chonker5App::new();
    
    while app.borrow().app.wait() {
        // Event loop
    }
    
    Ok(())
}