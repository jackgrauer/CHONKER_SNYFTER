#!/usr/bin/env python3
"""Create a simple text-based PDF with various fonts for testing"""

from reportlab.lib.pagesizes import letter
from reportlab.pdfgen import canvas
from reportlab.lib.units import inch

def create_test_pdf(output_path):
    """Create a PDF with various font sizes and styles"""
    c = canvas.Canvas(output_path, pagesize=letter)
    width, height = letter
    
    # Title in large font
    c.setFont("Helvetica-Bold", 24)
    c.drawString(1*inch, height - 1*inch, "Font Test Document")
    
    # Subtitle
    c.setFont("Helvetica", 18)
    c.drawString(1*inch, height - 1.5*inch, "Testing Font Extraction")
    
    # Regular paragraph
    c.setFont("Times-Roman", 12)
    y = height - 2.5*inch
    c.drawString(1*inch, y, "This is regular body text in Times Roman 12pt.")
    
    # Bold text
    c.setFont("Times-Bold", 12)
    y -= 0.5*inch
    c.drawString(1*inch, y, "This text is bold.")
    
    # Italic text
    c.setFont("Times-Italic", 12)
    y -= 0.5*inch
    c.drawString(1*inch, y, "This text is italic.")
    
    # Different sizes
    c.setFont("Helvetica", 10)
    y -= 0.5*inch
    c.drawString(1*inch, y, "Small text - 10pt")
    
    c.setFont("Helvetica", 14)
    y -= 0.5*inch
    c.drawString(1*inch, y, "Medium text - 14pt")
    
    c.setFont("Helvetica", 16)
    y -= 0.5*inch
    c.drawString(1*inch, y, "Large text - 16pt")
    
    # Courier (monospace)
    c.setFont("Courier", 12)
    y -= 1*inch
    c.drawString(1*inch, y, "def hello_world():")
    c.drawString(1.5*inch, y - 0.3*inch, "print('This is monospace font')")
    
    c.save()
    print(f"Created test PDF: {output_path}")

if __name__ == "__main__":
    create_test_pdf("font_test.pdf")