#!/usr/bin/env python3
"""
Docling Explorer - Discover all metadata and style information from Docling
"""

import json
import sys
from pathlib import Path
from typing import Any, Dict, List, Set
from docling.document_converter import DocumentConverter

def explore_object(obj, name="object", max_depth=3, current_depth=0, seen=None):
    """Recursively explore an object's attributes and values"""
    if seen is None:
        seen = set()
    
    # Avoid infinite recursion
    obj_id = id(obj)
    if obj_id in seen or current_depth > max_depth:
        return {"_type": type(obj).__name__, "_circular_ref": True}
    seen.add(obj_id)
    
    result = {
        "_type": type(obj).__name__,
        "_class": obj.__class__.__name__ if hasattr(obj, '__class__') else None,
    }
    
    # Get all attributes
    attrs = []
    try:
        attrs = [attr for attr in dir(obj) if not attr.startswith('__')]
    except:
        pass
    
    for attr in attrs:
        try:
            value = getattr(obj, attr)
            
            # Skip methods
            if callable(value) and not isinstance(value, type):
                continue
                
            # Handle different types
            if isinstance(value, (str, int, float, bool, type(None))):
                result[attr] = value
            elif isinstance(value, (list, tuple)):
                if len(value) > 0 and current_depth < max_depth:
                    result[attr] = [explore_object(item, f"{name}.{attr}[{i}]", max_depth, current_depth + 1, seen) 
                                   for i, item in enumerate(value[:3])]  # First 3 items
                else:
                    result[attr] = f"[{type(value).__name__} with {len(value)} items]"
            elif isinstance(value, dict):
                if current_depth < max_depth:
                    result[attr] = {k: explore_object(v, f"{name}.{attr}[{k}]", max_depth, current_depth + 1, seen) 
                                   for k, v in list(value.items())[:5]}  # First 5 items
                else:
                    result[attr] = f"[dict with {len(value)} keys]"
            else:
                if current_depth < max_depth:
                    result[attr] = explore_object(value, f"{name}.{attr}", max_depth, current_depth + 1, seen)
                else:
                    result[attr] = f"[{type(value).__name__}]"
        except Exception as e:
            result[attr] = f"[Error accessing: {str(e)}]"
    
    return result

def discover_docling_structure(pdf_path: str):
    """Discover the complete structure of Docling's output"""
    print(f"🔍 Exploring Docling structure for: {pdf_path}\n")
    
    # Convert the document
    converter = DocumentConverter()
    result = converter.convert(pdf_path)
    
    # Explore the result object
    print("1. EXPLORING RESULT OBJECT:")
    print("=" * 60)
    result_structure = explore_object(result, "result", max_depth=2)
    print(json.dumps(result_structure, indent=2, default=str))
    
    # Explore document structure
    print("\n2. EXPLORING DOCUMENT STRUCTURE:")
    print("=" * 60)
    if hasattr(result, 'document'):
        doc_structure = explore_object(result.document, "document", max_depth=2)
        print(json.dumps(doc_structure, indent=2, default=str))
    
    # Explore items in detail
    print("\n3. EXPLORING INDIVIDUAL ITEMS:")
    print("=" * 60)
    items = list(result.document.iterate_items())
    print(f"Total items: {len(items)}\n")
    
    # Sample different item types
    item_types = {}
    for idx, (item, level) in enumerate(items[:20]):  # First 20 items
        item_type = type(item).__name__
        if item_type not in item_types:
            item_types[item_type] = []
        item_types[item_type].append((idx, item, level))
    
    # Explore each unique item type
    for item_type, examples in item_types.items():
        print(f"\n--- {item_type} ---")
        idx, item, level = examples[0]  # First example
        
        # Deep explore the item
        item_structure = explore_object(item, f"{item_type}", max_depth=3)
        print(json.dumps(item_structure, indent=2, default=str))
        
        # If item has provenance, explore it specially
        if hasattr(item, 'prov') and item.prov:
            print(f"\n  Provenance for {item_type}:")
            for prov_idx, prov in enumerate(item.prov[:2]):  # First 2 provs
                prov_structure = explore_object(prov, f"prov[{prov_idx}]", max_depth=3)
                print(f"  Prov[{prov_idx}]:")
                print(json.dumps(prov_structure, indent=4, default=str))
    
    # Look for font/style patterns
    print("\n4. SEARCHING FOR FONT/STYLE INFORMATION:")
    print("=" * 60)
    font_related_attrs = set()
    style_values = []
    
    for idx, (item, level) in enumerate(items[:50]):  # First 50 items
        # Check item attributes
        for attr in dir(item):
            if any(keyword in attr.lower() for keyword in ['font', 'style', 'size', 'format', 'bold', 'italic']):
                font_related_attrs.add(f"item.{attr}")
                try:
                    value = getattr(item, attr)
                    if value and not callable(value):
                        style_values.append({
                            'location': f"item.{attr}",
                            'value': str(value)[:100],
                            'type': type(value).__name__,
                            'text': str(item.text)[:30] if hasattr(item, 'text') else None
                        })
                except:
                    pass
        
        # Check provenance
        if hasattr(item, 'prov') and item.prov:
            for prov in item.prov:
                for attr in dir(prov):
                    if any(keyword in attr.lower() for keyword in ['font', 'style', 'size', 'format', 'bold', 'italic']):
                        font_related_attrs.add(f"prov.{attr}")
                        try:
                            value = getattr(prov, attr)
                            if value and not callable(value):
                                style_values.append({
                                    'location': f"prov.{attr}",
                                    'value': str(value)[:100],
                                    'type': type(value).__name__,
                                    'text': str(item.text)[:30] if hasattr(item, 'text') else None
                                })
                        except:
                            pass
    
    print("Font-related attributes found:")
    for attr in sorted(font_related_attrs):
        print(f"  - {attr}")
    
    print(f"\nSample style values ({len(style_values)} found):")
    for val in style_values[:10]:  # First 10
        print(f"  {val['location']}: {val['value']} [{val['type']}]")
        if val['text']:
            print(f"    Text: {val['text']}")

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python docling_explorer.py <pdf_path>")
        sys.exit(1)
    
    pdf_path = sys.argv[1]
    if not Path(pdf_path).exists():
        print(f"Error: File not found: {pdf_path}")
        sys.exit(1)
    
    discover_docling_structure(pdf_path)