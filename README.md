# 🐹 CHONKER v10.0 - Spatial Intelligence Document Chunker

**Ultra-fast Rust TUI for adversarial PDF processing with Apple MLX acceleration**

## Current Status ✅

- **Working PDF processor** - Real text extraction from PDFs
- **Native file picker** - macOS file dialog integration
- **Terminal UI** - Clean ratatui interface with navigation
- **Basic chunking** - Splits PDFs into manageable chunks
- **Modular architecture** - Separate app/ui/processing/database modules
- **SQLite ready** - Database support implemented (not connected yet)

## How to Run

```bash
cargo run
```

**Controls:**
- `Tab` - Navigate between panes
- `Enter` - Open file picker (File Selection) or Process (Action)
- `Space` - Toggle OCR options
- `Up/Down` - Navigate chunks in preview
- `q` or `Esc` - Quit

## Next Steps 🚀

### Phase 1: Core Features
- [ ] Connect SQLite database for persistence
- [ ] Add PDF page visualization in viewer pane
- [ ] Implement OCR for scanned PDFs
- [ ] Enhanced chunking with spatial intelligence

### Phase 2: MLX Integration
- [ ] Apple MLX acceleration for OCR
- [ ] Spatial layout analysis
- [ ] Formula detection with ML
- [ ] Table structure recognition

### Phase 3: Advanced Features
- [ ] Export to markdown/text
- [ ] Search across processed documents
- [ ] Batch processing
- [ ] SNYFTER companion app for retrieval

## Architecture

```
src/
├── main.rs        # Terminal setup & event loop
├── app.rs         # Core application state & logic
├── ui.rs          # TUI layout & rendering
├── processing.rs  # PDF processing & chunking
└── database.rs    # SQLite storage (ready)
```

## Dependencies

- `ratatui` - Terminal UI framework
- `pdf-extract` - PDF text extraction
- `rfd` - Native file dialogs
- `rusqlite` - SQLite database
- `crossterm` - Terminal control

## Notes

This is a working foundation! The basic PDF processing pipeline is complete.
Ready to add MLX acceleration and advanced spatial features.

**For journalists fighting adversarial PDFs** 📰⚔️
