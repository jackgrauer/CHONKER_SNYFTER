#!/usr/bin/env python3
"""Check Docling format options and capabilities"""

from docling.document_converter import DocumentConverter, PdfFormatOption
from docling.datamodel.base_models import InputFormat
import inspect

print("=== Checking Docling Format Options ===\n")

# Check PdfFormatOption
print("PdfFormatOption attributes:")
for attr in dir(PdfFormatOption):
    if not attr.startswith('_'):
        print(f"  {attr}")

print("\nPdfFormatOption parameters:")
sig = inspect.signature(PdfFormatOption.__init__)
for param_name, param in sig.parameters.items():
    if param_name != 'self':
        print(f"  {param_name}: {param.annotation if param.annotation != inspect._empty else 'Any'}")
        if param.default != inspect._empty:
            print(f"    default: {param.default}")

# Try creating with different options
print("\n=== Testing Different PDF Options ===")

# Default options
default_opt = PdfFormatOption()
print(f"\nDefault PdfFormatOption: {default_opt}")

# Check if there are style-related options
print("\nChecking for style/font related options...")
for attr in ['extract_styles', 'extract_fonts', 'preserve_formatting', 'style_extraction', 
             'font_extraction', 'formatting', 'metadata', 'rich_text']:
    if hasattr(default_opt, attr):
        print(f"  Found: {attr} = {getattr(default_opt, attr)}")

# Check model fields if it's a Pydantic model
if hasattr(PdfFormatOption, 'model_fields'):
    print("\nPdfFormatOption model fields:")
    for field_name, field_info in PdfFormatOption.model_fields.items():
        print(f"  {field_name}: {field_info}")

# Look for any enum or config options
print("\nSearching for extraction options...")
from docling import document_converter
for name in dir(document_converter):
    obj = getattr(document_converter, name)
    if isinstance(obj, type) and 'option' in name.lower():
        print(f"\nFound option class: {name}")
        for attr in dir(obj):
            if not attr.startswith('_'):
                print(f"  {attr}")