# CHONKER 🐹

Elegant PDF processing with hamster wisdom. Extract, edit, and export PDF content to SQL databases.

## Features

- **ML-Powered Extraction**: Process PDFs with Docling's advanced ML models
- **Quality Control**: Edit and refine extracted content before export
- **Multi-Format Export**: Save to DuckDB, Arrow Dataset, JSON, or CSV
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

- **Cmd+O**: Open PDF
- **Cmd+P**: Process document
- **Cmd+E**: Export to SQL
- **Cmd+Shift+E**: Export to Arrow Dataset
- **Cmd+Q**: Quit application

## Export Formats

- **DuckDB**: Full-featured SQL database with metadata and content tables
- **Arrow Dataset**: Partitioned Parquet files with rich metadata for big data analytics
- **JSON**: Structured data for APIs and web apps
- **CSV**: Simple tabular export

## Project Structure

```
CHONKER/
├── chonker.py                     # Main application (1,745 lines)
├── assets/emojis/chonker.png      # UI icon
├── pyproject.toml                  # Modern Python config
├── requirements.txt                # Dependencies
├── justfile                        # Build automation
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

## Arrow Dataset Export

The new Arrow Dataset export feature provides:
- **Partitioned storage**: Data organized by document ID and element type
- **Rich metadata**: Includes style information (bold, italic, colors) and semantic roles
- **Efficient queries**: Use PyArrow's dataset API for fast filtering and aggregation
- **Example queries**: Auto-generated Python script with query examples

Example usage:
```python
import pyarrow.dataset as ds
dataset = ds.dataset("path/to/export", format="parquet")
# Find all bold headers about revenue
bold_headers = dataset.to_table(
    filter=(ds.field("style_bold") == True) & 
           (ds.field("semantic_role") == "header")
).to_pandas()
```

## Recent Updates (2025-07-20)

- **Added Arrow Dataset Export**: New PyArrow-based export for big data workflows
- **Migrated to uv**: Replaced pip with the blazing-fast uv package manager
- **Cleaned codebase**: Removed 40+ unnecessary files, keeping only essentials
- **Modern Python setup**: Added pyproject.toml for standard Python packaging
- **Simplified structure**: From 100+ files down to just 14 essential files

## License

MIT License - Feel free to use this hamster-powered technology responsibly!

---

Built with 🐹 by the CHONKER development team