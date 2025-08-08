# Chonker5-TUI

A terminal user interface (TUI) version of Chonker5 - PDF character matrix viewer and editor.

## Features

- **Split-pane interface**: PDF preview on left, character matrix editor on right
- **Vim-style navigation**: hjkl movement, modes (Normal/Insert/Visual)
- **Matrix editing**: Edit extracted character matrices in real-time
- **Copy/Cut/Paste**: Visual mode selection with clipboard operations
- **Adjustable layout**: Resize panes with Ctrl+/- 
- **Page navigation**: Arrow keys to move between PDF pages
- **Mouse support**: Click to focus panes

## Building

```bash
# Build the TUI version
cargo build --release --manifest-path Cargo-tui.toml

# Or run directly
cargo run --release --manifest-path Cargo-tui.toml
```

## Usage

### Key Bindings

#### Global
- `Tab` - Switch focus between PDF and Matrix panes
- `Ctrl-Q` - Quit application
- `?` - Show help (in Normal mode)
- `Esc` - Return to Normal mode

#### Normal Mode
- `o` - Open PDF file
- `m` - Extract character matrix from current page
- `i` - Enter Insert mode (Matrix pane only)
- `v` - Enter Visual mode (Matrix pane only)
- `←/→` - Navigate PDF pages (PDF pane focused)
- `h/j/k/l` - Move cursor (Matrix pane focused)
- `p` - Paste clipboard content

#### Insert Mode (Matrix pane)
- Type any character to insert at cursor
- `Backspace` - Delete character and move back

#### Visual Mode (Matrix pane)
- `h/j/k/l` - Extend selection
- `y` - Copy (yank) selection
- `d` - Cut (delete) selection
- `Esc` - Cancel selection

#### Layout
- `Ctrl +` - Increase PDF pane size
- `Ctrl -` - Decrease PDF pane size

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