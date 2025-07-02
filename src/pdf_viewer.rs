#[cfg(feature = "gui")]
use eframe::egui::{self, *};
use std::path::{Path, PathBuf};
use std::process::Command;
#[cfg(feature = "gui")]
use image;
use std::collections::HashMap;
use crate::coordinate_mapping::CoordinateMapper;

/// PDF Viewer component that uses pdftoppm for rendering at 72 DPI
/// This ensures perfect coordinate alignment with Docling's extraction
pub struct PdfViewer {
    current_file: Option<PathBuf>,
    current_page: usize,
    page_count: usize,
    is_loaded: bool,
    page_cache: HashMap<usize, TextureHandle>,
    
    // Coordinate mapping system
    pub coordinate_mapper: CoordinateMapper,
    pub debug_overlay: bool,
    zoom_level: f32,
    // Rendering state tracking to prevent infinite loops
    rendering_state: HashMap<usize, bool>,
}

impl PdfViewer {
    pub fn new() -> Self {
        Self {
            current_file: None,
            current_page: 0,
            page_count: 0,
            is_loaded: false,
            page_cache: HashMap::new(),
            coordinate_mapper: CoordinateMapper::new(),
            debug_overlay: false,
            zoom_level: 1.0,
            rendering_state: HashMap::new(),
        }
    }
    
    pub fn render(&mut self, ui: &mut Ui) {
        if self.is_loaded {
            self.render_pdf_content(ui);
        } else {
            self.render_empty_state(ui);
        }
    }
    
    fn render_empty_state(&mut self, ui: &mut Ui) {
        ui.allocate_ui_with_layout(
            ui.available_size(),
            Layout::centered_and_justified(Direction::TopDown),
            |ui| {
                ui.heading("📄 PDF Viewer");
                ui.add_space(20.0);
                ui.label("No PDF loaded");
                ui.label("Use Ctrl+O to load a document");
                ui.add_space(10.0);
                ui.label("🚀 Now using pdftoppm at 72 DPI for perfect coordinate alignment!");
            },
        );
    }
    
    fn render_pdf_content(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.heading("📄 PDF Content");
            ui.separator();
            
            if let Some(ref file) = self.current_file {
                ui.label(format!("File: {}", file.file_name().unwrap_or_default().to_string_lossy()));
                ui.label(format!("Pages: {}", self.page_count));
                ui.label(format!("Current page: {}/{}", self.current_page + 1, self.page_count));
                
                ui.separator();
                
                // Controls row
                ui.horizontal(|ui| {
                    // Page navigation
                    if ui.button("⬅ Previous").clicked() && self.current_page > 0 {
                        self.current_page -= 1;
                    }
                    
                    ui.label(format!("{} / {}", self.current_page + 1, self.page_count));
                    
                    if ui.button("➡ Next").clicked() && self.current_page < self.page_count.saturating_sub(1) {
                        self.current_page += 1;
                    }
                    
                    ui.separator();
                    
                    // Zoom controls
                    ui.label("Zoom:");
                    if ui.button("-").clicked() {
                        self.zoom_level = (self.zoom_level - 0.1).max(0.1);
                    }
                    ui.label(format!("{:.0}%", self.zoom_level * 100.0));
                    if ui.button("+").clicked() {
                        self.zoom_level = (self.zoom_level + 0.1).min(5.0);
                    }
                    
                    ui.separator();
                    
                    // Debug overlay toggle
                    ui.checkbox(&mut self.debug_overlay, "Debug overlay");
                });
                
                ui.separator();
                
                // Render actual PDF page (only if not already cached and not currently rendering)
                let needs_rendering = !self.page_cache.contains_key(&self.current_page) 
                    && !self.rendering_state.get(&self.current_page).unwrap_or(&false);
                
                if needs_rendering {
                    // Mark this page as currently being rendered to prevent loops
                    self.rendering_state.insert(self.current_page, true);
                    
                    // Show rendering message
                    ui.label("🖼️ Rendering PDF page...");
                    
                    // Try to render the current page
                    match self.render_page_to_cache(ui.ctx(), self.current_page) {
                        Ok(()) => {
                            // Mark rendering as complete
                            self.rendering_state.insert(self.current_page, false);
                            // No need to request repaint - the UI will update naturally
                        },
                        Err(e) => {
                            // Mark rendering as failed (not in progress)
                            self.rendering_state.insert(self.current_page, false);
                            ui.label(format!("⚠️ Failed to render page: {}", e));
                        }
                    }
                } else if *self.rendering_state.get(&self.current_page).unwrap_or(&false) {
                    // Currently rendering - show status without triggering another render
                    ui.label("🖼️ Rendering in progress...");
                }
                
                // Display the cached page with coordinate mapping
                if let Some(texture) = self.page_cache.get(&self.current_page) {
                    // Calculate size to fit in available space with zoom
                    let available_width = ui.available_width();
                    let available_height = ui.available_height() - 50.0; // Leave space for controls
                    
                    let texture_size = texture.size_vec2();
                    let base_scale = (available_width / texture_size.x).min(available_height / texture_size.y).min(1.0);
                    let final_scale = base_scale * self.zoom_level;
                    let display_size = texture_size * final_scale;
                    
                    ScrollArea::both()
                        .max_height(available_height)
                        .show(ui, |ui| {
                            // Display PDF as clickable image
                            let response = ui.add(
                                Image::from_texture(texture)
                                    .fit_to_exact_size(display_size)
                                    .sense(Sense::click())
                            );
                            
                            // Handle PDF clicks for coordinate mapping
                            if response.clicked() {
                                if let Some(click_pos) = response.interact_pointer_pos() {
                                    let relative_vec = click_pos - response.rect.min;
                                    let relative_pos = egui::pos2(relative_vec.x, relative_vec.y);
                                    let scale_factor = egui::vec2(final_scale, final_scale);
                                    
                                    if let Some(region_index) = self.coordinate_mapper.handle_pdf_click(relative_pos, scale_factor) {
                                        tracing::info!("Clicked PDF region: {}", region_index);
                                        // TODO: Send signal to markdown editor to highlight corresponding text
                                    }
                                }
                            }
                            
                            // Render debug overlay if enabled
                            if self.debug_overlay {
                                let scale_factor = egui::vec2(final_scale, final_scale);
                                self.coordinate_mapper.render_debug_overlay(ui, response.rect, scale_factor);
                            }
                        });
                } else {
                    ui.allocate_ui_with_layout(
                        [ui.available_width(), 400.0].into(),
                        Layout::centered_and_justified(Direction::TopDown),
                        |ui| {
                            ui.label("📄 Rendering PDF page...");
                            ui.label("(Using pdftoppm at 72 DPI)");
                        },
                    );
                }
            }
        });
    }
    
    pub fn load_pdf(&mut self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        // First check if the PDF exists
        if !path.exists() {
            return Err("PDF file does not exist".into());
        }
        
        println!("📄 Loading PDF: {}", path.display());
        
        // Use pdfinfo (part of poppler) to get page count
        let output = Command::new("pdfinfo")
            .arg(path)
            .output()?;
            
        if !output.status.success() {
            return Err(format!("Failed to read PDF info: {}", String::from_utf8_lossy(&output.stderr)).into());
        }
        
        // Parse page count from pdfinfo output
        let info_text = String::from_utf8_lossy(&output.stdout);
        let page_count = info_text
            .lines()
            .find(|line| line.starts_with("Pages:"))
            .and_then(|line| line.split_whitespace().nth(1))
            .and_then(|count| count.parse::<usize>().ok())
            .unwrap_or(1); // Default to 1 page if parsing fails
            
        println!("📄 PDF loaded: {} pages", page_count);
        
        self.page_count = page_count;
        self.current_file = Some(path.to_path_buf());
        self.current_page = 0;
        self.is_loaded = true;
        self.page_cache.clear(); // Clear any old cached pages
        self.rendering_state.clear(); // Clear rendering state
        Ok(())
    }

    fn render_page_to_cache(&mut self, ctx: &Context, page_num: usize) -> Result<(), Box<dyn std::error::Error>> {
        let pdf_path = self.current_file.as_ref().ok_or("No PDF loaded")?;
        let temp_dir = std::env::temp_dir();
        let temp_prefix = temp_dir.join(format!("chonker_page_{}", page_num + 1));
        
        println!("🖼️ Rendering PDF page {} at 72 DPI...", page_num + 1);
        println!("📁 Temp prefix: {}", temp_prefix.display());
        
        // Use pdftoppm to render the specific page at 72 DPI (matches Docling)
        let page_1_based = page_num + 1; // Convert to 1-based indexing for pdftoppm
        let output = Command::new("pdftoppm")
            .args(["-png", "-r", "72"]) // 72 DPI for coordinate alignment with Docling
            .args(["-f", &page_1_based.to_string(), "-l", &page_1_based.to_string()])
            .arg(pdf_path)
            .arg(&temp_prefix)
            .output()?;
            
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            println!("❌ pdftoppm stderr: {}", stderr);
            return Err(format!("pdftoppm failed: {}", stderr).into());
        }
        
        // pdftoppm creates files like "prefix-1.png" (no zero padding for single digits)
        let png_path = format!("{}-{}.png", temp_prefix.to_string_lossy(), page_1_based);
        println!("🔍 Looking for PNG at: {}", png_path);
        
        // Check what files were actually created
        if let Ok(entries) = std::fs::read_dir(&temp_dir) {
            println!("📂 Files in temp dir:");
            for entry in entries {
                if let Ok(entry) = entry {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name.starts_with("chonker_page") {
                        println!("  - {}", name);
                    }
                }
            }
        }
        
        if !std::path::Path::new(&png_path).exists() {
            return Err(format!("Generated PNG not found: {}\nExpected: {}", png_path, png_path).into());
        }
        
        // Load the PNG into memory
        let img_bytes = std::fs::read(&png_path)?;
        let dynamic_img = image::load_from_memory(&img_bytes)?;
        let rgba_img = dynamic_img.to_rgba8();
        let size = [rgba_img.width() as usize, rgba_img.height() as usize];
        
        // Create egui color image
        let color_image = ColorImage::from_rgba_unmultiplied(size, &rgba_img);
        
        // Load as texture
        let texture_name = format!("pdf_page_{}", page_num);
        let texture = ctx.load_texture(texture_name, color_image, TextureOptions::default());
        
        // Cache the texture
        self.page_cache.insert(page_num, texture);
        
        // Clean up temporary file
        if let Err(e) = std::fs::remove_file(&png_path) {
            eprintln!("⚠️ Failed to clean up temp file {}: {}", png_path, e);
        }
        
        println!("✅ PDF page {} rendered: {}x{}", page_num + 1, size[0], size[1]);
        Ok(())
    }
    
    pub fn get_page_count(&self) -> usize {
        self.page_count
    }
    
    pub fn get_current_page(&self) -> usize {
        self.current_page
    }
    
    /// Load coordinate mapping data from Docling extraction
    pub fn load_coordinate_mapping(&mut self, docling_json_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        self.coordinate_mapper.load_docling_output(docling_json_path)?;
        
        // If we have a loaded page, update the text regions
        if let Some(texture) = self.page_cache.get(&self.current_page) {
            let image_size = texture.size_vec2();
            self.coordinate_mapper.generate_text_regions(image_size);
            tracing::info!("Loaded coordinate mapping with {} regions", self.coordinate_mapper.text_regions.len());
        }
        
        Ok(())
    }
    
    /// Get the currently selected text from coordinate mapping
    pub fn get_selected_text(&self) -> Option<&str> {
        self.coordinate_mapper.get_selected_text()
    }
    
    /// Handle text selection from markdown editor to highlight PDF region
    pub fn highlight_text_region(&mut self, text_index: usize) -> Option<usize> {
        self.coordinate_mapper.handle_text_selection(text_index)
    }
}
