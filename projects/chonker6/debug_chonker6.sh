#!/bin/bash

echo "=== Chonker6 Debug Info ==="
echo ""

# Check terminal environment
echo "Terminal Environment:"
echo "  TERM: $TERM"
echo "  KITTY_WINDOW_ID: $KITTY_WINDOW_ID"
echo "  TERM_PROGRAM: $TERM_PROGRAM"
echo ""

# Check if we're in Kitty
if [[ "$TERM" == *"kitty"* ]] && [[ -n "$KITTY_WINDOW_ID" ]]; then
    echo "✓ Running in Kitty terminal"
else
    echo "✗ NOT running in Kitty terminal"
    echo "  To use PDF rendering, please run this in Kitty"
fi
echo ""

# Check library
echo "PDFium Library:"
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
if [ -f "$SCRIPT_DIR/lib/libpdfium.dylib" ]; then
    echo "✓ libpdfium.dylib found at $SCRIPT_DIR/lib/"
    ls -lh "$SCRIPT_DIR/lib/libpdfium.dylib"
else
    echo "✗ libpdfium.dylib NOT FOUND"
fi
echo ""

# Test Kitty graphics
echo "Testing Kitty Graphics Protocol:"
./test_kitty_simple.sh
echo ""

# Run chonker6 with debug output
echo "=== Starting Chonker6 with Debug Output ==="
echo "Watch the terminal panel (Ctrl+T) for rendering logs"
echo ""

# Force Kitty mode for testing if needed
# export CHONKER6_FORCE_KITTY=1

# Run with library path
export DYLD_LIBRARY_PATH="$SCRIPT_DIR/lib:$DYLD_LIBRARY_PATH"
cd "$SCRIPT_DIR"
./target/release/chonker6