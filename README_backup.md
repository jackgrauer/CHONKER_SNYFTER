# CHONKER_SNYFTER v10.0 - Hybrid PDF Processing Pipeline

![Development Status](https://img.shields.io/badge/status-alpha-orange)
![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)
![Python](https://img.shields.io/badge/python-3.8%2B-blue)

```
  \___/
  [o-·-o]  🐹 CHONKER - The Cutest Document Processing Pipeline!
  (")~(") 
```

## Version Note

**CHONKER_SNYFTER v10.0 is a hybrid Rust-Python application with intelligent document routing and a modern terminal interface.**

## 🚀 Features

- **⚡ Hybrid Architecture**: Sub-10ms fast path for 90% of documents + ML path for complex cases
- **🧠 Intelligent Routing**: Automatic complexity analysis to choose optimal processing path
- **💾 Database Integration**: SQLite with full-text search and export capabilities
- **🖥️ CLI + TUI**: Command-line interface and terminal user interface
- **📊 Analysis Ready**: Export to CSV/JSON/Parquet for data analysis
- **🐍 Python Bridge**: Seamless integration with ML tools (Docling, Magic-PDF)

## 📊 Current Implementation Status

### ✅ Completed
- Basic Rust CLI structure with clap
- TUI framework with ratatui
- SQLite database integration
- Python bridge for complex document processing
- Command routing system
- Binary builds to `./target/debug/chonker`

### 🚧 In Progress
- Fast path PDF processing with pdfium-render
- Performance optimization for sub-10ms processing
- Full-text search implementation
- Parquet export functionality
- Redis caching layer

### 📅 Planned
- Batch processing orchestration
- Advanced complexity scoring algorithm
- Multi-threaded document processing
- Web API interface

## 📁 Project Structure
```
CHONKER_SNYFTER/
├── Cargo.toml              # Rust project configuration
├── src/                    # Rust source code
│   ├── main.rs            # CLI entry point
│   ├── lib.rs             # Library root
│   ├── commands/          # CLI command implementations
│   ├── tui/               # Terminal UI modules
│   └── database/          # SQLite integration
├── target/                 # Rust build artifacts
│   └── debug/
│       └── chonker        # Compiled binary
├── python/                 # Python components
│   ├── chonker.py         # Complex document processing
│   └── snyfter.py         # AI extraction pipeline
├── requirements.txt        # Python dependencies
└── README.md              # This file
```

## 🚀 Quick Start

### Using Pre-built Binary
```bash
# If you have the compiled binary
./target/debug/chonker tui
```

### Building from Source
```bash
# Clone and build
git clone https://github.com/jackgrauer/CHONKER_SNYFTER
cd CHONKER_SNYFTER
cargo build
cargo run -- tui
```

### Python Components Setup
```bash
pip install -r requirements.txt
export ANTHROPIC_API_KEY=sk-ant-your-key-here
```

## 🏗️ Architecture Overview

CHONKER_SNYFTER uses a hybrid Rust-Python architecture:

1. **Rust Core** (./target/debug/chonker)
   - CLI interface and argument parsing
   - Document routing based on complexity
   - SQLite database management
   - TUI for interactive processing
   - Fast path for simple PDFs (when complete)

2. **Python ML Pipeline** (python/)
   - Docling integration for complex documents
   - AI-powered extraction with Anthropic
   - Advanced table and layout recognition
   - Fallback processing for unsupported formats

3. **Routing Logic**
   - Simple PDFs → Rust fast path (in development)
   - Complex documents → Python ML pipeline
   - Automatic fallback on processing errors

## ⚡ Performance Goals

| Metric | Target | Current Status |
|--------|--------|----------------|
| Simple PDF processing |  10ms | 🚧 In Development |
| Complex document processing | 1-5s | ✅ Achieved (Python) |
| Concurrent requests | 1000+ | 🚧 Architecture ready |
| Cache hit rate | 80% | 📅 Planned |
| Database queries |  1ms | ✅ Achieved |

## 💻 Usage Examples

### Currently Working
```bash
# Launch TUI (framework ready, features in development)
cargo run -- tui
./target/debug/chonker tui

# Python processing (fully functional)
python python/chonker.py
python python/snyfter.py
```

### Coming Soon
```bash
# Fast PDF extraction
cargo run -- extract simple.pdf --tool rust --store

# Batch processing
cargo run -- batch process ./documents/
```

## 🛠️ Dependencies

- Rust 1.70+
- Python 3.8+ (for ML path)
- SQLite
- Optional: Redis for caching

## 🎯 Perfect For

- **Investigative Journalism**: Process massive policy documents under deadline pressure
- **Document Analysis**: Extract and verify data from complex PDFs
- **Batch Processing**: Handle large document collections efficiently
- **Research**: Maintain perfect traceability from source to analysis

## 🤝 Contributing

The Rust implementation is actively being developed. Key areas needing help:
- Implementing pdfium-render integration for fast path
- Optimizing routing algorithm
- Adding comprehensive tests
- Improving TUI features

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

---

**🎯 Built for anxiety-free document processing with live monitoring and intelligent data extraction.**

# CHONKER_SNYFTER v10.0 - Hybrid PDF Processing Pipeline

![Development Status](https://img.shields.io/badge/status-alpha-orange)
![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)
![Python](https://img.shields.io/badge/python-3.8%2B-blue)

```
  \___/>
  [o-·-o]  🐹 CHONKER - The Cutest Document Processing Pipeline!
  (")~(") 
```

## Version Note

**CHONKER_SNYFTER v10.0 is a hybrid Rust-Python application with intelligent document routing and a modern terminal interface.**

## 🚀 Features

- **⚡ Hybrid Architecture**: Sub-10ms fast path for 90% of documents + ML path for complex cases
- **🧠 Intelligent Routing**: Automatic complexity analysis to choose optimal processing path
- **💾 Database Integration**: SQLite with full-text search and export capabilities
- **🖥️ CLI + TUI**: Command-line interface and terminal user interface
- **📊 Analysis Ready**: Export to CSV/JSON/Parquet for data analysis
- **🐍 Python Bridge**: Seamless integration with ML tools (Docling, Magic-PDF)

## 📊 Current Implementation Status

### ✅ Completed
- Basic Rust CLI structure with clap
- TUI framework with ratatui
- SQLite database integration
- Python bridge for complex document processing
- Command routing system
- Binary builds to `./target/debug/chonker`

### 🚧 In Progress
- Fast path PDF processing with pdfium-render
- Performance optimization for sub-10ms processing
- Full-text search implementation
- Parquet export functionality
- Redis caching layer

### 📅 Planned
- Batch processing orchestration
- Advanced complexity scoring algorithm
- Multi-threaded document processing
- Web API interface

## 📁 Project Structure
```
CHONKER_SNYFTER/
├── Cargo.toml              # Rust project configuration
├── src/                    # Rust source code
│   ├── main.rs            # CLI entry point
│   ├── lib.rs             # Library root
│   ├── commands/          # CLI command implementations
│   ├── tui/               # Terminal UI modules
│   └── database/          # SQLite integration
├── target/                 # Rust build artifacts
│   └── debug/
│       └── chonker        # Compiled binary
├── python/                 # Python components
│   ├── chonker.py         # Complex document processing
│   └── snyfter.py         # AI extraction pipeline
├── requirements.txt        # Python dependencies
└── README.md              # This file
```

## 🚀 Quick Start

### Using Pre-built Binary
```bash
# If you have the compiled binary
./target/debug/chonker tui
```
### Building from Source
```bash
# Clone and build
git clone https://github.com/jackgrauer/CHONKER_SNYFTER
cd CHONKER_SNYFTER
cargo build
cargo run -- tui
```
### Python Components Setup
```bash
pip install -r requirements.txt
export ANTHROPIC_API_KEY=sk-ant-your-key-here
```

## 🏗️ Architecture

**Fast Path (Rust Native)**:
- `pdfium-render` for PDF parsing
- Native text extraction
- Basic layout detection
- Target: 1-5ms per document

**Complex Path (Python ML)**:
- Advanced document understanding
- Table structure recognition
- Multi-column layout analysis
- Target: 1-5 seconds per document

**Smart Routing**:
- File size analysis
- Page count estimation
- Layout complexity scoring
- Automatic path selection

## 📊 Performance Goals

✅ 90% of documents processed in < 10ms  
✅ No regression in extraction quality  
✅ Seamless fallback for complex documents  
✅ Cache reduces repeated processing by 80%  
✅ System handles 1000+ concurrent requests  

## 🎯 Perfect For

- **Investigative Journalism**: Process massive policy documents under deadline pressure
- **Document Analysis**: Extract and verify data from complex PDFs
- **Batch Processing**: Handle large document collections efficiently
- **Research**: Maintain perfect traceability from source to analysis

## 🛠️ Dependencies

- Rust 1.70+
- Python 3.8+ (for ML path)
- SQLite
- Optional: Redis for caching

Built with investigative journalism in mind - accuracy, speed, and verifiability above all else.
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
