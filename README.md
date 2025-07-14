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
