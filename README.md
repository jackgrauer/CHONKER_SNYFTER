# 🐹 CHONKER - Comprehensive Document Processing and Editing Suite

CHONKER is a powerful document processing toolkit that provides multiple approaches for extracting, viewing, and editing PDF documents with advanced table preservation and WYSIWYG capabilities. Originally a streamlined document processing pipeline using **Python + Docling + HTML viewer generation**, CHONKER has evolved into a comprehensive suite of tools.

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

## 🏗️ Architecture

```
CHONKER_SNYFTER/
├── chonker.py              # Main web-based editor (primary version)
├── chonker_simple.py       # Simplified two-window version
├── chonker_qt.py          # Qt native desktop application
├── chonker_qt_faithful.py  # Chunk-based faithful editor
├── chonker_qt_mixed.py    # Mixed content renderer
├── apps/
│   └── doc-service/       # FastAPI document processing service
│       ├── main.py        # WebSocket-enabled API endpoints
│       ├── requirements.txt
│       └── processed_documents/  # Document output directory
├── requirements.txt        # Python dependencies for web versions
├── requirements_qt.txt     # Additional Qt dependencies
├── Justfile               # Task automation
└── turbo.json            # Turborepo configuration
```

## 📈 Recent Development

Recent commits show active development on CHONKER:
- **July 2025**: Added responsive text wrapping to editor pane
- **July 2025**: Enhanced WYSIWYG editor with PDF.js integration
- **July 2025**: Added base64 PDF embedding and improved editor
- **July 2025**: Turborepo integration completed
- **July 2025**: Made service command default to background mode

## 🎯 Use Cases

1. **Document Review**: View PDF while editing extracted content
2. **Table Extraction**: Complex tables preserved and editable
3. **Research Notes**: Extract and annotate academic papers
4. **Legal Documents**: Maintain formatting while editing
5. **Technical Documentation**: Process and update manuals
6. **Report Generation**: Extract data and create new reports

## 🐹 CHONKER Versions

### 1. **chonker.py** - Main Web-Based Editor (Primary Version)
The flagship CHONKER implementation featuring:
- **Dual-pane interface** with PDF viewer and WYSIWYG editor
- **Base64 PDF Encoding**: Documents embedded directly as base64 data URLs to circumvent CORS issues
- **PDF.js Integration**: Full-featured PDF viewer with navigation controls
- **CKEditor 5**: Rich text editing with table support
- **Responsive Design**: Adjustable pane sizing with drag-to-resize
- **Native File Picker**: macOS native file selection dialog
- **Auto-Save**: Optimized auto-save to localStorage with debouncing
- **Trackpad Support**: Two-finger scrolling and pinch-to-zoom
- **Zoom Controls**: PDF zoom from 25% to 800%
- **Platform-Agnostic Launching**: Chrome app mode with fallbacks

**Latest Improvements (July 2025):**
- Enhanced trackpad support for horizontal PDF navigation
- Responsive editor pane with automatic text wrapping
- Tables with horizontal scrolling when needed
- Images that scale to fit pane width
- Code blocks with proper overflow handling
- Initial zoom at 200% for better visibility

**Usage:**
```bash
python chonker.py [pdf_file]
# Or use the native file picker:
python chonker.py
```

### 2. **chonker_simple.py** - Streamlined Two-Window Version
A minimalist approach that:
- Opens PDF in system's native PDF viewer
- Displays extracted content in CKEditor
- Reduced dependencies and complexity
- Quick editing without dual-pane complexity

**Usage:**
```bash
python chonker_simple.py [pdf_file]
```

### 3. **chonker_qt.py** - Native Qt Desktop Application
PyQt-based native application featuring:
- Native PDF viewer widget (no web browser needed)
- Integrated text editor with syntax highlighting
- Native OS scrolling and UI controls
- True desktop application experience
- No JavaScript or web dependencies

**Usage:**
```bash
python chonker_qt.py [pdf_file]
```

### 4. **chonker_qt_faithful.py** - Chunk-Based Structure Preservation
Advanced editor that maintains document fidelity:
- Each document element (paragraph, table, heading) as separate widget
- Preserves document hierarchy and structure
- Individual editing of document components
- Faithful representation of original layout
- Best for documents with complex structure

**Usage:**
```bash
python chonker_qt_faithful.py [pdf_file]
```

### 5. **chonker_qt_mixed.py** - Mixed Content Renderer
Full-featured mixed content support:
- WebEngine integration for rich content
- Interactive table editing
- Media embedding support
- Advanced formatting preservation

**Usage:**
```bash
python chonker_qt_mixed.py [pdf_file]
```

## 📦 Installation Requirements

### Basic Web Version
```bash
pip install -r requirements.txt
```

### Qt Desktop Versions
```bash
pip install -r requirements_qt.txt
```

## 🎯 Feature Comparison

| Feature | chonker.py | simple | qt | qt_faithful | qt_mixed |
|---------|------------|--------|-------|-------------|----------|
| PDF Viewing | PDF.js | Native | Qt Widget | Qt Widget | WebEngine |
| Editor | CKEditor 5 | CKEditor 5 | Qt Text | Custom | WebEngine |
| Tables | ✓ | ✓ | Basic | ✓ | ✓ |
| Auto-save | ✓ | ✓ | ✓ | ✓ | ✓ |
| Trackpad | ✓ | - | Native | Native | Native |
| Dependencies | Minimal | Minimal | PyQt | PyQt | PyQt |
| Structure Preservation | Good | Good | Basic | Excellent | Excellent |

## 🛠️ Service Mode

Run CHONKER as a background service:
```bash
just service       # Start in background
just service-logs  # View service logs
just service-stop  # Stop service
```

API Endpoints:
- `POST /process-document`: Upload and process PDF
- `GET /documents/{doc_id}`: Retrieve processed document
- `WebSocket /ws/{session_id}`: Real-time processing updates

## 🚀 Future Roadmap

- [ ] Cloud storage integration
- [ ] Collaborative editing features
- [ ] Export to multiple formats
- [ ] OCR for scanned documents
- [ ] Plugin system for custom extractors
- [ ] Mobile-responsive editor
- [ ] Batch processing capabilities

## 🤝 Contributing

Contributions are welcome! Please:
1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is part of the CHONKER_SNYFTER suite. See LICENSE file for details.

## 🙏 Acknowledgments

- **Docling** for powerful document extraction
- **PDF.js** for PDF rendering capabilities
- **CKEditor 5** for rich text editing
- **PyQt** for native desktop applications
- **FastAPI** for the service backend

---

**CHONKER**: Making document processing chunky and delightful! 🐹
