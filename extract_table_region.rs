#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! image = "0.24"
//! pdf-render = "0.1"
//! ```

use std::env;
use std::process::Command;
use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 8 {
        eprintln!("Usage: {} input.pdf page_num x y width height output.png", args[0]);
        eprintln!("Example: {} document.pdf 1 100 200 400 150 table.png", args[0]);
        std::process::exit(1);
    }
    
    let pdf_path = &args[1];
    let page_num = &args[2];
    let x: u32 = args[3].parse()?;
    let y: u32 = args[4].parse()?;
    let width: u32 = args[5].parse()?;
    let height: u32 = args[6].parse()?;
    let output_path = &args[7];
    
    // First, extract the page as a high-res PNG using pdftoppm
    let temp_img = format!("/tmp/pdf_page_{}.png", page_num);
    
    println!("Extracting page {} from PDF...", page_num);
    let output = Command::new("pdftoppm")
        .args(&["-png", "-r", "300", "-f", page_num, "-l", page_num, pdf_path])
        .output()?;
    
    if !output.status.success() {
        eprintln!("Failed to extract PDF page: {:?}", String::from_utf8_lossy(&output.stderr));
        std::process::exit(1);
    }
    
    // pdftoppm outputs to stdout, save it to a file
    std::fs::write(&temp_img, output.stdout)?;
    
    // Load the image
    println!("Loading extracted page image...");
    let img = image::open(&temp_img)?;
    
    // Get image dimensions for coordinate conversion
    let (img_width, img_height) = img.dimensions();
    println!("Page image size: {}x{}", img_width, img_height);
    
    // PDF coordinates are bottom-left origin, image coordinates are top-left
    // Convert Y coordinate
    let img_y = if y < img_height {
        img_height - y - height
    } else {
        0
    };
    
    println!("Cropping region: x={}, y={} (converted from {}), width={}, height={}", 
        x, img_y, y, width, height);
    
    // Crop the region
    let cropped = img.crop_imm(x, img_y, width, height);
    
    // Save the cropped image
    cropped.save(output_path)?;
    println!("Saved cropped region to {}", output_path);
    
    // Clean up
    std::fs::remove_file(&temp_img)?;
    
    Ok(())
}