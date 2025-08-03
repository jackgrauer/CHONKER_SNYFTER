#!/usr/bin/env python3
"""
PDF Table Detection using various methods
Tries multiple approaches in order of preference
"""

import sys
import json
import os
from pathlib import Path

def detect_tables_with_camelot():
    """Try using Camelot for table detection"""
    try:
        import camelot
        
        def extract(pdf_path):
            tables = camelot.read_pdf(pdf_path, pages='all', flavor='lattice')
            results = []
            
            for i, table in enumerate(tables):
                # Get table location
                x1, y1, x2, y2 = table._bbox
                results.append({
                    'page': table.page,
                    'bbox': {'x1': x1, 'y1': y1, 'x2': x2, 'y2': y2},
                    'rows': table.shape[0],
                    'cols': table.shape[1],
                    'data': table.df.to_dict('records')
                })
            
            return {'method': 'camelot', 'tables': results}
            
        return extract
    except ImportError:
        return None

def detect_tables_with_pdfplumber():
    """Try using pdfplumber for table detection"""
    try:
        import pdfplumber
        
        def extract(pdf_path):
            results = []
            
            with pdfplumber.open(pdf_path) as pdf:
                for page_num, page in enumerate(pdf.pages, 1):
                    tables = page.find_tables()
                    
                    for i, table in enumerate(tables):
                        # Extract table bbox
                        bbox = table.bbox
                        results.append({
                            'page': page_num,
                            'bbox': {
                                'x1': bbox[0],
                                'y1': bbox[1],
                                'x2': bbox[2],
                                'y2': bbox[3]
                            },
                            'rows': len(table.rows),
                            'cols': len(table.rows[0]) if table.rows else 0,
                            'data': table.extract()
                        })
            
            return {'method': 'pdfplumber', 'tables': results}
            
        return extract
    except ImportError:
        return None

def detect_tables_with_tabula():
    """Try using tabula-py for table detection"""
    try:
        import tabula
        import pandas as pd
        
        def extract(pdf_path):
            # Read all tables from all pages
            dfs = tabula.read_pdf(pdf_path, pages='all', multiple_tables=True, 
                                 output_format='dataframe', lattice=True)
            
            results = []
            for i, df in enumerate(dfs):
                if not df.empty:
                    results.append({
                        'page': i + 1,  # tabula doesn't give exact page numbers
                        'bbox': None,   # tabula doesn't provide bbox easily
                        'rows': len(df),
                        'cols': len(df.columns),
                        'data': df.to_dict('records')
                    })
            
            return {'method': 'tabula', 'tables': results}
            
        return extract
    except ImportError:
        return None

def main():
    if len(sys.argv) != 2:
        print(json.dumps({'error': 'Usage: pdf_table_detector.py <pdf_path>'}))
        sys.exit(1)
    
    pdf_path = sys.argv[1]
    
    if not os.path.exists(pdf_path):
        print(json.dumps({'error': f'PDF file not found: {pdf_path}'}))
        sys.exit(1)
    
    # Try each method in order
    methods = [
        ('camelot', detect_tables_with_camelot()),
        ('pdfplumber', detect_tables_with_pdfplumber()),
        ('tabula', detect_tables_with_tabula()),
    ]
    
    for method_name, extractor in methods:
        if extractor:
            try:
                result = extractor(pdf_path)
                print(json.dumps(result, indent=2))
                return
            except Exception as e:
                # Try next method
                continue
    
    # If we get here, no methods worked
    print(json.dumps({
        'error': 'No table detection libraries available',
        'help': 'Install one of: pip install camelot-py[cv] pdfplumber tabula-py'
    }))

if __name__ == '__main__':
    main()