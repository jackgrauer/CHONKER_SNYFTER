mod chonker_types;
mod database;
mod processing;
mod extractor;
mod error;
mod pdf_renderer;

use chonker_types::*;
use database::Database;
use pdf_renderer::PdfRenderer;
// Note: processing and extractor modules are available but not used in simplified demo
use std::sync::Arc;
use tauri::{Manager, State};
use uuid::Uuid;

struct AppState {
    db: Arc<Database>,
    pdf_renderer: Arc<PdfRenderer>,
}

#[tauri::command]
async fn get_documents(state: State<'_, AppState>) -> Result<Vec<Document>, String> {
    state.db.get_documents().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_document_chunks(state: State<'_, AppState>, document_id: String) -> Result<Vec<DocumentChunk>, String> {
    let uuid = Uuid::parse_str(&document_id).map_err(|e| e.to_string())?;
    state.db.get_chunks_by_document(uuid).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_table_chunks(state: State<'_, AppState>) -> Result<Vec<DocumentChunk>, String> {
    state.db.get_table_chunks().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn test_command() -> Result<String, String> {
    Ok("🐹🐭 CHONKER Tauri API is working!".to_string())
}

#[tauri::command]
async fn render_pdf_page(_state: State<'_, AppState>, pdfPath: String, pageNum: i32, zoom: Option<f32>) -> Result<serde_json::Value, String> {
    let zoom = zoom.unwrap_or(1.0);
    
    tracing::info!("🐹 render_pdf_page called with pdfPath: {}, pageNum: {}, zoom: {}", pdfPath, pageNum, zoom);
    
    // Temporarily return dummy data to test parameter passing
    Ok(serde_json::json!({
        "success": true,
        "image_data": "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8/5+hHgAHggJ/PchI7wAAAABJRU5ErkJggg==", // 1x1 transparent PNG
        "page_num": pageNum,
        "zoom": zoom,
        "message": "PDF rendering temporarily disabled for testing"
    }))
    
    /* Original code - commented out to test parameter passing
    match state.pdf_renderer.render_page_to_base64(&pdfPath, pageNum, zoom) {
        Ok(base64_data) => {
            Ok(serde_json::json!({
                "success": true,
                "image_data": base64_data,
                "page_num": pageNum,
                "zoom": zoom
            }))
        }
        Err(e) => {
            tracing::error!("Failed to render PDF page: {}", e);
            Ok(serde_json::json!({
                "success": false,
                "error": e.to_string()
            }))
        }
    }
    */
}

#[tauri::command]
async fn get_pdf_page_count(_state: State<'_, AppState>, pdfPath: String) -> Result<serde_json::Value, String> {
    tracing::info!("🐹 get_pdf_page_count called with pdfPath: {}", pdfPath);
    
    // Temporarily return dummy data to test parameter passing
    Ok(serde_json::json!({
        "success": true,
        "page_count": 5,
        "message": "Parameter passing works! PDF path received successfully."
    }))
    
    /* Original code - commented out to test parameter passing
    match state.pdf_renderer.get_page_count(&pdfPath) {
        Ok(page_count) => {
            Ok(serde_json::json!({
                "success": true,
                "page_count": page_count
            }))
        }
        Err(e) => {
            tracing::error!("Failed to get PDF page count: {}", e);
            Ok(serde_json::json!({
                "success": false,
                "error": e.to_string()
            }))
        }
    }
    */
}

#[tauri::command]
async fn select_pdf_file(app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    use tauri_plugin_dialog::DialogExt;
    
    tracing::info!("🐹 Opening file dialog...");
    
    // Use Tauri file dialog to select PDF
    let file_path = app.dialog()
        .file()
        .add_filter("PDF files", &["pdf"])
        .set_title("Select PDF Document")
        .blocking_pick_file();
    
    match file_path {
        Some(path) => {
            let path_str = path.to_string();
            tracing::info!("🐹 Selected file: {}", path_str);
            
            Ok(serde_json::json!({
                "path": path_str,
                "success": true
            }))
        }
        None => {
            tracing::info!("🐹 No file selected");
            Ok(serde_json::json!({
                "success": false,
                "error": "No file selected"
            }))
        }
    }
}

#[tauri::command]
async fn process_document(filePath: String, options: serde_json::Value) -> Result<serde_json::Value, String> {
    use std::path::{Path, PathBuf};
    use processing::{ChonkerProcessor, ProcessingOptions};
    
    // Validate file exists
    let path_buf = PathBuf::from(&filePath);
    if !path_buf.exists() {
        return Ok(serde_json::json!({
            "success": false,
            "error": "File not found"
        }));
    }
    
    // Extract processing options
    let formula_recognition = options["formula_recognition"].as_bool().unwrap_or(true);
    let table_detection = options["table_detection"].as_bool().unwrap_or(true);
    let _language = options["language"].as_str().unwrap_or("English");
    
    tracing::info!("🐹 Real CHONKER processing: {} with Tables={}, Formulas={}", 
                   filePath, table_detection, formula_recognition);
    
    // Create processing options
    let processing_options = ProcessingOptions {
        tool: "docling".to_string(),
        extract_tables: table_detection,
        extract_formulas: formula_recognition,
    };
    
    // Initialize real CHONKER processor
    let mut processor = ChonkerProcessor::new();
    
    // Process the document with real CHONKER pipeline
    match processor.process_document(Path::new(&filePath), &processing_options).await {
        Ok(result) => {
            let tables_count = result.chunks.iter()
                .filter(|chunk| chunk.content_type == "table")
                .count();
            let formulas_count = result.chunks.iter()
                .filter(|chunk| chunk.content_type == "formula")
                .count();
            
            tracing::info!("🐹 Real processing complete: {} chunks, {} tables, {} formulas", 
                          result.chunks.len(), tables_count, formulas_count);
            
            // Convert real table data to frontend format
            let mut tables = Vec::new();
            for chunk in &result.chunks {
                if chunk.content_type == "table" {
                    if let Some(table_data) = &chunk.table_data {
                        let mut headers = Vec::new();
                        let mut rows = Vec::new();
                        
                        // Extract headers and data from TableData
                        if !table_data.data.is_empty() && !table_data.data[0].is_empty() {
                            // First row as headers
                            headers = table_data.data[0].iter()
                                .map(|cell| cell.content.clone())
                                .collect();
                                
                            // Remaining rows as data
                            for row in table_data.data.iter().skip(1) {
                                let row_data: Vec<String> = row.iter()
                                    .map(|cell| cell.content.clone())
                                    .collect();
                                rows.push(row_data);
                            }
                        }
                        
                        tables.push(serde_json::json!({
                            "headers": headers,
                            "rows": rows,
                            "metadata": chunk.metadata
                        }));
                    }
                }
            }
            
            Ok(serde_json::json!({
                "success": true,
                "data": {
                    "tables_found": tables_count,
                    "chunks_extracted": result.chunks.len(),
                    "formulas_detected": formulas_count,
                    "pages_processed": result.metadata.total_pages,
                    "processing_time_ms": result.metadata.processing_time_ms,
                    "tool_used": format!("🐹 CHONKER Real - {}", result.metadata.tool_used),
                    "tables": tables
                }
            }))
        }
        Err(e) => {
            tracing::error!("🐹 Processing failed: {}", e);
            Ok(serde_json::json!({
                "success": false,
                "error": format!("CHONKER processing failed: {}", e)
            }))
        }
    }
}

#[tauri::command]
async fn save_to_database(state: State<'_, AppState>, data: serde_json::Value) -> Result<serde_json::Value, String> {
    use chonker_types::{Document, DocumentChunk, TableData, TableCell};
    use uuid::Uuid;
    use chrono::Utc;
    
    tracing::info!("🐹 Saving processed data to real CHONKER database...");
    
    // Extract document metadata
    let file_path = data["file_path"].as_str().unwrap_or("unknown.pdf");
    let filename = std::path::Path::new(file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown.pdf");
    
    // Create document record
    let document = Document {
        id: Uuid::new_v4(),
        filename: filename.to_string(),
        file_path: file_path.to_string(),
        file_hash: format!("{:x}", md5::compute(file_path.as_bytes())),
        content_type: "application/pdf".to_string(),
        file_size: std::fs::metadata(file_path).map(|m| m.len() as i64).unwrap_or(0),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    // Convert tables to document chunks
    let mut chunks = Vec::new();
    if let Some(tables_array) = data["tables"].as_array() {
        for (index, table) in tables_array.iter().enumerate() {
            if let (Some(headers), Some(rows)) = (
                table["headers"].as_array(),
                table["rows"].as_array()
            ) {
                // Convert to TableData structure
                let mut table_rows = Vec::new();
                
                // Headers as first row
                let header_cells: Vec<TableCell> = headers.iter()
                    .map(|h| TableCell {
                        content: h.as_str().unwrap_or("").to_string(),
                        rowspan: None,
                        colspan: None,
                    })
                    .collect();
                table_rows.push(header_cells);
                
                // Data rows
                for row in rows {
                    if let Some(row_array) = row.as_array() {
                        let row_cells: Vec<TableCell> = row_array.iter()
                            .map(|cell| TableCell {
                                content: cell.as_str().unwrap_or("").to_string(),
                                rowspan: None,
                                colspan: None,
                            })
                            .collect();
                        table_rows.push(row_cells);
                    }
                }
                
                let table_data = TableData {
                    num_rows: table_rows.len(),
                    num_cols: table_rows.get(0).map(|r| r.len()).unwrap_or(0),
                    data: table_rows,
                };
                
                let chunk = DocumentChunk {
                    id: Uuid::new_v4(),
                    document_id: document.id,
                    chunk_index: index as i32,
                    content: format!("Table {} with {} rows", index + 1, table_data.num_rows),
                    content_type: "table".to_string(),
                    metadata: table["metadata"].as_str().map(|s| s.to_string()),
                    table_data: Some(table_data),
                    created_at: Utc::now(),
                };
                
                chunks.push(chunk);
            }
        }
    }
    
    // Save to database
    match state.db.save_document(&document, &chunks).await {
        Ok(document_id) => {
            tracing::info!("🐹 Successfully saved document {} with {} chunks", document_id, chunks.len());
            Ok(serde_json::json!({
                "success": true,
                "message": format!("🐹 Successfully saved {} tables to CHONKER database! 🐭", chunks.len()),
                "document_id": document_id,
                "chunks_saved": chunks.len()
            }))
        }
        Err(e) => {
            tracing::error!("🐹 Database save failed: {}", e);
            Ok(serde_json::json!({
                "success": false,
                "error": format!("Database save failed: {}", e)
            }))
        }
    }
}

#[tauri::command]
async fn generate_qc_report(_data: serde_json::Value) -> Result<serde_json::Value, String> {
    // Simulate QC report generation with Qwen cleaning
    use std::{thread, time::Duration};
    thread::sleep(Duration::from_secs(2)); // Simulate longer processing time
    
    Ok(serde_json::json!({
        "success": true,
        "message": "QC report generated successfully",
        "adversarial_content": [
            "Suspicious pattern detected in row 3",
            "Potential data injection in table 2"
        ]
    }))
}

#[tauri::command]
async fn render_markdown(content: String) -> Result<serde_json::Value, String> {
    // Convert table data to markdown format
    // This is a simple implementation - could be enhanced with a proper markdown parser
    
    let mut markdown_output = String::new();
    
    // Parse table data if it's JSON
    if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
        if let Some(tables) = data["tables"].as_array() {
            for (table_idx, table) in tables.iter().enumerate() {
                markdown_output.push_str(&format!("## Table {}\n\n", table_idx + 1));
                
                if let (Some(headers), Some(rows)) = (
                    table["headers"].as_array(),
                    table["rows"].as_array()
                ) {
                    // Headers
                    markdown_output.push_str("| ");
                    for header in headers {
                        markdown_output.push_str(&format!("{} | ", header.as_str().unwrap_or("")));
                    }
                    markdown_output.push_str("\n");
                    
                    // Separator
                    markdown_output.push_str("|");
                    for _ in headers {
                        markdown_output.push_str(" --- |");
                    }
                    markdown_output.push_str("\n");
                    
                    // Rows
                    for row in rows {
                        if let Some(row_array) = row.as_array() {
                            markdown_output.push_str("| ");
                            for cell in row_array {
                                markdown_output.push_str(&format!("{} | ", cell.as_str().unwrap_or("")));
                            }
                            markdown_output.push_str("\n");
                        }
                    }
                    
                    markdown_output.push_str("\n");
                }
            }
        }
    } else {
        // If not JSON, treat as plain text and add basic formatting
        markdown_output = content;
    }
    
    Ok(serde_json::json!({
        "success": true,
        "markdown": markdown_output
    }))
}

#[tauri::command]
async fn batch_process_pdfs(file_paths: Vec<String>, options: serde_json::Value) -> Result<serde_json::Value, String> {
    use std::path::Path;
    use processing::{ChonkerProcessor, ProcessingOptions};
    
    tracing::info!("🐹 Starting batch processing of {} PDFs", file_paths.len());
    
    let processing_options = ProcessingOptions {
        tool: "docling".to_string(),
        extract_tables: options["table_detection"].as_bool().unwrap_or(true),
        extract_formulas: options["formula_recognition"].as_bool().unwrap_or(true),
    };
    
    let mut processor = ChonkerProcessor::new();
    let mut results = Vec::new();
    let mut total_tables = 0;
    let mut total_formulas = 0;
    let mut total_chunks = 0;
    
    for (index, file_path) in file_paths.iter().enumerate() {
        if !std::path::Path::new(file_path).exists() {
            results.push(serde_json::json!({
                "file": file_path,
                "success": false,
                "error": "File not found"
            }));
            continue;
        }
        
        tracing::info!("🐹 Processing file {}/{}: {}", index + 1, file_paths.len(), file_path);
        
        match processor.process_document(Path::new(file_path), &processing_options).await {
            Ok(result) => {
                let tables_count = result.chunks.iter()
                    .filter(|chunk| chunk.content_type == "table")
                    .count();
                let formulas_count = result.chunks.iter()
                    .filter(|chunk| chunk.content_type == "formula")
                    .count();
                
                total_tables += tables_count;
                total_formulas += formulas_count;
                total_chunks += result.chunks.len();
                
                results.push(serde_json::json!({
                    "file": file_path,
                    "success": true,
                    "tables_found": tables_count,
                    "formulas_detected": formulas_count,
                    "chunks_extracted": result.chunks.len(),
                    "processing_time_ms": result.metadata.processing_time_ms
                }));
            }
            Err(e) => {
                tracing::error!("🐹 Failed to process {}: {}", file_path, e);
                results.push(serde_json::json!({
                    "file": file_path,
                    "success": false,
                    "error": e.to_string()
                }));
            }
        }
    }
    
    Ok(serde_json::json!({
        "success": true,
        "batch_results": results,
        "summary": {
            "total_files": file_paths.len(),
            "successful_files": results.iter().filter(|r| r["success"].as_bool().unwrap_or(false)).count(),
            "total_tables": total_tables,
            "total_formulas": total_formulas,
            "total_chunks": total_chunks
        }
    }))
}

#[tauri::command]
async fn get_pdf_info(file_path: String) -> Result<serde_json::Value, String> {
    use std::fs;
    
    if !std::path::Path::new(&file_path).exists() {
        return Ok(serde_json::json!({
            "success": false,
            "error": "File not found"
        }));
    }
    
    match fs::metadata(&file_path) {
        Ok(metadata) => {
            let file_size = metadata.len();
            let file_name = std::path::Path::new(&file_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown.pdf");
            
            Ok(serde_json::json!({
                "success": true,
                "file_name": file_name,
                "file_size": file_size,
                "file_size_mb": (file_size as f64) / (1024.0 * 1024.0),
                "file_path": file_path
            }))
        }
        Err(e) => {
            Ok(serde_json::json!({
                "success": false,
                "error": e.to_string()
            }))
        }
    }
}

#[tauri::command]
async fn search_documents(query: String, _limit: Option<i64>) -> Result<serde_json::Value, String> {
    // This would implement full-text search across processed documents
    // For now, return a placeholder implementation
    tracing::info!("🔍 Searching for: {}", query);
    
    Ok(serde_json::json!({
        "success": true,
        "query": query,
        "results": [],
        "message": "Search functionality will be implemented with vector embeddings"
    }))
}

#[tauri::command]
async fn get_processing_stats(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    // Get comprehensive database statistics
    let documents = state.db.get_documents().await.map_err(|e| e.to_string())?;
    
    let total_documents = documents.len();
    let total_size: u64 = documents.iter().map(|d| d.file_size as u64).sum();
    
    Ok(serde_json::json!({
        "success": true,
        "stats": {
            "total_documents": total_documents,
            "total_size_mb": (total_size as f64) / (1024.0 * 1024.0),
            "recent_documents": documents.iter().take(5).map(|d| {
                serde_json::json!({
                    "filename": d.filename,
                    "created_at": d.created_at.to_rfc3339(),
                    "file_size_mb": (d.file_size as f64) / (1024.0 * 1024.0)
                })
            }).collect::<Vec<_>>()
        }
    }))
}

#[tauri::command]
async fn export_data(items: serde_json::Value, format: String) -> Result<serde_json::Value, String> {
    use std::fs;
    use chrono::Utc;
    
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    let (file_extension, content) = match format.as_str() {
        "csv" => {
            let mut csv_content = String::new();
            if let Some(tables) = items["tables"].as_array() {
                for (table_idx, table) in tables.iter().enumerate() {
                    if table_idx > 0 {
                        csv_content.push_str("\n\n"); // Separate tables
                    }
                    
                    csv_content.push_str(&format!("# Table {}\n", table_idx + 1));
                    
                    if let (Some(headers), Some(rows)) = (
                        table["headers"].as_array(),
                        table["rows"].as_array()
                    ) {
                        // Headers
                        let header_line: Vec<String> = headers.iter()
                            .map(|h| format!("\"{}\"", h.as_str().unwrap_or("")))
                            .collect();
                        csv_content.push_str(&header_line.join(","));
                        csv_content.push_str("\n");
                        
                        // Rows
                        for row in rows {
                            if let Some(row_array) = row.as_array() {
                                let row_line: Vec<String> = row_array.iter()
                                    .map(|cell| format!("\"{}\"", cell.as_str().unwrap_or("")))
                                    .collect();
                                csv_content.push_str(&row_line.join(","));
                                csv_content.push_str("\n");
                            }
                        }
                    }
                }
            }
            ("csv", csv_content)
        },
        "json" => {
            let pretty_json = serde_json::to_string_pretty(&items)
                .unwrap_or_else(|_| items.to_string());
            ("json", pretty_json)
        },
        "markdown" => {
            let mut md_content = String::new();
            md_content.push_str(&format!("# CHONKER Export - {}\n\n", timestamp));
            
            if let Some(tables) = items["tables"].as_array() {
                for (table_idx, table) in tables.iter().enumerate() {
                    md_content.push_str(&format!("## Table {}\n\n", table_idx + 1));
                    
                    if let (Some(headers), Some(rows)) = (
                        table["headers"].as_array(),
                        table["rows"].as_array()
                    ) {
                        // Headers
                        md_content.push_str("| ");
                        for header in headers {
                            md_content.push_str(&format!("{} | ", header.as_str().unwrap_or("")));
                        }
                        md_content.push_str("\n");
                        
                        // Separator
                        md_content.push_str("|");
                        for _ in headers {
                            md_content.push_str(" --- |");
                        }
                        md_content.push_str("\n");
                        
                        // Rows
                        for row in rows {
                            if let Some(row_array) = row.as_array() {
                                md_content.push_str("| ");
                                for cell in row_array {
                                    md_content.push_str(&format!("{} | ", cell.as_str().unwrap_or("")));
                                }
                                md_content.push_str("\n");
                            }
                        }
                        
                        md_content.push_str("\n");
                    }
                }
            }
            ("md", md_content)
        },
        _ => {
            let content = format!("CHONKER Export - {}\n\n{}", timestamp, items.to_string());
            ("txt", content)
        }
    };
    
    let file_path = format!("/Users/jack/CHONKER_SNYFTER/chonker_export_{}.{}", timestamp, file_extension);
    
    // Write to file
    match fs::write(&file_path, content) {
        Ok(_) => {
            tracing::info!("🐹 Successfully exported data to {}", file_path);
            Ok(serde_json::json!({
                "success": true,
                "message": format!("🐹 Data exported successfully to {}", file_path),
                "path": file_path
            }))
        }
        Err(e) => {
            tracing::error!("🐹 Export failed: {}", e);
            Ok(serde_json::json!({
                "success": false,
                "error": format!("Export failed: {}", e)
            }))
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_dialog::init())
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }

      // Initialize database
      let runtime = tokio::runtime::Runtime::new().unwrap();
      let db = runtime.block_on(async {
          Database::new("sqlite:/Users/jack/CHONKER_SNYFTER/chonker.db").await
      }).expect("Failed to connect to database");
      
      // Initialize PDF renderer
      let pdf_renderer = PdfRenderer::new().expect("Failed to initialize PDF renderer");

      app.manage(AppState {
          db: Arc::new(db),
          pdf_renderer: Arc::new(pdf_renderer),
      });

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
        get_documents,
        get_document_chunks,
        get_table_chunks,
        test_command,
        render_pdf_page,
        get_pdf_page_count,
        select_pdf_file,
        process_document,
        batch_process_pdfs,
        get_pdf_info,
        search_documents,
        get_processing_stats,
        save_to_database,
        generate_qc_report,
        render_markdown,
        export_data
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
