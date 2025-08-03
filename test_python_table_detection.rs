#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! serde = { version = "1.0", features = ["derive"] }
//! serde_json = "1.0"
//! ```

use std::process::Command;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct TableDetectionResult {
    #[serde(default)]
    method: Option<String>,
    #[serde(default)]
    tables: Option<Vec<DetectedTable>>,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    help: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct DetectedTable {
    page: usize,
    bbox: Option<BBox>,
    rows: usize,
    cols: usize,
    data: serde_json::Value,
}

#[derive(Debug, Deserialize, Serialize)]
struct BBox {
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
}

fn detect_tables_python(pdf_path: &Path) -> Result<TableDetectionResult, String> {
    let output = Command::new("python3")
        .arg("pdf_table_detector.py")
        .arg(pdf_path)
        .output()
        .map_err(|e| format!("Failed to run Python script: {}", e))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Python script failed: {}", stderr));
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&stdout)
        .map_err(|e| format!("Failed to parse JSON: {} - Output was: {}", e, stdout))
}

fn main() {
    println!("🔍 Testing Python-based table detection...\n");
    
    // Test PDF path
    let test_pdf = "/Users/jack/Downloads/journal_entry-5.pdf";
    
    if !Path::new(test_pdf).exists() {
        println!("❌ Test PDF not found: {}", test_pdf);
        println!("Please provide a valid PDF path");
        return;
    }
    
    println!("📄 Testing with: {}", test_pdf);
    
    match detect_tables_python(Path::new(test_pdf)) {
        Ok(result) => {
            if let Some(error) = result.error {
                println!("❌ Error: {}", error);
                if let Some(help) = result.help {
                    println!("💡 {}", help);
                }
            } else if let Some(tables) = result.tables {
                println!("✅ Success using method: {}", result.method.unwrap_or_default());
                println!("📊 Found {} tables", tables.len());
                
                for (i, table) in tables.iter().enumerate() {
                    println!("\n📋 Table {} on page {}:", i + 1, table.page);
                    println!("   Size: {} rows × {} columns", table.rows, table.cols);
                    if let Some(bbox) = &table.bbox {
                        println!("   Location: ({:.1}, {:.1}) to ({:.1}, {:.1})", 
                               bbox.x1, bbox.y1, bbox.x2, bbox.y2);
                    }
                }
            } else {
                println!("⚠️ No tables found in the PDF");
            }
        }
        Err(e) => {
            println!("❌ Failed to detect tables: {}", e);
        }
    }
    
    println!("\n💡 To integrate this into CHONKER 5:");
    println!("1. Add a 'Detect Tables' button");
    println!("2. Call the Python script when clicked");
    println!("3. Overlay table boundaries on the PDF view");
    println!("4. Allow extraction of individual tables");
}