#!/usr/bin/env python3
"""
Test script to verify the complete enhanced extraction pipeline
This simulates what the Tauri/Rust application should be doing
"""

import json
import subprocess
import sys
from pathlib import Path

def test_enhanced_pipeline():
    print("🧪 Testing Enhanced Extraction Pipeline")
    print("=" * 50)
    
    # Test file
    test_pdf = "test.pdf"
    if not Path(test_pdf).exists():
        print(f"❌ Test file {test_pdf} not found!")
        return False
    
    # Run enhanced extraction bridge
    print(f"📄 Processing: {test_pdf}")
    try:
        result = subprocess.run([
            "python3", "python/enhanced_extraction_bridge.py", test_pdf
        ], capture_output=True, text=True, timeout=60)
        
        if result.returncode != 0:
            print(f"❌ Enhanced extraction failed: {result.stderr}")
            return False
            
        # Parse the JSON output
        try:
            data = json.loads(result.stdout)
        except json.JSONDecodeError as e:
            print(f"❌ Failed to parse JSON output: {e}")
            return False
            
        # Validate the output structure
        print("\n✅ Enhanced extraction successful!")
        print(f"📊 Success: {data.get('success', False)}")
        print(f"🔧 Tool: {data.get('tool', 'unknown')}")
        
        # Check structured_tables
        structured_tables = data.get('structured_tables', [])
        print(f"📋 Structured tables found: {len(structured_tables)}")
        
        if structured_tables:
            table = structured_tables[0]
            print(f"📐 Table dimensions: {table.get('num_rows')}x{table.get('num_cols')}")
            
            # Check context
            context = table.get('context', {})
            if context:
                print(f"📝 Table title: {context.get('table_title', 'N/A')}")
                print(f"📄 Text before: {context.get('text_before', 'N/A')[:50]}...")
            
            # Check grid structure
            grid = table.get('grid', [])
            if grid:
                print(f"🏗️ Grid structure: {len(grid)} rows")
                # Sample first row
                first_row = grid[0] if grid else []
                print(f"🔍 First row cells: {len(first_row)}")
                
                # Show sample cell structure
                if first_row:
                    sample_cell = first_row[0]
                    print(f"🧩 Sample cell: {json.dumps(sample_cell, indent=2)[:200]}...")
        
        # Test what Rust would do with this data
        print("\n🦀 Simulating Rust Processing...")
        chunks = convert_to_chunks(data)
        print(f"📦 Generated chunks: {len(chunks)}")
        
        for i, chunk in enumerate(chunks):
            print(f"   Chunk {i}: {chunk['content_type']} - {len(chunk.get('content', ''))} chars")
            if chunk['content_type'] == 'table' and chunk.get('table_data'):
                table_data = chunk['table_data']
                print(f"     Table: {table_data['num_rows']}x{table_data['num_cols']}")
        
        print("\n🎉 Full pipeline test successful!")
        return True
        
    except subprocess.TimeoutExpired:
        print("❌ Enhanced extraction timed out!")
        return False
    except Exception as e:
        print(f"❌ Unexpected error: {e}")
        return False

def convert_to_chunks(extraction_result):
    """
    Simulate the Rust chunk conversion logic
    """
    chunks = []
    chunk_index = 0
    
    # Check for structured_tables (priority)
    structured_tables = extraction_result.get('structured_tables', [])
    if structured_tables:
        print("🎯 Using structured_tables (enhanced data)")
        
        for table_idx, table in enumerate(structured_tables):
            # Create table chunk with structured data
            table_data = convert_table_to_table_data(table)
            
            # Enhanced metadata with context
            context = table.get('context', {})
            table_title = context.get('table_title', '')
            text_before = context.get('text_before', '')
            
            metadata = f"structured_table_{table_idx}: {table_title} | {text_before}"
            
            chunk = {
                'id': f'chunk_{chunk_index}',
                'chunk_index': chunk_index,
                'content': json.dumps(table, indent=2),
                'content_type': 'table',
                'metadata': metadata,
                'table_data': table_data
            }
            
            chunks.append(chunk)
            chunk_index += 1
    
    # Process page extractions for text content
    extractions = extraction_result.get('extractions', [])
    for page_extraction in extractions:
        # Text content
        text = page_extraction.get('text', '').strip()
        if text:
            chunk = {
                'id': f'chunk_{chunk_index}',
                'chunk_index': chunk_index,
                'content': text,
                'content_type': 'text',
                'metadata': f"page_{page_extraction.get('page_number', 1)}",
                'table_data': None
            }
            chunks.append(chunk)
            chunk_index += 1
        
        # Only process basic tables if no structured_tables
        if not structured_tables:
            print("⚠️ Falling back to basic tables (legacy)")
            tables = page_extraction.get('tables', [])
            for table_idx, table in enumerate(tables):
                table_data = convert_table_to_table_data(table)
                
                chunk = {
                    'id': f'chunk_{chunk_index}',
                    'chunk_index': chunk_index,
                    'content': json.dumps(table, indent=2),
                    'content_type': 'table',
                    'metadata': f"page_{page_extraction.get('page_number', 1)}_table_{table_idx}",
                    'table_data': table_data
                }
                chunks.append(chunk)
                chunk_index += 1
    
    return chunks

def convert_table_to_table_data(table):
    """
    Simulate the Rust table conversion logic
    """
    # Check for enhanced grid format
    grid = table.get('grid')
    if grid:
        num_rows = table.get('num_rows', len(grid))
        num_cols = table.get('num_cols', 0)
        
        data = []
        for row in grid:
            row_cells = []
            for cell in row:
                if isinstance(cell, dict):
                    cell_content = cell.get('text', '')
                    rowspan = cell.get('row_span')
                    colspan = cell.get('col_span')
                    
                    if rowspan and rowspan > 1:
                        rowspan = rowspan
                    else:
                        rowspan = None
                        
                    if colspan and colspan > 1:
                        colspan = colspan
                    else:
                        colspan = None
                else:
                    cell_content = str(cell)
                    rowspan = None
                    colspan = None
                
                row_cells.append({
                    'content': cell_content,
                    'rowspan': rowspan,
                    'colspan': colspan
                })
            data.append(row_cells)
        
        return {
            'num_rows': num_rows,
            'num_cols': num_cols,
            'data': data
        }
    else:
        # Legacy format fallback
        return {
            'num_rows': 1,
            'num_cols': 1,
            'data': [[{
                'content': json.dumps(table),
                'rowspan': None,
                'colspan': None
            }]]
        }

if __name__ == '__main__':
    success = test_enhanced_pipeline()
    sys.exit(0 if success else 1)
