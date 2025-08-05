use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use pdfium_render::prelude::*;
use image::{ImageBuffer, Rgb, RgbImage};

// Character matrix PDF representation system
// 1. PDF → Smallest viable character matrix
// 2. Vision model → Character region bounding boxes  
// 3. Pdfium → All text extraction
// 4. Map text into character matrix using vision bounding boxes

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterMatrix {
    pub width: usize,
    pub height: usize,
    pub matrix: Vec<Vec<char>>,
    pub text_regions: Vec<TextRegion>,
    pub original_text: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextRegion {
    pub bbox: CharBBox,
    pub confidence: f32,
    pub text_content: String,
    pub region_id: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharBBox {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

impl CharBBox {
    pub fn contains(&self, x: usize, y: usize) -> bool {
        x >= self.x && x < self.x + self.width &&
        y >= self.y && y < self.y + self.height
    }
    
    pub fn area(&self) -> usize {
        self.width * self.height
    }
}

pub struct CharacterMatrixEngine {
    pub char_width: f32,  // Character width in points
    pub char_height: f32, // Character height in points
}

impl CharacterMatrixEngine {
    pub fn new() -> Self {
        Self {
            char_width: 6.0,   // Approximate character width
            char_height: 12.0, // Approximate character height
        }
    }
    
    pub fn process_pdf(&self, pdf_path: &Path) -> Result<CharacterMatrix> {
        // Step 1: Convert PDF to character matrix representation
        let (matrix, page_width, page_height) = self.pdf_to_character_matrix(pdf_path)?;
        
        // Step 2: Create image from character matrix for vision model
        let matrix_image = self.character_matrix_to_image(&matrix);
        
        // Step 3: Use vision model to find text regions (placeholder for now)
        let text_regions = self.detect_text_regions_with_vision(&matrix_image, &matrix)?;
        
        // Step 4: Extract all text from PDF using pdfium
        let original_text = self.extract_all_text_with_pdfium(pdf_path)?;
        
        // Step 5: Map extracted text into character matrix using vision bounding boxes
        let final_matrix = self.map_text_to_matrix(matrix, &text_regions, &original_text)?;
        
        Ok(final_matrix)
    }
    
    fn pdf_to_character_matrix(&self, pdf_path: &Path) -> Result<(Vec<Vec<char>>, f32, f32)> {
        let pdfium = Pdfium::new(
            Pdfium::bind_to_system_library()
                .or_else(|_| Pdfium::bind_to_library("./lib/libpdfium.dylib"))
                .or_else(|_| Pdfium::bind_to_library("/usr/local/lib/libpdfium.dylib"))
                .map_err(|e| anyhow::anyhow!("Failed to bind pdfium: {}", e))?
        );
        
        let document = pdfium.load_pdf_from_file(pdf_path, None)
            .map_err(|e| anyhow::anyhow!("Failed to load PDF: {}", e))?;
        
        // For now, process just the first page
        let page = document.pages().get(0).ok_or_else(|| anyhow::anyhow!("No pages found"))?;
        let page_width = page.width().value;
        let page_height = page.height().value;
        
        // Calculate character matrix dimensions
        let char_cols = (page_width / self.char_width).ceil() as usize;
        let char_rows = (page_height / self.char_height).ceil() as usize;
        
        // Initialize matrix with spaces
        let mut matrix = vec![vec![' '; char_cols]; char_rows];
        
        // Render page to bitmap to understand layout
        let bitmap = page.render_with_config(
            &PdfRenderConfig::new()
                .set_target_width((page_width * 2.0) as i32) // Higher DPI for better text detection
                .set_maximum_height((page_height * 2.0) as i32)
        ).map_err(|e| anyhow::anyhow!("Failed to render page: {}", e))?;
        
        // Convert bitmap to image for analysis
        let width = bitmap.width() as u32;
        let height = bitmap.height() as u32;
        let stride = bitmap.stride();
        
        // Analyze bitmap to determine where text likely exists
        // This is a simplified approach - in reality you'd use more sophisticated analysis
        let bitmap_buffer = bitmap.as_bytes();
        
        for row in 0..char_rows {
            for col in 0..char_cols {
                // Map character position to bitmap coordinates
                let bitmap_x = ((col as f32 * self.char_width) * 2.0) as usize; // *2 for higher DPI
                let bitmap_y = ((row as f32 * self.char_height) * 2.0) as usize;
                
                // Sample a few pixels in this character cell to see if there's content
                let has_content = self.sample_character_cell(
                    bitmap_buffer, 
                    bitmap_x, 
                    bitmap_y, 
                    stride, 
                    width as usize, 
                    height as usize
                );
                
                if has_content {
                    matrix[row][col] = '█'; // Mark as potential text area
                }
            }
        }
        
        Ok((matrix, page_width, page_height))
    }
    
    fn sample_character_cell(
        &self,
        bitmap: &[u8],
        x: usize,
        y: usize,
        stride: usize,
        width: usize,
        height: usize,
    ) -> bool {
        let char_width = (self.char_width * 2.0) as usize; // Account for 2x DPI
        let char_height = (self.char_height * 2.0) as usize;
        
        let mut dark_pixels = 0;
        let mut total_pixels = 0;
        
        // Sample pixels in this character cell
        for dy in 0..char_height.min(10) { // Sample at most 10 pixels high
            for dx in 0..char_width.min(10) { // Sample at most 10 pixels wide
                let px = x + dx;
                let py = y + dy;
                
                if px < width && py < height {
                    let pixel_offset = py * stride + px * 4; // Assuming RGBA
                    if pixel_offset + 2 < bitmap.len() {
                        let r = bitmap[pixel_offset] as u32;
                        let g = bitmap[pixel_offset + 1] as u32;
                        let b = bitmap[pixel_offset + 2] as u32;
                        
                        let brightness = (r + g + b) / 3;
                        if brightness < 200 { // Pixel is dark enough to be text
                            dark_pixels += 1;
                        }
                        total_pixels += 1;
                    }
                }
            }
        }
        
        // If more than 10% of pixels are dark, consider this a text area
        total_pixels > 0 && (dark_pixels as f32 / total_pixels as f32) > 0.1
    }
    
    fn character_matrix_to_image(&self, matrix: &[Vec<char>]) -> RgbImage {
        let height = matrix.len();
        let width = if height > 0 { matrix[0].len() } else { 0 };
        
        let mut img = ImageBuffer::new(width as u32, height as u32);
        
        for (y, row) in matrix.iter().enumerate() {
            for (x, &ch) in row.iter().enumerate() {
                let pixel_value = if ch == ' ' { 255 } else { 0 }; // White for space, black for content
                img.put_pixel(x as u32, y as u32, Rgb([pixel_value, pixel_value, pixel_value]));
            }
        }
        
        img
    }
    
    fn detect_text_regions_with_vision(
        &self,
        _image: &RgbImage,
        matrix: &[Vec<char>],
    ) -> Result<Vec<TextRegion>> {
        // Placeholder vision model - in reality you'd use actual ML model here
        // For now, we'll do simple connected component analysis to find text regions
        
        let mut regions = Vec::new();
        let mut visited = vec![vec![false; matrix[0].len()]; matrix.len()];
        let mut region_id = 0;
        
        for y in 0..matrix.len() {
            for x in 0..matrix[y].len() {
                if matrix[y][x] != ' ' && !visited[y][x] {
                    // Found start of a text region, flood fill to get its bounds
                    let region = self.flood_fill_region(matrix, &mut visited, x, y)?;
                    
                    if region.area() > 2 { // Only keep regions with reasonable size
                        regions.push(TextRegion {
                            bbox: region,
                            confidence: 0.8, // Placeholder confidence
                            text_content: String::new(), // Will be filled later
                            region_id,
                        });
                        region_id += 1;
                    }
                }
            }
        }
        
        Ok(regions)
    }
    
    fn flood_fill_region(
        &self,
        matrix: &[Vec<char>],
        visited: &mut [Vec<bool>],
        start_x: usize,
        start_y: usize,
    ) -> Result<CharBBox> {
        let mut min_x = start_x;
        let mut max_x = start_x;
        let mut min_y = start_y;
        let mut max_y = start_y;
        
        let mut stack = vec![(start_x, start_y)];
        
        while let Some((x, y)) = stack.pop() {
            if visited[y][x] || matrix[y][x] == ' ' {
                continue;
            }
            
            visited[y][x] = true;
            
            // Update bounds
            min_x = min_x.min(x);
            max_x = max_x.max(x);
            min_y = min_y.min(y);
            max_y = max_y.max(y);
            
            // Add neighbors to stack
            for (dx, dy) in &[(0, 1), (1, 0), (0, -1), (-1, 0)] {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                
                if nx >= 0 && ny >= 0 {
                    let nx = nx as usize;
                    let ny = ny as usize;
                    
                    if ny < matrix.len() && nx < matrix[ny].len() {
                        if !visited[ny][nx] && matrix[ny][nx] != ' ' {
                            stack.push((nx, ny));
                        }
                    }
                }
            }
        }
        
        Ok(CharBBox {
            x: min_x,
            y: min_y,
            width: max_x - min_x + 1,
            height: max_y - min_y + 1,
        })
    }
    
    fn extract_all_text_with_pdfium(&self, pdf_path: &Path) -> Result<Vec<String>> {
        let pdfium = Pdfium::new(
            Pdfium::bind_to_system_library()
                .or_else(|_| Pdfium::bind_to_library("./lib/libpdfium.dylib"))
                .or_else(|_| Pdfium::bind_to_library("/usr/local/lib/libpdfium.dylib"))
                .map_err(|e| anyhow::anyhow!("Failed to bind pdfium: {}", e))?
        );
        
        let document = pdfium.load_pdf_from_file(pdf_path, None)
            .map_err(|e| anyhow::anyhow!("Failed to load PDF: {}", e))?;
        
        let mut all_text = Vec::new();
        
        for page in document.pages().iter() {
            let text_page = page.text().map_err(|e| anyhow::anyhow!("Failed to get text: {}", e))?;
            let page_text = text_page.all();
            
            // Split into logical text chunks (words, lines, etc.)
            for line in page_text.lines() {
                let line = line.trim();
                if !line.is_empty() {
                    // Further split into words if needed
                    for word in line.split_whitespace() {
                        all_text.push(word.to_string());
                    }
                }
            }
        }
        
        Ok(all_text)
    }
    
    fn map_text_to_matrix(
        &self,
        mut matrix: Vec<Vec<char>>,
        text_regions: &[TextRegion],
        original_text: &[String],
    ) -> Result<CharacterMatrix> {
        let mut updated_regions = Vec::new();
        let mut text_index = 0;
        
        // Sort regions by reading order (top to bottom, left to right)
        let mut sorted_regions: Vec<_> = text_regions.iter().collect();
        sorted_regions.sort_by_key(|r| (r.bbox.y, r.bbox.x));
        
        for region in sorted_regions {
            let mut region_text = String::new();
            let region_area = region.bbox.area();
            
            // Estimate how much text can fit in this region
            let estimated_chars = region_area;
            
            // Fill this region with text from our extracted text
            let mut chars_filled = 0;
            while text_index < original_text.len() && chars_filled < estimated_chars {
                let word = &original_text[text_index];
                
                if chars_filled + word.len() + 1 <= estimated_chars {
                    if !region_text.is_empty() {
                        region_text.push(' ');
                        chars_filled += 1;
                    }
                    region_text.push_str(word);
                    chars_filled += word.len();
                    text_index += 1;
                } else {
                    break;
                }
            }
            
            // Place text into the character matrix
            self.place_text_in_region(&mut matrix, &region.bbox, &region_text);
            
            let mut updated_region = region.clone();
            updated_region.text_content = region_text;
            updated_regions.push(updated_region);
        }
        
        Ok(CharacterMatrix {
            width: matrix[0].len(),
            height: matrix.len(),
            matrix,
            text_regions: updated_regions,
            original_text: original_text.to_vec(),
        })
    }
    
    fn place_text_in_region(
        &self,
        matrix: &mut [Vec<char>],
        bbox: &CharBBox,
        text: &str,
    ) {
        let mut text_chars: Vec<char> = text.chars().collect();
        let mut char_index = 0;
        
        // Fill the region with text, wrapping as needed
        for row in bbox.y..(bbox.y + bbox.height).min(matrix.len()) {
            for col in bbox.x..(bbox.x + bbox.width).min(matrix[row].len()) {
                if char_index < text_chars.len() {
                    let ch = text_chars[char_index];
                    if ch != ' ' || matrix[row][col] == '█' { // Only place non-space chars or replace content markers
                        matrix[row][col] = ch;
                        char_index += 1;
                    }
                } else {
                    break;
                }
            }
            
            if char_index >= text_chars.len() {
                break;
            }
        }
    }
    
    pub fn render_matrix_as_string(&self, char_matrix: &CharacterMatrix) -> String {
        let mut result = String::new();
        
        result.push_str(&format!("Character Matrix ({}x{}):\n", char_matrix.width, char_matrix.height));
        result.push_str(&format!("Text Regions: {}\n", char_matrix.text_regions.len()));
        result.push_str(&"═".repeat(char_matrix.width.min(80)));
        result.push('\n');
        
        for (row_idx, row) in char_matrix.matrix.iter().enumerate() {
            // Show line numbers for long documents
            if char_matrix.height > 20 {
                result.push_str(&format!("{:3} ", row_idx));
            }
            
            for &ch in row {
                result.push(ch);
            }
            result.push('\n');
        }
        
        result.push_str(&"═".repeat(char_matrix.width.min(80)));
        result.push('\n');
        
        // Show detected regions info
        for (i, region) in char_matrix.text_regions.iter().enumerate() {
            result.push_str(&format!(
                "Region {}: ({},{}) {}x{} - \"{}\"\n",
                i + 1,
                region.bbox.x,
                region.bbox.y,
                region.bbox.width,
                region.bbox.height,
                region.text_content.chars().take(50).collect::<String>()
            ));
        }
        
        result
    }
}

impl Default for CharacterMatrixEngine {
    fn default() -> Self {
        Self::new()
    }
}
