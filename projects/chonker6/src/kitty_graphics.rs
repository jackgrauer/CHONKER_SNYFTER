/// Kitty Graphics Protocol implementation for PDF display
/// Based on official Kitty graphics protocol specification
use std::io::{stdout, Write};

/// Send image to Kitty terminal using graphics protocol
pub fn send_image_to_kitty(
    base64_data: &str, 
    width: u32, 
    height: u32,
    cursor_row: u16, 
    cursor_col: u16
) -> Result<(), Box<dyn std::error::Error>> {
    // Clear any existing images with our ID first
    let mut out = stdout();
    out.write_all(b"\x1b_Ga=d,i=pdf\x1b\\")?;
    out.flush()?;
    
    // Debug information
    eprintln!("[KITTY] Sending image: {}x{} at row={}, col={}, data_len={}", 
        width, height, cursor_row, cursor_col, base64_data.len());
    
    // Kitty Graphics Protocol specification:
    // ESC _G <control_data> ; <base64_data> ESC \
    // For positioning, we use cursor movement instead of X,Y parameters
    // which can be unreliable in some implementations
    
    // Position cursor where we want the image
    write!(out, "\x1b[{};{}H", cursor_row, cursor_col)?;
    out.flush()?;
    
    const CHUNK_SIZE: usize = 4096;
    
    if base64_data.len() <= CHUNK_SIZE {
        // Single chunk transmission with absolute positioning
        // a=T: transmit and display
        // f=100: PNG format  
        // i=pdf: image ID (unique for persistence)
        // s=width, v=height: dimensions
        // X=col, Y=row: absolute positioning in pixels
        // C=1: don't move cursor after display
        // z=1: higher z-index to stay on top
        // Use more accurate pixel conversion and add placement control
        // Most terminals: ~8-12px wide, ~16-24px tall per character cell
        let pixel_x = (cursor_col - 1) * 9;  // Conservative estimate
        let pixel_y = (cursor_row - 1) * 18; // Conservative estimate
        let cmd = format!("\x1b_Ga=T,f=100,i=pdf,s={},v={},X={},Y={},C=1,z=10,U=1;{}\x1b\\", 
            width, height, pixel_x, pixel_y, base64_data);
        out.write_all(cmd.as_bytes())?;
    } else {
        // Multi-chunk transmission for large images
        let chunks: Vec<&str> = base64_data.as_bytes()
            .chunks(CHUNK_SIZE)
            .map(|chunk| std::str::from_utf8(chunk).unwrap_or(""))
            .collect();
        
        eprintln!("[KITTY] Multi-chunk transmission: {} chunks", chunks.len());
        
        // First chunk with metadata and positioning
        if !chunks.is_empty() {
            let pixel_x = (cursor_col - 1) * 9;  // Conservative estimate
            let pixel_y = (cursor_row - 1) * 18; // Conservative estimate
            let cmd = format!("\x1b_Ga=T,f=100,i=pdf,s={},v={},X={},Y={},C=1,z=10,U=1,m=1;{}\x1b\\", 
                width, height, pixel_x, pixel_y, chunks[0]);
            out.write_all(cmd.as_bytes())?;
        }
        
        // Middle chunks
        for (i, chunk) in chunks[1..chunks.len().saturating_sub(1)].iter().enumerate() {
            let cmd = format!("\x1b_Gm=1;{}\x1b\\", chunk);
            out.write_all(cmd.as_bytes())?;
            // Small delay between chunks for stability
            if i % 10 == 0 {
                out.flush()?;
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
        }
        
        // Final chunk  
        if chunks.len() > 1 {
            let cmd = format!("\x1b_Gm=0;{}\x1b\\", chunks[chunks.len()-1]);
            out.write_all(cmd.as_bytes())?;
        }
    }
    
    // Ensure all data is sent to stdout
    out.flush()?;
    
    // Give Kitty time to process the image
    std::thread::sleep(std::time::Duration::from_millis(100));
    
    eprintln!("[KITTY] Image transmission complete");
    
    Ok(())
}

/// Clear all images with specific ID
pub fn clear_kitty_images(image_id: &str) {
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