#!/bin/bash

# Extract a specific region from a PDF page as an image
# Usage: ./extract_table_region.sh input.pdf page_num x y width height output.png

if [ $# -ne 7 ]; then
    echo "Usage: $0 input.pdf page_num x y width height output.png"
    echo "Example: $0 document.pdf 1 100 200 400 150 table.png"
    exit 1
fi

PDF="$1"
PAGE="$2"
X="$3"
Y="$4"
WIDTH="$5"
HEIGHT="$6"
OUTPUT="$7"

# First extract the page as a high-res image
TEMP_IMG="/tmp/pdf_page_${PAGE}.png"
pdftoppm -png -r 300 -f "$PAGE" -l "$PAGE" "$PDF" > "$TEMP_IMG"

# Then crop the specific region
# Note: ImageMagick uses top-left origin, PDFs use bottom-left
# So we need to convert coordinates
convert "$TEMP_IMG" -crop "${WIDTH}x${HEIGHT}+${X}+${Y}" "$OUTPUT"

echo "Extracted region to $OUTPUT"

# Clean up
rm -f "$TEMP_IMG"