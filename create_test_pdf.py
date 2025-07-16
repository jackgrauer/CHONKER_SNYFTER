#!/usr/bin/env python3
"""
Create a test PDF for debugging CHONKER & SNYFTER
"""

from reportlab.lib.pagesizes import letter
from reportlab.pdfgen import canvas
from reportlab.lib.units import inch
from reportlab.lib.colors import HexColor
from datetime import datetime
import os

def create_test_pdf(filename="test_document.pdf"):
    """Create a test PDF with various content types"""
    
    # Create canvas
    c = canvas.Canvas(filename, pagesize=letter)
    width, height = letter
    
    # Page 1 - Title and Introduction
    c.setFont("Helvetica-Bold", 24)
    c.drawString(1*inch, height - 1*inch, "CHONKER & SNYFTER Test Document")
    
    c.setFont("Helvetica", 12)
    c.drawString(1*inch, height - 1.5*inch, f"Generated on: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    
    c.setFont("Helvetica", 14)
    y_position = height - 2.5*inch
    
    # Introduction paragraph
    intro_text = [
        "This is a test document created to debug the CHONKER & SNYFTER application.",
        "It contains various types of content including headings, paragraphs, lists,",
        "and tables to test the document processing capabilities."
    ]
    
    for line in intro_text:
        c.drawString(1*inch, y_position, line)
        y_position -= 20
    
    # Section 1
    y_position -= 30
    c.setFont("Helvetica-Bold", 16)
    c.drawString(1*inch, y_position, "1. Features of CHONKER")
    
    y_position -= 25
    c.setFont("Helvetica", 12)
    features = [
        "• PDF Processing - Extracts content from PDF files",
        "• De-scuzzifying - Cleans up messy PDFs",
        "• Chunk Extraction - Breaks documents into manageable pieces",
        "• Batch Processing - Handle multiple files at once"
    ]
    
    for feature in features:
        c.drawString(1.5*inch, y_position, feature)
        y_position -= 20
    
    # Section 2
    y_position -= 30
    c.setFont("Helvetica-Bold", 16)
    c.drawString(1*inch, y_position, "2. Features of SNYFTER")
    
    y_position -= 25
    c.setFont("Helvetica", 12)
    snyfter_features = [
        "• Database Storage - Archives all processed documents",
        "• Full-Text Search - Find content quickly",
        "• Export Options - JSON, CSV, Markdown formats",
        "• Metadata Tracking - Keep track of processing details"
    ]
    
    for feature in snyfter_features:
        c.drawString(1.5*inch, y_position, feature)
        y_position -= 20
    
    # Add a table-like structure
    y_position -= 40
    c.setFont("Helvetica-Bold", 16)
    c.drawString(1*inch, y_position, "3. Processing Statistics")
    
    y_position -= 30
    c.setFont("Helvetica", 12)
    
    # Table headers
    c.drawString(1.5*inch, y_position, "Document Type")
    c.drawString(3.5*inch, y_position, "Processing Time")
    c.drawString(5.5*inch, y_position, "Success Rate")
    
    y_position -= 20
    c.line(1.5*inch, y_position, 7*inch, y_position)
    
    # Table data
    y_position -= 15
    table_data = [
        ("Simple PDF", "0.5 seconds", "100%"),
        ("Complex PDF", "2.3 seconds", "95%"),
        ("Scanned PDF", "5.1 seconds", "85%"),
        ("Annotated PDF", "1.2 seconds", "98%")
    ]
    
    for row in table_data:
        c.drawString(1.5*inch, y_position, row[0])
        c.drawString(3.5*inch, y_position, row[1])
        c.drawString(5.5*inch, y_position, row[2])
        y_position -= 20
    
    # Page 2
    c.showPage()
    
    # Technical details
    c.setFont("Helvetica-Bold", 20)
    c.drawString(1*inch, height - 1*inch, "Technical Implementation")
    
    y_position = height - 2*inch
    c.setFont("Helvetica-Bold", 14)
    c.drawString(1*inch, y_position, "Architecture Overview")
    
    y_position -= 30
    c.setFont("Helvetica", 12)
    arch_details = [
        "The system uses a modular architecture with the following components:",
        "",
        "1. PDF Processing Engine (CHONKER)",
        "   - Uses PyMuPDF for PDF manipulation",
        "   - Docling for content extraction",
        "   - Multi-threaded processing for performance",
        "",
        "2. Database Layer (SNYFTER)",
        "   - SQLite with FTS5 for full-text search",
        "   - Structured schema for documents and chunks",
        "   - Efficient indexing for fast retrieval",
        "",
        "3. User Interface",
        "   - PyQt6 for modern, responsive UI",
        "   - Dark theme with animations",
        "   - Real-time progress feedback"
    ]
    
    for line in arch_details:
        c.drawString(1*inch if line and not line.startswith("   ") else 1.5*inch, y_position, line)
        y_position -= 18
    
    # Code example
    y_position -= 30
    c.setFont("Helvetica-Bold", 14)
    c.drawString(1*inch, y_position, "Code Example")
    
    y_position -= 25
    c.setFont("Courier", 10)
    code_lines = [
        "# Process a PDF with CHONKER",
        "worker = EnhancedChonkerWorker(pdf_path)",
        "worker.finished.connect(on_processing_complete)",
        "worker.start()",
        "",
        "# Search with SNYFTER",
        "results = db.search_documents('machine learning')",
        "for doc in results:",
        "    print(f'Found: {doc[\"filename\"]}')"
    ]
    
    for line in code_lines:
        c.drawString(1.5*inch, y_position, line)
        y_position -= 14
    
    # Footer
    c.setFont("Helvetica-Oblique", 10)
    c.drawString(1*inch, 0.5*inch, "CHONKER & SNYFTER - Making document processing fun!")
    
    # Save the PDF
    c.save()
    
    print(f"✅ Test PDF created: {filename}")
    return os.path.abspath(filename)

if __name__ == "__main__":
    # Check if reportlab is installed
    try:
        import reportlab
        pdf_path = create_test_pdf()
        print(f"📄 Test PDF location: {pdf_path}")
    except ImportError:
        print("❌ reportlab not installed. Installing...")
        import subprocess
        subprocess.check_call(["pip", "install", "reportlab"])
        print("✅ reportlab installed. Please run the script again.")