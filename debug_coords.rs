#!/usr/bin/env rust-script
//! Debug coordinate systems from ferrules output
//! ```cargo
//! [dependencies]
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//! ```

use std::fs;
use std::process::Command;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct FerrulesDocument {
    blocks: Vec<Block>,
    pages: Vec<Page>,
}

#[derive(Debug, Deserialize)]
struct Block {
    bbox: BBox,
    id: u32,
    kind: BlockKind,
    pages_id: Vec<u32>,
}

#[derive(Debug, Deserialize)]
struct BBox {
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
}

#[derive(Debug, Deserialize)]
struct BlockKind {
    block_type: String,
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Page {
    height: f64,
    width: f64,
    id: u32,
}

fn main() {
    let pdf_path = std::env::args().nth(1).expect("Usage: debug_coords.rs <pdf_file>");
    
    // Run ferrules
    println!("Running ferrules on {}...", pdf_path);
    let output = Command::new("ferrules")
        .arg(&pdf_path)
        .arg("-o")
        .arg("/tmp/ferrules_debug")
        .output()
        .expect("Failed to run ferrules");
    
    if !output.status.success() {
        eprintln!("Ferrules failed: {}", String::from_utf8_lossy(&output.stderr));
        return;
    }
    
    // Find the JSON file
    let json_path = std::fs::read_dir("/tmp/ferrules_debug")
        .unwrap()
        .filter_map(|e| e.ok())
        .find(|e| e.path().extension().map_or(false, |ext| ext == "json"))
        .expect("No JSON file found")
        .path();
    
    // Parse JSON
    let content = fs::read_to_string(&json_path).expect("Failed to read JSON");
    let doc: FerrulesDocument = serde_json::from_str(&content).expect("Failed to parse JSON");
    
    // Analyze coordinate system
    println!("\n=== COORDINATE SYSTEM ANALYSIS ===");
    println!("Number of pages: {}", doc.pages.len());
    
    for (i, page) in doc.pages.iter().enumerate() {
        println!("\nPage {} (id: {}):", i + 1, page.id);
        println!("  Dimensions: {}x{}", page.width, page.height);
        
        // Find all blocks on this page
        let page_blocks: Vec<&Block> = doc.blocks.iter()
            .filter(|b| b.pages_id.contains(&page.id))
            .collect();
        
        println!("  Blocks on this page: {}", page_blocks.len());
        
        // Find min/max coordinates
        let mut min_y = f64::MAX;
        let mut max_y = f64::MIN;
        
        for block in &page_blocks {
            min_y = min_y.min(block.bbox.y0).min(block.bbox.y1);
            max_y = max_y.max(block.bbox.y0).max(block.bbox.y1);
            
            if let Some(text) = &block.kind.text {
                let preview = if text.len() > 30 { 
                    format!("{}...", &text[..30]) 
                } else { 
                    text.clone() 
                };
                println!("    Block {}: y0={:.2}, y1={:.2} \"{}\"", 
                    block.id, block.bbox.y0, block.bbox.y1, preview);
            }
        }
        
        println!("  Y coordinate range: {:.2} to {:.2}", min_y, max_y);
        println!("  Y span: {:.2}", max_y - min_y);
        
        // Check coordinate system
        if i > 0 {
            // Compare with previous page
            let prev_page_blocks: Vec<&Block> = doc.blocks.iter()
                .filter(|b| b.pages_id.contains(&doc.pages[i-1].id))
                .collect();
            
            let prev_max_y = prev_page_blocks.iter()
                .map(|b| b.bbox.y1.max(b.bbox.y0))
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap_or(0.0);
            
            println!("  Distance from previous page max Y: {:.2}", min_y - prev_max_y);
            
            // Check if coordinates are cumulative
            if min_y > prev_max_y {
                println!("  ✓ Coordinates appear to be CUMULATIVE across pages");
            } else {
                println!("  ✗ Coordinates might be PAGE-RELATIVE");
            }
        }
    }
    
    // Check origin
    let first_block = doc.blocks.first().unwrap();
    if first_block.bbox.y0 < 50.0 {
        println!("\n✓ Origin appears to be TOP-LEFT (small Y values at top)");
    } else {
        println!("\n✗ Origin might be BOTTOM-LEFT (large Y values at top)");
    }
    
    // Clean up
    let _ = fs::remove_dir_all("/tmp/ferrules_debug");
}