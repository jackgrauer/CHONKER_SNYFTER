use tauri_plugin_dialog::DialogExt;


#[tauri::command]
async fn test_command() -> Result<String, String> {
    Ok("🐹 CHONKER is working!".to_string())
}

#[tauri::command]
async fn select_pdf_file(app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    let file_path = app.dialog()
        .file()
        .add_filter("PDF files", &["pdf"])
        .set_title("Select PDF Document")
        .blocking_pick_file();
    
    match file_path {
        Some(path) => Ok(serde_json::json!({
            "path": path.to_string(),
            "success": true
        })),
        None => Ok(serde_json::json!({
            "success": false,
            "error": "No file selected"
        }))
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
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            test_command,
            select_pdf_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
