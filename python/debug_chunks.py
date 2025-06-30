#!/usr/bin/env python3
"""
Debug script to see what table chunks we're extracting
"""

import re
from pathlib import Path

def extract_table_chunks(content):
    """Extract individual tables from markdown content"""
    lines = content.split('\n')
    table_chunks = []
    current_chunk = []
    in_table = False
    
    for line in lines:
        # Check if this looks like a table line
        if '|' in line and len(line.split('|')) >= 3:
            in_table = True
            current_chunk.append(line)
        elif in_table:
            # Check if we should continue the table
            if line.strip() == '' or line.strip().startswith('#'):
                # End of table
                if current_chunk:
                    table_chunks.append('\n'.join(current_chunk))
                    current_chunk = []
                in_table = False
            else:
                # Might be part of table (like wrapped content)
                current_chunk.append(line)
    
    # Don't forget the last chunk
    if current_chunk:
        table_chunks.append('\n'.join(current_chunk))
        
    return table_chunks

def has_misplaced_qualifiers(table_text):
    """Check if a table chunk has obvious misplaced qualifiers"""
    # Look for patterns like "number U" or "number J"
    qualifier_pattern = r'\b\d+\.?\d*\s+[UJ]\b'
    return bool(re.search(qualifier_pattern, table_text))

def main():
    input_file = "EXAMPLE_NIGHTMARE_PDF.md"
    
    if not Path(input_file).exists():
        print(f"❌ File not found: {input_file}")
        return
    
    with open(input_file, 'r', encoding='utf-8') as f:
        content = f.read()
    
    chunks = extract_table_chunks(content)
    print(f"🔍 Found {len(chunks)} table chunks")
    
    for i, chunk in enumerate(chunks):
        has_qualifiers = has_misplaced_qualifiers(chunk)
        chunk_size = len(chunk)
        lines = chunk.count('\n') + 1
        
        print(f"\n📋 Chunk {i+1}:")
        print(f"   Size: {chunk_size:,} characters")
        print(f"   Lines: {lines}")
        print(f"   Has qualifiers: {has_qualifiers}")
        
        if has_qualifiers and i < 3:  # Show first few with qualifiers
            print(f"   Preview (first 200 chars):")
            print(f"   {repr(chunk[:200])}")
            
            # Look for the actual qualifier patterns
            qualifier_matches = re.findall(r'\b\d+\.?\d*\s+[UJ]\b', chunk)
            print(f"   Qualifier patterns found: {qualifier_matches[:5]}")  # First 5

if __name__ == "__main__":
    main()
