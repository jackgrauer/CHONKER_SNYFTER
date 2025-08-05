#!/usr/bin/env python3
"""Create a simple test PDF with known text positioning"""

from reportlab.pdfgen import canvas
from reportlab.lib.pagesizes import letter

def create_test_pdf():
    c = canvas.Canvas("simple_test.pdf", pagesize=letter)
    width, height = letter
    
    # Simple text at known positions
    c.setFont("Helvetica", 12)
    c.drawString(100, height - 100, "Hello World")
    c.drawString(100, height - 130, "This is line 2")
    c.drawString(100, height - 160, "ABCDEFGHIJKLMNOP")
    
    # Different font size
    c.setFont("Helvetica", 18)
    c.drawString(100, height - 200, "Bigger Text")
    
    # Monospace font
    c.setFont("Courier", 10)
    c.drawString(100, height - 250, "Monospace: ABC123")
    
    c.save()
    print("Created simple_test.pdf")

if __name__ == "__main__":
    create_test_pdf()