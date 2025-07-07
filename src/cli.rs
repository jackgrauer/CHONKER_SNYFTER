use std::path::PathBuf;
use anyhow::Result;
use tracing::{info, warn, debug};
use crate::database::{ChonkerDatabase, ProcessingOptions};
use crate::processing::{ChonkerProcessor, ProcessingResult};
use crate::export::DataFrameExporter;
use crate::config::{ChonkerConfig, ToolPreference};
#[cfg(feature = "advanced_pdf")]
use crate::analyzer::ComplexityAnalyzer;

/// Extract PDF to markdown command with intelligent routing
pub async fn extract_command(
    pdf_path: PathBuf,
    output: Option<PathBuf>,
    tool: String,
    store: bool,
    _page: Option<usize>,
    vlm: bool,
    database: ChonkerDatabase,
) -> Result<()> {
    info!("🔍 Extracting PDF: {:?}", pdf_path);
    
    if !pdf_path.exists() {
        return Err(anyhow::anyhow!("PDF file not found: {:?}", pdf_path));
    }
    
    // Load configuration
    let config = ChonkerConfig::load_from_env();
    let tool_preference = ToolPreference::from(tool.as_str());
    
    // Check VLM mode
    if vlm {
        info!("🤖 SmolDocling VLM mode enabled - using enhanced document understanding");
        return extract_with_vlm(pdf_path, output, store, database).await;
    }
    
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

/// Extract PDF using SmolDocling VLM for enhanced understanding
async fn extract_with_vlm(
    pdf_path: PathBuf,
    output: Option<PathBuf>,
    store: bool,
    database: ChonkerDatabase,
) -> Result<()> {
    info!("🤖 Starting SmolDocling VLM extraction...");
    
    // Initialize processor with VLM mode
    let mut processor = ChonkerProcessor::new();
    
    // Enable VLM mode on the Python bridge
    if let Some(ref mut bridge) = processor.python_bridge {
        bridge.set_vlm_mode(true);
    }
    
    let processing_options = ProcessingOptions {
        tool: "smoldocling_vlm".to_string(),
        extract_tables: true,
        extract_formulas: true,
    };
    
    // Process with VLM
    let start_time = std::time::Instant::now();
    let processing_result = processor.process_document(&pdf_path, &processing_options).await
        .map_err(|e| anyhow::anyhow!("SmolDocling VLM processing failed: {:?}", e))?;
    
    let processing_time = start_time.elapsed();
    
    info!("✅ SmolDocling VLM processing successful!");
    info!("📄 Generated {} chunks with enhanced understanding", processing_result.chunks.len());
    info!("⚡ Processing time: {:.1}s", processing_time.as_secs_f64());
    
    // Convert chunks to markdown with VLM enhancements
    let markdown_content = convert_vlm_chunks_to_markdown(&processing_result);
    
    // Determine output path
    let output_path = output.unwrap_or_else(|| {
        let mut path = pdf_path.clone();
        path.set_extension("md");
        path
    });
    
    // Write markdown file
    std::fs::write(&output_path, &markdown_content)?;
    info!("📝 Enhanced markdown saved to: {:?}", output_path);
    
    // Store in database if requested
    if store {
        let processing_opts = ProcessingOptions {
            tool: "smoldocling_vlm".to_string(),
            extract_tables: true,
            extract_formulas: true,
        };
        
        let doc_id = database.save_document(
            &pdf_path.file_name().unwrap().to_string_lossy(),
            &pdf_path,
            &processing_result.chunks,
            &processing_opts,
            processing_time.as_millis() as u64,
        ).await?;
        info!("💾 Stored in database with ID: {}", doc_id);
    }
    
    // Print VLM-specific summary
    println!("  \\___/>");
    println!("  [o-·-o]");
    println!("  (\")~(\")  🤖 SmolDocling VLM Extraction Complete!");
    println!("          Model: SmolDocling Vision-Language Model");
    println!("          Enhanced chunks: {}", processing_result.chunks.len());
    println!("          Processing time: {:.1}s", processing_time.as_secs_f64());
    println!("          Output file: {:?}", output_path);
    if store {
        println!("          Stored in database: ✅");
    }
    println!("          VLM Features: Enhanced table detection, figure understanding, layout analysis");
    
    Ok(())
}

/// Convert VLM processing result chunks to enhanced markdown format
fn convert_vlm_chunks_to_markdown(result: &ProcessingResult) -> String {
    let mut markdown = String::new();
    
    // Add enhanced metadata header
    markdown.push_str(&format!("# SmolDocling VLM Document Processing\n\n"));
    markdown.push_str(&format!("**Model:** SmolDocling Vision-Language Model\n"));
    markdown.push_str(&format!("**Tool:** {}\n", result.metadata.tool_used));
    markdown.push_str(&format!("**Total Pages:** {}\n", result.metadata.total_pages));
    markdown.push_str(&format!("**Processing Time:** {}ms\n", result.metadata.processing_time_ms));
    markdown.push_str(&format!("**Processing Path:** {:?}\n", result.processing_path));
    markdown.push_str(&format!("**Enhanced Features:** Vision-Language understanding, improved table detection, figure analysis\n\n"));
    
    // Group chunks by element type
    let mut text_chunks = Vec::new();
    let mut table_chunks = Vec::new();
    let mut figure_chunks = Vec::new();
    let mut other_chunks = Vec::new();
    
    for chunk in &result.chunks {
        if chunk.element_types.contains(&"table".to_string()) {
            table_chunks.push(chunk);
        } else if chunk.element_types.contains(&"figure".to_string()) {
            figure_chunks.push(chunk);
        } else if chunk.element_types.contains(&"text".to_string()) {
            text_chunks.push(chunk);
        } else {
            other_chunks.push(chunk);
        }
    }
    
    // Add text content
    if !text_chunks.is_empty() {
        markdown.push_str("## Text Content\n\n");
        for chunk in text_chunks {
            markdown.push_str(&format!("### Chunk {} (Page {})\n\n", chunk.id, chunk.page_range));
            if !chunk.content.trim().is_empty() {
                markdown.push_str(&chunk.content);
                markdown.push_str("\n\n");
            }
        }
    }
    
    // Add table content with enhanced formatting
    if !table_chunks.is_empty() {
        markdown.push_str("## Tables (Enhanced VLM Detection)\n\n");
        for chunk in table_chunks {
            markdown.push_str(&format!("### Table {} (Page {})\n\n", chunk.id, chunk.page_range));
            markdown.push_str("> **VLM Enhancement:** This table was detected and structured using vision-language understanding\n\n");
            if !chunk.content.trim().is_empty() {
                markdown.push_str(&chunk.content);
                markdown.push_str("\n\n");
            }
            if let Some(table_data) = &chunk.table_data {
                markdown.push_str(&format!("**Structured Data:** `{}`\n\n", table_data));
            }
        }
    }
    
    // Add figure content
    if !figure_chunks.is_empty() {
        markdown.push_str("## Figures & Images (VLM Descriptions)\n\n");
        for chunk in figure_chunks {
            markdown.push_str(&format!("### Figure {} (Page {})\n\n", chunk.id, chunk.page_range));
            markdown.push_str("> **VLM Enhancement:** This figure description was generated using vision-language understanding\n\n");
            if !chunk.content.trim().is_empty() {
                markdown.push_str(&chunk.content);
                markdown.push_str("\n\n");
            }
        }
    }
    
    // Add other content
    if !other_chunks.is_empty() {
        markdown.push_str("## Other Elements\n\n");
        for chunk in other_chunks {
            markdown.push_str(&format!("### Element {} (Page {})\n\n", chunk.id, chunk.page_range));
            markdown.push_str(&format!("**Type:** {}\n\n", chunk.element_types.join(", ")));
            if !chunk.content.trim().is_empty() {
                markdown.push_str(&chunk.content);
                markdown.push_str("\n\n");
            }
        }
    }
    
    markdown.push_str("---\n\n");
    markdown.push_str("*Generated with SmolDocling Vision-Language Model for enhanced document understanding*\n");
    
    markdown
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
#[cfg(feature = "advanced_pdf")]
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

/// Analyze document complexity fallback for non-advanced_pdf builds
#[cfg(not(feature = "advanced_pdf"))]
async fn analyze_document_complexity(
    pdf_path: &PathBuf, 
    _config: &ChonkerConfig
) -> Result<BasicComplexityScore> {
    let metadata = std::fs::metadata(pdf_path)?;
    let file_size_mb = metadata.len() as f32 / 1_048_576.0;
    
    let score = if file_size_mb < 2.0 { 2.0 } else if file_size_mb < 10.0 { 5.0 } else { 8.0 };
    
    Ok(BasicComplexityScore {
        score,
        reasoning: format!("File size: {:.1}MB - Basic analysis (advanced features not available)", file_size_mb),
        should_use_fast_path: score <= 3.0,
    })
}

/// Basic complexity score for non-advanced_pdf builds
#[cfg(not(feature = "advanced_pdf"))]
#[derive(Clone, Debug)]
pub struct BasicComplexityScore {
    pub score: f32,
    pub reasoning: String,
    pub should_use_fast_path: bool,
}

/// Determine which processing tool to use based on preference and complexity
#[cfg(feature = "advanced_pdf")]
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

#[cfg(not(feature = "advanced_pdf"))]
fn determine_processing_tool(
    preference: &ToolPreference, 
    complexity: &BasicComplexityScore, 
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
#[cfg(feature = "advanced_pdf")]
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

/// Process document with fallback mechanism (non-advanced_pdf version)
#[cfg(not(feature = "advanced_pdf"))]
async fn process_with_fallback(
    pdf_path: &PathBuf,
    tool: &ToolPreference,
    _complexity_score: &BasicComplexityScore,
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
#[cfg(feature = "advanced_pdf")]
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

/// Store processing telemetry for learning and optimization (non-advanced_pdf version)
#[cfg(not(feature = "advanced_pdf"))]
async fn store_processing_telemetry(
    _database: &ChonkerDatabase,
    pdf_path: &PathBuf,
    complexity_score: &BasicComplexityScore,
    processing_result: &ProcessingResult,
) -> Result<()> {
    debug!("📊 Storing basic processing telemetry");
    
    // Create basic telemetry record
    let telemetry = serde_json::json!({
        "file_path": pdf_path.to_string_lossy(),
        "complexity_score": complexity_score.score,
        "processing_result": {
            "tool_used": processing_result.metadata.tool_used,
            "processing_time_ms": processing_result.metadata.processing_time_ms,
            "processing_path": format!("{:?}", processing_result.processing_path),
            "chunk_count": processing_result.chunks.len(),
            "success": true,
        },
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });
    
    debug!("📊 Basic telemetry: {}", telemetry);
    
    // For now, just log the telemetry
    // TODO: Add actual database storage for telemetry
    
    Ok(())
}
