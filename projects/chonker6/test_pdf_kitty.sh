#!/bin/bash

# Test script for PDF rendering via Kitty graphics protocol

echo "Testing PDF rendering with Kitty graphics protocol..."
echo "TERM=$TERM"
echo "KITTY_WINDOW_ID=$KITTY_WINDOW_ID"
echo ""

# Create a simple test program that renders a PDF and sends it to Kitty
cat > test_pdf_kitty.rs << 'EOF'
use pdfium_render::prelude::*;
use image::{ImageBuffer, RgbaImage};
use base64::Engine;
use std::io::{stdout, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Find a PDF to test with
    let pdf_path = std::env::args().nth(1).unwrap_or_else(|| {
        // Try to find a PDF in common locations
        if std::path::Path::new("test.pdf").exists() {
            "test.pdf".to_string()
        } else if std::path::Path::new("/Users/jack/Downloads/test.pdf").exists() {
            "/Users/jack/Downloads/test.pdf".to_string()
        } else {
            panic!("Please provide a PDF file path as argument");
        }
    });
    
    println!("Loading PDF: {}", pdf_path);
    
    // Initialize PDFium
    let bindings = Pdfium::bind_to_library(
        Pdfium::pdfium_platform_library_name_at_path("./lib/")
    )?;
    
    // Load the PDF
    let document = bindings.load_pdf_from_file(&pdf_path, None)?;
    let page = document.pages().get(0)?;
    
    println!("PDF loaded - Page size: {}x{} pts", page.width().value, page.height().value);
    
    // Render at reasonable size for terminal
    let render_config = PdfRenderConfig::new()
        .set_target_width(800)
        .set_target_height(600)
        .rotate_if_landscape(PdfPageRenderRotation::None, false);
    
    let bitmap = page.render_with_config(&render_config)?;
    let rgba_bytes = bitmap.as_rgba_bytes();
    
    // Convert to PNG
    let img: RgbaImage = ImageBuffer::from_raw(800, 600, rgba_bytes)
        .ok_or("Failed to create image buffer")?;
    
    let mut png_data = Vec::new();
    {
        use std::io::Cursor;
        let mut cursor = Cursor::new(&mut png_data);
        img.write_to(&mut cursor, image::ImageFormat::Png)?;
    }
    
    let base64_png = base64::engine::general_purpose::STANDARD.encode(&png_data);
    
    println!("PNG created: {} bytes, base64: {} chars", png_data.len(), base64_png.len());
    
    // Send to Kitty
    print!("\x1b_Ga=d\x1b\\"); // Clear existing images
    stdout().flush()?;
    
    // Position cursor
    print!("\x1b[5;5H"); // Row 5, Col 5
    stdout().flush()?;
    
    // Send image
    print!("\x1b_Ga=T,f=100,s=800,v=600;{}\x1b\\", base64_png);
    stdout().flush()?;
    
    println!("\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n"); // Make space
    println!("Image sent to Kitty! You should see the PDF page above.");
    
    Ok(())
}
EOF

# Compile the test
echo "Compiling test program..."
DYLD_LIBRARY_PATH=./lib rustc test_pdf_kitty.rs \
    -L ./target/release/deps \
    --extern pdfium_render=./target/release/deps/libpdfium_render.rlib \
    --extern image=./target/release/deps/libimage.rlib \
    --extern base64=./target/release/deps/libbase64.rlib \
    --edition 2021 \
    -o test_pdf_kitty 2>/dev/null

if [ $? -ne 0 ]; then
    echo "Compilation failed. Trying with cargo..."
    # Fallback: create a minimal Cargo project
    mkdir -p /tmp/pdf_kitty_test
    cd /tmp/pdf_kitty_test
    
    cat > Cargo.toml << 'TOML'
[package]
name = "pdf_kitty_test"
version = "0.1.0"
edition = "2021"

[dependencies]
pdfium-render = "0.8"
image = "0.24"
base64 = "0.21"
TOML
    
    mkdir -p src
    cp /Users/jack/chonker6/projects/chonker6/test_pdf_kitty.rs src/main.rs
    
    DYLD_LIBRARY_PATH=/Users/jack/chonker6/projects/chonker6/lib cargo build --release
    cp target/release/pdf_kitty_test /Users/jack/chonker6/projects/chonker6/
    cd /Users/jack/chonker6/projects/chonker6
    DYLD_LIBRARY_PATH=./lib ./pdf_kitty_test "$@"
else
    DYLD_LIBRARY_PATH=./lib ./test_pdf_kitty "$@"
fi