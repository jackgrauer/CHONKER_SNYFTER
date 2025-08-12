use std::path::PathBuf;
use anyhow::Result;
use pdfium_render::prelude::*;

pub struct PdfEngine {
    pdfium: Pdfium,
    current_path: Option<PathBuf>,
}

impl PdfEngine {
    pub fn new() -> Result<Self> {
        // Initialize PDFium with the bundled library
        // Try multiple paths to find the library with better error reporting
        let lib_paths = [
            "./lib",
            "./projects/chonker6/lib", 
            "/Users/jack/chonker6/projects/chonker6/lib",
        ];
        
        let mut last_error = None;
        for path in &lib_paths {
            let lib_name = Pdfium::pdfium_platform_library_name_at_path(path);
            eprintln!("Trying PDFium library at: {}", lib_name.display());
            
            match Pdfium::bind_to_library(&lib_name) {
                Ok(binding) => {
                    eprintln!("✓ Successfully bound to PDFium library at: {}", lib_name.display());
                    let pdfium = Pdfium::new(binding);
                    return Ok(Self {
                        pdfium,
                        current_path: None,
                    });
                }
                Err(e) => {
                    eprintln!("✗ Failed to bind to {}: {}", lib_name.display(), e);
                    last_error = Some(e);
                }
            }
        }
        
        // Try system library as fallback
        eprintln!("Trying system PDFium library...");
        match Pdfium::bind_to_system_library() {
            Ok(binding) => {
                eprintln!("✓ Successfully bound to system PDFium library");
                let pdfium = Pdfium::new(binding);
                Ok(Self {
                    pdfium,
                    current_path: None,
                })
            }
            Err(e) => {
                eprintln!("✗ Failed to bind to system library: {}", e);
                Err(anyhow::anyhow!(
                    "Failed to initialize PDFium library. Last error: {:?}. \
                     Make sure to set DYLD_LIBRARY_PATH or run with ./run_chonker6.sh", 
                    last_error.unwrap_or(e)
                ))
            }
        }
    }
    
    pub fn load_pdf(&mut self, path: &PathBuf) -> Result<(usize, String)> {
        // Load the PDF document just to get metadata
        let document = self.pdfium.load_pdf_from_file(path, None)?;
        let page_count = document.pages().len() as usize;
        let title = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();
        
        self.current_path = Some(path.clone());
        Ok((page_count, title))
    }
    
    pub fn extract_text_matrix(&self, page_index: usize) -> Result<Vec<Vec<char>>> {
        if let Some(path) = &self.current_path {
            // Load document fresh for extraction
            let document = self.pdfium.load_pdf_from_file(path, None)?;
            
            // Use spatial extraction like chonker5
            let page = document.pages().get(page_index as u16)?;
            let page_height = page.height().value;
            let text_page = page.text()?;
            
            // Fixed character dimensions
            let char_width = 6.0;
            let char_height = 12.0;
            
            // Collect text segments with positions (use segment-based approach like chonker5)
            let mut text_segments = vec![];
            
            for segment in text_page.segments().iter() {
                let bounds = segment.bounds();
                let text = segment.text();
                if !text.trim().is_empty() {
                    text_segments.push((
                        text,
                        bounds.left().value,
                        page_height - bounds.top().value,  // Convert to top-left origin
                        bounds.right().value - bounds.left().value, // width
                        bounds.top().value - bounds.bottom().value, // height
                    ));
                }
            }
            
            if text_segments.is_empty() {
                return Ok(vec![vec!['N', 'o', ' ', 't', 'e', 'x', 't', ' ', 'f', 'o', 'u', 'n', 'd']]);
            }
            
            // Find bounds from segments
            let min_x = text_segments.iter().map(|s| s.1).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(0.0);
            let min_y = text_segments.iter().map(|s| s.2).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(0.0);
            let max_x = text_segments.iter().map(|s| s.1 + s.3).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(100.0);
            let max_y = text_segments.iter().map(|s| s.2 + s.4).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(100.0);
            
            // Calculate matrix dimensions
            let cols = ((max_x - min_x) / char_width).ceil() as usize + 1;
            let rows = ((max_y - min_y) / char_height).ceil() as usize + 1;
            
            // Limit size for terminal display
            let cols = cols.min(120);
            let rows = rows.min(50);
            
            // Initialize matrix
            let mut matrix = vec![vec![' '; cols]; rows];
            
            // Place text segments in matrix with z-ordering (like chonker5)
            for (text, x, y, _width, height) in text_segments {
                // Calculate z-order based on position and size (larger text has priority)
                let z_order = if height > 14.0 && y < 100.0 {
                    150 // Headers at top
                } else if height > 14.0 {
                    125 // Large text
                } else if y > max_y - 100.0 {
                    75  // Footer text
                } else {
                    100 // Regular text
                };
                
                // Calculate starting position in matrix
                let start_col = ((x - min_x) / char_width) as usize;
                let start_row = ((y - min_y) / char_height) as usize;
                
                // Place each character from the segment
                for (char_idx, ch) in text.chars().enumerate() {
                    let col = start_col + char_idx;
                    let row = start_row;
                    
                    if row < rows && col < cols {
                        // Only overwrite if this has higher z-order or the cell is empty
                        if matrix[row][col] == ' ' || z_order > 100 {
                            matrix[row][col] = ch;
                        }
                    }
                }
            }
            
            Ok(matrix)
        } else {
            Err(anyhow::anyhow!("No PDF loaded"))
        }
    }
    
    pub fn get_current_page(&self) -> usize {
        0  // For now, always return first page
    }
    
    pub fn render_page_info(&self, page_index: usize) -> Result<String> {
        if let Some(path) = &self.current_path {
            let document = self.pdfium.load_pdf_from_file(path, None)?;
            let page = document.pages().get(page_index as u16)?;
            
            // Return basic page info - actual image rendering would need terminal image protocol
            Ok(format!(
                "\n  PDF VIEWER\n  \n  📄 {} \n  📊 Page {}/{}\n  📏 {:.1} x {:.1} pts\n\n  🖼️  PDF image rendering not yet implemented\n  🔧 Use terminal with image support for full display\n  \n  Extract text with Ctrl+E to edit →",
                path.file_name().unwrap_or_default().to_str().unwrap_or("Unknown"),
                page_index + 1,
                document.pages().len(),
                page.width().value,
                page.height().value
            ))
        } else {
            Ok("No PDF loaded".to_string())
        }
    }
    
    pub fn render_page_as_image(&self, page_index: usize, zoom_level: f32) -> Result<Vec<u8>> {
        if let Some(path) = &self.current_path {
            let document = self.pdfium.load_pdf_from_file(path, None)?;
            let page = document.pages().get(page_index as u16)?;
            
            // Calculate render dimensions based on zoom
            let base_width = 800;
            let base_height = 1000;
            let render_width = (base_width as f32 * zoom_level) as i32;
            let render_height = (base_height as f32 * zoom_level) as i32;
            
            // Render page as bitmap
            let bitmap = page.render_with_config(
                &PdfRenderConfig::new()
                    .set_target_width(render_width)
                    .set_target_height(render_height)
                    .rotate_if_landscape(PdfPageRenderRotation::None, false)
            )?;
            
            // Convert to raw bytes - iTerm2 can handle various formats
            let raw_data = bitmap.as_raw_bytes();
            
            Ok(raw_data.to_vec())
        } else {
            Err(anyhow::anyhow!("No PDF loaded"))
        }
    }
    
    pub fn render_page_for_kitty(&self, page_index: usize, width_px: u32, height_px: u32) -> Result<(Vec<u8>, u32, u32, String)> {
        if let Some(path) = &self.current_path {
            let document = self.pdfium.load_pdf_from_file(path, None)?;
            let page = document.pages().get(page_index as u16)?;
            
            // Get page dimensions
            let page_width = page.width().value;
            let page_height = page.height().value;
            let page_aspect = page_width / page_height;
            
            // Calculate best fit dimensions maintaining aspect ratio
            let display_aspect = width_px as f32 / height_px as f32;
            
            let (render_width, render_height) = if page_aspect > display_aspect {
                // Page is wider - fit to width
                let w = width_px;
                let h = (width_px as f32 / page_aspect) as u32;
                (w, h.min(height_px))
            } else {
                // Page is taller - fit to height
                let h = height_px;
                let w = (height_px as f32 * page_aspect) as u32;
                (w.min(width_px), h)
            };
            
            // Render page as RGBA bitmap
            let bitmap = page.render_with_config(
                &PdfRenderConfig::new()
                    .set_target_width(render_width as i32)
                    .set_target_height(render_height as i32)
                    .rotate_if_landscape(PdfPageRenderRotation::None, false)
            )?;
            
            // Get raw RGBA bytes from PDFium
            let rgba_bytes = bitmap.as_rgba_bytes();
            
            // Convert RGBA bytes to PNG format for Kitty
            use image::{ImageBuffer, RgbaImage};
            let img: RgbaImage = ImageBuffer::from_raw(render_width, render_height, rgba_bytes)
                .ok_or_else(|| anyhow::anyhow!("Failed to create image buffer from RGBA data"))?;
            
            // Encode as PNG
            let mut png_data = Vec::new();
            {
                use std::io::Cursor;
                let mut cursor = Cursor::new(&mut png_data);
                img.write_to(&mut cursor, image::ImageFormat::Png)
                    .map_err(|e| anyhow::anyhow!("Failed to encode PNG: {}", e))?;
            }
            
            // Convert to base64 for Kitty protocol
            use base64::Engine;
            let base64_png = base64::engine::general_purpose::STANDARD.encode(&png_data);
            
            // Return PNG data, dimensions, and base64 string
            Ok((png_data, render_width, render_height, base64_png))
        } else {
            Err(anyhow::anyhow!("No PDF loaded"))
        }
    }
}