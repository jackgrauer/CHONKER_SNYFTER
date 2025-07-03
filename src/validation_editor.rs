use std::collections::HashMap;
#[cfg(feature = "gui")]
use eframe::egui::{self, *};

#[derive(Debug, Clone)]
pub struct ValidationPane {
    pub content_blocks: Vec<ContentBlock>,
    pub issues: Vec<ValidationIssue>,
    pub view_mode: ViewMode, // IssueFirst | Structure | Linear
    pub editing_cell: Option<(String, usize, usize)>, // (block_id, row, col)
    pub editing_text: String,
    pub collapsed_blocks: HashMap<String, bool>, // Block collapse state
}

#[derive(Debug, Clone)]
pub enum ContentBlock {
    Table(TableBlock),
    List(ListBlock),
    Text(TextBlock),
    Form(FormBlock),
    Unknown(RawBlock),
}

#[derive(Debug, Clone)]
pub struct TableBlock {
    id: String,
    data: Vec<Vec<String>>, // 2D Vector for table cells
}

#[derive(Debug, Clone)]
pub struct ListBlock {
    id: String,
    items: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TextBlock {
    id: String,
    text: String,
}

#[derive(Debug, Clone)]
pub struct FormBlock {
    id: String,
    fields: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct RawBlock {
    id: String,
    raw_content: String,
}

#[derive(Debug, Clone)]
pub struct ValidationIssue {
    block_id: String,
    location: ContentLocation, // Where in the block
    issue_type: IssueType,
    severity: Severity,
}

#[derive(Debug, Clone)]
pub struct ContentLocation {
    line: Option<usize>,
    column: Option<usize>,
}

#[derive(Debug, Clone)]
pub enum IssueType {
    Truncated,        // "Departme..."
    MissingValue,     // Empty cell/field
    LowConfidence,    // OCR uncertainty
    StructuralError,  // Broken table/list
}

#[derive(Debug, Clone)]
pub enum Severity {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone)]
pub enum ViewMode {
    IssueFirst,
    Structure,
    Linear,
}

impl ValidationPane {
    pub fn new() -> Self {
        Self {
            content_blocks: Vec::new(),
            issues: Vec::new(),
            view_mode: ViewMode::Structure,
            editing_cell: None,
            editing_text: String::new(),
            collapsed_blocks: HashMap::new(),
        }
    }
    
    pub fn from_docling_output(docling_json: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let parsed: serde_json::Value = serde_json::from_str(docling_json)?;
        let mut validation_pane = Self::new();
        
        // Parse Docling content into structured blocks
        if let Some(content) = parsed.get("content") {
            validation_pane.parse_docling_content(content)?;
        } else if let Some(extractions) = parsed.get("extractions") {
            // Handle extraction bridge format
            validation_pane.parse_extraction_bridge_format(extractions)?;
        } else {
            // Fallback: create a simple text block from any text we can find
            let text_content = parsed.get("text")
                .and_then(|t| t.as_str())
                .or_else(|| parsed.as_str())
                .unwrap_or("No content found");
            
            validation_pane.content_blocks.push(ContentBlock::Text(TextBlock {
                id: "fallback_text".to_string(),
                text: text_content.to_string(),
            }));
        }
        
        // Extract issues from low-confidence areas
        validation_pane.detect_validation_issues();
        
        Ok(validation_pane)
    }
    
    fn parse_docling_content(&mut self, content: &serde_json::Value) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(array) = content.as_array() {
            for (idx, item) in array.iter().enumerate() {
                let block_id = format!("block_{}", idx);
                
                if let Some(item_type) = item.get("type").and_then(|t| t.as_str()) {
                    match item_type {
                        "table" => {
                            if let Some(table_data) = self.parse_table(item) {
                                self.content_blocks.push(ContentBlock::Table(TableBlock {
                                    id: block_id,
                                    data: table_data,
                                }));
                            }
                        },
                        "list" => {
                            if let Some(list_items) = self.parse_list(item) {
                                self.content_blocks.push(ContentBlock::List(ListBlock {
                                    id: block_id,
                                    items: list_items,
                                }));
                            }
                        },
                        "text" | "paragraph" => {
                            if let Some(text_content) = item.get("text").and_then(|t| t.as_str()) {
                                self.content_blocks.push(ContentBlock::Text(TextBlock {
                                    id: block_id,
                                    text: text_content.to_string(),
                                }));
                            }
                        },
                        _ => {
                            // Unknown content type - store as raw
                            self.content_blocks.push(ContentBlock::Unknown(RawBlock {
                                id: block_id,
                                raw_content: item.to_string(),
                            }));
                        }
                    }
                }
            }
        }
        Ok(())
    }
    
    fn parse_extraction_bridge_format(&mut self, extractions: &serde_json::Value) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(array) = extractions.as_array() {
            for (idx, extraction) in array.iter().enumerate() {
                let block_id = format!("extraction_{}", idx);
                
                // Extract text content
                if let Some(text) = extraction.get("text").and_then(|t| t.as_str()) {
                    if !text.trim().is_empty() {
                        self.content_blocks.push(ContentBlock::Text(TextBlock {
                            id: format!("{}_text", block_id),
                            text: text.to_string(),
                        }));
                    }
                }
                
                // Extract tables if present
                if let Some(tables) = extraction.get("tables").and_then(|t| t.as_array()) {
                    for (table_idx, table) in tables.iter().enumerate() {
                        if let Some(table_data) = self.parse_table_from_extraction(table) {
                            self.content_blocks.push(ContentBlock::Table(TableBlock {
                                id: format!("{}_table_{}", block_id, table_idx),
                                data: table_data,
                            }));
                        }
                    }
                }
                
                // Extract figures if present
                if let Some(figures) = extraction.get("figures").and_then(|f| f.as_array()) {
                    for (fig_idx, figure) in figures.iter().enumerate() {
                        self.content_blocks.push(ContentBlock::Unknown(RawBlock {
                            id: format!("{}_figure_{}", block_id, fig_idx),
                            raw_content: format!("Figure: {}", figure.to_string()),
                        }));
                    }
                }
            }
        }
        Ok(())
    }
    
    fn parse_table(&self, item: &serde_json::Value) -> Option<Vec<Vec<String>>> {
        if let Some(rows) = item.get("data").and_then(|d| d.as_array()) {
            let mut table_data = Vec::new();
            for row in rows {
                if let Some(cells) = row.as_array() {
                    let row_data: Vec<String> = cells.iter()
                        .map(|cell| cell.as_str().unwrap_or("").to_string())
                        .collect();
                    table_data.push(row_data);
                }
            }
            Some(table_data)
        } else {
            None
        }
    }
    
    fn parse_list(&self, item: &serde_json::Value) -> Option<Vec<String>> {
        if let Some(items) = item.get("items").and_then(|i| i.as_array()) {
            let list_items: Vec<String> = items.iter()
                .map(|item| item.as_str().unwrap_or("").to_string())
                .collect();
            Some(list_items)
        } else {
            None
        }
    }
    
    fn parse_table_from_extraction(&self, table: &serde_json::Value) -> Option<Vec<Vec<String>>> {
        // Try to extract table data from various possible formats
        if let Some(rows) = table.get("rows").and_then(|r| r.as_array()) {
            let mut table_data = Vec::new();
            for row in rows {
                if let Some(cells) = row.get("cells").and_then(|c| c.as_array()) {
                    let row_data: Vec<String> = cells.iter()
                        .map(|cell| cell.as_str().unwrap_or("").to_string())
                        .collect();
                    table_data.push(row_data);
                } else if let Some(cells) = row.as_array() {
                    let row_data: Vec<String> = cells.iter()
                        .map(|cell| cell.as_str().unwrap_or("").to_string())
                        .collect();
                    table_data.push(row_data);
                }
            }
            Some(table_data)
        } else if let Some(data) = table.get("data").and_then(|d| d.as_array()) {
            // Alternative format with direct data array
            let mut table_data = Vec::new();
            for row in data {
                if let Some(cells) = row.as_array() {
                    let row_data: Vec<String> = cells.iter()
                        .map(|cell| cell.as_str().unwrap_or("").to_string())
                        .collect();
                    table_data.push(row_data);
                }
            }
            Some(table_data)
        } else {
            None
        }
    }
    
    fn detect_validation_issues(&mut self) {
        let mut new_issues = Vec::new();
        
        for block in &self.content_blocks {
            match block {
                ContentBlock::Table(table) => {
                    Self::detect_table_issues_static(table, &mut new_issues);
                },
                ContentBlock::Text(text) => {
                    Self::detect_text_issues_static(text, &mut new_issues);
                },
                ContentBlock::List(list) => {
                    Self::detect_list_issues_static(list, &mut new_issues);
                },
                _ => {}
            }
        }
        
        self.issues.extend(new_issues);
    }
    
    fn detect_table_issues(&mut self, table: &TableBlock) {
        for (row_idx, row) in table.data.iter().enumerate() {
            for (col_idx, cell) in row.iter().enumerate() {
                // Detect truncated text
                if cell.ends_with("...") || cell.ends_with("…") {
                    self.issues.push(ValidationIssue {
                        block_id: table.id.clone(),
                        location: ContentLocation {
                            line: Some(row_idx),
                            column: Some(col_idx),
                        },
                        issue_type: IssueType::Truncated,
                        severity: Severity::High,
                    });
                }
                
                // Detect missing values
                if cell.trim().is_empty() {
                    self.issues.push(ValidationIssue {
                        block_id: table.id.clone(),
                        location: ContentLocation {
                            line: Some(row_idx),
                            column: Some(col_idx),
                        },
                        issue_type: IssueType::MissingValue,
                        severity: Severity::Medium,
                    });
                }
            }
        }
    }
    
    fn detect_text_issues(&mut self, text: &TextBlock) {
        // Detect truncated text
        if text.text.ends_with("...") || text.text.ends_with("…") {
            self.issues.push(ValidationIssue {
                block_id: text.id.clone(),
                location: ContentLocation {
                    line: None,
                    column: None,
                },
                issue_type: IssueType::Truncated,
                severity: Severity::High,
            });
        }
    }
    
    fn detect_list_issues(&mut self, list: &ListBlock) {
        for (idx, item) in list.items.iter().enumerate() {
            if item.ends_with("...") || item.ends_with("…") {
                self.issues.push(ValidationIssue {
                    block_id: list.id.clone(),
                    location: ContentLocation {
                        line: Some(idx),
                        column: None,
                    },
                    issue_type: IssueType::Truncated,
                    severity: Severity::High,
                });
            }
        }
    }
    
    // Static methods for issue detection to avoid borrow checker issues
    fn detect_table_issues_static(table: &TableBlock, issues: &mut Vec<ValidationIssue>) {
        for (row_idx, row) in table.data.iter().enumerate() {
            for (col_idx, cell) in row.iter().enumerate() {
                // Detect truncated text
                if cell.ends_with("...") || cell.ends_with("…") {
                    issues.push(ValidationIssue {
                        block_id: table.id.clone(),
                        location: ContentLocation {
                            line: Some(row_idx),
                            column: Some(col_idx),
                        },
                        issue_type: IssueType::Truncated,
                        severity: Severity::High,
                    });
                }
                
                // Detect missing values
                if cell.trim().is_empty() {
                    issues.push(ValidationIssue {
                        block_id: table.id.clone(),
                        location: ContentLocation {
                            line: Some(row_idx),
                            column: Some(col_idx),
                        },
                        issue_type: IssueType::MissingValue,
                        severity: Severity::Medium,
                    });
                }
            }
        }
    }
    
    fn detect_text_issues_static(text: &TextBlock, issues: &mut Vec<ValidationIssue>) {
        // Detect truncated text
        if text.text.ends_with("...") || text.text.ends_with("…") {
            issues.push(ValidationIssue {
                block_id: text.id.clone(),
                location: ContentLocation {
                    line: None,
                    column: None,
                },
                issue_type: IssueType::Truncated,
                severity: Severity::High,
            });
        }
    }
    
    fn detect_list_issues_static(list: &ListBlock, issues: &mut Vec<ValidationIssue>) {
        for (idx, item) in list.items.iter().enumerate() {
            if item.ends_with("...") || item.ends_with("…") {
                issues.push(ValidationIssue {
                    block_id: list.id.clone(),
                    location: ContentLocation {
                        line: Some(idx),
                        column: None,
                    },
                    issue_type: IssueType::Truncated,
                    severity: Severity::High,
                });
            }
        }
    }
    
    #[cfg(feature = "gui")]
    pub fn render(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            // Header with view mode selector
            ui.horizontal(|ui| {
                ui.heading("📝 Semantic Document Editor");
                ui.separator();
                
                // View mode selector
                ui.label("View:");
                if ui.selectable_label(matches!(self.view_mode, ViewMode::Structure), "📁 Structure").clicked() {
                    self.view_mode = ViewMode::Structure;
                }
                if ui.selectable_label(matches!(self.view_mode, ViewMode::IssueFirst), "⚠️ Issues First").clicked() {
                    self.view_mode = ViewMode::IssueFirst;
                }
                if ui.selectable_label(matches!(self.view_mode, ViewMode::Linear), "📄 Linear").clicked() {
                    self.view_mode = ViewMode::Linear;
                }
            });
            
            ui.separator();
            
            // Issue summary
            if !self.issues.is_empty() {
                ui.horizontal(|ui| {
                    let high_issues = self.issues.iter().filter(|i| matches!(i.severity, Severity::High)).count();
                    let medium_issues = self.issues.iter().filter(|i| matches!(i.severity, Severity::Medium)).count();
                    let low_issues = self.issues.iter().filter(|i| matches!(i.severity, Severity::Low)).count();
                    
                    if high_issues > 0 {
                        ui.colored_label(egui::Color32::RED, format!("🔴 {} High", high_issues));
                    }
                    if medium_issues > 0 {
                        ui.colored_label(egui::Color32::YELLOW, format!("🟡 {} Medium", medium_issues));
                    }
                    if low_issues > 0 {
                        ui.colored_label(egui::Color32::GREEN, format!("🟢 {} Low", low_issues));
                    }
                });
                ui.separator();
            }
            
            // Render content based on view mode
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    match self.view_mode {
                        ViewMode::IssueFirst => self.render_issues_first(ui),
                        ViewMode::Structure => self.render_structure_view(ui),
                        ViewMode::Linear => self.render_linear_view(ui),
                    }
                });
        });
    }
    
    #[cfg(feature = "gui")]
    fn render_issues_first(&mut self, ui: &mut Ui) {
        // First show all issues grouped by severity
        if !self.issues.is_empty() {
            ui.heading("🚨 Validation Issues");
            
            for severity in [Severity::High, Severity::Medium, Severity::Low] {
                let severity_issues: Vec<_> = self.issues.iter()
                    .filter(|issue| matches!(&issue.severity, severity))
                    .collect();
                    
                if !severity_issues.is_empty() {
                    let (_color, icon, label) = match severity {
                        Severity::High => (egui::Color32::RED, "🔴", "High Priority"),
                        Severity::Medium => (egui::Color32::YELLOW, "🟡", "Medium Priority"),
                        Severity::Low => (egui::Color32::GREEN, "🟢", "Low Priority"),
                    };
                    
                    egui::CollapsingHeader::new(format!("{} {} ({})", icon, label, severity_issues.len()))
                        .default_open(matches!(severity, Severity::High))
                        .show(ui, |ui| {
                            for issue in severity_issues {
                                Self::render_issue_card_static(ui, issue);
                            }
                        });
                }
            }
            
            ui.separator();
        }
        
        // Then show content with issues highlighted
        ui.heading("📄 Content (Issues Highlighted)");
        let block_count = self.content_blocks.len();
        for i in 0..block_count {
            // Use index-based iteration to avoid multiple mutable borrows
            let _block_id = self.get_block_id(&self.content_blocks[i]);
            let issues = &self.issues; // Borrow issues separately
            match &mut self.content_blocks[i] {
                ContentBlock::Table(table) => Self::render_table_editor_static(ui, table, issues),
                ContentBlock::List(list) => Self::render_list_editor_static(ui, list, issues),
                ContentBlock::Text(text) => Self::render_text_editor_static(ui, text, issues),
                ContentBlock::Form(form) => Self::render_form_editor_static(ui, form),
                ContentBlock::Unknown(raw) => Self::render_raw_editor_static(ui, raw),
            }
        }
    }
    
    #[cfg(feature = "gui")]
    fn render_structure_view(&mut self, ui: &mut Ui) {
        ui.heading("📁 Document Structure");
        
        let block_count = self.content_blocks.len();
        for i in 0..block_count {
            // Use index-based iteration to avoid multiple mutable borrows
            let block_id = self.get_block_id(&self.content_blocks[i]);
            let is_collapsed = self.collapsed_blocks.get(&block_id).copied().unwrap_or(false);
            
            let header_text = match &self.content_blocks[i] {
                ContentBlock::Table(table) => format!("📊 Table ({} rows)", table.data.len()),
                ContentBlock::List(list) => format!("📋 List ({} items)", list.items.len()),
                ContentBlock::Text(_) => "📄 Text Block".to_string(),
                ContentBlock::Form(form) => format!("📝 Form ({} fields)", form.fields.len()),
                ContentBlock::Unknown(_) => "❓ Unknown Content".to_string(),
            };
            
            egui::CollapsingHeader::new(header_text)
                .default_open(!is_collapsed)
                .show(ui, |ui| {
                    let issues = &self.issues; // Borrow issues separately
                    match &mut self.content_blocks[i] {
                        ContentBlock::Table(table) => Self::render_table_editor_static(ui, table, issues),
                        ContentBlock::List(list) => Self::render_list_editor_static(ui, list, issues),
                        ContentBlock::Text(text) => Self::render_text_editor_static(ui, text, issues),
                        ContentBlock::Form(form) => Self::render_form_editor_static(ui, form),
                        ContentBlock::Unknown(raw) => Self::render_raw_editor_static(ui, raw),
                    }
                });
        }
    }
    
    #[cfg(feature = "gui")]
    fn render_linear_view(&mut self, ui: &mut Ui) {
        ui.heading("📄 Linear Document View");
        
        let block_count = self.content_blocks.len();
        for i in 0..block_count {
            // Use index-based iteration to avoid multiple mutable borrows
            let issues = &self.issues; // Borrow issues separately
            match &mut self.content_blocks[i] {
                ContentBlock::Table(table) => Self::render_table_editor_static(ui, table, issues),
                ContentBlock::List(list) => Self::render_list_editor_static(ui, list, issues),
                ContentBlock::Text(text) => Self::render_text_editor_static(ui, text, issues),
                ContentBlock::Form(form) => Self::render_form_editor_static(ui, form),
                ContentBlock::Unknown(raw) => Self::render_raw_editor_static(ui, raw),
            }
            ui.separator();
        }
    }
    
    #[cfg(feature = "gui")]
    fn render_issue_card(&self, ui: &mut Ui, issue: &ValidationIssue) {
        ui.horizontal(|ui| {
            let (color, icon) = match issue.issue_type {
                IssueType::Truncated => (egui::Color32::RED, "✂️"),
                IssueType::MissingValue => (egui::Color32::YELLOW, "❓"),
                IssueType::LowConfidence => (egui::Color32::LIGHT_BLUE, "🤔"),
                IssueType::StructuralError => (egui::Color32::DARK_RED, "💥"),
            };
            
            ui.colored_label(color, icon);
            
            let description = match issue.issue_type {
                IssueType::Truncated => "Text appears truncated",
                IssueType::MissingValue => "Missing or empty value",
                IssueType::LowConfidence => "Low OCR confidence",
                IssueType::StructuralError => "Structural parsing error",
            };
            
            ui.label(format!("{} in {}", description, issue.block_id));
            
            if let (Some(line), Some(col)) = (issue.location.line, issue.location.column) {
                ui.label(format!("(Row {}, Col {})", line + 1, col + 1));
            }
        });
    }
    
    #[cfg(feature = "gui")]
    fn render_collapsible_block(&mut self, ui: &mut Ui, block: &mut ContentBlock) {
        let block_id = self.get_block_id(block);
        let is_collapsed = self.collapsed_blocks.get(&block_id).copied().unwrap_or(false);
        
        let header_text = match block {
            ContentBlock::Table(table) => format!("📊 Table ({} rows)", table.data.len()),
            ContentBlock::List(list) => format!("📋 List ({} items)", list.items.len()),
            ContentBlock::Text(_) => "📄 Text Block".to_string(),
            ContentBlock::Form(form) => format!("📝 Form ({} fields)", form.fields.len()),
            ContentBlock::Unknown(_) => "❓ Unknown Content".to_string(),
        };
        
        egui::CollapsingHeader::new(header_text)
            .default_open(!is_collapsed)
            .show(ui, |ui| {
                self.render_content_block(ui, block);
            });
    }
    
    #[cfg(feature = "gui")]
    fn render_content_block(&mut self, ui: &mut Ui, block: &mut ContentBlock) {
        match block {
            ContentBlock::Table(table) => self.render_table_editor(ui, table),
            ContentBlock::List(list) => self.render_list_editor(ui, list),
            ContentBlock::Text(text) => self.render_text_editor(ui, text),
            ContentBlock::Form(form) => self.render_form_editor(ui, form),
            ContentBlock::Unknown(raw) => self.render_raw_editor(ui, raw),
        }
    }
    
    #[cfg(feature = "gui")]
    fn render_content_block_with_issues(&mut self, ui: &mut Ui, block: &mut ContentBlock) {
        // Same as render_content_block but with issue highlighting
        self.render_content_block(ui, block);
    }
    
    #[cfg(feature = "gui")]
    fn render_table_editor(&mut self, ui: &mut Ui, table: &mut TableBlock) {
        if table.data.is_empty() {
            ui.label("Empty table");
            return;
        }
        
        ui.heading(format!("📊 Table: {}", table.id));
        
        // Table with editable cells
        egui::Grid::new(&table.id)
            .striped(true)
            .show(ui, |ui| {
                for (row_idx, row) in table.data.iter_mut().enumerate() {
                    for (col_idx, cell) in row.iter_mut().enumerate() {
                        let cell_id = format!("{}_{}_{}", table.id, row_idx, col_idx);
                        
                        // Check if this cell has issues
                        let has_issue = self.issues.iter().any(|issue| {
                            issue.block_id == table.id && 
                            issue.location.line == Some(row_idx) &&
                            issue.location.column == Some(col_idx)
                        });
                        
                        // Style the cell based on issues
                        let cell_response = if has_issue {
                            let frame = egui::Frame::default()
                                .fill(egui::Color32::from_rgba_unmultiplied(255, 200, 200, 100))
                                .stroke(egui::Stroke::new(2.0, egui::Color32::RED));
                            
                            frame.show(ui, |ui| {
                                ui.add(egui::TextEdit::singleline(cell).id(egui::Id::new(cell_id)))
                            })
                        } else {
                            egui::Frame::default().show(ui, |ui| {
                                ui.add(egui::TextEdit::singleline(cell).id(egui::Id::new(cell_id)))
                            })
                        };
                        
                        // Show issue tooltip on hover
                        if has_issue {
                            cell_response.response.on_hover_text("This cell has validation issues");
                        }
                    }
                    ui.end_row();
                }
            });
    }
    
    #[cfg(feature = "gui")]
    fn render_list_editor(&mut self, ui: &mut Ui, list: &mut ListBlock) {
        ui.heading(format!("📋 List: {}", list.id));
        
        for (idx, item) in list.items.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.label("•");
                
                // Check if this item has issues
                let has_issue = self.issues.iter().any(|issue| {
                    issue.block_id == list.id && issue.location.line == Some(idx)
                });
                
                let item_id = format!("{}_{}", list.id, idx);
                
                if has_issue {
                    let frame = egui::Frame::default()
                        .fill(egui::Color32::from_rgba_unmultiplied(255, 200, 200, 100))
                        .stroke(egui::Stroke::new(1.0, egui::Color32::RED));
                    
                    frame.show(ui, |ui| {
                        ui.add_sized([ui.available_width(), 20.0], 
                            egui::TextEdit::singleline(item).id(egui::Id::new(item_id)))
                    }).response.on_hover_text("This item has validation issues");
                } else {
                    ui.add_sized([ui.available_width(), 20.0], 
                        egui::TextEdit::singleline(item).id(egui::Id::new(item_id)));
                }
            });
        }
        
        // Add new item button
        if ui.button("➕ Add Item").clicked() {
            list.items.push(String::new());
        }
    }
    
    #[cfg(feature = "gui")]
    fn render_text_editor(&mut self, ui: &mut Ui, text: &mut TextBlock) {
        ui.heading(format!("📄 Text: {}", text.id));
        
        // Check if this text block has issues
        let has_issue = self.issues.iter().any(|issue| issue.block_id == text.id);
        
        if has_issue {
            let frame = egui::Frame::default()
                .fill(egui::Color32::from_rgba_unmultiplied(255, 200, 200, 100))
                .stroke(egui::Stroke::new(1.0, egui::Color32::RED));
            
            frame.show(ui, |ui| {
                ui.add_sized([ui.available_width(), 100.0], 
                    egui::TextEdit::multiline(&mut text.text)
                        .id(egui::Id::new(&text.id)))
            }).response.on_hover_text("This text has validation issues");
        } else {
            ui.add_sized([ui.available_width(), 100.0], 
                egui::TextEdit::multiline(&mut text.text)
                    .id(egui::Id::new(&text.id)));
        }
    }
    
    #[cfg(feature = "gui")]
    fn render_form_editor(&mut self, ui: &mut Ui, form: &mut FormBlock) {
        ui.heading(format!("📝 Form: {}", form.id));
        
        let field_keys: Vec<String> = form.fields.keys().cloned().collect();
        
        for field_name in field_keys {
            ui.horizontal(|ui| {
                ui.label(format!("{}:", field_name));
                if let Some(field_value) = form.fields.get_mut(&field_name) {
                    ui.add(egui::TextEdit::singleline(field_value));
                }
            });
        }
    }
    
    #[cfg(feature = "gui")]
    fn render_raw_editor(&mut self, ui: &mut Ui, raw: &mut RawBlock) {
        ui.heading(format!("❓ Unknown Content: {}", raw.id));
        ui.add_sized([ui.available_width(), 100.0], 
            egui::TextEdit::multiline(&mut raw.raw_content)
                .id(egui::Id::new(&raw.id)));
    }
    
    fn get_block_id(&self, block: &ContentBlock) -> String {
        match block {
            ContentBlock::Table(table) => table.id.clone(),
            ContentBlock::List(list) => list.id.clone(),
            ContentBlock::Text(text) => text.id.clone(),
            ContentBlock::Form(form) => form.id.clone(),
            ContentBlock::Unknown(raw) => raw.id.clone(),
        }
    }
    
    pub fn add_sample_content(&mut self) {
        // Add sample table with issues for demonstration
        let table_data = vec![
            vec!["Name".to_string(), "Department".to_string(), "Salary".to_string()],
            vec!["John Doe".to_string(), "Engineeri...".to_string(), "75000".to_string()], // Truncated
            vec!["Jane Smith".to_string(), "".to_string(), "82000".to_string()], // Missing value
            vec!["Bob Johnson".to_string(), "Marketing".to_string(), "68000".to_string()],
        ];
        
        self.content_blocks.push(ContentBlock::Table(TableBlock {
            id: "sample_table".to_string(),
            data: table_data,
        }));
        
        // Add sample text with issues
        self.content_blocks.push(ContentBlock::Text(TextBlock {
            id: "sample_text".to_string(),
            text: "This is a sample text block that demonstrates truncated content like this incomplte...".to_string(),
        }));
        
        // Add sample list
        self.content_blocks.push(ContentBlock::List(ListBlock {
            id: "sample_list".to_string(),
            items: vec![
                "First list item".to_string(),
                "Second incomplete ite...".to_string(), // Truncated
                "Third item".to_string(),
            ],
        }));
        
        // Detect issues in the sample content
        self.detect_validation_issues();
    }
    
    // Static rendering methods to avoid borrow checker issues
    #[cfg(feature = "gui")]
    fn render_issue_card_static(ui: &mut Ui, issue: &ValidationIssue) {
        ui.horizontal(|ui| {
            let (color, icon) = match issue.issue_type {
                IssueType::Truncated => (egui::Color32::RED, "✂️"),
                IssueType::MissingValue => (egui::Color32::YELLOW, "❓"),
                IssueType::LowConfidence => (egui::Color32::LIGHT_BLUE, "🤔"),
                IssueType::StructuralError => (egui::Color32::DARK_RED, "💥"),
            };
            
            ui.colored_label(color, icon);
            
            let description = match issue.issue_type {
                IssueType::Truncated => "Text appears truncated",
                IssueType::MissingValue => "Missing or empty value",
                IssueType::LowConfidence => "Low OCR confidence",
                IssueType::StructuralError => "Structural parsing error",
            };
            
            ui.label(format!("{} in {}", description, issue.block_id));
            
            if let (Some(line), Some(col)) = (issue.location.line, issue.location.column) {
                ui.label(format!("(Row {}, Col {})", line + 1, col + 1));
            }
        });
    }
    
    #[cfg(feature = "gui")]
    fn render_table_editor_static(ui: &mut Ui, table: &mut TableBlock, issues: &[ValidationIssue]) {
        if table.data.is_empty() {
            ui.label("Empty table");
            return;
        }
        
        ui.heading(format!("📊 Table: {}", table.id));
        
        // Table with editable cells
        egui::Grid::new(&table.id)
            .striped(true)
            .show(ui, |ui| {
                for (row_idx, row) in table.data.iter_mut().enumerate() {
                    for (col_idx, cell) in row.iter_mut().enumerate() {
                        let cell_id = format!("{}_{}_{}", table.id, row_idx, col_idx);
                        
                        // Check if this cell has issues
                        let has_issue = issues.iter().any(|issue| {
                            issue.block_id == table.id && 
                            issue.location.line == Some(row_idx) &&
                            issue.location.column == Some(col_idx)
                        });
                        
                        // Style the cell based on issues
                        let cell_response = if has_issue {
                            let frame = egui::Frame::default()
                                .fill(egui::Color32::from_rgba_unmultiplied(255, 200, 200, 100))
                                .stroke(egui::Stroke::new(2.0, egui::Color32::RED));
                            
                            frame.show(ui, |ui| {
                                ui.add(egui::TextEdit::singleline(cell).id(egui::Id::new(cell_id)))
                            })
                        } else {
                            egui::Frame::default().show(ui, |ui| {
                                ui.add(egui::TextEdit::singleline(cell).id(egui::Id::new(cell_id)))
                            })
                        };
                        
                        // Show issue tooltip on hover
                        if has_issue {
                            cell_response.response.on_hover_text("This cell has validation issues");
                        }
                    }
                    ui.end_row();
                }
            });
    }
    
    #[cfg(feature = "gui")]
    fn render_list_editor_static(ui: &mut Ui, list: &mut ListBlock, issues: &[ValidationIssue]) {
        ui.heading(format!("📋 List: {}", list.id));
        
        for (idx, item) in list.items.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.label("•");
                
                // Check if this item has issues
                let has_issue = issues.iter().any(|issue| {
                    issue.block_id == list.id && issue.location.line == Some(idx)
                });
                
                let item_id = format!("{}_{}", list.id, idx);
                
                if has_issue {
                    let frame = egui::Frame::default()
                        .fill(egui::Color32::from_rgba_unmultiplied(255, 200, 200, 100))
                        .stroke(egui::Stroke::new(1.0, egui::Color32::RED));
                    
                    frame.show(ui, |ui| {
                        ui.add_sized([ui.available_width(), 20.0], 
                            egui::TextEdit::singleline(item).id(egui::Id::new(item_id)))
                    }).response.on_hover_text("This item has validation issues");
                } else {
                    ui.add_sized([ui.available_width(), 20.0], 
                        egui::TextEdit::singleline(item).id(egui::Id::new(item_id)));
                }
            });
        }
        
        // Add new item button
        if ui.button("➕ Add Item").clicked() {
            list.items.push(String::new());
        }
    }
    
    #[cfg(feature = "gui")]
    fn render_text_editor_static(ui: &mut Ui, text: &mut TextBlock, issues: &[ValidationIssue]) {
        ui.heading(format!("📄 Text: {}", text.id));
        
        // Check if this text block has issues
        let has_issue = issues.iter().any(|issue| issue.block_id == text.id);
        
        if has_issue {
            let frame = egui::Frame::default()
                .fill(egui::Color32::from_rgba_unmultiplied(255, 200, 200, 100))
                .stroke(egui::Stroke::new(1.0, egui::Color32::RED));
            
            frame.show(ui, |ui| {
                ui.add_sized([ui.available_width(), 100.0], 
                    egui::TextEdit::multiline(&mut text.text)
                        .id(egui::Id::new(&text.id)))
            }).response.on_hover_text("This text has validation issues");
        } else {
            ui.add_sized([ui.available_width(), 100.0], 
                egui::TextEdit::multiline(&mut text.text)
                    .id(egui::Id::new(&text.id)));
        }
    }
    
    #[cfg(feature = "gui")]
    fn render_form_editor_static(ui: &mut Ui, form: &mut FormBlock) {
        ui.heading(format!("📝 Form: {}", form.id));
        
        let field_keys: Vec<String> = form.fields.keys().cloned().collect();
        
        for field_name in field_keys {
            ui.horizontal(|ui| {
                ui.label(format!("{}:", field_name));
                if let Some(field_value) = form.fields.get_mut(&field_name) {
                    ui.add(egui::TextEdit::singleline(field_value));
                }
            });
        }
    }
    
    #[cfg(feature = "gui")]
    fn render_raw_editor_static(ui: &mut Ui, raw: &mut RawBlock) {
        ui.heading(format!("❓ Unknown Content: {}", raw.id));
        ui.add_sized([ui.available_width(), 100.0], 
            egui::TextEdit::multiline(&mut raw.raw_content)
                .id(egui::Id::new(&raw.id)));
    }
}

impl Default for ValidationPane {
    fn default() -> Self {
        Self::new()
    }
}
