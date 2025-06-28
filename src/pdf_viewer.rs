use egui::*;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::sync::Arc;
use pdfium_render::prelude::*;
use image::GenericImageView;

pub struct PdfViewer {
    // Core PDF state
    pdfium: Option<Pdfium>,
    current_pdf: Option<PdfDocument<'static>>,
    current_file: Option<PathBuf>,
    current_page: usize,
    page_count: usize,
    zoom: f32,
    
    // Caching for performance
    page_cache: HashMap<usize, egui::TextureHandle>,
    cache_size_limit: usize,
    
    // Selection state
    selection_start: Option<Pos2>,
    selection_end: Option<Pos2>,
    selection_active: bool,
    
    // UI state
    auto_hide_menu: bool,
    menu_animation: f32,
    scroll_offset: Vec2,
}

impl PdfViewer {
    pub fn new() -> Self {
        // Initialize pdfium - this is the critical step
        let pdfium = match Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./"))
            .or_else(|_| Pdfium::bind_to_system_library())
        {
            Ok(bindings) => Some(Pdfium::new(bindings)),
            Err(e) => {
                eprintln!("Failed to initialize Pdfium: {}", e);
                None
            }
        };
        
        Self {
            pdfium,
            current_pdf: None,
            current_file: None,
            current_page: 0,
            page_count: 0,
            zoom: 1.0,
            page_cache: HashMap::new(),
            cache_size_limit: 10, // Cache up to 10 pages
            selection_start: None,
            selection_end: None,
            selection_active: false,
            auto_hide_menu: true,
            menu_animation: 0.0,
            scroll_offset: Vec2::ZERO,
        }
    }
    
    pub fn render(&mut self, ui: &mut Ui) {
        if self.current_pdf.is_some() {
            // Clone the necessary data to avoid borrowing issues
            let has_pdf = true;
            if has_pdf {
                self.render_pdf_viewer_internal(ui);
            }
        } else {
            self.render_empty_state(ui);
        }
    }
    
    fn render_empty_state(&mut self, ui: &mut Ui) {
        ui.allocate_ui_with_layout(
            ui.available_size(),
            Layout::centered_and_justified(Direction::TopDown),
            |ui| {
                ui.heading("PDF Viewer");
                ui.add_space(20.0);
                ui.label("No PDF loaded");
                ui.label("Use File → Open PDF to load a document");
            },
        );
    }
    
    fn render_pdf_viewer_internal(&mut self, ui: &mut Ui) {
        // Auto-hide menu animation
        self.update_menu_animation(ui);
        
        // Render menu if visible
        if self.menu_animation > 0.01 {
            self.render_pdf_menu(ui);
        }
        
        // Main PDF display area
        ScrollArea::both()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                // Get or render the current page
                if !self.page_cache.contains_key(&self.current_page) {
                    self.render_current_page_to_cache(ui.ctx());
                }
                
                if let Some(texture) = self.page_cache.get(&self.current_page) {
                    let response = ui.add(Image::from_texture(texture).fit_to_original_size(1.0));
                    let rect = response.rect;
                    
                    // Handle text selection
                    self.handle_selection_input(ui, rect, &response);
                    
                    // Draw selection overlay
                    self.draw_selection_overlay(ui, rect);
                }
            });
    }
    
    fn render_pdf_viewer(&mut self, ui: &mut Ui, pdf: &PdfDocument) {
        // Auto-hide menu animation
        self.update_menu_animation(ui);
        
        // Render menu if visible
        if self.menu_animation > 0.01 {
            self.render_pdf_menu(ui);
        }
        
        // Main PDF display area
        ScrollArea::both()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                self.render_pdf_page(ui, pdf);
            });
    }
    
    fn update_menu_animation(&mut self, ui: &mut Ui) {
        if self.auto_hide_menu {
            let mouse_y = ui.input(|i| {
                i.pointer.hover_pos().map(|p| p.y).unwrap_or(f32::MAX)
            });
            
            let target = if mouse_y < 50.0 { 1.0 } else { 0.0 };
            let dt = ui.input(|i| i.unstable_dt);
            self.menu_animation += (target - self.menu_animation) * dt * 5.0;
            
            if self.menu_animation != target {
                ui.ctx().request_repaint();
            }
        } else {
            self.menu_animation = 1.0;
        }
    }
    
    fn render_pdf_menu(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.set_opacity(self.menu_animation);
            
            // Page navigation
            if ui.button("◀").clicked() && self.current_page > 0 {
                self.current_page -= 1;
                self.render_current_page();
            }
            
            ui.label(format!("Page {} of {}", self.current_page + 1, self.page_count));
            
            if ui.button("▶").clicked() && self.current_page < self.page_count - 1 {
                self.current_page += 1;
                self.render_current_page();
            }
            
            ui.separator();
            
            // Zoom controls
            if ui.button("-").clicked() {
                self.zoom = (self.zoom - 0.1).max(0.5);
                self.render_current_page();
            }
            
            ui.label(format!("{}%", (self.zoom * 100.0) as i32));
            
            if ui.button("+").clicked() {
                self.zoom = (self.zoom + 0.1).min(3.0);
                self.render_current_page();
            }
            
            if ui.button("Fit").clicked() {
                if let Some(ref pdf) = self.current_pdf {
                if let Ok(page) = pdf.pages().get(self.current_page.try_into().unwrap_or(0)) {
                        let page_width = page.width().value;
                        let available_width = ui.available_width();
                        self.zoom = available_width / page_width;
                        self.render_current_page();
                    }
                }
            }
        });
    }
    
    fn render_pdf_page(&mut self, ui: &mut Ui, _pdf: &PdfDocument) {
        // Get or render the current page
        if !self.page_cache.contains_key(&self.current_page) {
            self.render_current_page_to_cache(ui.ctx());
        }
        
        if let Some(texture) = self.page_cache.get(&self.current_page) {
            let response = ui.add(Image::from_texture(texture).fit_to_original_size(1.0));
            let rect = response.rect;
            
            // Handle text selection
            self.handle_selection_input(ui, rect, &response);
            
            // Draw selection overlay
            self.draw_selection_overlay(ui, rect);
        }
    }
    
    fn render_current_page(&mut self) {
        // Clear cache entry for current page to force re-render
        self.page_cache.remove(&self.current_page);
    }
    
    fn render_current_page_to_cache(&mut self, ctx: &Context) {
        if let (Some(ref pdfium), Some(ref pdf)) = (&self.pdfium, &self.current_pdf) {
            if let Ok(page) = pdf.pages().get(self.current_page.try_into().unwrap_or(0)) {
                // Calculate render size
                let page_width = page.width().value;
                let page_height = page.height().value;
                let render_width = (page_width * self.zoom * 2.0) as i32; // 2x for high DPI
                let render_height = (page_height * self.zoom * 2.0) as i32;
                
                // Render page to bitmap
                let config = PdfRenderConfig::new()
                    .set_target_width(render_width)
                    .set_target_height(render_height)
                    .rotate_if_landscape(PdfPageRenderRotation::None, false);
                
                if let Ok(bitmap) = page.render_with_config(&config) {
                    // Convert to egui texture
                    let image = bitmap.as_image();
                    let size = [image.width() as usize, image.height() as usize];
                    let pixels: Vec<egui::Color32> = image
                        .pixels()
                        .map(|p| egui::Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
                        .collect();
                    
                    let color_image = egui::ColorImage {
                        size,
                        pixels,
                    };
                    
                    let texture = ctx.load_texture(
                        format!("pdf_page_{}", self.current_page),
                        color_image,
                        TextureOptions::default(),
                    );
                    
                    // Cache management - remove old pages if cache is full
                    if self.page_cache.len() >= self.cache_size_limit {
                        // Remove a page that's not the current or adjacent pages
                        let pages_to_keep: Vec<usize> = (self.current_page.saturating_sub(2)
                            ..=(self.current_page + 2).min(self.page_count - 1))
                            .collect();
                        
                        self.page_cache.retain(|&k, _| pages_to_keep.contains(&k));
                    }
                    
                    self.page_cache.insert(self.current_page, texture);
                }
            }
        }
    }
    
    fn handle_selection_input(&mut self, ui: &mut Ui, rect: Rect, response: &Response) {
        if response.clicked() {
            if let Some(hover_pos) = ui.input(|i| i.pointer.hover_pos()) {
                self.selection_start = Some(hover_pos - rect.min.to_vec2());
                self.selection_active = true;
                self.selection_end = None;
            }
        }
        
        if self.selection_active {
            if let Some(hover_pos) = ui.input(|i| i.pointer.hover_pos()) {
                self.selection_end = Some(hover_pos - rect.min.to_vec2());
            }
            
            if ui.input(|i| i.pointer.primary_released()) {
                self.selection_active = false;
                self.extract_selection();
            }
        }
    }
    
    fn draw_selection_overlay(&self, ui: &mut Ui, rect: Rect) {
        if let (Some(start), Some(end)) = (self.selection_start, self.selection_end) {
            let selection_rect = Rect::from_two_pos(
                rect.min + start.to_vec2(),
                rect.min + end.to_vec2(),
            );
            
            ui.painter().rect_filled(
                selection_rect,
                0.0,
                Color32::from_rgba_premultiplied(0, 100, 255, 50),
            );
        }
    }
    
    fn extract_selection(&self) {
        // TODO: Implement text extraction from selection bounds
        // This would involve:
        // 1. Converting selection coordinates to PDF coordinates
        // 2. Using pdfium to extract text from the selected region
        // 3. Triggering a callback with the extracted text
        println!("Selection extraction not yet implemented");
    }
    
    pub fn load_pdf(&mut self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref pdfium) = self.pdfium {
            match pdfium.load_pdf_from_file(path, None) {
                Ok(pdf) => {
                    self.page_count = pdf.pages().len() as usize;
                    self.current_page = 0;
                    self.current_file = Some(path.to_path_buf());
                    self.current_pdf = Some(pdf);
                    self.page_cache.clear();
                    Ok(())
                }
                Err(e) => Err(format!("Failed to load PDF: {}", e).into())
            }
        } else {
            Err("Pdfium not initialized".into())
        }
    }
    
    pub fn is_loaded(&self) -> bool {
        self.current_pdf.is_some()
    }
    
    pub fn get_page_count(&self) -> usize {
        self.page_count
    }
    
    pub fn get_current_page(&self) -> usize {
        self.current_page
    }
}
