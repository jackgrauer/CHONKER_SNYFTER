#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! pdfium-render = "0.8"
//! serde = { version = "1.0", features = ["derive"] }
//! serde_json = "1.0"
//! csv = "1.3"
//! ordered-float = "4.2"
//! ```

use pdfium_render::prelude::*;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::error::Error;
use serde::{Serialize, Deserialize};
use ordered_float::OrderedFloat;
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Cell {
    content: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    row_span: usize,
    col_span: usize,
    row_index: usize,
    col_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Table {
    cells: Vec<Vec<Option<Cell>>>,
    rows: usize,
    cols: usize,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    page_number: u16,
}

#[derive(Debug, Clone)]
struct TextFragment {
    text: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    font_size: f64,
    font_name: String,
    page_number: u16,
}

#[derive(Debug, Clone)]
struct Line {
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    page_number: u16,
}

struct PDFTableExtractor {
    pdfium: Pdfium,
    horizontal_tolerance: f64,
    vertical_tolerance: f64,
    min_table_rows: usize,
    min_table_cols: usize,
}

impl PDFTableExtractor {
    fn new() -> Result<Self, Box<dyn Error>> {
        let pdfium = Pdfium::new(
            Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./"))
                .or_else(|_| Pdfium::bind_to_system_library())?
        );

        Ok(PDFTableExtractor {
            pdfium,
            horizontal_tolerance: 3.0,
            vertical_tolerance: 3.0,
            min_table_rows: 2,
            min_table_cols: 2,
        })
    }

    fn extract_tables_from_file(&self, path: &Path) -> Result<Vec<Table>, Box<dyn Error>> {
        let document = self.pdfium.load_pdf_from_file(path, None)?;
        let mut all_tables = Vec::new();

        for (page_index, page) in document.pages().iter().enumerate() {
            let page_number = (page_index + 1) as u16;
            let text_fragments = self.extract_text_fragments(&page, page_number)?;
            let lines = self.extract_lines(&page, page_number)?;
            let tables = self.detect_tables(text_fragments, lines, page_number);
            all_tables.extend(tables);
        }

        Ok(all_tables)
    }

    fn extract_text_fragments(&self, page: &PdfPage, page_number: u16) -> Result<Vec<TextFragment>, Box<dyn Error>> {
        let mut fragments = Vec::new();
        let text_page = page.text()?;
        let char_count = text_page.chars().count();
        
        let mut current_fragment = String::new();
        let mut fragment_start_x = 0.0;
        let mut fragment_start_y = 0.0;
        let mut fragment_font_size = 0.0;
        let mut fragment_font_name = String::new();
        let mut last_x = 0.0;
        let mut last_y = 0.0;
        
        for index in 0..char_count {
            if let Some(char) = text_page.char(index) {
                let bounds = char.bounds();
                let font_size = char.font_size();
                let font_name = char.font_name().unwrap_or_default();
                
                let is_new_fragment = current_fragment.is_empty() ||
                    (bounds.top - last_y).abs() > self.vertical_tolerance ||
                    bounds.left - last_x > font_size * 0.3 ||
                    font_size != fragment_font_size ||
                    font_name != fragment_font_name;
                
                if is_new_fragment && !current_fragment.is_empty() {
                    fragments.push(TextFragment {
                        text: current_fragment.clone(),
                        x: fragment_start_x,
                        y: fragment_start_y,
                        width: last_x - fragment_start_x,
                        height: fragment_font_size,
                        font_size: fragment_font_size,
                        font_name: fragment_font_name.clone(),
                        page_number,
                    });
                    current_fragment.clear();
                }
                
                if current_fragment.is_empty() {
                    fragment_start_x = bounds.left;
                    fragment_start_y = bounds.top;
                    fragment_font_size = font_size;
                    fragment_font_name = font_name.clone();
                }
                
                current_fragment.push(char.unicode_char());
                last_x = bounds.right;
                last_y = bounds.top;
            }
        }
        
        if !current_fragment.is_empty() {
            fragments.push(TextFragment {
                text: current_fragment,
                x: fragment_start_x,
                y: fragment_start_y,
                width: last_x - fragment_start_x,
                height: fragment_font_size,
                font_size: fragment_font_size,
                font_name: fragment_font_name,
                page_number,
            });
        }
        
        Ok(fragments)
    }

    fn extract_lines(&self, page: &PdfPage, page_number: u16) -> Result<Vec<Line>, Box<dyn Error>> {
        let mut lines = Vec::new();
        
        for object in page.objects().iter() {
            if let Ok(path_object) = object.as_path_object() {
                let segments = path_object.segments();
                
                for i in 0..segments.len() {
                    if let Ok(segment) = segments.get(i) {
                        if segment.segment_type() == PdfPathSegmentType::LineTo {
                            if i > 0 {
                                if let Ok(prev_segment) = segments.get(i - 1) {
                                    let (x1, y1) = prev_segment.point();
                                    let (x2, y2) = segment.point();
                                    
                                    if (x1 - x2).abs() < 0.1 || (y1 - y2).abs() < 0.1 {
                                        lines.push(Line {
                                            x1, y1, x2, y2, page_number,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(lines)
    }

    fn detect_tables(&self, text_fragments: Vec<TextFragment>, lines: Vec<Line>, page_number: u16) -> Vec<Table> {
        let mut tables = Vec::new();
        let row_groups = self.group_fragments_by_row(&text_fragments);
        let table_regions = self.find_table_regions(&row_groups, &lines);
        
        for region in table_regions {
            if let Some(table) = self.extract_table_from_region(region, page_number) {
                tables.push(table);
            }
        }
        
        tables
    }

    fn group_fragments_by_row(&self, fragments: &[TextFragment]) -> Vec<Vec<TextFragment>> {
        let mut row_map: HashMap<OrderedFloat<f64>, Vec<TextFragment>> = HashMap::new();
        
        for fragment in fragments {
            let y_key = OrderedFloat((fragment.y / self.vertical_tolerance).round() * self.vertical_tolerance);
            row_map.entry(y_key).or_insert_with(Vec::new).push(fragment.clone());
        }
        
        let mut rows: Vec<(OrderedFloat<f64>, Vec<TextFragment>)> = row_map.into_iter().collect();
        rows.sort_by_key(|(y, _)| *y);
        
        rows.into_iter().map(|(_, mut fragments)| {
            fragments.sort_by(|a, b| OrderedFloat(a.x).cmp(&OrderedFloat(b.x)));
            fragments
        }).collect()
    }

    fn find_table_regions(&self, rows: &[Vec<TextFragment>], lines: &[Line]) -> Vec<Vec<Vec<TextFragment>>> {
        let mut regions = Vec::new();
        let mut current_region = Vec::new();
        let mut in_table = false;
        
        for (i, row) in rows.iter().enumerate() {
            if self.is_table_row(row, i, rows, lines) {
                if !in_table {
                    in_table = true;
                    current_region.clear();
                }
                current_region.push(row.clone());
            } else if in_table {
                if current_region.len() >= self.min_table_rows {
                    regions.push(current_region.clone());
                }
                current_region.clear();
                in_table = false;
            }
        }
        
        if in_table && current_region.len() >= self.min_table_rows {
            regions.push(current_region);
        }
        
        regions
    }

    fn is_table_row(&self, row: &[TextFragment], row_index: usize, all_rows: &[Vec<TextFragment>], lines: &[Line]) -> bool {
        if row.len() < self.min_table_cols {
            return false;
        }
        
        if row_index > 0 && row_index < all_rows.len() - 1 {
            let prev_row = &all_rows[row_index - 1];
            let next_row = &all_rows[row_index + 1];
            
            let mut aligned_count = 0;
            for fragment in row {
                let has_prev_aligned = prev_row.iter().any(|pf| 
                    (pf.x - fragment.x).abs() < self.horizontal_tolerance
                );
                let has_next_aligned = next_row.iter().any(|nf| 
                    (nf.x - fragment.x).abs() < self.horizontal_tolerance
                );
                
                if has_prev_aligned || has_next_aligned {
                    aligned_count += 1;
                }
            }
            
            if aligned_count < row.len() / 2 {
                return false;
            }
        }
        
        let row_y = row.first().map(|f| f.y).unwrap_or(0.0);
        let has_border = lines.iter().any(|line| {
            let is_horizontal = (line.y1 - line.y2).abs() < 0.1;
            let is_near = (line.y1 - row_y).abs() < 10.0;
            is_horizontal && is_near
        });
        
        true
    }

    fn extract_table_from_region(&self, region: Vec<Vec<TextFragment>>, page_number: u16) -> Option<Table> {
        if region.len() < self.min_table_rows {
            return None;
        }
        
        let mut all_x_positions = HashSet::new();
        for row in &region {
            for fragment in row {
                all_x_positions.insert(OrderedFloat(fragment.x));
            }
        }
        
        let mut column_positions: Vec<f64> = all_x_positions.into_iter()
            .map(|x| x.0)
            .collect();
        column_positions.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let mut merged_positions = Vec::new();
        let mut last_pos = None;
        for pos in column_positions {
            if let Some(last) = last_pos {
                if pos - last > self.horizontal_tolerance {
                    merged_positions.push(pos);
                }
            } else {
                merged_positions.push(pos);
            }
            last_pos = Some(pos);
        }
        column_positions = merged_positions;
        
        if column_positions.len() < self.min_table_cols {
            return None;
        }
        
        let rows = region.len();
        let cols = column_positions.len();
        let mut cells = vec![vec![None; cols]; rows];
        
        for (row_idx, row_fragments) in region.iter().enumerate() {
            for fragment in row_fragments {
                let col_idx = column_positions.iter()
                    .position(|&col_x| (fragment.x - col_x).abs() < self.horizontal_tolerance)
                    .unwrap_or_else(|| {
                        column_positions.iter()
                            .enumerate()
                            .min_by_key(|(_, &col_x)| OrderedFloat((fragment.x - col_x).abs()))
                            .map(|(idx, _)| idx)
                            .unwrap_or(0)
                    });
                
                if col_idx < cols {
                    if let Some(ref mut existing_cell) = cells[row_idx][col_idx] {
                        existing_cell.content.push(' ');
                        existing_cell.content.push_str(&fragment.text);
                        existing_cell.width = existing_cell.width.max(fragment.x + fragment.width - existing_cell.x);
                    } else {
                        cells[row_idx][col_idx] = Some(Cell {
                            content: fragment.text.clone(),
                            x: fragment.x,
                            y: fragment.y,
                            width: fragment.width,
                            height: fragment.height,
                            row_span: 1,
                            col_span: 1,
                            row_index: row_idx,
                            col_index: col_idx,
                        });
                    }
                }
            }
        }
        
        self.detect_and_mark_merged_cells(&mut cells);
        
        let min_x = region.iter()
            .flat_map(|row| row.iter().map(|f| f.x))
            .min_by_key(|&x| OrderedFloat(x))
            .unwrap_or(0.0);
        
        let max_x = region.iter()
            .flat_map(|row| row.iter().map(|f| f.x + f.width))
            .max_by_key(|&x| OrderedFloat(x))
            .unwrap_or(0.0);
        
        let min_y = region.first()
            .and_then(|row| row.first())
            .map(|f| f.y)
            .unwrap_or(0.0);
        
        let max_y = region.last()
            .and_then(|row| row.first())
            .map(|f| f.y + f.height)
            .unwrap_or(0.0);
        
        Some(Table {
            cells, rows, cols,
            x: min_x,
            y: min_y,
            width: max_x - min_x,
            height: max_y - min_y,
            page_number,
        })
    }

    fn detect_and_mark_merged_cells(&self, cells: &mut Vec<Vec<Option<Cell>>>) {
        let rows = cells.len();
        let cols = if rows > 0 { cells[0].len() } else { 0 };
        
        for row in 0..rows {
            for col in 0..cols {
                if let Some(ref mut cell) = cells[row][col] {
                    let mut span = 1;
                    
                    for next_col in (col + 1)..cols {
                        let next_empty = cells[row][next_col].is_none();
                        
                        if next_empty {
                            let column_has_content = (0..rows).any(|r| r != row && cells[r][next_col].is_some());
                            
                            if column_has_content {
                                span += 1;
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                    
                    cell.col_span = span;
                }
            }
        }
        
        for col in 0..cols {
            for row in 0..rows {
                if let Some(ref mut cell) = cells[row][col] {
                    if cell.col_span == 1 {
                        let mut span = 1;
                        
                        for next_row in (row + 1)..rows {
                            if cells[next_row][col].is_none() {
                                let row_has_content = (0..cols).any(|c| c != col && cells[next_row][c].is_some());
                                
                                if row_has_content {
                                    span += 1;
                                } else {
                                    break;
                                }
                            } else {
                                break;
                            }
                        }
                        
                        cell.row_span = span;
                    }
                }
            }
        }
    }

    fn export_to_csv(&self, table: &Table) -> String {
        let mut csv_writer = csv::Writer::from_writer(vec![]);
        
        for row in &table.cells {
            let row_values: Vec<String> = row.iter()
                .map(|cell| {
                    cell.as_ref()
                        .map(|c| c.content.clone())
                        .unwrap_or_else(|| String::new())
                })
                .collect();
            
            csv_writer.write_record(&row_values).unwrap();
        }
        
        String::from_utf8(csv_writer.into_inner().unwrap()).unwrap()
    }

    fn export_to_json(&self, table: &Table) -> Result<String, Box<dyn Error>> {
        let headers: Vec<String> = if !table.cells.is_empty() {
            table.cells[0].iter()
                .enumerate()
                .map(|(idx, cell)| {
                    cell.as_ref()
                        .map(|c| c.content.clone())
                        .unwrap_or_else(|| format!("Column_{}", idx + 1))
                })
                .collect()
        } else {
            Vec::new()
        };
        
        let mut json_rows = Vec::new();
        
        for row_idx in 1..table.rows {
            let mut row_obj = serde_json::Map::new();
            
            for (col_idx, header) in headers.iter().enumerate() {
                if col_idx < table.cols {
                    let value = table.cells[row_idx][col_idx]
                        .as_ref()
                        .map(|c| c.content.clone())
                        .unwrap_or_else(|| String::new());
                    
                    row_obj.insert(header.clone(), serde_json::Value::String(value));
                }
            }
            
            json_rows.push(serde_json::Value::Object(row_obj));
        }
        
        Ok(serde_json::to_string_pretty(&json_rows)?)
    }

    fn export_to_markdown(&self, table: &Table) -> String {
        let mut markdown = String::new();
        
        if !table.cells.is_empty() {
            markdown.push('|');
            for cell in &table.cells[0] {
                markdown.push(' ');
                markdown.push_str(&cell.as_ref().map(|c| c.content.as_str()).unwrap_or(""));
                markdown.push_str(" |");
            }
            markdown.push('\n');
            
            markdown.push('|');
            for _ in 0..table.cols {
                markdown.push_str(" --- |");
            }
            markdown.push('\n');
            
            for row_idx in 1..table.rows {
                markdown.push('|');
                for cell in &table.cells[row_idx] {
                    markdown.push(' ');
                    markdown.push_str(&cell.as_ref().map(|c| c.content.as_str()).unwrap_or(""));
                    markdown.push_str(" |");
                }
                markdown.push('\n');
            }
        }
        
        markdown
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <pdf_file> [output_format]", args[0]);
        eprintln!("Output formats: csv, json, markdown (default: all)");
        return Ok(());
    }
    
    let pdf_path = Path::new(&args[1]);
    let output_format = args.get(2).map(|s| s.as_str()).unwrap_or("all");
    
    if !pdf_path.exists() {
        eprintln!("Error: File '{}' not found", pdf_path.display());
        return Ok(());
    }
    
    println!("Extracting tables from: {}", pdf_path.display());
    
    let extractor = PDFTableExtractor::new()?;
    let tables = extractor.extract_tables_from_file(pdf_path)?;
    
    println!("Found {} tables in the document\n", tables.len());
    
    for (idx, table) in tables.iter().enumerate() {
        println!("=== Table {} (Page {}) ===", idx + 1, table.page_number);
        println!("Dimensions: {} rows × {} columns", table.rows, table.cols);
        println!("Position: ({:.2}, {:.2})\n", table.x, table.y);
        
        match output_format {
            "csv" => {
                println!("CSV Export:");
                println!("{}", extractor.export_to_csv(table));
            },
            "json" => {
                if let Ok(json) = extractor.export_to_json(table) {
                    println!("JSON Export:");
                    println!("{}", json);
                }
            },
            "markdown" => {
                println!("Markdown Export:");
                println!("{}", extractor.export_to_markdown(table));
            },
            _ => {
                println!("CSV Export:");
                println!("{}", extractor.export_to_csv(table));
                
                println!("\nMarkdown Export:");
                println!("{}", extractor.export_to_markdown(table));
                
                if let Ok(json) = extractor.export_to_json(table) {
                    println!("\nJSON Export:");
                    println!("{}", json);
                }
            }
        }
        
        println!("\n{}", "-".repeat(80));
        println!();
    }
    
    Ok(())
}