# Chonker5 🐹

A minimal PDF viewer built with Rust using minifb for direct framebuffer rendering.

## Features

- **Direct pixel buffer rendering** - Full control over every pixel
- **PDF rendering with PDFium** - High-quality PDF rendering
- **Pan and zoom** - Click and drag to pan, zoom in/out controls
- **Keyboard shortcuts** - O (open), ←/→ (navigate), +/- (zoom)
- **Single file** - Everything in one Rust file using rust-script

## Requirements

- Rust (with rust-script)
- PDFium library

## Installation

### Install PDFium

Since homebrew doesn't have a pdfium formula, you can either:

1. Download prebuilt binaries from https://github.com/bblanchon/pdfium-binaries/releases
2. Or build from source

For macOS (Apple Silicon):
```bash
# Download the prebuilt binary
curl -L https://github.com/bblanchon/pdfium-binaries/releases/latest/download/pdfium-mac-arm64.tgz -o pdfium.tgz
tar -xzf pdfium.tgz
# Move to a location and set PDFIUM_DYNAMIC_LIB_PATH
sudo mkdir -p /usr/local/lib/pdfium
sudo cp lib/* /usr/local/lib/pdfium/
export PDFIUM_DYNAMIC_LIB_PATH=/usr/local/lib/pdfium
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

- **Open PDF**: Click "Open" button or press 'O'
- **Navigate**: Use Prev/Next buttons or arrow keys
- **Zoom**: Use Zoom In/Out buttons or +/- keys
- **Pan**: Click and drag on the PDF
- **Exit**: Press Escape

## Implementation Details

This implementation uses:
- **minifb**: Direct window and framebuffer access
- **pdfium-render**: PDF rendering via Google's PDFium
- **rfd**: Native file dialogs
- **Manual UI rendering**: All UI elements are drawn pixel by pixel

The entire application is self-contained in a single Rust file, making it easy to understand and modify.

## Building from Source

If you want to compile it as a regular Rust binary:

```bash
# Extract dependencies from the rust-script header and create Cargo.toml
# Then:
rustc chonker5.rs
```

## License

MIT