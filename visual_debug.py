#!/usr/bin/env python3
"""Visual debug of ferrules coordinates"""

import json
import sys
from PIL import Image, ImageDraw, ImageFont

def visualize_ferrules(json_path):
    with open(json_path) as f:
        doc = json.load(f)
    
    # Create image for each page
    scale = 0.5  # Scale down for display
    
    for page_idx, page in enumerate(doc['pages']):
        width = int(page['width'] * scale)
        height = int(page['height'] * scale)
        
        img = Image.new('RGB', (width, height), 'white')
        draw = ImageDraw.Draw(img)
        
        # Draw page boundary
        draw.rectangle([0, 0, width-1, height-1], outline='black', width=2)
        
        # Calculate cumulative Y offset for this page
        y_offset = sum(p['height'] for p in doc['pages'][:page_idx])
        
        # Draw blocks
        blocks_on_page = 0
        for block in doc['blocks']:
            if page.get('id', page_idx) in block['pages_id']:
                blocks_on_page += 1
                
                # Get coordinates
                x0 = block['bbox']['x0'] * scale
                y0 = (block['bbox']['y0'] - y_offset) * scale  # Subtract cumulative offset
                x1 = block['bbox']['x1'] * scale
                y1 = (block['bbox']['y1'] - y_offset) * scale
                
                # Draw bounding box
                color = 'red' if y0 < 0 or y1 > height else 'green'
                draw.rectangle([x0, y0, x1, y1], outline=color, width=1)
                
                # Add block ID
                draw.text((x0, y0-10), f"B{block['id']}", fill=color)
        
        # Add page info
        draw.text((10, 10), f"Page {page_idx+1} - {blocks_on_page} blocks", fill='blue')
        draw.text((10, 25), f"Expected Y: {y_offset:.0f}-{y_offset+page['height']:.0f}", fill='blue')
        
        # Save image
        output_path = f'/tmp/ferrules_debug_page_{page_idx+1}.png'
        img.save(output_path)
        print(f"Saved {output_path}")

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: visual_debug.py <ferrules_json>")
        sys.exit(1)
    
    visualize_ferrules(sys.argv[1])