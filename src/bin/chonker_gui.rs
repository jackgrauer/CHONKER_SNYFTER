use eframe::{egui, NativeOptions};
use std::path::PathBuf;

// Import from the parent crate modules
use chonker_tui::app::ChonkerApp;
use chonker_tui::database::{ChonkerDatabase, DatabaseConfig};
use chonker_tui::logging::{LoggingConfig, log_system_info};

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    // Initialize logging
    let logging_config = LoggingConfig {
        level: "info".to_string(),
        ..LoggingConfig::default()
    };
    chonker_tui::logging::init_logging(&logging_config).expect("Failed to initialize logging");
    
    log_system_info();
    #[cfg(feature = "mupdf")]
    println!("🚀 Starting CHONKER GUI with MuPDF high-performance viewer and MLX optimizations!");
    #[cfg(not(feature = "mupdf"))]
    println!("🐹 Starting CHONKER GUI with MLX optimizations (add --features mupdf for high-performance PDF rendering)...");
    
    // Initialize database (same as CLI version)
    let db_path = PathBuf::from("chonker.db");
    let config = DatabaseConfig::default();
    
    println!("📀 Connecting to database: {:?}", db_path);
    let database = match ChonkerDatabase::new(db_path.to_str().unwrap(), config).await {
        Ok(db) => {
            let stats = db.get_stats().await.unwrap_or_default();
            println!("✅ Database connected: {} documents, {} chunks", 
                stats.document_count, stats.chunk_count);
            Some(db)
        }
        Err(e) => {
            println!("⚠️ Database connection failed: {}", e);
            println!("📋 GUI will run without database persistence");
            None
        }
    };

    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 800.0])
            .with_resizable(true)
            .with_title("CHONKER_SNYFTER - MLX-Optimized Document Processing Workbench"),
        ..Default::default()
    };

    eframe::run_native(
        "CHONKER_SNYFTER",
        options,
        Box::new(move |cc| {
            // Initialize app with database connection
            let mut app = ChonkerApp::new(cc, database);
            
            // Update status to show MLX optimization
            if app.database.is_some() {
                app.status_message = "🚀 CHONKER ready with MLX optimizations and database! Select a PDF to process".to_string();
            } else {
                app.status_message = "🚀 CHONKER ready with MLX optimizations (no database)! Select a PDF to process".to_string();
            }
            
            Ok(Box::new(app))
        }),
    )
}
