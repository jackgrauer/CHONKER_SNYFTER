# CHONKER 5 🐹

A modern PDF viewer built with Rust using FLTK for the GUI, featuring structured data extraction with ferrules.

## Features

- **Native PDF Rendering**: Uses MuPDF for high-quality PDF display
- **Text Extraction**: Extract text content with Extractous
- **Structured Data Extraction**: Perfect layout reconstruction with ferrules
- **Split Pane Interface**: View PDF and extracted content side-by-side
- **Keyboard Shortcuts**: Cmd+O (open), ←/→ (navigate), +/- (zoom), F (fit width)
- **Zoom & Navigation**: Smooth zooming and page navigation
- **Single File**: Everything in one Rust file using rust-script

## Requirements

- Rust (with rust-script)
- MuPDF tools (mutool)
- uv (for Python package management)

## Installation

### Install Dependencies

```bash
# Install MuPDF tools
brew install mupdf-tools

# Install uv for Python packages
curl -LsSf https://astral.sh/uv/install.sh | sh

# Install ferrules
uv pip install ferrules
```

### Run Chonker5

```bash
chmod +x chonker5.rs
./chonker5.rs
```

Or with rust-script:
```bash
rust-script chonker5.rs
```

## Usage

- **Open PDF**: Click "Open" button or press Cmd+O
- **Navigate**: Use Prev/Next buttons or arrow keys
- **Zoom**: Use Zoom In/Out buttons or +/- keys
- **Fit Width**: Press F to fit PDF to window width
- **Extract Text**: Press Cmd+P or click "Extract Text" for basic text extraction
- **Structured Data**: Click "Structured Data" for perfect layout reconstruction with ferrules

## Implementation Details

This implementation uses:
- **FLTK**: Cross-platform GUI toolkit with native widgets
- **MuPDF**: High-quality PDF rendering via command-line tools
- **Extractous**: Rust-based text extraction library
- **Ferrules**: Advanced structured data extraction preserving layout
- **rust-script**: Single-file Rust applications

## Features in Detail

### PDF Rendering
- Uses MuPDF's `mutool` for high-quality rendering
- Adjustable zoom levels (25% to 400%)
- Fit-to-width functionality
- Page navigation with visual feedback

### Text Extraction
Two modes available:
1. **Basic Text**: Fast extraction of plain text content
2. **Structured Data**: Ferrules-powered extraction that preserves:
   - Tables with proper alignment
   - Multi-column layouts
   - Form structures
   - Visual positioning

### UI Features
- Split pane interface with resizable panels
- Dark theme with teal accents
- Real-time logging of operations
- Status indicators for all operations
- Keyboard shortcuts for efficiency

## Building from Source

If you want to compile it as a regular Rust binary:

```bash
# Extract dependencies from the rust-script header and create Cargo.toml
# Then compile:
rustc chonker5.rs
```

## Recent Updates

### Latest - Ferrules Integration
- **HTML Rendering**: Switched to ferrules HTML output for better multi-page support
- **Fixed Scrollbar Issue**: Removed duplicate scrollbar by using HelpView's native scrolling
- **Improved Performance**: Faster rendering using native HTML display
- **Better Table Support**: Ferrules properly handles complex table structures

## License

MIT License - Feel free to use this hamster-powered technology responsibly!

---

Built with 🐹 by the CHONKER development team
