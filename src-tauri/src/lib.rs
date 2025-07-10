use tauri::api::dialog::FileDialogBuilder;
use std::process::Command;

#[tauri::command]
async fn test_command() -> Result<String, String> {
    Ok("🐹 CHONKER is working!".to_string())
}

#[tauri::command]
async fn select_pdf_file() -> Result<serde_json::Value, String> {
    use std::sync::{Arc, Mutex};
    use std::sync::mpsc;
    
    let (tx, rx) = mpsc::channel();
    let tx = Arc::new(Mutex::new(Some(tx)));
    
    FileDialogBuilder::new()
        .add_filter("PDF files", &["pdf"])
        .set_title("Select PDF Document")
        .pick_file(move |file_path| {
            if let Some(sender) = tx.lock().unwrap().take() {
                let _ = sender.send(file_path);
            }
        });
    
    match rx.recv() {
        Ok(Some(path)) => Ok(serde_json::json!({
            "path": path.to_string_lossy().to_string(),
            "success": true
        })),
        Ok(None) | Err(_) => Ok(serde_json::json!({
            "success": false,
            "error": "No file selected"
        }))
    }
}

#[tauri::command]
async fn process_with_docling(file_path: String) -> Result<serde_json::Value, String> {
    println!("🐹 Processing file with Docling: {}", file_path);
    
    // Activate virtual environment and run docling
    let venv_python = std::env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {}", e))?
        .join(".venv")
        .join("bin")
        .join("python");
    
    // Check if the Python file exists
    if !venv_python.exists() {
        return Err("Python virtual environment not found. Run 'just install' first.".to_string());
    }
    
    // Run a simple docling command (you can expand this)
    let output = Command::new(&venv_python)
        .arg("-c")
        .arg(format!(
            "import docling; print('🐹 Docling processing: {}'); print('Ready for processing!')",
            file_path
        ))
        .output()
        .map_err(|e| format!("Failed to execute docling: {}", e))?;
    
    if output.status.success() {
        let result = String::from_utf8_lossy(&output.stdout);
        Ok(serde_json::json!({
            "success": true,
            "message": result.trim(),
            "file_path": file_path
        }))
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        Err(format!("Docling processing failed: {}", error))
    }
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            test_command,
            select_pdf_file,
            process_with_docling
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
