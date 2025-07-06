#!/usr/bin/env python3
"""
Domain-Agnostic PDF Extraction Pipeline
Pipeline: PDF → Docling (Raw) → Post-Processor → JSON Output
Stages:   raw.json → processed.json → final.json

No domain-specific logic - pure table structure preservation.
"""

import json
import argparse
import sys
from pathlib import Path
from datetime import datetime
from typing import Dict, List, Any, Optional
import traceback
import re

# Import our custom table parser
from table_parser import TableParser

# Docling imports
from docling.document_converter import DocumentConverter, FormatOption
from docling.datamodel.base_models import InputFormat
from docling.datamodel.pipeline_options import PdfPipelineOptions, TableFormerMode
from docling.pipeline.standard_pdf_pipeline import StandardPdfPipeline
from docling.backend.docling_parse_backend import DoclingParseDocumentBackend

try:
    import docling
    DOCLING_VERSION = docling.__version__
except:
    DOCLING_VERSION = "unknown"

class ExtractionPipeline:
    """Domain-agnostic PDF extraction pipeline with full traceability"""
    
    def __init__(self, output_dir: str = "pipeline_outputs"):
        self.output_dir = Path(output_dir)
        self.output_dir.mkdir(exist_ok=True)
        self.outputs = {}
        self.metadata = {}
        
        # Initialize our advanced table parser
        self.table_parser = TableParser()
        
    def configure_docling(self) -> DocumentConverter:
        """Configure Docling for maximum table structure preservation"""
        
        # Key configuration for table structure preservation
        pipeline_options = PdfPipelineOptions(
            # Core table extraction
            do_table_structure=True,
            do_ocr=True,
            
            # Critical: Table structure options
            table_structure_options={
                "do_cell_matching": False,  # Key fix - prevent cell merging
                "mode": TableFormerMode.ACCURATE,  # Most accurate detection
                "detect_table_headers": True,
                "min_table_rows": 2,
                "column_separator_threshold": 0.05,
                "row_separator_threshold": 0.03,
            },
            
            # Additional processing
            do_formula_enrichment=False,  # Keep it simple
            do_picture_description=False,
            do_picture_classification=False,
            do_code_enrichment=False,
        )
        
        # Store configuration for debugging
        self.metadata['docling_config'] = {
            'pipeline_options': str(pipeline_options),
            'table_structure_options': pipeline_options.table_structure_options,
            'docling_version': DOCLING_VERSION,
            'timestamp': datetime.now().isoformat()
        }
        
        # Create converter with enhanced pipeline
        converter = DocumentConverter()
        
        enhanced_format_option = FormatOption(
            pipeline_cls=StandardPdfPipeline,
            pipeline_options=pipeline_options,
            backend=DoclingParseDocumentBackend
        )
        
        converter.format_to_options[InputFormat.PDF] = enhanced_format_option
        return converter
    
    def docling_extract(self, pdf_path: str) -> Dict[str, Any]:
        """Stage 1: Raw Docling extraction with full structure preservation"""
        print(f"🔍 Stage 1: Raw Docling extraction from {pdf_path}", file=sys.stderr)
        
        converter = self.configure_docling()
        result = converter.convert(pdf_path)
        
        # Get full document markdown for context extraction
        full_markdown = result.document.export_to_markdown()
        
        # Extract raw table structures with context
        raw_tables = []
        if hasattr(result.document, 'tables') and result.document.tables:
            print(f"📊 Found {len(result.document.tables)} tables", file=sys.stderr)
            
            for table_idx, table in enumerate(result.document.tables):
                table_data = self.extract_raw_table_structure(table, table_idx)
                if table_data:
                    # Extract context around this table from the markdown
                    context = self.extract_table_context(full_markdown, table_idx, table)
                    table_data['context'] = context
                    raw_tables.append(table_data)
        
        raw_output = {
            "metadata": {
                "source_file": pdf_path,
                "extraction_tool": "docling_raw",
                "extraction_time": datetime.now().isoformat(),
                "docling_version": DOCLING_VERSION,
                "config": self.metadata['docling_config']
            },
            "document": {
                "markdown": full_markdown,
                "page_count": len(result.document.pages) if hasattr(result.document, 'pages') else 1
            },
            "tables": raw_tables,
            "stage": "raw"
        }
        
        print(f"✅ Stage 1 complete: {len(raw_tables)} raw tables extracted", file=sys.stderr)
        return raw_output
    
    def extract_raw_table_structure(self, table, table_idx: int) -> Optional[Dict[str, Any]]:
        """Extract raw table structure preserving all cell metadata"""
        try:
            table_structure = {
                "table_index": table_idx,
                "formats": {},
                "raw_structure": None
            }
            
            # Try to get raw structure from Docling's internal representation
            if hasattr(table, 'data') and table.data:
                raw_data = table.data
                if hasattr(raw_data, 'grid') and raw_data.grid:
                    grid_structure = {
                        'num_rows': getattr(raw_data, 'num_rows', 0),
                        'num_cols': getattr(raw_data, 'num_cols', 0),
                        'grid': []
                    }
                    
                    # Extract each cell with full metadata
                    for row_idx, row in enumerate(raw_data.grid):
                        grid_row = []
                        for col_idx, cell in enumerate(row):
                            if cell is None:
                                grid_row.append(None)
                            elif hasattr(cell, 'text'):
                                cell_data = {
                                    'text': getattr(cell, 'text', ''),
                                    'row_span': getattr(cell, 'row_span', 1),
                                    'col_span': getattr(cell, 'col_span', 1),
                                    'is_header': getattr(cell, 'column_header', False) or getattr(cell, 'row_header', False),
                                    'row_index': row_idx,
                                    'col_index': col_idx,
                                    'cell_type': type(cell).__name__
                                }
                                grid_row.append(cell_data)
                            else:
                                # Simple cell
                                grid_row.append({
                                    'text': str(cell),
                                    'row_span': 1,
                                    'col_span': 1,
                                    'is_header': False,
                                    'row_index': row_idx,
                                    'col_index': col_idx,
                                    'cell_type': 'simple'
                                })
                        grid_structure['grid'].append(grid_row)
                    
                    table_structure['raw_structure'] = grid_structure
            
            # Also get standard formats for comparison
            try:
                table_df = table.export_to_dataframe()
                if table_df is not None and not table_df.empty:
                    table_structure['formats']['dataframe'] = {
                        'headers': table_df.columns.tolist(),
                        'data': table_df.values.tolist(),
                        'shape': table_df.shape
                    }
            except Exception as e:
                print(f"⚠️ DataFrame export failed for table {table_idx}: {e}")
            
            try:
                table_html = table.export_to_html()
                if table_html:
                    table_structure['formats']['html'] = table_html
            except Exception as e:
                print(f"⚠️ HTML export failed for table {table_idx}: {e}")
            
            return table_structure
            
        except Exception as e:
            print(f"❌ Error extracting table {table_idx}: {e}")
            return None
    
    def extract_table_context(self, markdown: str, table_idx: int, table) -> Dict[str, str]:
        """Extract text context around table from markdown"""
        try:
            # Split markdown into lines for easier processing
            lines = markdown.split('\n')
            
            # Find table markers in markdown (look for table patterns)
            table_start_line = None
            table_end_line = None
            
            # Look for table patterns - markdown tables start with | or have multiple | in a line
            table_lines = []
            for i, line in enumerate(lines):
                if '|' in line and line.count('|') >= 2:
                    table_lines.append(i)
            
            # Group consecutive table lines to find table boundaries
            if table_lines:
                # For the requested table_idx, find the appropriate table block
                table_blocks = []
                current_block = [table_lines[0]]
                
                for i in range(1, len(table_lines)):
                    if table_lines[i] - table_lines[i-1] <= 2:  # Allow for 1-2 line gaps (separators, etc.)
                        current_block.append(table_lines[i])
                    else:
                        table_blocks.append(current_block)
                        current_block = [table_lines[i]]
                table_blocks.append(current_block)
                
                # Select the table block corresponding to our table_idx
                if table_idx < len(table_blocks):
                    selected_block = table_blocks[table_idx]
                    table_start_line = selected_block[0]
                    table_end_line = selected_block[-1]
            
            # Extract context
            context = {
                'text_before': '',
                'text_after': '',
                'table_title': '',
                'table_notes': ''
            }
            
            if table_start_line is not None:
                # Look for text before table (up to 10 lines back)
                before_lines = []
                for i in range(max(0, table_start_line - 10), table_start_line):
                    line = lines[i].strip()
                    if line and not line.startswith('#') and '|' not in line:
                        before_lines.append(line)
                    elif line.startswith('#'):
                        # Include headers as they often contain table titles
                        before_lines.append(line)
                
                context['text_before'] = '\n'.join(before_lines)
                
                # Look for title-like text immediately before table
                if before_lines:
                    # Last non-empty line before table might be title
                    potential_title = before_lines[-1].strip()
                    if potential_title and len(potential_title) < 200:
                        context['table_title'] = potential_title
            
            if table_end_line is not None:
                # Look for text after table (up to 10 lines forward)
                after_lines = []
                for i in range(table_end_line + 1, min(len(lines), table_end_line + 11)):
                    line = lines[i].strip()
                    if line and not line.startswith('#') and '|' not in line:
                        after_lines.append(line)
                        # Break if we hit another table or major section
                        if line.startswith('#') and i > table_end_line + 3:
                            break
                
                context['text_after'] = '\n'.join(after_lines)
                
                # Look for notes-like text immediately after table
                notes_patterns = ['note:', 'notes:', 'explanation:', 'legend:', 'abbreviations:']
                for line in after_lines[:5]:  # Check first 5 lines after table
                    line_lower = line.lower()
                    if any(pattern in line_lower for pattern in notes_patterns):
                        # Found notes section, capture it and following lines
                        notes_start = after_lines.index(line)
                        context['table_notes'] = '\n'.join(after_lines[notes_start:])
                        break
            
            return context
            
        except Exception as e:
            print(f"⚠️ Error extracting context for table {table_idx}: {e}", file=sys.stderr)
            return {
                'text_before': '',
                'text_after': '',
                'table_title': '',
                'table_notes': ''
            }
    
    def post_process(self, raw_data: Dict[str, Any]) -> Dict[str, Any]:
        """Stage 2: Generic post-processing - no domain logic"""
        print("🔧 Stage 2: Generic post-processing", file=sys.stderr)
        
        processed_tables = []
        
        for table in raw_data.get('tables', []):
            if table.get('raw_structure'):
                processed_table = self.clean_extracted_data(table)
                processed_tables.append(processed_table)
        
        processed_output = {
            "metadata": {
                **raw_data['metadata'],
                "processing_stage": "post_processed",
                "processing_time": datetime.now().isoformat()
            },
            "document": raw_data['document'],
            "tables": processed_tables,
            "stage": "processed"
        }
        
        print(f"✅ Stage 2 complete: {len(processed_tables)} tables post-processed", file=sys.stderr)
        return processed_output
    
    def clean_extracted_data(self, table_data: Dict[str, Any]) -> Dict[str, Any]:
        """Generic table cleanup - no domain knowledge"""
        raw_grid = table_data['raw_structure']
        
        cleaned_grid = {
            'num_rows': raw_grid['num_rows'],
            'num_cols': raw_grid['num_cols'],
            'grid': []
        }
        
        for row in raw_grid['grid']:
            cleaned_row = []
            for cell in row:
                if cell is None:
                    cleaned_row.append(None)
                else:
                    cleaned_cell = cell.copy()
                    
                    # Split multi-value cells on whitespace (generic rule)
                    if self.contains_multiple_numbers(cell['text']):
                        values = cell['text'].split()
                        cleaned_cell['values'] = values
                        cleaned_cell['text'] = values[0] if values else cell['text']
                        cleaned_cell['has_multiple_values'] = True
                    else:
                        cleaned_cell['has_multiple_values'] = False
                    
                    # Separate qualifiers from numeric values (generic pattern)
                    cleaned_cell['qualifier'] = self.extract_qualifier(cell['text'])
                    cleaned_cell['numeric_value'] = self.extract_number(cell['text'])
                    
                    cleaned_row.append(cleaned_cell)
            
            cleaned_grid['grid'].append(cleaned_row)
        
        return {
            **table_data,
            'cleaned_structure': cleaned_grid
        }
    
    def contains_multiple_numbers(self, text: str) -> bool:
        """Check if text contains multiple numeric values"""
        if not text or not isinstance(text, str):
            return False
        
        # Find all numeric patterns (including decimals)
        numbers = re.findall(r'\d+\.?\d*', text.strip())
        return len(numbers) > 1
    
    def extract_qualifier(self, text: str) -> Optional[str]:
        """Extract letter qualifiers from text (generic pattern)"""
        if not text or not isinstance(text, str):
            return None
        
        # Look for single letters that could be qualifiers
        qualifier_match = re.search(r'\b([A-Z])\b', text.strip())
        return qualifier_match.group(1) if qualifier_match else None
    
    def extract_number(self, text: str) -> Optional[float]:
        """Extract first numeric value from text"""
        if not text or not isinstance(text, str):
            return None
        
        # Find first number (including decimals)
        number_match = re.search(r'(\d+\.?\d*)', text.strip())
        if number_match:
            try:
                return float(number_match.group(1))
            except ValueError:
                return None
        return None
    
    def structure_data(self, processed_data: Dict[str, Any]) -> Dict[str, Any]:
        """Stage 3: Advanced parsing with TableParser - handles array serialization and multi-value cells"""
        print("📋 Stage 3: Advanced table parsing with TableParser", file=sys.stderr)
        
        structured_tables = []
        parsing_errors = []
        
        for table in processed_data.get('tables', []):
            try:
                # Inject context into table metadata before parsing
                if 'context' in table:
                    table.setdefault('metadata', {})['context'] = table['context']
                
                # Use our advanced parser to handle the complex parsing issues
                table_structure = self.table_parser.parse_table(table)
                
                # Convert to both JSON and structured formats
                json_output = self.table_parser.to_json(table_structure)
                structured_output = self.table_parser.to_structured_table(table_structure)
                
                # Combine both formats for maximum flexibility
                enhanced_table = {
                    "table_index": table.get('table_index', len(structured_tables)),
                    "parsing_metadata": table_structure.metadata,
                    "structured_data": structured_output,
                    "detailed_cells": json_output,
                    "context": table.get('context', {}),
                    "parsing_success": True
                }
                
                structured_tables.append(enhanced_table)
                print(f"✅ Advanced parsing successful for table {table.get('table_index', 'unknown')}", file=sys.stderr)
                
            except Exception as e:
                print(f"⚠️ Advanced parsing failed for table {table.get('table_index', 'unknown')}: {e}", file=sys.stderr)
                
                # Fallback to original structure if parsing fails
                fallback_table = {
                    "table_index": table.get('table_index', len(structured_tables)),
                    "parsing_metadata": {"error": str(e), "fallback_used": True},
                    "raw_structure": table,
                    "parsing_success": False
                }
                
                structured_tables.append(fallback_table)
                parsing_errors.append({"table_index": table.get('table_index'), "error": str(e)})
        
        final_output = {
            "metadata": {
                **processed_data['metadata'],
                "final_stage": "advanced_parsed",
                "final_time": datetime.now().isoformat(),
                "parsing_errors": parsing_errors,
                "parser_version": "TableParser_v1.0"
            },
            "document": processed_data['document'],
            "structured_tables": structured_tables,
            "stage": "final"
        }
        
        success_count = sum(1 for t in structured_tables if t.get('parsing_success', False))
        print(f"✅ Stage 3 complete: {success_count}/{len(structured_tables)} tables successfully parsed", file=sys.stderr)
        return final_output
    
    def grid_to_structured_json(self, table_data: Dict[str, Any]) -> Dict[str, Any]:
        """Convert grid to hierarchical JSON - table-agnostic"""
        grid = table_data['cleaned_structure']
        
        # Identify header rows by checking is_header flags
        header_rows = self.find_header_rows(grid)
        data_rows = self.find_data_rows(grid)
        
        # Parse column groupings from col_span attributes
        column_groups = self.parse_column_spans(header_rows)
        
        return {
            "table_index": table_data['table_index'],
            "structure": {
                "headers": self.parse_headers(header_rows),
                "column_groups": column_groups,
                "data": self.parse_data_rows(data_rows, column_groups)
            },
            "metadata": {
                "original_dimensions": f"{grid['num_rows']}x{grid['num_cols']}",
                "header_rows": len(header_rows),
                "data_rows": len(data_rows)
            }
        }
    
    def find_header_rows(self, grid: Dict[str, Any]) -> List[List[Dict]]:
        """Find rows that contain header cells with improved detection"""
        header_rows = []
        
        for row_idx, row in enumerate(grid['grid']):
            # Multiple criteria for header detection
            is_header_row = False
            
            # 1. Explicit header flag
            if any(cell and cell.get('is_header', False) for cell in row):
                is_header_row = True
            
            # 2. Check if row contains spanning cells (common in headers)
            elif any(cell and (cell.get('col_span', 1) > 1 or cell.get('row_span', 1) > 1) for cell in row):
                is_header_row = True
            
            # 3. Check for header-like patterns (all caps, specific keywords)
            elif self.has_header_patterns(row):
                is_header_row = True
            
            # 4. Position-based detection (first few rows are often headers)
            elif row_idx < 3 and self.has_header_characteristics(row):
                is_header_row = True
                
            if is_header_row:
                header_rows.append(row)
                
        return header_rows
    
    def has_header_patterns(self, row: List[Dict]) -> bool:
        """Check for common header text patterns"""
        header_keywords = {
            'sample', 'id', 'date', 'concentration', 'conc', 'result', 'method',
            'laboratory', 'lab', 'analyte', 'parameter', 'limit', 'value',
            'screening', 'standard', 'criteria', 'msc', 'mdl', 'rl', 'qualifier'
        }
        
        text_cells = [cell.get('text', '').lower().strip() for cell in row if cell]
        non_empty_cells = [text for text in text_cells if text]
        
        if not non_empty_cells:
            return False
            
        # Check if majority of cells contain header-like keywords
        keyword_matches = sum(1 for text in non_empty_cells 
                            if any(keyword in text for keyword in header_keywords))
        
        return keyword_matches >= len(non_empty_cells) * 0.3  # 30% threshold
    
    def has_header_characteristics(self, row: List[Dict]) -> bool:
        """Check for general header characteristics"""
        text_cells = [cell.get('text', '').strip() for cell in row if cell]
        non_empty_cells = [text for text in text_cells if text and text != '-']
        
        if not non_empty_cells:
            return False
            
        # Headers often have:
        # 1. More text content than numbers
        # 2. Descriptive rather than measurement values
        numeric_cells = sum(1 for text in non_empty_cells if self.is_primarily_numeric(text))
        text_ratio = (len(non_empty_cells) - numeric_cells) / len(non_empty_cells)
        
        return text_ratio > 0.6  # More than 60% non-numeric content
    
    def is_primarily_numeric(self, text: str) -> bool:
        """Check if text is primarily numeric (values, measurements)"""
        # Remove common qualifiers and check if remaining is numeric
        cleaned_text = text.upper().strip()
        for qualifier in ['U', 'J', 'B', 'D', 'E', 'H', 'M', 'N', 'P', 'R', 'S', 'T', 'V', 'W', 'X', 'Y', 'Z']:
            cleaned_text = cleaned_text.replace(qualifier, '')
        
        cleaned_text = cleaned_text.strip()
        if not cleaned_text:
            return True  # Just a qualifier
            
        try:
            float(cleaned_text)
            return True
        except ValueError:
            return False
    
    def find_data_rows(self, grid: Dict[str, Any]) -> List[List[Dict]]:
        """Find rows that contain data cells"""
        data_rows = []
        for row in grid['grid']:
            if not any(cell and cell.get('is_header', False) for cell in row):
                # Skip empty rows
                if any(cell and cell.get('text', '').strip() for cell in row):
                    data_rows.append(row)
        return data_rows
    
    def parse_column_spans(self, header_rows: List[List[Dict]]) -> Dict[str, Any]:
        """Parse hierarchical column groupings from col_span attributes"""
        column_groups = {
            'hierarchical': [],
            'flat_mapping': {},
            'grouped_columns': {}
        }
        
        # Build hierarchical structure
        for row_idx, row in enumerate(header_rows):
            level_groups = []
            for cell in row:
                if cell:
                    group_info = {
                        'text': cell.get('text', ''),
                        'start_col': cell.get('col_index', 0),
                        'span': cell.get('col_span', 1),
                        'row_span': cell.get('row_span', 1),
                        'level': row_idx,
                        'is_spanning': cell.get('col_span', 1) > 1
                    }
                    level_groups.append(group_info)
                    
                    # Create flat mapping for easy lookup
                    for col_offset in range(cell.get('col_span', 1)):
                        col_idx = cell.get('col_index', 0) + col_offset
                        if col_idx not in column_groups['flat_mapping']:
                            column_groups['flat_mapping'][col_idx] = []
                        column_groups['flat_mapping'][col_idx].append(group_info)
                        
            column_groups['hierarchical'].append(level_groups)
        
        # Identify meaningful column groups
        column_groups['grouped_columns'] = self.identify_column_groups(column_groups['flat_mapping'])
        
        return column_groups
    
    def identify_column_groups(self, flat_mapping: Dict) -> Dict[str, List[int]]:
        """Identify meaningful column groups based on header analysis"""
        groups = {}
        
        # Group by common parent headers
        for col_idx, header_chain in flat_mapping.items():
            if len(header_chain) > 1:  # Has parent header
                parent_text = header_chain[0]['text'].strip()
                if parent_text:  # Only process non-empty parent text
                    if parent_text not in groups:
                        groups[parent_text] = []
                    if col_idx not in groups[parent_text]:
                        groups[parent_text].append(col_idx)
        
        # Group by semantic similarity (e.g., all "Conc" columns)
        semantic_groups = {}
        for col_idx, header_chain in flat_mapping.items():
            leaf_header = header_chain[-1]['text'].strip().lower() if header_chain else ''
            
            # Common analytical chemistry column types
            if 'conc' in leaf_header or 'concentration' in leaf_header:
                semantic_groups.setdefault('concentrations', []).append(col_idx)
            elif leaf_header in ['q', 'qual', 'qualifier']:
                semantic_groups.setdefault('qualifiers', []).append(col_idx)
            elif leaf_header in ['rl', 'reporting', 'limit']:
                semantic_groups.setdefault('reporting_limits', []).append(col_idx)
            elif leaf_header in ['mdl', 'detection', 'limit']:
                semantic_groups.setdefault('detection_limits', []).append(col_idx)
        
        groups.update(semantic_groups)
        return groups
    
    def parse_headers(self, header_rows: List[List[Dict]]) -> List[List[str]]:
        """Extract header text from header rows"""
        headers = []
        for row in header_rows:
            header_row = []
            for cell in row:
                text = cell.get('text', '') if cell else ''
                header_row.append(text)
            headers.append(header_row)
        return headers
    
    def parse_data_rows(self, data_rows: List[List[Dict]], column_groups: Dict) -> List[List[Dict]]:
        """Parse data rows with cell metadata"""
        parsed_data = []
        
        for row in data_rows:
            row_data = []
            for cell in row:
                if cell:
                    cell_info = {
                        'text': cell.get('text', ''),
                        'numeric_value': cell.get('numeric_value'),
                        'qualifier': cell.get('qualifier'),
                        'has_multiple_values': cell.get('has_multiple_values', False)
                    }
                    if cell.get('values'):
                        cell_info['all_values'] = cell['values']
                    row_data.append(cell_info)
                else:
                    row_data.append({'text': '', 'numeric_value': None, 'qualifier': None})
            
            parsed_data.append(row_data)
        
        return parsed_data
    
    def save(self, stage: str, data: Dict[str, Any]) -> Path:
        """Save pipeline output for debugging"""
        filename = f"{stage}.json"
        filepath = self.output_dir / filename
        
        with open(filepath, 'w', encoding='utf-8') as f:
            json.dump(data, f, indent=2, ensure_ascii=False, default=str)
        
        print(f"💾 Saved {stage} output to {filepath}", file=sys.stderr)
        self.outputs[stage] = filepath
        return filepath
    
    def run(self, pdf_path: str, save_intermediates: bool = True) -> Dict[str, Any]:
        """Run complete pipeline with optional intermediate saves"""
        print(f"🚀 Starting domain-agnostic extraction pipeline for {pdf_path}", file=sys.stderr)
        
        # Stage 1: Raw Docling
        raw = self.docling_extract(pdf_path)
        if save_intermediates:
            self.save("raw", raw)
        
        # Stage 2: Post-process
        processed = self.post_process(raw)
        if save_intermediates:
            self.save("processed", processed)
        
        # Stage 3: Structure
        final = self.structure_data(processed)
        if save_intermediates:
            self.save("final", final)
        
        print("🎉 Pipeline complete!", file=sys.stderr)
        return final
    
    def debug_mode(self, pdf_path: str) -> Dict[str, Any]:
        """Run pipeline with full debugging and validation"""
        print("🐛 Running in debug mode with full traceability", file=sys.stderr)
        
        results = self.run(pdf_path, save_intermediates=True)
        self.validate_pipeline()
        
        return results
    
    def validate_pipeline(self):
        """Check each stage for data integrity"""
        print("🔍 Validating pipeline integrity...", file=sys.stderr)
        
        if 'raw' not in self.outputs or 'final' not in self.outputs:
            print("⚠️  Missing pipeline outputs for validation", file=sys.stderr)
            return
        
        # Load outputs
        with open(self.outputs['raw']) as f:
            raw = json.load(f)
        with open(self.outputs['final']) as f:
            final = json.load(f)
        
        # Basic validation
        raw_table_count = len(raw.get('tables', []))
        final_table_count = len(final.get('structured_tables', []))
        
        print(f"📊 Raw tables: {raw_table_count}, Final tables: {final_table_count}", file=sys.stderr)
        
        if raw_table_count == final_table_count:
            print("✅ Table count preserved through pipeline", file=sys.stderr)
        else:
            print("❌ Table count mismatch - data loss detected", file=sys.stderr)
        
        # Check for table structure preservation
        for i, raw_table in enumerate(raw.get('tables', [])):
            if raw_table.get('raw_structure'):
                raw_dims = f"{raw_table['raw_structure']['num_rows']}x{raw_table['raw_structure']['num_cols']}"
                print(f"  Table {i}: {raw_dims}", file=sys.stderr)
        
        print("✅ Pipeline validation complete", file=sys.stderr)

def main():
    parser = argparse.ArgumentParser(description='Domain-Agnostic PDF Extraction Pipeline')
    parser.add_argument('--debug', action='store_true', help='Run in debug mode with full traceability')
    parser.add_argument('--docling-only', action='store_true', help='Extract Docling only (no post-processing)')
    parser.add_argument('--compare', nargs=2, metavar=('RUN1', 'RUN2'), help='Compare two extraction runs')
    parser.add_argument('--output-dir', default='pipeline_outputs', help='Output directory for pipeline files')
    parser.add_argument('pdf_file', nargs='?', default='test.pdf', help='PDF file to process (default: test.pdf)')
    
    args = parser.parse_args()
    
    if args.compare:
        print(f"🔍 Comparing {args.compare[0]} vs {args.compare[1]}", file=sys.stderr)
        # TODO: Implement comparison logic
        return
    
    # Validate PDF exists
    pdf_path = Path(args.pdf_file)
    if not pdf_path.exists():
        print(f"❌ PDF file not found: {pdf_path}", file=sys.stderr)
        sys.exit(1)
    
    # Create pipeline
    pipeline = ExtractionPipeline(output_dir=args.output_dir)
    
    try:
        if args.debug:
            results = pipeline.debug_mode(str(pdf_path))
        elif args.docling_only:
            raw = pipeline.docling_extract(str(pdf_path))
            pipeline.save("raw", raw)
            results = raw
        else:
            results = pipeline.run(str(pdf_path))
        
        print(f"🎯 Results: {len(results.get('structured_tables', results.get('tables', [])))} tables extracted", file=sys.stderr)
        
    except Exception as e:
        print(f"❌ Pipeline failed: {e}", file=sys.stderr)
        traceback.print_exc()
        sys.exit(1)

if __name__ == '__main__':
    main()
