#!/usr/bin/env python3
"""
CHONKER Document Processor
Process any document with Docling and generate an HTML viewer.
"""

import os
import sys
import json
import tempfile
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Optional

# Import the viewer generator
from generate_viewer import generate_document_viewer


def process_document(file_path: str) -> Dict:
    """
    Process a document with Docling and extract content.
    
    Args:
        file_path: Path to the document to process
        
    Returns:
        Dictionary containing extracted content
    """
    try:
        # Import Docling
        from docling.document_converter import DocumentConverter
        
        print(f"🔍 Processing: {file_path}")
        
        # Create converter
        converter = DocumentConverter()
        
        # Convert document
        result = converter.convert(file_path)
        
        # Extract text
        extracted_text = result.document.export_to_markdown()
        
        # Extract tables (simplified extraction)
        tables = []
        if hasattr(result.document, 'tables') and result.document.tables:
            for i, table in enumerate(result.document.tables):
                try:
                    table_data = {
                        'id': f"table_{i}",
                        'headers': table.get_headers() if hasattr(table, 'get_headers') else [],
                        'rows': table.get_rows() if hasattr(table, 'get_rows') else [],
                        'caption': table.caption if hasattr(table, 'caption') else None
                    }
                    tables.append(table_data)
                except Exception as e:
                    print(f"⚠️  Warning: Could not extract table {i}: {e}")
                    # Create a simple table representation
                    tables.append({
                        'id': f"table_{i}",
                        'headers': [],
                        'rows': [],
                        'caption': f"Table {i+1} (extraction failed)"
                    })
        
        # Extract metadata
        metadata = {
            'title': getattr(result.document, 'title', None) or Path(file_path).stem,
            'author': getattr(result.document, 'author', None),
            'page_count': getattr(result.document, 'page_count', None),
            'extracted_at': datetime.now().isoformat()
        }
        
        print(f"✅ Extracted {len(extracted_text)} characters")
        print(f"✅ Found {len(tables)} tables")
        print(f"✅ Metadata: {metadata['title']}")
        
        return {
            'text': extracted_text,
            'tables': tables,
            'metadata': metadata
        }
        
    except ImportError:
        print("❌ Error: Docling not installed. Install with: pip install docling")
        sys.exit(1)
    except Exception as e:
        print(f"❌ Error processing document: {e}")
        sys.exit(1)


def save_extraction_results(base_name: str, content: Dict, output_dir: str = "apps/doc-service/processed_documents"):
    """
    Save extraction results to separate files.
    
    Args:
        base_name: Base name for output files
        content: Extracted content dictionary
        output_dir: Directory to save files
    """
    output_path = Path(output_dir)
    output_path.mkdir(parents=True, exist_ok=True)
    
    output_files = []
    
    # Save text
    text_file = output_path / f"{base_name}_text.md"
    with open(text_file, 'w', encoding='utf-8') as f:
        f.write(content['text'])
    output_files.append(str(text_file))
    
    # Save tables
    tables_file = output_path / f"{base_name}_tables.json"
    with open(tables_file, 'w', encoding='utf-8') as f:
        json.dump(content['tables'], f, indent=2, ensure_ascii=False)
    output_files.append(str(tables_file))
    
    # Save metadata
    metadata_file = output_path / f"{base_name}_metadata.json"
    with open(metadata_file, 'w', encoding='utf-8') as f:
        json.dump(content['metadata'], f, indent=2, ensure_ascii=False)
    output_files.append(str(metadata_file))
    
    print(f"💾 Saved extraction results:")
    for file_path in output_files:
        print(f"   📄 {file_path}")
    
    return output_files


def main():
    """
    Command-line interface for document processing.
    Usage: python process_document.py <document_path>
    """
    if len(sys.argv) != 2:
        print("Usage: python process_document.py <document_path>")
        print("Example: python process_document.py mydocument.pdf")
        print("")
        print("Supported formats:")
        print("  • PDF (.pdf)")
        print("  • Word (.docx)")
        print("  • PowerPoint (.pptx)")
        print("  • HTML (.html)")
        print("  • Markdown (.md)")
        print("  • CSV (.csv)")
        print("  • Excel (.xlsx)")
        print("  • AsciiDoc (.asciidoc)")
        sys.exit(1)
    
    document_path = sys.argv[1]
    
    # Check if file exists
    if not os.path.exists(document_path):
        print(f"❌ Error: File '{document_path}' not found")
        sys.exit(1)
    
    # Get base name for output files
    base_name = Path(document_path).stem
    
    print(f"🚀 CHONKER Document Processor")
    print(f"📄 Document: {document_path}")
    print(f"📝 Base name: {base_name}")
    print("")
    
    # Process the document
    content = process_document(document_path)
    
    # Save extraction results
    output_files = save_extraction_results(base_name, content)
    
    # Generate HTML viewer
    print("")
    print("🎨 Generating HTML viewer...")
    
    viewer_path = f"{base_name}_viewer.html"
    generated_path = generate_document_viewer(
        document_text=content['text'],
        tables=content['tables'],
        metadata=content['metadata'],
        output_path=viewer_path,
        document_name=content['metadata']['title']
    )
    
    print(f"✅ Generated HTML viewer: {generated_path}")
    print("")
    print("🎉 Processing complete!")
    print(f"📖 Open viewer: open {generated_path}")
    print(f"📁 View files: ls -la {os.path.dirname(output_files[0])}")


if __name__ == "__main__":
    main()
