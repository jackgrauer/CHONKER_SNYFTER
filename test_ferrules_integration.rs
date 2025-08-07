use anyhow::Result;
use std::path::Path;

mod character_matrix_engine;
use character_matrix_engine::CharacterMatrixEngine;

fn main() -> Result<()> {
    // Get PDF path from command line argument or use default
    let args: Vec<String> = std::env::args().collect();
    let pdf_path_str = if args.len() > 1 {
        &args[1]
    } else {
        "chonker_test.pdf"
    };

    let test_pdf = Path::new(pdf_path_str);

    if !test_pdf.exists() {
        return Ok(());
    }

    // Create optimized engine
    let engine = CharacterMatrixEngine::new_optimized(test_pdf)?;

    // Process the PDF
    match engine.process_pdf(test_pdf) {
        Ok(matrix) => {
            // Just print the matrix
            for (row_idx, row) in matrix.matrix.iter().enumerate() {
                print!("{:3} ", row_idx); // Show row numbers
                for &ch in row.iter() {
                    print!("{}", if ch == ' ' { '·' } else { ch });
                }
                println!();
            }
        }
        Err(_e) => {
            // Silent error
        }
    }

    Ok(())
}
