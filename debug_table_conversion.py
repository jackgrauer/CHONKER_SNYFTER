#!/usr/bin/env python3
"""
Debug script to examine how the enhanced data is being converted
"""

import json
import sys

# Read the final.json from pipeline
try:
    with open('src-tauri/pipeline_outputs/final.json', 'r') as f:
        data = json.load(f)
        
    if 'structured_tables' not in data:
        print("❌ No structured_tables in final.json")
        sys.exit(1)
        
    table = data['structured_tables'][0]
    
    print("🔍 ENHANCED TABLE STRUCTURE ANALYSIS")
    print("=" * 50)
    
    print(f"📐 Dimensions: {table.get('num_rows')}x{table.get('num_cols')}")
    
    # Look at the grid structure
    grid = table.get('grid', [])
    print(f"🏗️ Grid rows: {len(grid)}")
    
    # Examine first few rows
    for i, row in enumerate(grid[:5]):
        print(f"\nRow {i}: {len(row)} cells")
        for j, cell in enumerate(row[:3]):  # First 3 cells
            if isinstance(cell, dict):
                text = cell.get('text', '')
                is_header = cell.get('is_header', False)
                print(f"  Cell[{i},{j}]: '{text}' (header: {is_header})")
            else:
                print(f"  Cell[{i},{j}]: '{cell}' (simple)")
    
    # Check if first row has headers
    first_row = grid[0] if grid else []
    header_flags = [cell.get('is_header', False) if isinstance(cell, dict) else False for cell in first_row]
    print(f"\n🏷️ First row header flags: {header_flags}")
    print(f"🏷️ All headers in first row: {all(header_flags)}")
    
    # Show what our Rust conversion should produce
    print(f"\n🦀 WHAT RUST SHOULD GENERATE:")
    print("Headers should be:", [cell.get('text', '') if isinstance(cell, dict) else str(cell) for cell in first_row])
    
    # Show second row as data
    if len(grid) > 1:
        second_row = grid[1]
        print("First data row should be:", [cell.get('text', '') if isinstance(cell, dict) else str(cell) for cell in second_row])
        
except Exception as e:
    print(f"❌ Error: {e}")
    sys.exit(1)
