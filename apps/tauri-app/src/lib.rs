use chrono::Local;

#[tauri::command]
async fn terminal_log(level: String, message: String) {
    let timestamp = Local::now().format("%H:%M:%S");
    match level.as_str() {
        "error" => println!("🔴 [{}] ERROR: {}", timestamp, message),
        "info" => println!("🔵 [{}] INFO: {}", timestamp, message),
        "debug" => println!("🟡 [{}] DEBUG: {}", timestamp, message),
        _ => println!("⚪ [{}] {}", timestamp, message),
    }
}

#[tauri::command]
async fn test_command() -> Result<String, String> {
    Ok("🐹 CHONKER SNYFTER is working!".to_string())
}

#[tauri::command]
async fn select_pdf_file() -> Result<serde_json::Value, String> {
    // For now, return a mock response
    // In Tauri v2, you would use the dialog plugin from the frontend
    Ok(serde_json::json!({
        "success": true,
        "message": "Use the frontend dialog API in Tauri v2"
    }))
}

#[tauri::command]
async fn process_file(file_path: String) -> Result<serde_json::Value, String> {
    println!("🐹 Processing file with doc service: {}", file_path);
    
    // Call the document processing service
    match call_doc_service(&file_path).await {
        Ok(result) => Ok(result),
        Err(e) => {
            println!("🔴 Doc service error: {}", e);
            // Fallback to mock processing
            Ok(serde_json::json!({
                "success": false,
                "message": format!("Processing failed: {}", e),
                "file_path": file_path
            }))
        }
    }
}

async fn call_doc_service(file_path: &str) -> Result<serde_json::Value, String> {
    // Check if file exists
    if !std::path::Path::new(file_path).exists() {
        return Err(format!("File not found: {}", file_path));
    }
    
    // Read file
    let file_content = std::fs::read(file_path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    let file_name = std::path::Path::new(file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");
    
    // Create multipart form
    let form = reqwest::multipart::Form::new()
        .part(
            "file",
            reqwest::multipart::Part::bytes(file_content)
                .file_name(file_name.to_string())
                .mime_str("application/octet-stream")
                .map_err(|e| format!("Failed to create form part: {}", e))?
        );
    
    // Send to doc service
    let client = reqwest::Client::new();
    let response = client
        .post("http://localhost:8000/process")
        .multipart(form)
        .send()
        .await
        .map_err(|e| format!("Failed to call doc service: {}", e))?;
    
    if response.status().is_success() {
        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;
        Ok(result)
    } else {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        Err(format!("Doc service error {}: {}", status, error_text))
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            test_command,
            select_pdf_file,
            process_file,
            terminal_log
        ])
        .setup(|_app| {
            println!("🐹 CHONKER SNYFTER is starting up...");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
