# 🐹 CHONKER & 🐁 SNYFTER - Enhanced Edition

## The Ultimate Document Processing Duo!

CHONKER & SNYFTER have been supercharged with incredible new features that make document processing a breeze!

## 🚀 What's New in the Enhanced Version

### 🎨 Modern UI/UX
- **Dark Theme**: Professional dark theme that's easy on the eyes
- **Smooth Animations**: Fade-in/fade-out effects for mode transitions
- **Enhanced Visual Feedback**: Real-time status updates and progress bars
- **Responsive Layout**: Resizable panels and adaptive interface

### 🧠 AI-Powered Features (with Instructor)
- **Structured Data Extraction**: Uses Pydantic models for consistent data structure
- **Smart Document Analysis**: AI-enhanced chunk classification
- **Confidence Scoring**: Each chunk gets a confidence score
- **Error Recovery**: Intelligent error handling with fallback options

### 📊 Advanced Processing
- **Batch Processing**: Process multiple PDFs at once
- **Progress Tracking**: Real-time progress for each document
- **Multi-format Export**: Export to JSON, CSV, or Markdown
- **Full-Text Search**: Lightning-fast search with SQLite FTS5

### 🛡️ Robust Error Handling
- **Graceful Degradation**: App works even without all dependencies
- **Asset Fallbacks**: Creates emoji placeholders if images missing
- **Thread Safety**: Proper threading for non-blocking UI
- **Comprehensive Logging**: Detailed logs with color-coded messages

### 📦 New Features
1. **Export Functionality**
   - Export documents in multiple formats
   - Customizable export options
   - Include/exclude metadata, chunks, HTML, Markdown

2. **Enhanced Search**
   - Search by content type (text, heading, list, table)
   - Full-text search with snippet highlights
   - Real-time result updates

3. **Document Details View**
   - Comprehensive document viewer
   - Chunk-by-chunk analysis
   - Export individual documents

4. **Batch Operations**
   - Clean PDFs (remove annotations)
   - Compress PDFs
   - OCR support (when available)
   - Bulk extraction and cataloging

5. **Statistics Dashboard**
   - Total documents processed
   - Total chunks extracted
   - Last update timestamp
   - Archive overview

## 🏃 Quick Start

### Run the Enhanced Version
```bash
cd /Users/jack/CHONKER_SNYFTER
source /Users/jack/chonksnyft-env/bin/activate
python run_enhanced.py
```

### Or Run Directly
```bash
python chonker_snyfter_enhanced.py
```

## 🔧 Configuration

### Optional: Set OpenAI API Key for AI Features
```bash
export OPENAI_API_KEY="your-api-key-here"
```

## 🎮 How to Use

### CHONKER Mode (PDF Processing)
1. Click "📂 Load PDF" or drag & drop a PDF
2. Click "🐹 CHONK IT!" to process
3. View extracted chunks in the table
4. See Markdown and HTML versions in tabs
5. Document automatically saved to SNYFTER's archive

### SNYFTER Mode (Archive Search)
1. Switch to SNYFTER mode using the dropdown
2. Enter search terms in the search box
3. Filter by content type if needed
4. Double-click results to see details
5. Export documents in various formats

### Batch Processing
1. Click "📦 Batch Process" in CHONKER mode
2. Add files or entire folders
3. Select operations to perform
4. Choose output directory
5. Click OK to start processing

## 🏗️ Architecture Improvements

### Code Structure
- **Pydantic Models**: Type-safe data structures
- **Instructor Integration**: Structured AI outputs
- **Enhanced Threading**: Better concurrency handling
- **Modular Design**: Cleaner separation of concerns

### Database Enhancements
- **Enhanced Schema**: More metadata fields
- **FTS5 Search**: Full-text search capability
- **Better Indexing**: Improved query performance
- **Export Support**: Multiple export formats

### UI Components
- **AnimatedLabel**: Smooth transitions
- **ModernPushButton**: Styled buttons with hover effects
- **Enhanced Dialogs**: Better user interaction
- **Progress Indicators**: Visual feedback

## 🐛 Troubleshooting

### Missing Dependencies
The launcher will automatically detect and offer to install missing dependencies.

### No Emoji Images
The app will create colored circle placeholders if emoji PNGs are missing.

### Database Issues
Delete `snyfter_archives.db` to start fresh (backup first!).

## 🎯 Future Enhancements
- Cloud storage integration
- PDF annotation support
- Advanced OCR capabilities
- Multi-language support
- Collaborative features
- API endpoints for automation

## 🙏 Credits
Enhanced with love by combining:
- CHONKER's enthusiastic PDF munching
- SNYFTER's meticulous cataloging
- Instructor's structured thinking
- PyQt6's powerful UI capabilities
- Your feedback and ideas!

---

**Enjoy the enhanced CHONKER & SNYFTER experience!** 🎉