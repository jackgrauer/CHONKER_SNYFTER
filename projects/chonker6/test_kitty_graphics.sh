#!/bin/bash

# Simple test to verify Kitty graphics protocol is working
# This creates a small red square to test if Kitty can display images

# Create a simple 10x10 red square in RGBA format (400 bytes)
# Each pixel is 4 bytes: R, G, B, A
RED_SQUARE=""
for i in {1..100}; do
    # Red pixel: FF 00 00 FF (red, no green, no blue, full alpha)
    RED_SQUARE="${RED_SQUARE}\xff\x00\x00\xff"
done

# Encode to base64
BASE64_DATA=$(echo -ne "$RED_SQUARE" | base64)

# Clear any existing images
echo -e "\x1b_Ga=d\x1b\\"

# Send the image using Kitty graphics protocol
# a=T means transmit and display
# f=32 means 32-bit RGBA
# s=10,10 means 10x10 pixels
echo -e "\x1b_Ga=T,f=32,s=10,10;${BASE64_DATA}\x1b\\"

echo "If you see a red square above, Kitty graphics protocol is working!"
echo "If not, your terminal may not be Kitty or graphics may be disabled."