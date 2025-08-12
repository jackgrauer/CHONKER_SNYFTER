#!/bin/bash

echo "=== Kitty Graphics Test ==="
echo "This test must be run directly in a Kitty terminal"
echo ""

# Check if we're in Kitty
if [[ "$TERM" == *"kitty"* ]] || [[ -n "$KITTY_WINDOW_ID" ]]; then
    echo "✓ Running in Kitty terminal"
else
    echo "❌ Not in Kitty terminal"
    echo "Please run this script directly in Kitty"
    exit 1
fi

# Test 1: Simple escape sequence
echo "Test 1: Basic escape sequence test"
printf "\x1b_Ga=d\x1b\\"  # Clear images
sleep 0.1

# Test 2: Kitty icat test
echo "Test 2: Using Kitty's built-in icat"
if command -v kitty &> /dev/null; then
    # Create a test image
    echo "Creating test image..."
    convert -size 50x50 xc:red /tmp/test_red.png 2>/dev/null || {
        echo "ImageMagick not available, using alternative method"
        # Use our Rust test program
        echo "Using Rust test program..."
        DYLD_LIBRARY_PATH=./lib ./target/release/test-kitty
        echo ""
        echo "=== Testing chonker6 with PDF ==="
        echo "To test PDF display:"
        echo "1. Find a PDF file"
        echo "2. Run: DYLD_LIBRARY_PATH=./lib ./target/release/chonker6"
        echo "3. Press Ctrl+O to open a PDF"
        echo "4. The PDF should display as an image in the terminal"
        echo ""
        echo "If you see escape sequences printed as text, there may be a TTY issue."
        exit 0
    }
    
    # Test with icat
    if [[ -f "/tmp/test_red.png" ]]; then
        echo "Displaying red square with icat..."
        kitty +kitten icat /tmp/test_red.png
        echo ""
        sleep 1
        rm /tmp/test_red.png
    fi
fi

# Test 3: Our implementation
echo "Test 3: Testing our implementation"
DYLD_LIBRARY_PATH=./lib ./target/release/test-kitty

echo ""
echo "=== Testing chonker6 PDF Display ==="
echo ""
echo "To test the full PDF display:"
echo "1. Run: DYLD_LIBRARY_PATH=./lib ./target/release/chonker6"
echo "2. Press Ctrl+O to open a PDF file"
echo "3. The PDF should render as an image in the Kitty terminal"
echo ""
echo "Expected behavior:"
echo "- PDF pages should appear as images"
echo "- Images should be positioned correctly within the TUI"
echo "- Navigation should work (Page Up/Down)"
echo ""
echo "If you see literal escape sequences, check that:"
echo "- You're running directly in Kitty (not through another terminal)"
echo "- Kitty graphics are enabled in your config"
echo "- No other terminal multiplexers are interfering"