use crate::app::App;
use ratatui::{
    Frame,
    layout::{Layout, Constraint, Direction},
    widgets::{Block, Borders, Gauge, Paragraph},
    text::{Line, Span, Text},
};

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(3), // File selection
                Constraint::Length(7), // Processing options
                Constraint::Length(3), // Action
                Constraint::Min(5),    // PDF Viewer
                Constraint::Length(3), // Status
            ]
            .as_ref(),
        )
        .split(f.size());

    // File Selection
    let file_block = Block::default()
        .borders(Borders::ALL)
        .title("📁 File Selection");
    
    let file_text = if app.file_input.is_empty() {
        "Press Enter to open file dialog".to_string()
    } else {
        format!("Selected: {}", app.file_input)
    };
    
    let file_paragraph = Paragraph::new(file_text)
        .block(file_block);
    f.render_widget(file_paragraph, chunks[0]);

    // Processing Options
    let options_block = Block::default()
        .borders(Borders::ALL)
        .title("Processing Options");
    let mut options_text = vec![Line::from(vec![Span::raw("OCR: "), Span::raw(if app.processing_options.ocr_enabled { "On" } else { "Off" })])];
    options_text.push(Line::from(vec![Span::raw("Formula Recognition: "), Span::raw(if app.processing_options.formula_recognition { "On" } else { "Off" })]));
    options_text.push(Line::from(vec![Span::raw("Table Detection: "), Span::raw(if app.processing_options.table_detection { "On" } else { "Off" })]));
    let options_paragraph = Paragraph::new(Text::from(options_text))
        .block(options_block);
    f.render_widget(options_paragraph, chunks[1]);

    // Action
    let action_block = Block::default()
        .borders(Borders::ALL)
        .title("Action");
    let action_paragraph = Paragraph::new("Press Enter to Process")
        .block(action_block);
    f.render_widget(action_paragraph, chunks[2]);

    // Progress Indicator
    if app.is_processing {
        let gauge = Gauge::default()
            .block(Block::default().title("Progress").borders(Borders::ALL))
            .gauge_style(ratatui::style::Style::default().fg(ratatui::style::Color::Green))
            .ratio(app.processing_progress / 100.0);
        f.render_widget(gauge, chunks[2]);
    }

    // PDF Viewer & Chunk Preview
    let viewer_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(50), // PDF Viewer
                Constraint::Percentage(50), // Chunk Preview
            ]
            .as_ref(),
        )
        .split(chunks[3]);

    let pdf_viewer_block = Block::default()
        .borders(Borders::ALL)
        .title("PDF Viewer");
    f.render_widget(pdf_viewer_block, viewer_chunks[0]);

    let chunk_preview_block = Block::default()
        .borders(Borders::ALL)
        .title("Chunk Preview");

    if let Some(chunk) = app.get_current_chunk() {
        let chunk_text = vec![Line::from(Span::raw(chunk.content.clone()))];
        let chunk_paragraph = Paragraph::new(Text::from(chunk_text))
            .block(chunk_preview_block);
        f.render_widget(chunk_paragraph, viewer_chunks[1]);
    } else {
        f.render_widget(chunk_preview_block, viewer_chunks[1]);
    }

    // Status Message
    let status_block = Block::default()
        .borders(Borders::ALL)
        .title("Status");
    let status_paragraph = Paragraph::new(app.status_message.clone())
        .block(status_block);
    f.render_widget(status_paragraph, chunks[4]);
}
