use anyhow::Result;
use std::path::Path;

mod character_matrix_engine;
use character_matrix_engine::CharacterMatrixEngine;

fn main() -> Result<()> {
    println!("🔍 Testing Ferrules Integration in Character Matrix Engine");
    println!("========================================================");

    // Get PDF path from command line argument or use default
    let args: Vec<String> = std::env::args().collect();
    let pdf_path_str = if args.len() > 1 {
        &args[1]
    } else {
        "chonker_test.pdf"
    };
    
    let test_pdf = Path::new(pdf_path_str);

    if !test_pdf.exists() {
        println!("❌ PDF not found: {}", pdf_path_str);
        return Ok(());
    }

    // Create optimized engine
    println!("\n📊 Creating optimized character matrix engine...");
    let engine = CharacterMatrixEngine::new_optimized(test_pdf)?;
    println!("✅ Engine created with:");
    println!("   • Character width: {:.2}", engine.char_width);
    println!("   • Character height: {:.2}", engine.char_height);

    // Process the PDF
    println!("\n🏃 Processing PDF with Ferrules vision model...");
    let start = std::time::Instant::now();

    match engine.process_pdf(test_pdf) {
        Ok(matrix) => {
            let elapsed = start.elapsed();
            println!(
                "✅ Successfully processed PDF in {:.2}s",
                elapsed.as_secs_f32()
            );
            println!("\n📊 Results:");
            println!("   • Matrix size: {}x{}", matrix.width, matrix.height);
            println!("   • Text regions detected: {}", matrix.text_regions.len());
            println!("   • Total text objects: {}", matrix.original_text.len());

            // Show detected regions
            println!("\n🔍 Detected text regions:");
            for (i, region) in matrix.text_regions.iter().take(5).enumerate() {
                println!(
                    "   {}. Position: ({}, {}) Size: {}x{} Confidence: {:.2}%",
                    i + 1,
                    region.bbox.x,
                    region.bbox.y,
                    region.bbox.width,
                    region.bbox.height,
                    region.confidence * 100.0
                );
                if !region.text_content.is_empty() {
                    println!(
                        "      Text: \"{}\"",
                        region.text_content.chars().take(50).collect::<String>()
                    );
                }
            }

            if matrix.text_regions.len() > 5 {
                println!("   ... and {} more regions", matrix.text_regions.len() - 5);
            }

            // Find where text actually appears by looking for the first non-space character
            let mut start_row = 0;
            let mut start_col = 0;
            let mut found_text = false;

            for (row_idx, row) in matrix.matrix.iter().enumerate() {
                for (col_idx, &ch) in row.iter().enumerate() {
                    if ch != ' ' && ch != '·' {
                        start_row = row_idx;
                        start_col = col_idx;
                        found_text = true;
                        break;
                    }
                }
                if found_text {
                    break;
                }
            }

            if found_text {
                println!(
                    "\n📝 Full character matrix ({}x{}):",
                    matrix.width, matrix.height
                );
                for (row_idx, row) in matrix.matrix.iter().enumerate() {
                    print!("{:3} ", row_idx); // Show row numbers
                    for &ch in row.iter() {
                        print!("{}", if ch == ' ' { '·' } else { ch });
                    }
                    println!();
                }
            } else {
                println!(
                    "\n📝 Full character matrix ({}x{} - no text found):",
                    matrix.width, matrix.height
                );
                for (row_idx, row) in matrix.matrix.iter().enumerate() {
                    print!("{:3} ", row_idx); // Show row numbers
                    for &ch in row.iter() {
                        print!("{}", if ch == ' ' { '·' } else { ch });
                    }
                    println!();
                }
            }

            #[cfg(feature = "ferrules")]
            println!("\n✨ Using Ferrules ML model for text detection!");

            #[cfg(not(feature = "ferrules"))]
            println!(
                "\n⚠️  Using flood-fill fallback (compile with --features ferrules for ML model)"
            );
        }
        Err(e) => {
            println!("❌ Failed to process PDF: {}", e);
        }
    }

    Ok(())
}
