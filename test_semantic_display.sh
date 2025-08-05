#!/bin/bash
# Test the semantic display widget with a sample PDF

echo "🚀 Testing Chonker5 Semantic Display Widget"
echo "==========================================="
echo
echo "1. Run: cargo run --release"
echo "2. Press [O] to open a PDF file"
echo "3. The app will automatically extract and analyze the document"
echo "4. The Semantic tab will show:"
echo "   - Document structure with fusion confidence"
echo "   - Semantic blocks in reading order"
echo "   - Tables with proper grid layout"
echo "   - Color-coded block types (Title, Heading, Table, etc.)"
echo
echo "Features demonstrated:"
echo "✅ Multi-modal fusion of PDFium text + ferrules vision"
echo "✅ Rich semantic document visualization"
echo "✅ Table structure extraction and display"
echo "✅ Reading order preservation"
echo "✅ Confidence scoring for each block"
echo
echo "Starting Chonker5..."
cargo run --release