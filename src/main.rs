#![allow(dead_code)]

mod cli;
mod database;
mod error;
mod logging;
mod extractor;
mod native_extractor;
mod processing;
mod tui_simple;
mod export;
mod pdf;
mod analyzer;
mod config;
mod smart_column_extractor;

use clap::{Parser, Subcommand};
use anyhow::Result;
use logging::{LoggingConfig, log_system_info};
use tracing::info;
use database::{ChonkerDatabase, DatabaseConfig};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "chonker")]
#[command(about = "CHONKER - CLI-First Document Processing Pipeline")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// Database path
    #[arg(short, long, default_value = "chonker.db")]
    database: PathBuf,
    
    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Extract text from PDF to markdown
    Extract {
        /// PDF file path
        pdf: PathBuf,
        
        /// Output markdown file (optional, defaults to pdf_name.md)
        #[arg(short, long)]
        output: Option<PathBuf>,
        
        /// Extraction tool preference
        #[arg(short, long, default_value = "auto")]
        tool: String,
        
        /// Store in database
        #[arg(long)]
        store: bool,
        
        /// Extract specific page only (1-indexed)
        #[arg(short = 'p', long)]
        page: Option<usize>,
        
        /// Use SmolDocling VLM for enhanced document understanding
        #[arg(long)]
        vlm: bool,
    },
    
    
    /// Export data to DataFrame formats
    Export {
        /// Export format
        #[arg(short, long, default_value = "csv")]
        format: String, // csv, json, parquet
        
        /// Output file path
        #[arg(short, long)]
        output: PathBuf,
        
        /// Filter by document ID
        #[arg(long)]
        doc_id: Option<String>,
    },
    
    /// Launch interactive TUI
    Tui,
    
    /// Show database status
    Status,
}

async fn initialize_database(db_path: &PathBuf) -> Result<ChonkerDatabase> {
    let config = DatabaseConfig::default();
    let db = ChonkerDatabase::new(db_path.to_str().unwrap(), config).await?;
    Ok(db)
}

#[tokio::main]
async fn main() -> Result<()> {
    // Handle help FIRST, before ANY initialization
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 2 && (args[1] == "-h" || args[1] == "--help") {
        println!("CHONKER - CLI-First Document Processing Pipeline");
        println!();
        println!("Usage: chonker [OPTIONS] <COMMAND>");
        println!();
        println!("Commands:");
        println!("  extract  Extract text from PDF to markdown with consensus validation");
        println!("  export   Export data to DataFrame formats");
        println!("  tui      Launch interactive TUI");
        println!("  status   Show database status");
        println!("  help     Print this message");
        println!();
        println!("Options:");
        println!("  -d, --database <DATABASE>  Database path [default: chonker.db]");
        println!("  -v, --verbose              Enable verbose logging");
        println!("  -h, --help                 Print help");
        return Ok(());
    }
    
    let cli = Cli::parse();
    
    // Initialize logging ONLY after handling help
    let logging_config = LoggingConfig {
        level: if cli.verbose { "debug" } else { "info" }.to_string(),
        ..LoggingConfig::default()
    };
    logging::init_logging(&logging_config)?;
    
    log_system_info();
    info!("🐹 CHONKER CLI starting up");
    
    // Initialize database
    let database = initialize_database(&cli.database).await?;
    info!("Database initialized: {:?}", cli.database);
    
    match cli.command {
        Commands::Extract { pdf, output, tool, store, page, vlm } => {
            cli::extract_command(pdf, output, tool, store, page, vlm, database).await?
        },
        
        Commands::Export { format, output, doc_id } => {
            cli::export_command(format, output, doc_id, database).await?
        },
        
        Commands::Tui => {
            tui_simple::run_tui(database).await?
        },
        
        Commands::Status => {
            cli::status_command(database).await?
        },
    }
    
    info!("🐹 CHONKER CLI completed successfully");
    Ok(())
}

