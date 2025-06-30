#!/usr/bin/env python3
"""
Debug the fixed chunking to see what we're actually sending to Qwen
"""

import re
from pathlib import Path

def break_large_table(table_lines):
    """Break a large table into smaller chunks while preserving header structure"""
    if not table_lines:
        return []
    
    chunks = []
    header_lines = []
    data_lines = []
    
    # Identify header (first few lines that define structure)
    for i, line in enumerate(table_lines[:5]):  # Check first 5 lines for header pattern
        if '|' in line:
            if any(char in line for char in ['---', '===', ':']):
                # This is a separator line
                header_lines = table_lines[:i+1]
                data_lines = table_lines[i+1:]
                break
    
    # If no clear header found, just use first line as header
    if not header_lines and table_lines:
        header_lines = [table_lines[0]]
        data_lines = table_lines[1:]
    
    # Break data lines into chunks of reasonable size
    chunk_size = 15  # ~15 rows per chunk to stay under 8KB
    
    for i in range(0, len(data_lines), chunk_size):
        chunk_data = data_lines[i:i + chunk_size]
        if chunk_data:  # Only create chunk if there's data
            # Combine header + chunk data
            chunk_lines = header_lines + chunk_data
            chunks.append('\n'.join(chunk_lines))
    
    return chunks

def extract_table_chunks(content):
    """Extract individual tables from markdown content with smart chunking"""
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
                    chunk_text = '\n'.join(current_chunk)
                    # Break large chunks into smaller pieces
                    if len(chunk_text) > 8000:  # 8KB limit for Qwen-7B
                        sub_chunks = break_large_table(current_chunk)
                        table_chunks.extend(sub_chunks)
                    else:
                        table_chunks.append(chunk_text)
                    current_chunk = []
                in_table = False
            else:
                # Might be part of table (like wrapped content)
                current_chunk.append(line)
    
    # Don't forget the last chunk
    if current_chunk:
        chunk_text = '\n'.join(current_chunk)
        if len(chunk_text) > 8000:
            sub_chunks = break_large_table(current_chunk)
            table_chunks.extend(sub_chunks)
        else:
            table_chunks.append(chunk_text)
        
    return table_chunks

def has_misplaced_qualifiers(table_text):
    """Check if a table chunk has obvious misplaced qualifiers"""
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
    print(f"🔍 Found {len(chunks)} table chunks after smart chunking")
    
    problem_chunks = []
    
    for i, chunk in enumerate(chunks):
        has_qualifiers = has_misplaced_qualifiers(chunk)
        chunk_size = len(chunk)
        lines = chunk.count('\n') + 1
        
        if has_qualifiers:
            problem_chunks.append((i+1, chunk_size, lines, chunk))
            print(f"\n📋 Problem Chunk {i+1}:")
            print(f"   Size: {chunk_size:,} characters")
            print(f"   Lines: {lines}")
            
            if chunk_size > 5000:  # Still too big
                print(f"   ⚠️ Still too large for Qwen-7B!")
                
                # Show a sample of the table structure
                first_lines = chunk.split('\n')[:10]
                print(f"   Sample structure:")
                for line in first_lines:
                    print(f"     {line[:100]}...")
    
    print(f"\n📊 Summary:")
    print(f"   Total chunks: {len(chunks)}")
    print(f"   Problem chunks: {len(problem_chunks)}")
    
    # Show the most problematic chunk
    if problem_chunks:
        worst_chunk = max(problem_chunks, key=lambda x: x[1])  # Largest by size
        chunk_num, size, lines, chunk_text = worst_chunk
        print(f"\n🔥 Worst chunk #{chunk_num}: {size:,} chars, {lines} lines")
        print("   First 500 characters:")
        print(chunk_text[:500])

if __name__ == "__main__":
    main()
