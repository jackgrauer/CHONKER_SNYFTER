#!/usr/bin/env python3
"""
Convert SmolDocling table data to human-readable formats (CSV, HTML, Markdown)
"""

import json
import csv
import sys
from typing import Dict, List, Any, Optional

def extract_table_from_smoldocling(json_data: Dict[str, Any]) -> Optional[Dict[str, Any]]:
    """Extract the first table from SmolDocling JSON output"""
    extractions = json_data.get('extractions', [])
    if not extractions:
        return None
    
    # Look for table data in structured_chunks
    for chunk in extractions[0].get('structured_chunks', []):
        if chunk.get('type') == 'table' and chunk.get('table_data'):
            return chunk['table_data']
    
    return None

def create_table_grid(table_data: Dict[str, Any]) -> List[List[str]]:
    """Create a 2D grid from SmolDocling table cell data"""
    cells = table_data.get('cells', [])
    if not cells:
        return []
    
    # Find maximum dimensions
    max_row = max(cell['end_row_offset_idx'] for cell in cells)
    max_col = max(cell['end_col_offset_idx'] for cell in cells)
    
    # Create empty grid
    grid = [[''] * max_col for _ in range(max_row)]
    
    # Fill grid with cell data
    for cell in cells:
        text = cell.get('text', '').strip()
        if text:  # Only fill non-empty cells
            start_row = cell['start_row_offset_idx']
            start_col = cell['start_col_offset_idx']
            end_row = cell['end_row_offset_idx']
            end_col = cell['end_col_offset_idx']
            
            # Fill all cells in the span
            for row in range(start_row, end_row):
                for col in range(start_col, end_col):
                    if row < len(grid) and col < len(grid[row]):
                        grid[row][col] = text
    
    return grid

def table_to_csv(table_data: Dict[str, Any], output_file: str):
    """Convert table data to CSV format"""
    grid = create_table_grid(table_data)
    
    with open(output_file, 'w', newline='', encoding='utf-8') as csvfile:
        writer = csv.writer(csvfile)
        for row in grid:
            writer.writerow(row)
    
    print(f"CSV saved to: {output_file}")

def table_to_html(table_data: Dict[str, Any], output_file: str):
    """Convert table data to HTML format"""
    grid = create_table_grid(table_data)
    
    html = ['<table border="1" style="border-collapse: collapse;">']
    
    for i, row in enumerate(grid):
        html.append('  <tr>')
        for j, cell in enumerate(row):
            # Check if this is a header row (first few rows typically)
            tag = 'th' if i < 3 else 'td'  # First 3 rows as headers
            html.append(f'    <{tag}>{cell}</{tag}>')
        html.append('  </tr>')
    
    html.append('</table>')
    
    with open(output_file, 'w', encoding='utf-8') as f:
        f.write('\n'.join(html))
    
    print(f"HTML saved to: {output_file}")

def table_to_markdown(table_data: Dict[str, Any], output_file: str):
    """Convert table data to Markdown format"""
    grid = create_table_grid(table_data)
    
    if not grid:
        print("No table data found")
        return
    
    markdown = []
    
    # Add header row
    if grid:
        header_row = '| ' + ' | '.join(grid[0]) + ' |'
        markdown.append(header_row)
        
        # Add separator
        separator = '| ' + ' | '.join(['---'] * len(grid[0])) + ' |'
        markdown.append(separator)
        
        # Add data rows
        for row in grid[1:]:
            data_row = '| ' + ' | '.join(row) + ' |'
            markdown.append(data_row)
    
    with open(output_file, 'w', encoding='utf-8') as f:
        f.write('\n'.join(markdown))
    
    print(f"Markdown saved to: {output_file}")

def print_table_summary(table_data: Dict[str, Any]):
    """Print a summary of the table structure"""
    cells = table_data.get('cells', [])
    grid = create_table_grid(table_data)
    
    print(f"\nTable Summary:")
    print(f"- Total cells: {len(cells)}")
    print(f"- Dimensions: {len(grid)} rows × {len(grid[0]) if grid else 0} columns")
    print(f"- Non-empty cells: {sum(1 for row in grid for cell in row if cell.strip())}")
    
    # Show first few rows for preview
    print(f"\nFirst 5 rows preview:")
    for i, row in enumerate(grid[:5]):
        print(f"Row {i}: {row[:8]}...")  # Show first 8 columns

def main():
    if len(sys.argv) < 2:
        print("Usage: python table_converter.py <smoldocling_output.json> [output_basename]")
        sys.exit(1)
    
    input_file = sys.argv[1]
    output_base = sys.argv[2] if len(sys.argv) > 2 else "table_output"
    
    try:
        with open(input_file, 'r', encoding='utf-8') as f:
            json_data = json.load(f)
        
        table_data = extract_table_from_smoldocling(json_data)
        if not table_data:
            print("No table data found in JSON")
            sys.exit(1)
        
        print_table_summary(table_data)
        
        # Convert to different formats
        table_to_csv(table_data, f"{output_base}.csv")
        table_to_html(table_data, f"{output_base}.html")
        table_to_markdown(table_data, f"{output_base}.md")
        
        print(f"\nTable converted to multiple formats with base name: {output_base}")
        
    except Exception as e:
        print(f"Error: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()
