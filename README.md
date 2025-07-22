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
- **Cmd+E**: Export (creates both SQL database and Arrow dataset)
- **Cmd+Q**: Quit application

## Export Formats

- **DuckDB Export (Cmd+E)**: Single `.duckdb` file containing:
  - All document content and structure
  - Style metadata (bold, italic, colors, fonts)
  - Semantic classifications (headers, financial text, etc.)
  - Edit history and version tracking
  - Full SQL query capabilities with JOINs
- **CSV Export**: Simple tabular format for spreadsheets

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

## Unified Export (Cmd+E)

CHONKER exports EVERYTHING into a single `.duckdb` file with multiple tables:

**Tables in your export:**
- `chonker_content` - Full text, HTML, and document structure
- `chonker_exports` - Export metadata and edit history
- `chonker_styles` - Style information (bold, italic, colors, fonts)
- `chonker_semantics` - Semantic roles (headers, financial text, etc.)

**Example queries:**
```python
import duckdb
conn = duckdb.connect("your_export.duckdb")

# Find all bold headers about revenue
bold_revenue = conn.execute("""
    SELECT c.element_text, s.style_bold, sem.semantic_role
    FROM chonker_content c
    JOIN chonker_styles s ON c.content_id = s.element_id
    JOIN chonker_semantics sem ON c.content_id = sem.element_id
    WHERE s.style_bold = true 
    AND sem.semantic_role = 'header'
    AND c.element_text LIKE '%revenue%'
""").df()

# Get all edited financial paragraphs
edited_financial = conn.execute("""
    SELECT c.element_text, e.edit_count, sem.word_count
    FROM chonker_content c
    JOIN chonker_exports e ON c.export_id = e.export_id
    JOIN chonker_semantics sem ON c.content_id = sem.element_id
    WHERE e.edit_count > 0 
    AND sem.semantic_role = 'financial_text'
""").df()
```

One file. All your data. Full SQL power.

## Recent Updates (2025-07-20)

- **Unified Export**: Single command creates both DuckDB and Arrow datasets
- **Migrated to uv**: Replaced pip with the blazing-fast uv package manager
- **Cleaned codebase**: Removed 40+ unnecessary files, keeping only essentials
- **Modern Python setup**: Added pyproject.toml for standard Python packaging
- **Simplified structure**: From 100+ files down to just 14 essential files

## License

MIT License - Feel free to use this hamster-powered technology responsibly!

---

Built with 🐹 by the CHONKER development team