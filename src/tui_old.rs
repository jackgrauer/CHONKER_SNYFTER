use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, List, ListItem, ListState, Paragraph, Tabs,
    },
    Frame, Terminal,
};
use std::io::{self, Write};
use std::path::PathBuf;
use tracing::{info, error};
use crate::database::ChonkerDatabase;

#[derive(Debug, Clone, PartialEq)]
enum WorkflowStep {
    SelectPdf,     // Step 1: Select a PDF file
    ExtractText,   // Step 2: Extract text to markdown
    ProcessText,   // Step 3: Process and clean markdown
    ExportData,    // Step 4: Export to DataFrame format
    Complete,      // All done!
}

/// TUI Application state
struct App {
    should_quit: bool,
    current_tab: usize,
    database: ChonkerDatabase,
    documents: Vec<DocumentItem>,
    #[allow(dead_code)]
    selected_document: Option<usize>,
    list_state: ListState,
    status_message: String,
    processing_log: Vec<String>,
    available_files: Vec<PathBuf>,
    file_picker_open: bool,
    file_list_state: ListState,
    // Oregon Trail-style workflow state
    workflow_step: WorkflowStep,
    current_pdf: Option<PathBuf>,
    current_markdown: Option<PathBuf>,
    workflow_messages: Vec<String>,
}

#[derive(Clone)]
struct DocumentItem {
    id: String,
    filename: String,
    created_at: String,
    chunk_count: usize,
}

impl App {
    fn new(database: ChonkerDatabase) -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        let mut file_list_state = ListState::default();
        file_list_state.select(Some(0));
        
        Self {
            should_quit: false,
            current_tab: 0,
            database,
            documents: Vec::new(),
            selected_document: None,
            list_state,
            status_message: "🎉 Welcome to CHONKER TUI! Your PDF processing adventure begins...".to_string(),
            processing_log: Vec::new(),
            available_files: Vec::new(),
            file_picker_open: false,
            file_list_state,
            workflow_step: WorkflowStep::SelectPdf,
            current_pdf: None,
            current_markdown: None,
            workflow_messages: vec![
                "🏔️  Welcome to CHONKER! Let's begin your document processing journey.".to_string(),
                "📄 Step 1: Choose a PDF file to extract text from".to_string(),
            ],
        }
    }
    
    async fn load_documents(&mut self) -> Result<()> {
        let docs = self.database.get_all_documents().await?;
        self.documents = docs.into_iter().map(|doc| DocumentItem {
            id: doc.id,
            filename: doc.filename,
            created_at: doc.created_at,
            chunk_count: 0, // We'll load this separately if needed
        }).collect();
        
        if !self.documents.is_empty() && self.list_state.selected().is_none() {
            self.list_state.select(Some(0));
        }
        
        Ok(())
    }
    
    fn next_tab(&mut self) {
        self.current_tab = (self.current_tab + 1) % 4;
    }
    
    fn previous_tab(&mut self) {
        if self.current_tab > 0 {
            self.current_tab -= 1;
        } else {
            self.current_tab = 3;
        }
    }
    
    fn next_document(&mut self) {
        if !self.documents.is_empty() {
            let i = match self.list_state.selected() {
                Some(i) => {
                    if i >= self.documents.len() - 1 {
                        0
                    } else {
                        i + 1
                    }
                }
                None => 0,
            };
            self.list_state.select(Some(i));
        }
    }
    
    fn previous_document(&mut self) {
        if !self.documents.is_empty() {
            let i = match self.list_state.selected() {
                Some(i) => {
                    if i == 0 {
                        self.documents.len() - 1
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            self.list_state.select(Some(i));
        }
    }
    
    fn next_file(&mut self) {
        if !self.available_files.is_empty() {
            let i = match self.file_list_state.selected() {
                Some(i) => {
                    if i >= self.available_files.len() - 1 {
                        0
                    } else {
                        i + 1
                    }
                }
                None => 0,
            };
            self.file_list_state.select(Some(i));
        }
    }
    
    fn previous_file(&mut self) {
        if !self.available_files.is_empty() {
            let i = match self.file_list_state.selected() {
                Some(i) => {
                    if i == 0 {
                        self.available_files.len() - 1
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            self.file_list_state.select(Some(i));
        }
    }
    
    async fn delete_selected_document(&mut self) -> Result<()> {
        if let Some(selected) = self.list_state.selected() {
            if selected < self.documents.len() {
                let doc = &self.documents[selected];
                self.database.delete_document(&doc.id).await?;
                self.status_message = format!("Deleted document: {}", doc.filename);
                self.load_documents().await?;
                
                // Adjust selection
                if self.documents.is_empty() {
                    self.list_state.select(None);
                } else if selected >= self.documents.len() {
                    self.list_state.select(Some(self.documents.len() - 1));
                }
            }
        }
        Ok(())
    }
    
    async fn load_available_files(&mut self) -> Result<()> {
        // Look for PDF files in current directory and common locations
        let mut files = Vec::new();
        
        // Current directory
        if let Ok(entries) = std::fs::read_dir(".") {
            for entry in entries.flatten() {
                if let Some(path) = entry.path().to_str() {
                    if path.to_lowercase().ends_with(".pdf") {
                        files.push(entry.path());
                    }
                }
            }
        }
        
        // Test fixtures
        if let Ok(entries) = std::fs::read_dir("tests/fixtures") {
            for entry in entries.flatten() {
                if let Some(path) = entry.path().to_str() {
                    if path.to_lowercase().ends_with(".pdf") {
                        files.push(entry.path());
                    }
                }
            }
        }
        
        self.available_files = files;
        if !self.available_files.is_empty() && self.file_list_state.selected().is_none() {
            self.file_list_state.select(Some(0));
        }
        
        Ok(())
    }
    
    async fn process_selected_file(&mut self) -> Result<()> {
        if let Some(selected) = self.file_list_state.selected() {
            if selected < self.available_files.len() {
                let file_path = self.available_files[selected].clone();
                let filename = file_path.file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("unknown.pdf")
                    .to_string();
                
                self.status_message = format!("🚀 Starting extraction of {}...", filename);
                self.file_picker_open = false;
                self.current_pdf = Some(file_path.clone());
                
                // Add helpful info about long processing times
                self.processing_log.push(format!("🚀 Starting extraction: {}", filename));
                self.processing_log.push("ℹ️  Complex PDFs may take 1-2 minutes to process".to_string());
                self.processing_log.push("⏳ The app may appear frozen - this is normal!".to_string());
                
                // Call the actual extract command with progress updates
                match self.extract_pdf_to_markdown_with_progress(&file_path, &filename).await {
                    Ok(markdown_path) => {
                        self.current_markdown = Some(markdown_path.clone());
                        self.workflow_step = WorkflowStep::ProcessText;
                        self.update_workflow_messages();
                        self.status_message = format!("✅ Extracted {} to markdown!", filename);
                        self.processing_log.push(format!("✅ Extraction complete: {} -> {}", filename, markdown_path.display()));
                    },
                    Err(e) => {
                        self.status_message = format!("❌ Failed to extract {}: {}", filename, e);
                        self.processing_log.push(format!("❌ Error extracting {}: {}", filename, e));
                    }
                }
            }
        }
        Ok(())
    }
    
    async fn extract_pdf_to_markdown(&mut self, pdf_path: &PathBuf) -> Result<PathBuf> {
        let filename = pdf_path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("document");
        let markdown_path = PathBuf::from(format!("{}.md", filename));
        
        // Call the chonker CLI as a subprocess
        let output = tokio::process::Command::new("./target/release/chonker")
            .arg("extract")
            .arg(pdf_path.as_os_str())
            .arg("--output")
            .arg(markdown_path.as_os_str())
            .arg("--store")
            .output()
            .await?;
        
        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Extract command failed: {}", error_msg));
        }
        
        Ok(markdown_path)
    }
    
    async fn extract_pdf_to_markdown_with_progress(&mut self, pdf_path: &PathBuf, filename: &str) -> Result<PathBuf> {
        let markdown_filename = pdf_path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("document");
        let markdown_path = PathBuf::from(format!("{}.md", markdown_filename));
        
        let start_time = std::time::Instant::now();
        
        // Update status to show we're starting
        self.status_message = format!("🔄 Analyzing {} complexity...", filename);
        
        // Spawn the command with timeout and progress tracking
        let pdf_path_clone = pdf_path.clone();
        let markdown_path_clone = markdown_path.clone();
        
        let mut task = tokio::spawn(async move {
            tokio::process::Command::new("./target/release/chonker")
                .arg("extract")
                .arg(pdf_path_clone.as_os_str())
                .arg("--output")
                .arg(markdown_path_clone.as_os_str())
                .arg("--store")
                .output()
                .await
        });
        
        // Show progress updates while waiting
        let mut progress_counter = 0;
        let progress_messages = vec![
            "🔍 Analyzing document structure...",
            "🧠 Applying ML extraction...",
            "📝 Processing text content...",
            "🔧 Cleaning and formatting...",
            "💾 Saving results..."
        ];
        
        loop {
            let sleep_future = tokio::time::sleep(tokio::time::Duration::from_secs(3));
            let task_future = &mut task;
            
            tokio::select! {
                _ = sleep_future => {
                    let elapsed = start_time.elapsed().as_secs();
                    let message_idx = (progress_counter % progress_messages.len());
                    self.status_message = format!(
                        "{} ({}s elapsed)", 
                        progress_messages[message_idx], 
                        elapsed
                    );
                    progress_counter += 1;
                    
                    // Add periodic log updates
                    if elapsed > 0 && elapsed % 15 == 0 {
                        self.processing_log.push(format!("⏱️  Still processing... {}s elapsed", elapsed));
                    }
                }
                result = task_future => {
                    let output = result??;
                    let elapsed = start_time.elapsed();
                    
                    if !output.status.success() {
                        let error_msg = String::from_utf8_lossy(&output.stderr);
                        return Err(anyhow::anyhow!("Extract command failed: {}", error_msg));
                    }
                    
                    self.processing_log.push(format!("✅ Extraction completed in {:.1}s", elapsed.as_secs_f64()));
                    return Ok(markdown_path);
                }
            }
        }
    }
    
    async fn process_markdown(&mut self) -> Result<()> {
        if let Some(markdown_path) = &self.current_markdown {
            let processed_path = PathBuf::from(format!(
                "{}_processed.md",
                markdown_path.file_stem().unwrap().to_str().unwrap()
            ));
            
            self.status_message = "🔄 Processing and cleaning markdown...".to_string();
            
            // Call the chonker CLI as a subprocess
            let output = tokio::process::Command::new("./target/release/chonker")
                .arg("process")
                .arg(markdown_path.as_os_str())
                .arg("--output")
                .arg(processed_path.as_os_str())
                .arg("--correct")
                .output()
                .await?;
            
            if output.status.success() {
                self.current_markdown = Some(processed_path.clone());
                self.workflow_step = WorkflowStep::ExportData;
                self.update_workflow_messages();
                self.status_message = "✅ Markdown processed and cleaned!".to_string();
                self.processing_log.push(format!("✅ Processed: {}", processed_path.display()));
            } else {
                let error_msg = String::from_utf8_lossy(&output.stderr);
                self.status_message = format!("❌ Failed to process markdown: {}", error_msg);
                self.processing_log.push(format!("❌ Error processing: {}", error_msg));
            }
        }
        Ok(())
    }
    
    async fn export_to_csv(&mut self) -> Result<()> {
        let csv_path = PathBuf::from("chonker_export.csv");
        
        self.status_message = "🔄 Exporting data to CSV...".to_string();
        
        // Call the chonker CLI as a subprocess
        let output = tokio::process::Command::new("./target/release/chonker")
            .arg("export")
            .arg("--format")
            .arg("csv")
            .arg("--output")
            .arg(csv_path.as_os_str())
            .output()
            .await?;
        
        if output.status.success() {
            self.workflow_step = WorkflowStep::Complete;
            self.update_workflow_messages();
            self.status_message = "🎉 Export complete! Your data journey is finished!".to_string();
            self.processing_log.push(format!("✅ Exported to: {}", csv_path.display()));
        } else {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            self.status_message = format!("❌ Failed to export: {}", error_msg);
            self.processing_log.push(format!("❌ Export error: {}", error_msg));
        }
        Ok(())
    }
    
    fn update_workflow_messages(&mut self) {
        self.workflow_messages.clear();
        match self.workflow_step {
            WorkflowStep::SelectPdf => {
                self.workflow_messages = vec![
                    "🏔️  Welcome to CHONKER! Let's begin your document processing journey.".to_string(),
                    "📄 Step 1: Choose a PDF file to extract text from".to_string(),
                    "     Press 'f' to open file picker, then Enter to select".to_string(),
                ];
            },
            WorkflowStep::ExtractText => {
                self.workflow_messages = vec![
                    "🔄 Step 2: Extracting text from your PDF...".to_string(),
                    "     Using intelligent routing to choose the best extraction tool".to_string(),
                ];
            },
            WorkflowStep::ProcessText => {
                self.workflow_messages = vec![
                    "✅ Text extracted successfully!".to_string(),
                    "📝 Step 3: Process and clean the markdown text".to_string(),
                    "     Press 'p' to clean and correct the extracted text".to_string(),
                ];
            },
            WorkflowStep::ExportData => {
                self.workflow_messages = vec![
                    "✅ Text processed and cleaned!".to_string(),
                    "📊 Step 4: Export your data to a structured format".to_string(),
                    "     Press 'e' to export to CSV format".to_string(),
                ];
            },
            WorkflowStep::Complete => {
                self.workflow_messages = vec![
                    "🎉 Congratulations! Your document processing journey is complete!".to_string(),
                    "📄 You have successfully extracted, processed, and exported your PDF".to_string(),
                    "🔄 Press 'n' to start a new document, or 'q' to quit".to_string(),
                ];
            }
        }
    }
    
    fn start_new_workflow(&mut self) {
        self.workflow_step = WorkflowStep::SelectPdf;
        self.current_pdf = None;
        self.current_markdown = None;
        self.update_workflow_messages();
        self.status_message = "🔄 Ready to start a new document processing workflow!".to_string();
    }
    
}

/// Restore terminal to normal state
fn restore_terminal() {
    let _ = disable_raw_mode();
    let _ = execute!(
        io::stdout(),
        LeaveAlternateScreen,
        DisableMouseCapture,
        crossterm::cursor::Show,
        crossterm::style::ResetColor
    );
    // Force flush to ensure all commands are executed
    let _ = io::stdout().flush();
}

/// Run the TUI application with simple, safe cleanup
pub async fn run_tui(database: ChonkerDatabase) -> Result<()> {
    info!("Starting CHONKER TUI");
    
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    // Create app state
    let mut app = App::new(database);
    app.update_workflow_messages(); // Initialize workflow guidance
    app.load_documents().await?;
    
    // Run the main loop
    let result = run_app(&mut terminal, &mut app).await;
    
    // Simple cleanup - no fancy signal handling
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    
    // Handle any errors
    if let Err(err) = result {
        error!("TUI error: {:?}", err);
        return Err(err);
    }
    
    info!("TUI shut down successfully");
    Ok(())
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;
        
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => {
                    app.should_quit = true;
                }
                KeyCode::Tab => {
                    app.next_tab();
                }
                KeyCode::BackTab => {
                    app.previous_tab();
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if app.current_tab == 0 {
                        app.next_file();
                    } else {
                        app.next_document();
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if app.current_tab == 0 {
                        app.previous_file();
                    } else {
                        app.previous_document();
                    }
                }
KeyCode::Char('p') => {
                    if app.workflow_step == WorkflowStep::ProcessText {
                        app.process_markdown().await?;
                    } else {
                        app.status_message = "🔄 Process step not available yet - follow the workflow!".to_string();
                    }
                }
KeyCode::Char('s') => {
                    app.status_message = "🔍 Search functionality coming soon!".to_string();
                }
KeyCode::Char('e') => {
                    if app.workflow_step == WorkflowStep::ExportData {
                        app.export_to_csv().await?;
                    } else {
                        app.status_message = "📤 Export step not available yet - follow the workflow!".to_string();
                    }
                }
KeyCode::Char('f') => {
                    app.current_tab = 0;  // Switch to documents tab
                    app.load_available_files().await?;
                    app.status_message = "📁 Select a PDF file with ↑/↓, Enter to process".to_string();
                }
                KeyCode::Enter => {
                    if app.current_tab == 0 && !app.available_files.is_empty() {
                        app.process_selected_file().await?;
                    }
                }
                KeyCode::Char('r') => {
                    app.load_documents().await?;
                    app.status_message = "Documents refreshed".to_string();
                }
                KeyCode::Char('d') => {
                    app.delete_selected_document().await?;
                }
                KeyCode::Char('n') => {
                    if app.workflow_step == WorkflowStep::Complete {
                        app.start_new_workflow();
                    } else {
                        app.status_message = "🔄 Complete current workflow before starting new one!".to_string();
                    }
                }
                _ => {}
            }
        }
        
        if app.should_quit {
            break;
        }
    }
    Ok(())
}

fn ui(f: &mut Frame, app: &App) {
    let size = f.size();
    
    // Create modern layout with generous spacing
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2) // More generous margin for premium feel
        .constraints([
            Constraint::Length(4), // Tabs - taller for breathing room
            Constraint::Min(0),    // Content
            Constraint::Length(4), // Status - taller for better readability
            Constraint::Length(7), // Workflow guidance - more space for content
            Constraint::Length(4), // Help commands - taller for comfort
        ])
        .split(size);
    
    // Modern tabs with subtle styling
    let tab_titles = vec!["  Documents  ", "  Processing  ", "  Export  ", "  Settings  "];
    let mascot_title = "   CHONKER   ";
    let tabs = Tabs::new(tab_titles)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(mascot_title)
            .border_style(Style::default().fg(Color::Rgb(64, 64, 64)))) // Subtle gray border
        .style(Style::default().fg(Color::Rgb(192, 192, 192))) // Muted off-white text
        .highlight_style(Style::default()
            .fg(Color::Rgb(240, 240, 240)) // Bright off-white for selected
            .bg(Color::Rgb(58, 128, 200)) // Warp blue accent
            .add_modifier(Modifier::BOLD))
        .select(app.current_tab);
    f.render_widget(tabs, chunks[0]);
    
    // Render content based on current tab
    match app.current_tab {
        0 => render_documents_tab(f, app, chunks[1]),
        1 => render_processing_tab(f, app, chunks[1]),
        2 => render_export_tab(f, app, chunks[1]),
        3 => render_settings_tab(f, app, chunks[1]),
        _ => {}
    }
    
    // Modern status bar with subtle styling
    let status = Paragraph::new(app.status_message.as_str())
        .block(Block::default()
            .borders(Borders::ALL)
            .title("  Status  ")
            .border_style(Style::default().fg(Color::Rgb(64, 64, 64))))
        .style(Style::default().fg(Color::Rgb(140, 200, 140))); // Soft green
    f.render_widget(status, chunks[2]);
    
    // Render modern workflow guidance pane
    render_workflow_guidance(f, app, chunks[3]);
    
    // Modern help commands pane
    let help_text = if app.file_picker_open {
        "  FILE PICKER: ↑/↓ navigate  ⏎ process  esc close  "
    } else {
        "  q quit   tab tabs   ↑/↓ navigate   p process   e export   f files   r refresh   n new  "
    };
    
    let help_pane = Paragraph::new(help_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("  Commands  ")
            .border_style(Style::default().fg(Color::Rgb(64, 64, 64))))
        .style(Style::default().fg(Color::Rgb(160, 160, 160))); // Muted gray
    f.render_widget(help_pane, chunks[4]);
    
}

fn render_documents_tab(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1) // Add breathing room
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);
    
    // Modern document list with padding
let items: Vec<ListItem> = app
        .available_files
        .iter()
        .map(|path| {
            let filename = path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("unknown.pdf");
            let content = Line::from(vec![
                Span::styled(filename, Style::default().fg(Color::Rgb(240, 240, 240))),
            ]);
            ListItem::new(content)
        })
        .collect();
    
    let documents_list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("  Documents  ")
            .border_style(Style::default().fg(Color::Rgb(64, 64, 64))))
        .highlight_style(Style::default()
            .bg(Color::Rgb(58, 128, 200))
            .fg(Color::Rgb(240, 240, 240))
            .add_modifier(Modifier::BOLD))
        .highlight_symbol("  ▶  ");
    
    f.render_stateful_widget(documents_list, chunks[0], &mut app.file_list_state.clone());
    
    // Modern document details with padding
    let details_text = if let Some(selected) = app.list_state.selected() {
        if selected < app.documents.len() {
            let doc = &app.documents[selected];
            format!(
                "\n  ID: {}\n\n  Filename: {}\n\n  Created: {}\n\n  Chunks: {}",
                doc.id, doc.filename, doc.created_at, doc.chunk_count
            )
        } else {
            "\n  No document selected".to_string()
        }
    } else {
        "\n  No documents available\n\n  Use 'f' to select a PDF file to process".to_string()
    };
    
    let details = Paragraph::new(details_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("  Details  ")
            .border_style(Style::default().fg(Color::Rgb(64, 64, 64))))
        .style(Style::default().fg(Color::Rgb(160, 200, 160)));
    f.render_widget(details, chunks[1]);
}

fn render_processing_tab(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1) // Add breathing room
        .constraints([
            Constraint::Length(8),  // PDF Info - more space
            Constraint::Length(6),  // Controls - more space
            Constraint::Min(0),     // Processing log
        ])
        .split(area);
    
    // Modern PDF Info Section
    let current_file = if let Some(pdf) = &app.current_pdf {
        pdf.file_name().unwrap().to_str().unwrap()
    } else {
        "No PDF selected"
    };
    
    let pdf_info = Paragraph::new(format!(
        "\n  Current PDF: {}\n\n  Workflow Status: {}\n\n  Ready for processing",
        current_file,
        match app.workflow_step {
            WorkflowStep::SelectPdf => "Waiting for PDF selection",
            WorkflowStep::ExtractText => "Extracting text...",
            WorkflowStep::ProcessText => "Ready to process text",
            WorkflowStep::ExportData => "Ready to export",
            WorkflowStep::Complete => "Processing complete",
        }
    ))
        .block(Block::default()
            .borders(Borders::ALL)
            .title("  Current Document  ")
            .border_style(Style::default().fg(Color::Rgb(64, 64, 64))))
        .style(Style::default().fg(Color::Rgb(180, 180, 180)));
    f.render_widget(pdf_info, chunks[0]);
    
    // Modern Controls Section
    let controls = Paragraph::new("\n  Follow the workflow guidance below\n\n  Use 'f' to select files, 'p' to process, 'e' to export")
        .block(Block::default()
            .borders(Borders::ALL)
            .title("  Instructions  ")
            .border_style(Style::default().fg(Color::Rgb(64, 64, 64))))
        .style(Style::default().fg(Color::Rgb(160, 200, 160)));
    f.render_widget(controls, chunks[1]);
    
    // Modern Processing Log
    let log_text = if app.processing_log.is_empty() {
        "\n  No processing activity yet\n\n  Start by selecting a PDF file with 'f'".to_string()
    } else {
        format!("\n  {}\n", app.processing_log.join("\n  "))
    };
    
    let log = Paragraph::new(log_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("  Processing Log  ")
            .border_style(Style::default().fg(Color::Rgb(64, 64, 64))))
        .style(Style::default().fg(Color::Rgb(200, 200, 120)));
    f.render_widget(log, chunks[2]);
}

fn render_export_tab(f: &mut Frame, _app: &App, area: Rect) {
    let content = Paragraph::new("■ Export functionality will be implemented here.\n\n▶ Formats:\n- CSV\n- JSON\n- Parquet\n\n▶ Filters and queries coming soon!")
        .block(Block::default().borders(Borders::ALL).title("■ Export ■").border_style(Style::default().fg(Color::Black)))
        .style(Style::default().fg(Color::Green));
    f.render_widget(content, area);
}

fn render_settings_tab(f: &mut Frame, _app: &App, area: Rect) {
    let content = Paragraph::new("■ Settings and configuration.\n\n▶ Keys:\n- q: Quit\n- Tab: Next tab\n- ↑/↓: Navigate\n- r: Refresh\n- d: Delete")
        .block(Block::default().borders(Borders::ALL).title("■ Settings & Help ■").border_style(Style::default().fg(Color::Black)))
        .style(Style::default().fg(Color::Magenta));
    f.render_widget(content, area);
}

fn render_workflow_guidance(f: &mut Frame, app: &App, area: Rect) {
    // Modern progress indicator with subtle styling
    let progress_indicator = match app.workflow_step {
        WorkflowStep::SelectPdf => "●●○○○○  Step 1 of 4",
        WorkflowStep::ExtractText => "●●●○○○  Step 2 of 4", 
        WorkflowStep::ProcessText => "●●●●○○  Step 3 of 4",
        WorkflowStep::ExportData => "●●●●●○  Step 4 of 4",
        WorkflowStep::Complete => "●●●●●●  Complete",
    };
    
    let workflow_text = format!(
        "\n  {}\n\n  {}\n\n  {}",
        progress_indicator,
        app.workflow_messages.join("\n  "),
        if app.current_pdf.is_some() {
            format!("Current: {}", app.current_pdf.as_ref().unwrap().file_name().unwrap().to_str().unwrap())
        } else {
            "No document selected".to_string()
        }
    );
    
    // Modern Warp-style colors: muted and sophisticated
    let workflow_color = match app.workflow_step {
        WorkflowStep::SelectPdf => Color::Rgb(58, 128, 200),   // Warp blue
        WorkflowStep::ExtractText => Color::Rgb(200, 160, 58), // Soft yellow
        WorkflowStep::ProcessText => Color::Rgb(58, 128, 200), // Back to blue
        WorkflowStep::ExportData => Color::Rgb(58, 128, 200),  // Blue again
        WorkflowStep::Complete => Color::Rgb(120, 180, 120),   // Soft green
    };
    
    let workflow_pane = Paragraph::new(workflow_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("  Workflow  ")
            .border_style(Style::default().fg(Color::Rgb(64, 64, 64))))
        .style(Style::default().fg(workflow_color));
    
    f.render_widget(workflow_pane, area);
}

