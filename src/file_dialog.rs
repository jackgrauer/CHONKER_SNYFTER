use std::env;
use std::fs;
use std::path::PathBuf;

pub fn get_pdf_path() -> Option<PathBuf> {
    // First, check command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let path = PathBuf::from(&args[1]);
        if path.exists() && path.extension().map_or(false, |ext| ext == "pdf") {
            return Some(path);
        }
    }
    
    // Check common locations for PDFs
    let common_paths = [
        "~/Downloads",
        "~/Documents",
        "~/Desktop",
        ".",
    ];
    
    for dir in &common_paths {
        let expanded = shellexpand::tilde(dir);
        if let Ok(entries) = fs::read_dir(expanded.as_ref()) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "pdf") {
                    // Return the first PDF found
                    return Some(path);
                }
            }
        }
    }
    
    None
}

pub fn list_pdfs_in_directory(dir: &str) -> Vec<PathBuf> {
    let mut pdfs = Vec::new();
    let expanded = shellexpand::tilde(dir);
    
    if let Ok(entries) = fs::read_dir(expanded.as_ref()) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "pdf") {
                pdfs.push(path);
            }
        }
    }
    
    pdfs.sort();
    pdfs
}