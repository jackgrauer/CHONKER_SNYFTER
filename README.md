# CHONKER 🐹

Elegant PDF processing with hamster wisdom. Extract, edit, and export PDF content to Parquet format for high-performance analysis.

## Features

- **ML-Powered Extraction**: Process PDFs with Docling's advanced ML models
- **Spatial Layout Preservation**: Form-aware extraction that maintains 2D positioning
- **Quality Control**: Edit and refine extracted content before export
- **Parquet Export**: Export to columnar Parquet format for blazing-fast analysis
- **Clean UI**: Minimalist PyQt6 interface with hamster charm
- **Fast**: Powered by uv package manager (10-100x faster than pip)

## Quick Start

```bash
# Run with launcher
./run_chonker.sh

# Or manually
source .venv/bin/activate
python chonker.py
```

## Installation

```bash
# Clone the repository
git clone <your-repo-url>
cd CHONKER

# Install uv (if not already installed)
curl -LsSf https://astral.sh/uv/install.sh | sh

# Set up environment
./migrate_to_uv.sh
```

## Keyboard Shortcuts

- **Cmd+O**: Open PDF from File
- **Cmd+U**: Open PDF from URL
- **Cmd+P**: Extract to HTML
- **Cmd+E**: Export to Parquet
- **Cmd+F**: Toggle Search
- **Cmd+Q**: Quit application

## Export Format: Parquet

When you export (Cmd+E), CHONKER creates a directory with 4 Parquet files:

### 1. `chonker_exports.parquet`
Export metadata including:
- Export ID and timestamp
- Source PDF path
- Original and edited HTML (for change tracking)
- User who performed QC
- Edit count

### 2. `chonker_content.parquet`
Document structure and content:
- Element type (h1, p, table, etc.)
- Element order (preserves document flow)
- Full text content
- Complete HTML with formatting
- Page numbers
- Metadata (level, position)

### 3. `chonker_styles.parquet`
Text formatting information:
- Bold, italic, underline flags
- Font sizes
- Text colors
- Preserves all visual styling

### 4. `chonker_semantics.parquet`
Content classification:
- Semantic roles (header, financial_text, data_table, etc.)
- Word and character counts
- Confidence scores
- Enables intelligent filtering

### Why Parquet?

- **10-100x faster** queries than row-based formats
- **70% smaller** files due to columnar compression
- **Universal support** - works with Python, R, Rust, SQL engines
- **Cloud-native** - integrates with S3, BigQuery, Snowflake
- **Perfect for SNYFTER** - Rust-based analysis tool for financial documents

## Project Structure

```
CHONKER/
├── main.py                        # Entry point
├── chonker/                       # Main package
│   ├── models/                    # Data models
│   │   ├── bbox.py               # Coordinate system
│   │   ├── document.py           # Document with edit tracking
│   │   └── layout_item.py        # Spatial items
│   ├── extraction/                # PDF processing
│   │   ├── pdf_extractor.py      # Docling wrapper
│   │   └── spatial_layout.py     # Layout engine (fixes overlaps!)
│   ├── export/                    # Export functionality
│   │   ├── html_generator.py     # HTML generation
│   │   └── parquet_exporter.py   # Parquet export
│   └── ui/                        # User interface
│       ├── main_window.py         # Main window
│       ├── editor_widget.py       # Content editor
│       └── pdf_viewer.py          # PDF display
├── assets/emojis/chonker.png      # Sacred hamster emoji
├── pyproject.toml                 # Modern Python config
├── requirements.txt               # Dependencies
└── run_chonker.sh                 # Launch script
```

## Dependencies

- PyQt6 & PyQt6-WebEngine (UI framework)
- Docling (ML-powered PDF extraction)
- DuckDB (SQL database engine)
- BeautifulSoup4 (HTML processing)
- pandas (Data manipulation)

## Development

```bash
# Install with dev dependencies
source .venv/bin/activate
uv pip install -r requirements.txt

# Run tests
just test

# Format code
just format

# Clean build artifacts
just clean
```

## Using the Parquet Export

### Python Example
```python
import pandas as pd
import pyarrow.parquet as pq

# Read the exported data
content = pd.read_parquet('output/chonker_content.parquet')
styles = pd.read_parquet('output/chonker_styles.parquet')
semantics = pd.read_parquet('output/chonker_semantics.parquet')

# Find all bold financial text
bold_financial = content.merge(styles, left_on='content_id', right_on='element_id') \
                       .merge(semantics, on='element_id') \
                       .query("style_bold == True and semantic_role == 'financial_text'")

# Get all headers from page 5
page5_headers = content[
    (content['element_type'].isin(['h1', 'h2', 'h3'])) & 
    (content['element_metadata'].str.contains('"page": 5'))
]
```

### Rust/SNYFTER Example
```rust
use arrow::record_batch::RecordBatch;
use parquet::arrow::ArrowReader;

// Lightning-fast columnar analytics
let content = ParquetReader::new("chonker_content.parquet")?;
let financial_elements = content
    .filter(|row| row.semantic_role == "financial_text")
    .select(&["element_text", "page"])
    .collect();
```

### DuckDB Query
```sql
-- Query Parquet files directly without loading
SELECT c.element_text, s.style_bold, sem.semantic_role
FROM 'output/chonker_content.parquet' c
JOIN 'output/chonker_styles.parquet' s ON c.content_id = s.element_id
JOIN 'output/chonker_semantics.parquet' sem ON c.element_id = sem.element_id
WHERE s.style_bold = true AND sem.semantic_role = 'header';
```

## Snyfter Integration

CHONKER now works with **Snyfter**, a high-performance PDF preprocessing tool written in Rust. Snyfter prepares scanned or image-heavy PDFs for better text extraction.

### Workflow
1. **Analyze PDFs** with Snyfter to check if preprocessing is needed
2. **Enhance PDFs** that have low text density  
3. **Process with CHONKER** for spatial layout extraction

See the [Snyfter README](snyfter/README.md) for detailed usage.

## Recent Updates

### 2025-07-28 - MAJOR REFACTOR 🐹
- **FIXED TEXT OVERLAP ISSUE!** Complete architectural overhaul
  - New `BoundingBox` class with unified coordinate system (top-left origin)
  - `SpatialLayoutEngine` detects and resolves overlaps before rendering
  - Clean separation: PDF → Document → Layout → HTML
  - Modular architecture replaces 2400-line monolith
- **Spatial Layout Engine**:
  - Items on same line shift horizontally to avoid overlap
  - Different lines shift vertically
  - Row grouping for form-like structures
  - Dynamic font sizing based on available space
- **Sacred hamster preserved**: All UI elements and wisdom intact
- **Result**: NO MORE TEXT OVERLAPPING! Form fields align properly!

### 2025-07-27
- **UI Improvements**: Major spatial layout enhancements for better readability
  - Fixed white pane bug: Right pane now starts with matching dark background (#525659)
  - Unified color scheme: All backgrounds now use consistent chrome color
  - Removed boxes around text for cleaner appearance
  - Reduced font size to 50% of base (6px at default zoom) to minimize overlap
  - Made all headings same size as regular text (distinguished by teal color only)
  - Added padding (3px/5px) and vertical spacing (5px) between text elements
  - Fixed font warning by replacing "-apple-system" with Arial
- **PDF Processing**: Enhanced split and merge functionality
  - Added PDF splitting capability for multi-page documents
  - Improved image preservation during enhancement
  - Better Python integration using pypdf for reliability
  
### 2025-07-26
- **Spatial Layout Mode**: New form-aware extraction that preserves 2D positioning
- **WebEngine Integration**: Right pane now uses QWebEngineView for proper CSS rendering
- **Coordinate Preservation**: Extracts and uses bounding box data from PDFs
- **Form Detection**: Automatically detects form-like documents for spatial mode
- **Visual Positioning**: Elements appear at their actual PDF coordinates instead of linear text
- **OCR Preprocessing Removed**: Removed 262 lines of OCR code - now handled by Snyfter
- **Snyfter Integration**: New Rust-based PDF preprocessing tool for enhanced extraction
- **Cleaner Dependencies**: Removed DuckDB, psutil, pymupdf - reduced to essentials

### 2025-07-24
- **Parquet Export**: Replaced DuckDB with columnar Parquet format
- **URL Support**: Open PDFs directly from web URLs (Cmd+U)
- **Enhanced Search**: Next/previous navigation with match counting
- **Code Reduction**: From 2,432 to 1,842 lines while adding features

### 2025-07-20
- **Migrated to uv**: Replaced pip with the blazing-fast uv package manager
- **Cleaned codebase**: Removed 40+ unnecessary files, keeping only essentials
- **Modern Python setup**: Added pyproject.toml for standard Python packaging
- **Simplified structure**: From 100+ files down to just 14 essential files

## License

MIT License - Feel free to use this hamster-powered technology responsibly!

---

Built with 🐹 by the CHONKER development team