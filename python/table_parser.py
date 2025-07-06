#!/usr/bin/env python3
"""
Table Parser Module
Handles post-Docling table structure parsing and data cleanup.

This module takes Docling's extracted table structures and performs:
1. Array deserialization from text representations
2. Multi-value cell splitting and parsing
3. Column structure reconstruction
4. Data type conversion and cleanup
"""

import json
import re
import sys
from typing import Dict, List, Any, Optional, Tuple, Union
from dataclasses import dataclass
import traceback


@dataclass
class CellValue:
    """Represents a parsed cell value with metadata"""
    raw_text: str
    parsed_values: List[str]
    numeric_values: List[float]
    qualifiers: List[str]
    is_header: bool = False
    is_empty: bool = False
    cell_type: str = "data"


@dataclass
class TableStructure:
    """Represents the parsed table structure"""
    headers: List[List[str]]
    data_rows: List[List[CellValue]]
    column_mapping: Dict[int, str]
    metadata: Dict[str, Any]


class TableParser:
    """
    Parses Docling-extracted table structures into clean, structured data.
    
    Handles the specific parsing challenges that Docling can't solve:
    - Array-serialized text content
    - Multi-value cells that need splitting
    - Column structure reconstruction
    - Data type inference and conversion
    """
    
    def __init__(self):
        self.debug = False
        
    def parse_table(self, docling_table: Dict[str, Any]) -> TableStructure:
        """
        Main entry point: parse a Docling table structure
        
        Args:
            docling_table: Table structure from Docling extraction
            
        Returns:
            TableStructure: Cleaned and parsed table structure
        """
        try:
            print(f"🔧 Parsing table {docling_table.get('table_index', 'unknown')}", file=sys.stderr)
            
            # Step 1: Extract raw grid structure
            raw_grid = self._extract_raw_grid(docling_table)
            if not raw_grid:
                raise ValueError("No valid grid structure found")
                
            # Step 2: Parse cell contents (handle arrays, multi-values)
            parsed_cells = self._parse_all_cells(raw_grid)
            
            # Step 3: Identify table structure (headers vs data)
            headers, data_rows = self._identify_table_structure(parsed_cells)
            
            # Step 4: Reconstruct column mapping
            column_mapping = self._reconstruct_columns(headers, data_rows)
            
            # Step 5: Clean and validate data
            cleaned_data = self._clean_data_rows(data_rows, column_mapping)
            
            # Include context metadata if available
            metadata = {
                'original_dimensions': f"{raw_grid['num_rows']}x{raw_grid['num_cols']}",
                'parsed_columns': len(column_mapping),
                'data_rows': len(cleaned_data),
                'header_rows': len(headers)
            }
            
            # Add context from the original docling_table
            if 'metadata' in docling_table and 'context' in docling_table['metadata']:
                metadata['context'] = docling_table['metadata']['context']
            elif 'context' in docling_table:
                metadata['context'] = docling_table['context']
            
            structure = TableStructure(
                headers=headers,
                data_rows=cleaned_data,
                column_mapping=column_mapping,
                metadata=metadata
            )
            
            print(f"✅ Parsed table: {len(headers)} header rows, {len(cleaned_data)} data rows, {len(column_mapping)} columns", file=sys.stderr)
            return structure
            
        except Exception as e:
            print(f"❌ Error parsing table: {e}", file=sys.stderr)
            if self.debug:
                traceback.print_exc()
            raise
    
    def _extract_raw_grid(self, docling_table: Dict[str, Any]) -> Optional[Dict[str, Any]]:
        """Extract the raw grid structure from various Docling formats"""
        
        # Try structured format first
        if 'raw_structure' in docling_table and docling_table['raw_structure']:
            return docling_table['raw_structure']
            
        # Try cleaned structure
        if 'cleaned_structure' in docling_table and docling_table['cleaned_structure']:
            return docling_table['cleaned_structure']
            
        # Try formats
        if 'formats' in docling_table:
            formats = docling_table['formats']
            if 'dataframe' in formats and formats['dataframe']:
                df_data = formats['dataframe']
                return {
                    'num_rows': df_data['shape'][0] + 1,  # +1 for headers
                    'num_cols': df_data['shape'][1],
                    'grid': [df_data['headers']] + df_data['data']
                }
        
        return None
    
    def _parse_all_cells(self, raw_grid: Dict[str, Any]) -> List[List[CellValue]]:
        """Parse all cells in the grid, handling various text formats"""
        parsed_grid = []
        
        for row_idx, row in enumerate(raw_grid.get('grid', [])):
            parsed_row = []
            
            for col_idx, cell in enumerate(row):
                parsed_cell = self._parse_cell(cell, row_idx, col_idx)
                parsed_row.append(parsed_cell)
                
            parsed_grid.append(parsed_row)
            
        return parsed_grid
    
    def _parse_cell(self, cell: Any, row_idx: int, col_idx: int) -> CellValue:
        """Parse a single cell, handling various input formats"""
        
        # Handle None/empty cells
        if cell is None:
            return CellValue(
                raw_text="",
                parsed_values=[],
                numeric_values=[],
                qualifiers=[],
                is_empty=True
            )
        
        # Extract text content based on cell format
        raw_text = self._extract_cell_text(cell)
        
        # Handle array-serialized content (main issue from current output)
        if self._is_array_serialized(raw_text):
            parsed_values = self._parse_array_content(raw_text)
        else:
            parsed_values = self._parse_multi_value_content(raw_text)
        
        # Extract numeric values and qualifiers
        numeric_values = []
        qualifiers = []
        
        for value in parsed_values:
            nums = self._extract_numbers(value)
            quals = self._extract_qualifiers(value)
            numeric_values.extend(nums)
            qualifiers.extend(quals)
        
        # Determine cell characteristics
        is_header = self._is_header_cell(cell, raw_text, row_idx)
        cell_type = self._determine_cell_type(raw_text, numeric_values, qualifiers)
        
        return CellValue(
            raw_text=raw_text,
            parsed_values=parsed_values,
            numeric_values=numeric_values,
            qualifiers=qualifiers,
            is_header=is_header,
            is_empty=len(parsed_values) == 0,
            cell_type=cell_type
        )
    
    def _extract_cell_text(self, cell: Any) -> str:
        """Extract text content from various cell formats"""
        
        if isinstance(cell, str):
            return cell.strip()
        elif isinstance(cell, dict):
            return str(cell.get('text', '')).strip()
        elif hasattr(cell, 'text'):
            return str(cell.text).strip()
        else:
            return str(cell).strip()
    
    def _is_array_serialized(self, text: str) -> bool:
        """Check if text is array-serialized like ['SAMPLE ID: ', ...]"""
        return (text.startswith('[') and text.endswith(']') and 
                ("'" in text or '"' in text))
    
    def _parse_array_content(self, text: str) -> List[str]:
        """Parse array-serialized content"""
        try:
            # Try to evaluate as Python literal
            import ast
            parsed = ast.literal_eval(text)
            if isinstance(parsed, list):
                return [str(item).strip() for item in parsed if str(item).strip()]
        except (ValueError, SyntaxError):
            pass
        
        # Fallback: manual parsing
        # Remove brackets and split on commas, handling quotes
        content = text.strip('[]')
        values = []
        
        # Simple regex to split on commas outside quotes
        parts = re.split(r",(?=(?:[^']*'[^']*')*[^']*$)", content)
        for part in parts:
            clean_part = part.strip().strip("'\"").strip()
            if clean_part:
                values.append(clean_part)
        
        return values
    
    def _parse_multi_value_content(self, text: str) -> List[str]:
        """Parse multi-value content like '27900 1310 262'"""
        if not text:
            return []
        
        # Split on whitespace, but preserve meaningful groupings
        values = []
        
        # Handle common patterns
        if self._contains_multiple_numbers(text):
            # Split numeric values
            numbers = re.findall(r'\d+\.?\d*', text)
            qualifiers = re.findall(r'\b[A-Z]\b', text)
            
            # Combine back strategically
            for i, num in enumerate(numbers):
                if i < len(qualifiers):
                    values.append(f"{num}{qualifiers[i]}")
                else:
                    values.append(num)
        else:
            # Simple whitespace split for non-numeric content
            parts = text.split()
            values = [part for part in parts if part.strip()]
        
        return values if values else [text]
    
    def _contains_multiple_numbers(self, text: str) -> bool:
        """Check if text contains multiple numeric values"""
        numbers = re.findall(r'\d+\.?\d*', text)
        return len(numbers) > 1
    
    def _extract_numbers(self, text: str) -> List[float]:
        """Extract numeric values from text"""
        numbers = []
        for match in re.finditer(r'\d+\.?\d*', text):
            try:
                numbers.append(float(match.group()))
            except ValueError:
                continue
        return numbers
    
    def _extract_qualifiers(self, text: str) -> List[str]:
        """Extract letter qualifiers from text"""
        return re.findall(r'\b[A-Z]\b', text)
    
    def _is_header_cell(self, cell: Any, text: str, row_idx: int) -> bool:
        """Determine if this is a header cell"""
        
        # Check explicit header flag
        if isinstance(cell, dict) and cell.get('is_header'):
            return True
        
        # Position-based detection (first few rows)
        if row_idx < 3:
            # Header-like content patterns
            header_keywords = [
                'sample', 'id', 'date', 'concentration', 'conc', 'analyte',
                'lab', 'laboratory', 'method', 'result', 'limit', 'criteria',
                'mdl', 'rl', 'qualifier', 'depth', 'matrix'
            ]
            
            text_lower = text.lower()
            if any(keyword in text_lower for keyword in header_keywords):
                return True
                
            # Non-numeric content in early rows
            if not re.search(r'\d+\.?\d*', text) and len(text) > 2:
                return True
        
        return False
    
    def _determine_cell_type(self, text: str, numeric_values: List[float], qualifiers: List[str]) -> str:
        """Determine the type of cell content"""
        
        if not text or text in ['-', 'NA', 'N/A', '']:
            return 'empty'
        elif numeric_values and qualifiers:
            return 'qualified_numeric'
        elif numeric_values:
            return 'numeric'
        elif qualifiers:
            return 'qualifier'
        elif re.match(r'\d{1,2}/\d{1,2}/\d{4}', text):
            return 'date'
        elif text.upper() in ['SOIL', 'WATER', 'AIR']:
            return 'matrix'
        else:
            return 'text'
    
    def _identify_table_structure(self, parsed_cells: List[List[CellValue]]) -> Tuple[List[List[str]], List[List[CellValue]]]:
        """Separate headers from data rows"""
        
        headers = []
        data_rows = []
        
        for row_idx, row in enumerate(parsed_cells):
            # Check if this row is primarily headers
            header_count = sum(1 for cell in row if cell.is_header)
            total_cells = len([cell for cell in row if not cell.is_empty])
            
            if total_cells > 0 and (header_count / total_cells) > 0.5:
                # This is a header row
                header_row = []
                for cell in row:
                    # Use the first parsed value or raw text
                    if cell.parsed_values:
                        header_row.append(cell.parsed_values[0])
                    else:
                        header_row.append(cell.raw_text)
                headers.append(header_row)
            else:
                # This is a data row
                data_rows.append(row)
        
        return headers, data_rows
    
    def _reconstruct_columns(self, headers: List[List[str]], data_rows: List[List[CellValue]]) -> Dict[int, str]:
        """Reconstruct column mapping from headers and data"""
        
        column_mapping = {}
        
        # Use the last header row as primary column names
        if headers:
            primary_headers = headers[-1]
            for i, header in enumerate(primary_headers):
                if header and header.strip():
                    column_mapping[i] = header.strip()
                else:
                    column_mapping[i] = f"Column_{i}"
        
        # Extend mapping based on actual data width
        if data_rows:
            max_cols = max(len(row) for row in data_rows)
            for i in range(len(column_mapping), max_cols):
                column_mapping[i] = f"Column_{i}"
        
        return column_mapping
    
    def _clean_data_rows(self, data_rows: List[List[CellValue]], column_mapping: Dict[int, str]) -> List[List[CellValue]]:
        """Clean and validate data rows"""
        
        cleaned_rows = []
        
        for row in data_rows:
            # Skip completely empty rows
            if all(cell.is_empty for cell in row):
                continue
                
            # Ensure row has correct width
            while len(row) < len(column_mapping):
                row.append(CellValue(
                    raw_text="",
                    parsed_values=[],
                    numeric_values=[],
                    qualifiers=[],
                    is_empty=True
                ))
            
            cleaned_rows.append(row)
        
        return cleaned_rows
    
    def to_json(self, structure: TableStructure) -> Dict[str, Any]:
        """Convert parsed structure to JSON format"""
        
        # Convert data rows to JSON-serializable format
        json_data_rows = []
        for row in structure.data_rows:
            json_row = []
            for cell in row:
                cell_dict = {
                    'raw_text': cell.raw_text,
                    'parsed_values': cell.parsed_values,
                    'numeric_values': cell.numeric_values,
                    'qualifiers': cell.qualifiers,
                    'is_empty': cell.is_empty,
                    'cell_type': cell.cell_type
                }
                json_row.append(cell_dict)
            json_data_rows.append(json_row)
        
        return {
            'headers': structure.headers,
            'data': json_data_rows,
            'column_mapping': structure.column_mapping,
            'metadata': structure.metadata
        }
    
    def to_structured_table(self, structure: TableStructure) -> Dict[str, Any]:
        """Convert to structured table format matching user preferences"""
        
        # Build structured data as arrays/objects, not markdown
        table_data = {
            'headers': structure.headers,
            'columns': structure.column_mapping,
            'rows': [],
            'context': structure.metadata.get('context', {})
        }
        
        for row in structure.data_rows:
            row_dict = {}
            for col_idx, cell in enumerate(row):
                column_name = structure.column_mapping.get(col_idx, f"col_{col_idx}")
                
                # Store structured cell data
                if cell.numeric_values:
                    # Prefer numeric values when available
                    if len(cell.numeric_values) == 1:
                        row_dict[column_name] = cell.numeric_values[0]
                    else:
                        row_dict[column_name] = cell.numeric_values
                        
                    # Add qualifiers if present
                    if cell.qualifiers:
                        row_dict[f"{column_name}_qualifiers"] = cell.qualifiers
                else:
                    # Use parsed text values
                    if len(cell.parsed_values) == 1:
                        row_dict[column_name] = cell.parsed_values[0]
                    elif cell.parsed_values:
                        row_dict[column_name] = cell.parsed_values
                    else:
                        row_dict[column_name] = cell.raw_text
            
            table_data['rows'].append(row_dict)
        
        return table_data


def main():
    """Test the parser with sample data"""
    
    # Example usage
    if len(sys.argv) < 2:
        print("Usage: python table_parser.py <docling_json_file>")
        sys.exit(1)
    
    json_file = sys.argv[1]
    
    try:
        with open(json_file, 'r') as f:
            docling_output = json.load(f)
        
        parser = TableParser()
        parser.debug = True
        
        for table in docling_output.get('tables', []):
            try:
                structure = parser.parse_table(table)
                json_output = parser.to_json(structure)
                structured_output = parser.to_structured_table(structure)
                
                print("\n" + "="*50)
                print(f"Parsed Table {table.get('table_index', 'unknown')}")
                print("="*50)
                print(json.dumps(structured_output, indent=2))
                
            except Exception as e:
                print(f"Failed to parse table {table.get('table_index', 'unknown')}: {e}")
    
    except Exception as e:
        print(f"Error: {e}")
        sys.exit(1)


if __name__ == '__main__':
    main()
