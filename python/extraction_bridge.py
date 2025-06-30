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

# Docling v2 imports
from docling.document_converter import DocumentConverter
from docling.datamodel.base_models import InputFormat, Table
from docling.datamodel.pipeline_options import PdfPipelineOptions, TableFormerMode
from docling.document_converter import FormatOption

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
        
        # Create converter with enhanced options
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
        
        # Export to markdown first to get full text
        markdown_content = result.document.export_to_markdown()
        
        # First pass: analyze document for conventions
        conventions = processor.analyze_document_conventions(markdown_content)
        
        # Second pass: process tables with lab awareness
        processed_tables = []
        total_issues = 0
        
        # Extract tables from the document using Docling v2 API
        # Tables are embedded in the markdown, so we'll parse them from there
        # This is more reliable than trying to access the internal table structure
        print(f"🔍 Looking for tables in markdown content...", file=sys.stderr)
        
        # Parse tables from markdown using simple regex approach
        table_lines = []
        lines = markdown_content.split('\n')
        
        for i, line in enumerate(lines):
            if '|' in line and len(line.split('|')) > 2:
                table_lines.append((i, line))
        
        if table_lines:
            print(f"📋 Found {len(table_lines)} table-like lines in markdown", file=sys.stderr)
            
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
                                'original_data': table_data,
                                'processed_data': table_result['processed_data'],
                                'patterns': patterns,
                                'issues': table_result['issues']
                            })
                            
                            total_issues += len(table_result['issues'])
                            print(f"📋 Processed table with {len(table_result['issues'])} issues found", file=sys.stderr)
                    
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
                        'original_data': table_data,
                        'processed_data': table_result['processed_data'],
                        'patterns': patterns,
                        'issues': table_result['issues']
                    })
                    
                    total_issues += len(table_result['issues'])
                    print(f"📋 Processed table with {len(table_result['issues'])} issues found", file=sys.stderr)
        
        # Export enhanced markdown
        markdown_content = result.document.export_to_markdown()
        
        # Add enhancement metadata to markdown
        enhancement_header = f"""# Enhanced Environmental Lab Document Analysis

**Qualifier Conventions Detected:**
{chr(10).join(f"- **{k}**: {v}" for k, v in conventions.items())}

**Processing Summary:**
- Tables processed: {len(processed_tables)}
- Qualifier issues found: {total_issues}
- Column patterns detected: {sum(len(t['patterns']) for t in processed_tables)}

---

"""
        
        enhanced_markdown = enhancement_header + markdown_content
        
        # Get metadata
        page_count = len(result.document.pages) if hasattr(result.document, 'pages') else 1
        tables_found = len(processed_tables)
        figures_found = len(result.document.figures) if hasattr(result.document, 'figures') else 0
        
        processing_time = int((time.time() - start_time) * 1000)
        
        print(f"✅ Enhanced extraction complete! Pages: {page_count}, Tables: {tables_found}, Issues: {total_issues}, Time: {processing_time}ms", file=sys.stderr)
        
        # Convert to expected format
        extraction = {
            'page_number': 1,
            'text': enhanced_markdown,
            'tables': [],  # Tables are embedded in enhanced markdown
            'figures': [],
            'formulas': [],
            'confidence': 0.95,
            'layout_boxes': [],
            'tool': 'docling_v2_enhanced_lab'
        }
        
        return {
            'success': True,
            'tool': 'docling_v2_enhanced_lab',
            'extractions': [extraction],
            'metadata': {
                'total_pages': page_count,
                'tables_found': tables_found,
                'figures_found': figures_found,
                'processing_time': processing_time,
                'qualifier_issues': total_issues,
                'conventions_found': len(conventions),
                'column_patterns': sum(len(t['patterns']) for t in processed_tables),
                'processed_tables': processed_tables
            }
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
