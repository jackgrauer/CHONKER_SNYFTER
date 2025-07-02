#!/bin/bash

echo "🚀 Building CHONKER GUI with MuPDF high-performance viewer..."

# Check if mupdf-sys dependencies are available
echo "📋 Checking system dependencies..."

# Check for MuPDF development libraries
if command -v pkg-config &> /dev/null && pkg-config --exists mupdf; then
    echo "✅ MuPDF development libraries found"
    MUPDF_VERSION=$(pkg-config --modversion mupdf)
    echo "📦 MuPDF version: $MUPDF_VERSION"
else
    echo "⚠️ MuPDF development libraries not found"
    echo "📥 Installing MuPDF..."
    
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        if command -v brew &> /dev/null; then
            brew install mupdf-tools
            echo "✅ MuPDF installed via Homebrew"
        else
            echo "❌ Homebrew not found. Please install MuPDF manually:"
            echo "   brew install mupdf-tools"
            exit 1
        fi
    elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
        # Linux
        if command -v apt-get &> /dev/null; then
            sudo apt-get update
            sudo apt-get install -y libmupdf-dev mupdf-tools
        elif command -v yum &> /dev/null; then
            sudo yum install -y mupdf-devel mupdf
        else
            echo "❌ Package manager not found. Please install MuPDF development libraries manually"
            exit 1
        fi
    else
        echo "❌ Unsupported OS. Please install MuPDF development libraries manually"
        exit 1
    fi
fi

echo ""
echo "🔨 Building with MuPDF support..."

# Build with mupdf feature
cargo build --bin chonker_gui --features "gui,mupdf" --release

if [ $? -eq 0 ]; then
    echo ""
    echo "✅ Build successful!"
    echo ""
    echo "🚀 Performance comparison:"
    echo "   Standard viewer: Uses pdftoppm (external process)"
    echo "   MuPDF viewer:    Direct C library integration"
    echo ""
    echo "🎯 Expected improvements:"
    echo "   - 3-5x faster PDF rendering"
    echo "   - Lower memory usage with smart caching"
    echo "   - Instant page navigation"
    echo "   - Real-time performance monitoring"
    echo ""
    echo "🎮 To test the high-performance viewer:"
    echo "   ./target/release/chonker_gui"
    echo ""
    echo "💡 The app will automatically use MuPDF when available!"
    echo "   Look for the '🚀 MuPDF Viewer' message in the PDF panel"
else
    echo ""
    echo "❌ Build failed!"
    echo ""
    echo "🔧 Common solutions:"
    echo "   1. Ensure MuPDF development libraries are installed"
    echo "   2. Try building without mupdf feature first:"
    echo "      cargo build --bin chonker_gui --features gui --release"
    echo "   3. Check that pkg-config can find mupdf:"
    echo "      pkg-config --cflags --libs mupdf"
    exit 1
fi
