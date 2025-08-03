#!/usr/bin/env python3
"""Analyze coordinate systems in PDF extraction tools"""

import json
import subprocess
import sys
import tempfile
import os
from pathlib import Path

def run_ferrules(pdf_path):
    """Run ferrules and return the JSON output"""
    with tempfile.TemporaryDirectory() as tmpdir:
        # Run ferrules
        cmd = ['ferrules', pdf_path, '-o', tmpdir]
        result = subprocess.run(cmd, capture_output=True, text=True)
        
        if result.returncode != 0:
            print(f"Ferrules failed: {result.stderr}")
            return None
            
        # Find JSON file
        json_files = list(Path(tmpdir).glob('*.json'))
        if not json_files:
            print("No JSON output found")
            return None
            
        with open(json_files[0]) as f:
            return json.load(f)

def analyze_coordinates(doc):
    """Analyze the coordinate system used by ferrules"""
    print("\n=== COORDINATE SYSTEM ANALYSIS ===")
    print(f"Number of pages: {len(doc['pages'])}")
    print(f"Number of blocks: {len(doc['blocks'])}")
    
    # Group blocks by page
    blocks_by_page = {}
    for block in doc['blocks']:
        for page_id in block['pages_id']:
            if page_id not in blocks_by_page:
                blocks_by_page[page_id] = []
            blocks_by_page[page_id].append(block)
    
    # Analyze each page
    prev_max_y = None
    for i, page in enumerate(doc['pages']):
        page_id = page['id']
        print(f"\n--- Page {i+1} (ID: {page_id}) ---")
        print(f"  Dimensions: {page['width']} x {page['height']}")
        
        if page_id not in blocks_by_page:
            print("  No blocks on this page")
            continue
            
        blocks = blocks_by_page[page_id]
        print(f"  Blocks: {len(blocks)}")
        
        # Find Y coordinate range
        y_coords = []
        for block in blocks:
            y_coords.extend([block['bbox']['y0'], block['bbox']['y1']])
        
        min_y = min(y_coords)
        max_y = max(y_coords)
        
        print(f"  Y range: {min_y:.2f} to {max_y:.2f}")
        print(f"  Y span: {max_y - min_y:.2f}")
        
        # Check if cumulative
        if prev_max_y is not None:
            gap = min_y - prev_max_y
            print(f"  Gap from previous page: {gap:.2f}")
            if gap > -10:  # Allow small overlap
                print("  ✓ Coordinates appear CUMULATIVE")
            else:
                print("  ✗ Coordinates appear PAGE-RELATIVE")
        
        # Show first few blocks
        print("  First 3 blocks:")
        for j, block in enumerate(sorted(blocks, key=lambda b: b['bbox']['y0'])[:3]):
            text = block['kind'].get('text', '')
            if text and len(text) > 40:
                text = text[:40] + "..."
            print(f"    {j+1}. Y: {block['bbox']['y0']:.2f}-{block['bbox']['y1']:.2f} \"{text}\"")
        
        prev_max_y = max_y
    
    # Determine coordinate origin
    first_blocks = sorted(doc['blocks'], key=lambda b: b['bbox']['y0'])[:5]
    avg_first_y = sum(b['bbox']['y0'] for b in first_blocks) / len(first_blocks)
    
    print(f"\n=== COORDINATE ORIGIN ===")
    print(f"Average Y of first 5 blocks: {avg_first_y:.2f}")
    if avg_first_y < 100:
        print("✓ Origin is likely TOP-LEFT (Y increases downward)")
    else:
        print("✗ Origin might be BOTTOM-LEFT (Y increases upward)")

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: analyze_coords.py <pdf_file>")
        sys.exit(1)
    
    pdf_path = sys.argv[1]
    
    # Run ferrules
    print(f"Analyzing {pdf_path}...")
    doc = run_ferrules(pdf_path)
    
    if doc:
        analyze_coordinates(doc)
        
        # Save for inspection
        with open('/tmp/ferrules_debug.json', 'w') as f:
            json.dump(doc, f, indent=2)
        print(f"\nFull output saved to /tmp/ferrules_debug.json")