#!/bin/bash

echo "=== Running Chonker5 with Debug Output ==="
echo
echo "Watch for these messages in the CONSOLE:"
echo "  🖱️ CELL SELECTED: (x, y) - when you click a cell"
echo "  🔤 TYPED 'X' at cell (x, y) - when you type"
echo
echo "In the app, look for the DEBUG LOG box at the bottom showing:"
echo "  - Recent log messages"
echo "  - State: Focus, Selected cell, Edit mode"
echo
echo "Starting app..."
echo

cargo run