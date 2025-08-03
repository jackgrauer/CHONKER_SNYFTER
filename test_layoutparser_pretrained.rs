#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! layoutparser-ort = { git = "https://github.com/styrowolf/layoutparser-ort.git" }
//! ort = { version = "2.0.0-rc.2", features = ["download-binaries"] }
//! image = "0.25"
//! ```

use layoutparser_ort::{
    models::{YOLOXModel, YOLOXPretrainedModels},
    Element, ElementType,
};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("🔍 Testing layoutparser-ort with pretrained model...");
    
    // First, we need a test image. Let's create a simple one or use an existing PNG
    println!("\n📥 Loading pretrained YOLOX model...");
    
    match YOLOXModel::pretrained(YOLOXPretrainedModels::Tiny) {
        Ok(model) => {
            println!("✅ Successfully loaded pretrained model!");
            
            // To use this with PDFs, we would need to:
            // 1. Convert PDF pages to images
            // 2. Run the model on each page image
            
            println!("\n📋 Model is ready for use!");
            println!("\nTo integrate with CHONKER 5:");
            println!("1. Add PDF to image conversion (using pdftoppm or similar)");
            println!("2. Run layoutparser on each page image");
            println!("3. Filter results for ElementType::Table");
            println!("4. Map image coordinates back to PDF coordinates");
            
            // If we had a test image:
            // let img = image::open("test_page.png")?;
            // let predictions = model.predict(&img)?;
            // 
            // for element in predictions {
            //     if element.element_type == ElementType::Table {
            //         println!("Found table at: {:?}", element.bbox);
            //     }
            // }
        }
        Err(e) => {
            println!("❌ Failed to load pretrained model: {}", e);
            println!("\nThis might be because:");
            println!("- Model download failed");
            println!("- Network issues");
            println!("- Missing ONNX runtime binaries");
            
            println!("\nTry installing ONNX Runtime first:");
            println!("brew install onnxruntime");
        }
    }
    
    Ok(())
}