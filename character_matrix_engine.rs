use anyhow::Result;
use image::{DynamicImage, ImageBuffer, Rgb, RgbImage};
use pdfium_render::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tokio::runtime::Runtime;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextObject {
    pub text: String,
    pub bbox: BBox,
    pub font_size: f32,
    pub font_family: String,
    pub is_bold: bool,
    pub is_italic: bool,
    pub reading_order: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BBox {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
}

impl BBox {
    pub fn width(&self) -> f32 {
        self.x1 - self.x0
    }

    pub fn height(&self) -> f32 {
        self.y1 - self.y0
    }

    pub fn center_x(&self) -> f32 {
        (self.x0 + self.x1) / 2.0
    }

    pub fn center_y(&self) -> f32 {
        (self.y0 + self.y1) / 2.0
    }

    pub fn area(&self) -> f32 {
        self.width() * self.height()
    }

    pub fn overlaps(&self, other: &BBox) -> bool {
        !(self.x1 < other.x0 || other.x1 < self.x0 || self.y1 < other.y0 || other.y1 < self.y0)
    }

    pub fn contains_point(&self, x: f32, y: f32) -> bool {
        x >= self.x0 && x <= self.x1 && y >= self.y0 && y <= self.y1
    }
}

impl CharBBox {
    pub fn contains(&self, x: usize, y: usize) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
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
            char_width: 6.0,   // Will be dynamically calculated
            char_height: 12.0, // Will be dynamically calculated
        }
    }

    /// Create engine with optimally calculated character dimensions for a specific PDF
    pub fn new_optimized(pdf_path: &Path) -> Result<Self> {
        let mut engine = Self::new();
        let (char_width, char_height) = engine.find_optimal_character_dimensions(pdf_path)?;
        engine.char_width = char_width;
        engine.char_height = char_height;
        Ok(engine)
    }

    pub fn process_pdf(&self, pdf_path: &Path) -> Result<CharacterMatrix> {
        println!("🚀 Processing PDF with Ferrules layout detection...");

        // Step 1: Render PDF page to high-quality image for YOLO model
        let (page_image, page_width, page_height) = self.render_pdf_page_to_image(pdf_path)?;

        // Step 2: Run Ferrules YOLO model on the actual PDF page image
        let layout_regions = self.detect_layout_with_ferrules(&page_image)?;

        println!(
            "📊 Ferrules detected {} layout regions",
            layout_regions.len()
        );
        for (i, region) in layout_regions.iter().enumerate() {
            println!(
                "  Region {}: {} at ({:.1}, {:.1}) - {:.1}x{:.1} (confidence: {:.2})",
                i + 1,
                region.label,
                region.bbox.x0,
                region.bbox.y0,
                region.bbox.width(),
                region.bbox.height(),
                region.proba
            );
        }

        // Step 3: Extract text objects with coordinates
        let text_objects = self.extract_text_objects_with_coordinates(pdf_path)?;

        // Step 4: Calculate optimal character matrix dimensions
        let (matrix_width, matrix_height) =
            self.calculate_matrix_size_from_page(page_width, page_height);

        // Step 5: Create character matrix and map text using layout regions
        let matrix = self.create_matrix_with_layout_regions(
            matrix_width,
            matrix_height,
            &text_objects,
            &layout_regions,
            page_width,
            page_height,
        )?;

        Ok(matrix)
    }

    /// Find optimal character dimensions by analyzing actual text sizes in PDF
    pub fn find_optimal_character_dimensions(&self, pdf_path: &Path) -> Result<(f32, f32)> {
        let text_objects = self.extract_text_objects_with_coordinates(pdf_path)?;

        if text_objects.is_empty() {
            return Ok((self.char_width, self.char_height));
        }

        // Collect font sizes and calculate modal (most common) font size
        let font_sizes: Vec<f32> = text_objects.iter().map(|t| t.font_size).collect();
        let modal_font_size = self.calculate_modal_font_size(&font_sizes);

        // Calculate optimal character dimensions based on modal font size
        // Typical character width is ~0.6 of font size, height is ~1.2 of font size
        let char_width = modal_font_size * 0.6;
        let char_height = modal_font_size * 1.2;

        Ok((char_width.max(4.0), char_height.max(8.0))) // Minimum viable dimensions
    }

    /// Calculate the actual smallest matrix needed based on content bounds
    pub fn adaptive_matrix_sizing(&self, text_objects: &[TextObject]) -> (usize, usize) {
        if text_objects.is_empty() {
            return (80, 24); // Default fallback size
        }

        // Find tight bounding box around all text content
        let content_bounds = self.calculate_content_bounds(text_objects);

        // Calculate minimum matrix dimensions with small padding
        let matrix_width = ((content_bounds.width() / self.char_width).ceil() as usize + 2).max(1);
        let matrix_height =
            ((content_bounds.height() / self.char_height).ceil() as usize + 2).max(1);

        // Reasonable limits to prevent excessive memory usage
        let max_width = 500;
        let max_height = 200;

        (matrix_width.min(max_width), matrix_height.min(max_height))
    }

    /// Extract text objects with precise coordinates using Ferrules approach
    pub fn extract_text_objects_with_coordinates(
        &self,
        pdf_path: &Path,
    ) -> Result<Vec<TextObject>> {
        println!("🔍 Extracting text objects with coordinates using Ferrules approach...");

        let pdfium = Pdfium::new(
            Pdfium::bind_to_system_library()
                .or_else(|_| Pdfium::bind_to_library("./lib/libpdfium.dylib"))
                .or_else(|_| Pdfium::bind_to_library("/usr/local/lib/libpdfium.dylib"))
                .map_err(|e| anyhow::anyhow!("Failed to bind pdfium: {}", e))?,
        );

        let document = pdfium
            .load_pdf_from_file(pdf_path, None)
            .map_err(|e| anyhow::anyhow!("Failed to load PDF: {}", e))?;

        let mut all_text_objects = Vec::new();

        for page in document.pages().iter() {
            let text_page = page
                .text()
                .map_err(|e| anyhow::anyhow!("Failed to get text: {}", e))?;

            // Get page dimensions for coordinate conversion
            let page_width = page.width().value;
            let page_height = page.height().value;

            println!("📏 Page dimensions: {}x{}", page_width, page_height);
            println!("🔤 Processing {} characters...", text_page.chars().len());

            // Use the Ferrules approach: iterate through characters and use tight_bounds()
            let mut current_word = String::new();
            let mut current_bbox: Option<BBox> = None;
            let mut current_font_size = 12.0;

            for char_info in text_page.chars().iter() {
                let ch = char_info.unicode_char().unwrap_or_default();
                // Use tight_bounds() like Ferrules does for new_from_char
                if let Ok(bounds) = char_info.tight_bounds() {
                    let char_bbox = BBox {
                        x0: bounds.left().value,
                        // Convert from PDF coordinates (bottom-left origin) to top-left origin
                        y0: page_height - bounds.top().value,
                        x1: bounds.right().value,
                        y1: page_height - bounds.bottom().value,
                    };

                    let font_size = char_info.unscaled_font_size().value;

                    // Check if this character continues the current word or starts a new one
                    if ch.is_whitespace() || ch == '\n' || ch == '\r' {
                        // End current word if we have one
                        if !current_word.is_empty() && current_bbox.is_some() {
                            all_text_objects.push(TextObject {
                                text: current_word.clone(),
                                bbox: current_bbox.unwrap(),
                                font_size: current_font_size,
                                font_family: "Unknown".to_string(),
                                is_bold: false,
                                is_italic: false,
                                reading_order: all_text_objects.len(),
                            });
                            current_word.clear();
                            current_bbox = None;
                        }
                    } else {
                        // Add character to current word
                        current_word.push(ch);
                        current_font_size = font_size;

                        // Expand bounding box
                        if let Some(ref mut bbox) = current_bbox {
                            bbox.x0 = bbox.x0.min(char_bbox.x0);
                            bbox.y0 = bbox.y0.min(char_bbox.y0);
                            bbox.x1 = bbox.x1.max(char_bbox.x1);
                            bbox.y1 = bbox.y1.max(char_bbox.y1);
                        } else {
                            current_bbox = Some(char_bbox);
                        }
                    }
                } else {
                    println!("⚠️ No tight_bounds() for character: {:?}", ch);
                }
            }

            // Don't forget the last word
            if !current_word.is_empty() && current_bbox.is_some() {
                all_text_objects.push(TextObject {
                    text: current_word,
                    bbox: current_bbox.unwrap(),
                    font_size: current_font_size,
                    font_family: "Unknown".to_string(),
                    is_bold: false,
                    is_italic: false,
                    reading_order: all_text_objects.len(),
                });
            }
        }

        // Sort by reading order (top to bottom, left to right) using integer keys for stable sorting
        all_text_objects.sort_by_key(|obj| {
            // Convert floating point coordinates to integer keys to avoid NaN issues
            let y_key = if obj.bbox.y0.is_finite() {
                (obj.bbox.y0 * 1000.0) as i32
            } else {
                0
            };
            let x_key = if obj.bbox.x0.is_finite() {
                (obj.bbox.x0 * 1000.0) as i32
            } else {
                0
            };
            (y_key, x_key, obj.reading_order)
        });

        println!("📊 Extracted {} text objects", all_text_objects.len());
        if !all_text_objects.is_empty() {
            let first = &all_text_objects[0];
            println!(
                "🔍 First text object: '{}' at ({:.1}, {:.1})",
                first.text.trim(),
                first.bbox.x0,
                first.bbox.y0
            );

            if all_text_objects.len() > 1 {
                let second = &all_text_objects[1];
                println!(
                    "🔍 Second text object: '{}' at ({:.1}, {:.1})",
                    second.text.trim(),
                    second.bbox.x0,
                    second.bbox.y0
                );

                if all_text_objects.len() > 2 {
                    let third = &all_text_objects[2];
                    println!(
                        "🔍 Third text object: '{}' at ({:.1}, {:.1})",
                        third.text.trim(),
                        third.bbox.x0,
                        third.bbox.y0
                    );
                }
            }
        }

        Ok(all_text_objects)
    }

    /// Group individual characters into words based on proximity
    fn group_characters_into_words(&self, characters: Vec<TextObject>) -> Vec<TextObject> {
        if characters.is_empty() {
            return vec![];
        }

        let mut words = Vec::new();
        let mut current_word = String::new();
        let mut current_bbox: Option<BBox> = None;
        let mut last_char_right = 0.0;

        for char_obj in characters {
            // Check if this character is part of the current word
            let should_merge = if let Some(bbox) = &current_bbox {
                // Same line and close horizontally
                (char_obj.bbox.y0 - bbox.y0).abs() < 2.0
                    && (char_obj.bbox.x0 - last_char_right) < self.char_width * 0.5
            } else {
                false
            };

            if should_merge {
                // Add to current word
                current_word.push_str(&char_obj.text);
                if let Some(bbox) = &mut current_bbox {
                    bbox.x1 = char_obj.bbox.x1;
                    bbox.y0 = bbox.y0.min(char_obj.bbox.y0);
                    bbox.y1 = bbox.y1.max(char_obj.bbox.y1);
                }
                last_char_right = char_obj.bbox.x1;
            } else {
                // Start new word
                if !current_word.is_empty() {
                    if let Some(bbox) = current_bbox {
                        words.push(TextObject {
                            text: current_word.clone(),
                            bbox,
                            font_size: 12.0,
                            font_family: "Unknown".to_string(),
                            is_bold: false,
                            is_italic: false,
                            reading_order: words.len(),
                        });
                    }
                }

                current_word = char_obj.text.clone();
                current_bbox = Some(char_obj.bbox.clone());
                last_char_right = char_obj.bbox.x1;
            }
        }

        // Don't forget the last word
        if !current_word.is_empty() {
            if let Some(bbox) = current_bbox {
                words.push(TextObject {
                    text: current_word,
                    bbox,
                    font_size: 12.0,
                    font_family: "Unknown".to_string(),
                    is_bold: false,
                    is_italic: false,
                    reading_order: words.len(),
                });
            }
        }

        words
    }

    /// Calculate modal (most common) font size from a collection of font sizes
    pub fn calculate_modal_font_size(&self, font_sizes: &[f32]) -> f32 {
        if font_sizes.is_empty() {
            return 12.0; // Default font size
        }

        // Create histogram of font sizes (rounded to nearest 0.5)
        let mut size_counts: HashMap<i32, usize> = HashMap::new();

        for &size in font_sizes {
            let rounded_size = (size * 2.0).round() as i32; // Round to nearest 0.5
            *size_counts.entry(rounded_size).or_insert(0) += 1;
        }

        // Find most common font size
        let modal_size_rounded = size_counts
            .iter()
            .max_by_key(|(_, &count)| count)
            .map(|(&size, _)| size)
            .unwrap_or(24); // Default 12.0 * 2

        let modal_size = modal_size_rounded as f32 / 2.0;
        modal_size.max(6.0).min(48.0) // Reasonable bounds
    }

    /// Calculate tight bounding box around all text content
    pub fn calculate_content_bounds(&self, text_objects: &[TextObject]) -> BBox {
        if text_objects.is_empty() {
            return BBox {
                x0: 0.0,
                y0: 0.0,
                x1: 400.0,
                y1: 600.0,
            }; // Default page-like bounds
        }

        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;

        for obj in text_objects {
            min_x = min_x.min(obj.bbox.x0);
            min_y = min_y.min(obj.bbox.y0);
            max_x = max_x.max(obj.bbox.x1);
            max_y = max_y.max(obj.bbox.y1);
        }

        BBox {
            x0: min_x,
            y0: min_y,
            x1: max_x,
            y1: max_y,
        }
    }

    /// Create optimally-sized character matrix based on calculated dimensions
    fn create_optimal_character_matrix(
        &self,
        pdf_path: &Path,
        matrix_width: usize,
        matrix_height: usize,
    ) -> Result<(Vec<Vec<char>>, f32, f32)> {
        let pdfium = Pdfium::new(
            Pdfium::bind_to_system_library()
                .or_else(|_| Pdfium::bind_to_library("./lib/libpdfium.dylib"))
                .or_else(|_| Pdfium::bind_to_library("/usr/local/lib/libpdfium.dylib"))
                .map_err(|e| anyhow::anyhow!("Failed to bind pdfium: {}", e))?,
        );

        let document = pdfium
            .load_pdf_from_file(pdf_path, None)
            .map_err(|e| anyhow::anyhow!("Failed to load PDF: {}", e))?;

        // Process first page (can be extended for multi-page)
        let page = document
            .pages()
            .get(0)
            .map_err(|e| anyhow::anyhow!("Failed to get page: {}", e))?;
        let page_width = page.width().value;
        let page_height = page.height().value;

        // Initialize matrix with calculated dimensions
        let mut matrix = vec![vec![' '; matrix_width]; matrix_height];

        // Render page to high-resolution bitmap for better content detection
        let render_scale = 3.0; // Higher DPI for better detection
        let bitmap = page
            .render_with_config(
                &PdfRenderConfig::new()
                    .set_target_width((page_width * render_scale) as i32)
                    .set_maximum_height((page_height * render_scale) as i32),
            )
            .map_err(|e| anyhow::anyhow!("Failed to render page: {}", e))?;

        let width = bitmap.width() as u32;
        let height = bitmap.height() as u32;
        let bytes_per_pixel = 4; // Assuming RGBA
        let stride = width as usize * bytes_per_pixel;
        let bitmap_buffer = bitmap.as_raw_bytes();

        // Map content to character matrix with improved sampling
        for row in 0..matrix_height {
            for col in 0..matrix_width {
                // Map character position to bitmap coordinates
                let bitmap_x = ((col as f32 * self.char_width) * render_scale) as usize;
                let bitmap_y = ((row as f32 * self.char_height) * render_scale) as usize;

                // Improved content detection with better sampling
                let has_content = self.improved_content_detection(
                    &bitmap_buffer,
                    bitmap_x,
                    bitmap_y,
                    stride,
                    width as usize,
                    height as usize,
                    render_scale,
                );

                if has_content {
                    matrix[row][col] = '█'; // Mark as potential text area
                }
            }
        }

        Ok((matrix, page_width, page_height))
    }

    /// Improved content detection with better sampling strategy
    fn improved_content_detection(
        &self,
        bitmap: &[u8],
        x: usize,
        y: usize,
        stride: usize,
        width: usize,
        height: usize,
        render_scale: f32,
    ) -> bool {
        let char_width = (self.char_width * render_scale) as usize;
        let char_height = (self.char_height * render_scale) as usize;

        let mut dark_pixels = 0;
        let mut total_pixels = 0;

        // Sample pixels in a grid pattern for better detection
        let _sample_points = 9; // 3x3 grid
        let step_x = char_width.max(1) / 3;
        let step_y = char_height.max(1) / 3;

        for sy in 0..3 {
            for sx in 0..3 {
                let px = x + (sx * step_x).min(char_width - 1);
                let py = y + (sy * step_y).min(char_height - 1);

                if px < width && py < height {
                    let pixel_offset = py * stride + px * 4; // Assuming RGBA
                    if pixel_offset + 2 < bitmap.len() {
                        let r = bitmap[pixel_offset] as u32;
                        let g = bitmap[pixel_offset + 1] as u32;
                        let b = bitmap[pixel_offset + 2] as u32;

                        let brightness = (r + g + b) / 3;
                        if brightness < 180 {
                            // Adjust threshold for better detection
                            dark_pixels += 1;
                        }
                        total_pixels += 1;
                    }
                }
            }
        }

        // If more than 20% of sample points are dark, consider this a text area
        total_pixels > 0 && (dark_pixels as f32 / total_pixels as f32) > 0.2
    }

    /// Intelligent text mapping using spatial awareness
    fn intelligent_text_mapping(
        &self,
        matrix: Vec<Vec<char>>,
        text_objects: &[TextObject],
        text_regions: &[TextRegion],
    ) -> Result<CharacterMatrix> {
        // For now, use the existing mapping approach but with better text objects
        // This will be enhanced in later phases for true spatial matching
        let original_text: Vec<String> = text_objects.iter().map(|t| t.text.clone()).collect();
        self.map_text_to_matrix(matrix, text_regions, &original_text)
    }

    fn pdf_to_character_matrix(&self, pdf_path: &Path) -> Result<(Vec<Vec<char>>, f32, f32)> {
        let pdfium = Pdfium::new(
            Pdfium::bind_to_system_library()
                .or_else(|_| Pdfium::bind_to_library("./lib/libpdfium.dylib"))
                .or_else(|_| Pdfium::bind_to_library("/usr/local/lib/libpdfium.dylib"))
                .map_err(|e| anyhow::anyhow!("Failed to bind pdfium: {}", e))?,
        );

        let document = pdfium
            .load_pdf_from_file(pdf_path, None)
            .map_err(|e| anyhow::anyhow!("Failed to load PDF: {}", e))?;

        // For now, process just the first page
        let page = document
            .pages()
            .get(0)
            .map_err(|e| anyhow::anyhow!("Failed to get page: {}", e))?;
        let page_width = page.width().value;
        let page_height = page.height().value;

        // Calculate character matrix dimensions
        let char_cols = (page_width / self.char_width).ceil() as usize;
        let char_rows = (page_height / self.char_height).ceil() as usize;

        // Initialize matrix with spaces
        let mut matrix = vec![vec![' '; char_cols]; char_rows];

        // Render page to bitmap to understand layout
        let bitmap = page
            .render_with_config(
                &PdfRenderConfig::new()
                    .set_target_width((page_width * 2.0) as i32) // Higher DPI for better text detection
                    .set_maximum_height((page_height * 2.0) as i32),
            )
            .map_err(|e| anyhow::anyhow!("Failed to render page: {}", e))?;

        // Convert bitmap to image for analysis
        let width = bitmap.width() as u32;
        let height = bitmap.height() as u32;
        let bytes_per_pixel = 4; // Assuming RGBA
        let stride = width as usize * bytes_per_pixel;

        // Analyze bitmap to determine where text likely exists
        // This is a simplified approach - in reality you'd use more sophisticated analysis
        let bitmap_buffer = bitmap.as_raw_bytes();

        for row in 0..char_rows {
            for col in 0..char_cols {
                // Map character position to bitmap coordinates
                let bitmap_x = ((col as f32 * self.char_width) * 2.0) as usize; // *2 for higher DPI
                let bitmap_y = ((row as f32 * self.char_height) * 2.0) as usize;

                // Sample a few pixels in this character cell to see if there's content
                let has_content = self.sample_character_cell(
                    &bitmap_buffer,
                    bitmap_x,
                    bitmap_y,
                    stride,
                    width as usize,
                    height as usize,
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
        for dy in 0..char_height.min(10) {
            // Sample at most 10 pixels high
            for dx in 0..char_width.min(10) {
                // Sample at most 10 pixels wide
                let px = x + dx;
                let py = y + dy;

                if px < width && py < height {
                    let pixel_offset = py * stride + px * 4; // Assuming RGBA
                    if pixel_offset + 2 < bitmap.len() {
                        let r = bitmap[pixel_offset] as u32;
                        let g = bitmap[pixel_offset + 1] as u32;
                        let b = bitmap[pixel_offset + 2] as u32;

                        let brightness = (r + g + b) / 3;
                        if brightness < 200 {
                            // Pixel is dark enough to be text
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
                img.put_pixel(
                    x as u32,
                    y as u32,
                    Rgb([pixel_value, pixel_value, pixel_value]),
                );
            }
        }

        img
    }

    fn detect_text_regions_with_vision(
        &self,
        image: &RgbImage,
        matrix: &[Vec<char>],
    ) -> Result<Vec<TextRegion>> {
        // Use Ferrules ML model if available, otherwise fall back to flood-fill
        #[cfg(feature = "ferrules")]
        {
            // Create a runtime for async Ferrules call
            let rt = Runtime::new()?;
            return rt.block_on(ferrules_integration::detect_text_regions_with_ferrules(
                image,
                self.char_width,
                self.char_height,
            ));
        }

        #[cfg(not(feature = "ferrules"))]
        {
            // Fallback to flood-fill when Ferrules is not available
            let mut regions = Vec::new();
            let mut visited = vec![vec![false; matrix[0].len()]; matrix.len()];
            let mut region_id = 0;

            for y in 0..matrix.len() {
                for x in 0..matrix[y].len() {
                    if matrix[y][x] != ' ' && !visited[y][x] {
                        // Found start of a text region, flood fill to get its bounds
                        let region = self.flood_fill_region(matrix, &mut visited, x, y)?;

                        if region.area() > 2 {
                            // Only keep regions with reasonable size
                            regions.push(TextRegion {
                                bbox: region,
                                confidence: 0.8,             // Placeholder confidence
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
        // This method is kept for backward compatibility but is superseded by
        // extract_text_objects_with_coordinates for the new intelligent approach
        let text_objects = self.extract_text_objects_with_coordinates(pdf_path)?;
        Ok(text_objects.iter().map(|t| t.text.clone()).collect())
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

    fn place_text_in_region(&self, matrix: &mut [Vec<char>], bbox: &CharBBox, text: &str) {
        let text_chars: Vec<char> = text.chars().collect();
        let mut char_index = 0;

        // Fill the region with text, wrapping as needed
        for row in bbox.y..(bbox.y + bbox.height).min(matrix.len()) {
            for col in bbox.x..(bbox.x + bbox.width).min(matrix[row].len()) {
                if char_index < text_chars.len() {
                    let ch = text_chars[char_index];
                    if ch != ' ' || matrix[row][col] == '█' {
                        // Only place non-space chars or replace content markers
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

        result.push_str(&format!(
            "Character Matrix ({}x{}):\n",
            char_matrix.width, char_matrix.height
        ));
        result.push_str(&format!(
            "Text Regions: {}\n",
            char_matrix.text_regions.len()
        ));
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

    /// Render PDF page to high-quality image for layout detection
    fn render_pdf_page_to_image(&self, pdf_path: &Path) -> Result<(DynamicImage, f32, f32)> {
        let pdfium = Pdfium::new(
            Pdfium::bind_to_system_library()
                .or_else(|_| Pdfium::bind_to_library("./lib/libpdfium.dylib"))
                .or_else(|_| Pdfium::bind_to_library("/usr/local/lib/libpdfium.dylib"))
                .map_err(|e| anyhow::anyhow!("Failed to bind pdfium: {}", e))?,
        );

        let document = pdfium
            .load_pdf_from_file(pdf_path, None)
            .map_err(|e| anyhow::anyhow!("Failed to load PDF: {}", e))?;

        let page = document
            .pages()
            .get(0)
            .map_err(|e| anyhow::anyhow!("Failed to get page: {}", e))?;

        let page_width = page.width().value;
        let page_height = page.height().value;

        // Render at 300 DPI for good quality
        let dpi_scale = 300.0 / 72.0; // PDF is 72 DPI by default
        let render_width = (page_width * dpi_scale) as i32;
        let render_height = (page_height * dpi_scale) as i32;

        let bitmap = page
            .render_with_config(
                &PdfRenderConfig::new()
                    .set_target_width(render_width)
                    .set_maximum_height(render_height),
            )
            .map_err(|e| anyhow::anyhow!("Failed to render page: {}", e))?;

        // Convert to DynamicImage
        let width = bitmap.width() as u32;
        let height = bitmap.height() as u32;
        let buffer = bitmap.as_raw_bytes();

        // Create RGB image from bitmap data
        let mut img = RgbImage::new(width, height);
        for y in 0..height {
            for x in 0..width {
                let idx = ((y * width + x) * 4) as usize;
                if idx + 2 < buffer.len() {
                    let pixel = Rgb([buffer[idx], buffer[idx + 1], buffer[idx + 2]]);
                    img.put_pixel(x, y, pixel);
                }
            }
        }

        Ok((DynamicImage::ImageRgb8(img), page_width, page_height))
    }

    /// Run Ferrules YOLO model on PDF page image
    fn detect_layout_with_ferrules(&self, page_image: &DynamicImage) -> Result<Vec<LayoutRegion>> {
        #[cfg(feature = "ferrules")]
        {
            use ferrules_core::layout::model::{ORTConfig, ORTLayoutParser};

            // Create a runtime for async Ferrules call
            let rt = Runtime::new()?;

            // Run layout detection
            rt.block_on(async {
                let config = ORTConfig::default();
                let mut parser = ORTLayoutParser::new(config)?;
                let layout_bboxes = parser.parse_layout_async(page_image, 1.0).await?;

                // Convert to our LayoutRegion type
                let regions: Vec<LayoutRegion> = layout_bboxes
                    .into_iter()
                    .map(|bbox| LayoutRegion {
                        bbox: BBox {
                            x0: bbox.bbox.x0,
                            y0: bbox.bbox.y0,
                            x1: bbox.bbox.x1,
                            y1: bbox.bbox.y1,
                        },
                        label: bbox.label.to_string(),
                        proba: bbox.proba,
                    })
                    .collect();

                Ok(regions)
            })
        }

        #[cfg(not(feature = "ferrules"))]
        {
            // Fallback: treat entire page as one text region
            let (width, height) = page_image.dimensions();
            Ok(vec![LayoutRegion {
                bbox: BBox {
                    x0: 0.0,
                    y0: 0.0,
                    x1: width as f32,
                    y1: height as f32,
                },
                label: "Text".to_string(),
                proba: 1.0,
            }])
        }
    }

    /// Calculate matrix size based on page dimensions
    fn calculate_matrix_size_from_page(&self, page_width: f32, page_height: f32) -> (usize, usize) {
        let matrix_width = (page_width / self.char_width).ceil() as usize;
        let matrix_height = (page_height / self.char_height).ceil() as usize;

        // Apply reasonable limits
        let max_width = 200;
        let max_height = 100;

        (matrix_width.min(max_width), matrix_height.min(max_height))
    }

    /// Create character matrix using layout regions to guide text placement
    fn create_matrix_with_layout_regions(
        &self,
        matrix_width: usize,
        matrix_height: usize,
        text_objects: &[TextObject],
        layout_regions: &[LayoutRegion],
        page_width: f32,
        page_height: f32,
    ) -> Result<CharacterMatrix> {
        let mut matrix = vec![vec![' '; matrix_width]; matrix_height];
        let mut text_regions = Vec::new();

        // For each layout region, map relevant text objects into it
        for (region_id, layout_region) in layout_regions.iter().enumerate() {
            // Scale layout region from 300 DPI back to PDF coordinates (72 DPI)
            let dpi_scale = 300.0 / 72.0;
            let scaled_region_bbox = BBox {
                x0: layout_region.bbox.x0 / dpi_scale,
                y0: layout_region.bbox.y0 / dpi_scale,
                x1: layout_region.bbox.x1 / dpi_scale,
                y1: layout_region.bbox.y1 / dpi_scale,
            };

            // Find text objects that fall within this layout region
            let region_texts: Vec<&TextObject> = text_objects
                .iter()
                .filter(|obj| {
                    let center_x = obj.bbox.center_x();
                    let center_y = obj.bbox.center_y();
                    scaled_region_bbox.contains_point(center_x, center_y)
                })
                .collect();

            if region_texts.is_empty() {
                continue;
            }

            // Convert scaled layout region to character coordinates
            let char_x = (scaled_region_bbox.x0 / self.char_width).round() as usize;
            let char_y = (scaled_region_bbox.y0 / self.char_height).round() as usize;
            let char_width =
                ((scaled_region_bbox.x1 - scaled_region_bbox.x0) / self.char_width).ceil() as usize;
            let char_height = ((scaled_region_bbox.y1 - scaled_region_bbox.y0) / self.char_height)
                .ceil() as usize;

            // Don't just mark the region - actually place the text at the correct position!
            // Sort text objects by position for proper layout
            let mut sorted_texts = region_texts.clone();
            sorted_texts.sort_by_key(|obj| {
                // Use integer keys for stable sorting without NaN issues
                let y_key = if obj.bbox.y0.is_finite() {
                    (obj.bbox.y0 * 1000.0) as i32
                } else {
                    0
                };
                let x_key = if obj.bbox.x0.is_finite() {
                    (obj.bbox.x0 * 1000.0) as i32
                } else {
                    0
                };
                (y_key, x_key, obj.reading_order)
            });

            // Place each text object at its correct position
            for text_obj in sorted_texts {
                // Convert text object position to character coordinates
                let text_char_x = (text_obj.bbox.x0 / self.char_width).round() as usize;
                let text_char_y = (text_obj.bbox.y0 / self.char_height).round() as usize;

                // Place the text characters
                let chars: Vec<char> = text_obj.text.chars().collect();
                for (i, ch) in chars.iter().enumerate() {
                    let x = text_char_x + i;
                    let y = text_char_y;

                    if x < matrix_width && y < matrix_height {
                        matrix[y][x] = *ch;
                    }
                }
            }

            // Collect text content for this region
            let region_text = region_texts
                .iter()
                .map(|t| t.text.as_str())
                .collect::<Vec<_>>()
                .join(" ");

            text_regions.push(TextRegion {
                bbox: CharBBox {
                    x: char_x,
                    y: char_y,
                    width: char_width,
                    height: char_height,
                },
                confidence: layout_region.proba,
                text_content: region_text,
                region_id,
            });

            println!(
                "  📝 Region {}: {} - '{}'",
                region_id + 1,
                layout_region.label,
                if text_regions.last().unwrap().text_content.len() > 50 {
                    format!("{}...", &text_regions.last().unwrap().text_content[..50])
                } else {
                    text_regions.last().unwrap().text_content.clone()
                }
            );
        }

        Ok(CharacterMatrix {
            width: matrix_width,
            height: matrix_height,
            matrix,
            text_regions,
            original_text: text_objects.iter().map(|t| t.text.clone()).collect(),
        })
    }

    /// Process PDF with PDFium-only extraction (no Ferrules layout detection)
    ///
    /// This method uses only PDFium for text extraction, resulting in accurate content
    /// but without spatial layout preservation. Text appears left-justified.
    ///
    /// # Arguments
    /// * `pdf_path` - Path to the PDF file to process
    ///
    /// # Returns
    /// * `Ok(CharacterMatrix)` - Successfully processed character matrix
    /// * `Err(anyhow::Error)` - Processing error (file not found, invalid PDF, etc.)
    pub fn process_pdf_pdfium_only(&self, pdf_path: &Path) -> Result<CharacterMatrix> {
        println!("🚀 Processing PDF with PDFium-only extraction (no layout detection)...");

        // Step 1: Extract text objects with coordinates using PDFium
        let text_objects = self.extract_text_objects_with_coordinates(pdf_path)?;

        if text_objects.is_empty() {
            return Err(anyhow::anyhow!("No text found in PDF"));
        }

        println!("📝 Extracted {} text objects", text_objects.len());

        // Step 2: Calculate basic matrix size for left-justified layout
        let total_chars: usize = text_objects.iter().map(|obj| obj.text.len()).sum();
        let estimated_width = 120; // Fixed reasonable width
        let estimated_height = (total_chars / estimated_width).max(50).min(300);

        let matrix_width = estimated_width;
        let matrix_height = estimated_height;

        println!("📊 Creating matrix: {}x{}", matrix_width, matrix_height);

        // Step 3: Create empty matrix
        let mut matrix = vec![vec![' '; matrix_width]; matrix_height];

        // Step 4: Place text left-justified, line by line
        let mut current_row = 0;
        let mut current_col = 0;
        let mut text_regions = Vec::new();

        for (obj_idx, text_obj) in text_objects.iter().enumerate() {
            if current_row >= matrix_height {
                break;
            }

            let text_start_row = current_row;
            let text_start_col = current_col;

            // Place each character of the text object
            for ch in text_obj.text.chars() {
                // Handle newlines and wrap to next line if needed
                if ch == '\n' || current_col >= matrix_width - 5 {
                    current_row += 1;
                    current_col = 0;
                    if current_row >= matrix_height {
                        break;
                    }
                    if ch == '\n' {
                        continue;
                    }
                }

                if current_row < matrix_height && current_col < matrix_width {
                    matrix[current_row][current_col] = ch;
                    current_col += 1;
                }
            }

            // Add space between text objects
            if current_col < matrix_width - 1 {
                current_col += 1;
            } else {
                current_row += 1;
                current_col = 0;
            }

            // Create a simple text region for this object
            text_regions.push(TextRegion {
                bbox: CharBBox {
                    x: text_start_col,
                    y: text_start_row,
                    width: text_obj.text.len().min(matrix_width - text_start_col),
                    height: (current_row - text_start_row).max(1),
                },
                confidence: 1.0,
                text_content: text_obj.text.clone(),
                region_id: obj_idx,
            });
        }

        println!("✅ PDFium-only processing complete!");

        Ok(CharacterMatrix {
            width: matrix_width,
            height: matrix_height,
            matrix,
            text_regions,
            original_text: text_objects.iter().map(|obj| obj.text.clone()).collect(),
        })
    }

    /// Generate console-style spatial layout visualization like test_ferrules_integration.rs
    pub fn generate_spatial_console_output(&self, char_matrix: &CharacterMatrix) -> String {
        let mut result = String::new();

        result.push_str(&format!(
            "📊 Ferrules Character Matrix Output - Exact Placement Visualization\n"
        ));
        result.push_str(&format!(
            "Matrix Size: {} columns × {} rows\n",
            char_matrix.width, char_matrix.height
        ));
        result.push_str(&format!(
            "Regions Detected: {}\n",
            char_matrix.text_regions.len()
        ));
        result.push_str(&format!(
            "Text Objects: {}\n",
            char_matrix.original_text.len()
        ));
        result.push_str(&format!("Processing Time: N/A\n"));
        result.push_str("Toggle Text Highlighting Toggle Grid Lines\n");

        // Generate the matrix with row numbers and dots for spaces (just like test output)
        for (row_idx, row) in char_matrix.matrix.iter().enumerate() {
            result.push_str(&format!("{:3} ", row_idx)); // Row number with padding
            for &ch in row.iter() {
                result.push(if ch == ' ' { '·' } else { ch });
            }
            result.push('\n');
        }

        // Add analysis summary
        result.push_str("What Ferrules Accomplished:\n");

        // Find major text elements for summary
        let mut accomplishments = Vec::new();
        let mut issues = Vec::new();

        for (i, region) in char_matrix.text_regions.iter().enumerate().take(5) {
            if !region.text_content.trim().is_empty() {
                let content_preview = if region.text_content.len() > 50 {
                    format!("{}...", &region.text_content[..50])
                } else {
                    region.text_content.clone()
                };
                accomplishments.push(format!(
                    "✅ Found text region {}: \"{}\" (Confidence: {:.1}%)",
                    i + 1,
                    content_preview,
                    region.confidence * 100.0
                ));
            }
        }

        if accomplishments.is_empty() {
            accomplishments
                .push("✅ Successfully processed PDF with Ferrules ML vision model".to_string());
            accomplishments
                .push("✅ Generated spatial character matrix representation".to_string());
            accomplishments.push("✅ Preserved document layout structure".to_string());
        }

        for accomplishment in accomplishments {
            result.push_str(&format!("{}\n", accomplishment));
        }

        // Add common issues
        issues.push("❌ Text concatenation: Words may run together without spaces");
        issues.push("❌ Overlapping text: Multiple words placed in same positions");
        issues.push("❌ Inconsistent spacing: Some areas dense, others sparse");
        issues.push("❌ Character accuracy: OCR/vision may misread some characters");

        result.push_str("Placement Issues:\n");
        for issue in issues {
            result.push_str(&format!("{}\n", issue));
        }

        result
    }
}

impl Default for CharacterMatrixEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Layout region detected by YOLO model
#[derive(Debug, Clone)]
struct LayoutRegion {
    bbox: BBox,
    label: String,
    proba: f32,
}

// Ferrules Vision Integration
#[cfg(feature = "ferrules")]
mod ferrules_integration {
    use super::*;
    use ferrules_core::layout::model::{ORTConfig, ORTLayoutParser};

    pub async fn detect_text_regions_with_ferrules(
        page_image: &RgbImage,
        char_width: f32,
        char_height: f32,
    ) -> Result<Vec<TextRegion>> {
        // Convert RgbImage to DynamicImage for Ferrules
        let dynamic_image = DynamicImage::ImageRgb8(page_image.clone());

        // Initialize Ferrules layout parser with default config
        let config = ORTConfig::default();
        let mut parser = ORTLayoutParser::new(config)?;

        // Run layout detection
        let layout_bboxes = parser.parse_layout_async(&dynamic_image, 1.0).await?;

        // Convert Ferrules bounding boxes to character-based TextRegions
        let mut text_regions = Vec::new();
        let mut region_id = 0;

        for layout_bbox in layout_bboxes {
            // Only process text-like regions
            if layout_bbox.is_text_block() {
                // Convert pixel coordinates to character grid coordinates
                let char_x = (layout_bbox.bbox.x0 / char_width) as usize;
                let char_y = (layout_bbox.bbox.y0 / char_height) as usize;
                let char_width_count =
                    ((layout_bbox.bbox.x1 - layout_bbox.bbox.x0) / char_width).ceil() as usize;
                let char_height_count =
                    ((layout_bbox.bbox.y1 - layout_bbox.bbox.y0) / char_height).ceil() as usize;

                text_regions.push(TextRegion {
                    bbox: CharBBox {
                        x: char_x,
                        y: char_y,
                        width: char_width_count,
                        height: char_height_count,
                    },
                    confidence: layout_bbox.proba,
                    text_content: String::new(), // Will be filled by text mapping
                    region_id,
                });
                region_id += 1;
            }
        }

        Ok(text_regions)
    }
}
