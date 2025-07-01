use serde::Deserialize;
use std::path::Path;
use eframe::egui;

/// Docling bounding box structure (matches their JSON output)
#[derive(Debug, Clone, Deserialize)]
pub struct DoclingBBox {
    pub l: f32,    // left
    pub t: f32,    // top  
    pub r: f32,    // right
    pub b: f32,    // bottom
    pub coord_origin: String, // "BOTTOMLEFT" or "TOPLEFT"
}

/// Individual text item with coordinates from Docling
#[derive(Debug, Clone, Deserialize)]
pub struct DoclingTextItem {
    pub text: String,
    pub bbox: DoclingBBox,
    pub page_no: u32,
    pub confidence: Option<f32>,
    pub element_type: Option<String>, // paragraph, table, etc.
}

/// Processed text region for UI interaction
#[derive(Debug, Clone)]
pub struct TextRegion {
    pub rect: egui::Rect,
    pub text: String,
    pub page_no: u32,
    pub text_index: usize,
    pub element_type: String,
    pub is_selected: bool,
}

/// Main coordinate mapping system
pub struct CoordinateMapper {
    pub text_items: Vec<DoclingTextItem>,
    pub text_regions: Vec<TextRegion>,
    pub selected_region: Option<usize>,
    pub selected_text_item: Option<usize>,
    pub pdf_image_size: egui::Vec2,
}

impl CoordinateMapper {
    pub fn new() -> Self {
        Self {
            text_items: Vec::new(),
            text_regions: Vec::new(),
            selected_region: None,
            selected_text_item: None,
            pdf_image_size: egui::Vec2::ZERO,
        }
    }

    /// Load Docling extraction results from JSON file
    pub fn load_docling_output(&mut self, json_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(json_path)?;
        self.text_items = serde_json::from_str(&content)?;
        tracing::info!("Loaded {} text items from Docling output", self.text_items.len());
        Ok(())
    }

    /// Convert Docling coordinates to screen coordinates
    pub fn convert_docling_to_screen_coords(&self, bbox: &DoclingBBox, image_height: f32) -> egui::Rect {
        // Docling typically uses bottom-left origin, egui uses top-left
        let left = bbox.l;
        let right = bbox.r;
        
        let (top, bottom) = if bbox.coord_origin == "BOTTOMLEFT" {
            // Flip Y coordinate for bottom-left origin
            let top = image_height - bbox.t;
            let bottom = image_height - bbox.b;
            (top.min(bottom), top.max(bottom))
        } else {
            // Top-left origin, use as-is
            (bbox.t, bbox.b)
        };
        
        egui::Rect::from_min_max(
            egui::pos2(left, top),
            egui::pos2(right, bottom)
        )
    }

    /// Generate clickable regions from loaded text items
    pub fn generate_text_regions(&mut self, pdf_image_size: egui::Vec2) {
        self.pdf_image_size = pdf_image_size;
        self.text_regions.clear();

        for (index, item) in self.text_items.iter().enumerate() {
            let rect = self.convert_docling_to_screen_coords(&item.bbox, pdf_image_size.y);
            
            let region = TextRegion {
                rect,
                text: item.text.clone(),
                page_no: item.page_no,
                text_index: index,
                element_type: item.element_type.clone().unwrap_or_else(|| "text".to_string()),
                is_selected: false,
            };
            
            self.text_regions.push(region);
        }

        tracing::info!("Generated {} clickable text regions", self.text_regions.len());
    }

    /// Handle click on PDF at given coordinates
    pub fn handle_pdf_click(&mut self, click_pos: egui::Pos2, scale_factor: egui::Vec2) -> Option<usize> {
        // Scale click position to original PDF coordinates
        let pdf_pos = egui::pos2(
            click_pos.x / scale_factor.x,
            click_pos.y / scale_factor.y
        );

        // Find the clicked region first
        let mut clicked_region = None;
        for (index, region) in self.text_regions.iter().enumerate() {
            if region.rect.contains(pdf_pos) {
                clicked_region = Some((index, region.text_index));
                break;
            }
        }

        // Clear previous selections
        self.clear_selections();

        // If we found a clicked region, select it
        if let Some((region_index, text_index)) = clicked_region {
            self.text_regions[region_index].is_selected = true;
            self.selected_region = Some(region_index);
            self.selected_text_item = Some(text_index);
            
            let text_preview = &self.text_regions[region_index].text[..self.text_regions[region_index].text.len().min(50)];
            tracing::info!("Selected region {}: '{}'", region_index, text_preview);
            return Some(region_index);
        }

        // No region clicked
        None
    }

    /// Handle text selection in the text panel
    pub fn handle_text_selection(&mut self, text_index: usize) -> Option<usize> {
        // Clear previous selections
        self.clear_selections();

        // Find the corresponding region
        for (region_index, region) in self.text_regions.iter_mut().enumerate() {
            if region.text_index == text_index {
                region.is_selected = true;
                self.selected_region = Some(region_index);
                self.selected_text_item = Some(text_index);
                
                tracing::info!("Selected text item {}: '{}'", text_index, &region.text[..region.text.len().min(50)]);
                return Some(region_index);
            }
        }

        None
    }

    /// Clear all selections
    pub fn clear_selections(&mut self) {
        for region in &mut self.text_regions {
            region.is_selected = false;
        }
        self.selected_region = None;
        self.selected_text_item = None;
    }

    /// Get currently selected text
    pub fn get_selected_text(&self) -> Option<&str> {
        if let Some(text_index) = self.selected_text_item {
            self.text_items.get(text_index).map(|item| item.text.as_str())
        } else {
            None
        }
    }

    /// Get selected region for highlighting
    pub fn get_selected_region(&self) -> Option<&TextRegion> {
        if let Some(region_index) = self.selected_region {
            self.text_regions.get(region_index)
        } else {
            None
        }
    }

    /// Render debug overlay showing all clickable regions
    pub fn render_debug_overlay(&self, ui: &mut egui::Ui, image_rect: egui::Rect, scale_factor: egui::Vec2) {
        let painter = ui.painter();
        
        for (index, region) in self.text_regions.iter().enumerate() {
            // Scale region to display coordinates
            let display_rect = egui::Rect::from_min_max(
                image_rect.min + egui::vec2(
                    region.rect.min.x * scale_factor.x,
                    region.rect.min.y * scale_factor.y
                ),
                image_rect.min + egui::vec2(
                    region.rect.max.x * scale_factor.x,
                    region.rect.max.y * scale_factor.y
                )
            );

            // Choose color based on selection and element type
            let color = if region.is_selected {
                egui::Color32::RED
            } else {
                match region.element_type.as_str() {
                    "table" => egui::Color32::BLUE,
                    "heading" => egui::Color32::GREEN,
                    _ => egui::Color32::YELLOW,
                }
            };

            // Draw debug rectangle
            painter.rect_stroke(
                display_rect,
                egui::Rounding::default(),
                egui::Stroke::new(if region.is_selected { 2.0 } else { 1.0 }, color)
            );

            // Draw index number for debugging
            if display_rect.width() > 20.0 && display_rect.height() > 10.0 {
                painter.text(
                    display_rect.left_top() + egui::vec2(2.0, 2.0),
                    egui::Align2::LEFT_TOP,
                    index.to_string(),
                    egui::FontId::monospace(10.0),
                    egui::Color32::WHITE
                );
            }
        }
    }
}

impl Default for CoordinateMapper {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to render PDF with coordinate mapping
pub fn render_pdf_page(pdf_path: &str, page_num: u32) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    use std::process::Command;
    
    let temp_path = format!("/tmp/chonker_pdf_page_{}", page_num);
    
    let output = Command::new("pdftoppm")
        .args(["-png", "-r", "72"])  // Critical: 72 DPI matches Docling
        .args(["-f", &page_num.to_string(), "-l", &page_num.to_string()])
        .args([pdf_path, &temp_path])
        .output()?;
    
    if !output.status.success() {
        return Err(format!("pdftoppm failed: {}", String::from_utf8_lossy(&output.stderr)).into());
    }
    
    Ok(std::path::PathBuf::from(format!("{}-{:02}.png", temp_path, page_num)))
}
