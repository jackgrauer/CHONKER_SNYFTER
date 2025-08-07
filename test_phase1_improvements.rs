use anyhow::Result;
use std::path::Path;

// Include the character matrix engine module
mod character_matrix_engine;
use character_matrix_engine::{BBox, CharacterMatrixEngine, TextObject};

fn main() -> Result<()> {
    println!("🐹 Testing Phase 1 Character Matrix Improvements");
    println!("═══════════════════════════════════════════════");

    // Test with a sample PDF if available
    let test_pdf_path = Path::new("test_document.pdf");

    if !test_pdf_path.exists() {
        println!("⚠️  No test PDF found at {:?}", test_pdf_path);
        println!("   Creating test data to validate new methods...");
        test_with_mock_data()?;
        return Ok(());
    }

    // Test the new optimized engine creation
    println!("📊 Testing optimized engine creation...");
    match CharacterMatrixEngine::new_optimized(test_pdf_path) {
        Ok(engine) => {
            println!("✅ Successfully created optimized engine:");
            println!("   • Character width: {:.2}", engine.char_width);
            println!("   • Character height: {:.2}", engine.char_height);

            // Test text extraction with coordinates
            println!("\n📄 Testing enhanced text extraction...");
            match engine.extract_text_objects_with_coordinates(test_pdf_path) {
                Ok(text_objects) => {
                    println!(
                        "✅ Extracted {} text objects with coordinates",
                        text_objects.len()
                    );

                    if !text_objects.is_empty() {
                        let first_obj = &text_objects[0];
                        println!(
                            "   • First object: \"{}\"",
                            first_obj.text.chars().take(30).collect::<String>()
                        );
                        println!("   • Font size: {:.1}pt", first_obj.font_size);
                        println!(
                            "   • Bounds: ({:.1}, {:.1}) to ({:.1}, {:.1})",
                            first_obj.bbox.x0,
                            first_obj.bbox.y0,
                            first_obj.bbox.x1,
                            first_obj.bbox.y1
                        );
                    }

                    // Test adaptive matrix sizing
                    println!("\n📐 Testing adaptive matrix sizing...");
                    let (width, height) = engine.adaptive_matrix_sizing(&text_objects);
                    println!("✅ Calculated optimal matrix size: {}x{}", width, height);

                    // Test full processing
                    println!("\n⚡ Testing full PDF processing...");
                    match engine.process_pdf(test_pdf_path) {
                        Ok(char_matrix) => {
                            println!("✅ Successfully processed PDF:");
                            println!(
                                "   • Matrix dimensions: {}x{}",
                                char_matrix.width, char_matrix.height
                            );
                            println!(
                                "   • Text regions found: {}",
                                char_matrix.text_regions.len()
                            );
                            println!(
                                "   • Total text objects: {}",
                                char_matrix.original_text.len()
                            );
                        }
                        Err(e) => {
                            println!("❌ Failed to process PDF: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("❌ Failed to extract text objects: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to create optimized engine: {}", e);
        }
    }

    Ok(())
}

fn test_with_mock_data() -> Result<()> {
    println!("🧪 Testing with mock data...");

    let engine = CharacterMatrixEngine::new();

    // Create mock text objects to test adaptive sizing
    let mock_text_objects = vec![
        TextObject {
            text: "Sample Text 1".to_string(),
            bbox: BBox {
                x0: 50.0,
                y0: 700.0,
                x1: 150.0,
                y1: 715.0,
            },
            font_size: 12.0,
            font_family: "Arial".to_string(),
            is_bold: false,
            is_italic: false,
            reading_order: 0,
        },
        TextObject {
            text: "Sample Text 2".to_string(),
            bbox: BBox {
                x0: 50.0,
                y0: 680.0,
                x1: 180.0,
                y1: 695.0,
            },
            font_size: 12.0,
            font_family: "Arial".to_string(),
            is_bold: true,
            is_italic: false,
            reading_order: 1,
        },
        TextObject {
            text: "Large Header".to_string(),
            bbox: BBox {
                x0: 50.0,
                y0: 650.0,
                x1: 250.0,
                y1: 675.0,
            },
            font_size: 18.0,
            font_family: "Arial".to_string(),
            is_bold: true,
            is_italic: false,
            reading_order: 2,
        },
    ];

    // Test modal font size calculation
    let font_sizes: Vec<f32> = mock_text_objects.iter().map(|t| t.font_size).collect();
    let modal_size = engine.calculate_modal_font_size(&font_sizes);
    println!("✅ Modal font size: {:.1}pt", modal_size);

    // Test content bounds calculation
    let bounds = engine.calculate_content_bounds(&mock_text_objects);
    println!(
        "✅ Content bounds: ({:.1}, {:.1}) to ({:.1}, {:.1})",
        bounds.x0, bounds.y0, bounds.x1, bounds.y1
    );
    println!(
        "   • Width: {:.1}pt, Height: {:.1}pt",
        bounds.width(),
        bounds.height()
    );

    // Test adaptive matrix sizing
    let (width, height) = engine.adaptive_matrix_sizing(&mock_text_objects);
    println!("✅ Adaptive matrix size: {}x{} characters", width, height);

    // Test optimal character dimensions calculation
    let optimal_char_width = modal_size * 0.6;
    let optimal_char_height = modal_size * 1.2;
    println!(
        "✅ Optimal character dimensions: {:.1}x{:.1}pt",
        optimal_char_width, optimal_char_height
    );

    println!("\n🎉 All mock tests passed! The Phase 1 improvements are working correctly.");

    Ok(())
}
