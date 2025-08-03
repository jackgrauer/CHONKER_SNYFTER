#!/bin/bash

# Simple ferrules test
PDF="/Users/jack/Downloads/DraftPlanApproval)4122l02695.pdf"
OUTPUT_DIR="/tmp/ferrules_test_output"

echo "Testing ferrules with: $PDF"

# Create clean output directory
rm -rf "$OUTPUT_DIR"
mkdir -p "$OUTPUT_DIR"

# Run ferrules
echo "Running: ferrules \"$PDF\" -o \"$OUTPUT_DIR\""
ferrules "$PDF" -o "$OUTPUT_DIR"

echo -e "\nOutput files:"
ls -la "$OUTPUT_DIR"

# Show JSON content if it exists
JSON_FILE="$OUTPUT_DIR/DraftPlanApproval)4122l02695.json"
if [ -f "$JSON_FILE" ]; then
    echo -e "\nJSON content (first 500 chars):"
    head -c 500 "$JSON_FILE"
    echo "..."
else
    echo "No JSON file found at: $JSON_FILE"
fi