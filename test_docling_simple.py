#!/usr/bin/env python3
"""
Simple test for Docling PDF processing
"""

import os
from rich.console import Console

console = Console()

def test_docling_simple():
    """Test Docling directly"""
    console.print("[bold]Testing Docling PDF Processing[/bold]\n")
    
    test_pdf = "test_document.pdf"
    if not os.path.exists(test_pdf):
        console.print("[red]Test PDF not found[/red]")
        return
    
    try:
        from docling.document_converter import DocumentConverter
        
        console.print("📄 Creating DocumentConverter...")
        converter = DocumentConverter()
        
        console.print(f"📄 Processing {test_pdf}...")
        result = converter.convert(test_pdf)
        
        console.print("✅ Processing successful!")
        
        # Check what we got
        console.print("\n📊 Results:")
        console.print(f"  • Document type: {type(result.document)}")
        
        # Try to iterate items
        items = list(result.document.iterate_items())
        console.print(f"  • Total items: {len(items)}")
        
        # Show first few items
        console.print("\n📝 First few items:")
        for i, (item, level) in enumerate(items[:5]):
            item_type = type(item).__name__
            text = getattr(item, 'text', str(item))[:100]
            console.print(f"  [{i}] {item_type} (level {level}): {text}...")
        
        # Try exports
        console.print("\n📤 Testing exports:")
        try:
            html = result.document.export_to_html()
            console.print(f"  • HTML export: {len(html)} characters")
        except Exception as e:
            console.print(f"  • HTML export failed: {e}")
        
        try:
            markdown = result.document.export_to_markdown()
            console.print(f"  • Markdown export: {len(markdown)} characters")
        except Exception as e:
            console.print(f"  • Markdown export failed: {e}")
        
    except Exception as e:
        console.print(f"\n[red]Error: {e}[/red]")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    test_docling_simple()