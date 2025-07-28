#!/usr/bin/env python3
"""Strip text layer from PDF, keeping only the visual content."""

import sys
from pathlib import Path
try:
    import fitz  # PyMuPDF
except ImportError:
    print("Installing PyMuPDF...")
    import subprocess
    subprocess.check_call([sys.executable, "-m", "pip", "install", "PyMuPDF"])
    import fitz

def strip_text_layer(input_path, output_path, quality_scale=1.0):
    """Remove text layer from PDF while preserving visual content."""
    # Open the PDF
    doc = fitz.open(input_path)
    
    # Create a new PDF
    new_doc = fitz.open()
    
    for page_num in range(len(doc)):
        # Get the page
        page = doc[page_num]
        
        # Get page dimensions
        rect = page.rect
        
        # Create a new page with same dimensions
        new_page = new_doc.new_page(width=rect.width, height=rect.height)
        
        # Check if page has images
        image_list = page.get_images()
        
        if image_list:
            # If page has images, extract and re-insert them directly
            for img_index, img in enumerate(image_list):
                xref = img[0]
                pix = fitz.Pixmap(doc, xref)
                
                # Insert the original image
                new_page.insert_image(rect, pixmap=pix)
                pix = None
        else:
            # Only render as image if no existing images (text-only page)
            # Use lower DPI for smaller file size
            mat = fitz.Matrix(quality_scale, quality_scale)
            pix = page.get_pixmap(matrix=mat, alpha=False)
            
            # Insert the image
            new_page.insert_image(rect, pixmap=pix)
            pix = None
        
        print(f"Processed page {page_num + 1}/{len(doc)}")
    
    # Save with compression
    new_doc.save(output_path, deflate=True, garbage=4, clean=True)
    new_doc.close()
    doc.close()
    
    # Get file sizes for comparison
    input_size = Path(input_path).stat().st_size / 1024 / 1024
    output_size = Path(output_path).stat().st_size / 1024 / 1024
    
    print(f"\nCreated {output_path} without text layer")
    print(f"Input size: {input_size:.2f} MB")
    print(f"Output size: {output_size:.2f} MB")
    print("This PDF contains only images, no extractable text.")

if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Usage: python strip_text_layer.py <input.pdf> <output.pdf> [quality_scale]")
        print("  quality_scale: Optional, default 1.0. Use 0.5 for smaller files.")
        sys.exit(1)
    
    input_path = Path(sys.argv[1])
    output_path = Path(sys.argv[2])
    quality_scale = float(sys.argv[3]) if len(sys.argv) > 3 else 1.0
    
    if not input_path.exists():
        print(f"Error: {input_path} does not exist")
        sys.exit(1)
    
    try:
        strip_text_layer(input_path, output_path, quality_scale)
    except Exception as e:
        print(f"Error: {e}")
        sys.exit(1)