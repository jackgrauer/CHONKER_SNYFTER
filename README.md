# CHONKER 🐹

## 🚀 CHONKER9 - Terminal PDF Editor Supremacy

### [CHONKER9 (Rust Terminal Editor)](chonker9/) ⭐ **NEW & RECOMMENDED**
Advanced terminal PDF viewer with spatial editing capabilities - the future of PDF editing

### CHONKER (Python GUI) - Legacy
Original ML-powered PDF processing with PyQt6 interface

## Features

### 🦀 CHONKER9 (Rust) - THE FUTURE
- **Terminal PDF Editing**: WYSIWYG editing with spatial layout preservation
- **ALTO XML Processing**: Uses pdfalto for high-quality text extraction  
- **Ropey Text Engine**: O(log n) text operations for large documents
- **Cosmic Text Layout**: Advanced text shaping and typography
- **Mouse & Selection**: Click-to-position cursor with block selection
- **Crash-Safe**: Robust bounds checking for reliable editing
- **Lightning Fast**: 2MB binary, instant startup, minimal dependencies
- **Native Feel**: Terminal-based but with modern editor features

### 🐍 CHONKER (Python) - Legacy Support
- ML-Powered Extraction with Docling models
- Parquet export for data analysis
- PyQt6 GUI interface

## Quick Start

### CHONKER (Python GUI)
```bash
# Run with launcher
./run_chonker.sh

# Or manually
source .venv/bin/activate
python chonker.py
```

### CHONKER9 (Rust Terminal)
```bash
# Build and run
cd chonker9
cargo build --release
./target/release/chonker9 document.pdf

# Or install globally
cargo install --path chonker9
chonker9 document.pdf
```

## Installation

```bash
# Clone the repository
git clone https://github.com/jackgrauer/CHONKER_SNYFTER.git
cd CHONKER_SNYFTER

# Install uv (if not already installed)
curl -LsSf https://astral.sh/uv/install.sh | sh

# Set up environment
./migrate_to_uv.sh
```

## Keyboard Shortcuts

### CHONKER (Python)
- **Cmd+O**: Open PDF from File
- **Cmd+U**: Open PDF from URL
- **Cmd+P**: Extract to HTML
- **Cmd+E**: Export to Parquet
- **Cmd+F**: Toggle Search
- **Cmd+Q**: Quit application

### CHONKER9 (Rust)
- **Arrow Keys**: Navigate cursor
- **Shift+Arrows**: Block selection
- **Click & Drag**: Mouse selection
- **Ctrl+A**: Select all
- **Ctrl+S**: Save edited text
- **Ctrl+Q**: Quit editor
- **Home/End**: Line navigation

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
CHONKER_SNYFTER/
├── chonker.py                         # Main Python application
├── chonker9/                          # Rust terminal editor
│   ├── src/main.rs                   # Terminal PDF editor
│   ├── Cargo.toml                    # Rust dependencies
│   └── README.md                     # Chonker9 documentation
├── snyfter/                           # Rust PDF preprocessing
├── pyproject.toml                     # Python configuration
├── requirements.txt                   # Python dependencies
└── README.md                          # This file
```

## Dependencies

### CHONKER (Python)
- PyQt6 & PyQt6-WebEngine (UI framework)
- Docling (ML-powered PDF extraction)
- DuckDB (SQL database engine)
- BeautifulSoup4 (HTML processing)
- pandas (Data manipulation)

### CHONKER9 (Rust)
- crossterm (Terminal UI framework)
- quick-xml (ALTO XML parsing)
- ropey (Advanced text editing)
- cosmic-text (Text layout engine)

## Development

```bash
# Python development
source .venv/bin/activate
uv pip install -r requirements.txt

# Rust development
cd chonker9
cargo build --release

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

### 2025-09-04 - CHONKER9 Added 🦀
- **NEW**: Advanced terminal PDF editor with spatial editing
- **ALTO XML Integration**: High-quality text extraction via pdfalto
- **Ropey + Cosmic Text**: Professional-grade text editing engine
- **Mouse Support**: Click-to-position cursor and drag selection
- **Block Selection**: Keyboard and mouse-based text selection
- **Crash-Safe**: Robust bounds checking for stable editing

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

## License

MIT License - Feel free to use this hamster-powered technology responsibly!

---

Built with 🐹 by the CHONKER development team