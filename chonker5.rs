#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! minifb = "0.25"
//! pdfium-render = "0.8"
//! rfd = "0.15"
//! image = "0.25"
//! ```

use minifb::{Key, Window, WindowOptions, Scale, MouseMode, MouseButton};
use pdfium_render::prelude::*;
use std::path::PathBuf;
use std::time::Instant;

const WINDOW_WIDTH: usize = 1200;
const WINDOW_HEIGHT: usize = 800;
const TOP_BAR_HEIGHT: usize = 60;
const BUTTON_HEIGHT: usize = 40;
const BUTTON_WIDTH: usize = 100;
const PADDING: usize = 10;

// Colors
const COLOR_TEAL: u32 = 0xFF1ABC9C;
const COLOR_TEAL_DARK: u32 = 0xFF16A085;
const COLOR_WHITE: u32 = 0xFFFFFFFF;
const COLOR_BLACK: u32 = 0xFF000000;
const COLOR_GRAY_LIGHT: u32 = 0xFFE5E5E5;
const COLOR_GRAY_MED: u32 = 0xFFCCCCCC;
const COLOR_GRAY_DARK: u32 = 0xFF999999;

struct Button {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    text: String,
    enabled: bool,
}

impl Button {
    fn new(x: usize, y: usize, text: &str) -> Self {
        Self {
            x,
            y,
            width: BUTTON_WIDTH,
            height: BUTTON_HEIGHT,
            text: text.to_string(),
            enabled: true,
        }
    }

    fn contains(&self, x: usize, y: usize) -> bool {
        x >= self.x && x < self.x + self.width && 
        y >= self.y && y < self.y + self.height
    }

    fn draw(&self, buffer: &mut [u32], width: usize) {
        let color = if self.enabled { COLOR_TEAL } else { COLOR_GRAY_MED };
        let text_color = if self.enabled { COLOR_WHITE } else { COLOR_GRAY_DARK };
        
        // Draw button background
        for y in self.y..self.y.min(self.y + self.height) {
            for x in self.x..self.x.min(self.x + self.width) {
                if y < buffer.len() / width && x < width {
                    buffer[y * width + x] = color;
                }
            }
        }
        
        // Draw simple text (centered)
        let text_x = self.x + (self.width - self.text.len() * 8) / 2;
        let text_y = self.y + (self.height - 16) / 2;
        draw_text(buffer, width, text_x, text_y, &self.text, text_color);
    }
}

struct Chonker5 {
    window: Window,
    buffer: Vec<u32>,
    width: usize,
    height: usize,
    
    // PDF state
    pdfium: Option<Pdfium>,
    pdf_document: Option<PdfDocument<'static>>,
    current_pdf: Option<PathBuf>,
    current_page: usize,
    total_pages: usize,
    zoom_level: f32,
    
    // Rendered page
    pdf_pixels: Option<Vec<u32>>,
    pdf_width: usize,
    pdf_height: usize,
    
    // UI state
    buttons: Vec<Button>,
    status_message: String,
    needs_redraw: bool,
    
    // Viewport
    viewport_x: i32,
    viewport_y: i32,
    dragging: bool,
    drag_start_x: f32,
    drag_start_y: f32,
    drag_start_viewport_x: i32,
    drag_start_viewport_y: i32,
}

impl Chonker5 {
    fn new() -> Self {
        let mut window = Window::new(
            "🐹 CHONKER 5 - PDF Viewer",
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
            WindowOptions {
                resize: true,
                scale: Scale::X1,
                ..WindowOptions::default()
            },
        ).unwrap();

        window.limit_update_rate(Some(std::time::Duration::from_micros(16600))); // ~60fps
        
        let buffer = vec![COLOR_GRAY_LIGHT; WINDOW_WIDTH * WINDOW_HEIGHT];
        
        // Initialize pdfium
        let pdfium = if let Ok(bindings) = Pdfium::bind_to_system_library() {
            Some(Pdfium::new(bindings))
        } else if let Ok(lib_path) = std::env::var("PDFIUM_DYNAMIC_LIB_PATH") {
            Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path(&lib_path))
                .map(|bindings| Pdfium::new(bindings))
                .ok()
        } else {
            eprintln!("Warning: PDFium not found. PDF rendering will not work.");
            eprintln!("Install with: brew install pdfium");
            None
        };
        
        // Create buttons
        let mut buttons = Vec::new();
        buttons.push(Button::new(PADDING, PADDING, "Open"));
        buttons.push(Button::new(PADDING + BUTTON_WIDTH + 10, PADDING, "Prev"));
        buttons.push(Button::new(PADDING + (BUTTON_WIDTH + 10) * 2, PADDING, "Next"));
        buttons.push(Button::new(PADDING + (BUTTON_WIDTH + 10) * 3, PADDING, "Zoom In"));
        buttons.push(Button::new(PADDING + (BUTTON_WIDTH + 10) * 4, PADDING, "Zoom Out"));
        
        // Initially disable navigation buttons
        buttons[1].enabled = false;
        buttons[2].enabled = false;
        
        Self {
            window,
            buffer,
            width: WINDOW_WIDTH,
            height: WINDOW_HEIGHT,
            pdfium,
            pdf_document: None,
            current_pdf: None,
            current_page: 0,
            total_pages: 0,
            zoom_level: 1.0,
            pdf_pixels: None,
            pdf_width: 0,
            pdf_height: 0,
            buttons,
            status_message: if pdfium.is_some() {
                "Ready! Click 'Open' to load a PDF".to_string()
            } else {
                "⚠️ PDFium not found - PDF rendering disabled".to_string()
            },
            needs_redraw: true,
            viewport_x: 0,
            viewport_y: 0,
            dragging: false,
            drag_start_x: 0.0,
            drag_start_y: 0.0,
            drag_start_viewport_x: 0,
            drag_start_viewport_y: 0,
        }
    }

    fn run(&mut self) {
        while self.window.is_open() && !self.window.is_key_down(Key::Escape) {
            // Handle window resize
            let (new_width, new_height) = self.window.get_size();
            if new_width != self.width || new_height != self.height {
                self.width = new_width;
                self.height = new_height;
                self.buffer.resize(new_width * new_height, COLOR_GRAY_LIGHT);
                self.needs_redraw = true;
            }
            
            // Handle input
            self.handle_input();
            
            // Render
            if self.needs_redraw {
                self.render();
                self.needs_redraw = false;
            }
            
            // Update window
            self.window
                .update_with_buffer(&self.buffer, self.width, self.height)
                .unwrap();
        }
    }

    fn handle_input(&mut self) {
        // Handle mouse clicks on buttons
        if let Some((mouse_x, mouse_y)) = self.window.get_mouse_pos(MouseMode::Discard) {
            let mouse_x = mouse_x as usize;
            let mouse_y = mouse_y as usize;
            
            // Handle dragging
            if self.window.get_mouse_down(MouseButton::Left) {
                if !self.dragging && mouse_y > TOP_BAR_HEIGHT && self.pdf_pixels.is_some() {
                    self.dragging = true;
                    self.drag_start_x = mouse_x as f32;
                    self.drag_start_y = mouse_y as f32;
                    self.drag_start_viewport_x = self.viewport_x;
                    self.drag_start_viewport_y = self.viewport_y;
                } else if self.dragging {
                    let dx = mouse_x as f32 - self.drag_start_x;
                    let dy = mouse_y as f32 - self.drag_start_y;
                    self.viewport_x = self.drag_start_viewport_x + dx as i32;
                    self.viewport_y = self.drag_start_viewport_y + dy as i32;
                    self.needs_redraw = true;
                }
            } else {
                if self.dragging {
                    self.dragging = false;
                }
                
                // Check button clicks
                if self.window.get_mouse_down(MouseButton::Left) {
                    for (i, button) in self.buttons.iter().enumerate() {
                        if button.enabled && button.contains(mouse_x, mouse_y) {
                            match i {
                                0 => self.open_file(),
                                1 => self.prev_page(),
                                2 => self.next_page(),
                                3 => self.zoom_in(),
                                4 => self.zoom_out(),
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
        
        // Keyboard shortcuts
        if self.window.is_key_pressed(Key::O, minifb::KeyRepeat::No) {
            self.open_file();
        }
        if self.window.is_key_pressed(Key::Left, minifb::KeyRepeat::No) {
            self.prev_page();
        }
        if self.window.is_key_pressed(Key::Right, minifb::KeyRepeat::No) {
            self.next_page();
        }
        if self.window.is_key_pressed(Key::Equal, minifb::KeyRepeat::No) {
            self.zoom_in();
        }
        if self.window.is_key_pressed(Key::Minus, minifb::KeyRepeat::No) {
            self.zoom_out();
        }
    }

    fn render(&mut self) {
        // Clear buffer
        self.buffer.fill(COLOR_GRAY_LIGHT);
        
        // Draw top bar
        for y in 0..TOP_BAR_HEIGHT {
            for x in 0..self.width {
                self.buffer[y * self.width + x] = COLOR_TEAL;
            }
        }
        
        // Draw buttons
        for button in &self.buttons {
            button.draw(&mut self.buffer, self.width);
        }
        
        // Draw status
        draw_text(&mut self.buffer, self.width, self.width - 400, 20, &self.status_message, COLOR_WHITE);
        
        // Draw page info
        if self.total_pages > 0 {
            let page_info = format!("Page {} / {}", self.current_page + 1, self.total_pages);
            draw_text(&mut self.buffer, self.width, self.width - 150, 20, &page_info, COLOR_WHITE);
        }
        
        // Draw PDF content
        if let Some(pdf_pixels) = &self.pdf_pixels {
            self.draw_pdf_page(pdf_pixels);
        } else if self.current_pdf.is_some() {
            draw_text(&mut self.buffer, self.width, self.width / 2 - 100, self.height / 2, "Loading PDF...", COLOR_BLACK);
        } else {
            draw_text(&mut self.buffer, self.width, self.width / 2 - 150, self.height / 2, "Click 'Open' to load a PDF", COLOR_BLACK);
        }
    }

    fn draw_pdf_page(&mut self, pdf_pixels: &[u32]) {
        let viewport_top = TOP_BAR_HEIGHT;
        let viewport_height = self.height - TOP_BAR_HEIGHT;
        
        // Calculate scaled dimensions
        let scaled_width = (self.pdf_width as f32 * self.zoom_level) as usize;
        let scaled_height = (self.pdf_height as f32 * self.zoom_level) as usize;
        
        // Draw the PDF page with scaling
        for y in 0..viewport_height {
            for x in 0..self.width {
                let screen_y = y + viewport_top;
                
                // Calculate source coordinates with viewport offset
                let src_x = ((x as i32 - self.viewport_x) as f32 / self.zoom_level) as i32;
                let src_y = ((y as i32 - self.viewport_y) as f32 / self.zoom_level) as i32;
                
                if src_x >= 0 && src_x < self.pdf_width as i32 && 
                   src_y >= 0 && src_y < self.pdf_height as i32 {
                    let src_idx = src_y as usize * self.pdf_width + src_x as usize;
                    if src_idx < pdf_pixels.len() {
                        self.buffer[screen_y * self.width + x] = pdf_pixels[src_idx];
                    }
                }
            }
        }
    }

    fn open_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("PDF", &["pdf"])
            .pick_file()
        {
            self.load_pdf(path);
        }
    }

    fn load_pdf(&mut self, path: PathBuf) {
        self.status_message = format!("Loading {}...", path.file_name().unwrap_or_default().to_string_lossy());
        self.needs_redraw = true;
        
        // Clear previous document
        self.pdf_document = None;
        self.pdf_pixels = None;
        
        if let Some(pdfium) = &self.pdfium {
            match pdfium.load_pdf_from_file(&path, None) {
                Ok(document) => {
                    self.total_pages = document.pages().len() as usize;
                    self.current_page = 0;
                    self.current_pdf = Some(path);
                    
                    // Store document
                    let static_doc: PdfDocument<'static> = unsafe { 
                        std::mem::transmute(document) 
                    };
                    self.pdf_document = Some(static_doc);
                    
                    self.status_message = format!("Loaded! {} pages", self.total_pages);
                    
                    // Enable navigation buttons
                    self.buttons[1].enabled = self.current_page > 0;
                    self.buttons[2].enabled = self.current_page < self.total_pages - 1;
                    
                    // Reset viewport
                    self.viewport_x = 0;
                    self.viewport_y = 0;
                    
                    self.needs_redraw = true;
                }
                Err(e) => {
                    self.status_message = format!("Failed to load PDF: {}", e);
                    self.needs_redraw = true;
                    return;
                }
            }
            
            // Render the first page
            self.render_current_page();
        }
    }

    fn render_current_page(&mut self) {
        if let Some(document) = &self.pdf_document {
            if let Ok(page) = document.pages().get(self.current_page as u16) {
                // Calculate render size
                let render_scale = 2.0; // Render at 2x for better quality
                
                let config = PdfRenderConfig::new()
                    .set_target_width((800.0 * render_scale) as i32)
                    .render_form_data(true)
                    .render_annotations(true);
                
                // Render to bitmap
                if let Ok(bitmap) = page.render_with_config(&config) {
                    let image_buffer = bitmap.as_image();
                    
                    self.pdf_width = image_buffer.width() as usize;
                    self.pdf_height = image_buffer.height() as usize;
                    
                    // Convert from BGRA to RGB u32
                    let mut pixels = Vec::with_capacity(self.pdf_width * self.pdf_height);
                    
                    for chunk in image_buffer.as_bytes().chunks(4) {
                        let b = chunk[0] as u32;
                        let g = chunk[1] as u32;
                        let r = chunk[2] as u32;
                        let a = chunk[3] as u32;
                        
                        // Blend with white background
                        let alpha = a as f32 / 255.0;
                        let r = (r as f32 * alpha + 255.0 * (1.0 - alpha)) as u32;
                        let g = (g as f32 * alpha + 255.0 * (1.0 - alpha)) as u32;
                        let b = (b as f32 * alpha + 255.0 * (1.0 - alpha)) as u32;
                        
                        pixels.push(0xFF000000 | (r << 16) | (g << 8) | b);
                    }
                    
                    self.pdf_pixels = Some(pixels);
                    self.needs_redraw = true;
                }
            }
        }
    }

    fn next_page(&mut self) {
        if self.current_page < self.total_pages - 1 {
            self.current_page += 1;
            self.render_current_page();
            self.buttons[1].enabled = true;
            self.buttons[2].enabled = self.current_page < self.total_pages - 1;
            self.status_message = format!("Page {} of {}", self.current_page + 1, self.total_pages);
            self.needs_redraw = true;
        }
    }

    fn prev_page(&mut self) {
        if self.current_page > 0 {
            self.current_page -= 1;
            self.render_current_page();
            self.buttons[1].enabled = self.current_page > 0;
            self.buttons[2].enabled = true;
            self.status_message = format!("Page {} of {}", self.current_page + 1, self.total_pages);
            self.needs_redraw = true;
        }
    }

    fn zoom_in(&mut self) {
        self.zoom_level = (self.zoom_level * 1.2).min(4.0);
        self.status_message = format!("Zoom: {}%", (self.zoom_level * 100.0) as i32);
        self.needs_redraw = true;
    }

    fn zoom_out(&mut self) {
        self.zoom_level = (self.zoom_level / 1.2).max(0.25);
        self.status_message = format!("Zoom: {}%", (self.zoom_level * 100.0) as i32);
        self.needs_redraw = true;
    }
}

// Simple text drawing function
fn draw_text(buffer: &mut [u32], width: usize, x: usize, y: usize, text: &str, color: u32) {
    // This is a placeholder - in a real implementation you'd use a proper font renderer
    // For now, just draw a simple representation
    for (i, _ch) in text.chars().enumerate() {
        let char_x = x + i * 8;
        if char_x < width && y < buffer.len() / width {
            // Draw a simple box for each character
            for dy in 0..16 {
                for dx in 0..6 {
                    let px = char_x + dx;
                    let py = y + dy;
                    if px < width && py < buffer.len() / width {
                        buffer[py * width + px] = color;
                    }
                }
            }
        }
    }
}

fn main() {
    let mut app = Chonker5::new();
    app.run();
}