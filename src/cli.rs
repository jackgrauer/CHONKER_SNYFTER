use std::path::PathBuf;
use anyhow::Result;
use tracing::{info, warn, debug};
use crate::database::{ChonkerDatabase, ProcessingOptions};
use crate::processing::{ChonkerProcessor, ProcessingResult};
use crate::export::DataFrameExporter;
use crate::config::{ChonkerConfig, ToolPreference};
use crate::analyzer::ComplexityAnalyzer;

/// Extract PDF to markdown command with intelligent routing
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
    
    // Load configuration
    let config = ChonkerConfig::load_from_env();
    let tool_preference = ToolPreference::from(tool.as_str());
    
    // Analyze document complexity
    let complexity_score = analyze_document_complexity(&pdf_path, &config).await?;
    
    info!("📊 Complexity analysis: {}", complexity_score.reasoning);
    info!("📈 Score: {:.1}/10.0", complexity_score.score);
    
    // Determine processing path based on tool preference and complexity
    let selected_tool = determine_processing_tool(&tool_preference, &complexity_score, &config);
    
    info!("🎯 Selected processing path: {:?}", selected_tool);
    
    // Initialize processor and process document
    let processing_result = process_with_fallback(
        &pdf_path, 
        &selected_tool, 
        &complexity_score, 
        &config
    ).await?;
    
    info!("✅ Processing successful using: {}", processing_result.metadata.tool_used);
    info!("📄 Generated {} chunks", processing_result.chunks.len());
    info!("📊 Final complexity score: {:.2}", processing_result.metadata.complexity_score);
    info!("⚡ Processing path: {:?}", processing_result.processing_path);
    
    // Store telemetry if enabled
    if config.database.enable_telemetry {
        store_processing_telemetry(&database, &pdf_path, &complexity_score, &processing_result).await?;
    }
    
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

/// Analyze document complexity using available methods
async fn analyze_document_complexity(
    pdf_path: &PathBuf, 
    _config: &ChonkerConfig
) -> Result<crate::analyzer::ComplexityScore> {
    // Try full PDF analysis first, fall back to simple analysis
    match ComplexityAnalyzer::new() {
        Ok(analyzer) => {
            match analyzer.analyze_pdf(pdf_path) {
                Ok(score) => Ok(score),
                Err(_) => {
                    warn!("📊 Full PDF analysis failed, using simple heuristics");
                    analyzer.analyze_simple(pdf_path)
                }
            }
        },
        Err(_) => {
            warn!("📊 ComplexityAnalyzer unavailable, using file-based heuristics");
            // Create a simple fallback analyzer
            use crate::analyzer::ComplexityScore;
            use crate::analyzer::complexity::ComplexityFactors;
            let metadata = std::fs::metadata(pdf_path)?;
            let file_size_mb = metadata.len() as f32 / 1_048_576.0;
            
            let score = if file_size_mb < 2.0 { 2.0 } else if file_size_mb < 10.0 { 5.0 } else { 8.0 };
            
            Ok(ComplexityScore {
                score,
                factors: ComplexityFactors {
                    page_count: 0,
                    has_images: false,
                    has_tables: false,
                    has_forms: false,
                    file_size_mb,
                    has_multiple_columns: false,
                },
                reasoning: format!("File size: {:.1}MB - Fallback analysis", file_size_mb),
                should_use_fast_path: score <= 3.0,
            })
        }
    }
}

/// Determine which processing tool to use based on preference and complexity
fn determine_processing_tool(
    preference: &ToolPreference, 
    complexity: &crate::analyzer::ComplexityScore, 
    config: &ChonkerConfig
) -> ToolPreference {
    match preference {
        ToolPreference::Auto => {
            // Check file extension first
            let should_force_python = config.routing.force_python_for_types
                .iter()
                .any(|ext| complexity.reasoning.to_lowercase().contains(ext));
                
            if should_force_python {
                debug!("🐍 Forcing Python due to file type");
                ToolPreference::Python
            } else if complexity.score <= config.routing.complexity_threshold {
                debug!("🦀 Selecting Rust fast path (score: {:.1})", complexity.score);
                ToolPreference::Rust
            } else {
                debug!("🐍 Selecting Python complex path (score: {:.1})", complexity.score);
                ToolPreference::Python
            }
        },
        other => other.clone(),
    }
}

/// Process document with fallback mechanism
async fn process_with_fallback(
    pdf_path: &PathBuf,
    tool: &ToolPreference,
    complexity_score: &crate::analyzer::ComplexityScore,
    config: &ChonkerConfig,
) -> Result<ProcessingResult> {
    let mut processor = ChonkerProcessor::new();
    
    let processing_options = ProcessingOptions {
        tool: format!("{:?}", tool).to_lowercase(),
        extract_tables: true,
        extract_formulas: true,
    };
    
    match tool {
        ToolPreference::Rust => {
            // Try Rust path first
            match processor.process_document(pdf_path, &processing_options).await {
                Ok(result) => {
                    info!("🦀 Rust fast path successful");
                    Ok(result)
                },
                Err(e) => {
                    if config.routing.enable_fallback {
                        warn!("🦀 Rust path failed: {}", e);
                        warn!("🐍 Falling back to Python path...");
                        
                        let python_options = ProcessingOptions {
                            tool: "python".to_string(),
                            extract_tables: true,
                            extract_formulas: true,
                        };
                        
                        processor.process_document(pdf_path, &python_options).await
                            .map_err(|e| anyhow::anyhow!("Both Rust and Python paths failed: {:?}", e))
                    } else {
                        Err(anyhow::anyhow!("Rust processing failed: {}", e))
                    }
                }
            }
        },
        ToolPreference::Python => {
            info!("🐍 Using Python ML path");
            processor.process_document(pdf_path, &processing_options).await
                .map_err(|e| anyhow::anyhow!("Python processing failed: {:?}", e))
        },
        ToolPreference::Auto => {
            // This should not happen as Auto is resolved earlier
            warn!("⚠️  Auto preference not resolved, defaulting to Python");
            let python_options = ProcessingOptions {
                tool: "python".to_string(),
                extract_tables: true,
                extract_formulas: true,
            };
            processor.process_document(pdf_path, &python_options).await
                .map_err(|e| anyhow::anyhow!("Auto fallback to Python failed: {:?}", e))
        }
    }
}

/// Store processing telemetry for learning and optimization
async fn store_processing_telemetry(
    _database: &ChonkerDatabase,
    pdf_path: &PathBuf,
    complexity_score: &crate::analyzer::ComplexityScore,
    processing_result: &ProcessingResult,
) -> Result<()> {
    debug!("📊 Storing processing telemetry");
    
    // Create telemetry record
    let telemetry = serde_json::json!({
        "file_path": pdf_path.to_string_lossy(),
        "file_size_mb": complexity_score.factors.file_size_mb,
        "complexity_score": complexity_score.score,
        "complexity_factors": {
            "page_count": complexity_score.factors.page_count,
            "has_images": complexity_score.factors.has_images,
            "has_tables": complexity_score.factors.has_tables,
            "has_forms": complexity_score.factors.has_forms,
        },
        "processing_result": {
            "tool_used": processing_result.metadata.tool_used,
            "processing_time_ms": processing_result.metadata.processing_time_ms,
            "processing_path": format!("{:?}", processing_result.processing_path),
            "chunk_count": processing_result.chunks.len(),
            "success": true,
        },
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });
    
    // Store in database (we'll add a telemetry table later)
    debug!("📊 Telemetry: {}", telemetry);
    
    // For now, just log the telemetry
    // TODO: Add actual database storage for telemetry
    
    Ok(())
}
