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
| Simple PDF processing | < 10ms | 🚧 In Development |
| Complex document processing | 1-5s | ✅ Achieved (Python) |
| Concurrent requests | 1000+ | 🚧 Architecture ready |
| Cache hit rate | 80% | 📅 Planned |
| Database queries | < 1ms | ✅ Achieved |

## 💻 Usage Examples

### Currently Working
```bash
# Launch TUI (framework ready, features in development)
cargo run -- tui
./target/debug/chonker tui

# Extract documents with auto-routing
./target/debug/chonker extract test.pdf --tool auto

# Store in database
./target/debug/chonker extract test.pdf --tool auto --store

# Check database status
./target/debug/chonker status

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
