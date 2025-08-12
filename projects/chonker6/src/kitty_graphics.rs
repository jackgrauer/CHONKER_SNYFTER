/// Kitty Graphics Protocol implementation for PDF display
/// Based on official Kitty graphics protocol specification
use std::io::{stdout, Write};

/// Send image to Kitty terminal using direct display method
pub fn send_image_to_kitty(
    base64_data: &str, 
    width: u32, 
    height: u32,
    cursor_row: u16, 
    cursor_col: u16
) -> Result<(), Box<dyn std::error::Error>> {
    let mut out = stdout();
    
    // Clear any existing images first
    write!(out, "\x1b_Ga=d,i=1\x1b\\")?;
    out.flush()?;
    
    // Move cursor to position
    write!(out, "\x1b[{};{}H", cursor_row, cursor_col)?;
    
    const CHUNK_SIZE: usize = 4096;
    
    if base64_data.len() <= CHUNK_SIZE {
        // Single chunk - direct display
        // a=T: transmit and display immediately
        // f=100: PNG format
        // i=1: image ID for tracking
        // s,v: dimensions
        // z index negative to go behind text
        let cmd = format!("\x1b_Ga=T,f=100,i=1,s={},v={},z=-1;{}\x1b\\", 
            width, height, base64_data);
        out.write_all(cmd.as_bytes())?;
    } else {
        // Multi-chunk for large images
        let chunks: Vec<&str> = base64_data.as_bytes()
            .chunks(CHUNK_SIZE)
            .map(|chunk| std::str::from_utf8(chunk).unwrap_or(""))
            .collect();
        
        
        // First chunk with all metadata
        if !chunks.is_empty() {
            let cmd = format!("\x1b_Ga=T,f=100,i=1,s={},v={},z=-1,m=1;{}\x1b\\", 
                width, height, chunks[0]);
            out.write_all(cmd.as_bytes())?;
            out.flush()?;
        }
        
        // Middle chunks
        for chunk in chunks[1..chunks.len().saturating_sub(1)].iter() {
            let cmd = format!("\x1b_Gm=1;{}\x1b\\", chunk);
            out.write_all(cmd.as_bytes())?;
        }
        
        // Final chunk
        if chunks.len() > 1 {
            let cmd = format!("\x1b_Gm=0;{}\x1b\\", chunks[chunks.len()-1]);
            out.write_all(cmd.as_bytes())?;
        }
    }
    
    out.flush()?;
    Ok(())
}

/// Clear all images with specific ID
pub fn clear_kitty_images(image_id: u32) {
    let mut out = stdout();
    let cmd = format!("\x1b_Ga=d,i={}\x1b\\", image_id);
    let _ = out.write_all(cmd.as_bytes());
    let _ = out.flush();
}

/// Test if terminal supports Kitty Graphics Protocol
pub fn test_kitty_graphics() -> bool {
    // Check environment variables
    let is_kitty = std::env::var("TERM")
        .map(|term| term.contains("kitty") || term == "xterm-kitty")
        .unwrap_or(false);
    
    let has_kitty_window = std::env::var("KITTY_WINDOW_ID").is_ok();
    
    // Allow forcing for testing
    let forced = std::env::var("CHONKER6_FORCE_KITTY").is_ok();
    
    is_kitty || has_kitty_window || forced
}

/// Alternative approach: render image to a specific area with persistence
pub fn render_pdf_in_area(
    base64_data: &str,
    width_px: u32,
    height_px: u32,
    area: &ratatui::layout::Rect,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut out = stdout();
    
    // Calculate cell-based positioning
    let cell_x = area.x;
    let cell_y = area.y;
    
    
    // First clear the specific area
    write!(out, "\x1b_Ga=d,i=99\x1b\\")?;
    
    // Save state
    write!(out, "\x1b7")?;
    
    // Position at top-left of area
    write!(out, "\x1b[{};{}H", cell_y + 1, cell_x + 1)?;
    
    // Send image with specific ID that won't conflict
    // Using z=-1 to ensure it stays behind text
    let cmd = format!("\x1b_Ga=T,f=100,i=99,s={},v={},z=-1,C=1;{}\x1b\\",
        width_px, height_px, base64_data);
    
    out.write_all(cmd.as_bytes())?;
    
    // Restore state
    write!(out, "\x1b8")?;
    
    out.flush()?;
    
    Ok(())
}