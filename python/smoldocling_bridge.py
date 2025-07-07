#!/Users/jack/CHONKER_SNYFTER/venv/bin/python3
"""
CHONKER SmolDocling VLM Integration Bridge
Advanced PDF extraction using Docling's SmolDocling VLM model for enhanced understanding
"""

import json
import sys
import argparse
import time
import subprocess
import tempfile
import os
from pathlib import Path
from typing import Dict, List, Any, Optional
import traceback

def extract_with_smoldocling(pdf_path: str, output_format: str = "json") -> Dict[str, Any]:
    """
    Extract PDF content using SmolDocling VLM model through docling CLI
    """
    try:
        start_time = time.time()
        
        print(f"🤖 Starting SmolDocling VLM extraction for: {pdf_path}", file=sys.stderr)
        print(f"🧠 Using Vision-Language Model for enhanced document understanding", file=sys.stderr)
        
        # Create temporary output directory
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_output = Path(temp_dir)
            
            # Build docling command with SmolDocling VLM
            cmd = [
                "docling",
                "--pipeline", "vlm",
                "--vlm-model", "smoldocling",
                "--to", output_format,
                "--output", str(temp_output),
                "--verbose",
                pdf_path
            ]
            
            print(f"🚀 Running: {' '.join(cmd)}", file=sys.stderr)
            
            # Execute docling with SmolDocling
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                timeout=1800  # 30 minute timeout for VLM processing
            )
            
            if result.returncode != 0:
                error_msg = f"SmolDocling extraction failed: {result.stderr}"
                print(f"❌ {error_msg}", file=sys.stderr)
                return {
                    'success': False,
                    'error': error_msg,
                    'tool': 'smoldocling_vlm'
                }
            
            # Find the output file
            pdf_name = Path(pdf_path).stem
            output_file = temp_output / f"{pdf_name}.json"
            
            if not output_file.exists():
                # Try to find any JSON file in the output directory
                json_files = list(temp_output.glob("*.json"))
                if json_files:
                    output_file = json_files[0]
                else:
                    return {
                        'success': False,
                        'error': f"No output file found in {temp_output}",
                        'tool': 'smoldocling_vlm'
                    }
            
            # Load the structured JSON output
            with open(output_file, 'r', encoding='utf-8') as f:
                docling_data = json.load(f)
            
            print(f"✅ SmolDocling extraction completed", file=sys.stderr)
            
            # Convert Docling JSON to CHONKER format
            chunks = convert_docling_to_chunks(docling_data)
            
            processing_time = int((time.time() - start_time) * 1000)
            
            # Extract metadata
            metadata = extract_metadata(docling_data)
            
            print(f"🔄 Converted to {len(chunks)} chunks in {processing_time}ms", file=sys.stderr)
            
            extraction = {
                'page_number': 1,
                'text': get_full_text(docling_data),
                'structured_chunks': chunks,
                'markdown_text': convert_to_markdown(docling_data),
                'docling_json': docling_data,  # Include raw docling output
                'tables': extract_tables(docling_data),
                'figures': extract_figures(docling_data),
                'formulas': extract_formulas(docling_data),
                'confidence': 0.98,  # VLM models generally have high confidence
                'layout_boxes': extract_layout_boxes(docling_data),
                'tool': 'smoldocling_vlm',
                'content_format': 'vlm_enhanced',
            }
            
            return {
                'success': True,
                'tool': 'smoldocling_vlm',
                'extractions': [extraction],
                'metadata': {
                    'total_pages': metadata.get('page_count', 1),
                    'tables_found': len(extraction['tables']),
                    'figures_found': len(extraction['figures']),
                    'processing_time': processing_time,
                    'vlm_model': 'smoldocling',
                    'docling_version': metadata.get('docling_version', 'unknown'),
                    'schema_version': docling_data.get('version', 'unknown')
                }
            }
            
    except subprocess.TimeoutExpired:
        return {
            'success': False,
            'error': 'SmolDocling extraction timed out (30 minutes)',
            'tool': 'smoldocling_vlm'
        }
    except Exception as e:
        error_msg = f'SmolDocling VLM extraction failed: {str(e)}'
        print(f"❌ {error_msg}", file=sys.stderr)
        print(f"🔍 Traceback: {traceback.format_exc()}", file=sys.stderr)
        return {
            'success': False,
            'error': error_msg,
            'traceback': traceback.format_exc(),
            'tool': 'smoldocling_vlm'
        }

def convert_docling_to_chunks(docling_data: Dict[str, Any]) -> List[Dict[str, Any]]:
    """Convert Docling JSON format to CHONKER chunk format (supports v1 and v2)"""
    chunks = []
    chunk_id = 1
    
    # Detect Docling version by checking structure
    is_v2 = 'document' in docling_data and 'markdown' in docling_data.get('document', {})
    
    if is_v2:
        print("🔍 Detected Docling v2 format", file=sys.stderr)
        return convert_docling_v2_to_chunks(docling_data)
    else:
        print("🔍 Detected Docling v1 format", file=sys.stderr)
        return convert_docling_v1_to_chunks(docling_data)

def convert_docling_v2_to_chunks(docling_data: Dict[str, Any]) -> List[Dict[str, Any]]:
    """Convert Docling v2 JSON format to CHONKER chunk format"""
    chunks = []
    chunk_id = 1
    
    # Extract markdown content as main text chunk
    document = docling_data.get('document', {})
    markdown_content = document.get('markdown', '')
    
    if markdown_content.strip():
        # Split markdown into logical sections
        sections = split_markdown_into_sections(markdown_content)
        
        for section_title, section_content in sections:
            if section_content.strip():
                chunk = {
                    'id': f"chunk_{chunk_id}",
                    'type': 'table' if '|' in section_content else 'text',
                    'element_type': detect_element_type(section_title, section_content),
                    'content': section_content.strip(),
                    'page_number': 1,  # v2 format doesn't always provide page info in structured way
                    'bbox': None,  # v2 format typically doesn't include bbox in main document
                    'confidence': 0.98
                }
                chunks.append(chunk)
                chunk_id += 1
    
    # Process structured tables if available
    structured_tables = docling_data.get('structured_tables', [])
    for table_idx, table_item in enumerate(structured_tables):
        table_content = convert_v2_table_to_text(table_item)
        if table_content.strip():
            chunk = {
                'id': f"chunk_{chunk_id}",
                'type': 'table',
                'element_type': 'structured_table',
                'content': table_content,
                'page_number': 1,
                'bbox': None,
                'table_data': extract_v2_table_data(table_item),
                'confidence': 0.98
            }
            chunks.append(chunk)
            chunk_id += 1
    
    return chunks

def convert_docling_v1_to_chunks(docling_data: Dict[str, Any]) -> List[Dict[str, Any]]:
    """Convert Docling v1 JSON format to CHONKER chunk format"""
    chunks = []
    chunk_id = 1
    
    # Process text elements (v1 format)
    texts = docling_data.get('texts', [])
    for text_item in texts:
        if text_item.get('text', '').strip():
            chunk = {
                'id': f"chunk_{chunk_id}",
                'type': 'text',
                'element_type': text_item.get('label', 'text'),
                'content': text_item.get('text', ''),
                'page_number': text_item.get('prov', [{}])[0].get('page', 1) if text_item.get('prov') else 1,
                'bbox': extract_bbox(text_item),
                'confidence': 0.98
            }
            chunks.append(chunk)
            chunk_id += 1
    
    # Process tables (v1 format)
    tables = docling_data.get('tables', [])
    for table_item in tables:
        content = convert_table_to_text(table_item)
        if content.strip():
            chunk = {
                'id': f"chunk_{chunk_id}",
                'type': 'table',
                'element_type': 'table',
                'content': content,
                'page_number': table_item.get('prov', [{}])[0].get('page', 1) if table_item.get('prov') else 1,
                'bbox': extract_bbox(table_item),
                'table_data': extract_table_data(table_item),
                'confidence': 0.98
            }
            chunks.append(chunk)
            chunk_id += 1
    
    # Process figures/pictures (v1 format)
    pictures = docling_data.get('pictures', [])
    for picture_item in pictures:
        description = picture_item.get('text', '')
        if description.strip():
            chunk = {
                'id': f"chunk_{chunk_id}",
                'type': 'figure',
                'element_type': 'figure',
                'content': description,
                'page_number': picture_item.get('prov', [{}])[0].get('page', 1) if picture_item.get('prov') else 1,
                'bbox': extract_bbox(picture_item),
                'confidence': 0.95
            }
            chunks.append(chunk)
            chunk_id += 1
    
    return chunks

def split_markdown_into_sections(markdown: str) -> List[tuple]:
    """Split markdown content into logical sections"""
    sections = []
    current_section = ""
    current_title = "Document Content"
    
    lines = markdown.split('\n')
    for line in lines:
        if line.strip().startswith('##'):
            # Save previous section if it has content
            if current_section.strip():
                sections.append((current_title, current_section.strip()))
            
            # Start new section
            current_title = line.strip()
            current_section = ""
        else:
            current_section += line + '\n'
    
    # Add final section
    if current_section.strip():
        sections.append((current_title, current_section.strip()))
    
    return sections

def detect_element_type(title: str, content: str) -> str:
    """Detect element type from title and content"""
    title_lower = title.lower()
    content_lower = content.lower()
    
    if 'table' in title_lower or '|' in content:
        return 'table'
    elif title.startswith('##'):
        return 'heading'
    elif 'notes:' in title_lower or 'note:' in title_lower:
        return 'notes'
    elif any(keyword in content_lower for keyword in ['sample', 'result', 'data', 'concentration']):
        return 'analytical_data'
    else:
        return 'text'

def convert_v2_table_to_text(table_item: Dict[str, Any]) -> str:
    """Convert Docling v2 structured table to readable text"""
    try:
        parsing_metadata = table_item.get('parsing_metadata', {})
        structured_data = table_item.get('structured_data', {})
        
        # Start with table context if available
        context = parsing_metadata.get('context', {})
        table_title = context.get('table_title', '')
        text_before = context.get('text_before', '')
        
        text_parts = []
        if text_before:
            text_parts.append(text_before)
        if table_title and table_title != text_before:
            text_parts.append(f"Table: {table_title}")
        
        # Add table structure info
        original_dims = parsing_metadata.get('original_dimensions', '')
        if original_dims:
            text_parts.append(f"Table dimensions: {original_dims}")
        
        # Process headers
        headers = structured_data.get('headers', [])
        if headers:
            text_parts.append("Table Headers:")
            for i, header_row in enumerate(headers):
                if isinstance(header_row, list):
                    header_text = ' | '.join(str(h) for h in header_row if h)
                    if header_text.strip():
                        text_parts.append(f"Header Row {i+1}: {header_text}")
        
        # Process rows if available
        rows = structured_data.get('rows', [])
        if rows:
            text_parts.append("Table Data:")
            for i, row in enumerate(rows[:5]):  # Limit to first 5 rows for readability
                if isinstance(row, dict):
                    row_text = ' | '.join(f"{k}: {v}" for k, v in row.items() if v)
                    if row_text.strip():
                        text_parts.append(f"Row {i+1}: {row_text}")
        
        return '\n'.join(text_parts)
    except Exception as e:
        print(f"⚠️ Error converting v2 table: {e}", file=sys.stderr)
        return f"Structured table (conversion error: {e})"

def extract_v2_table_data(table_item: Dict[str, Any]) -> Optional[Dict[str, Any]]:
    """Extract structured table data from Docling v2 format"""
    try:
        return {
            'parsing_metadata': table_item.get('parsing_metadata', {}),
            'structured_data': table_item.get('structured_data', {}),
            'table_index': table_item.get('table_index', 0),
            'format_version': 'docling_v2'
        }
    except Exception as e:
        print(f"⚠️ Error extracting v2 table data: {e}", file=sys.stderr)
        return None

def extract_bbox(item: Dict[str, Any]) -> Optional[Dict[str, float]]:
    """Extract bounding box information from docling item"""
    if 'prov' in item and item['prov']:
        prov = item['prov'][0]
        if 'bbox' in prov:
            bbox = prov['bbox']
            return {
                'x': float(bbox.get('l', 0)),
                'y': float(bbox.get('t', 0)),
                'width': float(bbox.get('r', 0)) - float(bbox.get('l', 0)),
                'height': float(bbox.get('b', 0)) - float(bbox.get('t', 0))
            }
    return None

def convert_table_to_text(table_item: Dict[str, Any]) -> str:
    """Convert table item to readable text format"""
    if 'data' not in table_item:
        return table_item.get('text', '')
    
    table_data = table_item['data']
    if not table_data.get('table_cells'):
        return table_item.get('text', '')
    
    # Build a simple text representation
    text_parts = [f"Table: {table_item.get('text', 'Untitled Table')}"]
    
    # Add table data if available
    cells = table_data['table_cells']
    if cells:
        text_parts.append("Table Data:")
        for cell in cells[:10]:  # Limit to first 10 cells for readability
            cell_text = cell.get('text', '').strip()
            if cell_text:
                text_parts.append(f"- {cell_text}")
    
    return '\n'.join(text_parts)

def extract_table_data(table_item: Dict[str, Any]) -> Optional[Dict[str, Any]]:
    """Extract structured table data"""
    if 'data' not in table_item:
        return None
    
    table_data = table_item['data']
    return {
        'cells': table_data.get('table_cells', []),
        'grid': table_data.get('grid', []),
        'raw_data': table_data
    }

def extract_metadata(docling_data: Dict[str, Any]) -> Dict[str, Any]:
    """Extract metadata from docling output"""
    return {
        'schema_name': docling_data.get('schema_name', 'DoclingDocument'),
        'schema_version': docling_data.get('version', 'unknown'),
        'page_count': len(set(
            item.get('prov', [{}])[0].get('page', 1) 
            for item in docling_data.get('texts', []) + docling_data.get('tables', []) + docling_data.get('pictures', [])
            if item.get('prov')
        )) or 1
    }

def get_full_text(docling_data: Dict[str, Any]) -> str:
    """Extract all text content as a single string"""
    text_parts = []
    
    # Add all text elements
    for text_item in docling_data.get('texts', []):
        text = text_item.get('text', '').strip()
        if text:
            text_parts.append(text)
    
    # Add table text
    for table_item in docling_data.get('tables', []):
        table_text = convert_table_to_text(table_item)
        if table_text.strip():
            text_parts.append(table_text)
    
    # Add figure descriptions
    for picture_item in docling_data.get('pictures', []):
        description = picture_item.get('text', '').strip()
        if description:
            text_parts.append(f"[Figure: {description}]")
    
    return '\n\n'.join(text_parts)

def convert_to_markdown(docling_data: Dict[str, Any]) -> str:
    """Convert docling data to markdown format"""
    md_parts = []
    
    # Group by page
    pages = {}
    for item in docling_data.get('texts', []) + docling_data.get('tables', []) + docling_data.get('pictures', []):
        page_num = item.get('prov', [{}])[0].get('page', 1) if item.get('prov') else 1
        if page_num not in pages:
            pages[page_num] = []
        pages[page_num].append(item)
    
    for page_num in sorted(pages.keys()):
        if len(pages) > 1:
            md_parts.append(f"# Page {page_num}\n")
        
        for item in pages[page_num]:
            if item in docling_data.get('texts', []):
                label = item.get('label', 'text')
                text = item.get('text', '').strip()
                if text:
                    if label in ['title', 'section-header']:
                        md_parts.append(f"## {text}\n")
                    else:
                        md_parts.append(f"{text}\n")
            
            elif item in docling_data.get('tables', []):
                table_md = convert_table_to_markdown(item)
                if table_md:
                    md_parts.append(table_md)
            
            elif item in docling_data.get('pictures', []):
                description = item.get('text', '').strip()
                if description:
                    md_parts.append(f"![Figure]({description})\n")
    
    return '\n'.join(md_parts)

def convert_table_to_markdown(table_item: Dict[str, Any]) -> str:
    """Convert table to markdown format"""
    # For now, just return a simple representation
    # Could be enhanced to create proper markdown tables
    table_text = convert_table_to_text(table_item)
    return f"```\n{table_text}\n```\n"

def extract_tables(docling_data: Dict[str, Any]) -> List[Dict[str, Any]]:
    """Extract table information"""
    tables = []
    for idx, table_item in enumerate(docling_data.get('tables', [])):
        table_info = {
            'index': idx,
            'text': table_item.get('text', ''),
            'data': extract_table_data(table_item),
            'bbox': extract_bbox(table_item),
            'page_number': table_item.get('prov', [{}])[0].get('page', 1) if table_item.get('prov') else 1
        }
        tables.append(table_info)
    return tables

def extract_figures(docling_data: Dict[str, Any]) -> List[Dict[str, Any]]:
    """Extract figure information"""
    figures = []
    for idx, picture_item in enumerate(docling_data.get('pictures', [])):
        figure_info = {
            'index': idx,
            'description': picture_item.get('text', ''),
            'bbox': extract_bbox(picture_item),
            'page_number': picture_item.get('prov', [{}])[0].get('page', 1) if picture_item.get('prov') else 1
        }
        figures.append(figure_info)
    return figures

def extract_formulas(docling_data: Dict[str, Any]) -> List[Dict[str, Any]]:
    """Extract formula information (if any)"""
    # SmolDocling might identify formulas in text or as separate elements
    formulas = []
    # Implementation would depend on how SmolDocling represents formulas
    return formulas

def extract_layout_boxes(docling_data: Dict[str, Any]) -> List[Dict[str, Any]]:
    """Extract layout bounding box information"""
    layout_boxes = []
    
    for item in docling_data.get('texts', []) + docling_data.get('tables', []) + docling_data.get('pictures', []):
        bbox = extract_bbox(item)
        if bbox:
            layout_box = {
                'type': item.get('label', 'unknown'),
                'bbox': bbox,
                'page_number': item.get('prov', [{}])[0].get('page', 1) if item.get('prov') else 1,
                'text_preview': item.get('text', '')[:100] if item.get('text') else ''
            }
            layout_boxes.append(layout_box)
    
    return layout_boxes

def main():
    parser = argparse.ArgumentParser(description='CHONKER SmolDocling VLM Extraction Bridge')
    parser.add_argument('pdf_path', help='Path to PDF file')
    parser.add_argument('--format', default='json', help='Output format (default: json)')
    parser.add_argument('--output', help='Output JSON file (default: stdout)')
    
    args = parser.parse_args()
    
    # Validate input
    pdf_path = Path(args.pdf_path)
    if not pdf_path.exists():
        result = {
            'success': False,
            'error': f'PDF file not found: {pdf_path}',
            'tool': 'smoldocling_vlm'
        }
    else:
        # Perform SmolDocling extraction
        result = extract_with_smoldocling(str(pdf_path), args.format)
    
    # Output results
    output_json = json.dumps(result, indent=2, ensure_ascii=False)
    
    if args.output:
        with open(args.output, 'w', encoding='utf-8') as f:
            f.write(output_json)
    else:
        print(output_json)

if __name__ == '__main__':
    main()
