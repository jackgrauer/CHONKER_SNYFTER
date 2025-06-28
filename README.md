# CHONKER v10.0 - Hybrid PDF Processing Pipeline

CHONKER, the cutest document processing pipeline, aims to deliver fast and accurate PDF processing with an intuitive interface and modular architecture.

## Features

- **Hybrid Architecture**: Combines Rust and Python for efficient PDF processing and understanding.
- **Intelligent Routing**: Analyzes complexity to choose the best processing path.
- **Database Integration**: Stores documents and chunks using SQLite for quick retrieval.
- **Extensible CLI and TUI**: Provides a command-line and terminal user interface for ease of use.
- **Data Analysis Ready**: Support for converting extracted content to markdown and further data analysis.

## Usage

1. **Extract Text**:
   ```bash
   cargo run -- extract path/to/document.pdf --tool auto
   ```

2. **Launch TUI**:
   ```bash
   cargo run -- tui
   ```

3. **Check Database Status**:
   ```bash
   cargo run -- status
   ```

## Design Philosophy

CHONKER is built with investigative journalism in mind, providing:

- **Accuracy and Transparency**: No AI hallucinations; what you see is what you extracted.
- **Correctability**: Easily correct extraction errors within the interface.
- **Learnability**: System adapts to user corrections for improved future extractions.
- **Perfect for Deadline-driven Workflows**: Fast processing paths ensure quick results.

### Innovation

CHONKER is more than just a PDF extractor. It’s:

- A synchronized document viewer supporting PDF to Markdown conversion.
- An interactive system that learns from user feedback.
- An integrated workbench for data analysis.

Think Jupyter Notebooks, but starting with PDF extraction, flowing into data analysis, maintaining perfect traceability to the original document.
cd chonker-snyfter

# Install dependencies
pip install -r requirements.txt

# Set your API key for SNYFTER
export ANTHROPIC_API_KEY=sk-ant-your-key-here
```

### Basic Usage

1. **Process a document with CHONKER:**
   ```bash
   python chonker.py
   # Then: load document.pdf
   ```

2. **Extract data with SNYFTER:**
   ```bash
   python snyfter.py
   # Then: load → classify → extract → export
   ```

## 📋 Features

### 🐹 CHONKER v6.0
- **Live Monitoring** - Real-time progress with anxiety-reducing heartbeat display
- **Smart Document Processing** - Docling integration with graceful fallbacks
- **Intelligent Chunking** - Respects word boundaries, optimized for AI processing
- **Entity Extraction** - 8 robust patterns (emails, phones, chemicals, etc.)
- **Keep-Awake System** - Prevents computer sleep during long processing
- **Cross-Platform** - Works on macOS, Windows, and Linux
- **Database Integration** - Optional DuckDB storage and search

### 🐭 SNYFTER v9.1
- **Adaptive Schema Discovery** - AI learns document structure as it processes
- **Two-Pass AI Processing** - Classification → Extraction pipeline
- **Multiple Export Formats** - CSV, Excel, JSON with auto-generated loading scripts
- **Step-by-Step Interface** - Build extraction pipeline incrementally
- **Custom Configuration** - Tailored extraction rules and focus areas
- **Data Type Detection** - Environmental, financial, tabular data recognition

## 🔧 Detailed Usage

### CHONKER Commands

| Command | Description |
|---------|-------------|
| `load` | Show available documents or load specific file |
| `load <filename>` | Process document with live monitoring |
| `list` | Show created chunks with previews |
| `show <n>` | Open specific chunk in editor |
| `search <term>` | Search entities across chunks |
| `info` | Display document processing summary |
| `export` | Export chunks for SNYFTER integration |
| `help` | Show all commands |

### SNYFTER Pipeline

1. **Load Chunks** (`load`)
   - Automatically finds CHONKER output
   - Supports loading specific chunks or ranges
   - Preview functionality to inspect content

2. **Classify Content** (`classify`)
   - AI-powered content type discovery
   - Adaptive schema that learns document patterns
   - Confidence scoring and reasoning

3. **Extract Data** (`extract`)
   - Structured data extraction based on classifications
   - Environmental, financial, and tabular data support
   - Automatic pattern recognition

4. **Configure Rules** (`config`) - Optional
   - Custom extraction instructions
   - Priority entity selection
   - Output format preferences

5. **Export Results** (`export`)
   - Python-ready datasets (CSV/Excel/JSON)
   - Auto-generated loading scripts
   - Summary reports

## 📁 Project Structure

```
project/
├── chonker.py              # Main CHONKER application
├── snyfter.py              # Main SNYFTER application
├── requirements.txt        # Python dependencies
├── README.md              # This file
├── saved_chonker_chunks/  # CHONKER output directory
│   ├── chunk_1.txt
│   ├── chunk_2.txt
│   └── ...
└── snyfter_output/        # SNYFTER export directory
    └── export_YYYYMMDD_HHMMSS/
        ├── environmental_data.csv
        ├── load_datasets.py
        └── extraction_summary.txt
```

## 🔑 API Key Setup

SNYFTER requires an Anthropic API key for AI classification and extraction:

1. Get your API key: https://console.anthropic.com/
2. Set environment variable:
   ```bash
   # Linux/macOS
   export ANTHROPIC_API_KEY=sk-ant-your-key-here
   
   # Windows
   set ANTHROPIC_API_KEY=sk-ant-your-key-here
   ```
3. Test with: `python snyfter.py` then `apikey`

## 🚨 Troubleshooting

### Common Issues

**CHONKER:**
- **Docling slow/hanging** → Press Ctrl+C to use fallback processing
- **No chunks found** → Check file permissions and supported formats
- **Memory issues** → Process smaller files or increase system RAM

**SNYFTER:**
- **API key errors** → Run `apikey` command to test configuration
- **No chunks found** → Ensure CHONKER has been run first
- **Classification fails** → Check internet connection and API key validity

### File Format Support

**CHONKER Supported Formats:**
- PDF (Docling + PyPDF2 fallback)
- DOCX (Docling)
- TXT, MD (native)

**Output Formats:**
- CHONKER: Text chunks in `saved_chonker_chunks/`
- SNYFTER: CSV, Excel, JSON with loading scripts

## 🔧 Advanced Configuration

### Environment Variables

```bash
# Custom chunk output directory
export CHONKER_OUTPUT_DIR=/path/to/chunks

# API configuration
export ANTHROPIC_API_KEY=sk-ant-your-key
```

### Custom Extraction Patterns

CHONKER includes these entity patterns by default:
- Email addresses
- Phone numbers
- Dates
- Sample IDs
- Chemical names
- Concentrations
- Numbers

Add custom patterns by modifying the `SimpleEntityExtractor` class.

## 🤝 Integration Workflow

1. **Document Processing**
   ```bash
   python chonker.py
   > load environmental_report.pdf
   ```

2. **Data Extraction**
   ```bash
   python snyfter.py
   > load
   > classify
   > extract
   > export csv
   ```

3. **Use Extracted Data**
   ```python
   # Auto-generated by SNYFTER
   exec(open('snyfter_output/export_*/load_datasets.py').read())
   
   # Your data is now available
   environmental_data.head()
   ```

## 📊 Example Output

**CHONKER Processing:**
- Input: `environmental_report.pdf` (2.3 MB)
- Output: 15 chunks, 127 entities found
- Processing time: 23.4s with live monitoring

**SNYFTER Extraction:**
- Discovered data types: environmental_lab_results, monitoring_well_coordinates
- Extracted datasets: environmental_data (156 rows × 6 columns)
- Export: CSV + loading script + summary

## 🐛 Development

### Running Tests
```bash
# Test CHONKER processing
python chonker.py
> load test_document.pdf

# Test SNYFTER pipeline
python snyfter.py
> apikey  # Verify API setup
> load
> status  # Check pipeline status
```

### Contributing
1. Fork the repository
2. Create feature branch
3. Test with sample documents
4. Submit pull request

## 📄 License

[Specify your license here]

## 🆘 Support

- **Issues**: Create GitHub issue with error logs
- **API Problems**: Check Anthropic console and billing
- **Performance**: Monitor system resources during processing

---

**🎯 Built for anxiety-free document processing with live monitoring and intelligent data extraction.**
>>>>>>> bc336c5a2d5c61d9d6676f7e7652451fb76fbbbc
