#!/usr/bin/env python3
"""
Create a simple test PDF for chonker5 validation
"""

try:
    from reportlab.pdfgen import canvas
    from reportlab.lib.pagesizes import letter
    from reportlab.lib import colors
    from reportlab.pdfbase import pdfmetrics
    from reportlab.pdfbase.ttfonts import TTFont
    
    def create_test_pdf():
        filename = "chonker_test.pdf"
        c = canvas.Canvas(filename, pagesize=letter)
        width, height = letter
        
        # Title
        c.setFont("Helvetica-Bold", 16)
        c.drawString(100, height-100, "CHONKER 5 TEST DOCUMENT")
        
        # Subtitle
        c.setFont("Helvetica", 12)
        c.drawString(100, height-130, "Character Matrix PDF Engine Test File")
        
        # Simple table-like structure
        c.setFont("Helvetica", 10)
        y_pos = height - 200
        
        # Header row
        c.drawString(100, y_pos, "Item")
        c.drawString(200, y_pos, "Description")
        c.drawString(350, y_pos, "Value")
        
        # Data rows
        data = [
            ("1", "Character Matrix Width", "120"),
            ("2", "Character Matrix Height", "80"),
            ("3", "Text Region Count", "15"),
            ("4", "Processing Time", "2.5s"),
            ("5", "Confidence Score", "0.92")
        ]
        
        y_pos -= 30
        for item, desc, value in data:
            c.drawString(100, y_pos, item)
            c.drawString(200, y_pos, desc)
            c.drawString(350, y_pos, value)
            y_pos -= 20
        
        # Add some text regions with different formatting
        y_pos -= 50
        c.setFont("Helvetica-Bold", 14)
        c.drawString(100, y_pos, "Text Extraction Test")
        
        y_pos -= 30
        c.setFont("Helvetica", 10)
        text_lines = [
            "This is a test paragraph to verify text extraction capabilities.",
            "The character matrix engine should be able to identify text regions",
            "and extract the content with proper spatial relationships preserved.",
            "",
            "Special characters: àáâãäåæçèéêë àáâãäåæçèéêë",
            "Numbers: 1234567890",
            "Symbols: !@#$%^&*()_+-=[]{}|;:,.<>?"
        ]
        
        for line in text_lines:
            c.drawString(100, y_pos, line)
            y_pos -= 15
        
        # Add some positioned text elements
        c.setFont("Helvetica", 8)
        c.drawString(400, height-300, "Corner text")
        c.drawString(50, 100, "Bottom left")
        c.drawString(400, 100, "Bottom right")
        
        c.save()
        print(f"✅ Created test PDF: {filename}")
        return filename
    
    if __name__ == "__main__":
        create_test_pdf()

except ImportError:
    print("❌ reportlab not available. Creating minimal test file instead.")
    # Create a simple text file as placeholder
    with open("chonker_test.txt", "w") as f:
        f.write("CHONKER 5 TEST DOCUMENT\n")
        f.write("This is a minimal test file since reportlab is not available.\n")
        f.write("For full testing, install reportlab: pip install reportlab\n")
    print("✅ Created minimal test file: chonker_test.txt")