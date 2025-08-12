use std::path::PathBuf;
use pdfium_render::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing PDF rendering for Kitty terminal...");
    
    // Initialize PDFium
    let pdfium = Pdfium::new(
        Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./lib"))
            .or_else(|_| Pdfium::bind_to_system_library())?
    );
    
    // Test PDF path
    let test_pdf = PathBuf::from("/Users/jack/chonker5/chonker_test.pdf");
    
    if !test_pdf.exists() {
        println!("Test PDF not found at {:?}", test_pdf);
        println!("Please provide a PDF file for testing.");
        return Ok(());
    }
    
    // Load PDF
    let document = pdfium.load_pdf_from_file(&test_pdf, None)?;
    println!("Loaded PDF with {} pages", document.pages().len());
    
    // Render first page
    let page = document.pages().get(0)?;
    let page_width = page.width().value;
    let page_height = page.height().value;
    println!("Page dimensions: {:.1} x {:.1} pts", page_width, page_height);
    
    // Calculate render dimensions (fit to ~800px width)
    let target_width = 800u32;
    let aspect_ratio = page_width / page_height;
    let target_height = (target_width as f32 / aspect_ratio) as u32;
    
    println!("Rendering at {}x{} pixels...", target_width, target_height);
    
    // Render to bitmap
    let bitmap = page.render_with_config(
        &PdfRenderConfig::new()
            .set_target_width(target_width as i32)
            .set_target_height(target_height as i32)
            .rotate_if_landscape(PdfPageRenderRotation::None, false)
    )?;
    
    // Get RGBA bytes
    let rgba_bytes = bitmap.as_rgba_bytes();
    println!("Generated {} bytes of RGBA data", rgba_bytes.len());
    
    // Test Kitty protocol output
    if std::env::var("KITTY_WINDOW_ID").is_ok() || std::env::var("TERM").unwrap_or_default().contains("kitty") {
        println!("\nKitty terminal detected!");
        println!("Displaying image using Kitty graphics protocol...\n");
        
        use base64::Engine;
        let base64_image = base64::engine::general_purpose::STANDARD.encode(&rgba_bytes);
        
        // Display using Kitty protocol
        print!("\x1b_Ga=T,f=32,s={},{},t=d;{}\x1b\\", target_width, target_height, base64_image);
        
        println!("\n\nImage should be displayed above if you're using Kitty terminal.");
    } else {
        println!("\nNot running in Kitty terminal. Set KITTY_WINDOW_ID=1 to test Kitty mode.");
    }
    
    Ok(())
}