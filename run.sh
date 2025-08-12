#!/bin/bash
# Launch script for Chonker6

cd /Users/jack/chonker6

echo "🚀 Starting Chonker6 - Clean Architecture Edition"
echo ""
echo "This is a complete rewrite with:"
echo "• Modular architecture (no more 2000-line files!)"
echo "• Redux-like state management"
echo "• Clean event handling"
echo "• Visual panel highlighting (no borders)"
echo ""
echo "Starting in 2 seconds..."
sleep 2

DYLD_LIBRARY_PATH=/Users/jack/chonker6/lib ./target/release/chonker6