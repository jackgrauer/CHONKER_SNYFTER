mod matrix;
mod chonker_matrix;
mod services;
mod file_selector_matrix;
mod theme;
mod config;
mod history;

use anyhow::Result;
use std::env;
use std::path::PathBuf;
use crate::chonker_matrix::ChonkerMatrix;

fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();
    
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    
    // Create the ChonkerMatrix app
    let mut app = ChonkerMatrix::new()?;
    
    // If a PDF path was provided, load it
    if args.len() > 1 {
        let pdf_path = PathBuf::from(&args[1]);
        if pdf_path.exists() && pdf_path.extension().map_or(false, |ext| ext == "pdf") {
            println!("Loading PDF: {}", pdf_path.display());
            app.load_pdf_on_start(pdf_path)?;
        } else if pdf_path.exists() {
            eprintln!("Error: File is not a PDF: {}", pdf_path.display());
        } else {
            eprintln!("Error: File not found: {}", pdf_path.display());
        }
    }
    
    // Run the app
    app.run()?;
    
    Ok(())
}