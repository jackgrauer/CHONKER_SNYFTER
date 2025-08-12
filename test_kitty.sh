#!/bin/bash

echo "Testing Kitty graphics protocol..."
echo "TERM=$TERM"
echo "KITTY_WINDOW_ID=$KITTY_WINDOW_ID"
echo ""

# Test 1: Check if we're in Kitty
if [ -n "$KITTY_WINDOW_ID" ]; then
    echo "✓ Running in Kitty terminal (KITTY_WINDOW_ID=$KITTY_WINDOW_ID)"
else
    echo "✗ Not running in Kitty terminal"
fi

# Test 2: Send a simple red square test image
echo "Sending test image (4x4 red square)..."

# Create a tiny 4x4 red PNG in base64
# This is a valid 4x4 red PNG
RED_PNG_BASE64="iVBORw0KGgoAAAANSUhEUgAAAAQAAAAECAYAAACp8Z5+AAAAEklEQVQIHWP4z8DwHwMDAwMABQgBALWZVNMAAAAASUVORK5CYII="

# Send using Kitty graphics protocol
printf '\033_Ga=T,f=100,s=4,v=4;%s\033\\' "$RED_PNG_BASE64"

echo ""
echo "You should see a tiny red square above if Kitty graphics is working."
echo ""

# Test 3: Query graphics support
echo "Querying graphics support..."
printf '\033_Gi=31,a=q\033\\'
sleep 0.1

echo ""
echo "Testing complete."
