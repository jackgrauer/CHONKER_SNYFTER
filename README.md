# Chonker5-TUI

A terminal user interface (TUI) version of Chonker5 - PDF character matrix viewer and editor.

## Features

- **Three-pane interface**: PDF preview, character matrix editor, and smart layout analysis
- **PDF Image Display**: Shows actual PDF pages in terminal (requires image-capable terminal)
- **Direct editing**: Just type to edit text, no mode switching required
- **Matrix editing**: Edit extracted character matrices in real-time
- **Smart Layout**: Structural analysis of PDF documents with 's' key toggle
- **Advanced Copy/Cut/Paste**: 
  - Rectangular selection with Shift+arrows
  - Standard Ctrl+C/X/V shortcuts
  - Preserves 2D text layout
- **Search**: Find text with Ctrl+F, navigate with F3/Shift+F3
- **Export**: Save edited matrices with native file dialogs (Ctrl+S)
- **Native File Dialogs**: macOS native open/save dialogs
- **Adjustable layout**: Resize panes with Ctrl+/- 
- **Page navigation**: Arrow keys or PageUp/PageDown to move between PDF pages
- **Mouse support**: Click to focus panes and position cursor

## Building

```bash
# Build the TUI version
cargo build --release --manifest-path Cargo-tui.toml

# Or run directly
cargo run --release --manifest-path Cargo-tui.toml
```

## Usage

### Key Bindings

#### Navigation
- `Tab` - Switch focus between PDF and Matrix panes
- `Arrow Keys` - Navigate (context-sensitive)
  - In PDF pane: ← → change pages
  - In Matrix pane: ↑ ↓ ← → move cursor
- `PageUp/PageDown` - Jump 10 pages in PDF

#### Editing (Matrix pane)
- **Type any character** - Direct text input at cursor
- `Backspace` - Delete character
- `Delete` - Delete character forward
- `Enter` - Insert new line

#### Selection & Clipboard
- `Shift + Arrow Keys` - Start/extend rectangular selection
- `Ctrl+C` - Copy selection
- `Ctrl+X` - Cut selection  
- `Ctrl+V` - Paste
- `Ctrl+A` - Select all
- `Esc` - Cancel selection

#### File Operations
- `o` - Open PDF file
- `m` - Extract character matrix from current page
- `Ctrl+S` - Export matrix to text file
- `Ctrl+Shift+S` - Export with timestamp

#### Search
- `Ctrl+F` - Find text
- `F3` - Find next
- `Shift+F3` - Find previous
- `Esc` - Cancel search

#### UI Controls
- `Ctrl +` - Increase PDF pane size
- `Ctrl -` - Decrease PDF pane size
- `?` - Show help
- `Ctrl+Q` - Quit application

### Terminal Requirements

- **Minimum**: 80x24 characters
- **Recommended**: 120x40 characters or larger
- **Unicode support**: Required for proper character display

### PDF Library Setup

The TUI version uses the same PDFium library as the GUI version:

```bash
# macOS
cp ./lib/libpdfium.dylib /usr/local/lib/

# Linux
cp ./lib/libpdfium.so /usr/local/lib/

# Windows
# Place pdfium.dll in the same directory as the executable
```

## Architecture

The TUI version maintains the core functionality of Chonker5 while adapting to terminal constraints:

- **PDF Rendering**: Currently shows metadata; full image support available with `ratatui-image`
- **Character Matrix**: Same extraction logic as GUI version
- **Editing**: Simplified but functionally equivalent to GUI matrix editor
- **Performance**: Optimized for terminal rendering with virtual scrolling

## Limitations

- PDF preview is simplified (ASCII representation) without the `images` feature
- No drag-and-drop for moving text blocks (use cut/paste instead)
- File selection via text input instead of file dialog
- No Ferrules integration in base version (can be added)

## Future Enhancements

1. **Image Support**: Enable `ratatui-image` for actual PDF rendering in supported terminals
2. **Ferrules Integration**: Add Smart Layout extraction 
3. **Multiple Tabs**: Support multiple PDFs open simultaneously
4. **Search**: Find text within character matrix
5. **Export**: Save edited matrices back to text/PDF