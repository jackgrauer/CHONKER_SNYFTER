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
    enums::{Color, Event, Font, FrameType, Key},
    frame::Frame,
    group::{Flex, Group, Scroll},
    prelude::*,
    text::{TextBuffer, TextDisplay},
    window::Window,
    image as fltk_image,
};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::process::Command;
use std::fs;
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

// StructuredTextWidget and Pretty View functionality removed

// Advanced table detection from ferrules blocks - ULTRA AGGRESSIVE MODE
fn detect_tables(blocks: &[FerrulesBlock], page_id: i32) -> Vec<DetectedTable> {
    let mut tables = Vec::new();
    
    // Filter blocks for this page
    let mut page_blocks: Vec<(usize, &FerrulesBlock)> = blocks
        .iter()
        .enumerate()
        .filter(|(_, b)| b.pages_id.contains(&page_id))
        .collect();
    
    println!("🔍 ULTRA table detection for page {}: {} blocks", page_id + 1, page_blocks.len());
    
    // Debug: Show what text we're actually seeing
    if page_blocks.len() < 20 {  // Only for small pages to avoid spam
        println!("  📝 Block contents:");
        for (idx, (_, block)) in page_blocks.iter().enumerate() {
            let text = match &block.kind {
                FerrulesKind::Text { text } => text,
                FerrulesKind::Structured { text, .. } => text,
                _ => "",
            };
            if !text.trim().is_empty() {
                println!("    Block {}: [{}]", idx, text.trim().replace('\n', " "));
            }
        }
    }
    
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
    // But first, filter out blocks that are clearly not table content
    let mut rows: Vec<Vec<(usize, &FerrulesBlock)>> = Vec::new();
    let _row_tolerance = 3.0; // Tighter tolerance for better accuracy
    
    // Pre-filter: Skip blocks that are table descriptions, not table content
    let filtered_blocks: Vec<(usize, &FerrulesBlock)> = page_blocks.iter()
        .filter(|(_, block)| {
            let text = match &block.kind {
                FerrulesKind::Text { text } => text.as_str(),
                FerrulesKind::Structured { text, .. } => text.as_str(),
                _ => "",
            };
            let lower = text.to_lowercase();
            
            // Skip if it's a table description
            !(lower.contains("table") && (lower.contains("shows") || lower.contains("summary") || lower.contains("presents"))) &&
            // Skip if it's a note or source
            !lower.starts_with("note") &&
            !lower.starts_with("source") &&
            // Skip very long text blocks (likely paragraphs)
            text.len() < 150 &&
            // Skip single words that are likely headers
            !(text.trim().split_whitespace().count() == 1 && text.len() > 20)
        })
        .map(|(idx, block)| (*idx, *block))
        .collect();
    
    // Use filtered blocks if we have enough, otherwise use all
    let blocks_to_process = if filtered_blocks.len() >= 3 {
        println!("  🎯 Using {} filtered blocks (removed {} non-table blocks)", 
            filtered_blocks.len(), page_blocks.len() - filtered_blocks.len());
        &filtered_blocks
    } else {
        println!("  ⚠️ Only {} filtered blocks, using all {} blocks", 
            filtered_blocks.len(), page_blocks.len());
        &page_blocks
    };
    
    for (idx, block) in blocks_to_process {
        let _y_center = (block.bbox.y0 + block.bbox.y1) / 2.0;
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
    
    // Phase 4: ULTRA AGGRESSIVE table detection
    // Multiple strategies to catch ALL possible tables
    if tables.is_empty() && rows.len() >= 1 {
        println!("  🔥 ULTRA AGGRESSIVE MODE ACTIVATED...");
        
        // Strategy 0: Look for horizontal alignment patterns (NEW!)
        // If multiple blocks share similar X coordinates, they might be table columns
        let mut x_positions: Vec<f64> = Vec::new();
        for row in &rows {
            for (_, block) in row {
                x_positions.push(block.bbox.x0);
                x_positions.push(block.bbox.x1);
            }
        }
        x_positions.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        // Find clusters of X positions (potential column boundaries)
        let mut x_clusters: Vec<Vec<f64>> = Vec::new();
        let x_tolerance = 10.0;
        
        for &x in &x_positions {
            let mut found_cluster = false;
            for cluster in &mut x_clusters {
                if cluster.iter().any(|&cx| (cx - x).abs() < x_tolerance) {
                    cluster.push(x);
                    found_cluster = true;
                    break;
                }
            }
            if !found_cluster {
                x_clusters.push(vec![x]);
            }
        }
        
        // If we have 3+ consistent X positions, we might have columns
        let significant_x_positions = x_clusters.iter()
            .filter(|cluster| cluster.len() >= 2)
            .count();
        
        println!("  📐 Found {} potential column boundaries", significant_x_positions);
        
        // Strategy 1: Look for any 2+ consecutive rows with consistent structure
        let mut i = 0;
        while i < rows.len() {
            // Check if this could be a table start
            let mut table_rows_indices = vec![i];
            let mut j = i + 1;
            
            // Look for consecutive rows that could be part of same table
            while j < rows.len() {
                let row_gap = if j > 0 {
                    rows[j][0].1.bbox.y0 - rows[j-1][0].1.bbox.y1
                } else {
                    0.0
                };
                
                // If gap is too large, probably not same table
                if row_gap > 50.0 {
                    break;
                }
                
                table_rows_indices.push(j);
                j += 1;
            }
            
            // Check if this looks like a table
            let multi_cell_rows = table_rows_indices.iter()
                .filter(|&&idx| rows[idx].len() >= 2)
                .count();
            
            let has_numeric_content = table_rows_indices.iter()
                .any(|&idx| {
                    rows[idx].iter().any(|(_, block)| {
                        let text = match &block.kind {
                            FerrulesKind::Text { text } => text,
                            FerrulesKind::Structured { text, .. } => text,
                            _ => "",
                        };
                        // Much more aggressive numeric detection
                        text.trim().parse::<f64>().is_ok() || 
                        text.contains('$') || 
                        text.contains('€') || 
                        text.contains('£') || 
                        text.contains('%') ||
                        text.contains('.') ||
                        text.chars().any(|c| c.is_numeric()) ||
                        text.contains(',') && text.chars().filter(|&c| c.is_numeric()).count() > 0 ||
                        // Date patterns
                        text.contains('/') && text.chars().filter(|&c| c.is_numeric()).count() >= 4 ||
                        text.contains('-') && text.chars().filter(|&c| c.is_numeric()).count() >= 4 ||
                        // Currency patterns
                        (text.starts_with('$') || text.starts_with('€') || text.starts_with('£')) ||
                        text.ends_with("USD") || text.ends_with("EUR") || text.ends_with("GBP")
                    })
                });
            
            // Check for table headers and separators
            let has_table_indicators = table_rows_indices.iter()
                .any(|&idx| {
                    rows[idx].iter().any(|(_, block)| {
                        let text = match &block.kind {
                            FerrulesKind::Text { text } => text,
                            FerrulesKind::Structured { text, .. } => text,
                            _ => "",
                        };
                        // Common table headers and content
                        let lower = text.to_lowercase();
                        // Common table headers
                        lower.contains("total") ||
                        lower.contains("amount") ||
                        lower.contains("date") ||
                        lower.contains("description") ||
                        lower.contains("quantity") ||
                        lower.contains("price") ||
                        lower.contains("item") ||
                        lower.contains("name") ||
                        lower.contains("value") ||
                        lower.contains("count") ||
                        // Invoice/financial table indicators
                        lower.contains("invoice") ||
                        lower.contains("subtotal") ||
                        lower.contains("tax") ||
                        lower.contains("balance") ||
                        lower.contains("payment") ||
                        lower.contains("due") ||
                        lower.contains("rate") ||
                        lower.contains("hours") ||
                        lower.contains("unit") ||
                        lower.contains("cost") ||
                        lower.contains("fee") ||
                        lower.contains("charge") ||
                        // Table structure indicators
                        text.contains("|") ||
                        text.contains("\t") ||
                        text.chars().filter(|&c| c == '-').count() > 3 ||
                        text.chars().filter(|&c| c == '_').count() > 3 ||
                        text.chars().filter(|&c| c == '=').count() > 3 ||
                        // Column headers are often short
                        (text.trim().len() < 20 && text.trim().len() > 0 && !text.chars().all(|c| c.is_whitespace()))
                    })
                });
            
            // ULTRA AGGRESSIVE: Detect if ANY of these conditions are met
            // NEW: Even MORE aggressive detection
            let grid_pattern_detected = table_rows_indices.len() >= 1 && 
                table_rows_indices.iter().all(|&idx| {
                    let row_len = rows[idx].len();
                    row_len >= 2 && row_len == rows[table_rows_indices[0]].len()
                });
            
            let has_separator_lines = table_rows_indices.iter()
                .any(|&idx| {
                    rows[idx].iter().any(|(_, block)| {
                        let text = match &block.kind {
                            FerrulesKind::Text { text } => text,
                            FerrulesKind::Structured { text, .. } => text,
                            _ => "",
                        };
                        // Lines made of dashes, underscores, equals, or pipes
                        text.chars().filter(|&c| c == '-' || c == '_' || c == '=' || c == '|').count() > 5
                    })
                });
            
            // Check for consistent spacing between rows (table-like)
            let has_consistent_row_spacing = if table_rows_indices.len() >= 3 {
                let mut spacings = Vec::new();
                for i in 1..table_rows_indices.len() {
                    let curr_idx = table_rows_indices[i];
                    let prev_idx = table_rows_indices[i-1];
                    if !rows[curr_idx].is_empty() && !rows[prev_idx].is_empty() {
                        let spacing = rows[curr_idx][0].1.bbox.y0 - rows[prev_idx][0].1.bbox.y1;
                        spacings.push(spacing);
                    }
                }
                if spacings.len() >= 2 {
                    let avg_spacing = spacings.iter().sum::<f64>() / spacings.len() as f64;
                    spacings.iter().all(|&s| (s - avg_spacing).abs() < 10.0)
                } else {
                    false
                }
            } else {
                false
            };
            
            let is_likely_table = (
                // Traditional table detection
                (table_rows_indices.len() >= 2 && multi_cell_rows >= 1) ||
                // Numeric content suggests table
                (table_rows_indices.len() >= 2 && has_numeric_content) ||
                // Table headers/indicators found
                (table_rows_indices.len() >= 2 && has_table_indicators) ||
                // Column alignment detected
                (table_rows_indices.len() >= 3 && significant_x_positions >= 3) ||
                // Single row with multiple cells (header row?)
                (table_rows_indices.len() == 1 && rows[table_rows_indices[0]].len() >= 3) ||
                // NEW: Grid pattern - all rows have same number of cells
                grid_pattern_detected ||
                // NEW: Has separator lines (common in tables)
                has_separator_lines ||
                // NEW: Consistent row spacing
                has_consistent_row_spacing ||
                // NEW: Any 2+ rows with 2+ cells each is a table!
                (table_rows_indices.len() >= 2 && table_rows_indices.iter().all(|&idx| rows[idx].len() >= 2))
            );
            
            if is_likely_table {
                // Check if this might be a table description rather than actual table data
                let is_table_description = table_rows_indices.iter().any(|&idx| {
                    rows[idx].iter().any(|(_, block)| {
                        let text = match &block.kind {
                            FerrulesKind::Text { text } => text,
                            FerrulesKind::Structured { text, .. } => text,
                            _ => "",
                        };
                        let lower = text.to_lowercase();
                        lower.contains("table") && (lower.contains("shows") || lower.contains("information") || 
                            lower.contains("presents") || lower.contains("city of") || lower.contains("summary"))
                    })
                });
                
                if is_table_description {
                    println!("    📋 DETECTED TABLE REFERENCE! This appears to be a table caption/title.");
                    println!("       The actual table data may be missing from the ferrules output.");
                } else {
                    println!("    🎯 DETECTED TABLE! Rows: {}, Multi-cell: {}, Numeric: {}, Indicators: {}, Columns: {}", 
                        table_rows_indices.len(), multi_cell_rows, has_numeric_content, has_table_indicators, significant_x_positions);
                }
                println!("       Grid: {}, Separators: {}, Consistent spacing: {}", 
                    grid_pattern_detected, has_separator_lines, has_consistent_row_spacing);
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
                    
                    for &row_idx in &table_rows_indices {
                        if let Some(row) = rows.get(row_idx) {
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
                        println!("    💡 Found table with {} rows", detected_table.rows.len());
                        tables.push(detected_table);
                    }
                
                i = j; // Skip past this table
            } else {
                i += 1;
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
    table_btn: Button,
    // structured_btn: Button, // REMOVED
    // compare_btn: Button,   // REMOVED
    extracted_text_display: TextDisplay,
    extracted_text_buffer: TextBuffer,
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
        let mut table_btn = Button::default()
            .with_pos(x_pos, y_pos)
            .with_size(140, 40)
            .with_label("📊 Analyze Structure");
        table_btn.set_color(Color::from_rgb(0x8B, 0x00, 0x8B)); // Purple color for distinction
        table_btn.set_label_color(Color::White);
        table_btn.set_frame(FrameType::UpBox);
        table_btn.set_label_size(14);
        table_btn.deactivate(); // Start disabled until PDF is loaded
        
        // Pretty View and Compare buttons removed - functionality was broken
        /*
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
        */
        
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
            table_btn: table_btn.clone(),
            // structured_btn: structured_btn.clone(), // REMOVED
            // compare_btn: compare_btn.clone(),   // REMOVED
            extracted_text_display: extracted_text_display.clone(),
            extracted_text_buffer,
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
        
        // Table analysis button
        {
            let state = app_state.clone();
            table_btn.set_callback(move |_| {
                state.borrow_mut().extract_and_analyze_structure();
            });
        }
        
        // Structured data button - REMOVED
        /* Pretty view button disabled - functionality removed
        structured_btn.deactivate();
        structured_btn.set_label("❌ Removed");
        */
        
        // Compare button - REMOVED
        /*
        {
            let state = app_state.clone();
            compare_btn.set_callback(move |_| {
                state.borrow_mut().toggle_compare_mode();
            });
        }
        */
        
        
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
                    
                    // Enable table analysis button
                    self.table_btn.activate();
                    self.table_btn.set_color(Color::from_rgb(0x8B, 0x00, 0x8B));
                    self.table_btn.set_label_color(Color::White);
                    
                    // Pretty View button removed
                    // self.structured_btn.activate();
                    // self.structured_btn.set_color(Color::from_rgb(0x00, 0x8C, 0x3A));
                    // self.structured_btn.set_label_color(Color::White);
                    
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
            // Show text display
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
        // Pretty view functionality has been removed
        self.log("❌ Pretty View has been removed - too broken");
        self.extracted_text_buffer.set_text("Pretty View functionality has been removed because it was too broken.\n\nUse the Raw JSON button instead.");
    }
    
    fn extract_and_analyze_structure(&mut self) {
        if let Some(pdf_path) = self.pdf_path.clone() {
            self.log("🔍 Extracting and analyzing document structure...");
            
            // Create temp dir for ferrules output
            let temp_dir = std::env::temp_dir();
            let ferrules_dir = temp_dir.join("chonker5_ferrules");
            
            if let Err(e) = fs::create_dir_all(&ferrules_dir) {
                self.extracted_text_buffer.set_text(&format!("Error creating temp directory: {}", e));
                self.log(&format!("❌ Failed to create temp dir: {}", e));
                return;
            }
            
            // Run ferrules to get structured data
            let output = Command::new("ferrules")
                .arg(&pdf_path)
                .arg("-o")
                .arg(&ferrules_dir)
                .output();
            
            match output {
                Ok(result) => {
                    if result.status.success() {
                        // Find the JSON file
                        let pdf_stem = pdf_path.file_stem().unwrap_or_default().to_string_lossy();
                        let safe_stem = pdf_stem.replace(")", "-").replace("(", "").replace("+", "-");
                        let results_dir = ferrules_dir.join(format!("{}-results", safe_stem));
                        let json_file = results_dir.join(format!("{}.json", safe_stem));
                        
                        // Parse the ferrules JSON
                        match fs::read_to_string(&json_file) {
                            Ok(json_content) => {
                                match serde_json::from_str::<FerrulesDocument>(&json_content) {
                                    Ok(doc) => {
                                        self.log(&format!("✅ Parsed ferrules data: {} pages, {} blocks", 
                                            doc.pages.len(), doc.blocks.len()));
                                        
                                        // Analyze document structure for each page
                                        let mut analysis = String::from("📊 DOCUMENT STRUCTURE ANALYSIS\n");
                                        analysis.push_str("==============================\n\n");
                                        
                                        for page in &doc.pages {
                                            let tables = detect_tables(&doc.blocks, page.id);
                                            
                                            analysis.push_str(&format!("📄 Page {} ({:.0}x{:.0} px)\n", 
                                                page.id + 1, page.width, page.height));
                                            
                                            if tables.is_empty() {
                                                analysis.push_str("   No structured sections detected\n");
                                            } else {
                                                analysis.push_str(&format!("   Found {} structured section(s):\n", tables.len()));
                                                
                                                for (i, table) in tables.iter().enumerate() {
                                                    analysis.push_str(&format!("\n   📄 Section {} - {} content blocks\n", i + 1, table.rows.len()));
                                                    analysis.push_str(&format!("      Location: ({:.0}, {:.0}) to ({:.0}, {:.0})\n",
                                                        table.bbox.x0, table.bbox.y0, table.bbox.x1, table.bbox.y1));
                                                    
                                                    // Extract section content
                                                    for (row_idx, row) in table.rows.iter().enumerate() {
                                                        analysis.push_str(&format!("      Block {}: ", row_idx + 1));
                                                        
                                                        let cell_texts: Vec<String> = row.cells.iter()
                                                            .filter_map(|cell| {
                                                                doc.blocks.get(cell.block_idx)
                                                                    .and_then(|block| match &block.kind {
                                                                        FerrulesKind::Text { text } => Some(text.trim().to_string()),
                                                                        FerrulesKind::Structured { text, .. } => Some(text.trim().to_string()),
                                                                        _ => None,
                                                                    })
                                                            })
                                                            .collect();
                                                        
                                                        analysis.push_str(&format!("[{}]\n", cell_texts.join(" | ")));
                                                    }
                                                }
                                            }
                                            analysis.push_str("\n");
                                        }
                                        
                                        // Add summary
                                        let total_tables: usize = doc.pages.iter()
                                            .map(|p| detect_tables(&doc.blocks, p.id).len())
                                            .sum();
                                        
                                        analysis.push_str(&format!("\n📈 SUMMARY\n"));
                                        analysis.push_str(&format!("Total pages: {}\n", doc.pages.len()));
                                        analysis.push_str(&format!("Total blocks: {}\n", doc.blocks.len()));
                                        analysis.push_str(&format!("Total structured sections: {}\n", total_tables));
                                        
                                        self.extracted_text_buffer.set_text(&analysis);
                                        self.log("✅ Document structure analysis complete");
                                    }
                                    Err(e) => {
                                        self.extracted_text_buffer.set_text(&format!("Error parsing JSON: {}", e));
                                        self.log(&format!("❌ Failed to parse JSON: {}", e));
                                    }
                                }
                            }
                            Err(e) => {
                                self.extracted_text_buffer.set_text(&format!("Error reading JSON: {}", e));
                                self.log(&format!("❌ Failed to read JSON: {}", e));
                            }
                        }
                        
                        // Clean up
                        let _ = fs::remove_dir_all(&ferrules_dir);
                    } else {
                        let stderr = String::from_utf8_lossy(&result.stderr);
                        self.extracted_text_buffer.set_text(&format!("Ferrules error: {}", stderr));
                        self.log(&format!("❌ Ferrules failed: {}", stderr));
                    }
                }
                Err(e) => {
                    self.extracted_text_buffer.set_text(&format!("Error running ferrules: {}", e));
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
            // Compare button removed
            // self.compare_btn.set_label("Normal View");
            // self.compare_btn.set_color(Color::from_rgb(0x00, 0x8C, 0xBA));
            self.log("📊 Compare mode enabled - showing position data");
            
            // The custom widget already shows position data through the bounding boxes
        } else {
            // Compare button removed  
            // self.compare_btn.set_label("Compare");
            // self.compare_btn.set_color(Color::from_rgb(0xFF, 0x85, 0x00));
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