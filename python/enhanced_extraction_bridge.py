#!/Users/jack/CHONKER_SNYFTER/venv/bin/python3
"""
Enhanced Extraction Bridge - Tauri Compatible
Adapts the enhanced extraction_pipeline.py to the Tauri app's expected API
"""

import json
import sys
import argparse
import time
from pathlib import Path
from typing import Dict, List, Any, Optional
import traceback

# Import the enhanced pipeline
from extraction_pipeline import ExtractionPipeline

def clean_markdown_table_duplicates(markdown: str) -> str:
    """Remove duplicate table content that appears after proper table formatting"""
    # Find the end of proper content
    end_marker = "Concentrations reported in miligrams per kilogram (mg/kg)"
    
    if end_marker in markdown:
        # Find the position and cut there
        end_pos = markdown.find(end_marker) + len(end_marker)
        cleaned_markdown = markdown[:end_pos]
        
        print(f"🧹 Cleaned markdown: removed {len(markdown) - end_pos} duplicate characters", file=sys.stderr)
        return cleaned_markdown
    
    return markdown

def convert_pipeline_output_to_tauri_format(pipeline_result: Dict[str, Any], pdf_path: str, page_num: Optional[int] = None) -> Dict[str, Any]:
    """Convert extraction_pipeline.py output to Tauri-compatible format with enhanced parsing support"""
    
    # USE STRUCTURED TABLES - the pipeline correctly extracts these!
    # The pipeline puts the processed tables in 'structured_tables', not 'tables'
    raw_tables = pipeline_result.get('structured_tables', [])
    
    print(f"🔍 Found {len(raw_tables)} structured tables in pipeline result", file=sys.stderr)
    
    # Convert STRUCTURED tables to Tauri-compatible format - USE THE PERFECT DATA!
    tables = []
    for table in raw_tables:
        print(f"🔍 Processing structured table {table.get('table_index', 'unknown')}", file=sys.stderr)
        
        # Get the structured data (the good stuff!)
        structured_data = table.get('structured_data', {})
        parsing_metadata = table.get('parsing_metadata', {})
        
        if not structured_data:
            print(f"⚠️  No structured_data found for table {table.get('table_index', 'unknown')}", file=sys.stderr)
            continue
            
        headers = structured_data.get('headers', [])
        rows = structured_data.get('rows', [])
        columns = structured_data.get('columns', {})
        context = structured_data.get('context', {})
        
        if not headers and not rows:
            print(f"⚠️  No headers or rows in structured_data for table {table.get('table_index', 'unknown')}", file=sys.stderr)
            continue
        
        # Build grid format for HTML rendering
        all_rows = []
        
        # Add header rows
        for header_row in headers:
            grid_row = []
            for header_cell in header_row:
                grid_row.append({
                    'text': str(header_cell),
                    'row_span': 1,
                    'col_span': 1,
                    'is_header': True
                })
            all_rows.append(grid_row)
        
        # Add data rows - USE THE STRUCTURED DATA!
        for row_data in rows:
            grid_row = []
            
            # Get the column order from the columns mapping
            sorted_columns = sorted(columns.items(), key=lambda x: int(x[0]) if isinstance(x[0], str) and x[0].isdigit() else (x[0] if isinstance(x[0], int) else 999))
            
            for col_idx, col_name in sorted_columns:
                cell_value = row_data.get(col_name, '')
                
                # Handle different value types from structured data
                if isinstance(cell_value, list):
                    # Multiple values - join them appropriately
                    display_text = ' '.join(str(v) for v in cell_value)
                elif isinstance(cell_value, (int, float)):
                    display_text = str(cell_value)
                else:
                    display_text = str(cell_value)
                
                grid_row.append({
                    'text': display_text,
                    'row_span': 1,
                    'col_span': 1,
                    'is_header': False,
                    'structured_value': cell_value  # Preserve the original structured value!
                })
            
            if grid_row:  # Only add non-empty rows
                all_rows.append(grid_row)
        
        # Create table data structure
        table_data = {
            'num_rows': len(all_rows),
            'num_cols': len(columns) if columns else 0,
            'grid': all_rows,
            'context': context,  # Include the context!
            'parsing_metadata': parsing_metadata,  # Include metadata!
            'table_index': table.get('table_index', len(tables))
        }
        
        print(f"✅ Converted structured table: {table_data['num_rows']}x{table_data['num_cols']} with context", file=sys.stderr)
        tables.append(table_data)
    
    # Clean the markdown to remove duplicate table content
    raw_markdown = pipeline_result.get('document', {}).get('markdown', '')
    cleaned_markdown = clean_markdown_table_duplicates(raw_markdown)
    
    # Create page extraction in Tauri format
    page_extraction = {
        'page_number': page_num if page_num is not None else 1,
        'text': cleaned_markdown,
        'tables': tables,
        'figures': [],
        'formulas': [],
        'confidence': 0.95,
        'layout_boxes': [],
        'tool': 'enhanced_extraction_pipeline'
    }
    
    # Calculate metadata
    metadata = pipeline_result.get('metadata', {})
    total_pages = metadata.get('page_count', 1)
    processing_time = int(time.time() * 1000)  # Current timestamp as fallback
    
    extraction_metadata = {
        'total_pages': total_pages,
        'processing_time': processing_time
    }
    
    # Return in Tauri expected format
    return {
        'success': True,
        'tool': 'enhanced_extraction_pipeline',
        'extractions': [page_extraction],
        'metadata': extraction_metadata,
        'structured_tables': tables,  # Also available at top level
        'pipeline_metadata': metadata  # Include original pipeline metadata
    }

def extract_with_enhanced_pipeline(pdf_path: str, page_num: Optional[int] = None) -> Dict[str, Any]:
    """Run enhanced extraction pipeline and convert to Tauri format"""
    try:
        start_time = time.time()
        
        print(f"🚀 Starting enhanced pipeline extraction for: {pdf_path}", file=sys.stderr)
        
        # Use the working extraction pipeline
        print(f"🚀 Using updated extraction pipeline", file=sys.stderr)
        
        # Create and run the extraction pipeline
        pipeline = ExtractionPipeline()
        results = pipeline.run(pdf_path, save_intermediates=False)
        
        print(f"✅ Raw Docling extraction complete", file=sys.stderr)
        
        processing_time = int((time.time() - start_time) * 1000)
        
        # Convert to Tauri format
        tauri_result = convert_pipeline_output_to_tauri_format(results, pdf_path, page_num)
        tauri_result['metadata']['processing_time'] = processing_time
        
        table_count = len(tauri_result.get('structured_tables', []))
        print(f"✅ Enhanced pipeline complete! Tables: {table_count}, Time: {processing_time}ms", file=sys.stderr)
        
        return tauri_result
        
    except Exception as e:
        error_msg = f'Enhanced pipeline extraction failed: {str(e)}'
        print(f"❌ {error_msg}", file=sys.stderr)
        print(f"🔍 Traceback: {traceback.format_exc()}", file=sys.stderr)
        
        
        return {
            'success': False,
            'error': error_msg,
            'traceback': traceback.format_exc(),
            'tool': 'enhanced_extraction_pipeline'
        }

def main():
    parser = argparse.ArgumentParser(description='Enhanced Extraction Bridge - Tauri Compatible')
    parser.add_argument('pdf_path', help='Path to PDF file')
    parser.add_argument('--page', type=int, help='Specific page number to extract (1-indexed)')
    parser.add_argument('--tool', default='enhanced_pipeline', help='Tool identifier (maintained for compatibility)')
    parser.add_argument('--output', help='Output JSON file (default: stdout)')
    
    args = parser.parse_args()
    
    # Validate input
    pdf_path = Path(args.pdf_path)
    if not pdf_path.exists():
        result = {
            'success': False,
            'error': f'PDF file not found: {pdf_path}',
            'tool': 'enhanced_extraction_pipeline'
        }
    else:
        # Perform enhanced extraction
        result = extract_with_enhanced_pipeline(str(pdf_path), args.page)
    
    # Output results
    output_json = json.dumps(result, indent=2, ensure_ascii=False, default=str)
    
    if args.output:
        with open(args.output, 'w', encoding='utf-8') as f:
            f.write(output_json)
    else:
        print(output_json)

if __name__ == '__main__':
    main()
