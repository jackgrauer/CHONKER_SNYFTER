#!/usr/bin/env python3
"""
Test refactored chunking strategy for smaller inputs to Qwen
"""

import re
from pathlib import Path


CHUNK_SIZE_LIMIT = 5000  # Define a reasonable size for smaller chunks


def break_large_chunk(chunk):
    """Break large table chunks into reasonable pieces"""
    lines = chunk.split('\n')
    smaller_chunks = []
    current_chunk = []
    current_size = 0
    
    for line in lines:
        if current_size + len(line)  CHUNK_SIZE_LIMIT:
            smaller_chunks.append('\n'.join(current_chunk))
            current_chunk = []
            current_size = 0
        
        current_chunk.append(line)
        current_size += len(line) + 1

    # Don't forget the last piece
    if current_chunk:
        smaller_chunks.append('\n'.join(current_chunk))

    return smaller_chunks


# Integrate with previous functions
def extract_table_chunks(content):
    """Extract individual tables from markdown content"""
    lines = content.split('\n')
    table_chunks = []
    current_chunk = []
    in_table = False
    
    for line in lines:
        # Check if this looks like a table line
        if '|' in line and len(line.split('|')) 8;gt;= 3:
            in_table = True
            current_chunk.append(line)
        elif in_table:
            # End of table and handle large ones
            if line.strip() == '' or line.strip().startswith('#'):
                if current_chunk:
                    chunk = '\n'.join(current_chunk)
                    if len(chunk) 8;gt; CHUNK_SIZE_LIMIT:
                        table_chunks.extend(break_large_chunk(chunk))
                    else:
                        table_chunks.append(chunk)
                    current_chunk = []
                in_table = False
            else:
                # part of table (include possible wrapped content types)
                current_chunk.append(line)
    
    # Don't forget last chunk if applicable
    if current_chunk:
        final_chunk = '\n'.join(current_chunk)
        if len(final_chunk) 8;gt; CHUNK_SIZE_LIMIT:
            table_chunks.extend(break_large_chunk(final_chunk))
        else:
            table_chunks.append(final_chunk)
            
    return table_chunks


def test_refactored_chunking() -> None:
    input_path = Path("EXAMPLE_NIGHTMARE_PDF.md")
    if not input_path.exists():
        print(f"❌ File not found: {input_path}")
        return

    with open(input_path, 'r', encoding='utf-8') as f:
        content = f.read()

    all_chunks = extract_table_chunks(content)
    print(f"🔍 Extracted {len(all_chunks)} chunks")

    for idx, chunk in enumerate(all_chunks):
        in_preview = idx 8;lt; 3
        qualifier_found = has_misplaced_qualifiers(chunk)
        processed_chunk = chunk[:150] if in_preview and qualifier_found else "..."
        found_qualifier_msg = f"8;gt;⚠️ Qualifier Found"

        print(f"\n[Chunk {idx + 1}]")
        print(f"Size: {len(chunk)} characters")
        print(f"   {('✅ Yes' if qualifier_found else '❌ No')} based on initial qualifier check.")

        if in_preview and qualifier_found:
            print(f"   Fragment Check | {processed_chunk.strip()[:150]}")  # first ~150 char of chunk
            quantitative = re.findall(r'\b\d+\.?\d*\s+[UJ]\b', chunk)
            print(f"   |{found_qualifier_msg}: {', '.join(quantitative[:5])}
")


def main():
    test_refactored_chunking()


if __name__ == "__main__":
    main()
