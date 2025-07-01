#!/usr/bin/env python3
"""
Quick test script to measure Docling processing speed
"""
import time
import sys
from pathlib import Path

def test_docling_speed(pdf_path):
    try:
        print(f"🔍 Testing Docling speed on: {pdf_path}")
        
        # Start timer
        start_time = time.time()
        
        # Import and process
        from docling.document_converter import DocumentConverter
        
        # Initialize converter
        converter = DocumentConverter()
        
        init_time = time.time()
        print(f"⏱️ Docling initialization took: {init_time - start_time:.2f}s")
        
        # Process the document
        result = converter.convert(pdf_path)
        
        end_time = time.time()
        processing_time = end_time - init_time
        total_time = end_time - start_time
        
        print(f"⏱️ Document processing took: {processing_time:.2f}s")
        print(f"⏱️ Total time (init + processing): {total_time:.2f}s")
        
        # Get some stats
        if hasattr(result, 'document') and hasattr(result.document, 'pages'):
            page_count = len(result.document.pages)
            print(f"📄 Pages processed: {page_count}")
            print(f"⚡ Time per page: {processing_time / page_count:.2f}s")
        
        # Get text length
        markdown_text = result.document.export_to_markdown()
        print(f"📝 Generated text length: {len(markdown_text)} characters")
        
        return True
        
    except ImportError:
        print("❌ Docling not installed. Install with: pip install docling")
        return False
    except Exception as e:
        print(f"❌ Error processing with Docling: {e}")
        return False

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python test_docling_speed.py <pdf_path>")
        print("Example: python test_docling_speed.py /Users/jack/Documents/1.pdf")
        sys.exit(1)
    
    pdf_path = sys.argv[1]
    if not Path(pdf_path).exists():
        print(f"❌ File not found: {pdf_path}")
        sys.exit(1)
    
    test_docling_speed(pdf_path)
