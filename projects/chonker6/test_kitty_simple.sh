#!/bin/bash

echo "Testing Kitty graphics protocol..."
echo ""

# Clear any existing images
printf '\x1b_Ga=d\x1b\\'

# Create a simple 2x2 red square (16 bytes of RGBA data)
# Each pixel is 4 bytes: R, G, B, A
# Red pixels: FF 00 00 FF
RED_DATA=$(printf '\377\000\000\377\377\000\000\377\377\000\000\377\377\000\000\377' | base64)

echo "Sending 2x2 red square..."
printf '\x1b_Ga=T,f=32,s=2,2;%s\x1b\\' "$RED_DATA"

echo ""
echo "You should see a tiny red square above if Kitty graphics are working."
echo ""

# Now test with a larger image (10x10 blue square)
echo "Sending 10x10 blue square..."
BLUE_DATA=""
for i in {1..100}; do
    # Blue pixel: 00 00 FF FF
    BLUE_DATA="${BLUE_DATA}\000\000\377\377"
done
BLUE_B64=$(printf "$BLUE_DATA" | base64)

printf '\x1b_Ga=T,f=32,s=10,10,X=20;%s\x1b\\' "$BLUE_B64"

echo ""
echo "You should see a blue square to the right if Kitty graphics are working."
echo ""
echo "If you see both squares, Kitty graphics protocol is working!"
echo "If not, you may not be in a Kitty terminal."