use crate::data_visualization::{
    ExtractedData, ContentBlock, ExtractionStatistics, QualityIssue, IssueType, 
    DataQualifier, QualifierSeverity, TextFormatting, TextAlignment, ListType,
    ChartDataPoint
};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use anyhow::Result;

/// Integration bridge to convert Python extraction output to GUI data format
pub struct ExtractionIntegrator {
    python_venv_path: String,
    extraction_script_path: String,
}

impl ExtractionIntegrator {
    pub fn new() -> Self {
        Self {
            python_venv_path: "venv/bin/python".to_string(),
            extraction_script_path: "python/extraction_bridge.py".to_string(),
        }
    }

    /// Process a PDF file and return extracted data for visualization
    pub async fn process_document(&self, pdf_path: &Path) -> Result<ExtractedData> {
        // Run the Python extraction script
        let extraction_output = self.run_extraction(pdf_path).await?;
        
        // Convert the JSON output to our visualization format
        self.convert_extraction_output(pdf_path, extraction_output)
    }

    /// Run the Python extraction bridge script
    async fn run_extraction(&self, pdf_path: &Path) -> Result<Value> {
        let output = Command::new(&self.python_venv_path)
            .arg(&self.extraction_script_path)
            .arg(pdf_path)
            .arg("--tool")
            .arg("docling_enhanced")
            .arg("--output-format")
            .arg("json")
            .output()?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "Extraction failed: {}", 
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let json_output = String::from_utf8(output.stdout)?;
        let parsed: Value = serde_json::from_str(&json_output)?;
        Ok(parsed)
    }

    /// Convert Python extraction output to visualization data format
    fn convert_extraction_output(&self, pdf_path: &Path, json_data: Value) -> Result<ExtractedData> {
        let start_time = std::time::Instant::now();
        
        // Extract metadata
        let tool_used = json_data["metadata"]["tool"].as_str()
            .unwrap_or("CHONKER Enhanced Docling").to_string();
        
        let processing_time_ms = json_data["metadata"]["processing_time_ms"].as_u64()
            .unwrap_or_else(|| start_time.elapsed().as_millis() as u64);

        // Extract document metadata
        let mut document_metadata = HashMap::new();
        if let Some(meta) = json_data["metadata"].as_object() {
            for (key, value) in meta {
                if let Some(str_value) = value.as_str() {
                    document_metadata.insert(key.clone(), str_value.to_string());
                }
            }
        }

        // Process content blocks
        let mut content_blocks = Vec::new();
        let mut tables_count = 0;
        let mut text_blocks_count = 0;
        let mut lists_count = 0;
        let mut images_count = 0;
        let mut formulas_count = 0;
        let mut charts_count = 0;
        let mut total_qualifiers = 0;
        let mut quality_issues = Vec::new();

        if let Some(blocks) = json_data["content_blocks"].as_array() {
            for (index, block) in blocks.iter().enumerate() {
                let content_block = self.convert_content_block(block, index, &mut quality_issues)?;
                
                // Count content types and qualifiers
                match &content_block {
                    ContentBlock::Table { qualifiers, .. } => {
                        tables_count += 1;
                        total_qualifiers += qualifiers.len();
                    },
                    ContentBlock::Text { .. } => text_blocks_count += 1,
                    ContentBlock::List { .. } => lists_count += 1,
                    ContentBlock::Image { .. } => images_count += 1,
                    ContentBlock::Formula { .. } => formulas_count += 1,
                    ContentBlock::Chart { .. } => charts_count += 1,
                }
                
                content_blocks.push(content_block);
            }
        }

        // Calculate confidence score (placeholder - would come from extraction metadata)
        let confidence_score = json_data["metadata"]["confidence_score"].as_f64()
            .unwrap_or(0.85) as f32;

        let statistics = ExtractionStatistics {
            total_content_blocks: content_blocks.len(),
            tables_count,
            text_blocks_count,
            lists_count,
            images_count,
            formulas_count,
            charts_count,
            total_qualifiers,
            confidence_score,
            quality_issues,
        };

        Ok(ExtractedData {
            source_file: pdf_path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("unknown.pdf")
                .to_string(),
            extraction_timestamp: chrono::Utc::now().to_rfc3339(),
            tool_used,
            processing_time_ms,
            content_blocks,
            metadata: document_metadata,
            statistics,
        })
    }

    /// Convert a single content block from JSON to our format
    fn convert_content_block(&self, block: &Value, index: usize, quality_issues: &mut Vec<QualityIssue>) -> Result<ContentBlock> {
        let block_type = block["type"].as_str().unwrap_or("unknown");
        let block_id = format!("block_{}", index);

        match block_type {
            "table" => self.convert_table_block(block, block_id, quality_issues),
            "text" | "paragraph" => self.convert_text_block(block, block_id, quality_issues),
            "list" => self.convert_list_block(block, block_id, quality_issues),
            "image" | "figure" => self.convert_image_block(block, block_id),
            "formula" | "equation" => self.convert_formula_block(block, block_id),
            "chart" | "graph" => self.convert_chart_block(block, block_id),
            _ => {
                // Convert unknown content as text
                Ok(ContentBlock::Text {
                    id: block_id,
                    title: block["title"].as_str().map(|s| s.to_string()),
                    content: block["content"].as_str().unwrap_or("").to_string(),
                    formatting: TextFormatting {
                        is_bold: false,
                        is_italic: false,
                        font_size: None,
                        alignment: TextAlignment::Left,
                    },
                    metadata: HashMap::new(),
                })
            }
        }
    }

    fn convert_table_block(&self, block: &Value, block_id: String, quality_issues: &mut Vec<QualityIssue>) -> Result<ContentBlock> {
        let title = block["title"].as_str().map(|s| s.to_string());
        
        // Extract headers
        let headers = if let Some(headers_array) = block["headers"].as_array() {
            headers_array.iter()
                .filter_map(|h| h.as_str())
                .map(|s| s.to_string())
                .collect()
        } else {
            Vec::new()
        };

        // Extract rows
        let rows = if let Some(rows_array) = block["rows"].as_array() {
            rows_array.iter()
                .filter_map(|row| {
                    row.as_array().map(|cells| {
                        cells.iter()
                            .filter_map(|cell| cell.as_str())
                            .map(|s| s.to_string())
                            .collect()
                    })
                })
                .collect()
        } else {
            Vec::new()
        };

        // Extract qualifiers
        let mut qualifiers = Vec::new();
        if let Some(quals) = block["qualifiers"].as_array() {
            for qual in quals {
                if let Some(qualifier) = self.convert_qualifier(qual) {
                    qualifiers.push(qualifier);
                }
            }
        }

        // Check for data quality issues
        self.detect_table_issues(&rows, &headers, &block_id, quality_issues);

        let mut metadata = HashMap::new();
        if let Some(meta) = block["metadata"].as_object() {
            for (key, value) in meta {
                if let Some(str_value) = value.as_str() {
                    metadata.insert(key.clone(), str_value.to_string());
                }
            }
        }

        Ok(ContentBlock::Table {
            id: block_id,
            title,
            headers,
            rows,
            qualifiers,
            metadata,
        })
    }

    fn convert_text_block(&self, block: &Value, block_id: String, _quality_issues: &mut Vec<QualityIssue>) -> Result<ContentBlock> {
        let title = block["title"].as_str().map(|s| s.to_string());
        let content = block["content"].as_str().unwrap_or("").to_string();
        
        let formatting = TextFormatting {
            is_bold: block["formatting"]["bold"].as_bool().unwrap_or(false),
            is_italic: block["formatting"]["italic"].as_bool().unwrap_or(false),
            font_size: block["formatting"]["font_size"].as_f64().map(|f| f as f32),
            alignment: match block["formatting"]["alignment"].as_str() {
                Some("center") => TextAlignment::Center,
                Some("right") => TextAlignment::Right,
                Some("justify") => TextAlignment::Justify,
                _ => TextAlignment::Left,
            },
        };

        let mut metadata = HashMap::new();
        if let Some(meta) = block["metadata"].as_object() {
            for (key, value) in meta {
                if let Some(str_value) = value.as_str() {
                    metadata.insert(key.clone(), str_value.to_string());
                }
            }
        }

        Ok(ContentBlock::Text {
            id: block_id,
            title,
            content,
            formatting,
            metadata,
        })
    }

    fn convert_list_block(&self, block: &Value, block_id: String, _quality_issues: &mut Vec<QualityIssue>) -> Result<ContentBlock> {
        let title = block["title"].as_str().map(|s| s.to_string());
        
        let items = if let Some(items_array) = block["items"].as_array() {
            items_array.iter()
                .filter_map(|item| item.as_str())
                .map(|s| s.to_string())
                .collect()
        } else {
            Vec::new()
        };

        let list_type = match block["list_type"].as_str() {
            Some("numbered") => ListType::Numbered,
            Some("nested") => ListType::Nested,
            _ => ListType::Bulleted,
        };

        let mut metadata = HashMap::new();
        if let Some(meta) = block["metadata"].as_object() {
            for (key, value) in meta {
                if let Some(str_value) = value.as_str() {
                    metadata.insert(key.clone(), str_value.to_string());
                }
            }
        }

        Ok(ContentBlock::List {
            id: block_id,
            title,
            items,
            list_type,
            metadata,
        })
    }

    fn convert_image_block(&self, block: &Value, block_id: String) -> Result<ContentBlock> {
        let title = block["title"].as_str().map(|s| s.to_string());
        let caption = block["caption"].as_str().map(|s| s.to_string());
        let file_path = block["file_path"].as_str().map(|s| s.to_string());

        let mut metadata = HashMap::new();
        if let Some(meta) = block["metadata"].as_object() {
            for (key, value) in meta {
                if let Some(str_value) = value.as_str() {
                    metadata.insert(key.clone(), str_value.to_string());
                }
            }
        }

        Ok(ContentBlock::Image {
            id: block_id,
            title,
            caption,
            file_path,
            metadata,
        })
    }

    fn convert_formula_block(&self, block: &Value, block_id: String) -> Result<ContentBlock> {
        let latex = block["latex"].as_str().unwrap_or("").to_string();
        let rendered_text = block["rendered_text"].as_str().map(|s| s.to_string());

        let mut metadata = HashMap::new();
        if let Some(meta) = block["metadata"].as_object() {
            for (key, value) in meta {
                if let Some(str_value) = value.as_str() {
                    metadata.insert(key.clone(), str_value.to_string());
                }
            }
        }

        Ok(ContentBlock::Formula {
            id: block_id,
            latex,
            rendered_text,
            metadata,
        })
    }

    fn convert_chart_block(&self, block: &Value, block_id: String) -> Result<ContentBlock> {
        let title = block["title"].as_str().map(|s| s.to_string());
        let chart_type = block["chart_type"].as_str().unwrap_or("unknown").to_string();
        
        let data = if let Some(data_array) = block["data"].as_array() {
            data_array.iter()
                .filter_map(|point| {
                    Some(ChartDataPoint {
                        label: point["label"].as_str()?.to_string(),
                        value: point["value"].as_f64()?,
                        category: point["category"].as_str().map(|s| s.to_string()),
                    })
                })
                .collect()
        } else {
            Vec::new()
        };

        let mut metadata = HashMap::new();
        if let Some(meta) = block["metadata"].as_object() {
            for (key, value) in meta {
                if let Some(str_value) = value.as_str() {
                    metadata.insert(key.clone(), str_value.to_string());
                }
            }
        }

        Ok(ContentBlock::Chart {
            id: block_id,
            title,
            chart_type,
            data,
            metadata,
        })
    }

    fn convert_qualifier(&self, qual: &Value) -> Option<DataQualifier> {
        let symbol = qual["symbol"].as_str()?.to_string();
        let description = qual["description"].as_str()?.to_string();
        
        let applies_to = if let Some(positions) = qual["applies_to"].as_array() {
            positions.iter()
                .filter_map(|pos| {
                    let row = pos[0].as_u64()? as usize;
                    let col = pos[1].as_u64()? as usize;
                    Some((row, col))
                })
                .collect()
        } else {
            Vec::new()
        };

        let severity = match qual["severity"].as_str() {
            Some("critical") => QualifierSeverity::Critical,
            Some("warning") => QualifierSeverity::Warning,
            _ => QualifierSeverity::Info,
        };

        Some(DataQualifier {
            symbol,
            description,
            applies_to,
            severity,
        })
    }

    fn detect_table_issues(&self, rows: &[Vec<String>], headers: &[String], block_id: &str, quality_issues: &mut Vec<QualityIssue>) {
        // Check for empty cells
        for (row_idx, row) in rows.iter().enumerate() {
            for (col_idx, cell) in row.iter().enumerate() {
                if cell.trim().is_empty() {
                    quality_issues.push(QualityIssue {
                        block_id: block_id.to_string(),
                        issue_type: IssueType::MissingData,
                        description: format!("Empty cell at row {}, column {}", row_idx + 1, col_idx + 1),
                        severity: QualifierSeverity::Warning,
                        suggested_fix: Some("Review source document for missing data".to_string()),
                    });
                }
            }
        }

        // Check for inconsistent row lengths
        if !rows.is_empty() {
            let expected_cols = headers.len().max(rows[0].len());
            for (row_idx, row) in rows.iter().enumerate() {
                if row.len() != expected_cols {
                    quality_issues.push(QualityIssue {
                        block_id: block_id.to_string(),
                        issue_type: IssueType::StructuralError,
                        description: format!("Row {} has {} columns, expected {}", row_idx + 1, row.len(), expected_cols),
                        severity: QualifierSeverity::Critical,
                        suggested_fix: Some("Check for merged cells or parsing errors".to_string()),
                    });
                }
            }
        }
    }
}

impl Default for ExtractionIntegrator {
    fn default() -> Self {
        Self::new()
    }
}

/// Example usage for ChonkerApp integration
#[cfg(feature = "gui")]
pub mod gui_integration {
    use super::*;
    use crate::data_visualization::DataVisualizationPane;
    use std::path::PathBuf;

    /// Helper function to integrate extraction with GUI
    pub async fn process_and_visualize(
        pdf_path: PathBuf,
        viz_pane: &mut DataVisualizationPane,
    ) -> Result<()> {
        let integrator = ExtractionIntegrator::new();
        
        // Process the document
        let extracted_data = integrator.process_document(&pdf_path).await?;
        
        // Load into visualization pane
        viz_pane.load_data(extracted_data);
        
        Ok(())
    }

    /// Create sample data for testing
    pub fn load_sample_data(viz_pane: &mut DataVisualizationPane) {
        let sample_data = ExtractedData::create_sample();
        viz_pane.load_data(sample_data);
    }
}
