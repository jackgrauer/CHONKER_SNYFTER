#!/usr/bin/env python3
"""Simple coordinate test for ferrules"""

import json

# Load the 2-page test document
with open("/tmp/ferrules_test/journal_entry-5--results/journal_entry-5-.json") as f:
    doc = json.load(f)

print("=== FERRULES COORDINATE TEST ===")
print(f"Pages: {len(doc['pages'])}")

# Show page info
for p in doc['pages']:
    print(f"Page {p['id']}: {p['width']}x{p['height']}")

# Group blocks by their assigned page
blocks_by_page = {}
for b in doc['blocks']:
    for pid in b['pages_id']:
        if pid not in blocks_by_page:
            blocks_by_page[pid] = []
        blocks_by_page[pid].append(b)

# Analyze each page
print("\n=== BLOCK DISTRIBUTION ===")
for pid in sorted(blocks_by_page.keys()):
    blocks = blocks_by_page[pid]
    print(f"\nPage {pid} has {len(blocks)} blocks:")
    
    # Sort by Y coordinate
    blocks.sort(key=lambda b: b['bbox']['y0'])
    
    # Show first and last few
    for i, b in enumerate(blocks[:3]):
        text = b['kind'].get('text', '')[:30] + '...' if b['kind'].get('text', '') else 'No text'
        print(f"  Block {i+1}: Y={b['bbox']['y0']:.1f}-{b['bbox']['y1']:.1f} \"{text}\"")
    
    if len(blocks) > 6:
        print("  ...")
        
    for i, b in enumerate(blocks[-3:]):
        text = b['kind'].get('text', '')[:30] + '...' if b['kind'].get('text', '') else 'No text'
        print(f"  Block {len(blocks)-2+i}: Y={b['bbox']['y0']:.1f}-{b['bbox']['y1']:.1f} \"{text}\"")

# Check for coordinate issues
print("\n=== COORDINATE ISSUES ===")
page_height = doc['pages'][0]['height']
print(f"Page height: {page_height}")

for pid, blocks in blocks_by_page.items():
    out_of_bounds = [b for b in blocks if b['bbox']['y0'] > page_height or b['bbox']['y1'] > page_height]
    if out_of_bounds:
        print(f"Page {pid}: {len(out_of_bounds)} blocks have Y > page height!")
        for b in out_of_bounds[:3]:
            print(f"  Y={b['bbox']['y0']:.1f}-{b['bbox']['y1']:.1f}")