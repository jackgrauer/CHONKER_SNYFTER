#[cfg(all(feature = "mupdf", feature = "gui"))]
use mupdf_sys as fz;
#[cfg(feature = "gui")]
use eframe::egui::{self, *};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::ffi::{CString, CStr};
use std::ptr;
#[cfg(feature = "gui")]
use crate::coordinate_mapping::CoordinateMapper;

/// Memory-managed PDF viewer using MuPDF for high performance
pub struct MuPdfViewer {
    current_file: Option<PathBuf>,
    current_page: usize,
    page_count: usize,
    is_loaded: bool,
    
    // MuPDF context and document - wrapped in Option for safe cleanup
    #[cfg(feature = "mupdf")]
    context: Option<*mut fz::fz_context>,
    #[cfg(feature = "mupdf")]
    document: Option<*mut fz::fz_document>,
    
    // Texture cache with memory limits
    page_cache: HashMap<usize, TextureHandle>,
    cache_memory_limit: usize, // Memory limit in bytes
    cache_memory_used: usize,  // Current memory usage
    
    // Coordinate mapping system
    pub coordinate_mapper: CoordinateMapper,
    pub debug_overlay: bool,
    zoom_level: f32,
    
    // Performance monitoring
    render_times: Vec<std::time::Duration>,
    memory_stats: MemoryStats,
}

#[derive(Debug, Clone)]
struct MemoryStats {
    peak_usage: usize,
    current_usage: usize,
    cache_hits: usize,
    cache_misses: usize,
    texture_count: usize,
}

impl Default for MemoryStats {
    fn default() -> Self {
        Self {
            peak_usage: 0,
            current_usage: 0,
            cache_hits: 0,
            cache_misses: 0,
            texture_count: 0,
        }
    }
}

impl MuPdfViewer {
    pub fn new() -> Self {
        println!("🚀 Initializing MuPDF viewer with memory management...");
        
        Self {
            current_file: None,
            current_page: 0,
            page_count: 0,
            is_loaded: false,
            
            #[cfg(feature = "mupdf")]
            context: None,
            #[cfg(feature = "mupdf")]
            document: None,
            
            page_cache: HashMap::new(),
            cache_memory_limit: 256 * 1024 * 1024, // 256MB cache limit
            cache_memory_used: 0,
            
            coordinate_mapper: CoordinateMapper::new(),
            debug_overlay: false,
            zoom_level: 1.0,
            
            render_times: Vec::new(),
            memory_stats: MemoryStats::default(),
        }
    }
    
    pub fn load_pdf(&mut self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        println!("📄 Loading PDF with MuPDF: {}", path.display());
        let start_time = std::time::Instant::now();
        
        #[cfg(feature = "mupdf")]
        {
            // First check if file exists
            if !path.exists() {
                return Err("PDF file does not exist".into());
            }
            
            // Clean up any existing resources
            self.cleanup_mupdf_resources();
            
            // Initialize MuPDF context with proper version string
            let version = std::ffi::CString::new("1.26.0").unwrap();
            let ctx = unsafe { 
                fz::fz_new_context_imp(
                    ptr::null(),      // Use default allocator
                    ptr::null(),      // Use default locks
                    128 * 1024 * 1024, // 128MB store limit
                    version.as_ptr()   // Version string
                )
            };
            
            if ctx.is_null() {
                return Err("Failed to create MuPDF context".into());
            }
            
            println!("✅ MuPDF context created successfully");
            
            // Register document handlers
            unsafe {
                fz::fz_register_document_handlers(ctx);
            }
            
            // Open document
            let path_cstr = CString::new(path.to_string_lossy().as_bytes())?;
            let doc = unsafe { 
                fz::fz_open_document(ctx, path_cstr.as_ptr())
            };
            
            if doc.is_null() {
                unsafe { fz::fz_drop_context(ctx); }
                return Err("Failed to open PDF document".into());
            }
            
            println!("✅ MuPDF document opened successfully");
            
            // Get page count
            let page_count = unsafe { fz::fz_count_pages(ctx, doc) } as usize;
            if page_count == 0 {
                unsafe { 
                    fz::fz_drop_document(ctx, doc);
                    fz::fz_drop_context(ctx);
                }
                return Err("PDF has no pages".into());
            }
            
            // Store MuPDF resources
            self.context = Some(ctx);
            self.document = Some(doc);
            self.page_count = page_count;
            
            println!("✅ MuPDF loaded: {} pages in {:?}", page_count, start_time.elapsed());
        }
        
        #[cfg(not(feature = "mupdf"))]
        {
            return Err("MuPDF feature not enabled. Please rebuild with --features mupdf".into());
        }
        
        self.current_file = Some(path.to_path_buf());
        self.current_page = 0;
        self.is_loaded = true;
        self.clear_cache();
        
        Ok(())
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
                ui.heading("🚀 MuPDF Viewer");
                ui.add_space(20.0);
                ui.label("No PDF loaded");
                ui.label("Use Ctrl+O to load a document");
                ui.add_space(10.0);
                ui.label("⚡ High-performance PDF rendering with MuPDF!");
                ui.label(format!("💾 Cache limit: {} MB", self.cache_memory_limit / (1024 * 1024)));
                ui.add_space(10.0);
                
                // Show memory stats
                ui.group(|ui| {
                    ui.label("📊 Memory Stats:");
                    ui.label(format!("Current usage: {} KB", self.memory_stats.current_usage / 1024));
                    ui.label(format!("Peak usage: {} KB", self.memory_stats.peak_usage / 1024));
                    ui.label(format!("Cached textures: {}", self.memory_stats.texture_count));
                    ui.label(format!("Cache hits: {}", self.memory_stats.cache_hits));
                    ui.label(format!("Cache misses: {}", self.memory_stats.cache_misses));
                });
            },
        );
    }
    
    fn render_pdf_content(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.heading("🚀 MuPDF Content");
            ui.separator();
            
            if let Some(ref file) = self.current_file {
                ui.horizontal(|ui| {
                    ui.label(format!("File: {}", file.file_name().unwrap_or_default().to_string_lossy()));
                    ui.separator();
                    ui.label(format!("Pages: {}", self.page_count));
                    ui.separator();
                    ui.label(format!("Current: {}/{}", self.current_page + 1, self.page_count));
                });
                
                ui.separator();
                
                // Enhanced controls row
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
                    
                    // Cache management
                    if ui.button("🗑️ Clear Cache").clicked() {
                        self.clear_cache();
                    }
                    
                    ui.separator();
                    ui.checkbox(&mut self.debug_overlay, "Debug overlay");
                });
                
                // Performance and memory info
                ui.horizontal(|ui| {
                    ui.label(format!("💾 Memory: {} KB / {} MB", 
                        self.memory_stats.current_usage / 1024,
                        self.cache_memory_limit / (1024 * 1024)
                    ));
                    ui.separator();
                    ui.label(format!("📊 Cache: {} hits, {} misses", 
                        self.memory_stats.cache_hits, 
                        self.memory_stats.cache_misses
                    ));
                    
                    if !self.render_times.is_empty() {
                        let avg_time = self.render_times.iter().sum::<std::time::Duration>() / self.render_times.len() as u32;
                        ui.separator();
                        ui.label(format!("⚡ Avg render: {:.2}ms", avg_time.as_secs_f64() * 1000.0));
                    }
                });
                
                ui.separator();
                
                // Render PDF page with smart caching
                self.render_current_page(ui);
            }
        });
    }
    
    fn render_current_page(&mut self, ui: &mut Ui) {
        // Check cache first
        if let Some(texture) = self.page_cache.get(&self.current_page).cloned() {
            self.memory_stats.cache_hits += 1;
            self.display_page_texture(ui, &texture);
        } else {
            self.memory_stats.cache_misses += 1;
            
            // Render new page
            match self.render_page_to_cache(ui.ctx(), self.current_page) {
                Ok(texture) => {
                    self.display_page_texture(ui, &texture);
                }
                Err(e) => {
                    ui.label(format!("⚠️ Failed to render page: {}", e));
                }
            }
        }
    }
    
    fn render_page_to_cache(&mut self, ctx: &Context, page_num: usize) -> Result<TextureHandle, Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();
        
        #[cfg(feature = "mupdf")]
        {
            let mupdf_ctx = self.context.ok_or("MuPDF context not initialized")?;
            let doc = self.document.ok_or("MuPDF document not loaded")?;
            
            // Load page
            let page = unsafe { fz::fz_load_page(mupdf_ctx, doc, page_num as i32) };
            if page.is_null() {
                return Err(format!("Failed to load page {}", page_num).into());
            }
            
            // Get page bounds
            let bounds = unsafe { fz::fz_bound_page(mupdf_ctx, page) };
            
            // Calculate render matrix for good quality
            let scale = 2.0 * self.zoom_level; // 2.0 for good quality
            let matrix = fz::fz_matrix {
                a: scale,
                b: 0.0,
                c: 0.0,
                d: scale,
                e: 0.0,
                f: 0.0,
            };
            
            // Transform bounds
            let mut bbox = fz::fz_irect { x0: 0, y0: 0, x1: 0, y1: 0 };
            unsafe { 
                let transformed_bounds = fz::fz_transform_rect(bounds, matrix);
                bbox = fz::fz_round_rect(transformed_bounds);
            }
            
            let width = (bbox.x1 - bbox.x0) as usize;
            let height = (bbox.y1 - bbox.y0) as usize;
            
            if width == 0 || height == 0 {
                unsafe { fz::fz_drop_page(mupdf_ctx, page); }
                return Err("Invalid page dimensions".into());
            }
            
            // Check memory before creating pixmap
            let estimated_memory = width * height * 4; // RGBA
            self.ensure_cache_space(estimated_memory);
            
            // Create pixmap
            let colorspace = unsafe { fz::fz_device_rgb(mupdf_ctx) };
            let pixmap = unsafe { fz::fz_new_pixmap_with_bbox(mupdf_ctx, colorspace, bbox, ptr::null_mut(), 0) };
            if pixmap.is_null() {
                unsafe { fz::fz_drop_page(mupdf_ctx, page); }
                return Err("Failed to create pixmap".into());
            }
            
            // Clear pixmap to white
            unsafe { fz::fz_clear_pixmap_with_value(mupdf_ctx, pixmap, 0xff) };
            
            // Create device and render
            let device = unsafe { fz::fz_new_draw_device(mupdf_ctx, matrix, pixmap) };
            if device.is_null() {
                unsafe { 
                    fz::fz_drop_pixmap(mupdf_ctx, pixmap);
                    fz::fz_drop_page(mupdf_ctx, page);
                }
                return Err("Failed to create device".into());
            }
            
            // Render page
            unsafe { 
                fz::fz_run_page(mupdf_ctx, page, device, matrix, ptr::null_mut());
                fz::fz_close_device(mupdf_ctx, device);
                fz::fz_drop_device(mupdf_ctx, device);
            }
            
            // Extract pixel data
            let samples = unsafe { fz::fz_pixmap_samples(mupdf_ctx, pixmap) };
            let stride = unsafe { fz::fz_pixmap_stride(mupdf_ctx, pixmap) } as usize;
            let pixel_data = unsafe { std::slice::from_raw_parts(samples, stride * height) };
            
            // Convert to RGBA (MuPDF uses RGB, we need RGBA for egui)
            let mut rgba_data = Vec::with_capacity(width * height * 4);
            for y in 0..height {
                for x in 0..width {
                    let pixel_idx = y * stride + x * 3;
                    if pixel_idx + 2 < pixel_data.len() {
                        rgba_data.push(pixel_data[pixel_idx]);     // R
                        rgba_data.push(pixel_data[pixel_idx + 1]); // G
                        rgba_data.push(pixel_data[pixel_idx + 2]); // B
                        rgba_data.push(255);                       // A
                    }
                }
            }
            
            // Clean up MuPDF resources
            unsafe { 
                fz::fz_drop_pixmap(mupdf_ctx, pixmap);
                fz::fz_drop_page(mupdf_ctx, page);
            }
            
            // Create egui texture
            let color_image = ColorImage::from_rgba_unmultiplied([width, height], &rgba_data);
            let texture_name = format!("mupdf_page_{}", page_num);
            let texture = ctx.load_texture(texture_name, color_image, TextureOptions::default());
            
            // Update cache and memory stats
            let texture_memory = width * height * 4;
            self.page_cache.insert(page_num, texture.clone());
            self.cache_memory_used += texture_memory;
            self.memory_stats.current_usage = self.cache_memory_used;
            self.memory_stats.peak_usage = self.memory_stats.peak_usage.max(self.cache_memory_used);
            self.memory_stats.texture_count = self.page_cache.len();
            
            // Record render time
            let render_time = start_time.elapsed();
            self.render_times.push(render_time);
            if self.render_times.len() > 10 {
                self.render_times.remove(0); // Keep only last 10 times
            }
            
            println!("✅ MuPDF rendered page {} in {:.2}ms ({}x{})", 
                page_num + 1, 
                render_time.as_secs_f64() * 1000.0,
                width, 
                height
            );
            
            Ok(texture)
        }
        
        #[cfg(not(feature = "mupdf"))]
        {
            Err("MuPDF feature not enabled".into())
        }
    }
    
    fn display_page_texture(&mut self, ui: &mut Ui, texture: &TextureHandle) {
        let available_width = ui.available_width();
        let available_height = ui.available_height() - 50.0;
        
        let texture_size = texture.size_vec2();
        let base_scale = (available_width / texture_size.x).min(available_height / texture_size.y).min(1.0);
        let final_scale = base_scale * self.zoom_level;
        let display_size = texture_size * final_scale;
        
        ScrollArea::both()
            .max_height(available_height)
            .show(ui, |ui| {
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
                        }
                    }
                }
                
                // Render debug overlay if enabled
                if self.debug_overlay {
                    let scale_factor = egui::vec2(final_scale, final_scale);
                    self.coordinate_mapper.render_debug_overlay(ui, response.rect, scale_factor);
                }
            });
    }
    
    fn ensure_cache_space(&mut self, needed_bytes: usize) {
        while self.cache_memory_used + needed_bytes > self.cache_memory_limit && !self.page_cache.is_empty() {
            // Remove the page that's furthest from current page
            let current = self.current_page;
            let furthest_page = self.page_cache.keys()
                .max_by_key(|&&page| {
                    (page as i32 - current as i32).abs()
                })
                .copied();
            
            if let Some(page_to_remove) = furthest_page {
                if let Some(texture) = self.page_cache.remove(&page_to_remove) {
                    let texture_size = texture.size_vec2();
                    let texture_memory = (texture_size.x * texture_size.y * 4.0) as usize;
                    self.cache_memory_used = self.cache_memory_used.saturating_sub(texture_memory);
                    println!("🗑️ Evicted page {} from cache (freed {} KB)", 
                        page_to_remove + 1, 
                        texture_memory / 1024
                    );
                }
            } else {
                break;
            }
        }
        
        self.memory_stats.current_usage = self.cache_memory_used;
        self.memory_stats.texture_count = self.page_cache.len();
    }
    
    fn clear_cache(&mut self) {
        let old_count = self.page_cache.len();
        let old_memory = self.cache_memory_used;
        
        self.page_cache.clear();
        self.cache_memory_used = 0;
        self.memory_stats.current_usage = 0;
        self.memory_stats.texture_count = 0;
        
        println!("🗑️ Cleared cache: {} textures, {} KB freed", old_count, old_memory / 1024);
    }
    
    #[cfg(feature = "mupdf")]
    fn cleanup_mupdf_resources(&mut self) {
        if let Some(doc) = self.document.take() {
            if let Some(ctx) = self.context {
                unsafe { fz::fz_drop_document(ctx, doc); }
            }
        }
        
        if let Some(ctx) = self.context.take() {
            unsafe { fz::fz_drop_context(ctx); }
        }
        
        self.clear_cache();
    }
    
    pub fn get_page_count(&self) -> usize {
        self.page_count
    }
    
    pub fn get_current_page(&self) -> usize {
        self.current_page
    }
    
    pub fn load_coordinate_mapping(&mut self, docling_json_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        self.coordinate_mapper.load_docling_output(docling_json_path)?;
        
        if let Some(texture) = self.page_cache.get(&self.current_page) {
            let image_size = texture.size_vec2();
            self.coordinate_mapper.generate_text_regions(image_size);
            tracing::info!("Loaded coordinate mapping with {} regions", self.coordinate_mapper.text_regions.len());
        }
        
        Ok(())
    }
    
    pub fn get_selected_text(&self) -> Option<&str> {
        self.coordinate_mapper.get_selected_text()
    }
    
    pub fn highlight_text_region(&mut self, text_index: usize) -> Option<usize> {
        self.coordinate_mapper.handle_text_selection(text_index)
    }
    
    /// Get performance statistics
    pub fn get_performance_stats(&self) -> (MemoryStats, Vec<std::time::Duration>) {
        (self.memory_stats.clone(), self.render_times.clone())
    }
    
    /// Set cache memory limit
    pub fn set_cache_limit(&mut self, limit_mb: usize) {
        self.cache_memory_limit = limit_mb * 1024 * 1024;
        println!("📝 Cache limit set to {} MB", limit_mb);
        
        // Ensure current usage is within new limit
        if self.cache_memory_used > self.cache_memory_limit {
            self.ensure_cache_space(0);
        }
    }
}

impl Drop for MuPdfViewer {
    fn drop(&mut self) {
        #[cfg(feature = "mupdf")]
        {
            self.cleanup_mupdf_resources();
            println!("🧹 MuPDF viewer cleaned up");
        }
    }
}

impl Default for MuPdfViewer {
    fn default() -> Self {
        Self::new()
    }
}
