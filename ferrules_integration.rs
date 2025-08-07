use anyhow::Result;
use ferrules_core::layout::{LayoutBBox, ORTLayoutParser, ORTConfig};
use image::{DynamicImage, RgbImage};
use crate::character_matrix_engine::{TextRegion, CharBBox};

/// Convert PDF page to image and detect text regions using Ferrules
pub async fn detect_text_regions_with_ferrules(
    page_image: &RgbImage,
    char_width: f32,
    char_height: f32,
) -> Result<Vec<TextRegion>> {
    // Convert RgbImage to DynamicImage for Ferrules
    let dynamic_image = DynamicImage::ImageRgb8(page_image.clone());
    
    // Initialize Ferrules layout parser with default config
    let config = ORTConfig::default();
    let parser = ORTLayoutParser::new(&config)?;
    
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
            let char_width_count = ((layout_bbox.bbox.x1 - layout_bbox.bbox.x0) / char_width).ceil() as usize;
            let char_height_count = ((layout_bbox.bbox.y1 - layout_bbox.bbox.y0) / char_height).ceil() as usize;
            
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

/// Integration point for character_matrix_engine.rs
pub fn replace_flood_fill_with_ferrules() -> String {
    r#"
    // In character_matrix_engine.rs, replace the detect_text_regions_with_vision method:
    
    async fn detect_text_regions_with_vision(
        &self,
        image: &RgbImage,
        _matrix: &[Vec<char>],
    ) -> Result<Vec<TextRegion>> {
        // Use Ferrules instead of flood-fill
        detect_text_regions_with_ferrules(image, self.char_width, self.char_height).await
    }
    "#.to_string()
}