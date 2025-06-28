#!/bin/bash
# CHONKER Development Workflow Script
# Quick commands for development and testing

set -e

CHONKER_BIN="./target/debug/chonker"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

log() {
    echo -e "${BLUE}[CHONKER]${NC} $1"
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

usage() {
    echo "  <\___/>"
    echo "  [o-·-o]  CHONKER Development Workflow"
    echo "  (\")~(\") "
    echo ""
    echo "Commands:"
    echo "  build                 Build the project"
    echo "  test                  Run tests"
    echo "  clean                 Clean build artifacts"
    echo "  demo                  Run demo extraction"
    echo "  tui                   Launch TUI"
    echo "  status                Show database status"
    echo "  benchmark FILE        Benchmark extraction on file"
    echo "  install-python        Install Python dependencies"
    echo ""
    echo "Usage: $0 <command> [args]"
}

case "${1:-help}" in
    build)
        log "🔨 Building CHONKER..."
        cargo build
        success "Build completed"
        ;;
    
    test)
        log "🧪 Running tests..."
        cargo test
        success "Tests completed"
        ;;
    
    clean)
        log "🧹 Cleaning build artifacts..."
        cargo clean
        rm -rf processed/ reports/ *.md *.csv *.json *.parquet
        success "Clean completed"
        ;;
    
    demo)
        log "🎬 Running demo extraction..."
        if [[ -f "/Users/jack/Documents/1.pdf" ]]; then
            time $CHONKER_BIN extract "/Users/jack/Documents/1.pdf" --output demo.md --store
            success "Demo completed - check demo.md"
        else
            echo "No demo PDF found at /Users/jack/Documents/1.pdf"
            echo "Usage: $0 benchmark <path-to-pdf>"
        fi
        ;;
    
    tui)
        log "🖥️  Launching TUI..."
        $CHONKER_BIN tui
        ;;
    
    status)
        log "📊 Database status..."
        $CHONKER_BIN status
        ;;
    
    benchmark)
        if [[ -z "$2" ]]; then
            echo "Usage: $0 benchmark <pdf-file>"
            exit 1
        fi
        
        log "⏱️  Benchmarking extraction on: $2"
        time $CHONKER_BIN extract "$2" --output benchmark.md
        
        # Show stats
        echo ""
        echo "📊 Benchmark Results:"
        echo "Input file: $2"
        echo "Output file: benchmark.md"
        echo "File size: $(wc -c < "$2") bytes"
        echo "Output size: $(wc -c < benchmark.md) bytes"
        echo "Word count: $(wc -w < benchmark.md) words"
        ;;
    
    install-python)
        log "🐍 Installing Python dependencies..."
        cd python
        pip3 install -r requirements.txt
        success "Python dependencies installed"
        ;;
    
    help|--help|-h)
        usage
        ;;
    
    *)
        echo "Unknown command: $1"
        usage
        exit 1
        ;;
esac
