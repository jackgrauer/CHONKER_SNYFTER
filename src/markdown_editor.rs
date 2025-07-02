#[cfg(feature = "gui")]
use eframe::egui;
use std::collections::HashMap;

/// Markdown Editor component for the right pane
/// Handles markdown editing, preview, and synchronization with PDF
pub struct MarkdownEditor {
    pub content: String,
    pub preview_mode: bool,
    pub selection_highlights: HashMap<usize, HighlightRange>,
    pub edit_history: Vec<EditOperation>,
    pub current_edit: usize,
    pub syntax_highlighting: bool,
}

#[derive(Debug, Clone)]
pub struct HighlightRange {
    pub start: usize,
    pub end: usize,
    pub source_page: usize,
    pub source_coords: (f32, f32, f32, f32),
}

#[derive(Debug, Clone)]
pub struct EditOperation {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub operation_type: EditType,
    pub before: String,
    pub after: String,
    pub position: usize,
}

#[derive(Debug, Clone)]
pub enum EditType {
    Insert,
    Replace,
}

impl Default for MarkdownEditor {
    fn default() -> Self {
        Self {
            content: String::new(),
            preview_mode: false,
            selection_highlights: HashMap::new(),
            edit_history: Vec::new(),
            current_edit: 0,
            syntax_highlighting: true,
        }
    }
}

impl MarkdownEditor {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn set_content(&mut self, content: String) {
        self.record_edit(EditType::Replace, self.content.clone(), content.clone(), 0);
        self.content = content;
    }
    
    pub fn render(&mut self, ui: &mut egui::Ui) {
        // Simple editor without buttons - just the content
        self.render_editor(ui);
    }
    
    fn render_editor(&mut self, ui: &mut egui::Ui) {
        // TODO: Implement syntax-highlighted editor
        // For now, use basic text edit
        
        egui::ScrollArea::vertical()
            .max_height(ui.available_height())
            .show(ui, |ui| {
                ui.add_sized(
                    [ui.available_width(), ui.available_height()],
                    egui::TextEdit::multiline(&mut self.content)
                        .font(egui::TextStyle::Monospace)
                        .interactive(true)
                );
            });
    }
    
    fn render_preview(&mut self, ui: &mut egui::Ui) {
        // TODO: Implement markdown preview with egui_commonmark
        // For now, show raw content
        
        egui::ScrollArea::vertical()
            .max_height(ui.available_height())
            .show(ui, |ui| {
                ui.label(&self.content);
            });
    }
    
    pub fn highlight_range(&mut self, range_id: usize, start: usize, end: usize, source_page: usize, source_coords: (f32, f32, f32, f32)) {
        self.selection_highlights.insert(range_id, HighlightRange {
            start,
            end,
            source_page,
            source_coords,
        });
    }
    
    pub fn clear_highlights(&mut self) {
        self.selection_highlights.clear();
    }
    
    fn record_edit(&mut self, edit_type: EditType, before: String, after: String, position: usize) {
        let operation = EditOperation {
            timestamp: chrono::Utc::now(),
            operation_type: edit_type,
            before,
            after,
            position,
        };
        
        // Truncate history if we're not at the end
        self.edit_history.truncate(self.current_edit);
        
        self.edit_history.push(operation);
        self.current_edit = self.edit_history.len();
        
        // Limit history size
        if self.edit_history.len() > 100 {
            self.edit_history.remove(0);
            self.current_edit -= 1;
        }
    }
    
    fn can_undo(&self) -> bool {
        self.current_edit > 0
    }
    
    fn can_redo(&self) -> bool {
        self.current_edit < self.edit_history.len()
    }
    
    fn undo(&mut self) {
        if self.can_undo() {
            self.current_edit -= 1;
            let operation = &self.edit_history[self.current_edit];
            self.content = operation.before.clone();
        }
    }
    
    fn redo(&mut self) {
        if self.can_redo() {
            let operation = &self.edit_history[self.current_edit];
            self.content = operation.after.clone();
            self.current_edit += 1;
        }
    }
    
    pub fn insert_formatting(&mut self, prefix: &str, suffix: &str) {
        // Simple implementation - just append to content
        // TODO: Implement proper text selection and cursor positioning
        let before = self.content.clone();
        self.content.push_str(&format!("{}{}{}", prefix, "selected text", suffix));
        self.record_edit(EditType::Insert, before, self.content.clone(), self.content.len());
    }
    
    pub fn insert_list_item(&mut self) {
        let before = self.content.clone();
        self.content.push_str("\n- List item");
        self.record_edit(EditType::Insert, before, self.content.clone(), self.content.len());
    }
    
    pub fn insert_numbered_list(&mut self) {
        let before = self.content.clone();
        self.content.push_str("\n1. Numbered item");
        self.record_edit(EditType::Insert, before, self.content.clone(), self.content.len());
    }
    
    pub fn insert_header(&mut self) {
        let before = self.content.clone();
        self.content.push_str("\n# Header");
        self.record_edit(EditType::Insert, before, self.content.clone(), self.content.len());
    }
    
    pub fn insert_code_block(&mut self) {
        let before = self.content.clone();
        self.content.push_str("\n```\ncode block\n```");
        self.record_edit(EditType::Insert, before, self.content.clone(), self.content.len());
    }
}
