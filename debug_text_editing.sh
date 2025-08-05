#!/bin/bash

echo "=== Debugging Text Editing Feature ==="
echo
echo "This script will help debug why text editing isn't working."
echo
echo "Test Steps:"
echo "1. Run the application"
echo "2. Load a PDF file"  
echo "3. Click 'PROCESS' to generate character matrix"
echo "4. Click on the 'Matrix' tab if not already selected"
echo "5. Click on any cell in the matrix to select it"
echo "6. Watch the console output for debug messages"
echo "7. Try these actions:"
echo "   - Type any character (should open edit dialog)"
echo "   - Press Enter (should open edit dialog with current cell content)"
echo "   - In the dialog, type and press Enter or click Apply"
echo
echo "Starting application with debug output..."

RUST_LOG=debug cargo run 2>&1 | grep -E "(Cell|Typed|Enter|edit|dialog|focus|keyboard|Text input|selected)" | sed 's/^.*chonker5/chonker5/'