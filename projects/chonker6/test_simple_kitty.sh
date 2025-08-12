#!/bin/bash

echo "Testing Kitty graphics with a simple image..."

# Create a minimal 2x2 red PNG in base64 (hand-crafted)
# This is a valid 2x2 pixel red PNG
BASE64_PNG="iVBORw0KGgoAAAANSUhEUgAAAAIAAAACCAYAAABytg0kAAAADklEQVQIHWP4z8DwHwAFAAH/VscvDQAAAABJRU5ErkJggg=="

# Clear existing images
printf "\x1b_Ga=d\x1b\\"

# Move to position
printf "\x1b[5;5H"

# Send the image - scale it up to 100x100
printf "\x1b_Ga=T,f=100,i=1,s=100,v=100;%s\x1b\\" "$BASE64_PNG"

echo ""
echo ""
echo ""
echo ""
echo ""
echo ""
echo "If you see a red square above, Kitty graphics are working!"