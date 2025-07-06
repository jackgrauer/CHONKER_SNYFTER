#!/usr/bin/env python3
"""
Test script to compare native DoclingDocument format vs flattened dataframe format
This implements Step 2 of the Docling Table Extraction Fix Action Plan
"""

import sys
import json
from pathlib import Path
from docling.document_converter import DocumentConverter

def test_native_vs_dataframe_extraction(pdf_path: str):
    """Test different extraction formats to see structure preservation"""
    
    print("🔍 Testing Native vs DataFrame Extraction Formats")
    print("=" * 60)
    
    # Initialize converter
    converter = DocumentConverter()
    
    # Convert document
    print(f"📄 Converting: {pdf_path}")
    result = converter.convert(pdf_path)
    
    print(f"✅ Document converted successfully")
    print(f"📊 Found {len(result.document.tables)} tables")
    
    # Test 1: Native DoclingDocument format
    print("\n" + "=" * 60)
    print("🔬 TEST 1: Native DoclingDocument Format")
    print("=" * 60)
    
    try:
        # Get native format
        native_doc = result.document.export_to_dict()
        
        print(f"📋 Native document structure keys: {list(native_doc.keys())}")
        
        # Check table structure specifically
        for i, table in enumerate(result.document.tables):
            print(f"\n📊 Table {i+1} Native Structure:")
            table_dict = table.export_to_dict()
            print(f"   Keys: {list(table_dict.keys())}")
            
            # Look for grid/cell information
            if 'data' in table_dict:
                print(f"   Data structure: {type(table_dict['data'])}")
                if isinstance(table_dict['data'], dict):
                    print(f"   Data keys: {list(table_dict['data'].keys())}")
                elif isinstance(table_dict['data'], list) and table_dict['data']:
                    print(f"   Data sample: {table_dict['data'][0] if table_dict['data'] else 'Empty'}")
            
            # Check for grid information
            if hasattr(table, 'grid'):
                print(f"   Has grid attribute: True")
                print(f"   Grid dimensions: {len(table.grid)} rows")
                if table.grid:
                    print(f"   First row cells: {len(table.grid[0])}")
                    # Check first cell structure
                    if table.grid[0] and table.grid[0][0] is not None:
                        first_cell = table.grid[0][0]
                        print(f"   First cell type: {type(first_cell)}")
                        if hasattr(first_cell, '__dict__'):
                            print(f"   First cell attributes: {list(first_cell.__dict__.keys())}")
            else:
                print(f"   Has grid attribute: False")
                
            # Check for other table attributes
            attrs = ['num_rows', 'num_cols', 'caption', 'header_info']
            for attr in attrs:
                if hasattr(table, attr):
                    print(f"   {attr}: {getattr(table, attr)}")
    
    except Exception as e:
        print(f"❌ Native format extraction failed: {e}")
    
    # Test 2: DataFrame format (current flattened approach)
    print("\n" + "=" * 60)
    print("🔬 TEST 2: DataFrame Format (Current)")
    print("=" * 60)
    
    try:
        for i, table in enumerate(result.document.tables):
            print(f"\n📊 Table {i+1} DataFrame Structure:")
            
            df = table.export_to_dataframe()
            if df is not None:
                print(f"   Shape: {df.shape}")
                print(f"   Columns: {list(df.columns)}")
                print(f"   Sample data:")
                print(f"   {df.head(2).to_string()}")
            else:
                print(f"   DataFrame export returned None")
    
    except Exception as e:
        print(f"❌ DataFrame format extraction failed: {e}")
    
    # Test 3: Raw table data comparison
    print("\n" + "=" * 60)
    print("🔬 TEST 3: Raw Table Data Deep Dive")
    print("=" * 60)
    
    try:
        for i, table in enumerate(result.document.tables):
            print(f"\n📊 Table {i+1} Raw Analysis:")
            
            # Try to access raw table data
            if hasattr(table, '_table_data'):
                print(f"   Has _table_data: True")
                raw_data = getattr(table, '_table_data')
                print(f"   Raw data type: {type(raw_data)}")
                if hasattr(raw_data, '__dict__'):
                    print(f"   Raw data attributes: {list(raw_data.__dict__.keys())}")
            
            # Check all available methods
            methods = [method for method in dir(table) if not method.startswith('_')]
            print(f"   Available methods: {methods}")
            
            # Try export_to_dict
            try:
                table_dict = table.export_to_dict()
                print(f"   export_to_dict keys: {list(table_dict.keys())}")
                
                # Save first table for detailed analysis
                if i == 0:
                    with open('native_table_structure.json', 'w') as f:
                        json.dump(table_dict, f, indent=2, default=str)
                    print(f"   💾 Saved detailed structure to native_table_structure.json")
                
            except Exception as export_error:
                print(f"   export_to_dict failed: {export_error}")
    
    except Exception as e:
        print(f"❌ Raw table analysis failed: {e}")
    
    print("\n" + "=" * 60)
    print("✅ Analysis Complete")
    print("=" * 60)

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python test_native_format.py <pdf_path>")
        sys.exit(1)
    
    pdf_path = sys.argv[1]
    if not Path(pdf_path).exists():
        print(f"❌ File not found: {pdf_path}")
        sys.exit(1)
    
    test_native_vs_dataframe_extraction(pdf_path)
