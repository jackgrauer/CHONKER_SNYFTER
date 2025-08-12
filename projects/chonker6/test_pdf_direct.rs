use pdfium_render::prelude::*;
use image::{ImageBuffer, RgbaImage};
use base64::Engine;
use std::io::{stdout, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Direct PDF to Kitty test");
    println!("TERM={}", std::env::var("TERM").unwrap_or_default());
    println!("KITTY_WINDOW_ID={}", std::env::var("KITTY_WINDOW_ID").unwrap_or_default());
    
    // Get PDF path from args or use a test file
    let pdf_path = std::env::args().nth(1).unwrap_or_else(|| {
        "/Users/jack/Downloads/test.pdf".to_string()
    });
    
    println!("Loading PDF: {}", pdf_path);
    
    // Initialize PDFium with the library path
    let bindings = Pdfium::bind_to_library(
        Pdfium::pdfium_platform_library_name_at_path("./lib/")
    )?;
    
    // Load PDF
    let document = bindings.load_pdf_from_file(&pdf_path, None)?;
    let page = document.pages().get(0)?;
    
    println!("PDF loaded - Page size: {}x{} pts", page.width().value, page.height().value);
    
    // Render to reasonable size
    let render_config = PdfRenderConfig::new()
        .set_target_width(600)
        .set_target_height(800)
        .rotate_if_landscape(PdfPageRenderRotation::None, false);
    
    let bitmap = page.render_with_config(&render_config)?;
    let rgba_bytes = bitmap.as_rgba_bytes();
    
    // Convert to PNG
    let img: RgbaImage = ImageBuffer::from_raw(600, 800, rgba_bytes)
        .ok_or("Failed to create image buffer")?;
    
    let mut png_data = Vec::new();
    {
        use std::io::Cursor;
        let mut cursor = Cursor::new(&mut png_data);
        img.write_to(&mut cursor, image::ImageFormat::Png)?;
    }
    
    let base64_png = base64::engine::general_purpose::STANDARD.encode(&png_data);
    
    println!("PNG created: {} bytes, base64: {} chars", png_data.len(), base64_png.len());
    
    // Clear screen and display
    print!("\x1b[2J");  // Clear screen
    print!("\x1b[H");   // Home cursor
    
    // Clear any existing images
    print!("\x1b_Ga=d\x1b\\");
    stdout().flush()?;
    
    // Position at row 3, col 3
    print!("\x1b[3;3H");
    
    // Send image with z=-1 to go behind any text
    print!("\x1b_Ga=T,f=100,i=1,s=600,v=800,z=-1;{}\x1b\\", base64_png);
    stdout().flush()?;
    
    // Add some text overlay to test z-index
    print!("\x1b[5;5H\x1b[47m\x1b[30m PDF PAGE 1 \x1b[0m");
    print!("\x1b[45;1H");  // Move cursor to bottom
    println!("\nPDF displayed! Press Ctrl+C to exit");
    
    // Keep running
    std::thread::sleep(std::time::Duration::from_secs(60));
    
    Ok(())
}