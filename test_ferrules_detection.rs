use anyhow::Result;
use ferrules_core::layout::{ORTLayoutParser, ORTConfig};
use image::io::Reader as ImageReader;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔍 Testing Ferrules Layout Detection");
    println!("====================================");
    
    // Load a test image (you can use a PDF page rendered to image)
    let test_image_path = "test_page.png"; // You'll need to create this
    
    if std::path::Path::new(test_image_path).exists() {
        let img = ImageReader::open(test_image_path)?.decode()?;
        
        // Initialize Ferrules
        let config = ORTConfig::default();
        let parser = ORTLayoutParser::new(&config)?;
        
        println!("✅ Ferrules initialized successfully");
        println!("🏃 Running layout detection...");
        
        // Detect layout
        let layout_bboxes = parser.parse_layout_async(&img, 1.0).await?;
        
        println!("\n📦 Detected {} layout elements:", layout_bboxes.len());
        
        for (i, bbox) in layout_bboxes.iter().enumerate() {
            println!("  {}. {} - confidence: {:.2}%", 
                i + 1, 
                bbox.label, 
                bbox.proba * 100.0
            );
            println!("     Position: ({:.0}, {:.0}) to ({:.0}, {:.0})",
                bbox.bbox.x0, bbox.bbox.y0, 
                bbox.bbox.x1, bbox.bbox.y1
            );
            
            if bbox.is_text_block() {
                println!("     ✅ This is a text region!");
            }
        }
    } else {
        println!("⚠️  No test image found at {}", test_image_path);
        println!("   Please render a PDF page to test_page.png first");
        
        // Quick way to create test image from PDF
        println!("\n💡 You can create a test image with:");
        println!("   pdftoppm -png -f 1 -l 1 chonker_test.pdf test_page");
    }
    
    Ok(())
}