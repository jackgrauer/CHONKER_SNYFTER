# 🐹 CHONKER_SNYFTER v10.0 - Quick Reference Cheat Sheet

## 🚀 Build & Setup
```bash
# Build the project
cargo build

# Install Python dependencies (if using ML features)
pip install -r python/requirements.txt

# Set API key for advanced features (optional)
export ANTHROPIC_API_KEY=sk-ant-your-key-here
```

## 💻 Core Commands

### Launch TUI (Terminal User Interface)
```bash
cargo run -- tui
# OR use the binary directly:
./target/debug/chonker tui
```

### Document Processing
```bash
# Process a PDF with auto-routing (recommended)
cargo run -- extract document.pdf --tool auto

# Process and store in database
cargo run -- extract document.pdf --tool auto --store

# Force Python processing (for complex docs)
cargo run -- extract document.pdf --tool python --store

# Process multiple documents
cargo run -- extract *.pdf --tool auto --store
```

### Database Operations
```bash
# Check database status and stats
cargo run -- status

# Search documents (full-text search)
cargo run -- search "your search terms"

# List all stored documents
cargo run -- list
```

### Export Data
```bash
# Export to CSV
cargo run -- export -f csv -o output.csv

# Export to JSON
cargo run -- export -f json -o output.json

# Export to Parquet (best compression)
cargo run -- export -f parquet -o output.parquet

# Export with custom query/filter
cargo run -- export -f csv -o filtered.csv --query "search terms"
```

## 🧪 Testing & Validation

### Quick Functionality Test
```bash
# 1. Process a test document
cargo run -- extract tests/fixtures/simple.pdf --store

# 2. Check it was stored
cargo run -- status

# 3. Search for content
cargo run -- search "test"

# 4. Export the data
cargo run -- export -f csv -o test_output.csv
```

### Full Test Suite
```bash
# Run all unit tests
cargo test

# Run integration tests
cargo test --test integration

# Run load testing script
./tests/load_test.sh

# Verify Parquet compatibility with Python
python3 tests/verify_parquet.py
```

### Performance Testing
```bash
# Load test with concurrent operations
./tests/load_test.sh

# Memory usage monitoring
cargo run -- extract large_document.pdf --tool auto --store
# Check memory with: htop or Activity Monitor
```

## 📁 Test Files
```bash
# Use provided test fixtures
tests/fixtures/simple.pdf      # Basic PDF for testing
tests/fixtures/sample.pdf      # More complex document
tests/fixtures/corrupted.md    # Error handling test

# Process test files
cargo run -- extract tests/fixtures/simple.pdf --store
cargo run -- extract tests/fixtures/sample.pdf --store
```

## 🎛️ TUI Navigation
```
Tab / Shift+Tab    - Switch between tabs
Arrow Keys         - Navigate within tabs
Enter             - Select/activate item
q / Ctrl+C        - Quit

Tabs:
- Documents: Browse/delete stored docs
- Processing: View processing status
- Export: Export configuration (placeholder)
- Settings: Help and keybindings
```

## 🔧 Troubleshooting

### Common Issues
```bash
# Clean build if having issues
cargo clean && cargo build

# Check database location
ls -la chonker.db*

# Reset database (if needed)
rm -f chonker.db* && cargo run -- status

# Check Python bridge works
python3 python/chonker.py
```

### Debug Mode
```bash
# Run with debug output
RUST_LOG=debug cargo run -- extract test.pdf --store

# Verbose mode
cargo run -- extract test.pdf --store --verbose
```

## 📊 Performance Benchmarks
```bash
# Expected performance targets:
# - Memory usage: <15MB peak
# - Simple docs: <500ms processing
# - Database ops: <1ms
# - Export: ~500ms for typical datasets
# - Parquet compression: ~73% vs CSV
```

## 🐍 Python Components (Optional)
```bash
# Direct Python processing
python3 python/chonker.py

# AI-powered extraction
python3 python/snyfter.py

# Test Python bridge
python3 python/extraction_bridge.py
```

## 📈 Monitoring Commands
```bash
# Database statistics
cargo run -- status

# Recent processing activity
cargo run -- list --recent

# Export summary
cargo run -- export -f json | jq '.[] | .metadata'

# System resource usage
htop  # or Activity Monitor on macOS
```

## 🎯 One-Liner Demos
```bash
# Full pipeline demo
cargo run -- extract tests/fixtures/simple.pdf --store && cargo run -- search "test" && cargo run -- export -f parquet -o demo.parquet

# Quick TUI demo
cargo run -- tui

# Performance test
time cargo run -- extract tests/fixtures/sample.pdf --store

# Export validation
cargo run -- export -f parquet -o test.parquet && python3 tests/verify_parquet.py
```

## 🏃‍♂️ Quick Start Workflow
1. `cargo build` - Build the project
2. `cargo run -- tui` - Launch interface
3. Switch to Documents tab, see what's stored
4. `cargo run -- extract your_file.pdf --store` - Process a document
5. `cargo run -- search "keywords"` - Search content
6. `cargo run -- export -f parquet -o data.parquet` - Export data

---
**🎯 For anxiety-free document processing with live monitoring!**
