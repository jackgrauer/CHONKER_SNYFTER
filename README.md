```
  <\___/>
  [o-·-o]  CHONKER v10.0 - The Cutest Document Processing Pipeline!
  (")~(") 
```

**Ultra-fast CLI-First system for adversarial PDF processing with spatial intelligence**

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

## Design Philosophy & Vision 🎯

### The Problem: Journalism Under Pressure

Journalists receive massive policy documents - think 300-page government reports filled with tables, statistics, and arguments buried in dense text. Under deadline pressure, they need to:

- **Extract all the data accurately**
- **Understand the arguments being made** 
- **Test and verify the claims with the data**

Current tools fail because they either just extract text (missing the data) or they use AI that might hallucinate (unacceptable for journalism).

### The Solution: A Three-Stage Workflow

#### Stage 1: Interactive Extraction
Imagine a split-screen interface:

- **Left pane:** The original PDF document
- **Right pane:** The extracted content in markdown

**Key features:**
- Click on a table in the PDF, it highlights in the markdown
- Click on text in markdown, it highlights in the PDF  
- When extraction is wrong (and it often is), click and correct it
- The system learns from corrections for better future extractions

*This is what MinerU tries to do but fails at - they show you errors but don't let you fix them.*

#### Stage 2: The Sliding Transition
Once the journalist is satisfied with extraction accuracy, they "slide" the interface:

- The PDF view slides off to the left
- The extracted database moves to the left pane
- A new analysis pane appears on the right

This creates a seamless workflow from extraction to analysis.

#### Stage 3: Data Analysis Mode
Now the interface shows:

- **Left pane:** All extracted data as queryable datasets (using Polars)
- **Right pane:** Live code editor for data analysis

The journalist can:
- Query the data directly
- Test claims made in the document
- Create visualizations
- Export findings

Crucially, if they spot something wrong in the data, they can "slide back" to extraction mode to fix it.

### Why This Matters

- **Accuracy:** No AI hallucinations - journalists see exactly what was extracted
- **Correctability:** Unlike existing tools, errors can be fixed on the spot
- **Learnability:** The system improves from corrections
- **Verifiability:** Direct path from PDF source to data analysis
- **Speed:** Optimized for deadline-driven journalism

### The Technical Innovation
This isn't just another PDF extractor. It's:

- A synchronized document viewer (PDF ↔ Markdown)
- An interactive correction system that learns
- A data analysis workbench built into the same tool
- A sliding interface that maintains context between stages

Think of it as building **Jupyter Notebooks specifically for investigative data journalism**, where the "notebook" starts with PDF extraction and flows naturally into data analysis, all while maintaining perfect traceability back to the source document.

The name **CHONKER** reflects handling these massive, chunky policy documents, while **"snyfter"** (the analysis component) helps journalists sniff out the real story in the data.

---

## Current Implementation Status ✅

This is a working foundation! The basic PDF processing pipeline is complete.
Ready to add MLX acceleration and advanced spatial features.

**For journalists fighting adversarial PDFs** 📰⚔️
