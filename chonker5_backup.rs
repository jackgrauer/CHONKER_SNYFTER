#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! fltk = { version = "1.4", features = ["fltk-bundled"] }
//! rfd = "0.15"
//! image = "0.25"
//! extractous = "0.3"
//! serde = { version = "1.0", features = ["derive"] }
//! serde_json = "1.0"
//! ```

use fltk::{
    app::{self, App, Scheme},
    button::Button,
    draw,
    enums::{Color, Event, Font, FrameType, Key},
    frame::Frame,
    group::{Flex, Group, Scroll},
    input::MultilineInput,
    prelude::*,
    text::{TextBuffer, TextDisplay},
    window::Window,
    widget::Widget,
    widget_extends,
    image as fltk_image,
};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::process::Command;
use std::fs;
use extractous::Extractor;
use serde::{Deserialize, Serialize};
use serde_json;

const WINDOW_WIDTH: i32 = 1200;
const WINDOW_HEIGHT: i32 = 800;
const TOP_BAR_HEIGHT: i32 = 60;
const LOG_HEIGHT: i32 = 100;

// Color scheme
const COLOR_TEAL: Color = Color::from_rgb(0x1A, 0xBC, 0x9C);
const COLOR_DARK_BG: Color = Color::from_rgb(0x52, 0x56, 0x59);
const COLOR_DARKER_BG: Color = Color::from_rgb(0x2D, 0x2F, 0x31);

// Ferrules JSON structures
#[derive(Debug, Deserialize, Serialize, Clone)]
struct FerrulesDocument {
    pages: Vec<FerrulesPage>,
    blocks: Vec<FerrulesBlock>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct FerrulesPage {
    id: i32,
    width: f64,
    height: f64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct FerrulesBlock {
    id: i32,
    pages_id: Vec<i32>,
    bbox: FerrulesBox,
    kind: FerrulesKind,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct FerrulesBox {
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
enum FerrulesKind {
    Structured { block_type: String, text: String },
    Text { text: String },
    Other(serde_json::Value),
}

// Table detection structures
#[derive(Debug, Clone)]
struct TableCell {
    block_idx: usize,
    text: String,
    bbox: FerrulesBox,
}

#[derive(Debug, Clone)]
struct TableRow {
    cells: Vec<TableCell>,
    y_center: f64,
}

#[derive(Debug, Clone)]
struct DetectedTable {
    rows: Vec<TableRow>,
    bbox: FerrulesBox, // Overall table boundaries
}

/* PRETTY VIEW REMOVED - Too broken
// Simple placeholder widget - Pretty view was removed because it was broken
#[derive(Debug, Clone)]
struct StructuredTextWidget {
    inner: Widget,
}
*/

impl StructuredTextWidget {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        let mut inner = Widget::default().with_pos(x, y).with_size(w, h);
        inner.set_frame(FrameType::FlatBox);
        inner.set_color(Color::White);
        
        let document = Rc::new(RefCell::new(None));
        let selected_block = Rc::new(RefCell::new(None));
        let scroll_offset = Rc::new(RefCell::new((0.0, 0.0)));
        let zoom = Rc::new(RefCell::new(1.0));
        let dragging = Rc::new(RefCell::new(None));
        
        let doc_clone = document.clone();
        let selected_clone = selected_block.clone();
        let scroll_clone = scroll_offset.clone();
        let zoom_clone = zoom.clone();
        
        inner.draw({
            let doc_clone = doc_clone.clone();
            let selected_clone = selected_clone.clone();
            let scroll_clone = scroll_clone.clone();
            let zoom_clone = zoom_clone.clone();
            move |widget| {
                draw::push_clip(widget.x(), widget.y(), widget.width(), widget.height());
                draw::draw_box(widget.frame(), widget.x(), widget.y(), widget.width(), widget.height(), widget.color());
                
                // Draw status indicator
                draw::set_draw_color(Color::from_rgb(100, 100, 100));
                draw::set_font(Font::Helvetica, 10);
                draw::draw_text("Custom Renderer Active", widget.x() + 5, widget.y() + 15);
                
                if let Some(ref doc) = *doc_clone.borrow() {
                    Self::draw_document(widget, doc, &selected_clone, &scroll_clone, &zoom_clone);
                } else {
                    draw::set_draw_color(Color::Black);
                    draw::set_font(Font::Helvetica, 14);
                    draw::draw_text("No structured data loaded", widget.x() + 10, widget.y() + 30);
                }
                
                draw::pop_clip();
            }
        });
        
        let doc_clone = document.clone();
        let selected_clone = selected_block.clone();
        let scroll_clone = scroll_offset.clone();
        let zoom_clone = zoom.clone();
        let dragging_clone = dragging.clone();
        
        // Comment out interaction for now - focus on rendering
        inner.handle({
            move |widget, event| {
                match event {
                    Event::MouseWheel => {
                        let dy = app::event_dy();
                        let mut offset = scroll_clone.borrow_mut();
                        offset.1 += match dy {
                            app::MouseWheel::None => 0.0,
                            app::MouseWheel::Down => 20.0,
                            app::MouseWheel::Up => -20.0,
                            app::MouseWheel::Left => 0.0,
                            app::MouseWheel::Right => 0.0,
                        };
                        widget.redraw();
                        return true;
                    }
                    Event::KeyDown => {
                        let key = app::event_key();
                        if key == Key::from_char('+') || key == Key::from_char('=') {
                            let mut zoom = zoom_clone.borrow_mut();
                            *zoom = (*zoom * 1.1).min(3.0);
                            widget.redraw();
                            return true;
                        } else if key == Key::from_char('-') {
                            let mut zoom = zoom_clone.borrow_mut();
                            *zoom = (*zoom / 1.1).max(0.5);
                            widget.redraw();
                            return true;
                        }
                    }
                    _ => {}
                }
                false
            }
        });
        
        Self {
            inner,
            document,
            selected_block,
            scroll_offset,
            zoom,
            dragging,
        }
    }
    
    pub fn set_document(&mut self, doc: FerrulesDocument) {
        *self.document.borrow_mut() = Some(doc);
        *self.selected_block.borrow_mut() = None;
        *self.scroll_offset.borrow_mut() = (0.0, 0.0);
        self.inner.redraw();
    }
    
    fn draw_document(
        widget: &Widget,
        doc: &FerrulesDocument,
        selected: &Rc<RefCell<Option<usize>>>,
        scroll: &Rc<RefCell<(f64, f64)>>,
        zoom: &Rc<RefCell<f32>>,
    ) {
        let (_scroll_x, scroll_y) = *scroll.borrow();
        let zoom_factor = *zoom.borrow();
        let selected_idx = *selected.borrow();
        
        // Calculate total document height for all pages
        let mut _total_height = 0.0;
        let page_gap = 20.0;
        
        for page in &doc.pages {
            _total_height += page.height + page_gap;
        }
        
        // Draw each page
        let mut current_y = widget.y() as f64 + scroll_y + 30.0;
        
        // Update status to show we're in facsimile mode
        draw::set_draw_color(Color::from_rgb(0, 150, 0));
        draw::set_font(Font::Helvetica, 10);
        draw::draw_text("🔧 Custom Renderer - True Facsimile Mode", widget.x() + 5, widget.y() + 15);
        
        for (page_idx, page) in doc.pages.iter().enumerate() {
            // Skip if page is above viewport
            if current_y + page.height * (zoom_factor as f64) < widget.y() as f64 {
                current_y += page.height * zoom_factor as f64 + page_gap;
                continue;
            }
            
            // Stop if page is below viewport
            if current_y > (widget.y() + widget.height()) as f64 {
                break;
            }
            
            // Draw page background (white like actual PDF)
            let page_x = widget.x() as f64 + (widget.width() as f64 - page.width * zoom_factor as f64) / 2.0;
            draw::set_draw_color(Color::White);
            draw::draw_rectf(page_x as i32, current_y as i32, 
                           (page.width * zoom_factor as f64) as i32,
                           (page.height * zoom_factor as f64) as i32);
            
            // Draw page border
            draw::set_draw_color(Color::from_rgb(200, 200, 200));
            draw::draw_rect(page_x as i32, current_y as i32,
                          (page.width * zoom_factor as f64) as i32,
                          (page.height * zoom_factor as f64) as i32);
            
            // Detect and visualize tables for this page (cached for performance)
            // TODO: Cache this to avoid repeated detection
            let page_tables: Vec<DetectedTable> = Vec::new(); // detect_tables(&doc.blocks, page.id);
            
            // Draw table regions first (as background)
            for (table_idx, table) in page_tables.iter().enumerate() {
                let table_x = (page_x + table.bbox.x0 * zoom_factor as f64) as i32;
                let table_y = (current_y + table.bbox.y0 * zoom_factor as f64) as i32;
                let table_width = ((table.bbox.x1 - table.bbox.x0) * zoom_factor as f64) as i32;
                let table_height = ((table.bbox.y1 - table.bbox.y0) * zoom_factor as f64) as i32;
                
                // Draw table background
                draw::set_draw_color(Color::from_rgb(240, 240, 255)); // Light blue
                draw::draw_rectf(table_x, table_y, table_width, table_height);
                
                // Draw table border
                draw::set_draw_color(Color::from_rgb(100, 100, 200));
                draw::set_line_style(draw::LineStyle::Solid, 2);
                draw::draw_rect(table_x, table_y, table_width, table_height);
                
                // Draw table label
                draw::set_font(Font::HelveticaBold, 12);
                draw::set_draw_color(Color::from_rgb(50, 50, 150));
                draw::draw_text(&format!("📊 Table {}", table_idx + 1), table_x + 5, table_y - 5);
            }
            
            // Draw blocks for this page at their EXACT PDF positions
            for (block_idx, block) in doc.blocks.iter().enumerate() {
                if !block.pages_id.contains(&page.id) {
                    continue;
                }
                
                // Use exact coordinates from PDF
                let x = page_x + block.bbox.x0 * zoom_factor as f64;
                let y = current_y + block.bbox.y0 * zoom_factor as f64;
                let w = (block.bbox.x1 - block.bbox.x0) * zoom_factor as f64;
                let h = (block.bbox.y1 - block.bbox.y0) * zoom_factor as f64;
                
                // Only highlight selected blocks, don't draw backgrounds
                if Some(block_idx) == selected_idx {
                    draw::set_draw_color(Color::from_rgb(255, 255, 200));
                    draw::draw_rectf(x as i32, y as i32, w as i32, h as i32);
                }
                
                // Very faint bounding box for debugging
                draw::set_draw_color(Color::from_rgb(230, 230, 230));
                draw::set_line_style(draw::LineStyle::Dash, 1);
                draw::draw_rect(x as i32, y as i32, w as i32, h as i32);
                draw::set_line_style(draw::LineStyle::Solid, 1);
                
                // Get text content
                let text_content = match &block.kind {
                    FerrulesKind::Structured { text, block_type } => Some((text, block_type.as_str())),
                    FerrulesKind::Text { text } => Some((text, "Text")),
                    _ => None,
                };
                
                if let Some((text, block_type)) = text_content {
                    // Set color and font based on block type
                    let (font, font_size, color) = match block_type {
                        "Title" => {
                            (Font::HelveticaBold, ((h * 0.7) as i32).clamp(16, 24), Color::from_rgb(0, 51, 102))
                        },
                        "Header" => {
                            (Font::HelveticaBold, ((h * 0.7) as i32).clamp(14, 18), Color::from_rgb(51, 51, 51))
                        },
                        "Footer" => {
                            (Font::HelveticaItalic, ((h * 0.5) as i32).clamp(8, 10), Color::from_rgb(128, 128, 128))
                        },
                        "TextBlock" => {
                            // Check for emphasis patterns in text
                            if text.contains("Table") || text.contains("TABLE") {
                                (Font::HelveticaBold, ((h * 0.6) as i32).clamp(11, 13), Color::from_rgb(0, 0, 150))
                            } else {
                                (Font::Helvetica, ((h * 0.6) as i32).clamp(10, 12), Color::Black)
                            }
                        },
                        _ => {
                            (Font::Helvetica, ((h * 0.6) as i32).clamp(9, 12), Color::Black)
                        },
                    };
                    
                    draw::set_draw_color(color);
                    draw::set_font(font, font_size);
                    
                    // Calculate approximate characters per line
                    let char_width = font_size as f64 * 0.6;
                    let chars_per_line = (w / char_width).max(1.0) as usize;
                    
                    // Word wrap the text, preserving line breaks
                    let mut wrapped_lines = Vec::new();
                    
                    // First split by newlines to preserve paragraph breaks
                    for paragraph in text.split('\n') {
                        if paragraph.trim().is_empty() {
                            wrapped_lines.push(String::new()); // Preserve empty lines
                            continue;
                        }
                        
                        let words: Vec<&str> = paragraph.split_whitespace().collect();
                        let mut current_line = String::new();
                        
                        for word in words {
                            if current_line.is_empty() {
                                current_line = word.to_string();
                            } else if current_line.len() + word.len() + 1 <= chars_per_line {
                                current_line.push(' ');
                                current_line.push_str(word);
                            } else {
                                wrapped_lines.push(current_line.clone());
                                current_line = word.to_string();
                            }
                        }
                        if !current_line.is_empty() {
                            wrapped_lines.push(current_line);
                        }
                    }
                    
                    // Draw each line
                    let line_height = font_size as f64 * 1.2;
                    for (i, line) in wrapped_lines.iter().enumerate() {
                        let text_y = y + font_size as f64 + (i as f64 * line_height);
                        if text_y < y + h {
                            draw::draw_text(line, x as i32 + 2, text_y as i32);
                        }
                    }
                } else {
                    // No text content - show what we got instead
                    draw::set_draw_color(Color::Red);
                    draw::set_font(Font::Helvetica, 10);
                    draw::draw_text("NO TEXT DATA", x as i32 + 5, y as i32 + 15);
                }
            }
            
            // Draw page number at bottom
            draw::set_draw_color(Color::from_rgb(100, 100, 100));
            draw::set_font(Font::HelveticaBold, 11);
            draw::draw_text(
                &format!("— Page {} —", page_idx + 1),
                (page_x + page.width * zoom_factor as f64 / 2.0 - 30.0) as i32,
                (current_y + page.height * zoom_factor as f64 + 15.0) as i32
            );
            
            // Draw page separator line
            if page_idx < doc.pages.len() - 1 {
                draw::set_draw_color(Color::from_rgb(200, 200, 200));
                draw::set_line_style(draw::LineStyle::Solid, 2);
                let separator_y = (current_y + page.height * zoom_factor as f64 + page_gap / 2.0) as i32;
                draw::draw_line(
                    (page_x - 20.0) as i32,
                    separator_y,
                    (page_x + page.width * zoom_factor as f64 + 20.0) as i32,
                    separator_y
                );
            }
            
            current_y += page.height * zoom_factor as f64 + page_gap;
        }
    }
    
    fn handle_click(
        widget: &Widget,
        mouse_x: i32,
        mouse_y: i32,
        doc: &Rc<RefCell<Option<FerrulesDocument>>>,
        selected: &Rc<RefCell<Option<usize>>>,
        scroll: &Rc<RefCell<(f64, f64)>>,
    ) {
        if let Some(ref doc) = *doc.borrow() {
            let (_scroll_x, scroll_y) = *scroll.borrow();
            
            // Find which block was clicked
            let mut current_y = widget.y() as f64 - scroll_y + 10.0;
            let page_gap = 20.0;
            
            for (_page_idx, page) in doc.pages.iter().enumerate() {
                let page_x = widget.x() as f64 + (widget.width() as f64 - page.width) / 2.0;
                
                for (block_idx, block) in doc.blocks.iter().enumerate() {
                    if !block.pages_id.contains(&page.id) {
                        continue;
                    }
                    
                    let x = page_x + block.bbox.x0;
                    let y = current_y + block.bbox.y0;
                    let w = block.bbox.x1 - block.bbox.x0;
                    let h = block.bbox.y1 - block.bbox.y0;
                    
                    if mouse_x >= x as i32 && mouse_x <= (x + w) as i32 &&
                       mouse_y >= y as i32 && mouse_y <= (y + h) as i32 {
                        *selected.borrow_mut() = Some(block_idx);
                        return;
                    }
                }
                
                current_y += page.height + page_gap;
            }
            
            // No block clicked, deselect
            *selected.borrow_mut() = None;
        }
    }
}

widget_extends!(StructuredTextWidget, Widget, inner);

// Advanced table detection from ferrules blocks
fn detect_tables(blocks: &[FerrulesBlock], page_id: i32) -> Vec<DetectedTable> {
    let mut tables = Vec::new();
    
    // Filter blocks for this page
    let mut page_blocks: Vec<(usize, &FerrulesBlock)> = blocks
        .iter()
        .enumerate()
        .filter(|(_, b)| b.pages_id.contains(&page_id))
        .collect();
    
    println!("🔍 Table detection for page {}: {} blocks", page_id + 1, page_blocks.len());
    
    // Sort by Y then X for consistent processing
    page_blocks.sort_by(|a, b| {
        let y_cmp = a.1.bbox.y0.partial_cmp(&b.1.bbox.y0).unwrap();
        if y_cmp == std::cmp::Ordering::Equal {
            a.1.bbox.x0.partial_cmp(&b.1.bbox.x0).unwrap()
        } else {
            y_cmp
        }
    });
    
    // Phase 1: Cluster blocks into rows based on Y-coordinate alignment
    let mut rows: Vec<Vec<(usize, &FerrulesBlock)>> = Vec::new();
    let row_tolerance = 3.0; // Tighter tolerance for better accuracy
    
    for (idx, block) in &page_blocks {
        let y_center = (block.bbox.y0 + block.bbox.y1) / 2.0;
        let block_height = block.bbox.y1 - block.bbox.y0;
        
        // Find the best matching row
        let mut best_row = None;
        let mut best_overlap = 0.0;
        
        for (row_idx, row) in rows.iter().enumerate() {
            if let Some((_, first_block)) = row.first() {
                let row_y0 = first_block.bbox.y0;
                let row_y1 = first_block.bbox.y1;
                
                // Calculate vertical overlap
                let overlap_start = block.bbox.y0.max(row_y0);
                let overlap_end = block.bbox.y1.min(row_y1);
                let overlap = (overlap_end - overlap_start).max(0.0);
                let overlap_ratio = overlap / block_height;
                
                // If blocks overlap significantly (>70%), they're in the same row
                if overlap_ratio > 0.7 && overlap_ratio > best_overlap {
                    best_row = Some(row_idx);
                    best_overlap = overlap_ratio;
                }
            }
        }
        
        if let Some(row_idx) = best_row {
            rows[row_idx].push((*idx, block));
        } else {
            // Create new row
            rows.push(vec![(*idx, block)]);
        }
    }
    
    // Sort rows by Y coordinate
    rows.sort_by(|a, b| {
        let y_a = a[0].1.bbox.y0;
        let y_b = b[0].1.bbox.y0;
        y_a.partial_cmp(&y_b).unwrap()
    });
    
    println!("  📊 Clustered into {} rows", rows.len());
    for (i, row) in rows.iter().take(5).enumerate() {
        println!("    Row {}: {} blocks at Y={:.0}", i, row.len(), row[0].1.bbox.y0);
    }
    
    // Sort blocks within each row by X coordinate
    for row in &mut rows {
        row.sort_by(|a, b| a.1.bbox.x0.partial_cmp(&b.1.bbox.x0).unwrap());
    }
    
    // Phase 2: Detect column structure
    #[derive(Debug)]
    struct ColumnPattern {
        x_positions: Vec<f64>,
        consistency_score: f64,
    }
    
    // Analyze column patterns in multi-cell rows
    let mut column_patterns: Vec<ColumnPattern> = Vec::new();
    
    for row in &rows {
        if row.len() >= 2 {
            let x_positions: Vec<f64> = row.iter().map(|(_, b)| b.bbox.x0).collect();
            
            // Check if this pattern matches any existing pattern
            let mut matched = false;
            for pattern in &mut column_patterns {
                if pattern.x_positions.len() == x_positions.len() {
                    let mut all_match = true;
                    let tolerance = 15.0;
                    
                    for (i, &x) in x_positions.iter().enumerate() {
                        if (x - pattern.x_positions[i]).abs() > tolerance {
                            all_match = false;
                            break;
                        }
                    }
                    
                    if all_match {
                        // Update pattern with average positions
                        for (i, &x) in x_positions.iter().enumerate() {
                            pattern.x_positions[i] = (pattern.x_positions[i] + x) / 2.0;
                        }
                        pattern.consistency_score += 1.0;
                        matched = true;
                        break;
                    }
                }
            }
            
            if !matched {
                column_patterns.push(ColumnPattern {
                    x_positions,
                    consistency_score: 1.0,
                });
            }
        }
    }
    
    // Find the most consistent column pattern
    column_patterns.sort_by(|a, b| b.consistency_score.partial_cmp(&a.consistency_score).unwrap());
    
    println!("  🏛️ Found {} column patterns", column_patterns.len());
    for (i, pattern) in column_patterns.iter().take(3).enumerate() {
        println!("    Pattern {}: {} columns, score={:.1}, X positions: {:?}", 
            i, pattern.x_positions.len(), pattern.consistency_score,
            pattern.x_positions.iter().map(|x| format!("{:.0}", x)).collect::<Vec<_>>());
    }
    
    // Phase 3: Identify table regions using the column pattern
    if let Some(best_pattern) = column_patterns.first() {
        if best_pattern.consistency_score >= 2.0 {
            // We have a consistent column pattern
            let mut i = 0;
            while i < rows.len() {
                // Look for consecutive rows that match the pattern
                let mut table_rows = Vec::new();
                let mut j = i;
                
                while j < rows.len() {
                    let row = &rows[j];
                    
                    // Check if this row matches the column pattern
                    let mut matches_pattern = false;
                    
                    if row.len() == best_pattern.x_positions.len() {
                        matches_pattern = true;
                        let tolerance = 20.0;
                        
                        for (k, (_, block)) in row.iter().enumerate() {
                            if (block.bbox.x0 - best_pattern.x_positions[k]).abs() > tolerance {
                                matches_pattern = false;
                                break;
                            }
                        }
                    } else if row.len() == 1 {
                        // Single cell row might be a header or merged cell
                        // Check if it spans the table width
                        if let Some((_, block)) = row.first() {
                            let table_left = best_pattern.x_positions[0] - 10.0;
                            let table_right = if let Some((_, last_block)) = rows.iter()
                                .find(|r| r.len() == best_pattern.x_positions.len())
                                .and_then(|r| r.last()) {
                                last_block.bbox.x1 + 10.0
                            } else {
                                best_pattern.x_positions.last().unwrap() + 100.0
                            };
                            
                            if block.bbox.x0 >= table_left && block.bbox.x1 <= table_right {
                                matches_pattern = true; // Include as potential header
                            }
                        }
                    }
                    
                    if matches_pattern {
                        table_rows.push(rows[j].clone());
                        j += 1;
                    } else if !table_rows.is_empty() {
                        // End of table
                        break;
                    } else {
                        // Haven't found table start yet
                        j += 1;
                        i = j;
                    }
                }
                
                // Create table if we found at least 2 rows
                if table_rows.len() >= 2 {
                    let mut detected_table = DetectedTable {
                        rows: Vec::new(),
                        bbox: FerrulesBox {
                            x0: f64::MAX,
                            y0: f64::MAX,
                            x1: f64::MIN,
                            y1: f64::MIN,
                        },
                    };
                    
                    for row_blocks in table_rows {
                        let y_center = if let Some((_, first)) = row_blocks.first() {
                            (first.bbox.y0 + first.bbox.y1) / 2.0
                        } else {
                            0.0
                        };
                        
                        let mut table_row = TableRow {
                            cells: Vec::new(),
                            y_center,
                        };
                        
                        for (idx, block) in row_blocks {
                            // Update table bounds
                            detected_table.bbox.x0 = detected_table.bbox.x0.min(block.bbox.x0);
                            detected_table.bbox.y0 = detected_table.bbox.y0.min(block.bbox.y0);
                            detected_table.bbox.x1 = detected_table.bbox.x1.max(block.bbox.x1);
                            detected_table.bbox.y1 = detected_table.bbox.y1.max(block.bbox.y1);
                            
                            // Extract text
                            let text = match &block.kind {
                                FerrulesKind::Structured { text, .. } => text.clone(),
                                FerrulesKind::Text { text } => text.clone(),
                                _ => String::new(),
                            };
                            
                            table_row.cells.push(TableCell {
                                block_idx: idx,
                                text,
                                bbox: block.bbox.clone(),
                            });
                        }
                        
                        detected_table.rows.push(table_row);
                    }
                    
                    tables.push(detected_table);
                    i = j;
                } else {
                    i += 1;
                }
            }
        }
    }
    
    // Phase 4: Try alternative detection for missed tables
    // Look for regions with high density of small text blocks in grid-like arrangement
    if tables.is_empty() && rows.len() > 5 {
        // Simple heuristic: find sequences of rows with 2+ blocks
        let mut consecutive_multi_cell_rows = 0;
        let mut table_start = 0;
        
        for (i, row) in rows.iter().enumerate() {
            if row.len() >= 2 {
                if consecutive_multi_cell_rows == 0 {
                    table_start = i;
                }
                consecutive_multi_cell_rows += 1;
            } else {
                if consecutive_multi_cell_rows >= 3 {
                    // Found a potential table
                    let mut detected_table = DetectedTable {
                        rows: Vec::new(),
                        bbox: FerrulesBox {
                            x0: f64::MAX,
                            y0: f64::MAX,
                            x1: f64::MIN,
                            y1: f64::MIN,
                        },
                    };
                    
                    for j in table_start..i {
                        if let Some(row) = rows.get(j) {
                            let y_center = if let Some((_, first)) = row.first() {
                                (first.bbox.y0 + first.bbox.y1) / 2.0
                            } else {
                                0.0
                            };
                            
                            let mut table_row = TableRow {
                                cells: Vec::new(),
                                y_center,
                            };
                            
                            for (idx, block) in row {
                                detected_table.bbox.x0 = detected_table.bbox.x0.min(block.bbox.x0);
                                detected_table.bbox.y0 = detected_table.bbox.y0.min(block.bbox.y0);
                                detected_table.bbox.x1 = detected_table.bbox.x1.max(block.bbox.x1);
                                detected_table.bbox.y1 = detected_table.bbox.y1.max(block.bbox.y1);
                                
                                let text = match &block.kind {
                                    FerrulesKind::Structured { text, .. } => text.clone(),
                                    FerrulesKind::Text { text } => text.clone(),
                                    _ => String::new(),
                                };
                                
                                table_row.cells.push(TableCell {
                                    block_idx: *idx,
                                    text,
                                    bbox: block.bbox.clone(),
                                });
                            }
                            
                            detected_table.rows.push(table_row);
                        }
                    }
                    
                    if detected_table.rows.len() >= 2 {
                        tables.push(detected_table);
                    }
                }
                consecutive_multi_cell_rows = 0;
            }
        }
    }
    
    println!("  ✅ Detected {} tables on page {}", tables.len(), page_id + 1);
    for (i, table) in tables.iter().enumerate() {
        println!("    Table {}: {} rows, bbox: X{:.0}-{:.0} Y{:.0}-{:.0}", 
            i, table.rows.len(), table.bbox.x0, table.bbox.x1, table.bbox.y0, table.bbox.y1);
    }
    
    tables
}

impl StructuredTextWidget {
    
    pub fn clear_document(&mut self) {
        *self.document.borrow_mut() = None;
        *self.selected_block.borrow_mut() = None;
        self.redraw();
    }
    
    pub fn edit_selected_block(&mut self) {
        let block_idx = match *self.selected_block.borrow() {
            Some(idx) => idx,
            None => return,
        };
        
        if let Some(ref mut doc) = *self.document.borrow_mut() {
            if let Some(block) = doc.blocks.get_mut(block_idx) {
                let text_mut = match &mut block.kind {
                    FerrulesKind::Structured { ref mut text, .. } => Some(text),
                    FerrulesKind::Text { ref mut text } => Some(text),
                    _ => None,
                };
                
                if let Some(text) = text_mut {
                    // Create edit dialog
                    let mut dialog_window = Window::default()
                        .with_size(500, 300)
                        .with_label("Edit Text Block");
                    dialog_window.make_modal(true);
                    
                    let mut input = MultilineInput::default()
                        .with_size(480, 250)
                        .with_pos(10, 10);
                    input.set_value(text);
                    
                    let mut ok_btn = Button::default()
                        .with_size(80, 30)
                        .with_pos(320, 265)
                        .with_label("OK");
                    
                    let mut cancel_btn = Button::default()
                        .with_size(80, 30)
                        .with_pos(410, 265)
                        .with_label("Cancel");
                    
                    dialog_window.end();
                    dialog_window.show();
                    
                    let _text_clone = text.clone();
                    let (s, r) = app::channel::<bool>();
                    
                    ok_btn.set_callback(move |_| {
                        s.send(true);
                    });
                    
                    cancel_btn.set_callback(move |_| {
                        s.send(false);
                    });
                    
                    while dialog_window.shown() {
                        app::wait();
                        if let Some(msg) = r.recv() {
                            if msg {
                                *text = input.value();
                            }
                            dialog_window.hide();
                        }
                    }
                }
            }
        }
        self.redraw();
    }
}

struct Chonker5App {
    app: App,
    window: Window,
    pdf_frame: Frame,
    status_label: Frame,
    zoom_label: Frame,
    page_label: Frame,
    log_display: TextDisplay,
    log_buffer: TextBuffer,
    prev_btn: Button,
    next_btn: Button,
    extract_btn: Button,
    structured_btn: Button,
    compare_btn: Button,
    extracted_text_display: TextDisplay,
    extracted_text_buffer: TextBuffer,
    structured_view: StructuredTextWidget,
    structured_html_content: String,
    structured_json_data: Option<FerrulesDocument>,
    compare_mode: bool,
    
    // PDF state
    pdf_path: Option<PathBuf>,
    current_page: usize,
    total_pages: usize,
    zoom_level: f32,
    
}

impl Chonker5App {
    fn new() -> Rc<RefCell<Self>> {
        let app = App::default().with_scheme(Scheme::Gtk);
        
        // Create main window
        let mut window = Window::new(100, 100, WINDOW_WIDTH, WINDOW_HEIGHT, "🐹 CHONKER 5 - PDF Viewer");
        window.set_color(COLOR_DARK_BG);
        window.make_resizable(true);
        
        // Create main vertical layout
        let mut main_flex = Flex::default()
            .with_size(WINDOW_WIDTH, WINDOW_HEIGHT)
            .column();
        
        // Top bar
        let mut top_bar = fltk::group::Group::default()
            .with_size(WINDOW_WIDTH, TOP_BAR_HEIGHT);
        top_bar.set_color(COLOR_TEAL);
        top_bar.set_frame(FrameType::FlatBox);
        
        // Position buttons manually with explicit positions
        let mut x_pos = 10;
        let y_pos = 10;
        
        let mut open_btn = Button::default()
            .with_pos(x_pos, y_pos)
            .with_size(100, 40)
            .with_label("Open");
        open_btn.set_color(Color::White);
        open_btn.set_label_color(Color::Black);
        open_btn.set_frame(FrameType::UpBox);
        open_btn.set_label_size(14);
        
        x_pos += 110;
        let mut prev_btn = Button::default()
            .with_pos(x_pos, y_pos)
            .with_size(80, 40)
            .with_label("◀ Prev");
        prev_btn.set_color(Color::from_rgb(0xDD, 0xDD, 0xDD));
        prev_btn.set_label_color(Color::from_rgb(0x66, 0x66, 0x66));
        prev_btn.set_frame(FrameType::UpBox);
        prev_btn.set_label_size(14);
        prev_btn.deactivate();
        
        x_pos += 90;
        let mut next_btn = Button::default()
            .with_pos(x_pos, y_pos)
            .with_size(80, 40)
            .with_label("Next ▶");
        next_btn.set_color(Color::from_rgb(0xDD, 0xDD, 0xDD));
        next_btn.set_label_color(Color::from_rgb(0x66, 0x66, 0x66));
        next_btn.set_frame(FrameType::UpBox);
        next_btn.set_label_size(14);
        next_btn.deactivate();
        
        x_pos += 90;
        let mut zoom_in_btn = Button::default()
            .with_pos(x_pos, y_pos)
            .with_size(100, 40)
            .with_label("Zoom In +");
        zoom_in_btn.set_color(Color::White);
        zoom_in_btn.set_label_color(Color::Black);
        zoom_in_btn.set_frame(FrameType::UpBox);
        zoom_in_btn.set_label_size(14);
        
        x_pos += 110;
        let mut zoom_out_btn = Button::default()
            .with_pos(x_pos, y_pos)
            .with_size(100, 40)
            .with_label("Zoom Out -");
        zoom_out_btn.set_color(Color::White);
        zoom_out_btn.set_label_color(Color::Black);
        zoom_out_btn.set_frame(FrameType::UpBox);
        zoom_out_btn.set_label_size(14);
        
        x_pos += 110;
        let mut fit_width_btn = Button::default()
            .with_pos(x_pos, y_pos)
            .with_size(100, 40)
            .with_label("Fit Width");
        fit_width_btn.set_color(Color::White);
        fit_width_btn.set_label_color(Color::Black);
        fit_width_btn.set_frame(FrameType::UpBox);
        fit_width_btn.set_label_size(14);
        
        x_pos += 110;
        let mut extract_btn = Button::default()
            .with_pos(x_pos, y_pos)
            .with_size(120, 40)
            .with_label("📋 Raw JSON");
        extract_btn.set_color(Color::from_rgb(0x00, 0x8C, 0xBA)); // Blue color for distinction
        extract_btn.set_label_color(Color::White);
        extract_btn.set_frame(FrameType::UpBox);
        extract_btn.set_label_size(14);
        extract_btn.deactivate(); // Start disabled until PDF is loaded
        
        x_pos += 130;
        let mut structured_btn = Button::default()
            .with_pos(x_pos, y_pos)
            .with_size(140, 40)
            .with_label("✨ Pretty View");
        structured_btn.set_color(Color::from_rgb(0x00, 0x8C, 0x3A)); // Green color for distinction
        structured_btn.set_label_color(Color::White);
        structured_btn.set_frame(FrameType::UpBox);
        structured_btn.set_label_size(14);
        structured_btn.deactivate(); // Start disabled until PDF is loaded
        
        x_pos += 150;
        let mut compare_btn = Button::default()
            .with_pos(x_pos, y_pos)
            .with_size(100, 40)
            .with_label("Compare");
        compare_btn.set_color(Color::from_rgb(0xFF, 0x85, 0x00)); // Orange color
        compare_btn.set_label_color(Color::White);
        compare_btn.set_frame(FrameType::UpBox);
        compare_btn.set_label_size(14);
        compare_btn.deactivate(); // Start disabled until extraction is done
        
        x_pos += 110;
        let mut status_label = Frame::default()
            .with_pos(x_pos, y_pos)
            .with_size(300, 40)
            .with_label("Ready! Click 'Open' to load a PDF");
        status_label.set_label_color(Color::White);
        
        x_pos += 310;
        let mut zoom_label = Frame::default()
            .with_pos(x_pos, y_pos)
            .with_size(100, 40)
            .with_label("Zoom: 100%");
        zoom_label.set_label_color(Color::White);
        
        x_pos += 110;
        let mut page_label = Frame::default()
            .with_pos(x_pos, y_pos)
            .with_size(100, 40)
            .with_label("Page: 0/0");
        page_label.set_label_color(Color::White);
        
        top_bar.end();
        top_bar.redraw();
        main_flex.fixed(&mut top_bar, TOP_BAR_HEIGHT);
        
        // Create horizontal split for PDF and text panels
        let content_flex = Flex::default()
            .with_size(WINDOW_WIDTH, WINDOW_HEIGHT - TOP_BAR_HEIGHT - LOG_HEIGHT)
            .row();
        
        // Left pane: PDF viewing area with scroll
        let mut pdf_scroll = Scroll::default()
            .with_size(WINDOW_WIDTH / 2, WINDOW_HEIGHT - TOP_BAR_HEIGHT - LOG_HEIGHT);
        pdf_scroll.set_color(COLOR_DARK_BG);
        
        let mut pdf_frame = Frame::default()
            .with_size(WINDOW_WIDTH / 2 - 20, 1000);
        pdf_frame.set_frame(FrameType::FlatBox);
        pdf_frame.set_color(Color::White);
        pdf_frame.set_label("Click 'Open' to load a PDF");
        pdf_frame.set_label_color(Color::Black);
        
        pdf_scroll.end();
        
        // Right pane: Create a group to hold both text display and structured view
        let mut right_group = Group::default()
            .with_size(WINDOW_WIDTH / 2, WINDOW_HEIGHT - TOP_BAR_HEIGHT - LOG_HEIGHT);
        right_group.set_color(COLOR_DARKER_BG);
        
        // Text display for basic extraction
        let mut extracted_text_display = TextDisplay::default()
            .with_pos(right_group.x(), right_group.y())
            .with_size(right_group.w(), right_group.h());
        extracted_text_display.set_color(COLOR_DARKER_BG);
        extracted_text_display.set_text_color(Color::White);
        extracted_text_display.set_text_font(Font::Helvetica);
        extracted_text_display.set_text_size(14);
        extracted_text_display.set_frame(FrameType::FlatBox);
        extracted_text_display.wrap_mode(fltk::text::WrapMode::AtBounds, 0);
        
        let mut extracted_text_buffer = TextBuffer::default();
        extracted_text_buffer.set_text("PDF text will appear here after clicking 'Extract Text' button...");
        extracted_text_display.set_buffer(extracted_text_buffer.clone());
        extracted_text_display.show();  // Start with text display visible
        
        // Structured view with custom widget for ferrules JSON rendering
        let mut structured_view = StructuredTextWidget::new(
            right_group.x(), right_group.y(),
            right_group.w(), right_group.h()
        );
        structured_view.set_frame(FrameType::FlatBox);
        structured_view.set_color(Color::from_rgb(240, 240, 240));
        structured_view.hide();  // Start with structured view hidden
        
        right_group.end();
        
        content_flex.end();
        
        // Log area
        let mut log_display = TextDisplay::default()
            .with_size(WINDOW_WIDTH, LOG_HEIGHT);
        log_display.set_color(COLOR_DARKER_BG);
        log_display.set_text_color(COLOR_TEAL);
        log_display.set_text_font(Font::Courier);
        log_display.set_text_size(11);
        log_display.set_frame(FrameType::DownBox);
        
        let mut log_buffer = TextBuffer::default();
        log_buffer.append("🐹 CHONKER 5 Ready!\n");
        log_display.set_buffer(log_buffer.clone());
        
        main_flex.fixed(&mut log_display, LOG_HEIGHT);
        main_flex.end();
        
        window.resizable(&window);
        window.end();
        window.show();
        
        // Force redraw of all widgets
        window.redraw();
        app::redraw();
        
        log_buffer.append("🐹 CHONKER 5 Ready!\n");
        log_buffer.append("📌 Using MuPDF for PDF rendering + Extractous/Ferrules for text extraction\n");
        log_buffer.append("📌 Keyboard shortcuts: Cmd+O (Open), Cmd+P (Extract Text), ←/→ (Navigate), +/- (Zoom), F (Fit width)\n");
        log_buffer.append("📌 Extract Text: Basic text extraction | Structured Data: Perfect layout reconstruction\n");
        
        let app_state = Rc::new(RefCell::new(Self {
            app,
            window: window.clone(),
            pdf_frame,
            status_label,
            zoom_label,
            page_label,
            log_display,
            log_buffer,
            prev_btn: prev_btn.clone(),
            next_btn: next_btn.clone(),
            extract_btn: extract_btn.clone(),
            structured_btn: structured_btn.clone(),
            compare_btn: compare_btn.clone(),
            extracted_text_display: extracted_text_display.clone(),
            extracted_text_buffer,
            structured_view: structured_view.clone(),
            structured_html_content: String::new(),
            structured_json_data: None,
            compare_mode: false,
            pdf_path: None,
            current_page: 0,
            total_pages: 0,
            zoom_level: 1.0,
        }));
        
        // Set up event handlers
        
        // Open button
        {
            let state = app_state.clone();
            open_btn.set_callback(move |_| {
                state.borrow_mut().open_file();
            });
        }
        
        // Navigation buttons
        {
            let state = app_state.clone();
            prev_btn.set_callback(move |_| {
                let mut state_ref = state.borrow_mut();
                
                // Always navigate PDF pages since structured view shows entire document
                state_ref.prev_page();
            });
        }
        
        {
            let state = app_state.clone();
            next_btn.set_callback(move |_| {
                let mut state_ref = state.borrow_mut();
                
                // Always navigate PDF pages since structured view shows entire document
                state_ref.next_page();
            });
        }
        
        // Zoom buttons
        {
            let state = app_state.clone();
            zoom_in_btn.set_callback(move |_| {
                state.borrow_mut().zoom_in();
            });
        }
        
        {
            let state = app_state.clone();
            zoom_out_btn.set_callback(move |_| {
                state.borrow_mut().zoom_out();
            });
        }
        
        {
            let state = app_state.clone();
            fit_width_btn.set_callback(move |_| {
                state.borrow_mut().fit_to_width();
            });
        }
        
        // Extract text button
        {
            let state = app_state.clone();
            extract_btn.set_callback(move |_| {
                state.borrow_mut().process_pdf();
            });
        }
        
        // Structured data button
        {
            let state = app_state.clone();
            structured_btn.set_callback(move |_| {
                state.borrow_mut().extract_structured_data();
            });
        }
        
        // Compare button
        {
            let state = app_state.clone();
            compare_btn.set_callback(move |_| {
                state.borrow_mut().toggle_compare_mode();
            });
        }
        
        
        // Remove focus tracking event handlers to avoid borrow checker issues
        // Focus will be determined by mouse position when needed
        
        // Make window respond to close events
        window.set_callback(|_| {
            if app::event() == Event::Close {
                app::quit();
            }
        });
        
        // Keyboard shortcuts and window events
        {
            let state = app_state.clone();
            let mut win_clone = window.clone();
            window.handle(move |_, ev| match ev {
                Event::Show => {
                    win_clone.show();
                    win_clone.set_visible_focus();
                    true
                }
                Event::KeyDown => {
                    let key = app::event_key();
                    if app::is_event_command() && key == Key::from_char('o') {
                        state.borrow_mut().open_file();
                        true
                    } else if app::is_event_command() && key == Key::from_char('p') {
                        state.borrow_mut().process_pdf();
                        true
                    } else if key == Key::Left {
                        let mut state_ref = state.borrow_mut();
                        
                        // Always navigate PDF pages since structured view shows entire document
                        state_ref.prev_page();
                        true
                    } else if key == Key::Right {
                        let mut state_ref = state.borrow_mut();
                        
                        // Always navigate PDF pages since structured view shows entire document
                        state_ref.next_page();
                        true
                    } else if key == Key::from_char('+') || key == Key::from_char('=') {
                        state.borrow_mut().zoom_in();
                        true
                    } else if key == Key::from_char('-') {
                        state.borrow_mut().zoom_out();
                        true
                    } else if key == Key::from_char('f') {
                        state.borrow_mut().fit_to_width();
                        true
                    } else {
                        false
                    }
                }
                _ => false,
            });
        }
        
        app_state
    }
    
    fn log(&mut self, message: &str) {
        self.log_buffer.append(&format!("{}\n", message));
        // Scroll to bottom
        let len = self.log_buffer.length();
        self.log_display.scroll(len, 0);
    }
    
    fn open_file(&mut self) {
        self.log("📂 Opening file dialog...");
        
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("PDF", &["pdf"])
            .pick_file()
        {
            self.load_pdf(path);
        } else {
            self.log("❌ No file selected");
        }
    }
    
    fn process_pdf(&mut self) {
        if self.pdf_path.is_some() {
            self.log("🔄 Extracting text...");
            self.extract_current_page_text();
        } else {
            self.log("⚠️ No PDF loaded. Press Cmd+O to open a file first.");
        }
    }
    
    fn load_pdf(&mut self, path: PathBuf) {
        let filename = path.file_name().unwrap_or_default().to_string_lossy();
        self.log(&format!("📄 Loading: {}", filename));
        
        // Use mupdf info command to get page count with timeout
        match Command::new("timeout")
            .arg("5")  // 5 second timeout
            .arg("mutool")
            .arg("info")
            .arg(&path)
            .output()
        {
            Ok(output) => {
                let info = String::from_utf8_lossy(&output.stdout);
                
                // Parse page count from output
                let mut total_pages = 0;
                for line in info.lines() {
                    if line.contains("Pages:") {
                        if let Some(count_str) = line.split("Pages:").nth(1) {
                            if let Ok(count) = count_str.trim().parse::<usize>() {
                                total_pages = count;
                                break;
                            }
                        }
                    }
                }
                
                if total_pages > 0 {
                    self.pdf_path = Some(path);
                    self.total_pages = total_pages;
                    self.current_page = 0;
                    
                    self.log(&format!("✅ PDF loaded successfully: {} pages", self.total_pages));
                    self.update_status(&format!("Loaded! {} pages", self.total_pages));
                    
                    // Enable navigation buttons
                    if self.total_pages > 1 {
                        self.next_btn.activate();
                    }
                    
                    // Enable extract buttons
                    self.extract_btn.activate();
                    self.extract_btn.set_color(Color::from_rgb(0x00, 0x8C, 0xBA));
                    self.extract_btn.set_label_color(Color::White);
                    
                    self.structured_btn.activate();
                    self.structured_btn.set_color(Color::from_rgb(0x00, 0x8C, 0x3A));
                    self.structured_btn.set_label_color(Color::White);
                    
                    // Update UI
                    self.update_page_label();
                    
                    // Render the PDF page immediately
                    self.render_current_page();
                    
                    // But don't extract text yet - wait for Extract button
                    self.extracted_text_buffer.set_text("Click '📋 Raw JSON' to see ferrules data or '✨ Pretty View' to see formatted text...");
                } else {
                    self.log("❌ Failed to parse PDF info");
                    self.update_status("Failed to parse PDF info");
                }
            }
            Err(e) => {
                let error_msg = format!("Failed to run mutool: {}", e);
                self.log(&format!("❌ {}", error_msg));
                self.update_status(&error_msg);
            }
        }
    }
    
    fn render_current_page(&mut self) {
        if let Some(pdf_path) = &self.pdf_path {
            // Create temp file for rendered page
            let temp_dir = std::env::temp_dir();
            let png_path = temp_dir.join(format!("chonker5_page_{}.png", self.current_page));
            
            // Calculate DPI based on zoom level
            let dpi = (150.0 * self.zoom_level) as i32;
            
            // Use mutool draw to render page to PNG with timeout
            let output = Command::new("timeout")
                .arg("5")  // 5 second timeout
                .arg("mutool")
                .arg("draw")
                .arg("-o")
                .arg(&png_path)
                .arg("-r")
                .arg(dpi.to_string())
                .arg("-F")
                .arg("png")
                .arg(&pdf_path)
                .arg((self.current_page + 1).to_string())
                .output();
            
            match output {
                Ok(_) => {
                    // Load the rendered PNG
                    if let Ok(img) = fltk_image::PngImage::load(&png_path) {
                        // Convert to RgbImage
                        let width = img.width();
                        let height = img.height();
                        
                        // Update the frame size and redraw
                        self.pdf_frame.set_size(width, height);
                        self.pdf_frame.set_image(Some(img));
                        self.pdf_frame.set_label("");
                        self.pdf_frame.redraw();
                        
                        self.log(&format!("✅ Page {} rendered", self.current_page + 1));
                    }
                    
                    // Clean up temp file
                    let _ = fs::remove_file(&png_path);
                }
                Err(e) => {
                    self.log(&format!("❌ Failed to render page: {}", e));
                }
            }
            
            // Don't extract text automatically - wait for Cmd+P
        }
    }
    
    fn extract_current_page_text(&mut self) {
        if let Some(pdf_path) = self.pdf_path.clone() {
            // Show text display and hide structured view
            self.structured_view.hide();
            self.extracted_text_display.show();
            
            self.log("🔄 Extracting raw JSON with ferrules...");
            
            // Create temp dir for ferrules output
            let temp_dir = std::env::temp_dir();
            let ferrules_dir = temp_dir.join("chonker5_ferrules");
            
            // Create the directory if it doesn't exist
            if let Err(e) = fs::create_dir_all(&ferrules_dir) {
                self.extracted_text_buffer.set_text(&format!("Error creating temp directory: {}", e));
                self.log(&format!("❌ Failed to create temp dir: {}", e));
                return;
            }
            
            let json_path = ferrules_dir.join("output.json");
            
            // Debug: log the path we're using
            self.log(&format!("📂 Using PDF path: {:?}", pdf_path));
            
            // Check if file exists
            if !pdf_path.exists() {
                self.extracted_text_buffer.set_text(&format!("Error: PDF file not found at {:?}", pdf_path));
                self.log(&format!("❌ PDF file not found: {:?}", pdf_path));
                return;
            }
            
            // Convert path to string for ferrules
            let pdf_path_str = pdf_path.to_str().unwrap_or("");
            let json_path_str = json_path.to_str().unwrap_or("");
            
            self.log(&format!("📄 PDF: {}", pdf_path_str));
            self.log(&format!("📝 Output: {}", json_path_str));
            
            // Run ferrules command to get JSON
            // Note: ferrules might need the output directory, not the full file path
            let output = Command::new("ferrules")
                .arg(pdf_path_str)
                .arg("-o")
                .arg(&ferrules_dir)
                .output();
            
            match output {
                Ok(result) => {
                    self.log(&format!("🔧 Ferrules exit code: {}", result.status.code().unwrap_or(-1)));
                    if !result.stderr.is_empty() {
                        let stderr = String::from_utf8_lossy(&result.stderr);
                        self.log(&format!("⚠️ Ferrules stderr: {}", stderr));
                    }
                    
                    if result.status.success() {
                        // Ferrules creates a results directory
                        let pdf_stem = pdf_path.file_stem().unwrap_or_default().to_string_lossy();
                        // Remove special characters from filename (match ferrules' sanitization)
                        let safe_stem = pdf_stem.replace(")", "-").replace("(", "").replace("+", "-");
                        let results_dir = ferrules_dir.join(format!("{}-results", safe_stem));
                        let actual_json_path = results_dir.join(format!("{}.json", safe_stem));
                        
                        self.log(&format!("📋 Looking for JSON at: {:?}", actual_json_path));
                        self.log(&format!("📂 PDF stem: '{}' -> Safe stem: '{}'", pdf_stem, safe_stem));
                        
                        // Read the JSON file
                        match fs::read_to_string(&actual_json_path) {
                            Ok(json_content) => {
                                // Pretty print the JSON
                                match serde_json::from_str::<serde_json::Value>(&json_content) {
                                    Ok(json_value) => {
                                        let pretty_json = serde_json::to_string_pretty(&json_value)
                                            .unwrap_or(json_content);
                                        self.extracted_text_buffer.set_text(&pretty_json);
                                        self.log("✅ Raw JSON extracted with ferrules");
                                    }
                                    Err(_) => {
                                        self.extracted_text_buffer.set_text(&json_content);
                                        self.log("✅ Raw JSON extracted (unparsed)");
                                    }
                                }
                            }
                            Err(e) => {
                                // Try to list what files were created
                                if let Ok(entries) = fs::read_dir(&ferrules_dir) {
                                    self.log("📁 Files in ferrules output:");
                                    for entry in entries {
                                        if let Ok(entry) = entry {
                                            self.log(&format!("  - {:?}", entry.file_name()));
                                        }
                                    }
                                }
                                
                                self.extracted_text_buffer.set_text(&format!("Error reading JSON: {}\nExpected at: {:?}", e, actual_json_path));
                                self.log(&format!("❌ Failed to read JSON: {}", e));
                            }
                        }
                        
                        // Clean up directory
                        let _ = fs::remove_dir_all(&ferrules_dir);
                    } else {
                        let stderr = String::from_utf8_lossy(&result.stderr);
                        self.extracted_text_buffer.set_text(&format!("Ferrules error:\n{}", stderr));
                        self.log(&format!("❌ Ferrules failed: {}", stderr));
                    }
                }
                Err(e) => {
                    self.extracted_text_buffer.set_text(&format!("Error running ferrules: {}", e));
                    self.log(&format!("❌ Failed to run ferrules: {}", e));
                }
            }
            
            app::awake();
        }
    }
    
    fn extract_structured_data(&mut self) {
        if let Some(pdf_path) = &self.pdf_path.clone() {
            self.log("🔄 Loading pretty view with ferrules...");
            
            // Show structured view and hide text display
            self.extracted_text_display.hide();
            self.structured_view.show();
            
            // Create temp dir for ferrules output
            let temp_dir = std::env::temp_dir();
            let ferrules_dir = temp_dir.join("chonker5_ferrules");
            
            // Create the directory if it doesn't exist
            if let Err(e) = fs::create_dir_all(&ferrules_dir) {
                self.extracted_text_buffer.set_text(&format!("Error creating temp directory: {}", e));
                self.log(&format!("❌ Failed to create temp dir: {}", e));
                return;
            }
            
            // Run ferrules command
            let output = Command::new("ferrules")
                .arg(pdf_path)
                .arg("-o")
                .arg(&ferrules_dir)
                .output();
            
            match output {
                Ok(result) => {
                    self.log(&format!("🔧 Ferrules exit code: {}", result.status.code().unwrap_or(-1)));
                    if !result.stderr.is_empty() {
                        let stderr = String::from_utf8_lossy(&result.stderr);
                        self.log(&format!("⚠️ Ferrules stderr: {}", stderr));
                    }
                    
                    if result.status.success() {
                        // Ferrules creates a results directory - use same logic as Raw JSON button
                        let pdf_stem = pdf_path.file_stem().unwrap_or_default().to_string_lossy();
                        // Remove special characters from filename (match ferrules' sanitization)
                        let safe_stem = pdf_stem.replace(")", "-").replace("(", "").replace("+", "-");
                        let results_dir = ferrules_dir.join(format!("{}-results", safe_stem));
                        
                        self.log(&format!("📂 Looking for files in: {:?}", results_dir));
                        self.log(&format!("📂 PDF stem: '{}' -> Safe stem: '{}'", pdf_stem, safe_stem));
                        
                        // Look for JSON file in the results directory
                        let json_file = results_dir.join(format!("{}.json", safe_stem));
                        
                        if let Ok(json_content) = fs::read_to_string(&json_file) {
                            self.log(&format!("📄 Found JSON: {:?}", json_file));
                            
                            // Parse JSON directly for pretty view
                            match serde_json::from_str::<FerrulesDocument>(&json_content) {
                                Ok(doc) => {
                                    self.structured_json_data = Some(doc.clone());
                                    self.log(&format!("✅ Parsed ferrules JSON data: {} pages, {} blocks", 
                                        doc.pages.len(), doc.blocks.len()));
                                    
                                    // Update the structured view widget with the document
                                    self.structured_view.set_document(doc.clone());
                                    
                                    // Enable compare button
                                    self.compare_btn.activate();
                                    self.compare_btn.set_color(Color::from_rgb(0xFF, 0x85, 0x00));
                                    
                                    self.log("✅ Pretty view loaded successfully");
                                }
                                Err(e) => {
                                    self.log(&format!("⚠️ Failed to parse JSON: {}", e));
                                    self.extracted_text_buffer.set_text(&format!("Error parsing JSON: {}", e));
                                }
                            }
                        } else {
                            self.log("❌ Failed to read JSON file");
                            self.extracted_text_buffer.set_text("Error: Could not read JSON file");
                        }
                        
                        // Clean up temp directory
                        let _ = fs::remove_dir_all(&ferrules_dir);
                    } else {
                        let error_msg = String::from_utf8_lossy(&result.stderr);
                        self.extracted_text_buffer.set_text(&format!("Ferrules error: {}", error_msg));
                        self.log(&format!("❌ Ferrules failed: {}", error_msg));
                    }
                }
                Err(e) => {
                    self.extracted_text_buffer.set_text(&format!("Failed to run ferrules: {}", e));
                    self.log(&format!("❌ Failed to run ferrules: {}", e));
                }
            }
            
            app::awake();
        } else {
            self.log("⚠️ No PDF loaded. Press Cmd+O to open a file first.");
        }
    }
    
    fn prev_page(&mut self) {
        if self.current_page > 0 {
            self.current_page -= 1;
            self.update_page_label();
            self.update_nav_buttons();
            self.log(&format!("◀ Page {}", self.current_page + 1));
            
            // Render the new page
            self.render_current_page();
            
            // Clear extracted text - user needs to extract again
            self.extracted_text_buffer.set_text("Click '📋 Raw JSON' to see ferrules data or '✨ Pretty View' to see formatted text...");
        }
    }
    
    fn next_page(&mut self) {
        if self.current_page < self.total_pages - 1 {
            self.current_page += 1;
            self.update_page_label();
            self.update_nav_buttons();
            self.log(&format!("▶ Page {}", self.current_page + 1));
            
            // Render the new page
            self.render_current_page();
            
            // Clear extracted text - user needs to extract again
            self.extracted_text_buffer.set_text("Click '📋 Raw JSON' to see ferrules data or '✨ Pretty View' to see formatted text...");
        }
    }
    
    fn zoom_in(&mut self) {
        self.zoom_level = (self.zoom_level * 1.2).min(4.0);
        self.update_zoom_label();
        self.render_current_page();
        self.log(&format!("🔍+ Zoom: {}%", (self.zoom_level * 100.0) as i32));
    }
    
    fn zoom_out(&mut self) {
        self.zoom_level = (self.zoom_level / 1.2).max(0.25);
        self.update_zoom_label();
        self.render_current_page();
        self.log(&format!("🔍- Zoom: {}%", (self.zoom_level * 100.0) as i32));
    }
    
    fn fit_to_width(&mut self) {
        // Calculate zoom to fit width (now using half window width due to split pane)
        let viewport_width = self.window.width() / 2 - 40;
        let base_width = 800.0;
        
        self.zoom_level = (viewport_width as f32 / base_width / 2.0).clamp(0.25, 4.0);
        self.update_zoom_label();
        self.render_current_page();
        self.log(&format!("📐 Fit to width - Zoom: {}%", (self.zoom_level * 100.0) as i32));
    }
    
    fn update_status(&mut self, text: &str) {
        self.status_label.set_label(text);
    }
    
    fn update_zoom_label(&mut self) {
        self.zoom_label.set_label(&format!("Zoom: {}%", (self.zoom_level * 100.0) as i32));
    }
    
    fn update_page_label(&mut self) {
        if self.total_pages > 0 {
            self.page_label.set_label(&format!("Page: {}/{}", self.current_page + 1, self.total_pages));
        } else {
            self.page_label.set_label("Page: 0/0");
        }
    }
    
    fn update_nav_buttons(&mut self) {
        if self.current_page > 0 {
            self.prev_btn.activate();
            self.prev_btn.set_color(Color::White);
            self.prev_btn.set_label_color(Color::Black);
        } else {
            self.prev_btn.deactivate();
            self.prev_btn.set_color(Color::from_rgb(0xDD, 0xDD, 0xDD));
            self.prev_btn.set_label_color(Color::from_rgb(0x66, 0x66, 0x66));
        }
        
        if self.current_page < self.total_pages - 1 {
            self.next_btn.activate();
            self.next_btn.set_color(Color::White);
            self.next_btn.set_label_color(Color::Black);
        } else {
            self.next_btn.deactivate();
            self.next_btn.set_color(Color::from_rgb(0xDD, 0xDD, 0xDD));
            self.next_btn.set_label_color(Color::from_rgb(0x66, 0x66, 0x66));
        }
    }
    
    fn post_process_html(&self, html: &str) -> String {
        let mut processed = html.to_string();
        
        // Add CSS for better table rendering and layout
        let enhanced_css = r#"
        <style>
            body {
                font-family: Arial, sans-serif;
                line-height: 1.6;
                padding: 20px;
                background-color: #f5f5f5;
            }
            table {
                border-collapse: collapse;
                width: 100%;
                margin: 10px 0;
                background-color: white;
                box-shadow: 0 2px 4px rgba(0,0,0,0.1);
            }
            th, td {
                border: 1px solid #ddd;
                padding: 8px;
                text-align: left;
            }
            th {
                background-color: #1ABC9C;
                color: white;
                font-weight: bold;
            }
            tr:nth-child(even) {
                background-color: #f9f9f9;
            }
            tr:hover {
                background-color: #f5f5f5;
            }
            h1, h2, h3 {
                color: #333;
                margin-top: 20px;
            }
            p {
                margin: 10px 0;
            }
            .page-break {
                border-bottom: 2px solid #1ABC9C;
                margin: 20px 0;
                padding-bottom: 10px;
            }
            .column-layout {
                column-count: 2;
                column-gap: 20px;
            }
            @media (max-width: 800px) {
                .column-layout {
                    column-count: 1;
                }
            }
        </style>
        "#;
        
        // Insert CSS after <head> tag or at beginning
        if processed.contains("<head>") {
            processed = processed.replace("<head>", &format!("<head>{}", enhanced_css));
        } else if processed.contains("<html>") {
            processed = processed.replace("<html>", &format!("<html><head>{}</head>", enhanced_css));
        } else {
            processed = format!("{}{}", enhanced_css, processed);
        }
        
        // Clean up common artifacts
        processed = self.clean_text_artifacts(&processed);
        
        // Improve table structure detection
        processed = self.enhance_table_structure(&processed);
        
        // Fix spacing issues
        processed = self.fix_spacing_issues(&processed);
        
        processed
    }
    
    fn clean_text_artifacts(&self, html: &str) -> String {
        let mut cleaned = html.to_string();
        
        // Remove multiple consecutive spaces
        while cleaned.contains("  ") {
            cleaned = cleaned.replace("  ", " ");
        }
        
        // Fix common OCR artifacts
        cleaned = cleaned.replace("•", "·");
        cleaned = cleaned.replace("—", "-");
        // Note: Smart quote replacement removed due to Rust string literal issues
        // Would need to handle Unicode quotes differently
        
        // Remove empty paragraphs
        cleaned = cleaned.replace("<p></p>", "");
        cleaned = cleaned.replace("<p> </p>", "");
        
        cleaned
    }
    
    fn enhance_table_structure(&self, html: &str) -> String {
        let mut enhanced = html.to_string();
        
        // Add table headers if missing
        if enhanced.contains("<table>") && !enhanced.contains("<thead>") {
            // Simple heuristic: if first row has all bold text, make it header
            enhanced = enhanced.replace("<table>", "<table class='data-table'>");
        }
        
        enhanced
    }
    
    fn fix_spacing_issues(&self, html: &str) -> String {
        let mut fixed = html.to_string();
        
        // Add proper spacing between sections
        fixed = fixed.replace("</p><p>", "</p>\n<p>");
        fixed = fixed.replace("</table><p>", "</table>\n<p>");
        fixed = fixed.replace("</p><table>", "</p>\n<table>");
        
        // Fix line breaks
        fixed = fixed.replace("<br><br>", "<br>");
        
        fixed
    }
    
    fn toggle_compare_mode(&mut self) {
        self.compare_mode = !self.compare_mode;
        
        if self.compare_mode {
            self.compare_btn.set_label("Normal View");
            self.compare_btn.set_color(Color::from_rgb(0x00, 0x8C, 0xBA));
            self.log("📊 Compare mode enabled - showing position data");
            
            // The custom widget already shows position data through the bounding boxes
        } else {
            self.compare_btn.set_label("Compare");
            self.compare_btn.set_color(Color::from_rgb(0xFF, 0x85, 0x00));
            self.log("📄 Normal view restored");
            
            // The custom widget handles this automatically
        }
    }
    
    fn add_position_highlights(&self, html: &str) -> String {
        let mut highlighted = html.to_string();
        
        // Add CSS for position highlighting
        let highlight_css = r#"
        <style>
            .pdf-position {
                position: relative;
                border-left: 3px solid #1ABC9C;
                padding-left: 10px;
                margin-left: 5px;
            }
            .pdf-position::before {
                content: attr(data-page) " - " attr(data-position);
                position: absolute;
                left: -80px;
                font-size: 10px;
                color: #1ABC9C;
                white-space: nowrap;
            }
            .table-position {
                border: 2px solid #1ABC9C;
            }
        </style>
        "#;
        
        // Insert highlight CSS
        if highlighted.contains("</style>") {
            highlighted = highlighted.replace("</style>", &format!("{}</style>", highlight_css));
        } else if highlighted.contains("<head>") {
            highlighted = highlighted.replace("<head>", &format!("<head>{}", highlight_css));
        }
        
        highlighted
    }
    
    
    fn run(app_state: Rc<RefCell<Self>>) {
        let app = app_state.borrow().app.clone();
        app.run().unwrap();
    }
}


fn main() {
    let app_state = Chonker5App::new();
    Chonker5App::run(app_state);
}