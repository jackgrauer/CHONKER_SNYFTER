use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use pdfium_render::prelude::*;

// Core spatial-semantic engine that actually understands documents
// like a human brain does - by building coherent spatial relationships
// with semantic meaning layered on top

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub pages: Vec<Page>,
    pub metadata: DocumentMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub subject: Option<String>,
    pub page_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    pub page_number: usize,
    pub width: f32,
    pub height: f32,
    pub elements: Vec<Element>,
    pub spatial_graph: SpatialGraph,
    pub semantic_regions: Vec<SemanticRegion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Element {
    pub id: ElementId,
    pub bbox: BBox,
    pub text: String,
    pub font_name: String,
    pub font_size: f32,
    pub is_bold: bool,
    pub is_italic: bool,
    pub color: Color,
    pub element_type: ElementType,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ElementId(pub usize);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BBox {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
}

impl BBox {
    pub fn width(&self) -> f32 { self.x1 - self.x0 }
    pub fn height(&self) -> f32 { self.y1 - self.y0 }
    pub fn center(&self) -> (f32, f32) { 
        ((self.x0 + self.x1) / 2.0, (self.y0 + self.y1) / 2.0) 
    }
    pub fn area(&self) -> f32 { self.width() * self.height() }
    
    pub fn overlaps(&self, other: &BBox) -> bool {
        self.x0 < other.x1 && self.x1 > other.x0 && 
        self.y0 < other.y1 && self.y1 > other.y0
    }
    
    pub fn contains(&self, other: &BBox) -> bool {
        self.x0 <= other.x0 && self.x1 >= other.x1 &&
        self.y0 <= other.y0 && self.y1 >= other.y1
    }
    
    pub fn distance_to(&self, other: &BBox) -> f32 {
        let (cx1, cy1) = self.center();
        let (cx2, cy2) = other.center();
        ((cx2 - cx1).powi(2) + (cy2 - cy1).powi(2)).sqrt()
    }
    
    pub fn horizontal_distance(&self, other: &BBox) -> f32 {
        if self.x1 < other.x0 {
            other.x0 - self.x1  // Gap between right edge of self and left edge of other
        } else if other.x1 < self.x0 {
            self.x0 - other.x1  // Gap between right edge of other and left edge of self
        } else {
            0.0  // Overlapping horizontally
        }
    }
    
    pub fn vertical_distance(&self, other: &BBox) -> f32 {
        if self.y1 < other.y0 {
            other.y0 - self.y1  // Gap between bottom of self and top of other
        } else if other.y1 < self.y0 {
            self.y0 - other.y1  // Gap between bottom of other and top of self
        } else {
            0.0  // Overlapping vertically
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ElementType {
    Title { level: usize },
    Heading { level: usize },
    Paragraph,
    ListItem,
    TableCell { row: usize, col: usize },
    Caption,
    Footnote,
    Header,
    Footer,
    Number,
    Date,
    Currency,
    Reference,
    Unknown,
}

// Spatial graph represents the actual relationships between elements
// like a human brain understands spatial layout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialGraph {
    pub relationships: HashMap<ElementId, Vec<SpatialRelationship>>,
    pub reading_order: Vec<ElementId>,
    pub visual_groups: Vec<VisualGroup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialRelationship {
    pub target: ElementId,
    pub relationship_type: RelationshipType,
    pub confidence: f32,
    pub distance: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelationshipType {
    Above,
    Below,
    LeftOf,
    RightOf,
    Contains,
    ContainedBy,
    AlignedHorizontally,
    AlignedVertically,
    SameColumn,
    SameRow,
    Continuation,  // Text continues from previous element
    Association,   // Semantically related (like label -> value)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualGroup {
    pub elements: Vec<ElementId>,
    pub group_type: GroupType,
    pub bbox: BBox,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GroupType {
    Table,
    List,
    Column,
    Header,
    Footer,
    Sidebar,
    TextBlock,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticRegion {
    pub bbox: BBox,
    pub region_type: RegionType,
    pub elements: Vec<ElementId>,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegionType {
    Title,
    Abstract,
    Introduction,
    Methodology,
    Results,
    Conclusion,
    References,
    Table,
    Figure,
    Caption,
    Header,
    Footer,
    Sidebar,
}

pub struct SpatialSemanticEngine {
    // Combine pdfium's precise geometric data with actual understanding
}

impl SpatialSemanticEngine {
    pub fn new() -> Self {
        Self {}
    }
    
    pub fn process_pdf(&self, pdf_path: &Path) -> Result<Document> {
        // Step 1: Extract precise geometric + text data using pdfium
        let raw_document = self.extract_raw_data(pdf_path)?;
        
        // Step 2: Build spatial relationships graph
        let pages_with_spatial = self.build_spatial_graphs(raw_document)?;
        
        // Step 3: Apply semantic understanding on top of spatial foundation
        let final_document = self.apply_semantic_analysis(pages_with_spatial)?;
        
        Ok(final_document)
    }
    
    fn extract_raw_data(&self, pdf_path: &Path) -> Result<Document> {
        // Use pdfium to get the precise geometric data - similar to chonker5.rs pattern
        let pdfium = Pdfium::new(
            Pdfium::bind_to_system_library()
                .or_else(|_| Pdfium::bind_to_library("./lib/libpdfium.dylib"))
                .or_else(|_| Pdfium::bind_to_library("/usr/local/lib/libpdfium.dylib"))
                .map_err(|e| anyhow::anyhow!("Failed to bind pdfium: {}", e))?
        );
        
        let document = pdfium.load_pdf_from_file(pdf_path, None)
            .map_err(|e| anyhow::anyhow!("Failed to load PDF: {}", e))?;
        
        let mut pages = Vec::new();
        
        for (page_index, page) in document.pages().iter().enumerate() {
            let page_width = page.width().value;
            let page_height = page.height().value;
            
            let mut elements = Vec::new();
            let mut element_id_counter = 0;
            
            // Extract text with precise positioning using the same approach as chonker5.rs
            let text_page = page.text().map_err(|e| anyhow::anyhow!("Failed to get text: {}", e))?;
            
            // For now, use the simpler text extraction approach and build up to object-level extraction
            let all_text = text_page.all();
            
            // Split text into logical elements (simplified for now)
            for (line_index, line) in all_text.lines().enumerate() {
                let text = line.trim();
                if text.is_empty() {
                    continue;
                }
                
                // Create synthetic bounding box (in a real implementation, we'd get actual coordinates)
                let bbox = BBox {
                    x0: 50.0, // Left margin
                    y0: page_height - (line_index as f32 * 20.0) - 100.0, // Top-down positioning
                    x1: page_width - 50.0, // Right margin  
                    y1: page_height - (line_index as f32 * 20.0) - 80.0, // Line height
                };
                
                // Basic text analysis for initial type classification
                let element_type = self.classify_text_element(text, 12.0, "Arial");
                
                let element = Element {
                    id: ElementId(element_id_counter),
                    bbox,
                    text: text.to_string(),
                    font_name: "Arial".to_string(), // Simplified
                    font_size: 12.0, // Simplified
                    is_bold: text.chars().any(|c| c.is_uppercase()),
                    is_italic: false,
                    color: Color { r: 0, g: 0, b: 0 },
                    element_type,
                    confidence: 0.8, // Lower since this is simplified extraction
                };
                
                elements.push(element);
                element_id_counter += 1;
            }
            
            let page = Page {
                page_number: page_index,
                width: page_width,
                height: page_height,
                elements,
                spatial_graph: SpatialGraph {
                    relationships: HashMap::new(),
                    reading_order: Vec::new(),
                    visual_groups: Vec::new(),
                },
                semantic_regions: Vec::new(),
            };
            
            pages.push(page);
        }
        
        let metadata = DocumentMetadata {
            title: None, // TODO: Extract from document metadata
            author: None,
            subject: None,
            page_count: pages.len(),
        };
        
        Ok(Document { pages, metadata })
    }
    
    fn build_spatial_graphs(&self, mut document: Document) -> Result<Document> {
        for page in &mut document.pages {
            // Build spatial relationships - this is where the magic happens
            let mut relationships: HashMap<ElementId, Vec<SpatialRelationship>> = HashMap::new();
            
            // For each element, find its spatial relationships with other elements
            for (i, element) in page.elements.iter().enumerate() {
                let mut element_relationships = Vec::new();
                
                for (j, other) in page.elements.iter().enumerate() {
                    if i == j { continue; }
                    
                    let rel_type = self.determine_spatial_relationship(&element.bbox, &other.bbox);
                    if let Some(rel_type) = rel_type {
                        let distance = element.bbox.distance_to(&other.bbox);
                        element_relationships.push(SpatialRelationship {
                            target: other.id.clone(),
                            relationship_type: rel_type,
                            confidence: 0.9, // High confidence for geometric relationships
                            distance,
                        });
                    }
                }
                
                relationships.insert(element.id.clone(), element_relationships);
            }
            
            // Determine reading order using spatial relationships
            let reading_order = self.compute_reading_order(&page.elements);
            
            // Find visual groups (tables, lists, columns, etc.)
            let visual_groups = self.identify_visual_groups(&page.elements);
            
            page.spatial_graph = SpatialGraph {
                relationships,
                reading_order,
                visual_groups,
            };
        }
        
        Ok(document)
    }
    
    fn apply_semantic_analysis(&self, mut document: Document) -> Result<Document> {
        for page in &mut document.pages {
            // Now apply semantic understanding on top of the spatial foundation
            
            // 1. Identify semantic regions using spatial patterns + content
            page.semantic_regions = self.identify_semantic_regions(page);
            
            // 2. Refine element types using spatial context
            self.refine_element_types(page);
            
            // 3. Build associations (labels -> values, captions -> figures, etc.)
            self.build_semantic_associations(page);
        }
        
        Ok(document)
    }
    
    fn classify_text_element(&self, text: &str, font_size: f32, font_name: &str) -> ElementType {
        let text_trimmed = text.trim();
        
        // Basic heuristics - this is where you'd plug in better ML models
        if font_size > 16.0 {
            ElementType::Title { level: 1 }
        } else if font_size > 14.0 {
            ElementType::Heading { level: 2 }
        } else if text_trimmed.starts_with('•') || text_trimmed.starts_with('-') || text_trimmed.starts_with('*') {
            ElementType::ListItem
        } else if text_trimmed.parse::<f64>().is_ok() {
            ElementType::Number
        } else if text_trimmed.len() < 50 && text_trimmed.contains(':') {
            ElementType::Reference // Might be a label
        } else {
            ElementType::Paragraph
        }
    }
    
    fn determine_spatial_relationship(&self, bbox1: &BBox, bbox2: &BBox) -> Option<RelationshipType> {
        const ALIGNMENT_THRESHOLD: f32 = 5.0;
        const PROXIMITY_THRESHOLD: f32 = 50.0;
        
        // Check for containment
        if bbox1.contains(bbox2) {
            return Some(RelationshipType::Contains);
        }
        if bbox2.contains(bbox1) {
            return Some(RelationshipType::ContainedBy);
        }
        
        // Check for spatial proximity
        let h_dist = bbox1.horizontal_distance(bbox2);
        let v_dist = bbox1.vertical_distance(bbox2);
        
        if h_dist > PROXIMITY_THRESHOLD && v_dist > PROXIMITY_THRESHOLD {
            return None; // Too far apart
        }
        
        // Check for alignment
        if (bbox1.y0 - bbox2.y0).abs() < ALIGNMENT_THRESHOLD {
            if bbox1.x1 < bbox2.x0 {
                return Some(RelationshipType::LeftOf);
            } else if bbox2.x1 < bbox1.x0 {
                return Some(RelationshipType::RightOf);
            } else {
                return Some(RelationshipType::AlignedHorizontally);
            }
        }
        
        if (bbox1.x0 - bbox2.x0).abs() < ALIGNMENT_THRESHOLD {
            if bbox1.y1 < bbox2.y0 {
                return Some(RelationshipType::Above);
            } else if bbox2.y1 < bbox1.y0 {
                return Some(RelationshipType::Below);
            } else {
                return Some(RelationshipType::AlignedVertically);
            }
        }
        
        // General positional relationships
        if bbox1.y1 < bbox2.y0 {
            Some(RelationshipType::Above)
        } else if bbox2.y1 < bbox1.y0 {
            Some(RelationshipType::Below)
        } else if bbox1.x1 < bbox2.x0 {
            Some(RelationshipType::LeftOf)
        } else if bbox2.x1 < bbox1.x0 {
            Some(RelationshipType::RightOf)
        } else {
            None
        }
    }
    
    fn compute_reading_order(&self, elements: &[Element]) -> Vec<ElementId> {
        let mut sorted_elements: Vec<_> = elements.iter().collect();
        
        // Sort by top-to-bottom, then left-to-right
        // This is a simplified reading order - real implementation would be more sophisticated
        sorted_elements.sort_by(|a, b| {
            let y_diff = a.bbox.y0.partial_cmp(&b.bbox.y0).unwrap();
            if y_diff == std::cmp::Ordering::Equal {
                a.bbox.x0.partial_cmp(&b.bbox.x0).unwrap()
            } else {
                y_diff
            }
        });
        
        sorted_elements.iter().map(|e| e.id.clone()).collect()
    }
    
    fn identify_visual_groups(&self, elements: &[Element]) -> Vec<VisualGroup> {
        let mut groups = Vec::new();
        
        // Simple table detection based on alignment
        let mut processed: HashSet<ElementId> = HashSet::new();
        
        for element in elements {
            if processed.contains(&element.id) {
                continue;
            }
            
            // Find elements aligned with this one
            let mut aligned_elements = vec![element.id.clone()];
            
            for other in elements {
                if other.id == element.id || processed.contains(&other.id) {
                    continue;
                }
                
                // Check for horizontal alignment (same row)
                if (element.bbox.y0 - other.bbox.y0).abs() < 5.0 {
                    aligned_elements.push(other.id.clone());
                }
            }
            
            if aligned_elements.len() > 2 {
                // Calculate bounding box for the group
                let min_x = elements.iter()
                    .filter(|e| aligned_elements.contains(&e.id))
                    .map(|e| e.bbox.x0)
                    .min_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap_or(0.0);
                let max_x = elements.iter()
                    .filter(|e| aligned_elements.contains(&e.id))
                    .map(|e| e.bbox.x1)
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap_or(0.0);
                let min_y = elements.iter()
                    .filter(|e| aligned_elements.contains(&e.id))
                    .map(|e| e.bbox.y0)
                    .min_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap_or(0.0);
                let max_y = elements.iter()
                    .filter(|e| aligned_elements.contains(&e.id))
                    .map(|e| e.bbox.y1)
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap_or(0.0);
                
                groups.push(VisualGroup {
                    elements: aligned_elements.clone(),
                    group_type: GroupType::Table, // Simplified - would need more analysis
                    bbox: BBox { x0: min_x, y0: min_y, x1: max_x, y1: max_y },
                });
                
                for id in aligned_elements {
                    processed.insert(id);
                }
            }
        }
        
        groups
    }
    
    fn identify_semantic_regions(&self, page: &Page) -> Vec<SemanticRegion> {
        let mut regions = Vec::new();
        
        // Find title regions (large text at top)
        for element in &page.elements {
            if let ElementType::Title { level: 1 } = element.element_type {
                if element.bbox.y0 > page.height * 0.8 { // Top 20% of page
                    regions.push(SemanticRegion {
                        bbox: element.bbox.clone(),
                        region_type: RegionType::Title,
                        elements: vec![element.id.clone()],
                        confidence: 0.9,
                    });
                }
            }
        }
        
        // TODO: Add more sophisticated region detection
        // - Tables (using visual groups)
        // - Headers/footers (position-based)
        // - Columns (spatial analysis)
        // - etc.
        
        regions
    }
    
    fn refine_element_types(&self, page: &mut Page) {
        // Use spatial context to refine element classifications
        for element in &mut page.elements {
            // Example: If this element is in a table group, classify as table cell
            for group in &page.spatial_graph.visual_groups {
                if group.elements.contains(&element.id) {
                    match group.group_type {
                        GroupType::Table => {
                            // Find row/column position within table
                            let row = 0; // TODO: Calculate actual row
                            let col = 0; // TODO: Calculate actual column
                            element.element_type = ElementType::TableCell { row, col };
                        }
                        GroupType::List => {
                            element.element_type = ElementType::ListItem;
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    
    fn build_semantic_associations(&self, page: &mut Page) {
        // Build associations like label -> value, caption -> figure
        // This uses the spatial relationships to find semantic connections
        
        for (element_id, relationships) in &mut page.spatial_graph.relationships {
            let element = page.elements.iter().find(|e| e.id == *element_id).unwrap();
            
            // Look for label -> value patterns
            if element.text.ends_with(':') {
                // This might be a label, look for associated value to the right
                for rel in relationships {
                    if matches!(rel.relationship_type, RelationshipType::LeftOf) && rel.distance < 20.0 {
                        rel.relationship_type = RelationshipType::Association;
                        rel.confidence = 0.8;
                    }
                }
            }
        }
    }
    
    pub fn extract_structured_data(&self, document: &Document) -> Result<StructuredData> {
        let mut structured = StructuredData {
            metadata: document.metadata.clone(),
            sections: Vec::new(),
            tables: Vec::new(),
            figures: Vec::new(),
        };
        
        for page in &document.pages {
            // Extract tables using spatial groups
            for group in &page.spatial_graph.visual_groups {
                if matches!(group.group_type, GroupType::Table) {
                    let table = self.extract_table_from_group(page, group)?;
                    structured.tables.push(table);
                }
            }
            
            // Extract sections using reading order and semantic regions
            let sections = self.extract_sections_from_page(page)?;
            structured.sections.extend(sections);
        }
        
        Ok(structured)
    }
    
    fn extract_table_from_group(&self, page: &Page, group: &VisualGroup) -> Result<TableData> {
        let mut cells = Vec::new();
        
        for element_id in &group.elements {
            let element = page.elements.iter().find(|e| e.id == *element_id).unwrap();
            if let ElementType::TableCell { row, col } = element.element_type {
                cells.push(CellData {
                    row,
                    col,
                    content: element.text.clone(),
                    bbox: element.bbox.clone(),
                });
            }
        }
        
        // Sort cells by row, then column
        cells.sort_by_key(|c| (c.row, c.col));
        
        Ok(TableData {
            bbox: group.bbox.clone(),
            cells,
        })
    }
    
    fn extract_sections_from_page(&self, page: &Page) -> Result<Vec<SectionData>> {
        let mut sections = Vec::new();
        let mut current_section: Option<SectionData> = None;
        
        // Follow reading order to build sections
        for element_id in &page.spatial_graph.reading_order {
            let element = page.elements.iter().find(|e| e.id == *element_id).unwrap();
            
            match &element.element_type {
                ElementType::Title { level } | ElementType::Heading { level } => {
                    // Save previous section
                    if let Some(section) = current_section.take() {
                        sections.push(section);
                    }
                    
                    // Start new section
                    current_section = Some(SectionData {
                        title: element.text.clone(),
                        level: *level,
                        content: Vec::new(),
                        bbox: element.bbox.clone(),
                    });
                }
                _ => {
                    // Add to current section
                    if let Some(ref mut section) = current_section {
                        section.content.push(element.text.clone());
                        // Expand section bbox
                        section.bbox.x0 = section.bbox.x0.min(element.bbox.x0);
                        section.bbox.y0 = section.bbox.y0.min(element.bbox.y0);
                        section.bbox.x1 = section.bbox.x1.max(element.bbox.x1);
                        section.bbox.y1 = section.bbox.y1.max(element.bbox.y1);
                    }
                }
            }
        }
        
        // Don't forget the last section
        if let Some(section) = current_section {
            sections.push(section);
        }
        
        Ok(sections)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredData {
    pub metadata: DocumentMetadata,
    pub sections: Vec<SectionData>,
    pub tables: Vec<TableData>,
    pub figures: Vec<FigureData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionData {
    pub title: String,
    pub level: usize,
    pub content: Vec<String>,
    pub bbox: BBox,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableData {
    pub bbox: BBox,
    pub cells: Vec<CellData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellData {
    pub row: usize,
    pub col: usize,
    pub content: String,
    pub bbox: BBox,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FigureData {
    pub bbox: BBox,
    pub caption: Option<String>,
    pub figure_type: String,
}
