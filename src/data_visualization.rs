use eframe::egui::{self, Color32, Ui, ScrollArea, RichText};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Document-agnostic extracted data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedData {
    pub source_file: String,
    pub extraction_timestamp: String,
    pub tool_used: String,
    pub processing_time_ms: u64,
    pub content_blocks: Vec<ContentBlock>,
    pub metadata: HashMap<String, String>,
    pub statistics: ExtractionStatistics,
}

/// Generic content block that can represent any type of extracted content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentBlock {
    Table {
        id: String,
        title: Option<String>,
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
        qualifiers: Vec<DataQualifier>,
        metadata: HashMap<String, String>,
    },
    Text {
        id: String,
        title: Option<String>,
        content: String,
        formatting: TextFormatting,
        metadata: HashMap<String, String>,
    },
    List {
        id: String,
        title: Option<String>,
        items: Vec<String>,
        list_type: ListType,
        metadata: HashMap<String, String>,
    },
    Image {
        id: String,
        title: Option<String>,
        caption: Option<String>,
        file_path: Option<String>,
        metadata: HashMap<String, String>,
    },
    Formula {
        id: String,
        latex: String,
        rendered_text: Option<String>,
        metadata: HashMap<String, String>,
    },
    Chart {
        id: String,
        title: Option<String>,
        chart_type: String,
        data: Vec<ChartDataPoint>,
        metadata: HashMap<String, String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataQualifier {
    pub symbol: String,
    pub description: String,
    pub applies_to: Vec<(usize, usize)>, // (row, col) positions
    pub severity: QualifierSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QualifierSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextFormatting {
    pub is_bold: bool,
    pub is_italic: bool,
    pub font_size: Option<f32>,
    pub alignment: TextAlignment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextAlignment {
    Left,
    Center,
    Right,
    Justify,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ListType {
    Bulleted,
    Numbered,
    Nested,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartDataPoint {
    pub label: String,
    pub value: f64,
    pub category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionStatistics {
    pub total_content_blocks: usize,
    pub tables_count: usize,
    pub text_blocks_count: usize,
    pub lists_count: usize,
    pub images_count: usize,
    pub formulas_count: usize,
    pub charts_count: usize,
    pub total_qualifiers: usize,
    pub confidence_score: f32,
    pub quality_issues: Vec<QualityIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityIssue {
    pub block_id: String,
    pub issue_type: IssueType,
    pub description: String,
    pub severity: QualifierSeverity,
    pub suggested_fix: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueType {
    MissingData,
    StructuralError,
    FormattingIssue,
    LowConfidence,
    DataInconsistency,
}

/// Data visualization widget for the GUI
pub struct DataVisualizationPane {
    pub extracted_data: Option<ExtractedData>,
    pub selected_block_index: Option<usize>,
    pub search_filter: String,
    pub show_metadata: bool,
    pub show_qualifiers: bool,
    pub expanded_blocks: Vec<bool>,
}

impl DataVisualizationPane {
    pub fn new() -> Self {
        Self {
            extracted_data: None,
            selected_block_index: None,
            search_filter: String::new(),
            show_metadata: false,
            show_qualifiers: true,
            expanded_blocks: Vec::new(),
        }
    }

    /// Load extracted data into the visualization pane
    pub fn load_data(&mut self, data: ExtractedData) {
        self.expanded_blocks = vec![false; data.content_blocks.len()];
        self.extracted_data = Some(data);
        self.selected_block_index = None;
    }

    /// Clear all data
    pub fn clear(&mut self) {
        self.extracted_data = None;
        self.selected_block_index = None;
        self.expanded_blocks.clear();
        self.search_filter.clear();
    }

    /// Render the data visualization pane
    pub fn render(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            self.render_header(ui);
            ui.separator();
            
            if let Some(ref data) = self.extracted_data.clone() {
                self.render_search_filter(ui);
                ui.separator();
                
                // Always show detailed view
                self.render_detailed_view(ui, &data);
            } else {
                self.render_empty_state(ui);
            }
        });
    }

    fn render_header(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.heading("📊 Extracted Data");
            
            if let Some(ref data) = self.extracted_data {
                ui.separator();
                ui.label(format!("🗂️ {}", data.source_file));
                ui.separator();
                ui.label(format!("⚙️ {}", data.tool_used));
                ui.separator();
                ui.label(format!("⏱️ {}ms", data.processing_time_ms));
            }
        });
    }

    fn render_search_filter(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("🔍 Filter:");
            ui.text_edit_singleline(&mut self.search_filter);
            if ui.button("Clear").clicked() {
                self.search_filter.clear();
            }
            
            ui.separator();
            
            // Keep the useful options
            ui.checkbox(&mut self.show_metadata, "📝 Metadata");
            ui.checkbox(&mut self.show_qualifiers, "🏷️ Qualifiers");
        });
    }

    fn render_overview(&mut self, ui: &mut Ui, data: &ExtractedData) {
        ScrollArea::vertical().show(ui, |ui| {
            // Statistics summary
            self.render_statistics_summary(ui, &data.statistics);
            ui.separator();
            
            // Content blocks overview
            ui.heading("📄 Content Blocks");
            for (index, block) in data.content_blocks.iter().enumerate() {
                if self.matches_filter(block) {
                    self.render_block_summary(ui, block, index);
                }
            }
        });
    }

    fn render_detailed_view(&mut self, ui: &mut Ui, data: &ExtractedData) {
        ScrollArea::vertical().show(ui, |ui| {
            for (index, block) in data.content_blocks.iter().enumerate() {
                if self.matches_filter(block) {
                    let is_expanded = self.expanded_blocks.get(index).copied().unwrap_or(false);
                    
                    let header = match block {
                        ContentBlock::Table { title, headers, .. } => {
                            format!("📊 Table: {} ({}×{})", 
                                title.as_deref().unwrap_or("Untitled"), 
                                headers.len(), 
                                self.get_table_row_count(block))
                        },
                        ContentBlock::Text { title, content, .. } => {
                            format!("📄 Text: {} ({} chars)", 
                                title.as_deref().unwrap_or("Untitled"), 
                                content.len())
                        },
                        ContentBlock::List { title, items, .. } => {
                            format!("📋 List: {} ({} items)", 
                                title.as_deref().unwrap_or("Untitled"), 
                                items.len())
                        },
                        ContentBlock::Image { title, .. } => {
                            format!("🖼️ Image: {}", title.as_deref().unwrap_or("Untitled"))
                        },
                        ContentBlock::Formula { latex, .. } => {
                            format!("🧮 Formula: {}", &latex[..latex.len().min(30)])
                        },
                        ContentBlock::Chart { title, data: chart_data, .. } => {
                            format!("📈 Chart: {} ({} points)", 
                                title.as_deref().unwrap_or("Untitled"), 
                                chart_data.len())
                        },
                    };
                    
                    egui::CollapsingHeader::new(header)
                        .default_open(is_expanded)
                        .show(ui, |ui| {
                            self.render_content_block_detailed(ui, block, index);
                        });
                }
            }
        });
    }

    fn render_tables_only(&mut self, ui: &mut Ui, data: &ExtractedData) {
        ScrollArea::vertical().show(ui, |ui| {
            let table_blocks: Vec<_> = data.content_blocks.iter().enumerate()
                .filter(|(_, block)| matches!(block, ContentBlock::Table { .. }))
                .collect();
                
            if table_blocks.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.label("No tables found in extracted data");
                });
            } else {
                for (index, block) in table_blocks {
                    if let ContentBlock::Table { title, headers, rows, qualifiers, .. } = block {
                        self.render_table_detailed(ui, title, headers, rows, qualifiers, index);
                        ui.separator();
                    }
                }
            }
        });
    }

    fn render_issues_only(&mut self, ui: &mut Ui, data: &ExtractedData) {
        ScrollArea::vertical().show(ui, |ui| {
            if data.statistics.quality_issues.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.label(RichText::new("✅ No quality issues detected").color(Color32::GREEN));
                });
            } else {
                ui.heading(format!("⚠️ {} Quality Issues", data.statistics.quality_issues.len()));
                
                for issue in &data.statistics.quality_issues {
                    self.render_quality_issue(ui, issue);
                }
            }
        });
    }

    fn render_statistics_summary(&mut self, ui: &mut Ui, stats: &ExtractionStatistics) {
        ui.heading("📊 Extraction Statistics");
        
        egui::Grid::new("stats_grid")
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                ui.label("📄 Total Blocks:");
                ui.label(stats.total_content_blocks.to_string());
                ui.end_row();
                
                ui.label("📊 Tables:");
                ui.label(stats.tables_count.to_string());
                ui.end_row();
                
                ui.label("📝 Text Blocks:");
                ui.label(stats.text_blocks_count.to_string());
                ui.end_row();
                
                ui.label("📋 Lists:");
                ui.label(stats.lists_count.to_string());
                ui.end_row();
                
                ui.label("🖼️ Images:");
                ui.label(stats.images_count.to_string());
                ui.end_row();
                
                ui.label("🧮 Formulas:");
                ui.label(stats.formulas_count.to_string());
                ui.end_row();
                
                ui.label("📈 Charts:");
                ui.label(stats.charts_count.to_string());
                ui.end_row();
                
                ui.label("🏷️ Qualifiers:");
                ui.label(stats.total_qualifiers.to_string());
                ui.end_row();
                
                ui.label("🎯 Confidence:");
                let confidence_color = if stats.confidence_score > 0.8 {
                    Color32::GREEN
                } else if stats.confidence_score > 0.6 {
                    Color32::YELLOW
                } else {
                    Color32::RED
                };
                ui.colored_label(confidence_color, format!("{:.1}%", stats.confidence_score * 100.0));
                ui.end_row();
                
                ui.label("⚠️ Issues:");
                let issues_color = if stats.quality_issues.is_empty() {
                    Color32::GREEN
                } else if stats.quality_issues.len() < 3 {
                    Color32::YELLOW
                } else {
                    Color32::RED
                };
                ui.colored_label(issues_color, stats.quality_issues.len().to_string());
                ui.end_row();
            });
    }

    fn render_block_summary(&mut self, ui: &mut Ui, block: &ContentBlock, index: usize) {
        ui.horizontal(|ui| {
            let (icon, title, details) = match block {
                ContentBlock::Table { title, headers, rows, .. } => {
                    ("📊", title.as_deref().unwrap_or("Table"), format!("{}×{}", headers.len(), rows.len()))
                },
                ContentBlock::Text { title, content, .. } => {
                    ("📄", title.as_deref().unwrap_or("Text"), format!("{} chars", content.len()))
                },
                ContentBlock::List { title, items, .. } => {
                    ("📋", title.as_deref().unwrap_or("List"), format!("{} items", items.len()))
                },
                ContentBlock::Image { title, .. } => {
                    ("🖼️", title.as_deref().unwrap_or("Image"), "".to_string())
                },
                ContentBlock::Formula { latex, .. } => {
                    ("🧮", "Formula", latex[..latex.len().min(30)].to_string())
                },
                ContentBlock::Chart { title, data, .. } => {
                    ("📈", title.as_deref().unwrap_or("Chart"), format!("{} points", data.len()))
                },
            };
            
            ui.label(icon);
            ui.label(RichText::new(title).strong());
            ui.label(RichText::new(details).weak());
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("👁 View").clicked() {
                    self.selected_block_index = Some(index);
                }
            });
        });
    }

    fn render_content_block_detailed(&mut self, ui: &mut Ui, block: &ContentBlock, _index: usize) {
        match block {
            ContentBlock::Table { title, headers, rows, qualifiers, metadata, .. } => {
                self.render_table_detailed(ui, title, headers, rows, qualifiers, _index);
                if self.show_metadata && !metadata.is_empty() {
                    self.render_metadata(ui, metadata);
                }
            },
            ContentBlock::Text { content, formatting, metadata, .. } => {
                let mut rich_text = RichText::new(content);
                if formatting.is_bold { rich_text = rich_text.strong(); }
                if formatting.is_italic { rich_text = rich_text.italics(); }
                if let Some(size) = formatting.font_size {
                    rich_text = rich_text.size(size);
                }
                ui.label(rich_text);
                
                if self.show_metadata && !metadata.is_empty() {
                    self.render_metadata(ui, metadata);
                }
            },
            ContentBlock::List { items, list_type, metadata, .. } => {
                for (i, item) in items.iter().enumerate() {
                    ui.horizontal(|ui| {
                        let bullet = match list_type {
                            ListType::Bulleted => "•",
                            ListType::Numbered => &format!("{}.", i + 1),
                            ListType::Nested => "  •",
                        };
                        ui.label(bullet);
                        ui.label(item);
                    });
                }
                
                if self.show_metadata && !metadata.is_empty() {
                    self.render_metadata(ui, metadata);
                }
            },
            ContentBlock::Image { title, caption, file_path, metadata, .. } => {
                ui.heading(title.as_deref().unwrap_or("Image"));
                if let Some(caption) = caption {
                    ui.label(RichText::new(caption).italics());
                }
                if let Some(path) = file_path {
                    ui.label(format!("📁 {}", path));
                }
                
                if self.show_metadata && !metadata.is_empty() {
                    self.render_metadata(ui, metadata);
                }
            },
            ContentBlock::Formula { latex, rendered_text, metadata, .. } => {
                ui.heading("🧮 Formula");
                ui.label(RichText::new(latex).code());
                if let Some(text) = rendered_text {
                    ui.label(format!("Rendered: {}", text));
                }
                
                if self.show_metadata && !metadata.is_empty() {
                    self.render_metadata(ui, metadata);
                }
            },
            ContentBlock::Chart { title, chart_type, data, metadata, .. } => {
                ui.heading(title.as_deref().unwrap_or("Chart"));
                ui.label(format!("Type: {}", chart_type));
                ui.label(format!("Data points: {}", data.len()));
                
                // Simple data preview
                for point in data.iter().take(5) {
                    ui.label(format!("  {} = {}", point.label, point.value));
                }
                if data.len() > 5 {
                    ui.label(format!("  ... and {} more", data.len() - 5));
                }
                
                if self.show_metadata && !metadata.is_empty() {
                    self.render_metadata(ui, metadata);
                }
            },
        }
    }

    fn render_table_detailed(&mut self, ui: &mut Ui, title: &Option<String>, headers: &[String], rows: &[Vec<String>], qualifiers: &[DataQualifier], _index: usize) {
        if let Some(title) = title {
            ui.heading(title);
        }
        
        // Table with headers and data
        egui::Grid::new("data_table")
            .striped(true)
            .show(ui, |ui| {
                // Headers
                for header in headers {
                    ui.label(RichText::new(header).strong());
                }
                ui.end_row();
                
                // Data rows
                for (row_idx, row) in rows.iter().enumerate() {
                    for (col_idx, cell) in row.iter().enumerate() {
                        // Check if this cell has qualifiers
                        let has_qualifier = qualifiers.iter().any(|q| 
                            q.applies_to.contains(&(row_idx, col_idx))
                        );
                        
                        let mut rich_text = RichText::new(cell);
                        if has_qualifier {
                            rich_text = rich_text.color(Color32::BLUE);
                        }
                        
                        ui.label(rich_text);
                    }
                    ui.end_row();
                }
            });
        
        // Show qualifiers if enabled
        if self.show_qualifiers && !qualifiers.is_empty() {
            ui.separator();
            ui.heading("🏷️ Qualifiers");
            for qualifier in qualifiers {
                self.render_qualifier(ui, qualifier);
            }
        }
    }

    fn render_qualifier(&mut self, ui: &mut Ui, qualifier: &DataQualifier) {
        ui.horizontal(|ui| {
            let color = match qualifier.severity {
                QualifierSeverity::Info => Color32::BLUE,
                QualifierSeverity::Warning => Color32::YELLOW,
                QualifierSeverity::Critical => Color32::RED,
            };
            
            ui.colored_label(color, &qualifier.symbol);
            ui.label(&qualifier.description);
            ui.label(format!("(applies to {} cells)", qualifier.applies_to.len()));
        });
    }

    fn render_quality_issue(&mut self, ui: &mut Ui, issue: &QualityIssue) {
        let color = match issue.severity {
            QualifierSeverity::Info => Color32::BLUE,
            QualifierSeverity::Warning => Color32::YELLOW,
            QualifierSeverity::Critical => Color32::RED,
        };
        
        ui.horizontal(|ui| {
            let icon = match issue.issue_type {
                IssueType::MissingData => "❓",
                IssueType::StructuralError => "🔧",
                IssueType::FormattingIssue => "🎨",
                IssueType::LowConfidence => "🤔",
                IssueType::DataInconsistency => "⚠️",
            };
            
            ui.colored_label(color, icon);
            ui.label(RichText::new(&issue.description).strong());
            ui.label(format!("Block: {}", issue.block_id));
        });
        
        if let Some(fix) = &issue.suggested_fix {
            ui.indent("suggested_fix", |ui| {
                ui.label(RichText::new(format!("💡 {}", fix)).italics().color(Color32::GRAY));
            });
        }
    }

    fn render_metadata(&mut self, ui: &mut Ui, metadata: &HashMap<String, String>) {
        ui.collapsing("📝 Metadata", |ui| {
            for (key, value) in metadata {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(key).strong());
                    ui.label(value);
                });
            }
        });
    }

    fn render_empty_state(&mut self, ui: &mut Ui) {
        ui.centered_and_justified(|ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.heading("📊 Data Visualization");
                ui.add_space(20.0);
                ui.label("No extracted data to display");
                ui.label("Process a document to see its extracted content here");
            });
        });
    }

    fn matches_filter(&self, block: &ContentBlock) -> bool {
        if self.search_filter.is_empty() {
            return true;
        }
        
        let filter = self.search_filter.to_lowercase();
        
        match block {
            ContentBlock::Table { title, headers, rows, .. } => {
                title.as_ref().map_or(false, |t| t.to_lowercase().contains(&filter)) ||
                headers.iter().any(|h| h.to_lowercase().contains(&filter)) ||
                rows.iter().any(|row| row.iter().any(|cell| cell.to_lowercase().contains(&filter)))
            },
            ContentBlock::Text { title, content, .. } => {
                title.as_ref().map_or(false, |t| t.to_lowercase().contains(&filter)) ||
                content.to_lowercase().contains(&filter)
            },
            ContentBlock::List { title, items, .. } => {
                title.as_ref().map_or(false, |t| t.to_lowercase().contains(&filter)) ||
                items.iter().any(|item| item.to_lowercase().contains(&filter))
            },
            ContentBlock::Image { title, caption, .. } => {
                title.as_ref().map_or(false, |t| t.to_lowercase().contains(&filter)) ||
                caption.as_ref().map_or(false, |c| c.to_lowercase().contains(&filter))
            },
            ContentBlock::Formula { latex, rendered_text, .. } => {
                latex.to_lowercase().contains(&filter) ||
                rendered_text.as_ref().map_or(false, |t| t.to_lowercase().contains(&filter))
            },
            ContentBlock::Chart { title, .. } => {
                title.as_ref().map_or(false, |t| t.to_lowercase().contains(&filter))
            },
        }
    }

    fn get_table_row_count(&self, block: &ContentBlock) -> usize {
        match block {
            ContentBlock::Table { rows, .. } => rows.len(),
            _ => 0,
        }
    }
}

impl Default for DataVisualizationPane {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper functions to create sample data for testing
impl ExtractedData {
    pub fn create_sample() -> Self {
        let mut metadata = HashMap::new();
        metadata.insert("pages".to_string(), "3".to_string());
        metadata.insert("language".to_string(), "English".to_string());
        
        let content_blocks = vec![
            ContentBlock::Table {
                id: "table_1".to_string(),
                title: Some("Sample Data Table".to_string()),
                headers: vec!["Parameter".to_string(), "Value".to_string(), "Unit".to_string()],
                rows: vec![
                    vec!["Temperature".to_string(), "25.4".to_string(), "°C".to_string()],
                    vec!["Pressure".to_string(), "1013.25".to_string(), "hPa".to_string()],
                    vec!["Humidity".to_string(), "65".to_string(), "%".to_string()],
                ],
                qualifiers: vec![
                    DataQualifier {
                        symbol: "U".to_string(),
                        description: "Uncertainty in measurement".to_string(),
                        applies_to: vec![(0, 1)],
                        severity: QualifierSeverity::Warning,
                    }
                ],
                metadata: HashMap::new(),
            },
            ContentBlock::Text {
                id: "text_1".to_string(),
                title: Some("Introduction".to_string()),
                content: "This document contains sample extracted data for testing the visualization component.".to_string(),
                formatting: TextFormatting {
                    is_bold: false,
                    is_italic: false,
                    font_size: None,
                    alignment: TextAlignment::Left,
                },
                metadata: HashMap::new(),
            },
        ];
        
        let statistics = ExtractionStatistics {
            total_content_blocks: content_blocks.len(),
            tables_count: 1,
            text_blocks_count: 1,
            lists_count: 0,
            images_count: 0,
            formulas_count: 0,
            charts_count: 0,
            total_qualifiers: 1,
            confidence_score: 0.92,
            quality_issues: vec![],
        };
        
        Self {
            source_file: "sample_document.pdf".to_string(),
            extraction_timestamp: "2024-01-01T12:00:00Z".to_string(),
            tool_used: "CHONKER Enhanced Docling".to_string(),
            processing_time_ms: 1250,
            content_blocks,
            metadata,
            statistics,
        }
    }
}
