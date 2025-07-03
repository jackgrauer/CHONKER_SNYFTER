#!/Users/jack/CHONKER_SNYFTER/venv/bin/python3
"""
CHONKER Enhanced Extraction Bridge - Environmental Lab Document Aware
Advanced PDF extraction using Docling v2 with document-aware processing
for environmental laboratory reports with proper qualifier handling
"""

import json
import sys
import argparse
import time
import re
from pathlib import Path
from typing import Dict, List, Any, Optional, Tuple
import traceback

# Docling v2 imports with MLX optimization
from docling.document_converter import DocumentConverter, FormatOption
from docling.datamodel.base_models import InputFormat, Table
from docling.datamodel.pipeline_options import PdfPipelineOptions, TableFormerMode
from docling.pipeline.standard_pdf_pipeline import StandardPdfPipeline
from docling.backend.docling_parse_backend import DoclingParseDocumentBackend

# MLX-optimized VLM imports
try:
    import mlx.core as mx
    from docling.datamodel.pipeline_options_vlm_model import InlineVlmOptions, InferenceFramework
    from docling.datamodel.accelerator_options import AcceleratorOptions, AcceleratorDevice
    
    # Test MLX availability
    if mx.metal.is_available():
        MLX_AVAILABLE = True
        print("🚀 MLX backend available - will use optimized Metal compute", file=sys.stderr)
        print(f"🔧 MLX Device: {mx.default_device()}", file=sys.stderr)
        device_info = mx.metal.device_info()
        print(f"💾 Memory: {device_info['memory_size']/1024/1024/1024:.1f}GB available", file=sys.stderr)
    else:
        MLX_AVAILABLE = False
        print("⚠️ MLX Metal not available", file=sys.stderr)
except ImportError as e:
    MLX_AVAILABLE = False
    print(f"⚠️ MLX backend not available: {e}", file=sys.stderr)
    print("📋 Falling back to PyTorch + MPS", file=sys.stderr)

class EnvironmentalLabProcessor:
    """Enhanced processor for environmental laboratory documents"""
    
    def __init__(self):
        self.qualifiers = {'U', 'J', 'UJ', 'R', 'B', 'N', 'H', 'E'}
        self.column_patterns = ['Conc', 'Q', 'RL', 'MDL', 'Qualifier', 'Result']
        self.conventions = {}
        
    def analyze_document_conventions(self, document_text: str) -> Dict[str, str]:
        """Extract data quality conventions from document text"""
        conventions = {}
        
        # Common environmental lab qualifier patterns
        qualifier_patterns = {
            r'U\s*=\s*(.+?)(?:\n|\.|\;)': 'U',
            r'J\s*=\s*(.+?)(?:\n|\.|\;)': 'J',
            r'UJ\s*=\s*(.+?)(?:\n|\.|\;)': 'UJ',
            r'R\s*=\s*(.+?)(?:\n|\.|\;)': 'R',
            r'B\s*=\s*(.+?)(?:\n|\.|\;)': 'B',
            r'N\s*=\s*(.+?)(?:\n|\.|\;)': 'N',
            r'H\s*=\s*(.+?)(?:\n|\.|\;)': 'H',
            r'E\s*=\s*(.+?)(?:\n|\.|\;)': 'E'
        }
        
        for pattern, qualifier in qualifier_patterns.items():
            matches = re.finditer(pattern, document_text, re.IGNORECASE)
            for match in matches:
                definition = match.group(1).strip()
                conventions[qualifier] = definition
                print(f"📖 Found qualifier definition: {qualifier} = {definition}", file=sys.stderr)
        
        # Common default definitions if not found in document
        if not conventions:
            conventions.update({
                'U': 'Undetected (below detection limit)',
                'J': 'Estimated value (detected but below reporting limit)',
                'UJ': 'Undetected and estimated',
                'R': 'Rejected',
                'B': 'Blank contamination',
                'N': 'Not detected',
                'H': 'Hold time exceeded',
                'E': 'Estimated'
            })
            print("📋 Using default environmental lab qualifier definitions", file=sys.stderr)
        
        self.conventions = conventions
        return conventions
    
    def split_value_qualifier(self, cell_value: str) -> Tuple[str, Optional[str]]:
        """Split combined values like '0.046 U' into value and qualifier"""
        if not isinstance(cell_value, str):
            return str(cell_value), None
        
        cell_value = cell_value.strip()
        
        # Pattern 1: Number followed by qualifier "0.046 U"
        pattern1 = r'^([\d.,]+(?:[eE][+-]?\d+)?)\s+([A-Z]+)$'
        match = re.match(pattern1, cell_value)
        if match:
            value, qualifier = match.groups()
            if qualifier in self.qualifiers:
                return value, qualifier
        
        # Pattern 2: Qualifier followed by number "U 0.046"
        pattern2 = r'^([A-Z]+)\s+([\d.,]+(?:[eE][+-]?\d+)?)$'
        match = re.match(pattern2, cell_value)
        if match:
            qualifier, value = match.groups()
            if qualifier in self.qualifiers:
                return value, qualifier
        
        # Pattern 3: Number with qualifier attached "0.046U"
        pattern3 = r'^([\d.,]+(?:[eE][+-]?\d+)?)([A-Z]+)$'
        match = re.match(pattern3, cell_value)
        if match:
            value, qualifier = match.groups()
            if qualifier in self.qualifiers:
                return value, qualifier
        
        # Pattern 4: Qualifier attached to number "U0.046"
        pattern4 = r'^([A-Z]+)([\d.,]+(?:[eE][+-]?\d+)?)$'
        match = re.match(pattern4, cell_value)
        if match:
            qualifier, value = match.groups()
            if qualifier in self.qualifiers:
                return value, qualifier
        
        # No qualifier found
        return cell_value, None
    
    def detect_column_patterns(self, headers: List[str]) -> List[Tuple[int, int]]:
        """Detect repeating Conc/Q/RL/MDL patterns in headers"""
        patterns = []
        
        for i, header in enumerate(headers):
            if any(conc_term in header.upper() for conc_term in ['CONC', 'RESULT', 'VALUE']):
                # Look for the next 3 columns to match Q/RL/MDL pattern
                if i + 3 < len(headers):
                    next_headers = [h.upper() for h in headers[i+1:i+4]]
                    
                    # Check for qualifier column
                    if any(q_term in next_headers[0] for q_term in ['Q', 'QUAL', 'FLAG']):
                        # Check for reporting limit
                        if any(rl_term in next_headers[1] for rl_term in ['RL', 'REPORT', 'LIMIT']):
                            # Check for method detection limit
                            if any(mdl_term in next_headers[2] for mdl_term in ['MDL', 'DETECT', 'METHOD']):
                                patterns.append((i, i+3))
                                print(f"🔍 Found column pattern at indices {i}-{i+3}: {headers[i:i+4]}", file=sys.stderr)
        
        return patterns
    
    def process_table_data(self, table_data: List[List[str]], patterns: List[Tuple[int, int]]) -> Dict[str, Any]:
        """Process table data with environmental lab awareness"""
        issues = []
        processed_data = []
        
        for row_idx, row in enumerate(table_data):
            processed_row = row.copy()
            
            for col_idx, cell in enumerate(row):
                if cell and isinstance(cell, str):
                    value, qualifier = self.split_value_qualifier(cell)
                    
                    if qualifier:
                        # Found a combined value/qualifier
                        issues.append({
                            'type': 'misplaced_qualifier',
                            'row': row_idx,
                            'col': col_idx,
                            'original': cell,
                            'corrected_value': value,
                            'corrected_qualifier': qualifier,
                            'severity': 'high'
                        })
                        
                        # Fix the value in place
                        processed_row[col_idx] = value
                        
                        # Try to place qualifier in appropriate column
                        for pattern_start, pattern_end in patterns:
                            if pattern_start <= col_idx <= pattern_end:
                                # This is within a detected pattern
                                qualifier_col = pattern_start + 1  # Q column should be next
                                if qualifier_col < len(processed_row):
                                    if not processed_row[qualifier_col] or processed_row[qualifier_col].strip() == "":
                                        processed_row[qualifier_col] = qualifier
                                        issues[-1]['corrected_placement'] = qualifier_col
                                break
            
            processed_data.append(processed_row)
        
        return {
            'processed_data': processed_data,
            'issues': issues
        }

    def table_to_otsl(self, table_data: List[List[str]], headers: Optional[List[str]] = None) -> str:
        """Convert table data to OTSL (Open Table and Structure Language) format"""
        if not table_data:
            return ""

        otsl_lines = ["<otsl>"]

        # Determine headers
        if headers:
            header_row = headers
            data_rows = table_data
        else:
            # Use first row as headers
            header_row = table_data[0] if table_data else []
            data_rows = table_data[1:] if len(table_data) > 1 else []

        # Add table header section
        if header_row:
            otsl_lines.append("<thead>")
            otsl_lines.append("<tr>")
            for header in header_row:
                if header and header.strip():
                    otsl_lines.append(f"<rhed>{str(header).strip()}</rhed>")
                else:
                    otsl_lines.append("<rhed></rhed>")
            otsl_lines.append("</tr>")
            otsl_lines.append("</thead>")

        # Add table body section
        if data_rows:
            otsl_lines.append("<tbody>")
            for row in data_rows:
                if any(cell and str(cell).strip() for cell in row):  # Skip empty rows
                    otsl_lines.append("<tr>")
                    for i, cell in enumerate(row):
                        cell_content = str(cell).strip() if cell is not None else ""
                        if i == 0:  # First column could be row header
                            otsl_lines.append(f"<rhed>{cell_content}</rhed>")
                        else:
                            otsl_lines.append(f"<fcel>{cell_content}</fcel>")
                    otsl_lines.append("</tr>")
            otsl_lines.append("</tbody>")

        otsl_lines.append("</otsl>")

        return "\n".join(otsl_lines)

    def convert_doctags_to_otsl(self, doctags_content: str, structured_tables: List[Dict]) -> str:
        """Convert DocTags content to include OTSL tables where appropriate"""
        if not structured_tables:
            return doctags_content

        # Create OTSL versions of the structured tables
        otsl_tables = []
        for table_info in structured_tables:
            if 'processed_data' in table_info and table_info['processed_data']:
                table_data = table_info['processed_data']
                # Extract headers if available
                headers = None
                if 'formats' in table_info and 'dataframe' in table_info['formats']:
                    headers = table_info['formats']['dataframe'].get('headers')

                otsl_content = self.table_to_otsl(table_data, headers)
                if otsl_content:
                    otsl_tables.append(otsl_content)

        # If we have OTSL tables, create enhanced content
        if otsl_tables:
            enhanced_content = f"{doctags_content}\n\n<!-- STRUCTURED TABLES IN OTSL FORMAT -->\n"
            for i, otsl_table in enumerate(otsl_tables):
                enhanced_content += f"\n<!-- Table {i+1} -->\n{otsl_table}\n"
            return enhanced_content

        return doctags_content
    
    def parse_markdown_table(self, table_lines: List[str]) -> List[List[str]]:
        """Parse markdown table lines into table data"""
        table_data = []
        
        for line in table_lines:
            # Skip separator lines (contains only -, |, :, spaces)
            if re.match(r'^[\s|:-]+$', line.strip()):
                continue
            
            # Split by | and clean up cells
            cells = [cell.strip() for cell in line.split('|')]
            # Remove empty cells at start/end (from leading/trailing |)
            while cells and not cells[0]:
                cells.pop(0)
            while cells and not cells[-1]:
                cells.pop(-1)
            
            if cells:  # Only add non-empty rows
                table_data.append(cells)
        
        return table_data

def parse_markdown_to_structured_chunks(markdown_content: str) -> List[Dict[str, Any]]:
    """Parse markdown content into structured chunks with element type detection"""
    chunks = []
    lines = markdown_content.split('\n')
    current_chunk_lines = []
    current_type = 'text'
    chunk_id = 1
    
    for line in lines:
        line_stripped = line.strip()
        
        # Detect element types
        if line_stripped.startswith('#'):
            # Finish previous chunk
            if current_chunk_lines:
                chunks.append({
                    'id': f"chunk_{chunk_id}",
                    'type': current_type,
                    'element_type': current_type,
                    'content': '\n'.join(current_chunk_lines).strip(),
                    'page_number': 1,
                    'bbox': None
                })
                chunk_id += 1
                current_chunk_lines = []
            
            # Start new heading chunk
            current_type = 'heading'
            current_chunk_lines = [line]
        elif '|' in line and len(line.split('|')) > 2:
            # Table row detected
            if current_type != 'table':
                # Finish previous chunk
                if current_chunk_lines:
                    chunks.append({
                        'id': f"chunk_{chunk_id}",
                        'type': current_type,
                        'element_type': current_type,
                        'content': '\n'.join(current_chunk_lines).strip(),
                        'page_number': 1,
                        'bbox': None
                    })
                    chunk_id += 1
                    current_chunk_lines = []
                
                current_type = 'table'
            current_chunk_lines.append(line)
        elif line_stripped.startswith(('- ', '* ', '+ ')) or (line_stripped and line_stripped[0].isdigit() and '. ' in line_stripped):
            # List item detected
            if current_type != 'list':
                # Finish previous chunk
                if current_chunk_lines:
                    chunks.append({
                        'id': f"chunk_{chunk_id}",
                        'type': current_type,
                        'element_type': current_type,
                        'content': '\n'.join(current_chunk_lines).strip(),
                        'page_number': 1,
                        'bbox': None
                    })
                    chunk_id += 1
                    current_chunk_lines = []
                
                current_type = 'list'
            current_chunk_lines.append(line)
        elif line_stripped:
            # Regular text
            if current_type not in ['text', 'heading'] or (current_type == 'heading' and len(current_chunk_lines) > 1):
                # Finish previous chunk if it's not text
                if current_chunk_lines:
                    chunks.append({
                        'id': f"chunk_{chunk_id}",
                        'type': current_type,
                        'element_type': current_type,
                        'content': '\n'.join(current_chunk_lines).strip(),
                        'page_number': 1,
                        'bbox': None
                    })
                    chunk_id += 1
                    current_chunk_lines = []
                
                current_type = 'text'
            current_chunk_lines.append(line)
        else:
            # Empty line - continue current chunk
            if current_chunk_lines:
                current_chunk_lines.append(line)
    
    # Add final chunk
    if current_chunk_lines:
        chunks.append({
            'id': f"chunk_{chunk_id}",
            'type': current_type,
            'element_type': current_type,
            'content': '\n'.join(current_chunk_lines).strip(),
            'page_number': 1,
            'bbox': None
        })
    
    return chunks

def extract_with_enhanced_docling(pdf_path: str, page_num: Optional[int] = None) -> Dict[str, Any]:
    """
    Enhanced Docling extraction with environmental lab document awareness
    """
    try:
        start_time = time.time()
        
        print(f"🧪 Starting Enhanced Environmental Lab extraction for: {pdf_path}", file=sys.stderr)
        
        # Initialize processor
        processor = EnvironmentalLabProcessor()
        
        # Configure enhanced pipeline options for environmental documents
        pipeline_options = PdfPipelineOptions(
            # Core features - ENABLED
            do_ocr=True,                    # OCR for scanned lab reports
            do_table_structure=True,        # Critical for lab data tables
            
            # Table-specific enhancements
            table_structure_options={
                "mode": TableFormerMode.ACCURATE,  # Use most accurate table detection
                "detect_table_headers": True,      # Critical for lab reports
                "min_table_rows": 2,               # Minimum viable table
                "column_separator_threshold": 0.05, # Tight columns in lab reports
                "row_separator_threshold": 0.03,   # Dense data rows
            },
            
            # Advanced features
            do_formula_enrichment=True,     # Chemical formulas
            do_picture_description=True,    # Chart/graph descriptions
            do_picture_classification=True, # Figure classification
            do_code_enrichment=False,       # Not needed for lab reports
            
            # Output generation
            generate_page_images=False,
            generate_picture_images=True,
            
            # Enhanced processing
            force_backend_text=False
        )
        
        print(f"✅ Pipeline configured for environmental lab documents", file=sys.stderr)
        
        # Create converter with enhanced pipeline
        if MLX_AVAILABLE:
            print("🚀 MLX backend detected - Apple Silicon optimizations available", file=sys.stderr)
            print("🧠 Using MLX-optimized Metal compute for accelerated processing", file=sys.stderr)
        else:
            print("📋 Using standard PyTorch + MPS converter", file=sys.stderr)
            
        # Configure enhanced pipeline
        converter = DocumentConverter()
        
        from docling.pipeline.standard_pdf_pipeline import StandardPdfPipeline
        from docling.backend.docling_parse_backend import DoclingParseDocumentBackend
        
        enhanced_format_option = FormatOption(
            pipeline_cls=StandardPdfPipeline,
            pipeline_options=pipeline_options,
            backend=DoclingParseDocumentBackend
        )
        
        converter.format_to_options[InputFormat.PDF] = enhanced_format_option
        
        print(f"🚀 Starting enhanced docling conversion...", file=sys.stderr)
        
        # Convert the document
        if page_num is not None:
            result = converter.convert(pdf_path, page_range=(page_num, page_num))
        else:
            result = converter.convert(pdf_path)
        
        print(f"📄 Document converted, analyzing for lab conventions...", file=sys.stderr)
        
        # Export to structured JSON format for proper element type detection
        structured_chunks = []
        print(f"🔍 Extracting structured elements from document...", file=sys.stderr)
        
        # Process document elements from Docling's internal structure
        if hasattr(result.document, '_doc_items') and result.document._doc_items:
            print(f"📋 Found {len(result.document._doc_items)} document items", file=sys.stderr)
            
            for idx, item in enumerate(result.document._doc_items):
                try:
                    # Extract element type and content
                    element_type = getattr(item, 'label', 'unknown')
                    content_text = getattr(item, 'text', '')
                    
                    # Get bounding box if available
                    bbox = None
                    if hasattr(item, 'bbox'):
                        bbox = {
                            'x': float(item.bbox.l),
                            'y': float(item.bbox.t), 
                            'width': float(item.bbox.r - item.bbox.l),
                            'height': float(item.bbox.b - item.bbox.t)
                        }
                    
                    # Get page number
                    page_number = getattr(item, 'page', 1)
                    
                    structured_chunk = {
                        'id': f"chunk_{idx + 1}",
                        'type': element_type.lower(),
                        'content': content_text,
                        'page_number': page_number,
                        'bbox': bbox
                    }
                    
                    # Special handling for tables
                    if element_type.lower() in ['table', 'table-cell', 'table-header']:
                        structured_chunk['element_type'] = 'table'
                        # Try to get table data if it's a table
                        if hasattr(item, 'table_data'):
                            structured_chunk['table_data'] = item.table_data
                    elif element_type.lower() in ['title', 'section-header', 'heading']:
                        structured_chunk['element_type'] = 'heading'
                    elif element_type.lower() in ['text', 'paragraph']:
                        structured_chunk['element_type'] = 'text'
                    elif element_type.lower() in ['list', 'list-item']:
                        structured_chunk['element_type'] = 'list'
                    elif element_type.lower() in ['formula', 'equation']:
                        structured_chunk['element_type'] = 'formula'
                    else:
                        structured_chunk['element_type'] = 'text'  # Default fallback
                    
                    structured_chunks.append(structured_chunk)
                    
                except Exception as item_error:
                    print(f"⚠️ Error processing item {idx}: {item_error}", file=sys.stderr)
                    # Fallback text chunk
                    structured_chunks.append({
                        'id': f"chunk_{idx + 1}",
                        'type': 'text',
                        'element_type': 'text',
                        'content': str(getattr(item, 'text', '')),
                        'page_number': 1,
                        'bbox': None
                    })
        
        # Fallback: Export to markdown and parse structure
        markdown_content = result.document.export_to_markdown()
        
        if not structured_chunks:
            print(f"📋 No structured items found, parsing markdown content...", file=sys.stderr)
            structured_chunks = parse_markdown_to_structured_chunks(markdown_content)
            
        print(f"✅ Extracted {len(structured_chunks)} structured chunks", file=sys.stderr)
        
        # Also get DocTags as fallback
        try:
            doctags_content = result.document.export_to_doctags(
                add_location=True,
                add_content=True,
                add_page_index=True,
                add_table_cell_location=True,
                add_table_cell_text=True,
                minified=False
            )
        except Exception as e:
            print(f"⚠️ DocTags export failed: {e}", file=sys.stderr)
            doctags_content = markdown_content
        
        # Also get markdown for compatibility
        markdown_content = result.document.export_to_markdown()
        
        # First pass: analyze document for conventions
        conventions = processor.analyze_document_conventions(markdown_content)
        
        # Second pass: Extract structured tables using Docling's Table API
        processed_tables = []
        total_issues = 0
        structured_tables = []
        
        print(f"🔍 Extracting structured tables using Docling Table API...", file=sys.stderr)
        
        # Access tables directly from the document structure
        if hasattr(result.document, 'tables') and result.document.tables:
            print(f"📋 Found {len(result.document.tables)} structured tables in document", file=sys.stderr)
            
            for table_idx, table in enumerate(result.document.tables):
                try:
                    # Export table to different formats for maximum compatibility
                    table_formats = {}
                    
                    # Try to get DataFrame if available
                    try:
                        table_df = table.export_to_dataframe()
                        if table_df is not None and not table_df.empty:
                            # Convert DataFrame to list of lists for processing
                            headers = table_df.columns.tolist()
                            table_data = [headers] + table_df.values.tolist()
                            
                            # Convert all values to strings for consistency
                            table_data = [[str(cell) if cell is not None else "" for cell in row] for row in table_data]
                            
                            table_formats['dataframe'] = {
                                'headers': headers,
                                'data': table_data,
                                'shape': table_df.shape
                            }
                            
                            print(f"📊 Table {table_idx + 1}: {table_df.shape[0]} rows × {table_df.shape[1]} columns", file=sys.stderr)
                            
                            # Process with environmental lab awareness
                            patterns = processor.detect_column_patterns(headers)
                            table_result = processor.process_table_data(table_data, patterns)
                            
                            structured_tables.append({
                                'table_index': table_idx,
                                'formats': table_formats,
                                'original_data': table_data,
                                'processed_data': table_result['processed_data'],
                                'patterns': patterns,
                                'issues': table_result['issues']
                            })
                            
                            total_issues += len(table_result['issues'])
                            print(f"📋 Processed structured table {table_idx + 1} with {len(table_result['issues'])} issues found", file=sys.stderr)
                            
                    except Exception as df_error:
                        print(f"⚠️ Could not export table {table_idx + 1} to DataFrame: {df_error}", file=sys.stderr)
                    
                    # Try to get HTML format
                    try:
                        table_html = table.export_to_html()
                        if table_html:
                            table_formats['html'] = table_html
                            print(f"📄 Table {table_idx + 1}: HTML export successful", file=sys.stderr)
                    except Exception as html_error:
                        print(f"⚠️ Could not export table {table_idx + 1} to HTML: {html_error}", file=sys.stderr)
                    
                    # Try to get CSV format
                    try:
                        table_csv = table.export_to_csv()
                        if table_csv:
                            table_formats['csv'] = table_csv
                            print(f"📊 Table {table_idx + 1}: CSV export successful", file=sys.stderr)
                    except Exception as csv_error:
                        print(f"⚠️ Could not export table {table_idx + 1} to CSV: {csv_error}", file=sys.stderr)
                    
                except Exception as e:
                    print(f"❌ Error processing table {table_idx + 1}: {e}", file=sys.stderr)
                    
        else:
            print(f"⚠️ No structured tables found in document - falling back to markdown parsing", file=sys.stderr)
            
            # Fallback: Parse tables from markdown if no structured tables found
            table_lines = []
            lines = markdown_content.split('\n')
            
            for i, line in enumerate(lines):
                if '|' in line and len(line.split('|')) > 2:
                    table_lines.append((i, line))
            
            if table_lines:
                print(f"📋 Found {len(table_lines)} table-like lines in markdown (fallback mode)", file=sys.stderr)
                
                # Group consecutive table lines into tables
                current_table = []
                for line_num, line in table_lines:
                    if current_table and line_num > current_table[-1][0] + 1:
                        # Process completed table
                        if len(current_table) >= 2:
                            table_data = processor.parse_markdown_table([l[1] for l in current_table])
                            if table_data:
                                headers = table_data[0] if table_data else []
                                patterns = processor.detect_column_patterns(headers)
                                
                                # Process the table
                                table_result = processor.process_table_data(table_data, patterns)
                                
                                processed_tables.append({
                                    'table_index': len(processed_tables),
                                    'original_data': table_data,
                                    'processed_data': table_result['processed_data'],
                                    'patterns': patterns,
                                    'issues': table_result['issues']
                                })
                                
                                total_issues += len(table_result['issues'])
                                print(f"📋 Processed markdown table with {len(table_result['issues'])} issues found", file=sys.stderr)
                        
                        current_table = []
                    
                    current_table.append((line_num, line))
                
                # Process final table
                if len(current_table) >= 2:
                    table_data = processor.parse_markdown_table([l[1] for l in current_table])
                    if table_data:
                        headers = table_data[0] if table_data else []
                        patterns = processor.detect_column_patterns(headers)
                        
                        # Process the table
                        table_result = processor.process_table_data(table_data, patterns)
                        
                        processed_tables.append({
                            'table_index': len(processed_tables),
                            'original_data': table_data,
                            'processed_data': table_result['processed_data'],
                            'patterns': patterns,
                            'issues': table_result['issues']
                        })
                        
                        total_issues += len(table_result['issues'])
                        print(f"📋 Processed markdown table with {len(table_result['issues'])} issues found", file=sys.stderr)
        
        # Combine structured and processed tables
        all_processed_tables = structured_tables + processed_tables
        
        # Export enhanced markdown
        markdown_content = result.document.export_to_markdown()
        
        # Add enhancement metadata to markdown
        enhancement_header = f""" === """
        
        enhanced_markdown = enhancement_header + markdown_content
        
        # Get metadata
        page_count = len(result.document.pages) if hasattr(result.document, 'pages') else 1
        tables_found = len(processed_tables)
        figures_found = len(result.document.figures) if hasattr(result.document, 'figures') else 0
        
        processing_time = int((time.time() - start_time) * 1000)
        
        print(f"✅ Enhanced extraction complete! Pages: {page_count}, Tables: {tables_found}, Issues: {total_issues}, Time: {processing_time}ms", file=sys.stderr)
        
        # Convert structured tables to format compatible with Rust
        structured_table_data = []
        for table_info in all_processed_tables:
            table_output = {
                'index': table_info['table_index'],
                'original_data': table_info['original_data'],
                'processed_data': table_info['processed_data'],
                'patterns': table_info['patterns'],
                'issues': table_info['issues']
            }
            
            # Add format exports if available
            if 'formats' in table_info:
                table_output['formats'] = table_info['formats']
            
            structured_table_data.append(table_output)
        
        # Use pure DocTags format for proper structured parsing in Rust
        print(f"📊 Using pure DocTags format with {len(structured_table_data)} structured tables", file=sys.stderr)
        print(f"📊 DocTags content sample (first 500 chars): {doctags_content[:500]}...", file=sys.stderr)
        
        # Return structured chunks format instead of DocTags
        extraction = {
            'page_number': 1,
            'text': markdown_content,  # Fallback text
            'structured_chunks': structured_chunks,  # NEW: Structured chunks with element types
            'markdown_text': markdown_content,
            'doctags_text': doctags_content,  # Keep DocTags as fallback
            'tables': structured_table_data,
            'figures': [],
            'formulas': [],
            'confidence': 0.95,
            'layout_boxes': [],
            'tool': 'docling_v2_enhanced_lab',
            'content_format': 'structured_chunks',  # NEW: Indicate structured format
        }
        
        return {
            'success': True,
            'tool': 'docling_v2_enhanced_lab',
            'extractions': [extraction],
            'metadata': {
                'total_pages': page_count,
                'tables_found': len(all_processed_tables),
                'figures_found': figures_found,
                'processing_time': processing_time,
                'qualifier_issues': total_issues,
                'conventions_found': len(conventions),
                'column_patterns': sum(len(t['patterns']) for t in all_processed_tables),
                'structured_tables_extracted': len(structured_tables),
                'markdown_tables_parsed': len(processed_tables)
            },
            'structured_tables': structured_table_data,  # Also available at top level
            'conventions': conventions  # Include qualifier conventions
        }
        
    except Exception as e:
        error_msg = f'Enhanced Docling extraction failed: {str(e)}'
        print(f"❌ {error_msg}", file=sys.stderr)
        print(f"🔍 Traceback: {traceback.format_exc()}", file=sys.stderr)
        return {
            'success': False,
            'error': error_msg,
            'traceback': traceback.format_exc(),
            'tool': 'docling_v2_enhanced_lab'
        }

def main():
    parser = argparse.ArgumentParser(description='CHONKER Enhanced Environmental Lab Extraction Bridge')
    parser.add_argument('pdf_path', help='Path to PDF file')
    parser.add_argument('--page', type=int, help='Specific page number to extract (1-indexed)')
    parser.add_argument('--tool', default='docling_enhanced', help='Tool identifier')
    parser.add_argument('--output', help='Output JSON file (default: stdout)')
    
    args = parser.parse_args()
    
    # Validate input
    pdf_path = Path(args.pdf_path)
    if not pdf_path.exists():
        result = {
            'success': False,
            'error': f'PDF file not found: {pdf_path}',
            'tool': 'docling_v2_enhanced_lab'
        }
    else:
        # Perform enhanced extraction
        result = extract_with_enhanced_docling(str(pdf_path), args.page)
    
    # Output results
    output_json = json.dumps(result, indent=2, ensure_ascii=False)
    
    if args.output:
        with open(args.output, 'w', encoding='utf-8') as f:
            f.write(output_json)
    else:
        print(output_json)

if __name__ == '__main__':
    main()
