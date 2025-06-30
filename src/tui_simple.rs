use crate::database::ChonkerDatabase;
use anyhow::Result;
use std::io;

/// Simple TUI runner
pub async fn run_tui(database: ChonkerDatabase) -> Result<()> {
    println!("\n🐹 CHONKER - CLI-First Document Processing Pipeline");
    println!("======================================================\n");
    
    println!("Available Commands:");
    println!("  extract  - Extract text from PDF using consensus validation (Magic-PDF + Docling)");
    println!("  export   - Export data to DataFrame formats");
    println!("  status   - Show database status\n");
    
    // Show database status by default
    match database.get_stats().await {
        Ok(stats) => {
            println!("📊 Database Status:");
            println!("   Documents: {}", stats.document_count);
            println!("   Total chunks: {}", stats.chunk_count);
            println!("   Database size: {:.2} MB\n", stats.database_size_mb);
        }
        Err(e) => {
            println!("❌ Error getting database stats: {}\n", e);
        }
    }
    
    println!("💡 To use CHONKER, exit this TUI and run:");
    println!("   cargo run --bin chonker extract path/to/file.pdf  # Consensus extraction");
    println!("   cargo run --bin chonker export -f csv -o output.csv");
    println!("   cargo run --bin chonker status\n");
    
    println!("Press Enter to exit...");
    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(|e| anyhow::anyhow!(e))?;
    
    Ok(())
}
