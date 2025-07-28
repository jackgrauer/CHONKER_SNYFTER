#!/usr/bin/env python3
"""
Font Explorer - Find font/style information in Docling output
"""

import json
import sys
from pathlib import Path
from docling.document_converter import DocumentConverter

def explore_font_info(pdf_path: str):
    """Find all font and style related information in Docling output"""
    print(f"🔍 Searching for font information in: {pdf_path}\n")
    
    # Convert the document
    converter = DocumentConverter()
    result = converter.convert(pdf_path)
    
    # Get all items
    items = list(result.document.iterate_items())
    print(f"Total items: {len(items)}\n")
    
    # Track what we find
    font_locations = {}
    sample_values = []
    
    # Keywords to search for
    keywords = ['font', 'style', 'size', 'bold', 'italic', 'weight', 'family', 'format']
    
    print("SEARCHING ITEMS FOR FONT INFORMATION:")
    print("=" * 60)
    
    for idx, (item, level) in enumerate(items[:20]):  # First 20 items
        item_type = type(item).__name__
        text_preview = str(item.text)[:40] if hasattr(item, 'text') else 'NO TEXT'
        
        print(f"\nItem {idx}: {item_type}")
        print(f"  Text: {text_preview}")
        
        # Search item attributes
        item_attrs = [attr for attr in dir(item) if not attr.startswith('_')]
        font_attrs = [attr for attr in item_attrs if any(k in attr.lower() for k in keywords)]
        
        if font_attrs:
            print(f"  Font-related attributes on item: {font_attrs}")
            for attr in font_attrs:
                try:
                    value = getattr(item, attr)
                    if not callable(value):
                        print(f"    item.{attr} = {value}")
                        font_locations[f'item.{attr}'] = type(value).__name__
                        sample_values.append((f'item.{attr}', value, text_preview))
                except:
                    pass
        
        # Check for prov
        if hasattr(item, 'prov') and item.prov:
            print(f"  Has {len(item.prov)} provenance entries")
            
            for prov_idx, prov in enumerate(item.prov[:1]):  # First prov only
                # Get all prov attributes
                prov_attrs = [attr for attr in dir(prov) if not attr.startswith('_')]
                font_prov_attrs = [attr for attr in prov_attrs if any(k in attr.lower() for k in keywords)]
                
                if font_prov_attrs:
                    print(f"  Font-related attributes in prov[{prov_idx}]: {font_prov_attrs}")
                    for attr in font_prov_attrs:
                        try:
                            value = getattr(prov, attr)
                            if not callable(value):
                                print(f"    prov.{attr} = {value}")
                                font_locations[f'prov.{attr}'] = type(value).__name__
                                sample_values.append((f'prov.{attr}', value, text_preview))
                        except:
                            pass
                
                # Also check bbox and page
                for attr in ['bbox', 'page', 'page_no', 'metadata']:
                    if hasattr(prov, attr):
                        value = getattr(prov, attr)
                        print(f"    prov.{attr} = {value}")
        
        # Check for any nested style objects
        for attr in ['style', 'format', 'metadata']:
            if hasattr(item, attr):
                value = getattr(item, attr)
                if value and not callable(value):
                    print(f"  {attr}: {type(value).__name__} = {str(value)[:100]}")
                    if isinstance(value, dict):
                        for k, v in value.items():
                            if any(keyword in str(k).lower() for keyword in keywords):
                                print(f"    {attr}['{k}'] = {v}")
                                sample_values.append((f'{attr}[{k}]', v, text_preview))
    
    # Summary
    print("\n" + "=" * 60)
    print("SUMMARY OF FONT LOCATIONS FOUND:")
    for location, value_type in sorted(font_locations.items()):
        print(f"  {location}: {value_type}")
    
    if not font_locations:
        print("\n⚠️  NO FONT ATTRIBUTES FOUND!")
        print("\nChecking alternative locations...")
        
        # Try assembled model
        if hasattr(result, 'assembled'):
            print("\nChecking result.assembled.elements...")
            for elem in result.assembled.elements[:5]:
                print(f"  Element type: {type(elem).__name__}")
                elem_attrs = [attr for attr in dir(elem) if not attr.startswith('_') and not callable(getattr(elem, attr, None))]
                print(f"  Attributes: {elem_attrs[:20]}")
                
                # Check cluster
                if hasattr(elem, 'cluster'):
                    cluster = elem.cluster
                    if cluster:
                        cluster_attrs = [attr for attr in dir(cluster) if not attr.startswith('_')]
                        font_cluster_attrs = [attr for attr in cluster_attrs if any(k in attr.lower() for k in keywords)]
                        if font_cluster_attrs:
                            print(f"  Cluster font attrs: {font_cluster_attrs}")

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python font_explorer.py <pdf_path>")
        sys.exit(1)
    
    pdf_path = sys.argv[1]
    if not Path(pdf_path).exists():
        print(f"Error: File not found: {pdf_path}")
        sys.exit(1)
    
    explore_font_info(pdf_path)