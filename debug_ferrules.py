#!/usr/bin/env python3
"""Debug ferrules coordinates by showing them visually"""

import json
import sys
from pathlib import Path

def analyze_ferrules_json(json_path):
    with open(json_path) as f:
        doc = json.load(f)
    
    print(f"=== FERRULES COORDINATE ANALYSIS ===")
    print(f"Pages: {len(doc['pages'])}")
    
    # Calculate page boundaries
    page_boundaries = []
    cumulative_y = 0
    for page in doc['pages']:
        page_boundaries.append({
            'id': page['id'],
            'start_y': cumulative_y,
            'end_y': cumulative_y + page['height'],
            'height': page['height']
        })
        cumulative_y += page['height']
    
    print("\nPage boundaries:")
    for pb in page_boundaries:
        print(f"  Page {pb['id']}: Y {pb['start_y']:.1f} to {pb['end_y']:.1f} (height: {pb['height']:.1f})")
    
    # Analyze blocks
    print("\nBlock analysis:")
    for i, block in enumerate(doc['blocks'][:10]):  # First 10 blocks
        y0, y1 = block['bbox']['y0'], block['bbox']['y1']
        text = block['kind'].get('text', '')[:40]
        
        # Find which page this block belongs to based on Y coordinate
        page_num = None
        for j, pb in enumerate(page_boundaries):
            if pb['start_y'] <= y0 < pb['end_y']:
                page_num = j + 1
                page_relative_y = y0 - pb['start_y']
                break
        
        print(f"\nBlock {i}:")
        print(f"  Absolute Y: {y0:.1f} to {y1:.1f}")
        print(f"  On page: {page_num}")
        print(f"  Page-relative Y: {page_relative_y:.1f}" if page_num else "  ERROR: Outside page bounds!")
        print(f"  Text: \"{text}...\"" if text else "  No text")
        
    # Check if coordinates are cumulative
    print("\n=== COORDINATE SYSTEM ===")
    if page_boundaries[-1]['end_y'] > page_boundaries[-1]['height'] * 1.5:
        print("✓ Coordinates are CUMULATIVE across pages")
        print("  (Y coordinates continue increasing across page boundaries)")
    else:
        print("✗ Coordinates might be PAGE-RELATIVE")

if __name__ == "__main__":
    if len(sys.argv) != 2:
        # Try to find a recent ferrules JSON
        json_files = list(Path('/tmp').rglob('*ferrules*/**.json'))
        if json_files:
            print(f"Using {json_files[0]}")
            analyze_ferrules_json(json_files[0])
        else:
            print("Usage: debug_ferrules.py <ferrules_json_file>")
    else:
        analyze_ferrules_json(sys.argv[1])