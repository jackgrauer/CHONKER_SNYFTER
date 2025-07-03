# 📊 CHONKER Data Visualization System

This document explains how to use and integrate the data visualization system in CHONKER's GUI application.

## Overview

The data visualization system provides a document-agnostic way to display extracted content from any type of document in the GUI's right-hand pane. It's designed to work seamlessly with CHONKER's extraction pipeline while being flexible enough to handle any document type.

## Components

### 1. DataVisualizationPane (`src/data_visualization.rs`)

The main GUI component that renders extracted data in multiple view modes:

- **Overview**: Summary statistics and content block previews
- **Detailed**: Expandable view of all content with full details
- **Tables Only**: Focus on extracted table data with qualifiers
- **Issues Only**: Quality control view showing extraction problems

#### Features:
- 📊 Interactive table display with qualifier highlighting
- 🔍 Content filtering and search
- 📝 Metadata display (toggleable)
- 🏷️ Qualifier and quality issue visualization
- 📈 Extraction statistics summary
- 🎨 Rich text formatting support

### 2. ExtractionIntegrator (`src/extraction_integration.rs`)

Bridges the Python extraction output with the GUI visualization system:

- Converts JSON output from Python scripts to GUI data structures
- Handles all document types (tables, text, lists, images, formulas, charts)
- Performs quality control and issue detection
- Provides async document processing interface

### 3. Content Types Supported

The system is document-agnostic and supports:

- **Tables**: With headers, data rows, and qualifiers
- **Text**: With formatting (bold, italic, alignment)
- **Lists**: Bulleted, numbered, and nested
- **Images**: With captions and file paths
- **Formulas**: LaTeX and rendered text
- **Charts**: With data points and categories

## Integration Guide

### Basic Integration

Add the visualization components to your existing ChonkerApp:

```rust
use crate::data_visualization::DataVisualizationPane;
use crate::extraction_integration::ExtractionIntegrator;

pub struct ChonkerApp {
    // ... existing fields ...
    pub data_viz_pane: DataVisualizationPane,
    pub extraction_integrator: ExtractionIntegrator,
}

impl ChonkerApp {
    pub fn new(cc: &eframe::CreationContext, database: Option<ChonkerDatabase>) -> Self {
        Self {
            // ... existing initialization ...
            data_viz_pane: DataVisualizationPane::new(),
            extraction_integrator: ExtractionIntegrator::new(),
        }
    }
}
```

### Rendering in the Right Pane

Replace your right pane content with the visualization:

```rust
fn render_right_pane(&mut self, ui: &mut egui::Ui) {
    self.data_viz_pane.render(ui);
}
```

### Processing Documents

Add document processing with visualization:

```rust
pub async fn process_document(&mut self, pdf_path: PathBuf) -> Result<(), anyhow::Error> {
    let extracted_data = self.extraction_integrator.process_document(&pdf_path).await?;
    self.data_viz_pane.load_data(extracted_data);
    Ok(())
}
```

## Usage Examples

### Loading Sample Data

For testing and development:

```rust
use crate::data_visualization::ExtractedData;

let sample_data = ExtractedData::create_sample();
app.data_viz_pane.load_data(sample_data);
```

### Processing Real Documents

```rust
// Process a PDF file
let pdf_path = PathBuf::from("document.pdf");
let extracted_data = app.extraction_integrator.process_document(&pdf_path).await?;
app.data_viz_pane.load_data(extracted_data);
```

### Filtering and Search

Users can filter content in the GUI:

```rust
// This is handled automatically by the UI
// Users type in the search box to filter content
app.data_viz_pane.search_filter = "temperature".to_string();
```

## Data Structure

The system uses a flexible `ExtractedData` structure:

```rust
pub struct ExtractedData {
    pub source_file: String,
    pub extraction_timestamp: String,
    pub tool_used: String,
    pub processing_time_ms: u64,
    pub content_blocks: Vec<ContentBlock>,
    pub metadata: HashMap<String, String>,
    pub statistics: ExtractionStatistics,
}
```

### Content Blocks

Each content block can be one of:

- `Table { headers, rows, qualifiers, metadata }`
- `Text { content, formatting, metadata }`
- `List { items, list_type, metadata }`
- `Image { title, caption, file_path, metadata }`
- `Formula { latex, rendered_text, metadata }`
- `Chart { data, chart_type, metadata }`

## Python Integration

The system works with the existing Python extraction bridge:

```bash
# The ExtractionIntegrator automatically calls:
python venv/bin/python python/extraction_bridge.py document.pdf --tool docling_enhanced --output-format json
```

## View Modes

### 1. Overview Mode
- Extraction statistics summary
- Content block previews
- Quick navigation to detailed view

### 2. Detailed Mode
- Expandable content blocks
- Full data display with formatting
- Metadata and qualifier information

### 3. Tables Only Mode
- Focus on tabular data
- Qualifier highlighting
- Column/row information

### 4. Issues Only Mode
- Quality control problems
- Suggested fixes
- Issue severity indicators

## Quality Control Features

The system automatically detects and highlights:

- Missing data in tables
- Structural inconsistencies
- Low confidence extractions
- Formatting issues
- Data inconsistencies

## Customization

### Themes and Colors

The visualization uses egui's color system with semantic colors:

- 🔴 Critical issues (Red)
- 🟡 Warnings (Yellow)
- 🔵 Information (Blue)
- 🟢 Success/Good (Green)

### View Options

Users can toggle:
- Metadata display
- Qualifier visibility
- Content filtering
- Expansion state

## Performance

The system is designed for:
- Large documents (1000+ content blocks)
- Real-time filtering and search
- Smooth scrolling with egui
- Memory-efficient rendering

## Future Enhancements

Planned features:
- Export functionality (CSV, JSON, HTML)
- Interactive table editing
- Chart visualization with plots
- Custom qualifier definitions
- Batch processing views

## Troubleshooting

### Common Issues

1. **No data displayed**: Check that the Python extraction script is working
2. **Formatting problems**: Verify the JSON output format from Python
3. **Performance issues**: Use content filtering for large documents
4. **Missing qualifiers**: Check the qualifier detection in the extraction pipeline

### Debug Mode

Enable debug information:

```rust
app.data_viz_pane.show_metadata = true; // Show all metadata
```

## Example Complete Integration

See `src/app_integration_example.rs` for a complete working example of how to integrate the data visualization system into a new or existing CHONKER application.

The example shows:
- Three-pane layout (file browser, document preview, data visualization)
- Async document processing
- Sample data loading
- Status management
- Error handling

This provides a complete, document-agnostic visualization system that can display any type of extracted content in the GUI's right-hand pane.
