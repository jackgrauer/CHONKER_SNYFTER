# 🐹 CHONKER & 🐁 SNYFTER Quick Start Guide

## Launch the App
```bash
python chonker_snyfter.py
```

## Key Features

### 🐹 CHONKER MODE (PDF Processing)

1. **Feed PDF** - Load a PDF to process
2. **PDF Tools** - Comprehensive PDF manipulation:
   - 🔄 Rotate pages (90°, 180°, reset to 0°)
   - ✂️ Split into individual pages
   - 🔗 Merge multiple PDFs
   - 📄 Extract specific pages
   - ➕ Insert pages from another PDF
   - 🗑️ Delete pages
   - 🧹 Clean PDF (remove annotations)
   - 📦 Compress PDF
   - 🔍 Optimize for extraction
   - 💾 Save/Revert changes
3. **Batch Process** - Process multiple PDFs at once
4. **Digest & Extract** - Extract content with Docling
5. **Pass to SNYFTER** - Archive in database

### 🐁 SNYFTER MODE (Database & Research)

1. **Search Archives** - Find documents by keyword
2. **View Recent** - Browse recently processed documents
3. **Add Research Notes** - Annotate documents

### 🎯 Pro Tips

- **Press TAB to switch between CHONKER and SNYFTER modes!**
- Click on PDF to highlight corresponding extracted text
- Click on extracted chunks to jump to PDF location
- All PDF edits are preview-only until you save
- Batch processing creates new files with "_processed" suffix

### 🎨 Character Personalities

**CHONKER says:**
- "🐹 *nom nom nom* Processing..."
- "🐹 This PDF is too scuzzy!"
- "🐹 *burp* All done!"

**SNYFTER says:**
- "🐁 *adjusts tiny glasses* Welcome to the archive."
- "🐁 *shuffles index cards* Searching..."
- "🐁 Filed successfully!"

## Troubleshooting

- **Missing emojis?** Check that `assets/emojis/` contains the PNG files
- **Dependencies error?** Run: `pip install docling PyMuPDF PyQt6`
- **Database issues?** Delete `snyfter_archives.db` to start fresh

---
*Built with real Android 7.1 emojis embedded as images!*
*$200/month worthy solution achieved! 💰*