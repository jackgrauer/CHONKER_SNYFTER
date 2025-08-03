#!/bin/bash

# Convert PDF pages to images for layout analysis
# Usage: ./pdf_to_images.sh input.pdf output_dir

if [ $# -ne 2 ]; then
    echo "Usage: $0 input.pdf output_dir"
    exit 1
fi

INPUT_PDF="$1"
OUTPUT_DIR="$2"

if [ ! -f "$INPUT_PDF" ]; then
    echo "Error: PDF file not found: $INPUT_PDF"
    exit 1
fi

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Check if pdftoppm is available
if command -v pdftoppm &> /dev/null; then
    echo "Converting PDF to images using pdftoppm..."
    pdftoppm -png -r 300 "$INPUT_PDF" "$OUTPUT_DIR/page"
    echo "✅ Conversion complete! Images saved to $OUTPUT_DIR"
else
    echo "❌ pdftoppm not found. Install with: brew install poppler"
    exit 1
fi