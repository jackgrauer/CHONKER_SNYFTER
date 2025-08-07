#!/usr/bin/env python3
"""Simple PDF creator for testing Phase 1 improvements"""

try:
    from reportlab.pdfgen import canvas
    from reportlab.lib.pagesizes import letter
    import os

    def create_test_pdf():
        filename = "test_document.pdf"
        c = canvas.Canvas(filename, pagesize=letter)
        
        # Add some test text with different font sizes
        c.setFont("Helvetica-Bold", 18)
        c.drawString(72, 720, "Phase 1 Test Document")
        
        c.setFont("Helvetica", 12)
        c.drawString(72, 680, "This is a sample document to test the character matrix improvements.")
        c.drawString(72, 660, "It contains various text elements at different font sizes.")
        
        c.setFont("Helvetica-Bold", 14)
        c.drawString(72, 620, "Important Section")
        
        c.setFont("Helvetica", 10)
        c.drawString(72, 580, "Small text: The quick brown fox jumps over the lazy dog.")
        c.drawString(72, 560, "More small text for testing font size detection algorithms.")
        
        c.setFont("Helvetica", 16)
        c.drawString(72, 520, "Larger Text Example")
        
        c.setFont("Helvetica", 12)
        c.drawString(72, 480, "Regular text continues here with multiple words on a single line.")
        c.drawString(72, 460, "Another line of regular text for comprehensive testing.")
        
        # Add some text at different positions to test bounds detection
        c.drawString(300, 400, "Right-aligned text")
        c.drawString(72, 380, "Left text")
        c.drawString(200, 380, "Center text")
        c.drawString(400, 380, "Right text")
        
        c.save()
        print(f"✅ Created {filename} successfully!")
        return filename

    if __name__ == "__main__":
        create_test_pdf()

except ImportError:
    print("❌ reportlab not available, creating minimal test file...")
    # Fallback: copy an existing PDF if available
    import shutil
    import os
    
    # Look for any existing PDF files
    pdf_files = [f for f in os.listdir('.') if f.endswith('.pdf')]
    if pdf_files:
        source = pdf_files[0]
        shutil.copy(source, "test_document.pdf")
        print(f"✅ Copied {source} to test_document.pdf")
    else:
        print("❌ No PDF files found and reportlab not available")
        print("   Please ensure a test PDF exists or install reportlab")