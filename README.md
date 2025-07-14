# CHONKER Document Processor

A streamlined document processing pipeline using **Python + Docling + HTML viewer generation**.

## 🚀 Quick Start

### 1. Install Dependencies
```bash
just install
```

### 2. Process Any Document
```bash
just process mydocument.pdf
```

### 3. View Results
The command automatically generates an HTML viewer that opens in your browser.

## 📋 Available Commands

| Command | Description |
|---------|-------------|
| `just info` | Show project status and stats |
| `just check` | Check Python environment |
| `just install` | Install Python dependencies |
| `just process FILE` | Process any document file |
| `just viewer BASENAME` | Generate HTML viewer for processed document |
| `just list` | List processed documents and HTML viewers |
| `just clean` | Clean all processed files |
| `just backend` | Start Python backend service |

## 📁 Project Structure

```
CHONKER_SNYFTER/
├── process_document.py          # Main document processor
├── generate_viewer.py           # HTML viewer generator
├── justfile                     # Command automation
├── apps/
│   └── doc-service/
│       ├── main.py             # FastAPI backend (optional)
│       ├── requirements.txt    # Python dependencies
│       └── processed_documents/ # Output directory
└── *.html                      # Generated HTML viewers
```

## 🔧 Supported Document Types

- **PDF** (.pdf)
- **Word** (.docx)
- **PowerPoint** (.pptx)
- **HTML** (.html)
- **Markdown** (.md)
- **CSV** (.csv)
- **Excel** (.xlsx)
- **AsciiDoc** (.asciidoc)

## 📊 Processing Pipeline

```
Document → Docling → Extract Text/Tables/Metadata → Generate HTML Viewer
```

### Example Usage

```bash
# Process a PDF document
just process report.pdf

# This creates:
# - report_text.md         (extracted text)
# - report_tables.json     (table data)
# - report_metadata.json   (document metadata)
# - report_viewer.html     (HTML viewer)
```

## 🎯 Features

- **Universal Document Support**: Process any document type
- **Professional HTML Output**: Clean, styled viewers
- **Table Extraction**: Structured table data preservation
- **Metadata Extraction**: Document properties and statistics
- **Responsive Design**: Works on any device
- **Interactive Elements**: Toggle sections, collapsible content
- **Document Statistics**: Word count, table count, etc.

## 🛠️ Development

### Direct Python Usage

```bash
# Process document directly
python process_document.py document.pdf

# Generate viewer from existing processed data
python generate_viewer.py document_basename
```

### Backend Service (Optional)

```bash
# Start WebSocket-enabled backend
just backend

# Access at http://localhost:8000
```

## 🏆 Why This Approach?

- **Simple**: 2 Python scripts, no complex frameworks
- **Fast**: Direct processing, no GUI overhead
- **Reliable**: Proven Python + Docling combination
- **Universal**: Works with any document type
- **Shareable**: HTML files work anywhere
- **Maintainable**: Clean, focused codebase

## 🔄 Workflow

1. **Drop document** → `just process document.pdf`
2. **Get HTML viewer** → Opens automatically in browser
3. **Share results** → Send HTML file to anyone
4. **Process next document** → Repeat

That's it! No complex setup, no GUI maintenance, just document processing that works.

### Updates in CHONKER 🐹

The `chonker.py` script has been significantly enhanced with the following features:

- **Base64 PDF Encoding**: Documents are now embedded directly as base64 data URLs to circumvent CORS issues, allowing for seamless local file access.
- **WYSIWYG Editor with PDF.js**: Enhanced editor with a PDF viewer on the left and editable content on the right, supporting all major editing functionalities.
- **Native File Picker for macOS**: Uses a native macOS file picker for better user experience.
- **Resizing and Fullscreen Capabilities**: The interface supports dynamic resizing of panes and an improved fullscreen mode.
- **Optimized Auto-Save**: Auto-save to `localStorage` with debouncing, ensuring minimal data loss.
- **Improved Toolbar and Custom Controls**: New toolbar with options for navigation, zoom, and document optimization.
- **Advanced Interaction**: Supports keyboard shortcuts, Apple trackpad gestures, and includes a customizable context menu for table editing.
- **Error Handling and Performance**: Better error handling, and performance improvements with lazy loading and resource optimization.
- **Platform Agnostic Browser Launching**: Tries to launch in Chrome app mode on macOS, with fallbacks to Safari and other system defaults.

### Usage

After processing, the HTML editor automatically launches with:

- **PDF Viewer Controls**: Navigate, zoom, and more using the toolbar.
- **Editable Content**: Click-to-edit functionality for document content.
- **Easy File Management**: Open new documents easily from the interface.
- **Customizable Interface**: Users can edit the HTML and JavaScript embedded in `chonker.py` for customization.
