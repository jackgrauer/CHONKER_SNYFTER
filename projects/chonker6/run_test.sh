#!/bin/bash

echo "Starting chonker6 with debug output..."
echo "TERM=$TERM"
echo "KITTY_WINDOW_ID=$KITTY_WINDOW_ID"
echo ""
echo "Instructions:"
echo "1. Press Ctrl+O to open a PDF"
echo "2. Navigate to /Users/jack/Desktop and select BERF-CERT.pdf"
echo "3. Watch the debug output in stderr"
echo "4. Press Ctrl+Q to quit"
echo ""
echo "Starting in 3 seconds..."
sleep 3

DYLD_LIBRARY_PATH=./lib ./target/release/chonker6 2>debug.log

echo ""
echo "Debug output saved to debug.log"
echo "Checking for Kitty graphics commands..."
grep -E "KITTY|PDF|Kitty" debug.log | tail -20