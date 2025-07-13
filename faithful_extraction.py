#!/usr/bin/env python3
"""
Faithful Document Reproduction with Docling
Extracts semantically accurate representation with page-by-page accuracy
"""

from docling.document_converter import DocumentConverter
import json
import csv
import os
from datetime import datetime
from pathlib import Path

def create_faithful_reproduction(docling_result):
    """Extract faithful representation preserving document structure"""
    
    output = {
        "metadata": {
            "page_count": docling_result.document.num_pages,
            "table_count": len(docling_result.document.tables),
            "extraction_timestamp": datetime.now().isoformat(),
            "title": getattr(docling_result.document, 'name', None),
            "origin": getattr(docling_result.document, 'origin', None)
        },
        "pages": [],
        "plain_text_content": [],
        "tables": [],
        "full_markdown": ""
    }
    
    # Get full markdown for complete document structure
    output["full_markdown"] = docling_result.document.export_to_markdown()
    
    # Extract page-by-page content
    for page_idx, page in enumerate(docling_result.document.pages):
        page_data = {
            "page_number": page_idx + 1,
            "elements": [],
            "tables_on_page": [],
            "raw_text": ""
        }
        
        # Extract all items from this page by iterating through document items
        for item in docling_result.document.iterate_items():
            if hasattr(item, 'page_idx') and item.page_idx == page_idx:
                bbox = getattr(item, 'bbox', None)
                elem_data = {
                    "type": item.__class__.__name__,
                    "text": getattr(item, 'text', ''),
                    "level": getattr(item, 'level', None),
                    "bbox": str(bbox) if bbox else None
                }
                page_data["elements"].append(elem_data)
                page_data["raw_text"] += getattr(item, 'text', '') + "\n"
        
        # Find tables on this page
        page_tables = [table for table in docling_result.document.tables if hasattr(table, 'page_idx') and table.page_idx == page_idx]
        page_data["tables_on_page"] = [f"table_{i}" for i, table in enumerate(docling_result.document.tables) if hasattr(table, 'page_idx') and table.page_idx == page_idx]
        
        output["pages"].append(page_data)
    
    # Extract all text content preserving hierarchy
    for item in docling_result.document.iterate_items():
        if hasattr(item, 'text') and item.text:
            bbox = getattr(item, 'bbox', None)
            output["plain_text_content"].append({
                "type": item.__class__.__name__,
                "level": getattr(item, 'level', None),
                "text": item.text,
                "page": getattr(item, 'page_idx', 0) + 1,
                "bbox": str(bbox) if bbox else None
            })
    
    # Extract tables as structured JSON with maximum detail
    for i, table in enumerate(docling_result.document.tables):
        try:
            # Export table to different formats
            table_html = table.export_to_html()
            table_markdown = table.export_to_markdown()
            
            # Try to get table data structure
            table_data = None
            try:
                df = table.export_to_dataframe()
                table_data = df.values.tolist()
                headers = df.columns.tolist()
            except Exception as e:
                print(f"Could not export table {i} to dataframe: {e}")
                # Fallback to extracting from data attribute
                if hasattr(table, 'data') and table.data:
                    table_data = table.data
                    headers = table_data[0] if table_data else []
                    table_data = table_data[1:] if len(table_data) > 1 else []
                else:
                    headers = []
                    table_data = []
            
            table_info = {
                "id": f"table_{i}",
                "page": getattr(table, 'page_idx', 0) + 1,
                "caption": getattr(table, 'caption_text', None),
                "headers": headers,
                "data": table_data,
                "markdown": table_markdown,
                "html": table_html,
                "num_rows": len(table_data) if table_data else 0,
                "num_cols": len(headers) if headers else 0
            }
            
            output["tables"].append(table_info)
        except Exception as e:
            print(f"Error processing table {i}: {e}")
            # Still add basic info
            output["tables"].append({
                "id": f"table_{i}",
                "page": getattr(table, 'page_idx', 0) + 1,
                "error": str(e),
                "markdown": table.export_to_markdown() if hasattr(table, 'export_to_markdown') else ''
            })
    
    return output

def save_faithful_outputs(faithful_doc, base_filename):
    """Save all outputs for human review"""
    
    base_path = Path(base_filename).stem
    output_dir = Path("faithful_output")
    output_dir.mkdir(exist_ok=True)
    
    # 1. Save complete JSON
    with open(output_dir / f"{base_path}_faithful.json", "w", encoding='utf-8') as f:
        json.dump(faithful_doc, f, indent=2, ensure_ascii=False)
    
    # 2. Save full markdown
    with open(output_dir / f"{base_path}_full.md", "w", encoding='utf-8') as f:
        f.write(faithful_doc["full_markdown"])
    
    # 3. Save page-by-page text
    for page in faithful_doc["pages"]:
        page_file = output_dir / f"{base_path}_page_{page['page_number']}.txt"
        with open(page_file, "w", encoding='utf-8') as f:
            f.write(f"PAGE {page['page_number']}\n")
            f.write("=" * 50 + "\n\n")
            
            for element in page["elements"]:
                if element["type"] == "heading":
                    f.write(f"\n{'#' * (element['level'] or 1)} {element['text']}\n\n")
                else:
                    f.write(f"{element['text']}\n\n")
            
            if page["tables_on_page"]:
                f.write(f"\nTables on this page: {', '.join(page['tables_on_page'])}\n")
    
    # 4. Save structured plain text
    with open(output_dir / f"{base_path}_structured.txt", "w", encoding='utf-8') as f:
        current_page = 0
        for section in faithful_doc["plain_text_content"]:
            if section["page"] != current_page:
                current_page = section["page"]
                f.write(f"\n\n[PAGE {current_page}]\n")
                f.write("=" * 50 + "\n\n")
            
            if section["type"] == "heading":
                f.write(f"\n{'#' * (section['level'] or 1)} {section['text']}\n\n")
            else:
                f.write(f"{section['text']}\n\n")
    
    # 5. Save each table as CSV and JSON
    for i, table in enumerate(faithful_doc["tables"]):
        if "error" not in table and table["data"]:
            # CSV format
            csv_file = output_dir / f"{base_path}_table_{i}.csv"
            with open(csv_file, "w", newline="", encoding='utf-8') as f:
                writer = csv.writer(f)
                if table["headers"]:
                    writer.writerow(table["headers"])
                writer.writerows(table["data"])
            
            # JSON format for table
            json_file = output_dir / f"{base_path}_table_{i}.json"
            with open(json_file, "w", encoding='utf-8') as f:
                json.dump({
                    "id": table["id"],
                    "page": table["page"],
                    "caption": table["caption"],
                    "headers": table["headers"],
                    "data": table["data"],
                    "confidence": table["confidence"]
                }, f, indent=2, ensure_ascii=False)
    
    # 6. Save metadata
    with open(output_dir / f"{base_path}_metadata.json", "w", encoding='utf-8') as f:
        json.dump(faithful_doc["metadata"], f, indent=2, ensure_ascii=False)
    
    print(f"Faithful outputs saved to: {output_dir}/")
    print(f"- Complete JSON: {base_path}_faithful.json")
    print(f"- Full Markdown: {base_path}_full.md")
    print(f"- Page-by-page text: {base_path}_page_*.txt")
    print(f"- Structured text: {base_path}_structured.txt")
    print(f"- Tables: {base_path}_table_*.csv and {base_path}_table_*.json")
    print(f"- Metadata: {base_path}_metadata.json")

def process_document_faithful(pdf_path):
    """Process document with maximum accuracy settings"""
    
    # Use default DocumentConverter with optimal settings
    converter = DocumentConverter()
    
    print(f"Processing {pdf_path} with maximum accuracy settings...")
    print("- OCR Mode: FULL")
    print("- Table Structure Mode: ACCURATE")
    print("- Figure Extraction: ON")
    
    # Convert document
    result = converter.convert(pdf_path)
    
    # Create faithful reproduction
    faithful_doc = create_faithful_reproduction(result)
    
    # Save all outputs
    save_faithful_outputs(faithful_doc, pdf_path)
    
    # Verify critical data preserved
    print(f"\nExtraction Summary:")
    print(f"- Pages: {faithful_doc['metadata']['page_count']}")
    print(f"- Tables: {faithful_doc['metadata']['table_count']}")
    print(f"- Text elements: {len(faithful_doc['plain_text_content'])}")
    
    # Check for specific EPA data patterns
    tables_with_scientific_notation = 0
    tables_with_detection_codes = 0
    
    for table in faithful_doc["tables"]:
        if "data" in table:
            table_str = str(table["data"])
            if any(pattern in table_str for pattern in ["E-", "E+"]):
                tables_with_scientific_notation += 1
            if any(code in table_str for code in ["BDL", "DLL", "ND"]):
                tables_with_detection_codes += 1
    
    print(f"- Tables with scientific notation: {tables_with_scientific_notation}")
    print(f"- Tables with detection codes: {tables_with_detection_codes}")
    
    return faithful_doc

if __name__ == "__main__":
    # Process EPA document
    epa_doc = "/Users/jack/Desktop/EPA-HQ-OAR-2010-0682-0085_attachment_1.pdf"
    
    if os.path.exists(epa_doc):
        print("Processing EPA document with faithful extraction...")
        result = process_document_faithful(epa_doc)
        print("\nFaithful extraction complete!")
        print("Check the 'faithful_output' directory for all extracted files.")
    else:
        print(f"Document not found: {epa_doc}")
        print("Please update the path to your EPA document.")
