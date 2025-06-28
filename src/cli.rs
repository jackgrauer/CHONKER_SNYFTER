use std::path::PathBuf;
use anyhow::Result;
use tracing::info;
use crate::database::{ChonkerDatabase, ProcessingOptions};
use crate::processing::{ChonkerProcessor, ProcessingResult};
use crate::markdown::MarkdownProcessor;
use crate::export::DataFrameExporter;

/// Extract PDF to markdown command
pub async fn extract_command(
    pdf_path: PathBuf,
    output: Option<PathBuf>,
    tool: String,
    store: bool,
    _page: Option<usize>,
    database: ChonkerDatabase,
) -> Result<()> {
    info!("🔍 Extracting PDF: {:?}", pdf_path);
    
    if !pdf_path.exists() {
        return Err(anyhow::anyhow!("PDF file not found: {:?}", pdf_path));
    }
    
    // Initialize hybrid processor
    let mut processor = ChonkerProcessor::new();
    
    // Configure processing options
    let proc_options = ProcessingOptions {
        tool: tool.clone(),
        extract_tables: true,
        extract_formulas: true,
    };
    
    // Process document with hybrid architecture
    let processing_result = processor.process_document(&pdf_path, &proc_options).await.map_err(|e| {
        anyhow::anyhow!("Processing failed: {:?}", e)
    })?;
    
    info!("✅ Processing successful using: {}", processing_result.metadata.tool_used);
    info!("📄 Generated {} chunks", processing_result.chunks.len());
    info!("📊 Complexity score: {:.2}", processing_result.metadata.complexity_score);
    info!("⚡ Processing path: {:?}", processing_result.processing_path);
    
    // Convert chunks to markdown
    let markdown_content = convert_chunks_to_markdown(&processing_result);
    
    // Determine output path
    let output_path = output.unwrap_or_else(|| {
        let mut path = pdf_path.clone();
        path.set_extension("md");
        path
    });
    
    // Write markdown file
    std::fs::write(&output_path, &markdown_content)?;
    info!("📝 Markdown saved to: {:?}", output_path);
    
    // Store in database if requested
    if store {
        let processing_opts = ProcessingOptions {
            tool: tool.clone(),
            extract_tables: true,
            extract_formulas: true,
        };
        
        let doc_id = database.save_document(
            &pdf_path.file_name().unwrap().to_string_lossy(),
            &pdf_path,
            &processing_result.chunks,
            &processing_opts,
            processing_result.metadata.processing_time_ms,
        ).await?;
        info!("💾 Stored in database with ID: {}", doc_id);
    }
    
    // Print summary with properly spaced mascot
    println!("  \\___/>");
    println!("  [o-·-o]");
    println!("  (\")~(\")  🎉 Extraction Complete!");
    println!("          Tool used: {}", processing_result.metadata.tool_used);
    println!("          Chunks generated: {}", processing_result.chunks.len());
    println!("          Processing time: {}ms", processing_result.metadata.processing_time_ms);
    println!("          Output file: {:?}", output_path);
    if store {
        println!("          Stored in database: ✅");
    }
    
    Ok(())
}

/// Process markdown command
pub async fn process_command(
    markdown_path: PathBuf,
    output: Option<PathBuf>,
    correct: bool,
) -> Result<()> {
    info!("📝 Processing markdown: {:?}", markdown_path);
    
    if !markdown_path.exists() {
        return Err(anyhow::anyhow!("Markdown file not found: {:?}", markdown_path));
    }
    
    // Read markdown content
    let content = std::fs::read_to_string(&markdown_path)?;
    
    // Initialize markdown processor
    let processor = MarkdownProcessor::new();
    
    // Process content
    let processed_content = if correct {
        processor.apply_corrections(&content)?
    } else {
        processor.normalize(&content)?
    };
    
    // Determine output path
    let output_path = output.unwrap_or_else(|| {
        let mut path = markdown_path.clone();
        let stem = path.file_stem().unwrap().to_string_lossy();
        path.set_file_name(format!("{}_processed.md", stem));
        path
    });
    
    // Write processed file
    std::fs::write(&output_path, &processed_content)?;
    info!("📝 Processed markdown saved to: {:?}", output_path);
    
    // Print summary
    println!("🎉 Processing Complete!");
    println!("   Input file: {:?}", markdown_path);
    println!("   Output file: {:?}", output_path);
    println!("   Corrections applied: {}", if correct { "✅" } else { "❌" });
    
    Ok(())
}

/// Export data to DataFrame formats
pub async fn export_command(
    format: String,
    output: PathBuf,
    doc_id: Option<String>,
    database: ChonkerDatabase,
) -> Result<()> {
    info!("📊 Exporting data to {} format", format);
    
    let exporter = DataFrameExporter::new(database);
    
    match format.as_str() {
        "csv" => {
            exporter.export_to_csv(&output, doc_id.as_deref()).await?;
        },
        "json" => {
            exporter.export_to_json(&output, doc_id.as_deref()).await?;
        },
        "parquet" => {
            exporter.export_to_parquet(&output, doc_id.as_deref()).await?;
        },
        _ => {
            return Err(anyhow::anyhow!("Unsupported export format: {}", format));
        }
    }
    
    info!("📊 Export completed: {:?}", output);
    
    // Print summary
    println!("🎉 Export Complete!");
    println!("   Format: {}", format);
    println!("   Output file: {:?}", output);
    if let Some(id) = doc_id {
        println!("   Document filter: {}", id);
    }
    
    Ok(())
}

/// Show database status
pub async fn status_command(database: ChonkerDatabase) -> Result<()> {
    info!("📊 Checking database status");
    
    let stats = database.get_stats().await?;
    
    println!("📊 CHONKER Database Status");
    println!("========================");
    println!("Documents: {}", stats.document_count);
    println!("Total chunks: {}", stats.chunk_count);
    println!("Database size: {:.2} MB", stats.database_size_mb);
    println!("Last updated: {}", stats.last_updated);
    
    // Show recent documents
    let recent_docs = database.get_recent_documents(5).await?;
    if !recent_docs.is_empty() {
        println!("\nRecent Documents:");
        println!("-----------------");
        for doc in recent_docs {
            println!("• {} ({})", doc.filename, doc.created_at);
        }
    }
    
    Ok(())
}

/// Convert processing result chunks to markdown format
fn convert_chunks_to_markdown(result: &ProcessingResult) -> String {
    let mut markdown = String::new();
    
    // Add metadata header
    markdown.push_str(&format!("# Document Processing\n\n"));
    markdown.push_str(&format!("**Tool:** {}\n", result.metadata.tool_used));
    markdown.push_str(&format!("**Total Pages:** {}\n", result.metadata.total_pages));
    markdown.push_str(&format!("**Processing Time:** {}ms\n", result.metadata.processing_time_ms));
    markdown.push_str(&format!("**Complexity Score:** {:.2}\n", result.metadata.complexity_score));
    markdown.push_str(&format!("**Processing Path:** {:?}\n\n", result.processing_path));
    
    // Add chunk content
    for chunk in &result.chunks {
        markdown.push_str(&format!("## Chunk {} (Page {})\n\n", chunk.id, chunk.page_range));
        
        // Add element type information
        if !chunk.element_types.is_empty() {
            markdown.push_str(&format!("**Elements:** {}\n\n", chunk.element_types.join(", ")));
        }
        
        // Add text content
        if !chunk.content.trim().is_empty() {
            markdown.push_str(&chunk.content);
            markdown.push_str("\n\n");
        }
        
        // Add spatial bounds if available
        if let Some(bounds) = &chunk.spatial_bounds {
            markdown.push_str(&format!("**Spatial Bounds:** `{}`\n\n", bounds));
        }
        
        markdown.push_str(&format!("**Character Count:** {}\n\n", chunk.char_count));
        markdown.push_str("---\n\n");
    }
    
    markdown
}
