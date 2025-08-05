#!/usr/bin/env rust-script
//! # Chonker 5: Character Matrix PDF Engine
//!
//! A PDF processing application that converts PDFs into character matrices for spatial analysis.
//! This tool combines PDF text extraction with vision-based region detection to create faithful
//! character representations of PDF documents.
//!
//! ## Key Features
//! - PDF to character matrix conversion
//! - Text region detection using character coordinate analysis
//! - Precise text extraction using PDFium
//! - Interactive GUI with real-time preview
//! - Export capabilities for processed matrices
//!
//! ## Architecture
//! - `CharacterMatrixEngine`: Core processing engine for PDF analysis
//! - `Chonker5App`: Main GUI application with egui framework
//! - `CharacterMatrix`: Data structure for spatial text representation
//! - `TextRegion`: Character-space bounding boxes with confidence scores
//!
//! ```cargo
//! [dependencies]
//! eframe = "0.24"
//! egui = "0.24"
//! rfd = "0.15"
//! image = "0.25"
//! pdfium-render = { version = "0.8", features = ["thread_safe"] }
//! tokio = { version = "1.38", features = ["full", "rt-multi-thread"] }
//! anyhow = "1.0"
//! tracing = "0.1"
//! tracing-subscriber = { version = "0.3", features = ["env-filter"] }
//! serde = { version = "1.0", features = ["derive"] }
//! serde_json = "1.0"
//! ```

use anyhow::Result;
use eframe::egui;
use egui::{Color32, FontId, RichText, Rounding, Stroke};
use image::{ImageBuffer, Rgb, RgbImage};
use pdfium_render::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use tokio::sync::mpsc;

// Teal and chrome color scheme
const TERM_BG: Color32 = Color32::from_rgb(10, 15, 20); // Dark blue-black
const TERM_FG: Color32 = Color32::from_rgb(26, 188, 156); // Teal (#1ABC9C)
                                                          // const TERM_BORDER: Color32 = Color32::from_rgb(192, 192, 192); // Chrome/silver (unused)
const TERM_HIGHLIGHT: Color32 = Color32::from_rgb(22, 160, 133); // Darker teal (#16A085)
const TERM_ERROR: Color32 = Color32::from_rgb(255, 80, 80); // Soft red
const TERM_DIM: Color32 = Color32::from_rgb(80, 100, 100); // Muted teal-gray
const TERM_YELLOW: Color32 = Color32::from_rgb(255, 200, 0); // Gold accent
const CHROME: Color32 = Color32::from_rgb(82, 86, 89); // Chrome (#525659)

// Box drawing characters (for future use)
#[allow(dead_code)]
const BOX_TL: &str = "╔";
#[allow(dead_code)]
const BOX_TR: &str = "╗";
#[allow(dead_code)]
const BOX_BL: &str = "╚";
#[allow(dead_code)]
const BOX_BR: &str = "╝";
#[allow(dead_code)]
const BOX_H: &str = "═";
#[allow(dead_code)]
const BOX_V: &str = "║";
#[allow(dead_code)]
const BOX_T: &str = "╦";
#[allow(dead_code)]
const BOX_B: &str = "╩";
#[allow(dead_code)]
const BOX_L: &str = "╠";
#[allow(dead_code)]
const BOX_R: &str = "╣";
#[allow(dead_code)]
const BOX_CROSS: &str = "╬";

// ============= ENHANCED CHARACTER MATRIX ENGINE =============
// Character matrix PDF representation system with precise text extraction

/// Core data structure representing a PDF as a character matrix.
///
/// This structure contains the spatial character representation of a PDF page,
/// along with identified text regions and their corresponding content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterMatrix {
    pub width: usize,
    pub height: usize,
    pub matrix: Vec<Vec<char>>,
    pub text_regions: Vec<TextRegion>,
    pub original_text: Vec<String>,
    pub char_width: f32,
    pub char_height: f32,
}

/// Represents a rectangular region within the character matrix that contains text.
///
/// Each region has a bounding box in character coordinates and associated metadata
/// including confidence score and extracted text content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextRegion {
    pub bbox: CharBBox,
    pub confidence: f32,
    pub text_content: String,
    pub region_id: usize,
}

/// Character-space bounding box coordinates.
///
/// Unlike PDF coordinates which use points, this uses discrete character positions
/// within the character matrix grid.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharBBox {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

impl CharBBox {
    /// Check if the given character coordinates are within this bounding box.
    ///
    /// # Arguments
    /// * `x` - Character column position
    /// * `y` - Character row position
    ///
    /// # Returns
    /// `true` if the coordinates are within the bounding box
    pub fn contains(&self, x: usize, y: usize) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }

    /// Calculate the area of this bounding box in character units.
    ///
    /// # Returns
    /// The area as width × height in character positions
    pub fn area(&self) -> usize {
        self.width * self.height
    }
}

#[derive(Debug, Clone)]
struct PreciseTextObject {
    text: String,
    bbox: PDFBBox,
    font_size: f32,
    #[allow(dead_code)] // May be used in future enhancements
    font_name: String,
    #[allow(dead_code)] // May be used in future enhancements
    page_index: usize,
}

#[derive(Debug, Clone)]
struct PDFBBox {
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
}

impl PDFBBox {
    #[allow(dead_code)] // Utility method for future use
    fn width(&self) -> f32 {
        self.x1 - self.x0
    }

    #[allow(dead_code)] // Utility method for future use
    fn height(&self) -> f32 {
        self.y1 - self.y0
    }
}

/// Main processing engine for converting PDFs to character matrices.
///
/// This engine handles the core workflow:
/// 1. Extract text objects with precise coordinates from PDF
/// 2. Convert PDF space to character matrix space
/// 3. Create character matrix representation
/// 4. Detect text regions using spatial analysis
/// 5. Map extracted text content to regions
pub struct CharacterMatrixEngine {
    pub char_width: f32,
    pub char_height: f32,
}

impl CharacterMatrixEngine {
    pub fn new() -> Self {
        Self {
            char_width: 6.0,   // Will be calculated dynamically
            char_height: 12.0, // Will be calculated dynamically
        }
    }

    // STEP 1: Extract precise text objects with coordinates (Enhanced Pdfium)
    fn extract_text_objects_with_precise_coords(
        &self,
        pdf_path: &PathBuf,
    ) -> Result<Vec<PreciseTextObject>> {
        let pdfium = Pdfium::new(
            Pdfium::bind_to_system_library()
                .or_else(|_| Pdfium::bind_to_library("./lib/libpdfium.dylib"))
                .or_else(|_| Pdfium::bind_to_library("/usr/local/lib/libpdfium.dylib"))
                .map_err(|e| anyhow::anyhow!("Failed to bind pdfium: {}", e))?,
        );

        let document = pdfium.load_pdf_from_file(pdf_path, None)?;
        let mut text_objects = Vec::new();

        for (page_index, page) in document.pages().iter().enumerate() {
            let text_page = page.text()?;
            let page_height = page.height().value;

            // Get all characters with their precise positions
            // Note: chars() API might not provide individual bounds, so let's try a different approach
            // We'll extract text in smaller segments for better positioning
            let text_segments = text_page.segments();

            for segment in text_segments.iter() {
                let bounds = segment.bounds();
                let text = segment.text();

                // Only process non-whitespace segments
                if !text.trim().is_empty() {
                    // PDF coordinates have origin at bottom-left, convert to top-left
                    let y_from_top = page_height - bounds.top().value;

                    text_objects.push(PreciseTextObject {
                        text: text.clone(),
                        bbox: PDFBBox {
                            x0: bounds.left().value,
                            y0: y_from_top,
                            x1: bounds.right().value,
                            y1: y_from_top + (bounds.top().value - bounds.bottom().value),
                        },
                        font_size: 12.0, // Default font size for segments
                        font_name: "Segment".to_string(),
                        page_index,
                    });
                }
            }
        }

        // If no characters found, fall back to word-level extraction
        if text_objects.is_empty() {
            for (page_index, page) in document.pages().iter().enumerate() {
                let text_page = page.text()?;
                let _page_height = page.height().value;

                // Try to get text segments (words/lines)
                let all_text = text_page.all();
                let lines: Vec<&str> = all_text.lines().collect();

                for (line_idx, line) in lines.iter().enumerate() {
                    if !line.trim().is_empty() {
                        // Estimate line position
                        let line_y = 50.0 + (line_idx as f32 * 20.0);

                        text_objects.push(PreciseTextObject {
                            text: line.to_string(),
                            bbox: PDFBBox {
                                x0: 50.0,
                                y0: line_y,
                                x1: 500.0,
                                y1: line_y + 15.0,
                            },
                            font_size: 12.0,
                            font_name: "Line".to_string(),
                            page_index,
                        });
                    }
                }
            }
        }

        Ok(text_objects)
    }

    // STEP 2: Calculate optimal matrix size based on actual content
    fn calculate_optimal_matrix_size(
        &self,
        text_objects: &[PreciseTextObject],
    ) -> (usize, usize, f32, f32) {
        if text_objects.is_empty() {
            return (50, 50, 6.0, 12.0);
        }

        // Find the modal (most common) font size
        let mut font_size_counts: HashMap<i32, usize> = HashMap::new();
        for obj in text_objects {
            let rounded_size = obj.font_size.round() as i32;
            *font_size_counts.entry(rounded_size).or_insert(0) += 1;
        }

        let modal_font_size = font_size_counts
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(size, _)| *size as f32)
            .unwrap_or(12.0);

        // Calculate character dimensions based on modal font
        let char_width = modal_font_size * 0.6; // Typical character width ratio
        let char_height = modal_font_size * 1.2; // Typical line height ratio

        // Find actual content bounds (not page bounds)
        let min_x = text_objects
            .iter()
            .map(|t| t.bbox.x0)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);
        let max_x = text_objects
            .iter()
            .map(|t| t.bbox.x1)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(100.0);
        let min_y = text_objects
            .iter()
            .map(|t| t.bbox.y0)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);
        let max_y = text_objects
            .iter()
            .map(|t| t.bbox.y1)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(100.0);

        let content_width = max_x - min_x;
        let content_height = max_y - min_y;

        // Calculate smallest matrix that can contain all content
        let matrix_width = ((content_width / char_width).ceil() as usize).max(10);
        let matrix_height = ((content_height / char_height).ceil() as usize).max(10);

        (matrix_width, matrix_height, char_width, char_height)
    }

    // STEP 3: Generate optimized character matrix
    fn generate_optimal_character_matrix(
        &self,
        text_objects: &[PreciseTextObject],
        matrix_width: usize,
        matrix_height: usize,
        char_width: f32,
        char_height: f32,
    ) -> Result<Vec<Vec<char>>> {
        // Initialize matrix with spaces
        let mut matrix = vec![vec![' '; matrix_width]; matrix_height];

        // Mark positions where text actually exists
        for text_obj in text_objects {
            let char_x = ((text_obj.bbox.x0 / char_width) as usize).min(matrix_width - 1);
            let char_y = ((text_obj.bbox.y0 / char_height) as usize).min(matrix_height - 1);

            // Mark this position as containing text
            if char_y < matrix.len() && char_x < matrix[char_y].len() {
                matrix[char_y][char_x] = '█'; // Block character to mark text areas
            }
        }

        Ok(matrix)
    }

    // STEP 4: Enhanced vision processing with ferrules
    fn run_ferrules_on_matrix(
        &self,
        matrix: &[Vec<char>],
        ferrules_path: &PathBuf,
    ) -> Result<Vec<TextRegion>> {
        // Convert matrix to high-quality image for ferrules
        let image = self.character_matrix_to_image_high_quality(matrix, 300.0)?;

        // Save as temp file
        let temp_image = std::env::temp_dir().join("chonker5_matrix_ferrules.png");
        image.save(&temp_image)?;

        // Run ferrules
        let output = Command::new(ferrules_path)
            .arg(&temp_image)
            .arg("--output-json")
            .arg("--confidence-threshold")
            .arg("0.5")
            .output()?;

        if !output.status.success() {
            // Fallback to simple region detection if ferrules fails
            return self.detect_text_regions_fallback(matrix);
        }

        // Parse ferrules JSON output
        let ferrules_result: serde_json::Value = serde_json::from_slice(&output.stdout)?;

        // Convert ferrules regions to character coordinates
        let char_regions = self.convert_ferrules_to_char_regions(&ferrules_result, matrix)?;

        // Clean up
        let _ = std::fs::remove_file(&temp_image);

        Ok(char_regions)
    }

    fn character_matrix_to_image_high_quality(
        &self,
        matrix: &[Vec<char>],
        dpi: f32,
    ) -> Result<RgbImage> {
        let scale_factor = dpi / 72.0; // 72 DPI base
        let pixel_width = (matrix[0].len() as f32 * 8.0 * scale_factor) as u32;
        let pixel_height = (matrix.len() as f32 * 12.0 * scale_factor) as u32;

        let mut img = ImageBuffer::new(pixel_width, pixel_height);

        for (y, row) in matrix.iter().enumerate() {
            for (x, &ch) in row.iter().enumerate() {
                let pixel_x = (x as f32 * 8.0 * scale_factor) as u32;
                let pixel_y = (y as f32 * 12.0 * scale_factor) as u32;

                let color = if ch == ' ' { 255 } else { 0 }; // White for space, black for content

                // Fill a block for each character
                for dy in 0..(12.0 * scale_factor) as u32 {
                    for dx in 0..(8.0 * scale_factor) as u32 {
                        if pixel_x + dx < pixel_width && pixel_y + dy < pixel_height {
                            img.put_pixel(pixel_x + dx, pixel_y + dy, Rgb([color, color, color]));
                        }
                    }
                }
            }
        }

        Ok(img)
    }

    fn convert_ferrules_to_char_regions(
        &self,
        _ferrules_result: &serde_json::Value,
        matrix: &[Vec<char>],
    ) -> Result<Vec<TextRegion>> {
        // Placeholder for ferrules JSON parsing
        // In real implementation, parse ferrules output and convert pixel coordinates to character coordinates
        self.detect_text_regions_fallback(matrix)
    }

    fn detect_text_regions_fallback(&self, matrix: &[Vec<char>]) -> Result<Vec<TextRegion>> {
        let mut regions = Vec::new();
        let mut visited = vec![vec![false; matrix[0].len()]; matrix.len()];
        let mut region_id = 0;

        for y in 0..matrix.len() {
            for x in 0..matrix[y].len() {
                if matrix[y][x] != ' ' && !visited[y][x] {
                    // Found start of a text region, flood fill to get its bounds
                    let region = self.flood_fill_region(matrix, &mut visited, x, y)?;

                    if region.area() > 2 {
                        regions.push(TextRegion {
                            bbox: region,
                            confidence: 0.8,
                            text_content: String::new(),
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

                    if ny < matrix.len()
                        && nx < matrix[ny].len()
                        && !visited[ny][nx]
                        && matrix[ny][nx] != ' '
                    {
                        stack.push((nx, ny));
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

    // STEP 5: Intelligent text mapping - Place text at exact positions
    fn map_text_objects_to_regions(
        &self,
        mut matrix: Vec<Vec<char>>,
        text_objects: &[PreciseTextObject],
        text_regions: &[TextRegion],
        char_width: f32,
        char_height: f32,
    ) -> Result<CharacterMatrix> {
        let mut updated_regions = Vec::new();

        // Convert text objects to simple string list for compatibility
        let original_text: Vec<String> = text_objects.iter().map(|obj| obj.text.clone()).collect();

        // NEW: Place each text object at its exact position in the matrix
        for text_obj in text_objects {
            // Calculate character position from PDF coordinates
            let char_x = (text_obj.bbox.x0 / char_width) as usize;
            let char_y = (text_obj.bbox.y0 / char_height) as usize;

            // Place each character of the text at the correct position
            for (i, ch) in text_obj.text.chars().enumerate() {
                let x = char_x + i;
                if char_y < matrix.len() && x < matrix[char_y].len() {
                    matrix[char_y][x] = ch;
                }
            }
        }

        // Update regions with their actual text content
        for region in text_regions {
            let mut region_text = String::new();

            // Extract text from the matrix for this region
            for y in region.bbox.y..(region.bbox.y + region.bbox.height).min(matrix.len()) {
                for x in region.bbox.x..(region.bbox.x + region.bbox.width).min(matrix[y].len()) {
                    let ch = matrix[y][x];
                    if ch != ' ' && ch != '█' {
                        region_text.push(ch);
                    }
                }
                // Add space between lines
                if !region_text.is_empty() && !region_text.ends_with(' ') {
                    region_text.push(' ');
                }
            }

            let mut updated_region = region.clone();
            updated_region.text_content = region_text.trim().to_string();
            updated_regions.push(updated_region);
        }

        Ok(CharacterMatrix {
            width: matrix[0].len(),
            height: matrix.len(),
            matrix,
            text_regions: updated_regions,
            original_text,
            char_width,
            char_height,
        })
    }

    #[allow(dead_code)] // Method for future text placement features
    fn place_text_in_region(&self, matrix: &mut [Vec<char>], bbox: &CharBBox, text: &str) {
        let text_chars: Vec<char> = text.chars().collect();
        let mut char_index = 0;

        // Fill the region with text, wrapping as needed
        for row in bbox.y..(bbox.y + bbox.height).min(matrix.len()) {
            for col in bbox.x..(bbox.x + bbox.width).min(matrix[row].len()) {
                if char_index < text_chars.len() {
                    let ch = text_chars[char_index];
                    matrix[row][col] = ch;
                    char_index += 1;
                } else {
                    break;
                }
            }

            if char_index >= text_chars.len() {
                break;
            }
        }
    }

    /// Process a PDF file and convert it to a character matrix representation.
    ///
    /// This is the main entry point for PDF processing. It performs the complete
    /// workflow from PDF analysis to character matrix creation.
    ///
    /// # Arguments
    /// * `pdf_path` - Path to the PDF file to process
    ///
    /// # Returns
    /// * `Ok(CharacterMatrix)` - Successfully processed character matrix
    /// * `Err(anyhow::Error)` - Processing error (file not found, invalid PDF, etc.)
    ///
    /// # Example
    /// ```
    /// let engine = CharacterMatrixEngine::new();
    /// let matrix = engine.process_pdf(&PathBuf::from("document.pdf"))?;
    /// println!("Matrix size: {}x{}", matrix.width, matrix.height);
    /// ```
    pub fn process_pdf(&self, pdf_path: &PathBuf) -> Result<CharacterMatrix> {
        // Step 1: Extract precise text objects with coordinates
        let text_objects = self.extract_text_objects_with_precise_coords(pdf_path)?;

        // Step 2: Calculate optimal matrix size based on actual content
        let (matrix_width, matrix_height, char_width, char_height) =
            self.calculate_optimal_matrix_size(&text_objects);

        // Step 3: Generate optimized character matrix
        let matrix = self.generate_optimal_character_matrix(
            &text_objects,
            matrix_width,
            matrix_height,
            char_width,
            char_height,
        )?;

        // Step 4: Use fallback vision processing
        let text_regions = self.detect_text_regions_fallback(&matrix)?;

        // Step 5: Map text objects to regions intelligently
        let final_matrix = self.map_text_objects_to_regions(
            matrix,
            &text_objects,
            &text_regions,
            char_width,
            char_height,
        )?;

        Ok(final_matrix)
    }

    pub fn process_pdf_with_ferrules(
        &self,
        pdf_path: &PathBuf,
        ferrules_path: &PathBuf,
    ) -> Result<CharacterMatrix> {
        // Enhanced processing with ferrules vision model
        let text_objects = self.extract_text_objects_with_precise_coords(pdf_path)?;
        let (matrix_width, matrix_height, char_width, char_height) =
            self.calculate_optimal_matrix_size(&text_objects);

        let matrix = self.generate_optimal_character_matrix(
            &text_objects,
            matrix_width,
            matrix_height,
            char_width,
            char_height,
        )?;

        // Use ferrules for vision processing
        let text_regions = self.run_ferrules_on_matrix(&matrix, ferrules_path)?;

        let final_matrix = self.map_text_objects_to_regions(
            matrix,
            &text_objects,
            &text_regions,
            char_width,
            char_height,
        )?;

        Ok(final_matrix)
    }

    pub fn render_matrix_as_string(&self, char_matrix: &CharacterMatrix) -> String {
        let mut result = String::new();

        result.push_str(&format!(
            "Character Matrix ({}x{}) | Char: {:.1}x{:.1}pt:\n",
            char_matrix.width, char_matrix.height, char_matrix.char_width, char_matrix.char_height
        ));
        result.push_str(&format!(
            "Text Regions: {} | Original Text Objects: {}\n",
            char_matrix.text_regions.len(),
            char_matrix.original_text.len()
        ));
        result.push_str(&"═".repeat(char_matrix.width.min(80)));
        result.push('\n');

        for (row_idx, row) in char_matrix.matrix.iter().enumerate() {
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
                "Region {}: ({},{}) {}x{} conf:{:.2} - \"{}\"\n",
                i + 1,
                region.bbox.x,
                region.bbox.y,
                region.bbox.width,
                region.bbox.height,
                region.confidence,
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

// ============= END CHARACTER MATRIX ENGINE =============

#[derive(Default)]
struct ExtractionResult {
    content: String,
    character_matrix: Option<CharacterMatrix>,
    is_loading: bool,
    error: Option<String>,
    // NEW: Editable character matrix
    editable_matrix: Option<Vec<Vec<char>>>,
    matrix_dirty: bool,
    original_matrix: Option<Vec<Vec<char>>>, // Keep track of original for comparison
}

// Using spatial-semantic engine types instead of old ferrules types

#[derive(Clone)]
#[allow(dead_code)] // Struct for future bounding box features
struct BoundingBox {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    label: String,
    confidence: f32,
    color: Color32,
}

struct Chonker5App {
    // PDF state
    pdf_path: Option<PathBuf>,
    current_page: usize,
    total_pages: usize,
    zoom_level: f32,
    pdf_texture: Option<egui::TextureHandle>,
    needs_render: bool,

    // UI assets
    hamster_texture: Option<egui::TextureHandle>,

    // Extraction state
    page_range: String,
    matrix_result: ExtractionResult,
    active_tab: ExtractionTab,

    // Character matrix engine
    matrix_engine: CharacterMatrixEngine,

    // Ferrules binary path
    ferrules_binary: Option<PathBuf>,

    // Async runtime
    runtime: Arc<tokio::runtime::Runtime>,

    // Channel for async results
    vision_receiver: Option<mpsc::Receiver<Result<CharacterMatrix, String>>>,

    // Log messages
    log_messages: Vec<String>,

    // UI state
    show_bounding_boxes: bool,
    #[allow(dead_code)] // Field for future multi-page support
    selected_page: usize,
    split_ratio: f32,

    // NEW: Editing state
    selected_cell: Option<(usize, usize)>, // (x, y) of selected character

    // NEW: Dark mode for PDF
    pdf_dark_mode: bool,

    // NEW: Focus state for panes
    focused_pane: FocusedPane,

    // NEW: Selection state for drag selection
    selection_start: Option<(usize, usize)>, // Start of drag selection
    selection_end: Option<(usize, usize)>,   // End of drag selection
    is_dragging: bool,                       // Currently dragging
    clipboard: String,                       // Internal clipboard for copy/paste
}

#[derive(PartialEq, Clone)]
enum ExtractionTab {
    Pdf,
    Matrix,
    Debug,
}

#[derive(PartialEq, Clone, Copy)]
enum FocusedPane {
    PdfView,
    MatrixView,
}

#[derive(Clone, Copy)]
enum DragAction {
    StartDrag(usize, usize),
    UpdateDrag(usize, usize),
    EndDrag,
    SingleClick(usize, usize),
    None,
}

impl Chonker5App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Initialize async runtime
        let runtime =
            Arc::new(tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime"));

        // Initialize tracing
        tracing_subscriber::fmt::init();

        // Load hamster image
        let hamster_texture = if let Ok(image_data) = std::fs::read("./assets/emojis/chonker.png") {
            if let Ok(image) = image::load_from_memory(&image_data) {
                let size = [image.width() as _, image.height() as _];
                let image_buffer = image.to_rgba8();
                let pixels = image_buffer.as_flat_samples();
                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
                Some(
                    cc.egui_ctx
                        .load_texture("hamster", color_image, Default::default()),
                )
            } else {
                None
            }
        } else {
            None
        };

        let mut app = Self {
            pdf_path: None,
            current_page: 0,
            total_pages: 0,
            zoom_level: 1.0,
            pdf_texture: None,
            needs_render: false,
            hamster_texture,
            page_range: "1-10".to_string(),
            matrix_result: Default::default(),
            active_tab: ExtractionTab::Pdf,
            ferrules_binary: None,
            runtime,
            vision_receiver: None,
            log_messages: vec![
                "🐹 CHONKER 5 Ready!".to_string(),
                "📌 Character Matrix Engine: PDF → Char Matrix → Vision Boxes → Text Mapping"
                    .to_string(),
                "📌 Faithful character representation: smallest matrix + vision + pdfium text"
                    .to_string(),
            ],
            show_bounding_boxes: true,
            selected_page: 0,
            split_ratio: 0.7,
            matrix_engine: CharacterMatrixEngine::new(),
            selected_cell: None,
            pdf_dark_mode: false,
            focused_pane: FocusedPane::PdfView,
            selection_start: None,
            selection_end: None,
            is_dragging: false,
            clipboard: String::new(),
        };

        // Initialize ferrules binary path
        app.init_ferrules_binary();

        app
    }

    fn init_ferrules_binary(&mut self) {
        self.log("🔄 Looking for Ferrules binary...");

        // Check common locations for ferrules binary
        let possible_paths = vec![
            PathBuf::from("./ferrules/target/release/ferrules"),
            PathBuf::from("./ferrules/target/debug/ferrules"),
            PathBuf::from("./ferrules"),
            PathBuf::from("/usr/local/bin/ferrules"),
        ];

        for path in &possible_paths {
            if path.exists() {
                self.ferrules_binary = Some(path.clone());
                self.log(&format!("✅ Found Ferrules binary at: {}", path.display()));
                return;
            }
        }

        // Try to find it in PATH
        if let Ok(output) = Command::new("which").arg("ferrules").output() {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                self.ferrules_binary = Some(PathBuf::from(path.clone()));
                self.log(&format!("✅ Found Ferrules binary in PATH: {}", path));
                return;
            }
        }

        self.log("⚠️ Ferrules binary not found. Vision extraction will use fallback.");
    }

    fn log(&mut self, message: &str) {
        self.log_messages.push(message.to_string());
        if self.log_messages.len() > 100 {
            self.log_messages.remove(0);
        }
    }

    fn open_file(&mut self, ctx: &egui::Context) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("PDF files", &["pdf"])
            .pick_file()
        {
            self.pdf_path = Some(path.clone());
            self.current_page = 0;
            self.pdf_texture = None;

            // Get PDF info
            match self.get_pdf_info(&path) {
                Ok(pages) => {
                    self.total_pages = pages;
                    self.log(&format!(
                        "📄 Loaded PDF: {} ({} pages)",
                        path.display(),
                        pages
                    ));

                    // Set default page range for large PDFs
                    if pages > 20 {
                        self.page_range = "1-10".to_string();
                        self.log("📄 Large PDF detected - Default page range set to 1-10");
                    } else {
                        self.page_range.clear();
                    }

                    // Render the first page
                    self.render_current_page(ctx);

                    // AUTOMATICALLY EXTRACT CHARACTER MATRIX - ONE STEP PROCESS
                    self.log("🚀 Auto-extracting character matrix...");
                    self.extract_character_matrix(ctx);
                    self.active_tab = ExtractionTab::Matrix;
                }
                Err(e) => {
                    self.log(&format!("❌ Failed to load PDF: {}", e));
                }
            }
        }
    }

    fn get_pdf_info(&self, path: &PathBuf) -> Result<usize> {
        let output = Command::new("mutool").arg("info").arg(path).output()?;

        let info = String::from_utf8_lossy(&output.stdout);
        for line in info.lines() {
            if line.contains("Pages:") {
                if let Some(pages_str) = line.split(':').nth(1) {
                    return pages_str
                        .trim()
                        .parse()
                        .map_err(|e| anyhow::anyhow!("Parse error: {}", e));
                }
            }
        }

        Err(anyhow::anyhow!("Could not determine page count"))
    }

    fn render_current_page(&mut self, ctx: &egui::Context) {
        if let Some(pdf_path) = &self.pdf_path {
            // Use mutool to render the current page to a PNG
            let temp_png =
                std::env::temp_dir().join(format!("chonker5_page_{}.png", self.current_page));

            let dpi = 150.0 * self.zoom_level;
            let result = Command::new("mutool")
                .arg("draw")
                .arg("-o")
                .arg(&temp_png)
                .arg("-r")
                .arg(dpi.to_string())
                .arg("-F")
                .arg("png")
                .arg(pdf_path)
                .arg(format!("{}", self.current_page + 1))
                .output();

            match result {
                Ok(output) => {
                    if output.status.success() {
                        // Load the PNG as a texture
                        if let Ok(image_data) = std::fs::read(&temp_png) {
                            if let Ok(mut image) = image::load_from_memory(&image_data) {
                                // Apply dark mode if enabled
                                if self.pdf_dark_mode {
                                    let mut rgba_image = image.to_rgba8();
                                    image::imageops::colorops::invert(&mut rgba_image);
                                    image = image::DynamicImage::ImageRgba8(rgba_image);
                                }

                                let size = [image.width() as _, image.height() as _];
                                let image_buffer = image.to_rgba8();
                                let pixels = image_buffer.as_flat_samples();

                                let image = egui::ColorImage::from_rgba_unmultiplied(
                                    size,
                                    pixels.as_slice(),
                                );

                                self.pdf_texture = Some(ctx.load_texture(
                                    format!("pdf_page_{}", self.current_page),
                                    image,
                                    Default::default(),
                                ));

                                self.log(&format!(
                                    "📄 Rendered page {} {}",
                                    self.current_page + 1,
                                    if self.pdf_dark_mode { "🌙" } else { "" }
                                ));
                            }
                        }

                        // Clean up temp file
                        let _ = std::fs::remove_file(&temp_png);
                    } else {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        self.log(&format!("❌ Failed to render page: {}", stderr));
                    }
                }
                Err(e) => {
                    self.log(&format!("❌ Failed to run mutool: {}", e));
                }
            }
        }
    }

    fn extract_character_matrix(&mut self, ctx: &egui::Context) {
        if self.pdf_path.is_none() {
            self.log("⚠️ No PDF loaded. Open a file first.");
            return;
        }

        let pdf_path = match &self.pdf_path {
            Some(path) => path.clone(),
            None => {
                self.log("❌ No PDF file selected");
                return;
            }
        };
        let runtime = self.runtime.clone();
        let ctx = ctx.clone();

        self.matrix_result.is_loading = true;
        self.matrix_result.error = None;
        self.log("🔄 Creating enhanced character matrix representation...");
        self.log("📝 Step 1: Pdfium → Extract precise text objects with coordinates");
        self.log("📐 Step 2: Calculate optimal matrix size from modal font size");
        self.log("🗂️ Step 3: Generate smallest viable character matrix");
        if self.ferrules_binary.is_some() {
            self.log("👁️ Step 4: Ferrules vision → Text region detection");
        } else {
            self.log("👁️ Step 4: Fallback vision → Text region detection");
        }
        self.log("🗺️ Step 5: Intelligent spatial text mapping");

        // Create channel for results
        let (tx, rx) = mpsc::channel(1);
        self.vision_receiver = Some(rx);

        // Check for ferrules binary
        let ferrules_binary = self.ferrules_binary.clone();

        // Spawn async task
        runtime.spawn(async move {
            let result = async {
                let engine = CharacterMatrixEngine::new();

                // Process PDF with enhanced approach
                let character_matrix = if let Some(ferrules_path) = &ferrules_binary {
                    // Use ferrules if available
                    engine
                        .process_pdf_with_ferrules(&pdf_path, ferrules_path)
                        .map_err(|e| format!("Ferrules processing failed: {}", e))?
                } else {
                    // Fallback to enhanced processing without ferrules
                    engine
                        .process_pdf(&pdf_path)
                        .map_err(|e| format!("Character matrix processing failed: {}", e))?
                };

                Ok::<_, String>(character_matrix)
            }
            .await;

            // Send result through channel
            let _ = tx.send(result).await;

            // Update UI on main thread
            ctx.request_repaint();
        });
    }

    // Check if a cell is within the selection rectangle
    #[allow(dead_code)] // Method for future cell selection features
    fn is_cell_selected(&self, x: usize, y: usize) -> bool {
        if let (Some(start), Some(end)) = (self.selection_start, self.selection_end) {
            let min_x = start.0.min(end.0);
            let max_x = start.0.max(end.0);
            let min_y = start.1.min(end.1);
            let max_y = start.1.max(end.1);

            x >= min_x && x <= max_x && y >= min_y && y <= max_y
        } else {
            false
        }
    }

    // Copy selected text to clipboard
    #[allow(dead_code)] // Method for future copy/paste features
    fn copy_selection(&mut self) {
        if let (Some(start), Some(end)) = (self.selection_start, self.selection_end) {
            if let Some(editable_matrix) = &self.matrix_result.editable_matrix {
                let min_x = start.0.min(end.0);
                let max_x = start.0.max(end.0);
                let min_y = start.1.min(end.1);
                let max_y = start.1.max(end.1);

                self.clipboard.clear();

                for y in min_y..=max_y {
                    if y < editable_matrix.len() {
                        for x in min_x..=max_x {
                            if x < editable_matrix[y].len() {
                                self.clipboard.push(editable_matrix[y][x]);
                            }
                        }
                        if y < max_y {
                            self.clipboard.push('\n');
                        }
                    }
                }

                self.log(&format!(
                    "📋 Copied {} characters to clipboard",
                    self.clipboard.len()
                ));
            }
        }
    }

    // Paste clipboard content at current position
    #[allow(dead_code)] // Method for future copy/paste features
    fn paste_at_position(&mut self, start_x: usize, start_y: usize) {
        if let Some(editable_matrix) = &mut self.matrix_result.editable_matrix {
            let lines: Vec<&str> = self.clipboard.lines().collect();

            for (line_idx, line) in lines.iter().enumerate() {
                let y = start_y + line_idx;
                if y >= editable_matrix.len() {
                    break;
                }

                for (char_idx, ch) in line.chars().enumerate() {
                    let x = start_x + char_idx;
                    if x < editable_matrix[y].len() {
                        editable_matrix[y][x] = ch;
                    }
                }
            }

            self.matrix_result.matrix_dirty = true;
            self.log(&format!(
                "📋 Pasted {} characters at ({}, {})",
                self.clipboard.len(),
                start_x,
                start_y
            ));
        }
    }

    fn save_edited_matrix(&mut self) {
        if let Some(editable_matrix) = &self.matrix_result.editable_matrix {
            if let Some(pdf_path) = &self.pdf_path {
                let output_path = pdf_path.with_extension("matrix.txt");

                let mut content = String::new();
                for row in editable_matrix {
                    for ch in row {
                        content.push(*ch);
                    }
                    content.push('\n');
                }

                match std::fs::write(&output_path, content) {
                    Ok(_) => {
                        self.log(&format!(
                            "✅ Saved edited matrix to: {}",
                            output_path.display()
                        ));
                        self.matrix_result.matrix_dirty = false;
                    }
                    Err(e) => {
                        self.log(&format!("❌ Failed to save matrix: {}", e));
                    }
                }
            }
        }
    }

    fn show_debug_info(&mut self) {
        if let Some(char_matrix) = &self.matrix_result.character_matrix {
            let mut debug_content = String::new();
            debug_content.push_str("╔═══ DEBUG: ENHANCED CHARACTER MATRIX ANALYSIS ═══╗\n\n");
            debug_content.push_str(&format!(
                "Matrix Dimensions: {}x{}\n",
                char_matrix.width, char_matrix.height
            ));
            debug_content.push_str(&format!(
                "Character Size: {:.1}x{:.1}pt\n",
                char_matrix.char_width, char_matrix.char_height
            ));
            debug_content.push_str(&format!(
                "Text Regions Found: {}\n",
                char_matrix.text_regions.len()
            ));
            debug_content.push_str(&format!(
                "Original Text Objects: {}\n\n",
                char_matrix.original_text.len()
            ));

            debug_content.push_str("TEXT REGIONS:\n");
            for (i, region) in char_matrix.text_regions.iter().enumerate() {
                debug_content.push_str(&format!(
                    "Region {}: ({},{}) {}x{}\n",
                    i + 1,
                    region.bbox.x,
                    region.bbox.y,
                    region.bbox.width,
                    region.bbox.height
                ));
                debug_content.push_str(&format!("  Confidence: {:.2}\n", region.confidence));
                debug_content.push_str(&format!(
                    "  Content: {}\n",
                    region.text_content.chars().take(100).collect::<String>()
                ));
                debug_content.push('\n');
            }

            debug_content.push_str("ORIGINAL TEXT SAMPLE:\n");
            for (_i, text_chunk) in char_matrix.original_text.iter().take(20).enumerate() {
                debug_content.push_str(&format!("{}: {}\n", _i + 1, text_chunk));
            }

            if char_matrix.original_text.len() > 20 {
                debug_content.push_str(&format!(
                    "... and {} more text chunks\n",
                    char_matrix.original_text.len() - 20
                ));
            }

            self.matrix_result.content = debug_content;
        } else {
            self.matrix_result.content =
                "No debug data available. Extract character matrix first.".to_string();
        }
    }

    #[allow(dead_code)] // Alternative text extraction method
    fn pdfium_text_extraction(
        &self,
        pdf_path: &std::path::Path,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut result = String::new();
        result.push_str("╔═══ DATA EXTRACTION RESULTS ═══╗\n\n");

        // Initialize pdfium
        let pdfium = pdfium_render::prelude::Pdfium::new(
            pdfium_render::prelude::Pdfium::bind_to_library("./lib/libpdfium.dylib")
                .or_else(|_| {
                    pdfium_render::prelude::Pdfium::bind_to_library(
                        "/usr/local/lib/libpdfium.dylib",
                    )
                })
                .or_else(|_| pdfium_render::prelude::Pdfium::bind_to_system_library())
                .map_err(|e| format!("Failed to bind to pdfium library: {}", e))?,
        );

        // Load the PDF
        let document = pdfium
            .load_pdf_from_file(pdf_path, None)
            .map_err(|e| format!("Failed to load PDF: {}", e))?;

        result.push_str(&format!(
            "Document: {}\n",
            pdf_path.file_name().unwrap_or_default().to_string_lossy()
        ));
        result.push_str(&format!("Total Pages: {}\n\n", document.pages().len()));

        // Process each page
        for (page_index, page) in document.pages().iter().enumerate() {
            let page_number = page_index + 1;
            result.push_str(&format!("╔═══ PAGE {} ═══╗\n", page_number));

            // Extract all text
            let text_page = page
                .text()
                .map_err(|e| format!("Failed to get text: {}", e))?;
            let page_text = text_page.all();

            if !page_text.trim().is_empty() {
                result.push_str(&page_text);
                result.push('\n');
            } else {
                result.push_str("║ [No text content detected] ║\n");
            }

            result.push_str("╚═════════════════════════════╝\n\n");
        }

        Ok(result)
    }

    // Character matrix elements overlay - shows text regions from the matrix
    fn draw_character_matrix_overlay(&self, ui: &mut egui::Ui, image_response: &egui::Response) {
        if let Some(char_matrix) = &self.matrix_result.character_matrix {
            let painter = ui.painter();
            let image_rect = image_response.rect;

            // Map character coordinates to screen coordinates
            let char_width = image_rect.width() / char_matrix.width as f32;
            let char_height = image_rect.height() / char_matrix.height as f32;

            // Draw subtle grid lines to show character positions
            let grid_color = TERM_DIM.gamma_multiply(0.2); // Very faint grid

            // Vertical lines every 10 characters
            for x in (0..char_matrix.width).step_by(10) {
                let screen_x = image_rect.left() + (x as f32 * char_width);
                painter.line_segment(
                    [
                        egui::pos2(screen_x, image_rect.top()),
                        egui::pos2(screen_x, image_rect.bottom()),
                    ],
                    egui::Stroke::new(0.5, grid_color),
                );
            }

            // Horizontal lines every 10 characters
            for y in (0..char_matrix.height).step_by(10) {
                let screen_y = image_rect.top() + (y as f32 * char_height);
                painter.line_segment(
                    [
                        egui::pos2(image_rect.left(), screen_y),
                        egui::pos2(image_rect.right(), screen_y),
                    ],
                    egui::Stroke::new(0.5, grid_color),
                );
            }

            // Highlight the selected cell if any
            if let Some((sel_x, sel_y)) = self.selected_cell {
                if sel_y < char_matrix.height && sel_x < char_matrix.width {
                    let x1 = image_rect.left() + (sel_x as f32 * char_width);
                    let y1 = image_rect.top() + (sel_y as f32 * char_height);
                    let cell_rect = egui::Rect::from_min_size(
                        egui::pos2(x1, y1),
                        egui::vec2(char_width, char_height),
                    );
                    painter.rect_filled(cell_rect, 0.0, TERM_HIGHLIGHT.gamma_multiply(0.2));
                    painter.rect_stroke(cell_rect, 0.0, egui::Stroke::new(2.0, TERM_HIGHLIGHT));
                }
            }

            // Draw text regions from character matrix
            for region in char_matrix.text_regions.iter() {
                let x1 = image_rect.left() + (region.bbox.x as f32 * char_width);
                let y1 = image_rect.top() + (region.bbox.y as f32 * char_height);
                let x2 = x1 + (region.bbox.width as f32 * char_width);
                let y2 = y1 + (region.bbox.height as f32 * char_height);

                let rect = egui::Rect::from_min_max(egui::pos2(x1, y1), egui::pos2(x2, y2));

                // Color based on confidence
                let color = if region.confidence > 0.8 {
                    TERM_HIGHLIGHT // High confidence - bright teal
                } else if region.confidence > 0.5 {
                    TERM_YELLOW // Medium confidence - yellow
                } else {
                    TERM_DIM // Low confidence - dim
                };

                // Draw bounding box
                painter.rect_stroke(rect, 0.0, egui::Stroke::new(2.0, color));

                // Draw region ID if there's space
                if rect.width() > 20.0 && rect.height() > 15.0 {
                    let label_pos = rect.min + egui::vec2(2.0, 2.0);
                    painter.text(
                        label_pos,
                        egui::Align2::LEFT_TOP,
                        format!("R{}", region.region_id + 1),
                        FontId::monospace(10.0),
                        color,
                    );
                }
            }
        }
    }
}

// Helper to draw terminal-style box with chrome borders
fn draw_terminal_box(
    ui: &mut egui::Ui,
    title: &str,
    is_focused: bool,
    add_contents: impl FnOnce(&mut egui::Ui),
) {
    let stroke_color = if is_focused { TERM_HIGHLIGHT } else { CHROME };
    let stroke_width = if is_focused { 2.0 } else { 1.0 };

    let frame = egui::Frame::none()
        .fill(TERM_BG)
        .stroke(Stroke::new(stroke_width, stroke_color))
        .inner_margin(egui::Margin::same(5.0))
        .outer_margin(egui::Margin::same(1.0))
        .rounding(Rounding::same(2.0));

    frame.show(ui, |ui| {
        // Draw title with chrome accent (or highlight if focused)
        ui.horizontal(|ui| {
            ui.label(RichText::new("▸").color(TERM_HIGHLIGHT).monospace());
            ui.label(
                RichText::new(title)
                    .color(if is_focused { TERM_HIGHLIGHT } else { CHROME })
                    .monospace()
                    .strong(),
            );
            if is_focused {
                ui.label(
                    RichText::new(" [ACTIVE]")
                        .color(TERM_HIGHLIGHT)
                        .monospace()
                        .size(10.0),
                );
            }
        });

        ui.add_space(5.0);
        add_contents(ui);
    });
}

impl eframe::App for Chonker5App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle deferred rendering
        if self.needs_render {
            self.needs_render = false;
            self.render_current_page(ctx);
        }

        // Set up terminal style
        let mut style = (*ctx.style()).clone();
        style.visuals.dark_mode = true;
        style.visuals.override_text_color = Some(TERM_FG);
        style.visuals.window_fill = TERM_BG;
        style.visuals.panel_fill = TERM_BG;
        style.visuals.extreme_bg_color = TERM_BG;
        style.visuals.widgets.noninteractive.bg_fill = TERM_BG;
        style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TERM_FG);
        style.visuals.widgets.inactive.bg_fill = TERM_BG;
        style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, CHROME);
        style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(30, 40, 50);
        style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, TERM_HIGHLIGHT);
        style.visuals.widgets.active.bg_fill = Color32::from_rgb(40, 50, 60);
        style.visuals.widgets.active.bg_stroke = Stroke::new(1.0, TERM_HIGHLIGHT);
        style.visuals.widgets.noninteractive.weak_bg_fill = TERM_BG;
        style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(20, 25, 30);
        style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, TERM_DIM);
        style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(30, 40, 45);
        style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, TERM_HIGHLIGHT);
        style.visuals.widgets.active.bg_fill = Color32::from_rgb(40, 50, 55);
        style.visuals.widgets.active.fg_stroke = Stroke::new(1.0, TERM_YELLOW);
        style.visuals.selection.bg_fill = Color32::from_rgb(0, 150, 140);
        style.visuals.selection.stroke = Stroke::new(1.0, TERM_HIGHLIGHT);
        ctx.set_style(style);

        // Handle global keyboard shortcuts for focus switching
        ctx.input(|i| {
            for event in &i.events {
                if let egui::Event::Key {
                    key: egui::Key::Tab,
                    pressed: true,
                    modifiers,
                    ..
                } = event
                {
                    if modifiers.shift {
                        // Shift+Tab - switch focus backwards
                        self.focused_pane = match self.focused_pane {
                            FocusedPane::PdfView => FocusedPane::MatrixView,
                            FocusedPane::MatrixView => FocusedPane::PdfView,
                        };
                    } else {
                        // Tab - switch focus forward
                        self.focused_pane = match self.focused_pane {
                            FocusedPane::PdfView => FocusedPane::MatrixView,
                            FocusedPane::MatrixView => FocusedPane::PdfView,
                        };
                    }
                }
            }
        });

        // Check for async results
        if let Some(mut receiver) = self.vision_receiver.take() {
            if let Ok(result) = receiver.try_recv() {
                match result {
                    Ok(character_matrix) => {
                        self.matrix_result.content = self
                            .matrix_engine
                            .render_matrix_as_string(&character_matrix);
                        // Create editable copy of the matrix
                        self.matrix_result.editable_matrix = Some(character_matrix.matrix.clone());
                        self.matrix_result.original_matrix = Some(character_matrix.matrix.clone());
                        self.matrix_result.character_matrix = Some(character_matrix);
                        self.matrix_result.is_loading = false;
                        self.matrix_result.matrix_dirty = false;
                        self.log("✅ Enhanced character matrix extraction completed");
                        self.log("🎯 Text positioned at exact PDF coordinates");
                        self.log("✏️ Matrix is now editable - click to modify text");
                    }
                    Err(e) => {
                        self.matrix_result.error = Some(e);
                        self.matrix_result.is_loading = false;
                    }
                }
            } else {
                // Put the receiver back if no message yet
                self.vision_receiver = Some(receiver);
            }
        }

        // Main panel with terminal background
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(TERM_BG))
            .show(ctx, |ui| {
                // Compact header and controls
                ui.horizontal(|ui| {
                    // Display hamster image if available, otherwise emoji
                    if let Some(hamster) = &self.hamster_texture {
                        ui.image(egui::load::SizedTexture::new(hamster.id(), egui::vec2(32.0, 32.0)));
                    } else {
                        ui.label(
                            RichText::new("🐹")
                                .size(24.0)
                        );
                    }

                    ui.label(
                        RichText::new("CHONKER 5")
                            .color(TERM_HIGHLIGHT)
                            .monospace()
                            .size(16.0)
                            .strong()
                    );

                    ui.label(RichText::new("│").color(CHROME).monospace());

                    if ui.button(RichText::new("[O] Open").color(TERM_FG).monospace().size(12.0)).clicked() {
                        self.open_file(ctx);
                    }

                    ui.label(RichText::new("│").color(CHROME).monospace());

                    // Navigation
                    ui.add_enabled_ui(self.pdf_path.is_some() && self.current_page > 0, |ui| {
                        if ui.button(RichText::new("←").color(TERM_FG).monospace().size(12.0)).clicked() {
                            self.current_page = self.current_page.saturating_sub(1);
                            self.render_current_page(ctx);
                        }
                    });

                    if self.pdf_path.is_some() {
                        ui.label(RichText::new(format!("{}/{}", self.current_page + 1, self.total_pages))
                            .color(TERM_FG)
                            .monospace()
                            .size(12.0));
                    }

                    ui.add_enabled_ui(self.pdf_path.is_some() && self.current_page < self.total_pages - 1, |ui| {
                        if ui.button(RichText::new("→").color(TERM_FG).monospace().size(12.0)).clicked() {
                            self.current_page += 1;
                            self.render_current_page(ctx);
                        }
                    });

                    ui.label(RichText::new("│").color(CHROME).monospace());

                    // Zoom controls
                    ui.add_enabled_ui(self.pdf_path.is_some(), |ui| {
                        if ui.button(RichText::new("-").color(TERM_FG).monospace().size(12.0)).clicked() {
                            self.zoom_level = (self.zoom_level - 0.25).max(0.5);
                            self.render_current_page(ctx);
                        }

                        ui.label(RichText::new(format!("{}%", (self.zoom_level * 100.0) as i32))
                            .color(TERM_FG)
                            .monospace()
                            .size(12.0));

                        if ui.button(RichText::new("+").color(TERM_FG).monospace().size(12.0)).clicked() {
                            self.zoom_level = (self.zoom_level + 0.25).min(3.0);
                            self.render_current_page(ctx);
                        }
                    });

                    ui.label(RichText::new("│").color(CHROME).monospace());

                    // Page range
                    ui.label(RichText::new("R:").color(TERM_FG).monospace().size(12.0));
                    ui.add(egui::TextEdit::singleline(&mut self.page_range)
                        .desired_width(50.0)
                        .font(FontId::monospace(12.0)));

                    ui.label(RichText::new("│").color(CHROME).monospace());

                    // Extraction buttons
                    ui.add_enabled_ui(self.pdf_path.is_some(), |ui| {
                        if ui.button(RichText::new("[M]").color(TERM_FG).monospace().size(12.0)).clicked() {
                            self.extract_character_matrix(ctx);
                            self.active_tab = ExtractionTab::Matrix;
                        }

                        if ui.button(RichText::new("[G]").color(TERM_FG).monospace().size(12.0)).clicked() {
                            self.show_debug_info();
                            self.active_tab = ExtractionTab::Debug;
                        }

                        ui.label(RichText::new("│").color(CHROME).monospace());

                        // Spatial elements toggle
                        let bbox_text = if self.show_bounding_boxes { "[B]✓" } else { "[B]" };
                        if ui.button(RichText::new(bbox_text).color(TERM_FG).monospace().size(12.0)).clicked() {
                            self.show_bounding_boxes = !self.show_bounding_boxes;
                        }

                        // Dark mode toggle for PDF
                        ui.label(RichText::new("│").color(CHROME).monospace());
                        let dark_text = if self.pdf_dark_mode { "[D]✓" } else { "[D]" };
                        if ui.button(RichText::new(dark_text).color(TERM_FG).monospace().size(12.0))
                            .on_hover_text("Toggle dark mode for PDF")
                            .clicked() {
                            self.pdf_dark_mode = !self.pdf_dark_mode;
                            self.render_current_page(ctx);
                        }

                        // Save edited matrix
                        if self.matrix_result.matrix_dirty {
                            ui.label(RichText::new("│").color(CHROME).monospace());
                            if ui.button(RichText::new("[S] Save").color(TERM_YELLOW).monospace().size(12.0)).clicked() {
                                self.save_edited_matrix();
                            }
                        }
                    });
                });

                ui.add_space(2.0);

                // Main content area - Split pane view
                if self.pdf_path.is_some() {
                    let available_size = ui.available_size();
                    let available_width = available_size.x;
                    let available_height = available_size.y;
                    let separator_width = 8.0;
                    let padding = 20.0; // Increased padding to ensure right border is visible
                    let usable_width = available_width - (padding * 2.0);
                    let left_width = (usable_width - separator_width) * self.split_ratio;
                    let right_width = (usable_width - separator_width) * (1.0 - self.split_ratio);

                    ui.horizontal_top(|ui| {
                        ui.add_space(padding); // Add left padding
                        // Left pane - PDF View
                        ui.allocate_ui_with_layout(
                            egui::vec2(left_width, available_height),
                            egui::Layout::left_to_right(egui::Align::TOP),
                            |ui| {
                                draw_terminal_box(ui, "PDF VIEW", self.focused_pane == FocusedPane::PdfView, |ui| {
                                    egui::ScrollArea::both()
                                        .auto_shrink([false; 2])
                                        .show(ui, |ui| {
                                            // Detect interaction with PDF view
                                            if ui.ui_contains_pointer() && ui.input(|i| i.pointer.any_click()) {
                                                self.focused_pane = FocusedPane::PdfView;
                                            }
                                            if let Some(texture) = &self.pdf_texture {
                                                let size = texture.size_vec2();
                                                let available_size = ui.available_size();

                                                // Calculate scaling to fit with zoom
                                                let base_scale = (available_size.x / size.x).min(available_size.y / size.y).min(1.0);
                                                let scale = base_scale * self.zoom_level;
                                                let display_size = size * scale;

                                                // Extract values needed for the closure
                                                let texture_id = texture.id();
                                                let current_zoom = self.zoom_level;
                                                let current_page = self.current_page;
                                                let total_pages = self.total_pages;

                                                // Center the image with trackpad support
                                                ui.vertical_centered(|ui| {
                                                    let response = ui.image(egui::load::SizedTexture::new(texture_id, display_size));

                                                    // Draw character matrix overlay if enabled
                                                    if self.show_bounding_boxes {
                                                        self.draw_character_matrix_overlay(ui, &response);
                                                    }

                                                    // Handle trackpad zoom (pinch gesture)
                                                    if response.hovered() {
                                                        let zoom_delta = ui.input(|i| i.zoom_delta());
                                                        if zoom_delta != 1.0 {
                                                            self.zoom_level = (current_zoom * zoom_delta).clamp(0.5, 3.0);
                                                            self.needs_render = true;
                                                        }

                                                        // Handle scroll for navigation
                                                        let scroll_delta = ui.input(|i| i.scroll_delta);
                                                        if scroll_delta.y.abs() > 10.0 {
                                                            if scroll_delta.y > 0.0 && current_page > 0 {
                                                                self.current_page = current_page - 1;
                                                                self.needs_render = true;
                                                            } else if scroll_delta.y < 0.0 && current_page < total_pages - 1 {
                                                                self.current_page = current_page + 1;
                                                                self.needs_render = true;
                                                            }
                                                        }
                                                    }
                                                });
                                            } else {
                                                ui.centered_and_justified(|ui| {
                                                    ui.label(RichText::new("Loading page...")
                                                        .color(TERM_DIM)
                                                        .monospace());
                                                });
                                            }
                                        });
                                });
                            }
                        );

                        // Draggable separator with visual indicator
                        let separator_rect = ui.available_rect_before_wrap();
                        let separator_rect = egui::Rect::from_min_size(
                            separator_rect.min,
                            egui::vec2(separator_width, available_height)
                        );
                        let separator_response = ui.allocate_rect(separator_rect, egui::Sense::drag());

                        // Visual feedback
                        let separator_color = if separator_response.hovered() {
                            TERM_HIGHLIGHT
                        } else {
                            CHROME
                        };
                        ui.painter().rect_filled(separator_response.rect, 0.0, separator_color);

                        // Draw grip dots
                        let center = separator_response.rect.center();
                        for i in -2..=2 {
                            ui.painter().circle_filled(
                                egui::pos2(center.x, center.y + i as f32 * 10.0),
                                1.5,
                                TERM_DIM
                            );
                        }

                        // Change cursor on hover
                        if separator_response.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
                        }

                        if separator_response.dragged() {
                            let delta = separator_response.drag_delta().x;
                            self.split_ratio = (self.split_ratio + delta / available_width).clamp(0.2, 0.8);
                        }

                        // Right pane - Extraction Results
                        ui.allocate_ui_with_layout(
                            egui::vec2(right_width, available_height),
                            egui::Layout::top_down(egui::Align::LEFT),
                            |ui| {
                                draw_terminal_box(ui, "EXTRACTION RESULTS", self.focused_pane == FocusedPane::MatrixView, |ui| {
                                    // Detect interaction with this pane
                                    if ui.ui_contains_pointer() && ui.input(|i| i.pointer.any_click()) {
                                        self.focused_pane = FocusedPane::MatrixView;
                                    }

                                    // Tab buttons
                                    ui.horizontal(|ui| {
                                        let matrix_label = if self.active_tab == ExtractionTab::Matrix {
                                            RichText::new("[MATRIX]").color(TERM_HIGHLIGHT).monospace()
                                        } else {
                                            RichText::new(" Matrix ").color(TERM_DIM).monospace()
                                        };
                                        if ui.button(matrix_label).clicked() {
                                            self.active_tab = ExtractionTab::Matrix;
                                        }

                                        ui.label(RichText::new("│").color(CHROME).monospace());

                                        let debug_label = if self.active_tab == ExtractionTab::Debug {
                                            RichText::new("[DEBUG]").color(TERM_HIGHLIGHT).monospace()
                                        } else {
                                            RichText::new(" Debug ").color(TERM_DIM).monospace()
                                        };
                                        if ui.button(debug_label).clicked() {
                                            self.active_tab = ExtractionTab::Debug;
                                        }
                                    });

                                    ui.separator();

                                    // Content area
                                    egui::ScrollArea::both()
                                        .auto_shrink([false; 2])
                                        .show(ui, |ui| {
                                            match self.active_tab {
                                                ExtractionTab::Matrix => {
                                                    if self.matrix_result.is_loading {
                                                        ui.centered_and_justified(|ui| {
                                                            ui.spinner();
                                                            ui.label(RichText::new("\nCreating character matrix...\n\n1. PDF → Character Matrix\n2. Vision → Text Regions\n3. Pdfium → Extract Text\n4. Map Text → Matrix")
                                                                .color(TERM_FG)
                                                                .monospace());
                                                        });
                                                    } else if let Some(error) = &self.matrix_result.error {
                                                        ui.label(RichText::new(error).color(TERM_ERROR).monospace());
                                                    } else if let Some(editable_matrix) = &mut self.matrix_result.editable_matrix {
                                                        // EDITABLE CHARACTER MATRIX VIEW
                                                        if let Some(char_matrix) = &self.matrix_result.character_matrix {
                                                            ui.horizontal(|ui| {
                                                                ui.label(RichText::new(format!("Matrix {}x{} | Char: {:.1}x{:.1}pt",
                                                                    char_matrix.width, char_matrix.height,
                                                                    char_matrix.char_width, char_matrix.char_height))
                                                                    .color(TERM_FG)
                                                                    .monospace()
                                                                    .size(10.0));

                                                                if self.matrix_result.matrix_dirty {
                                                                    ui.label(RichText::new(" [MODIFIED]").color(TERM_YELLOW).monospace().size(10.0));
                                                                }

                                                                // Show selected cell position
                                                                if let Some((x, y)) = self.selected_cell {
                                                                    ui.label(RichText::new(format!(" | Pos: ({},{})", x, y))
                                                                        .color(TERM_HIGHLIGHT)
                                                                        .monospace()
                                                                        .size(10.0));

                                                                    // Check if we're in a text region
                                                                    for region in &char_matrix.text_regions {
                                                                        if region.bbox.contains(x, y) {
                                                                            ui.label(RichText::new(format!(" | Region {}", region.region_id + 1))
                                                                                .color(TERM_HIGHLIGHT)
                                                                                .monospace()
                                                                                .size(10.0));
                                                                            break;
                                                                        }
                                                                    }
                                                                }

                                                                // Show selection size
                                                                if let (Some(start), Some(end)) = (self.selection_start, self.selection_end) {
                                                                    let width = (end.0 as i32 - start.0 as i32).abs() + 1;
                                                                    let height = (end.1 as i32 - start.1 as i32).abs() + 1;
                                                                    ui.label(RichText::new(format!(" | Selection: {}x{}", width, height))
                                                                        .color(TERM_YELLOW)
                                                                        .monospace()
                                                                        .size(10.0));
                                                                }

                                                                // Show clipboard status
                                                                if !self.clipboard.is_empty() {
                                                                    ui.label(RichText::new(format!(" | 📋 {} chars", self.clipboard.len()))
                                                                        .color(TERM_YELLOW)
                                                                        .monospace()
                                                                        .size(10.0));
                                                                }
                                                            });
                                                            ui.separator();
                                                        }

                                                        // Create a monospace text layout for the editable matrix
                                                        let font_id = FontId::monospace(12.0);
                                                        let char_size = ui.fonts(|f| f.glyph_width(&font_id, ' '));
                                                        let line_height = ui.text_style_height(&egui::TextStyle::Monospace);

                                                        // Handle keyboard input ONCE, outside the matrix rendering
                                                        // Only process keyboard input if matrix view has focus
                                                        let mut needs_copy = false;
                                                        let mut needs_paste_at = None;

                                                        if self.focused_pane == FocusedPane::MatrixView {
                                                            if let Some((sel_x, sel_y)) = self.selected_cell {
                                                                let matrix_height = editable_matrix.len();
                                                                if sel_y < matrix_height && sel_x < editable_matrix[sel_y].len() {
                                                                    ui.input(|i| {
                                                                    for event in &i.events {
                                                                        if let egui::Event::Text(text) = event {
                                                                            if let Some(new_char) = text.chars().next() {
                                                                                editable_matrix[sel_y][sel_x] = new_char;
                                                                                self.matrix_result.matrix_dirty = true;
                                                                                // Move to next cell
                                                                                if sel_x < editable_matrix[sel_y].len() - 1 {
                                                                                    self.selected_cell = Some((sel_x + 1, sel_y));
                                                                                }
                                                                            }
                                                                        } else if let egui::Event::Key { key, pressed: true, modifiers, .. } = event {
                                                                            match key {
                                                                                egui::Key::ArrowLeft if sel_x > 0 => {
                                                                                    self.selected_cell = Some((sel_x - 1, sel_y));
                                                                                }
                                                                                egui::Key::ArrowRight if sel_x < editable_matrix[sel_y].len() - 1 => {
                                                                                    self.selected_cell = Some((sel_x + 1, sel_y));
                                                                                }
                                                                                egui::Key::ArrowUp if sel_y > 0 => {
                                                                                    self.selected_cell = Some((sel_x, sel_y - 1));
                                                                                }
                                                                                egui::Key::ArrowDown if sel_y < matrix_height - 1 => {
                                                                                    self.selected_cell = Some((sel_x, sel_y + 1));
                                                                                }
                                                                                egui::Key::Delete | egui::Key::Backspace => {
                                                                                    editable_matrix[sel_y][sel_x] = ' ';
                                                                                    self.matrix_result.matrix_dirty = true;
                                                                                }
                                                                                egui::Key::Tab => {
                                                                                    if sel_x < editable_matrix[sel_y].len() - 1 {
                                                                                        self.selected_cell = Some((sel_x + 1, sel_y));
                                                                                    } else if sel_y < matrix_height - 1 {
                                                                                        self.selected_cell = Some((0, sel_y + 1));
                                                                                    }
                                                                                }
                                                                                egui::Key::Escape => {
                                                                                    self.selected_cell = None;
                                                                                    self.selection_start = None;
                                                                                    self.selection_end = None;
                                                                                }
                                                                                egui::Key::C if modifiers.command || modifiers.ctrl => {
                                                                                    // Copy selection
                                                                                    needs_copy = true;
                                                                                }
                                                                                egui::Key::V if modifiers.command || modifiers.ctrl => {
                                                                                    // Paste at current position
                                                                                    needs_paste_at = self.selected_cell;
                                                                                }
                                                                                _ => {}
                                                                            }
                                                                        }
                                                                    }
                                                                });
                                                                }
                                                            }
                                                        }

                                                        // Execute copy/paste operations outside of input closure
                                                        let mut copy_log_msg = None;
                                                        let mut paste_log_msg = None;

                                                        if needs_copy {
                                                            if let (Some(start), Some(end)) = (self.selection_start, self.selection_end) {
                                                                let min_x = start.0.min(end.0);
                                                                let max_x = start.0.max(end.0);
                                                                let min_y = start.1.min(end.1);
                                                                let max_y = start.1.max(end.1);

                                                                self.clipboard.clear();

                                                                for y in min_y..=max_y {
                                                                    if y < editable_matrix.len() {
                                                                        for x in min_x..=max_x {
                                                                            if x < editable_matrix[y].len() {
                                                                                self.clipboard.push(editable_matrix[y][x]);
                                                                            }
                                                                        }
                                                                        if y < max_y {
                                                                            self.clipboard.push('\n');
                                                                        }
                                                                    }
                                                                }

                                                                copy_log_msg = Some(format!("📋 Copied {} characters to clipboard", self.clipboard.len()));
                                                            }
                                                        }
                                                        if let Some((start_x, start_y)) = needs_paste_at {
                                                            let lines: Vec<&str> = self.clipboard.lines().collect();

                                                            for (line_idx, line) in lines.iter().enumerate() {
                                                                let y = start_y + line_idx;
                                                                if y >= editable_matrix.len() {
                                                                    break;
                                                                }

                                                                for (char_idx, ch) in line.chars().enumerate() {
                                                                    let x = start_x + char_idx;
                                                                    if x < editable_matrix[y].len() {
                                                                        editable_matrix[y][x] = ch;
                                                                    }
                                                                }
                                                            }

                                                            self.matrix_result.matrix_dirty = true;
                                                            paste_log_msg = Some(format!("📋 Pasted {} characters at ({}, {})", self.clipboard.len(), start_x, start_y));
                                                        }

                                                        // Extract values needed for the closure
                                                        let matrix_width = editable_matrix[0].len();
                                                        let matrix_height = editable_matrix.len();
                                                        let selected_cell = self.selected_cell;
                                                        let selection_start = self.selection_start;
                                                        let selection_end = self.selection_end;
                                                        let original_matrix = self.matrix_result.original_matrix.as_ref();
                                                        let is_dragging = self.is_dragging;

                                                        // Helper to check if a cell is selected
                                                        let is_cell_selected = |x: usize, y: usize| -> bool {
                                                            if let (Some(start), Some(end)) = (selection_start, selection_end) {
                                                                let min_x = start.0.min(end.0);
                                                                let max_x = start.0.max(end.0);
                                                                let min_y = start.1.min(end.1);
                                                                let max_y = start.1.max(end.1);

                                                                x >= min_x && x <= max_x && y >= min_y && y <= max_y
                                                            } else {
                                                                false
                                                            }
                                                        };

                                                        // Display the editable matrix as a tight grid (Dwarf Fortress style)
                                                        let scroll_output = egui::ScrollArea::both()
                                                            .auto_shrink([false; 2])
                                                            .show(ui, |ui| {
                                                                // Use a custom widget to draw the entire matrix at once
                                                                let matrix_size = egui::vec2(
                                                                    matrix_width as f32 * char_size,
                                                                    matrix_height as f32 * line_height
                                                                );

                                                                let (response, painter) = ui.allocate_painter(matrix_size, egui::Sense::click());
                                                                let rect = response.rect;

                                                                // Draw all characters in a tight grid
                                                                for (y, row) in editable_matrix.iter().enumerate() {
                                                                    for (x, ch) in row.iter().enumerate() {
                                                                        let pos = rect.min + egui::vec2(
                                                                            x as f32 * char_size + char_size / 2.0,
                                                                            y as f32 * line_height + line_height / 2.0
                                                                        );

                                                                        // Color based on character type and modification status
                                                                        let is_modified = if let Some(original) = original_matrix {
                                                                            y < original.len() && x < original[y].len() && original[y][x] != *ch
                                                                        } else {
                                                                            false
                                                                        };

                                                                        let color = if is_modified {
                                                                            TERM_YELLOW
                                                                        } else if *ch == ' ' {
                                                                            Color32::from_rgba_premultiplied(0, 0, 0, 0) // Transparent for spaces
                                                                        } else if *ch == '█' {
                                                                            TERM_DIM
                                                                        } else {
                                                                            TERM_FG
                                                                        };

                                                                        // Only draw non-space characters
                                                                        if *ch != ' ' {
                                                                            painter.text(
                                                                                pos,
                                                                                egui::Align2::CENTER_CENTER,
                                                                                ch.to_string(),
                                                                                font_id.clone(),
                                                                                color,
                                                                            );
                                                                        }

                                                                        // Highlight selection or selected cell
                                                                        if is_cell_selected(x, y) {
                                                                            // Part of drag selection
                                                                            let cell_rect = egui::Rect::from_min_size(
                                                                                rect.min + egui::vec2(x as f32 * char_size, y as f32 * line_height),
                                                                                egui::vec2(char_size, line_height)
                                                                            );
                                                                            painter.rect_filled(cell_rect, 0.0, TERM_HIGHLIGHT.gamma_multiply(0.2));
                                                                        } else if selected_cell == Some((x, y)) {
                                                                            // Single cell selection
                                                                            let cell_rect = egui::Rect::from_min_size(
                                                                                rect.min + egui::vec2(x as f32 * char_size, y as f32 * line_height),
                                                                                egui::vec2(char_size, line_height)
                                                                            );
                                                                            painter.rect_filled(cell_rect, 0.0, TERM_HIGHLIGHT.gamma_multiply(0.3));
                                                                            painter.rect_stroke(cell_rect, 0.0, egui::Stroke::new(2.0, TERM_HIGHLIGHT));
                                                                        }
                                                                    }
                                                                }

                                                                // Return drag action to be handled outside the closure
                                                                let mut drag_action = DragAction::None;

                                                                // Handle clicks and drags on the matrix
                                                                if response.drag_started() {
                                                                    if let Some(pos) = response.interact_pointer_pos() {
                                                                        let rel_pos = pos - rect.min;
                                                                        let x = (rel_pos.x / char_size) as usize;
                                                                        let y = (rel_pos.y / line_height) as usize;

                                                                        if y < matrix_height && x < matrix_width {
                                                                            drag_action = DragAction::StartDrag(x, y);
                                                                        }
                                                                    }
                                                                }

                                                                if is_dragging && response.dragged() {
                                                                    if let Some(pos) = response.interact_pointer_pos() {
                                                                        let rel_pos = pos - rect.min;
                                                                        let x = ((rel_pos.x / char_size) as usize).min(matrix_width - 1);
                                                                        let y = ((rel_pos.y / line_height) as usize).min(matrix_height - 1);

                                                                        drag_action = DragAction::UpdateDrag(x, y);
                                                                    }
                                                                }

                                                                if response.drag_released() {
                                                                    drag_action = DragAction::EndDrag;
                                                                }

                                                                // Single click (no drag)
                                                                if response.clicked() && selection_start == selection_end {
                                                                    if let Some(pos) = response.interact_pointer_pos() {
                                                                        let rel_pos = pos - rect.min;
                                                                        let x = (rel_pos.x / char_size) as usize;
                                                                        let y = (rel_pos.y / line_height) as usize;

                                                                        if y < matrix_height && x < matrix_width {
                                                                            drag_action = DragAction::SingleClick(x, y);
                                                                        }
                                                                    }
                                                                }

                                                                // Return the drag action
                                                                drag_action
                                                            });

                                                        // Handle the drag action returned from the closure
                                                        match scroll_output.inner {
                                                                DragAction::StartDrag(x, y) => {
                                                                    self.selection_start = Some((x, y));
                                                                    self.selection_end = Some((x, y));
                                                                    self.is_dragging = true;
                                                                    self.selected_cell = None;
                                                                }
                                                                DragAction::UpdateDrag(x, y) => {
                                                                    self.selection_end = Some((x, y));
                                                                }
                                                                DragAction::EndDrag => {
                                                                    self.is_dragging = false;
                                                                }
                                                                DragAction::SingleClick(x, y) => {
                                                                    self.selected_cell = Some((x, y));
                                                                    self.selection_start = None;
                                                                    self.selection_end = None;
                                                                }
                                                                DragAction::None => {}
                                                        }

                                                        // Log any copy/paste messages after the borrow is done
                                                        if let Some(msg) = copy_log_msg {
                                                            self.log(&msg);
                                                        }
                                                        if let Some(msg) = paste_log_msg {
                                                            self.log(&msg);
                                                        }

                                                        // Help text
                                                        ui.separator();
                                                        ui.horizontal(|ui| {
                                                            ui.label(RichText::new("Click to select | Drag to select area | Ctrl+C to copy | Ctrl+V to paste | Arrow keys | Esc to clear | [S] to save")
                                                                .color(TERM_DIM)
                                                                .monospace()
                                                                .size(10.0));
                                                        });
                                                    } else {
                                                        ui.centered_and_justified(|ui| {
                                                            ui.label(RichText::new("No character matrix yet\n\nPress [M] to extract text at exact positions")
                                                                .color(TERM_DIM)
                                                                .monospace());
                                                        });
                                                    }
                                                }
                                                ExtractionTab::Debug => {
                                                    if !self.matrix_result.content.is_empty() {
                                                        ui.label(RichText::new(&self.matrix_result.content)
                                                            .color(TERM_FG)
                                                            .monospace());
                                                    } else {
                                                        ui.centered_and_justified(|ui| {
                                                            ui.label(RichText::new("No debug data available\n\nPress [G] to show debug info")
                                                                .color(TERM_DIM)
                                                                .monospace());
                                                        });
                                                    }
                                                }
                                                _ => {}
                                            }
                                        });
                                });
                            }
                        );

                        ui.add_space(padding); // Add right padding
                    });
                } else {
                    // No PDF loaded - show welcome screen
                    draw_terminal_box(ui, "WELCOME", false, |ui| {
                        ui.centered_and_justified(|ui| {
                            ui.label(RichText::new("🐹 CHONKER 5\n\nCharacter Matrix PDF Representation\n\nPress [O] to open a PDF file\n\nThen [M] to create faithful character matrix")
                                .color(TERM_FG)
                                .monospace()
                                .size(16.0));
                        });
                    });
                }

                // Collapsible log panel at bottom
                ui.add_space(5.0);
                egui::CollapsingHeader::new(RichText::new("▼ LOG").color(CHROME).monospace())
                    .default_open(false)
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical()
                            .max_height(60.0)
                            .auto_shrink([false; 2])
                            .show(ui, |ui| {
                                for message in &self.log_messages {
                                    ui.label(RichText::new(message).color(TERM_FG).monospace().size(10.0));
                                }
                            });
                    });
            });
    }
}

// Helper functions
#[allow(dead_code)] // Utility function for future page range parsing
fn parse_page_range(range_str: &str) -> Result<std::ops::Range<usize>, String> {
    if range_str.trim().is_empty() {
        return Err("Empty page range".to_string());
    }

    let parts: Vec<&str> = range_str.split('-').collect();
    if parts.len() != 2 {
        return Err("Invalid range format. Use format like '1-10'".to_string());
    }

    let start = parts[0]
        .trim()
        .parse::<usize>()
        .map_err(|_| format!("Invalid start page: {}", parts[0]))?;
    let end = parts[1]
        .trim()
        .parse::<usize>()
        .map_err(|_| format!("Invalid end page: {}", parts[1]))?;

    if start == 0 {
        return Err("Page numbers start at 1, not 0".to_string());
    }

    if start > end {
        return Err(format!(
            "Start page {} is greater than end page {}",
            start, end
        ));
    }

    // Convert to 0-based indexing
    Ok((start - 1)..end)
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1200.0, 800.0]),
        ..Default::default()
    };

    eframe::run_native(
        "🐹 CHONKER 5 - PDF Viewer",
        options,
        Box::new(|cc| Box::new(Chonker5App::new(cc))),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char_bbox_contains() {
        let bbox = CharBBox {
            x: 10,
            y: 5,
            width: 20,
            height: 15,
        };

        // Test points inside the box
        assert!(bbox.contains(10, 5)); // Top-left corner
        assert!(bbox.contains(15, 10)); // Center
        assert!(bbox.contains(29, 19)); // Bottom-right corner (exclusive)

        // Test points outside the box
        assert!(!bbox.contains(9, 5)); // Left of box
        assert!(!bbox.contains(10, 4)); // Above box
        assert!(!bbox.contains(30, 10)); // Right of box
        assert!(!bbox.contains(15, 20)); // Below box
    }

    #[test]
    fn test_char_bbox_area() {
        let bbox = CharBBox {
            x: 0,
            y: 0,
            width: 10,
            height: 5,
        };

        assert_eq!(bbox.area(), 50);

        // Test zero area
        let zero_bbox = CharBBox {
            x: 0,
            y: 0,
            width: 0,
            height: 10,
        };
        assert_eq!(zero_bbox.area(), 0);
    }

    #[test]
    fn test_character_matrix_engine_new() {
        let engine = CharacterMatrixEngine::new();
        assert_eq!(engine.char_width, 6.0);
        assert_eq!(engine.char_height, 12.0);
    }

    #[test]
    fn test_character_matrix_creation() {
        let matrix = CharacterMatrix {
            width: 80,
            height: 25,
            matrix: vec![vec![' '; 80]; 25],
            text_regions: vec![],
            original_text: vec!["Test text".to_string()],
            char_width: 6.0,
            char_height: 12.0,
        };

        assert_eq!(matrix.width, 80);
        assert_eq!(matrix.height, 25);
        assert_eq!(matrix.matrix.len(), 25);
        assert_eq!(matrix.matrix[0].len(), 80);
        assert_eq!(matrix.original_text.len(), 1);
    }

    #[test]
    fn test_text_region_creation() {
        let region = TextRegion {
            bbox: CharBBox {
                x: 5,
                y: 3,
                width: 10,
                height: 2,
            },
            confidence: 0.95,
            text_content: "Hello World".to_string(),
            region_id: 0,
        };

        assert_eq!(region.bbox.area(), 20);
        assert_eq!(region.confidence, 0.95);
        assert_eq!(region.text_content, "Hello World");
        assert_eq!(region.region_id, 0);
    }

    #[test]
    fn test_pdfbbox_width_height() {
        let bbox = PDFBBox {
            x0: 10.0,
            y0: 20.0,
            x1: 30.0,
            y1: 50.0,
        };

        assert_eq!(bbox.width(), 20.0);
        assert_eq!(bbox.height(), 30.0);
    }

    #[test]
    fn test_serialization() {
        let matrix = CharacterMatrix {
            width: 10,
            height: 5,
            matrix: vec![vec!['X'; 10]; 5],
            text_regions: vec![TextRegion {
                bbox: CharBBox {
                    x: 0,
                    y: 0,
                    width: 5,
                    height: 2,
                },
                confidence: 0.8,
                text_content: "Test".to_string(),
                region_id: 0,
            }],
            original_text: vec!["Original".to_string()],
            char_width: 8.0,
            char_height: 14.0,
        };

        // Test serialization
        let json = serde_json::to_string(&matrix);
        assert!(json.is_ok());

        // Test deserialization
        let deserialized: Result<CharacterMatrix, _> = serde_json::from_str(&json.unwrap());
        assert!(deserialized.is_ok());

        let deserialized_matrix = deserialized.unwrap();
        assert_eq!(deserialized_matrix.width, matrix.width);
        assert_eq!(deserialized_matrix.height, matrix.height);
        assert_eq!(deserialized_matrix.text_regions.len(), 1);
    }
}
