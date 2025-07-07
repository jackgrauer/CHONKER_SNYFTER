#!/usr/bin/env python3
import sys
from PyPDF2 import PdfReader, PdfWriter

def rotate_pdf_to_portrait(input_path, output_path):
    """Rotate a landscape PDF 90 degrees to make it portrait"""
    reader = PdfReader(input_path)
    writer = PdfWriter()
    
    for page in reader.pages:
        # Get current page dimensions
        box = page.mediabox
        width = float(box.width)
        height = float(box.height)
        
        print(f"Original page size: {width} x {height}")
        
        # If landscape (width > height), rotate 90 degrees clockwise
        if width > height:
            print("Rotating landscape page 90 degrees clockwise...")
            page.rotate(90)
        
        writer.add_page(page)
    
    with open(output_path, 'wb') as output_file:
        writer.write(output_file)
    
    print(f"Rotated PDF saved to: {output_path}")

if __name__ == "__main__":
    if len(sys.argv) != 3:
        print("Usage: python rotate_pdf.py input.pdf output.pdf")
        sys.exit(1)
    
    input_file = sys.argv[1]
    output_file = sys.argv[2]
    
    rotate_pdf_to_portrait(input_file, output_file)
