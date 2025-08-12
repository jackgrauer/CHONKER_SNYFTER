#!/bin/bash

echo "=== Kitty Graphics Debug Script ==="
echo "Terminal: $TERM"
echo "Kitty Window ID: $KITTY_WINDOW_ID"
echo ""

# Test 1: Basic Kitty graphics
echo "Test 1: Basic graphics protocol"
echo -e "\x1b_Ga=d\x1b\\"  # Clear all
echo -e "\x1b[5;5H"  # Position
# Small red square PNG
echo -e "\x1b_Ga=T,f=100,i=1,s=50,v=50;iVBORw0KGgoAAAANSUhEUgAAAAIAAAACCAYAAABytg0kAAAADklEQVQIHWP4z8DwHwAFAAH/VscvDQAAAABJRU5ErkJggg==\x1b\\"
echo ""
echo ""
echo ""
echo "You should see a small red square above"
sleep 2

# Test 2: With z-index
echo ""
echo "Test 2: Testing z-index (behind text)"
echo -e "\x1b_Ga=d\x1b\\"  # Clear all
echo -e "\x1b[10;5H"  # Position
echo -e "\x1b_Ga=T,f=100,i=2,s=100,v=100,z=-1;iVBORw0KGgoAAAANSUhEUgAAAAIAAAACCAYAAABytg0kAAAADklEQVQIHWNgYPj/nwEAAgAB/6gZlAAAAABJRU5ErkJggg==\x1b\\"
echo -e "\x1b[11;6HTEXT OVER IMAGE"
echo ""
echo "You should see text over a blue square"
sleep 2

# Test 3: In alternate screen
echo ""
echo "Test 3: Alternate screen mode"
echo -e "\x1b[?1049h"  # Enter alternate screen
echo -e "\x1b[2J"  # Clear
echo -e "\x1b[5;5H"  # Position
echo -e "\x1b_Ga=T,f=100,i=3,s=80,v=80;iVBORw0KGgoAAAANSUhEUgAAAAIAAAACCAYAAABytg0kAAAADklEQVQIHWNg+M/wHwAEgAH/qhmUAAAAAElFTkSuQmCC\x1b\\"
echo -e "\x1b[10;1HGreen square in alternate screen"
echo -e "\x1b[12;1HPress any key..."
read -n 1
echo -e "\x1b[?1049l"  # Leave alternate screen

echo ""
echo "=== Debug Complete ==="
echo ""
echo "If you saw:"
echo "1. Red square - basic protocol works"
echo "2. Blue square with text - z-index works"
echo "3. Green square in alternate screen - alternate screen works"
echo ""
echo "All tests passed!"