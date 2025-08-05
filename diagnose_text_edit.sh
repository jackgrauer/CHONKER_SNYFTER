#!/bin/bash

echo "=== Text Edit Diagnosis Script ==="
echo
echo "This will help diagnose why text editing isn't working."
echo
echo "1. Running the application..."
echo "2. When the app opens:"
echo "   - Load a PDF file"
echo "   - Click PROCESS"
echo "   - Click on the Matrix tab"
echo "   - Click on a cell in the matrix"
echo "   - Type a character (like 'X')"
echo "   - Watch the log panel for these messages:"
echo "     🎯 Matrix view focused"
echo "     🖱️ Cell (x, y) selected"
echo "     📝 Opening text edit dialog..."
echo
echo "If you don't see ALL three messages, we know where the problem is."
echo

cargo run