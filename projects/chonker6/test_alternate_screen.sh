#!/bin/bash

echo "Testing Kitty graphics in alternate screen mode..."

# Enter alternate screen
printf "\x1b[?1049h"

# Clear screen
printf "\x1b[2J"

# Clear existing images
printf "\x1b_Ga=d\x1b\\"

# Move to position
printf "\x1b[5;5H"

# Send a test image
BASE64_PNG="iVBORw0KGgoAAAANSUhEUgAAAAIAAAACCAYAAABytg0kAAAADklEQVQIHWP4z8DwHwAFAAH/VscvDQAAAABJRU5ErkJggg=="
printf "\x1b_Ga=T,f=100,i=1,s=100,v=100;%s\x1b\\" "$BASE64_PNG"

# Move cursor down
printf "\x1b[10;1H"
echo "Testing image in alternate screen..."
echo "Press any key to continue"

# Wait for input
read -n 1

# Leave alternate screen
printf "\x1b[?1049l"

echo "Back to normal screen"