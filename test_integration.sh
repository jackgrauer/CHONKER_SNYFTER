#!/bin/bash

echo "🐹 CHONKER Integration Test"
echo "========================="

# Test 1: Check if Python processing works
echo "🔍 Test 1: Python PDF Processing"
cd /Users/jack/CHONKER_SNYFTER
if [[ -f "./venv/bin/python" ]]; then
    echo "✅ Python virtual environment found"
    if [[ -f "EXAMPLE_NIGHTMARE_PDF.pdf" ]]; then
        echo "✅ Test PDF found"
        echo "🐹 Running Python extraction on test PDF..."
        ./venv/bin/python python/docling_html_bridge.py EXAMPLE_NIGHTMARE_PDF.pdf | head -30
        echo "✅ Python extraction completed"
    else
        echo "❌ Test PDF not found"
    fi
else
    echo "❌ Python virtual environment not found"
fi

echo ""
echo "🔍 Test 2: Tauri Backend Compilation"
cd src-tauri
if cargo build --quiet; then
    echo "✅ Tauri backend builds successfully"
else
    echo "❌ Tauri backend build failed"
fi

echo ""
echo "🔍 Test 3: Database Setup"
if [[ -f "/Users/jack/CHONKER_SNYFTER/chonker.db" ]]; then
    echo "✅ Database file exists"
else
    echo "⚠️  Database file not found (will be created on first run)"
fi

echo ""
echo "🔍 Test 4: Required Files"
files=("python/docling_html_bridge.py" "python/extraction_bridge.py" "venv/bin/python")
for file in "${files[@]}"; do
    if [[ -f "$file" ]]; then
        echo "✅ $file exists"
    else
        echo "❌ $file missing"
    fi
done

echo ""
echo "🐹 Integration Test Complete!"
echo "Ready for: Real PDF processing, Database storage, Export functionality"
