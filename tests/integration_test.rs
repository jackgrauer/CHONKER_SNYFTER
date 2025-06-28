use std::path::Path;
use crate::{process_document, ToolPreference, ProcessingPath, search_documents, export_to_parquet};  // Ensure functions are correctly imported

#[test]
fn test_end_to_end_simple_pdf() {
    // 1. Process simple PDF with Rust path
    let result = process_document("tests/fixtures/simple.pdf", ToolPreference::Auto);
    assert!(matches!(result.path_used, ProcessingPath::Rust));
    assert!(result.processing_time_ms < 50);

    // 2. Verify FTS5 search works
    let search_results = search_documents("simple");
    assert!(!search_results.is_empty());

    // 3. Export to Parquet
    let export_path = export_to_parquet(&search_results);
    assert!(Path::new(&export_path).exists());
}

#[test]
fn test_fallback_to_python() {
    // Force a failure in Rust path
    let result = process_document("tests/fixtures/corrupted.pdf", ToolPreference::Rust);
    assert!(matches!(result, ProcessingResult::FallbackSuccess { .. }));
}

