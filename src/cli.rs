use std::path::PathBuf;
use anyhow::Result;
use tracing::info;
use crate::database::ChonkerDatabase;
use crate::extractor::{Extractor, ExtractionResult};
use crate::markdown::MarkdownProcessor;
use crate::export::DataFrameExporter;

/// Extract PDF to markdown command
pub async fn extract_command(
    pdf_path: PathBuf,
    output: Option<PathBuf>,
    tool: String,
    store: bool,
    page: Option<usize>,
    mut database: ChonkerDatabase,
) -> Result<()> {
    info!("🔍 Extracting PDF: {:?}", pdf_path);
    
    if !pdf_path.exists() {
        return Err(anyhow::anyhow!("PDF file not found: {:?}", pdf_path));
    }
    
    // Initialize extractor
    let mut extractor = Extractor::new();
    extractor.set_preferred_tool(tool.clone());
    
    // Extract content (specific page or all pages)
    let extraction_result = if let Some(page_num) = page {
        info!("📄 Extracting page {} only", page_num);
        extractor.extract_page(&pdf_path, page_num).await?
    } else {
        extractor.extract_pdf(&pdf_path).await?
    };
    
    if !extraction_result.success {
        return Err(anyhow::anyhow!("Extraction failed: {:?}", extraction_result.error));
    }
    
    info!("✅ Extraction successful using tool: {}", extraction_result.tool);
    info!("📄 Extracted {} pages", extraction_result.extractions.len());
    
    // Convert to markdown
    let markdown_content = convert_extraction_to_markdown(&extraction_result);
    
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
        let doc_id = database.store_document(
            pdf_path.to_string_lossy().to_string(),
            markdown_content.clone(),
            serde_json::to_value(&extraction_result)?,
        ).await?;
        info!("💾 Stored in database with ID: {}", doc_id);
    }
    
    // Print summary with properly spaced mascot
    println!("  <\\___/>");
    println!("  [o-·-o]");
    println!("  (\")~(\")  🎉 Extraction Complete!");
    println!("          Tool used: {}", extraction_result.tool);
    println!("          Pages processed: {}", extraction_result.extractions.len());
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

/// Convert extraction result to markdown format
fn convert_extraction_to_markdown(result: &ExtractionResult) -> String {
    let mut markdown = String::new();
    
    // Add metadata header
    markdown.push_str(&format!("# Document Extraction\n\n"));
    markdown.push_str(&format!("**Tool:** {}\n", result.tool));
    markdown.push_str(&format!("**Pages:** {}\n", result.metadata.total_pages));
    markdown.push_str(&format!("**Processing Time:** {}ms\n\n", result.metadata.processing_time));
    
    // Add page content
    for page in &result.extractions {
        markdown.push_str(&format!("## Page {}\n\n", page.page_number));
        
        // Add text content
        if !page.text.trim().is_empty() {
            markdown.push_str(&page.text);
            markdown.push_str("\n\n");
        }
        
        // Add tables if any
        if !page.tables.is_empty() {
            markdown.push_str("### Tables\n\n");
            for (i, table) in page.tables.iter().enumerate() {
                markdown.push_str(&format!("**Table {}:**\n", i + 1));
                markdown.push_str(&format!("```json\n{}\n```\n\n", serde_json::to_string_pretty(table).unwrap_or_default()));
            }
        }
        
        // Add formulas if any
        if !page.formulas.is_empty() {
            markdown.push_str("### Formulas\n\n");
            for (i, formula) in page.formulas.iter().enumerate() {
                markdown.push_str(&format!("**Formula {}:**\n", i + 1));
                markdown.push_str(&format!("```json\n{}\n```\n\n", serde_json::to_string_pretty(formula).unwrap_or_default()));
            }
        }
        
        markdown.push_str("---\n\n");
    }
    
    markdown
}
