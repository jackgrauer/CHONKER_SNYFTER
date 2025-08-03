#!/usr/bin/env python3
"""Test rendering ferrules coordinates to understand the issue"""

import json
import matplotlib.pyplot as plt
import matplotlib.patches as patches

def render_ferrules_json(json_path):
    with open(json_path) as f:
        doc = json.load(f)
    
    # Create figure with subplots for each page
    n_pages = len(doc['pages'])
    fig, axes = plt.subplots(1, n_pages, figsize=(8*n_pages, 10))
    if n_pages == 1:
        axes = [axes]
    
    # Render each page
    for page_idx, page in enumerate(doc['pages']):
        ax = axes[page_idx]
        ax.set_title(f'Page {page_idx + 1}')
        ax.set_xlim(0, page['width'])
        ax.set_ylim(0, page['height'])
        ax.invert_yaxis()  # PDF coordinates have origin at bottom-left
        
        # Draw page boundary
        rect = patches.Rectangle((0, 0), page['width'], page['height'], 
                                linewidth=2, edgecolor='black', facecolor='white')
        ax.add_patch(rect)
        
        # Draw blocks based on ferrules' page assignment
        blocks_on_page = [b for b in doc['blocks'] if page['id'] in b['pages_id']]
        
        for block in blocks_on_page:
            # Draw bounding box
            x0, y0 = block['bbox']['x0'], block['bbox']['y0']
            width = block['bbox']['x1'] - x0
            height = block['bbox']['y1'] - y0
            
            rect = patches.Rectangle((x0, y0), width, height,
                                   linewidth=1, edgecolor='red', facecolor='none')
            ax.add_patch(rect)
            
            # Add text preview
            if 'text' in block['kind'] and block['kind']['text']:
                text = block['kind']['text'][:20] + '...' if len(block['kind']['text']) > 20 else block['kind']['text']
                ax.text(x0, y0-5, text, fontsize=8, color='blue')
        
        ax.set_aspect('equal')
        ax.text(10, 20, f'{len(blocks_on_page)} blocks', fontsize=10, color='green')
    
    plt.tight_layout()
    plt.savefig('/tmp/ferrules_render_test.png', dpi=150, bbox_inches='tight')
    print("Saved visualization to /tmp/ferrules_render_test.png")
    
    # Also print coordinate analysis
    print("\n=== Coordinate Analysis ===")
    for page_idx, page in enumerate(doc['pages']):
        blocks = [b for b in doc['blocks'] if page['id'] in b['pages_id']]
        if blocks:
            y_coords = [b['bbox']['y0'] for b in blocks]
            print(f"Page {page_idx + 1}: Y range {min(y_coords):.1f} - {max(y_coords):.1f}")

if __name__ == "__main__":
    import sys
    if len(sys.argv) > 1:
        render_ferrules_json(sys.argv[1])
    else:
        # Use the test file
        render_ferrules_json("/tmp/ferrules_test/journal_entry-5--results/journal_entry-5-.json")