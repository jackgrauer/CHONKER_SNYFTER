#!/usr/bin/env python3
"""Strip text layer from PDF efficiently by removing text operations from content streams."""

import sys
from pathlib import Path
try:
    from pypdf import PdfReader, PdfWriter
    from pypdf.generic import ContentStream, TextStringObject, NameObject
except ImportError:
    print("Installing pypdf...")
    import subprocess
    subprocess.check_call([sys.executable, "-m", "pip", "install", "pypdf"])
    from pypdf import PdfReader, PdfWriter
    from pypdf.generic import ContentStream, TextStringObject, NameObject

def remove_text_from_content(content):
    """Remove text-related operations from a content stream."""
    # Text operators to remove
    text_operators = {
        b"BT", b"ET",  # Begin/End text
        b"Tf", b"Ts", b"Tz", b"TL", b"T*", b"Td", b"TD", b"Tm",  # Text state
        b"Tj", b"TJ", b"'", b'"',  # Show text
        b"Tc", b"Tw", b"Tr",  # Character/word spacing, rendering mode
    }
    
    new_operations = []
    i = 0
    operations = content.operations if hasattr(content, 'operations') else []
    
    in_text_block = False
    
    for operands, operator in operations:
        # Skip text blocks entirely
        if operator == b"BT":
            in_text_block = True
            continue
        elif operator == b"ET":
            in_text_block = False
            continue
        elif in_text_block or operator in text_operators:
            continue
        
        # Keep non-text operations (graphics, images, etc)
        new_operations.append((operands, operator))
    
    return new_operations

def strip_text_layer_efficiently(input_path, output_path):
    """Remove text layer while keeping graphics and images."""
    reader = PdfReader(input_path)
    writer = PdfWriter()
    
    for page_num, page in enumerate(reader.pages):
        # Create new page
        new_page = writer.add_page(page)
        
        # Get content streams
        if "/Contents" in new_page:
            contents = new_page["/Contents"]
            
            # Handle different content types
            if hasattr(contents, "get_data"):
                # Single content stream
                content = ContentStream(contents, reader)
                content.operations = remove_text_from_content(content)
                new_page[NameObject("/Contents")] = content
            elif isinstance(contents, list):
                # Multiple content streams
                new_contents = []
                for content_ref in contents:
                    content = ContentStream(content_ref, reader)
                    content.operations = remove_text_from_content(content)
                    new_contents.append(content)
                new_page[NameObject("/Contents")] = new_contents
        
        print(f"Processed page {page_num + 1}/{len(reader.pages)}")
    
    # Compress and save
    writer.compress_identical_objects(remove_use_from_pools=True)
    
    with open(output_path, "wb") as output_file:
        writer.write(output_file)
    
    # Get file sizes
    input_size = Path(input_path).stat().st_size / 1024
    output_size = Path(output_path).stat().st_size / 1024
    
    print(f"\nCreated {output_path} without text layer")
    print(f"Input size: {input_size:.1f} KB")
    print(f"Output size: {output_size:.1f} KB")

if __name__ == "__main__":
    if len(sys.argv) != 3:
        print("Usage: python strip_text_efficiently.py <input.pdf> <output.pdf>")
        sys.exit(1)
    
    input_path = Path(sys.argv[1])
    output_path = Path(sys.argv[2])
    
    if not input_path.exists():
        print(f"Error: {input_path} does not exist")
        sys.exit(1)
    
    try:
        strip_text_layer_efficiently(input_path, output_path)
    except Exception as e:
        print(f"Error: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)