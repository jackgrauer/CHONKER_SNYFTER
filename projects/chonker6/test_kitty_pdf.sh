#!/bin/bash

echo "Testing Kitty PDF rendering..."
echo "TERM=$TERM"
echo "KITTY_WINDOW_ID=$KITTY_WINDOW_ID"

# Create a simple red square as PNG
python3 << 'EOF'
from PIL import Image
import base64
import io

# Create a 100x100 red square
img = Image.new('RGB', (100, 100), color='red')
buffer = io.BytesIO()
img.save(buffer, format='PNG')
png_data = buffer.getvalue()
b64 = base64.b64encode(png_data).decode('ascii')

print(f"PNG size: {len(png_data)} bytes")
print(f"Base64 size: {len(b64)} chars")

# Send to Kitty
# Clear any existing images
print("\x1b_Ga=d\x1b\\", end='', flush=True)

# Position cursor
print("\x1b[5;5H", end='', flush=True)

# Send image
print(f"\x1b_Ga=T,f=100,i=1,s=100,v=100;{b64}\x1b\\", end='', flush=True)

# Move cursor down
print("\n" * 10)
print("You should see a red square above if Kitty graphics are working!")
EOF