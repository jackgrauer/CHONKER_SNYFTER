use tauri::api::dialog::FileDialogBuilder;

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

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            test_command,
            select_pdf_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
