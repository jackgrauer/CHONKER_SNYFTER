// Minimal TUI Mockup for Chonker Document Processing
// Clean, minimal design with solid color blocks
//
// To run: cargo run --bin tui_mockup
// 
// [dependencies]
// ratatui = "0.24"
// crossterm = "0.27"

use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Alignment},
    style::{Color, Style, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs, Gauge, Clear},
    Frame, Terminal,
};
use std::{error::Error, io, time::Instant};

#[derive(Debug, Clone, PartialEq)]
enum ProcessingStage {
    None,
    Extract,
    Process, 
    Analyze,
    AutoVerify,  // New: AI checking for suspicious data
    Export,
    Complete,
    Error(String),
}

#[derive(Debug, Clone, PartialEq)]
enum DocumentStatus {
    New,
    Processing(ProcessingStage),
    Processed,
    Error(String),
}

#[derive(Debug, Clone)]
struct Document {
    name: String,
    size: String,
    status: DocumentStatus,
    confidence: Option<u8>, // 0-100 extraction confidence
}

#[derive(Debug, Clone)]
struct DataChunk {
    id: usize,
    content: String,
    chunk_type: ChunkType,
    confidence: u8,
    verified: bool,
    flagged_for_review: bool,
    in_basket: bool,
}

#[derive(Debug, Clone, PartialEq)]
enum ChunkType {
    Header,
    Table,
    Contact,
    Measurement,
    Other,
}

#[derive(Debug, Clone)]
struct SuspiciousItem {
    chunk_id: usize,
    reason: String,
    suggestion: String,
}

#[derive(PartialEq)]
enum View {
    Files,
    Processing,
    Data,
}

#[derive(PartialEq)]  
enum EditMode {
    View,
    Edit,
    Explain,
}

#[derive(PartialEq)]
enum Focus {
    DocumentList,
    ProcessingPanel,
    OriginalView,
    ExtractedView,
    EditPanel,
    ExplainPanel,
}

struct App {
    view: View,
    focus: Focus,
    selected_doc: usize,
    documents: Vec<Document>,
    current_document: Option<Document>,
    edit_mode: EditMode,
    explanation: String,
    original_text: String,
    extracted_markdown: String,
    processing_progress: u16, // 0-100
    processing_stage: ProcessingStage,
    show_help: bool,
    last_update: Instant,
    
    // New features
    data_chunks: Vec<DataChunk>,
    suspicious_items: Vec<SuspiciousItem>,
    selected_chunk: usize,
    basket_count: usize,  // Number of verified chunks in basket
    show_verification: bool,  // Show auto-verification results
}

impl App {
    fn new() -> App {
        let documents = vec![
            Document {
                name: "environmental_report.pdf".to_string(),
                size: "2.3M".to_string(),
                status: DocumentStatus::Processed,
                confidence: Some(94),
            },
            Document {
                name: "water_analysis.docx".to_string(),
                size: "890K".to_string(),
                status: DocumentStatus::Processing(ProcessingStage::Analyze),
                confidence: None,
            },
            Document {
                name: "lab_results.xlsx".to_string(),
                size: "1.2M".to_string(),
                status: DocumentStatus::New,
                confidence: None,
            },
            Document {
                name: "field_notes.pdf".to_string(),
                size: "567K".to_string(),
                status: DocumentStatus::Error("OCR failed: corrupted file".to_string()),
                confidence: None,
            },
        ];

        let data_chunks = vec![
            DataChunk {
                id: 0,
                content: "# Environmental Monitoring Report".to_string(),
                chunk_type: ChunkType::Header,
                confidence: 98,
                verified: true,
                flagged_for_review: false,
                in_basket: true,
            },
            DataChunk {
                id: 1,
                content: "Station B: pH 6.8 ⚠️ Warning - Below threshold".to_string(),
                chunk_type: ChunkType::Measurement,
                confidence: 85,
                verified: false,
                flagged_for_review: true,
                in_basket: false,
            },
            DataChunk {
                id: 2,
                content: "s.chen@enviro.gov".to_string(),
                chunk_type: ChunkType::Contact,
                confidence: 92,
                verified: true,
                flagged_for_review: false,
                in_basket: true,
            },
            DataChunk {
                id: 3,
                content: "Phosphates: 0.8 mg/L (Elevated)".to_string(),
                chunk_type: ChunkType::Measurement,
                confidence: 76,
                verified: false,
                flagged_for_review: true,
                in_basket: false,
            },
        ];

        let suspicious_items = vec![
            SuspiciousItem {
                chunk_id: 1,
                reason: "Low confidence on pH measurement".to_string(),
                suggestion: "OCR may have confused decimal point. Check if 6.8 should be 6.9".to_string(),
            },
            SuspiciousItem {
                chunk_id: 3,
                reason: "Chemical measurement formatting inconsistent".to_string(),
                suggestion: "Units format differs from other measurements. Verify mg/L is correct".to_string(),
            },
        ];

        App {
            view: View::Files,
            focus: Focus::DocumentList,
            selected_doc: 0,
            documents,
            current_document: None,
            edit_mode: EditMode::View,
            explanation: String::new(),
            original_text: "ENVIRONMENTAL MONITORING REPORT\n\nDate: 2024-01-15\nLocation: Industrial Site Alpha\n\nWATER QUALITY MEASUREMENTS\n\nStation A: pH 7.2 ✓ Normal\nStation B: pH 6.8 ⚠️ Warning - Below threshold\nStation C: pH 7.1 ✓ Normal\nStation D: pH 6.9 ⚠️ Monitoring required\n\nCHEMICAL ANALYSIS\nNitrates: 2.1 mg/L (Normal)\nPhosphates: 0.8 mg/L (Elevated)\nDissolved O2: 8.2 mg/L (Good)\n\nRECOMMENDations:\n- Investigate Station B drainage\n- Monitor phosphate sources\n- Retest in 48 hours\n\nCONTACT INFORMATION\nLead Analyst: Dr. Sarah Chen\nEmail: s.chen@enviro.gov\nPhone: (555) 123-4567\n\nReport ID: ENV-2024-001\nApproved by: Environmental Authority".to_string(),
            extracted_markdown: "# Environmental Monitoring Report\n\n**Date**: 2024-01-15  \n**Location**: Industrial Site Alpha  \n**Report ID**: ENV-2024-001\n\n## Water Quality Measurements\n\n| Station | pH Level | Status |\n|---------|----------|--------|\n| Station A | 7.2 | ✓ Normal |\n| Station B | 6.8 | ⚠️ Warning - Below threshold |\n| Station C | 7.1 | ✓ Normal |\n| Station D | 6.9 | ⚠️ Monitoring required |\n\n## Chemical Analysis\n\n- **Nitrates**: 2.1 mg/L (Normal)\n- **Phosphates**: 0.8 mg/L (Elevated) ⚠️\n- **Dissolved O2**: 8.2 mg/L (Good)\n\n## Recommendations\n\n1. Investigate Station B drainage\n2. Monitor phosphate sources  \n3. Retest in 48 hours\n\n## Contact Information\n\n**Lead Analyst**: Dr. Sarah Chen  \n**Email**: s.chen@enviro.gov  \n**Phone**: (555) 123-4567  \n\n---\n*Approved by: Environmental Authority*".to_string(),
            processing_progress: 0,
            processing_stage: ProcessingStage::None,
            show_help: false,
            last_update: Instant::now(),
            
            // New features
            data_chunks,
            suspicious_items,
            selected_chunk: 0,
            basket_count: 2,  // Header and email already in basket
            show_verification: false,
        }
    }

    fn handle_key(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        if modifiers.contains(KeyModifiers::CONTROL) {
            match key {
                KeyCode::Char('c') | KeyCode::Char('q') => std::process::exit(0),
                KeyCode::Char('h') => self.show_help = !self.show_help,
                _ => {}
            }
            return;
        }

        if self.show_help {
            self.show_help = false;
            return;
        }

        match key {
            // Global navigation
            KeyCode::Char('1') => {
                self.view = View::Files;
                self.focus = Focus::DocumentList;
            }
            KeyCode::Char('2') => {
                self.view = View::Processing;
                self.focus = Focus::ProcessingPanel;
            }
            KeyCode::Char('3') if self.current_document.is_some() => {
                self.view = View::Data;
                self.focus = Focus::OriginalView;
            }
            KeyCode::Tab => self.cycle_focus(),
            KeyCode::Char('?') => self.show_help = true,
            KeyCode::Esc => {
                if self.edit_mode != EditMode::View {
                    self.edit_mode = EditMode::View;
                } else {
                    self.view = View::Files;
                    self.focus = Focus::DocumentList;
                }
            }
            
            // View-specific handling
            _ => match self.view {
                View::Files => self.handle_files_key(key),
                View::Processing => self.handle_processing_key(key),
                View::Data => self.handle_data_key(key),
            }
        }
    }

    fn handle_files_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Up => {
                if self.selected_doc > 0 {
                    self.selected_doc -= 1;
                }
            }
            KeyCode::Down => {
                if self.selected_doc < self.documents.len() - 1 {
                    self.selected_doc += 1;
                }
            }
            KeyCode::Enter => {
                let doc = self.documents[self.selected_doc].clone();
                match doc.status {
                    DocumentStatus::New => {
                        // Start processing
                        self.documents[self.selected_doc].status = 
                            DocumentStatus::Processing(ProcessingStage::Extract);
                        self.processing_stage = ProcessingStage::Extract;
                        self.processing_progress = 0;
                        self.view = View::Processing;
                        self.focus = Focus::ProcessingPanel;
                    }
                    DocumentStatus::Processed => {
                        // Load document for viewing
                        self.current_document = Some(doc);
                        self.view = View::Data;
                        self.focus = Focus::OriginalView;
                    }
                    DocumentStatus::Processing(_) => {
                        // Show processing view
                        self.view = View::Processing;
                        self.focus = Focus::ProcessingPanel;
                    }
                    DocumentStatus::Error(_) => {
                        // Could show error details or retry options
                    }
                }
            }
            KeyCode::Char('r') => {
                // Retry failed document
                if let DocumentStatus::Error(_) = self.documents[self.selected_doc].status {
                    self.documents[self.selected_doc].status = DocumentStatus::New;
                }
            }
            KeyCode::Delete => {
                // Remove document from list
                if self.documents.len() > 1 {
                    self.documents.remove(self.selected_doc);
                    if self.selected_doc >= self.documents.len() {
                        self.selected_doc = self.documents.len() - 1;
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_processing_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char(' ') => {
                // Simulate processing progress
                if self.processing_progress < 100 {
                    self.processing_progress += 10;
                    self.processing_stage = match self.processing_progress {
                        0..=20 => ProcessingStage::Extract,
                        21..=40 => ProcessingStage::Process,
                        41..=60 => ProcessingStage::Analyze,
                        61..=80 => ProcessingStage::AutoVerify,
                        81..=99 => ProcessingStage::Export,
                        _ => ProcessingStage::Complete,
                    };
                    
                    // Show verification results when auto-verify completes
                    if self.processing_progress == 80 {
                        self.show_verification = true;
                    }
                    
                    if self.processing_progress >= 100 {
                        self.documents[self.selected_doc].status = DocumentStatus::Processed;
                        self.documents[self.selected_doc].confidence = Some(94);
                        self.current_document = Some(self.documents[self.selected_doc].clone());
                    }
                }
            }
            KeyCode::Char('c') => {
                // Cancel processing
                self.processing_stage = ProcessingStage::None;
                self.processing_progress = 0;
                self.documents[self.selected_doc].status = DocumentStatus::New;
                self.view = View::Files;
            }
            _ => {}
        }
    }

    fn handle_data_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('e') => {
                self.edit_mode = if self.edit_mode == EditMode::Edit {
                    EditMode::View
                } else {
                    EditMode::Edit
                };
                self.focus = if self.edit_mode == EditMode::Edit {
                    Focus::EditPanel
                } else {
                    Focus::ExtractedView
                };
            }
            KeyCode::Char('x') if self.edit_mode == EditMode::Edit => {
                self.edit_mode = if self.edit_mode == EditMode::Explain {
                    EditMode::Edit
                } else {
                    EditMode::Explain
                };
                self.focus = Focus::ExplainPanel;
            }
            KeyCode::Char('v') => {
                // Toggle verification overlay
                self.show_verification = !self.show_verification;
            }
            KeyCode::Char('b') => {
                // Add current chunk to basket (if any selected)
                if self.selected_chunk < self.data_chunks.len() {
                    let chunk = &mut self.data_chunks[self.selected_chunk];
                    if !chunk.in_basket {
                        chunk.in_basket = true;
                        chunk.verified = true;
                        self.basket_count += 1;
                    }
                }
            }
            KeyCode::Char('r') => {
                // Remove from basket
                if self.selected_chunk < self.data_chunks.len() {
                    let chunk = &mut self.data_chunks[self.selected_chunk];
                    if chunk.in_basket {
                        chunk.in_basket = false;
                        self.basket_count -= 1;
                    }
                }
            }
            KeyCode::Up if self.show_verification => {
                if self.selected_chunk > 0 {
                    self.selected_chunk -= 1;
                }
            }
            KeyCode::Down if self.show_verification => {
                if self.selected_chunk < self.data_chunks.len() - 1 {
                    self.selected_chunk += 1;
                }
            }
            KeyCode::Left => {
                if self.edit_mode == EditMode::View {
                    self.focus = Focus::OriginalView;
                }
            }
            KeyCode::Right => {
                if self.edit_mode == EditMode::View {
                    self.focus = Focus::ExtractedView;
                }
            }
            _ => {}
        }
    }

    fn cycle_focus(&mut self) {
        self.focus = match (&self.view, &self.focus) {
            (View::Files, Focus::DocumentList) => Focus::DocumentList,
            (View::Processing, Focus::ProcessingPanel) => Focus::ProcessingPanel,
            (View::Data, Focus::OriginalView) => Focus::ExtractedView,
            (View::Data, Focus::ExtractedView) => {
                if self.edit_mode != EditMode::View {
                    Focus::EditPanel
                } else {
                    Focus::OriginalView
                }
            }
            (View::Data, Focus::EditPanel) => {
                if self.edit_mode == EditMode::Explain {
                    Focus::ExplainPanel
                } else {
                    Focus::OriginalView
                }
            }
            (View::Data, Focus::ExplainPanel) => Focus::OriginalView,
            _ => Focus::DocumentList,
        };
    }
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(1)])
        .split(f.size());

    // Header
    render_header(f, chunks[0], app);
    
    // Main content
    match app.view {
        View::Files => render_files_view(f, chunks[1], app),
        View::Processing => render_processing_view(f, chunks[1], app),
        View::Data => render_data_view(f, chunks[1], app),
    }
    
    // Status bar
    render_status_bar(f, chunks[2], app);
    
    // Help overlay
    if app.show_help {
        render_help_overlay(f, app);
    }
    
    // Verification overlay
    if app.show_verification {
        render_verification_overlay(f, app);
    }
}

fn render_header(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(20), Constraint::Min(0)])
        .split(area);

    // Simple title
    let title = Paragraph::new("🐹 CHONKER")
        .style(Style::default().fg(Color::White));
    f.render_widget(title, header_chunks[0]);

    // Simple tabs
    let tab_titles = vec!["Files", "Process", "Data"];
    let selected_tab = match app.view {
        View::Files => 0,
        View::Processing => 1,
        View::Data => 2,
    };
    
    let tabs = Tabs::new(tab_titles)
        .select(selected_tab)
        .style(Style::default().fg(Color::Gray))
        .highlight_style(Style::default().fg(Color::White).bg(Color::DarkGray));
    f.render_widget(tabs, header_chunks[1]);
}

fn render_files_view(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(area);

    // Document list - clean solid block
    let items: Vec<ListItem> = app.documents
        .iter()
        .enumerate()
        .map(|(i, doc)| {
            let status_text = match &doc.status {
                DocumentStatus::New => "NEW",
                DocumentStatus::Processing(_) => "PROCESSING",
                DocumentStatus::Processed => "DONE",
                DocumentStatus::Error(_) => "ERROR",
            };
            
            let confidence_text = match doc.confidence {
                Some(conf) => format!(" {}%", conf),
                None => String::new(),
            };
            
            let line = format!("{} {} {}{}", 
                status_text, doc.name, doc.size, confidence_text);
            
            let style = if i == app.selected_doc {
                Style::default().bg(Color::Blue).fg(Color::White)
            } else {
                match &doc.status {
                    DocumentStatus::Error(_) => Style::default().fg(Color::Red).bg(Color::Black),
                    DocumentStatus::Processing(_) => Style::default().fg(Color::Yellow).bg(Color::Black),
                    DocumentStatus::Processed => Style::default().fg(Color::Green).bg(Color::Black),
                    DocumentStatus::New => Style::default().fg(Color::Gray).bg(Color::Black),
                }
            };
            
            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items)
        .style(Style::default().bg(Color::Black));
    f.render_widget(list, chunks[0]);

    // Details panel - clean solid block
    if let Some(doc) = app.documents.get(app.selected_doc) {
        let details = match &doc.status {
            DocumentStatus::New => "Ready to process\n\nPress Enter to start".to_string(),
            DocumentStatus::Processing(stage) => format!("Processing\nStage: {:?}", stage),
            DocumentStatus::Processed => format!("Complete\nConfidence: {}%\n\nPress Enter to view", doc.confidence.unwrap_or(0)),
            DocumentStatus::Error(err) => format!("Error\n\n{}\n\nPress r to retry", err),
        };
        
        let panel = Paragraph::new(details)
            .style(Style::default().fg(Color::White).bg(Color::DarkGray));
        f.render_widget(panel, chunks[1]);
    }
}

fn render_processing_view(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),
            Constraint::Length(6),
            Constraint::Min(0),
        ])
        .split(area);

    // Processing pipeline visualization
    let pipeline_text = match app.processing_stage {
        ProcessingStage::Extract => "EXTRACT → process → analyze → verify → export",
        ProcessingStage::Process => "extract → PROCESS → analyze → verify → export", 
        ProcessingStage::Analyze => "extract → process → ANALYZE → verify → export",
        ProcessingStage::AutoVerify => "extract → process → analyze → VERIFY → export",
        ProcessingStage::Export => "extract → process → analyze → verify → EXPORT",
        ProcessingStage::Complete => "extract → process → analyze → verify → export → ✓",
        _ => "extract → process → analyze → verify → export",
    };
    
    let pipeline = Paragraph::new(pipeline_text)
        .block(Block::default()
            .title("🔄 Processing Pipeline")
            .borders(Borders::ALL))
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center);
    f.render_widget(pipeline, chunks[0]);

    // Progress bar
    let progress = Gauge::default()
        .block(Block::default()
            .title("Progress")
            .borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Green))
        .percent(app.processing_progress)
        .label(format!("{}%", app.processing_progress));
    f.render_widget(progress, chunks[1]);

    // Processing log
    let log_text = format!(
        "Processing log:\n\n✓ Document loaded\n✓ PDF pages: 3\n→ Extracting text...\n→ Analyzing structure...\n\nPress SPACE to advance\nPress 'c' to cancel"
    );
    
    let log = Paragraph::new(log_text)
        .block(Block::default()
            .title("📝 Processing Log")
            .borders(Borders::ALL))
        .style(Style::default().fg(Color::White));
    f.render_widget(log, chunks[2]);
}

fn render_data_view(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Original document view
    let original = Paragraph::new(app.original_text.as_str())
        .block(Block::default()
            .title("📄 Original Document")
            .borders(Borders::ALL)
            .border_style(if app.focus == Focus::OriginalView {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::Gray)
            }))
        .style(Style::default().fg(Color::White).bg(Color::Blue))
        .wrap(ratatui::widgets::Wrap { trim: true });
    f.render_widget(original, chunks[0]);

    // Right panel - extracted data with controls
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(chunks[1]);

    // Control buttons
    let controls = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(10), Constraint::Length(12)])
        .split(right_chunks[0]);

    let title = Paragraph::new("📊 Extracted Data (Markdown)")
        .style(Style::default().fg(Color::Cyan));
    f.render_widget(title, controls[0]);

    let edit_btn = Paragraph::new(if app.edit_mode == EditMode::Edit { "View" } else { "Edit" })
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Black).bg(Color::Yellow));
    f.render_widget(edit_btn, controls[1]);

    if app.edit_mode == EditMode::Edit {
        let explain_btn = Paragraph::new("🧠 Explain")
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::White).bg(Color::Magenta));
        f.render_widget(explain_btn, controls[2]);
    }

    // Content area based on edit mode
    match app.edit_mode {
        EditMode::View => {
            render_markdown_view(f, right_chunks[1], app);
        }
        EditMode::Edit => {
            render_edit_mode(f, right_chunks[1], app);
        }
        EditMode::Explain => {
            render_edit_mode(f, right_chunks[1], app);
        }
    }
}

fn render_markdown_view(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let markdown_lines: Vec<Line> = app.extracted_markdown
        .lines()
        .map(|line| {
            let color = if line.starts_with('#') {
                Color::Cyan
            } else if line.starts_with("**") {
                Color::Yellow
            } else if line.contains("⚠️") {
                Color::Red
            } else if line.contains('@') {
                Color::Green
            } else if line.contains('|') {
                Color::Magenta
            } else {
                Color::White
            };
            Line::from(Span::styled(line, Style::default().fg(color)))
        })
        .collect();

    let markdown = Paragraph::new(markdown_lines)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(if app.focus == Focus::ExtractedView {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::Gray)
            }))
        .wrap(ratatui::widgets::Wrap { trim: true });
    f.render_widget(markdown, area);
}

fn render_edit_mode(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let edit_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            if app.edit_mode == EditMode::Explain { 
                Constraint::Length(8) 
            } else { 
                Constraint::Length(0) 
            },
            Constraint::Min(0)
        ])
        .split(area);

    // Explain panel (when in explain mode)
    if app.edit_mode == EditMode::Explain {
        let explain_text = if app.explanation.is_empty() {
            "🧠 Experimental: AI-Powered OCR Correction\n\nDescribe the extraction error in plain English:\n• \"OCR confused 6 with G in pH values\"\n• \"Missed decimal point in measurement 45.2\"\n• \"Table columns are misaligned\"\n\n[Auto-Fix will be available after description]"
        } else {
            &app.explanation
        };
        
        let explain_box = Paragraph::new(explain_text)
            .block(Block::default()
                .title("🧠 Explain OCR Issues")
                .borders(Borders::ALL)
                .border_style(if app.focus == Focus::ExplainPanel {
                    Style::default().fg(Color::Magenta)
                } else {
                    Style::default().fg(Color::Gray)
                }))
            .style(Style::default().fg(Color::Magenta).bg(Color::Black))
            .wrap(ratatui::widgets::Wrap { trim: true });
        f.render_widget(explain_box, edit_chunks[0]);
    }

    // Edit area
    let editor = Paragraph::new(app.extracted_markdown.as_str())
        .block(Block::default()
            .title("✏️ Edit Markdown")
            .borders(Borders::ALL)
            .border_style(if app.focus == Focus::EditPanel {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Gray)
            }))
        .style(Style::default().fg(Color::White).bg(Color::DarkGray))
        .wrap(ratatui::widgets::Wrap { trim: true });
    f.render_widget(editor, edit_chunks[1]);
}

fn render_status_bar(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let help_text = match app.view {
        View::Files => "↑↓: Navigate | Enter: Process/View | Del: Remove | r: Retry | ?: Help | Ctrl-Q: Quit",
        View::Processing => "Space: Advance | c: Cancel | ?: Help | Ctrl-Q: Quit",
        View::Data => "e: Edit | x: Explain | v: Verify | b: Basket | r: Remove | ←→: Navigate | ?: Help | Ctrl-Q: Quit",
    };
    
    let status = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Gray).bg(Color::Black));
    f.render_widget(status, area);
}

fn render_verification_overlay(f: &mut Frame, app: &App) {
    let area = f.size();
    let popup_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(15),
            Constraint::Percentage(70),
            Constraint::Percentage(15),
        ])
        .split(area)[1];
    
    let popup_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(10),
            Constraint::Percentage(80), 
            Constraint::Percentage(10),
        ])
        .split(popup_area)[1];

    f.render_widget(Clear, popup_area);
    
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(popup_area);
    
    // Data chunks list
    let chunk_items: Vec<ListItem> = app.data_chunks
        .iter()
        .enumerate()
        .map(|(i, chunk)| {
            let status_icon = if chunk.in_basket {
                "🗂️"  // In basket
            } else if chunk.flagged_for_review {
                "⚠️"  // Needs review
            } else if chunk.verified {
                "✅"  // Verified
            } else {
                "❓"  // Unverified
            };
            
            let line = format!("{} {} ({}%)", 
                status_icon, 
                chunk.content.chars().take(35).collect::<String>(),
                chunk.confidence
            );
            
            let style = if i == app.selected_chunk {
                Style::default().bg(Color::Yellow).fg(Color::Black)
            } else if chunk.flagged_for_review {
                Style::default().fg(Color::Red)
            } else if chunk.in_basket {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::White)
            };
            
            ListItem::new(line).style(style)
        })
        .collect();

    let chunks_list = List::new(chunk_items)
        .block(Block::default()
            .title(format!("🔍 Auto-Verification Results (Basket: {})", app.basket_count))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)))
        .style(Style::default().bg(Color::Black));
    f.render_widget(chunks_list, chunks[0]);
    
    // Action panel
    let selected_chunk = app.data_chunks.get(app.selected_chunk);
    let action_text = if let Some(chunk) = selected_chunk {
        let suspicious_item = app.suspicious_items
            .iter()
            .find(|item| item.chunk_id == chunk.id);
            
        if let Some(item) = suspicious_item {
            format!(
                "🔍 FLAGGED FOR REVIEW\n\nContent: {}\n\nReason: {}\n\nSuggestion: {}\n\nActions:\n• 'b' - Add to basket (verify)\n• 'r' - Remove from basket\n• 'v' - Close verification\n\nConfidence: {}%",
                chunk.content,
                item.reason,
                item.suggestion,
                chunk.confidence
            )
        } else {
            format!(
                "✅ CHUNK DETAILS\n\nContent: {}\n\nType: {:?}\nConfidence: {}%\nVerified: {}\nIn Basket: {}\n\nActions:\n• 'b' - Add to basket\n• 'r' - Remove from basket\n• 'v' - Close verification",
                chunk.content,
                chunk.chunk_type,
                chunk.confidence,
                if chunk.verified { "Yes" } else { "No" },
                if chunk.in_basket { "Yes" } else { "No" }
            )
        }
    } else {
        "Select a chunk to see details".to_string()
    };
    
    let action_panel = Paragraph::new(action_text)
        .block(Block::default()
            .title("📋 Chunk Details")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow)))
        .style(Style::default().fg(Color::White).bg(Color::Black))
        .wrap(ratatui::widgets::Wrap { trim: true });
    f.render_widget(action_panel, chunks[1]);
}

fn render_help_overlay(f: &mut Frame, app: &App) {
    let area = f.size();
    let popup_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(60),
            Constraint::Percentage(20),
        ])
        .split(area)[1];
    
    let popup_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(60), 
            Constraint::Percentage(20),
        ])
        .split(popup_area)[1];

    f.render_widget(Clear, popup_area);
    
    let help_text = "🐹 CHONKER Document Intelligence - Help\n\n\
        GLOBAL SHORTCUTS:\n\
        • 1, 2, 3    : Switch between views\n\
        • Tab        : Cycle focus within view\n\
        • Esc        : Go back / Exit edit mode\n\
        • Ctrl-H     : Toggle this help\n\
        • Ctrl-Q     : Quit application\n\n\
        FILES VIEW:\n\
        • ↑↓         : Navigate document list\n\
        • Enter      : Process new / View processed\n\
        • Delete     : Remove from list\n\
        • r          : Retry failed document\n\n\
        DATA VIEW:\n\
        • e          : Toggle edit mode\n\
        • x          : Explain mode (experimental)\n\
        • ←→         : Navigate between panels\n\n\
        PROCESSING:\n\
        • Space      : Advance simulation\n\
        • c          : Cancel processing\n\n\
        Press any key to close this help.";
    
    let help = Paragraph::new(help_text)
        .block(Block::default()
            .title("Help")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)))
        .style(Style::default().fg(Color::White).bg(Color::Black))
        .wrap(ratatui::widgets::Wrap { trim: true });
    f.render_widget(help, popup_area);
}

fn main() -> Result<(), Box<dyn Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui(f, &app))?;

        // Handle events
        let timeout = std::time::Duration::from_millis(50);
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == crossterm::event::KeyEventKind::Press {
                    app.handle_key(key.code, key.modifiers);
                }
            }
        }

        // Simulate processing progress
        if app.processing_progress > 0 && app.processing_progress < 100 && 
           last_tick.elapsed() > std::time::Duration::from_millis(500) {
            app.processing_progress = (app.processing_progress + 1).min(100);
            if app.processing_progress >= 100 {
                app.processing_stage = ProcessingStage::Complete;
                if app.selected_doc < app.documents.len() {
                    app.documents[app.selected_doc].status = DocumentStatus::Processed;
                    app.documents[app.selected_doc].confidence = Some(94);
                    app.current_document = Some(app.documents[app.selected_doc].clone());
                }
            }
            last_tick = Instant::now();
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
