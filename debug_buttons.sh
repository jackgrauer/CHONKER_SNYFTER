#!/bin/bash

# Debug script to test the exact path logic used in chonker5.rs

PDF_PATH="/Users/jack/Downloads/DraftPlanApproval)4122l02695.pdf"
TEMP_DIR="/tmp"
FERRULES_DIR="$TEMP_DIR/chonker5_ferrules"

echo "🔍 Debugging button functionality..."
echo "PDF: $PDF_PATH"
echo "Temp dir: $TEMP_DIR"
echo "Ferrules dir: $FERRULES_DIR"

# Extract filename without extension
PDF_BASENAME=$(basename "$PDF_PATH" .pdf)
echo "PDF basename: $PDF_BASENAME"

# Apply the same transformation as the Rust code
SAFE_STEM=$(echo "$PDF_BASENAME" | sed 's/)/-/g' | sed 's/(/-/g')
echo "Safe stem: $SAFE_STEM"

RESULTS_DIR="$FERRULES_DIR/${SAFE_STEM}-results"
JSON_FILE="$RESULTS_DIR/${SAFE_STEM}.json"

echo "Expected results dir: $RESULTS_DIR"
echo "Expected JSON file: $JSON_FILE"

# Create the directory and run ferrules
mkdir -p "$FERRULES_DIR"
echo ""
echo "📋 Running ferrules..."
ferrules "$PDF_PATH" -o "$FERRULES_DIR"

echo ""
echo "📁 Checking actual structure:"
if [ -d "$RESULTS_DIR" ]; then
    echo "✅ Results directory exists: $RESULTS_DIR"
    ls -la "$RESULTS_DIR"
    
    if [ -f "$JSON_FILE" ]; then
        echo "✅ JSON file exists: $JSON_FILE"
        echo "File size: $(wc -c < "$JSON_FILE") bytes"
    else
        echo "❌ JSON file NOT found: $JSON_FILE"
        echo "Available files:"
        find "$RESULTS_DIR" -name "*.json"
    fi
else
    echo "❌ Results directory NOT found: $RESULTS_DIR"
    echo "Available directories:"
    ls -la "$FERRULES_DIR"
fi

# Cleanup
rm -rf "$FERRULES_DIR"