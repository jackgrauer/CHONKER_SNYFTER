#!/usr/bin/env python3
"""
Enhanced test script to explore all available table methods and find native structure
This implements Step 2 of the Docling Table Extraction Fix Action Plan
"""

import sys
import json
from pathlib import Path
from docling.document_converter import DocumentConverter

def explore_table_methods(pdf_path: str):
    """Explore all available methods and attributes to find native table structure"""
    
    print("🔍 Exploring Table Methods and Native Structure")
    print("=" * 60)
    
    # Initialize converter
    converter = DocumentConverter()
    
    # Convert document
    print(f"📄 Converting: {pdf_path}")
    result = converter.convert(pdf_path)
    
    print(f"✅ Document converted successfully")
    print(f"📊 Found {len(result.document.tables)} tables")
    
    if not result.document.tables:
        print("❌ No tables found")
        return
    
    # Focus on first table for detailed analysis
    table = result.document.tables[0]
    print(f"\n🔬 Analyzing Table 1 in detail:")
    print("=" * 60)
    
    # 1. Explore all attributes (including private ones)
    print("\n📋 All Attributes:")
    all_attrs = [attr for attr in dir(table)]
    for attr in all_attrs:
        try:
            value = getattr(table, attr)
            attr_type = type(value).__name__
            if not callable(value):
                print(f"   {attr}: {attr_type}")
                # Special handling for important attributes
                if attr in ['data', '_table_data', 'grid', '_grid', 'cells', '_cells']:
                    print(f"      → {str(value)[:100]}...")
        except Exception as e:
            print(f"   {attr}: Error accessing - {e}")
    
    # 2. Try various export methods
    print("\n📊 Export Methods:")
    
    # Try model_dump (Pydantic models often have this)
    try:
        model_data = table.model_dump()
        print("   ✅ model_dump() works!")
        print(f"      Keys: {list(model_data.keys())}")
        
        # Save for inspection
        with open('table_model_dump.json', 'w') as f:
            json.dump(model_data, f, indent=2, default=str)
        print("      💾 Saved to table_model_dump.json")
        
        # Look for data structure
        if 'data' in model_data:
            data_type = type(model_data['data'])
            print(f"      Data type: {data_type}")
            if hasattr(model_data['data'], '__dict__'):
                print(f"      Data attributes: {list(model_data['data'].__dict__.keys())}")
                
    except Exception as e:
        print(f"   ❌ model_dump() failed: {e}")
    
    # Try dict() method
    try:
        dict_data = table.dict()
        print("   ✅ dict() works!")
        print(f"      Keys: {list(dict_data.keys())}")
    except Exception as e:
        print(f"   ❌ dict() failed: {e}")
    
    # Try json() method
    try:
        json_data = table.json()
        print("   ✅ json() works!")
        print(f"      Length: {len(json_data)} characters")
        
        # Parse and save
        parsed_json = json.loads(json_data)
        with open('table_json_export.json', 'w') as f:
            json.dump(parsed_json, f, indent=2)
        print("      💾 Saved to table_json_export.json")
        
    except Exception as e:
        print(f"   ❌ json() failed: {e}")
    
    # 3. Try accessing data attribute directly
    print("\n📊 Direct Data Access:")
    
    if hasattr(table, 'data'):
        print("   ✅ Has 'data' attribute")
        data = getattr(table, 'data')
        print(f"      Data type: {type(data)}")
        
        # If it's a table data object, explore it
        if hasattr(data, '__dict__'):
            print(f"      Data attributes: {list(data.__dict__.keys())}")
            
            # Look for grid or cells
            for attr in ['grid', 'cells', 'table_cells', 'rows', 'columns']:
                if hasattr(data, attr):
                    attr_value = getattr(data, attr)
                    print(f"      → {attr}: {type(attr_value)} (length: {len(attr_value) if hasattr(attr_value, '__len__') else 'N/A'})")
                    
                    # If it's a grid/cells structure, examine first element
                    if hasattr(attr_value, '__len__') and len(attr_value) > 0:
                        first_item = attr_value[0]
                        print(f"         First item type: {type(first_item)}")
                        if hasattr(first_item, '__dict__'):
                            print(f"         First item attributes: {list(first_item.__dict__.keys())}")
                        elif hasattr(first_item, '__len__') and len(first_item) > 0:
                            # It's a row, check first cell
                            first_cell = first_item[0]
                            print(f"         First cell type: {type(first_cell)}")
                            if hasattr(first_cell, '__dict__'):
                                print(f"         First cell attributes: {list(first_cell.__dict__.keys())}")
    
    # 4. Try other export formats to see what preserves structure
    print("\n📊 Other Export Methods:")
    
    # Try OTSL (Open Table and Structure Language)
    try:
        otsl_data = table.export_to_otsl()
        print("   ✅ export_to_otsl() works!")
        print(f"      Length: {len(otsl_data)} characters")
        
        with open('table_otsl_export.txt', 'w') as f:
            f.write(otsl_data)
        print("      💾 Saved to table_otsl_export.txt")
        
    except Exception as e:
        print(f"   ❌ export_to_otsl() failed: {e}")
    
    # Try DocTags
    try:
        doctags_data = table.export_to_doctags()
        print("   ✅ export_to_doctags() works!")
        print(f"      Length: {len(doctags_data)} characters")
        
        with open('table_doctags_export.txt', 'w') as f:
            f.write(doctags_data)
        print("      💾 Saved to table_doctags_export.txt")
        
    except Exception as e:
        print(f"   ❌ export_to_doctags() failed: {e}")
    
    # Try HTML with structure preservation
    try:
        html_data = table.export_to_html()
        print("   ✅ export_to_html() works!")
        print(f"      Length: {len(html_data)} characters")
        
        with open('table_html_export.html', 'w') as f:
            f.write(html_data)
        print("      💾 Saved to table_html_export.html")
        
    except Exception as e:
        print(f"   ❌ export_to_html() failed: {e}")
    
    # 5. Document-level export to see native format
    print("\n📊 Document-level Native Export:")
    
    try:
        # Try the document's export_to_dict
        doc_dict = result.document.model_dump()
        print("   ✅ Document model_dump() works!")
        print(f"      Keys: {list(doc_dict.keys())}")
        
        # Look at tables section
        if 'tables' in doc_dict:
            tables_data = doc_dict['tables']
            print(f"      Tables data type: {type(tables_data)}")
            print(f"      Number of tables: {len(tables_data) if hasattr(tables_data, '__len__') else 'N/A'}")
            
            if tables_data and len(tables_data) > 0:
                first_table = tables_data[0]
                print(f"      First table keys: {list(first_table.keys()) if isinstance(first_table, dict) else 'Not a dict'}")
                
                # Save first table for analysis
                with open('native_table_from_document.json', 'w') as f:
                    json.dump(first_table, f, indent=2, default=str)
                print("      💾 Saved first table to native_table_from_document.json")
        
    except Exception as e:
        print(f"   ❌ Document model_dump() failed: {e}")
    
    print("\n" + "=" * 60)
    print("✅ Exploration Complete")
    print("=" * 60)
    print("\nCheck the generated files for detailed structure analysis:")
    print("  • table_model_dump.json")
    print("  • table_json_export.json") 
    print("  • table_otsl_export.txt")
    print("  • table_doctags_export.txt")
    print("  • table_html_export.html")
    print("  • native_table_from_document.json")

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python test_table_methods.py <pdf_path>")
        sys.exit(1)
    
    pdf_path = sys.argv[1]
    if not Path(pdf_path).exists():
        print(f"❌ File not found: {pdf_path}")
        sys.exit(1)
    
    explore_table_methods(pdf_path)
