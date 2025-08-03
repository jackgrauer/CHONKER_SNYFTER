#!/bin/bash

# Debug script to test ferrules

PDF_PATH="/tmp/ferrules_test/journal_entry-5-.pdf"
OUTPUT_DIR="/tmp/ferrules_debug"

echo "🔍 Debugging ferrules..."
echo "PDF: $PDF_PATH"
echo "Output: $OUTPUT_DIR"

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Test 1: Basic ferrules command
echo -e "\n📋 Test 1: Basic ferrules command"
ferrules "$PDF_PATH" -o "$OUTPUT_DIR" 2>&1

# Check what files were created
echo -e "\n📁 Files created:"
ls -la "$OUTPUT_DIR"

# Test 2: Try with explicit JSON output
echo -e "\n📋 Test 2: Explicit JSON output"
ferrules "$PDF_PATH" -o "$OUTPUT_DIR/test.json" 2>&1

# Check again
echo -e "\n📁 Files after test 2:"
ls -la "$OUTPUT_DIR"

# Test 3: Check ferrules help
echo -e "\n📋 Test 3: Ferrules help"
ferrules --help 2>&1 | head -20