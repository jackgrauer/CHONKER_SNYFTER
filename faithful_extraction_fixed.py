#!/usr/bin/env python3
"""
Faithful Document Reproduction with Docling - FIXED VERSION
Extracts semantically accurate representation with page-by-page accuracy
"""

from docling.document_converter import DocumentConverter
import json
import pandas as pd
from datetime import datetime
from pathlib import Path
import re
from html.parser import HTMLParser

class TableParser(HTMLParser):
    """Custom HTML parser to extract table data from Docling's HTML output"""
    
    def __init__(self):
        super().__init__()
        self.headers = []
        self.rows = []
        self.current_row = []
        self.in_header = False
        self.in_cell = False
        self.current_data = ""
        
    def handle_starttag(self, tag, attrs):
        if tag == "th":
            self.in_header = True
            self.in_cell = True
        elif tag == "td":
            self.in_cell = True
            
    def handle_endtag(self, tag):
        if tag == "th":
            self.headers.append(self.current_data.strip())
            self.current_data = ""
            self.in_header = False
            self.in_cell = False
        elif tag == "td":
            self.current_row.append(self.current_data.strip())
            self.current_data = ""
            self.in_cell = False
        elif tag == "tr" and self.current_row:
            self.rows.append(self.current_row)
            self.current_row = []
            
    def handle_data(self, data):
        if self.in_cell:
            self.current_data += data

def create_faithful_reproduction(docling_result):
    """Extract faithful representation preserving document structure"""
    
    output = {
        "metadata": {
            "page_count": len(docling_result.document.pages),
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
    for page_idx in range(len(docling_result.document.pages)):
        page_data = {
            "page_number": page_idx + 1,
            "elements": [],
            "tables_on_page": [],
            "raw_text": ""
        }
        
        # Extract all items from this page
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
        page_tables = []
        for i, table in enumerate(docling_result.document.tables):
            if hasattr(table, 'page_idx') and table.page_idx == page_idx:
                page_tables.append(f"table_{i}")
        page_data["tables_on_page"] = page_tables
        
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
    
    # Extract tables with maximum accuracy
    for i, table in enumerate(docling_result.document.tables):
        try:
            print(f"Processing table {i}...")
            
            # Method 1: Try pandas dataframe (most accurate)
            try:
                df = table.export_to_dataframe()
                table_data = {
                    "id": f"table_{i}",
                    "page": getattr(table, 'page_idx', 0) + 1,
                    "caption": getattr(table, 'caption_text', None),
                    "headers": df.columns.tolist(),
                    "data": df.values.tolist(),
                    "num_rows": len(df),
                    "num_cols": len(df.columns),
                    "extraction_method": "dataframe"
                }
                print(f"  ✓ Successfully extracted table {i} via DataFrame")
                
            except Exception as e:
                print(f"  DataFrame export failed for table {i}: {e}")
                
                # Method 2: Parse HTML output
                try:
                    html_content = table.export_to_html(docling_result.document)
                    parser = TableParser()
                    parser.feed(html_content)
                    
                    table_data = {
                        "id": f"table_{i}",
                        "page": getattr(table, 'page_idx', 0) + 1,
                        "caption": getattr(table, 'caption_text', None),
                        "headers": parser.headers,
                        "data": parser.rows,
                        "num_rows": len(parser.rows),
                        "num_cols": len(parser.headers),
                        "extraction_method": "html_parsing"
                    }
                    print(f"  ✓ Successfully extracted table {i} via HTML parsing")
                    
                except Exception as e2:
                    print(f"  HTML parsing failed for table {i}: {e2}")
                    
                    # Method 3: Try markdown output
                    try:
                        markdown_content = table.export_to_markdown(docling_result.document)
                        table_data = {
                            "id": f"table_{i}",
                            "page": getattr(table, 'page_idx', 0) + 1,
                            "caption": getattr(table, 'caption_text', None),
                            "markdown": markdown_content,
                            "extraction_method": "markdown_only",
                            "error": f"Structured extraction failed: {e} / {e2}"
                        }
                        print(f"  ⚠ Fallback to markdown for table {i}")
                        
                    except Exception as e3:
                        print(f"  All extraction methods failed for table {i}: {e3}")
                        table_data = {
                            "id": f"table_{i}",
                            "page": getattr(table, 'page_idx', 0) + 1,
                            "error": f"All extraction methods failed: {e} / {e2} / {e3}",
                            "extraction_method": "failed"
                        }
            
            output["tables"].append(table_data)
            
        except Exception as e:
            print(f"Critical error processing table {i}: {e}")
            output["tables"].append({
                "id": f"table_{i}",
                "page": getattr(table, 'page_idx', 0) + 1,
                "error": f"Critical error: {e}",
                "extraction_method": "failed"
            })
    
    return output

def clean_for_json(obj):
    """Recursively clean object for JSON serialization"""
    if isinstance(obj, (str, int, float, bool, type(None))):
        return obj
    elif isinstance(obj, dict):
        return {k: clean_for_json(v) for k, v in obj.items()}
    elif isinstance(obj, (list, tuple)):
        return [clean_for_json(item) for item in obj]
    else:
        return str(obj)

def save_faithful_outputs(faithful_doc, base_filename):
    """Save all outputs for human review"""
    
    base_path = Path(base_filename).stem
    output_dir = Path("faithful_output")
    output_dir.mkdir(exist_ok=True)
    
    # Clean data for JSON serialization
    cleaned_doc = clean_for_json(faithful_doc)
    
    # 1. Save complete JSON
    json_path = output_dir / f"{base_path}_faithful.json"
    with open(json_path, "w", encoding='utf-8') as f:
        json.dump(cleaned_doc, f, indent=2, ensure_ascii=False)
    
    # 2. Save full markdown
    md_path = output_dir / f"{base_path}_full.md"
    with open(md_path, "w", encoding='utf-8') as f:
        f.write(faithful_doc["full_markdown"])
    
    # 3. Save page-by-page text
    for page in faithful_doc["pages"]:
        page_file = output_dir / f"{base_path}_page_{page['page_number']}.txt"
        with open(page_file, "w", encoding='utf-8') as f:
            f.write(f"PAGE {page['page_number']}\n")
            f.write("=" * 50 + "\n\n")
            
            for element in page["elements"]:
                if element["type"] == "HeadingItem":
                    f.write(f"\n### {element['text']}\n\n")
                else:
                    f.write(f"{element['text']}\n\n")
            
            if page["tables_on_page"]:
                f.write(f"\nTables on this page: {', '.join(page['tables_on_page'])}\n")
    
    # 4. Save structured plain text
    text_path = output_dir / f"{base_path}_structured.txt"
    with open(text_path, "w", encoding='utf-8') as f:
        current_page = 0
        for section in faithful_doc["plain_text_content"]:
            if section["page"] != current_page:
                current_page = section["page"]
                f.write(f"\n\n[PAGE {current_page}]\n")
                f.write("=" * 50 + "\n\n")
            
            if section["type"] == "HeadingItem":
                f.write(f"\n### {section['text']}\n\n")
            else:
                f.write(f"{section['text']}\n\n")
    
    # 5. Save each table as CSV and JSON
    for i, table in enumerate(faithful_doc["tables"]):
        if "data" in table and "headers" in table and table["data"]:
            # CSV format
            csv_file = output_dir / f"{base_path}_table_{i}.csv"
            try:
                df = pd.DataFrame(table["data"], columns=table["headers"])
                df.to_csv(csv_file, index=False)
                print(f"  ✓ Saved table {i} as CSV")
            except Exception as e:
                print(f"  ✗ Failed to save table {i} as CSV: {e}")
            
            # JSON format for table
            json_file = output_dir / f"{base_path}_table_{i}.json"
            with open(json_file, "w", encoding='utf-8') as f:
                table_json_data = {
                    "id": table["id"],
                    "page": table["page"],
                    "caption": table.get("caption"),
                    "headers": table["headers"],
                    "data": table["data"],
                    "extraction_method": table.get("extraction_method", "unknown")
                }
                # Clean for JSON serialization
                clean_table_data = clean_for_json(table_json_data)
                json.dump(clean_table_data, f, indent=2, ensure_ascii=False)
        else:
            print(f"  ⚠ Table {i} has no structured data, skipping CSV export")
    
    # 6. Save metadata
    metadata_file = output_dir / f"{base_path}_metadata.json"
    with open(metadata_file, "w", encoding='utf-8') as f:
        clean_metadata = clean_for_json(faithful_doc["metadata"])
        json.dump(clean_metadata, f, indent=2, ensure_ascii=False)
    
    print(f"\n📁 Faithful outputs saved to: {output_dir}/")
    print(f"   - Complete JSON: {base_path}_faithful.json")
    print(f"   - Full Markdown: {base_path}_full.md")
    print(f"   - Page-by-page text: {base_path}_page_*.txt")
    print(f"   - Structured text: {base_path}_structured.txt")
    print(f"   - Tables: {base_path}_table_*.csv and {base_path}_table_*.json")
    print(f"   - Metadata: {base_path}_metadata.json")

def process_document_faithful(pdf_path):
    """Process document with maximum accuracy settings"""
    
    # Use default DocumentConverter with optimal settings
    converter = DocumentConverter()
    
    print(f"🔄 Processing {pdf_path} with faithful extraction...")
    print("   - Using default Docling pipeline for stability")
    print("   - Multiple extraction methods per table")
    print("   - Page-by-page content preservation")
    
    # Convert document
    result = converter.convert(pdf_path)
    
    # Create faithful reproduction
    faithful_doc = create_faithful_reproduction(result)
    
    # Save all outputs
    save_faithful_outputs(faithful_doc, pdf_path)
    
    # Summary
    print(f"\n📊 Extraction Summary:")
    print(f"   - Pages: {faithful_doc['metadata']['page_count']}")
    print(f"   - Tables: {faithful_doc['metadata']['table_count']}")
    print(f"   - Text elements: {len(faithful_doc['plain_text_content'])}")
    
    # Check for specific EPA data patterns
    tables_with_scientific_notation = 0
    tables_with_detection_codes = 0
    successful_table_extractions = 0
    
    for table in faithful_doc["tables"]:
        if "data" in table and table["data"]:
            successful_table_extractions += 1
            table_str = str(table["data"])
            if any(pattern in table_str for pattern in ["E-", "E+"]):
                tables_with_scientific_notation += 1
            if any(code in table_str for code in ["BDL", "DLL", "ND"]):
                tables_with_detection_codes += 1
    
    print(f"   - Successful table extractions: {successful_table_extractions}")
    print(f"   - Tables with scientific notation: {tables_with_scientific_notation}")
    print(f"   - Tables with detection codes: {tables_with_detection_codes}")
    
    return faithful_doc

if __name__ == "__main__":
    # Process EPA document
    epa_doc = "/Users/jack/Desktop/EPA-HQ-OAR-2010-0682-0085_attachment_1.pdf"
    
    if Path(epa_doc).exists():
        print("🚀 Starting EPA document faithful extraction...")
        result = process_document_faithful(epa_doc)
        print("\n✅ Faithful extraction complete!")
        print("   Check the 'faithful_output' directory for all extracted files.")
    else:
        print(f"❌ Document not found: {epa_doc}")
        print("   Please update the path to your EPA document.")
