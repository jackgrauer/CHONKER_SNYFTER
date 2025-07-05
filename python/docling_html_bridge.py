#!/usr/bin/env python3
"""
CHONKER HTML Bridge - Docling to HTML Pipeline
Converts PDFs to HTML using Docling for document-agnostic processing
"""

import argparse
import json
import sys
import traceback
from pathlib import Path

try:
    from docling.document_converter import DocumentConverter
    from docling.datamodel.base_models import InputFormat
    from docling.datamodel.pipeline_options import PdfPipelineOptions
    from docling.document_converter import PdfFormatOption
except ImportError as e:
    print(f"Error: Docling not available: {e}", file=sys.stderr)
    print("Install with: pip install docling", file=sys.stderr)
    sys.exit(1)


class DoclingHtmlBridge:
    """Bridge to convert documents to HTML using Docling"""
    
    def __init__(self):
        self.setup_docling()
    
    def setup_docling(self):
        """Setup Docling converter with optimal settings"""
        # Enhanced pipeline options for better HTML output
        pipeline_options = PdfPipelineOptions(
            do_ocr=True,
            do_table_structure=True,
            table_structure_options={
                "do_cell_matching": True,
                "mode": "accurate"  # More accurate table detection
            },
            images_scale=2.0,  # Higher resolution for better OCR
        )
        
        # Format options
        format_options = {
            InputFormat.PDF: PdfFormatOption(
                pipeline_options=pipeline_options
            )
        }
        
        self.converter = DocumentConverter(
            format_options=format_options
        )
    
    def convert_to_structured_json(self, pdf_path: str, output_path: str = None) -> str:
        """
        Convert PDF to structured JSON using Docling with table structure preserved
        
        Args:
            pdf_path: Path to input PDF file
            output_path: Optional path to save JSON output
            
        Returns:
            JSON content as string with all document structure preserved
        """
        try:
            # Convert document
            source = Path(pdf_path)
            conversion_result = self.converter.convert(source)
            
            # Get the converted document
            document = conversion_result.document
            
            # Extract structured elements
            structured_data = self.extract_structured_elements(document, pdf_path)
            
            # Convert to JSON
            json_content = json.dumps(structured_data, indent=2, ensure_ascii=False)
            
            # Save to file if requested
            if output_path:
                with open(output_path, 'w', encoding='utf-8') as f:
                    f.write(json_content)
                print(f"Structured JSON saved to: {output_path}", file=sys.stderr)
            
            return json_content
            
        except Exception as e:
            error_msg = f"Error converting {pdf_path} to JSON: {str(e)}"
            print(error_msg, file=sys.stderr)
            traceback.print_exc(file=sys.stderr)
            raise
    
    def convert_to_markdown(self, pdf_path: str, output_path: str = None) -> str:
        """
        Convert PDF to markdown using Docling - simple and accurate
        
        Args:
            pdf_path: Path to input PDF file
            output_path: Optional path to save markdown output
            
        Returns:
            Markdown content as string with all document structure preserved
        """
        try:
            # Convert document
            source = Path(pdf_path)
            conversion_result = self.converter.convert(source)
            
            # Get the converted document
            document = conversion_result.document
            
            # Export as markdown - this preserves all content with structure
            markdown_content = document.export_to_markdown()
            
            # Add CHONKER metadata as markdown comments
            enhanced_markdown = self.enhance_markdown_output(markdown_content, pdf_path, document)
            
            # Save to file if requested
            if output_path:
                with open(output_path, 'w', encoding='utf-8') as f:
                    f.write(enhanced_markdown)
                print(f"Markdown saved to: {output_path}", file=sys.stderr)
            
            return enhanced_markdown
            
        except Exception as e:
            error_msg = f"Error converting {pdf_path} to markdown: {str(e)}"
            print(error_msg, file=sys.stderr)
            traceback.print_exc(file=sys.stderr)
            raise
    
    def enhance_markdown_output(self, markdown_content: str, source_file: str, document) -> str:
        """
        Enhance markdown output with CHONKER-specific metadata
        
        Args:
            markdown_content: Original markdown from Docling
            source_file: Source PDF file path
            document: Docling document object
            
        Returns:
            Enhanced markdown with metadata header
        """
        # Add metadata header as markdown comments
        metadata_header = f"""<!-- CHONKER Document Metadata
Source: {source_file}
Extraction Tool: Docling Markdown Bridge
Extraction Time: {self.get_current_timestamp()}
Page Count: {getattr(document, 'page_count', 'unknown')}
-->

"""
        
        # Prepend metadata to markdown content
        enhanced_markdown = metadata_header + markdown_content
        
        return enhanced_markdown
    
    def enhance_elements_for_accuracy(self, elements: list) -> list:
        """
        Enhance document elements with better typing and structure for accurate rendering
        """
        enhanced_elements = []
        
        for element in elements:
            if isinstance(element, dict):
                # Add element type detection
                element_type = element.get("type", "unknown")
                
                # Enhance table elements
                if "table" in element_type.lower():
                    element["chonker_enhanced_type"] = "table"
                    # Ensure table structure is preserved
                    if "data" in element:
                        element["chonker_table_structure"] = self.analyze_table_structure(element["data"])
                
                # Enhance text elements
                elif element_type in ["paragraph", "text"]:
                    element["chonker_enhanced_type"] = "text"
                    # Preserve text formatting
                    if "text" in element:
                        element["chonker_text_analysis"] = self.analyze_text_content(element["text"])
                
                # Enhance heading elements
                elif "heading" in element_type.lower():
                    element["chonker_enhanced_type"] = "heading"
                    level = element.get("level", 1)
                    element["chonker_heading_level"] = min(max(level, 1), 6)  # Ensure valid range
                
                # Enhance list elements
                elif "list" in element_type.lower():
                    element["chonker_enhanced_type"] = "list"
                    element["chonker_list_type"] = "ordered" if "ordered" in element_type.lower() else "unordered"
                
                enhanced_elements.append(element)
            else:
                enhanced_elements.append(element)
        
        return enhanced_elements
    
    def analyze_table_structure(self, table_data) -> dict:
        """
        Analyze table structure to ensure accurate rendering
        """
        analysis = {
            "has_headers": False,
            "row_count": 0,
            "col_count": 0,
            "cell_types": []
        }
        
        if isinstance(table_data, list) and table_data:
            analysis["row_count"] = len(table_data)
            if isinstance(table_data[0], list):
                analysis["col_count"] = len(table_data[0])
                # Analyze first row to detect headers
                first_row = table_data[0]
                analysis["has_headers"] = any(
                    isinstance(cell, dict) and cell.get("is_header", False) 
                    for cell in first_row
                )
        
        return analysis
    
    def analyze_text_content(self, text: str) -> dict:
        """
        Analyze text content for formatting preservation
        """
        return {
            "length": len(text),
            "has_bold": "**" in text or "<strong>" in text,
            "has_italic": "*" in text or "<em>" in text,
            "has_code": "`" in text or "<code>" in text,
            "line_count": len(text.split("\n"))
        }
    
    def extract_structured_elements(self, document, source_file: str) -> dict:
        """
        Extract structured elements from Docling document with table data preserved
        
        Args:
            document: Docling document object
            source_file: Source PDF file path
            
        Returns:
            Dictionary with structured document elements
        """
        structured_data = {
            "metadata": {
                "source_file": source_file,
                "extraction_tool": "Docling Structured Bridge",
                "extraction_time": self.get_current_timestamp(),
                "page_count": getattr(document, 'page_count', 0)
            },
            "elements": []
        }
        
        try:
            # Process tables first (most important for accuracy)
            if hasattr(document, 'tables') and document.tables:
                print(f"Found {len(document.tables)} tables", file=sys.stderr)
                for i, table in enumerate(document.tables):
                    table_element = self.extract_table_structure(table, i)
                    if table_element:
                        structured_data["elements"].append(table_element)
            
            # Process text elements
            if hasattr(document, 'texts') and document.texts:
                print(f"Found {len(document.texts)} text elements", file=sys.stderr)
                for i, text in enumerate(document.texts):
                    text_element = self.extract_text_element(text, i)
                    if text_element:
                        structured_data["elements"].append(text_element)
            
            # If no structured elements found, extract from document body
            if not structured_data["elements"]:
                print("No structured elements found, extracting main text", file=sys.stderr)
                main_text = document.export_to_markdown()
                if main_text.strip():
                    structured_data["elements"].append({
                        "id": "main_content",
                        "type": "text",
                        "content": main_text,
                        "page_number": 1,
                        "element_index": 0
                    })
        
        except Exception as e:
            print(f"Error extracting structured elements: {e}", file=sys.stderr)
            traceback.print_exc(file=sys.stderr)
        
        return structured_data
    
    def extract_table_structure(self, table, table_index: int) -> dict:
        """
        Extract table structure with all data preserved
        
        Args:
            table: Docling table object
            table_index: Index of the table in the document
            
        Returns:
            Dictionary with table structure and data
        """
        try:
            table_data = {
                "id": f"table_{table_index}",
                "type": "table",
                "element_index": table_index,
                "table_structure": {
                    "num_rows": table.num_rows if hasattr(table, 'num_rows') else 0,
                    "num_cols": table.num_cols if hasattr(table, 'num_cols') else 0,
                    "cells": []
                }
            }
            
            # Extract cell data if available
            if hasattr(table, 'table_cells') and table.table_cells:
                for cell in table.table_cells:
                    cell_data = {
                        "text": getattr(cell, 'text', ''),
                        "row_span": getattr(cell, 'row_span', 1),
                        "col_span": getattr(cell, 'col_span', 1),
                        "start_row": getattr(cell, 'start_row_offset_idx', 0),
                        "end_row": getattr(cell, 'end_row_offset_idx', 1),
                        "start_col": getattr(cell, 'start_col_offset_idx', 0),
                        "end_col": getattr(cell, 'end_col_offset_idx', 1),
                        "is_header": getattr(cell, 'column_header', False) or getattr(cell, 'row_header', False)
                    }
                    table_data["table_structure"]["cells"].append(cell_data)
            
            # Try to extract grid representation
            if hasattr(table, 'data') and table.data:
                # Convert table data to serializable format
                try:
                    grid_data = self.convert_table_data_to_json(table.data)
                    table_data["grid_data"] = grid_data
                except Exception as e:
                    print(f"Warning: Could not serialize table data: {e}", file=sys.stderr)
            
            print(f"Extracted table {table_index} with {len(table_data['table_structure']['cells'])} cells", file=sys.stderr)
            return table_data
        
        except Exception as e:
            print(f"Error extracting table {table_index}: {e}", file=sys.stderr)
            return None
    
    def convert_table_data_to_json(self, table_data) -> dict:
        """
        Convert Docling TableData object to JSON-serializable format
        
        Args:
            table_data: Docling TableData object
            
        Returns:
            JSON-serializable dictionary representation
        """
        try:
            json_data = {
                "num_rows": getattr(table_data, 'num_rows', 0),
                "num_cols": getattr(table_data, 'num_cols', 0),
                "grid": []
            }
            
            # Convert grid data
            if hasattr(table_data, 'grid') and table_data.grid:
                for row in table_data.grid:
                    json_row = []
                    for cell in row:
                        if cell is None:
                            json_row.append(None)
                        elif hasattr(cell, 'text'):
                            json_row.append({
                                "text": getattr(cell, 'text', ''),
                                "row_span": getattr(cell, 'row_span', 1),
                                "col_span": getattr(cell, 'col_span', 1)
                            })
                        else:
                            # Simple text cell
                            json_row.append(str(cell))
                    json_data["grid"].append(json_row)
            
            return json_data
        
        except Exception as e:
            print(f"Error converting table data: {e}", file=sys.stderr)
            return {"error": "table_data_conversion_failed"}
    
    def extract_text_element(self, text_element, text_index: int) -> dict:
        """
        Extract text element with formatting preserved
        
        Args:
            text_element: Docling text object
            text_index: Index of the text element
            
        Returns:
            Dictionary with text element data
        """
        try:
            text_data = {
                "id": f"text_{text_index}",
                "type": "text",
                "content": getattr(text_element, 'text', ''),
                "element_index": text_index
            }
            
            # Add any additional metadata
            if hasattr(text_element, 'level'):
                text_data["heading_level"] = text_element.level
                text_data["type"] = "heading"
            
            return text_data if text_data["content"].strip() else None
        
        except Exception as e:
            print(f"Error extracting text element {text_index}: {e}", file=sys.stderr)
            return None
    
    def get_current_timestamp(self) -> str:
        """Get current timestamp in ISO format"""
        from datetime import datetime, timezone
        return datetime.now(timezone.utc).isoformat()
    
    def analyze_document_structure(self, document) -> dict:
        """
        Analyze document structure for enhanced HTML output
        
        Args:
            document: Docling document object
            
        Returns:
            Dictionary with structure analysis
        """
        analysis = {
            "element_counts": {},
            "has_tables": False,
            "has_images": False,
            "has_forms": False,
            "document_type": "unknown"
        }
        
        try:
            # Count elements by type
            for element in getattr(document, 'elements', []):
                element_type = type(element).__name__
                analysis["element_counts"][element_type] = analysis["element_counts"].get(element_type, 0) + 1
                
                # Detect special content
                if 'table' in element_type.lower():
                    analysis["has_tables"] = True
                elif 'image' in element_type.lower():
                    analysis["has_images"] = True
                elif 'form' in element_type.lower():
                    analysis["has_forms"] = True
            
            # Determine document type based on content
            if analysis["has_tables"] and analysis["element_counts"].get("TableElement", 0) > 2:
                analysis["document_type"] = "data_report"
            elif analysis["has_forms"]:
                analysis["document_type"] = "form"
            elif analysis["element_counts"].get("HeadingElement", 0) > 5:
                analysis["document_type"] = "structured_document"
            else:
                analysis["document_type"] = "text_document"
        
        except Exception as e:
            print(f"Warning: Could not analyze document structure: {e}", file=sys.stderr)
        
        return analysis


def main():
    """Main entry point for the HTML bridge"""
    parser = argparse.ArgumentParser(
        description="Convert PDF to HTML using Docling for CHONKER processing"
    )
    parser.add_argument("input_file", help="Input PDF file path")
    parser.add_argument("-o", "--output", help="Output HTML file path")
    parser.add_argument("--print-json", action="store_true", 
                       help="Print structured JSON to stdout (default)")
    parser.add_argument("--markdown", action="store_true",
                       help="Export as markdown instead of JSON")
    parser.add_argument("--metadata-only", action="store_true",
                       help="Print only document metadata as JSON")
    
    args = parser.parse_args()
    
    # Validate input file
    if not Path(args.input_file).exists():
        print(f"Error: Input file not found: {args.input_file}", file=sys.stderr)
        sys.exit(1)
    
    try:
        # Create bridge and convert
        bridge = DoclingHtmlBridge()
        
        if args.metadata_only:
            # Just extract and print metadata
            conversion_result = bridge.converter.convert(Path(args.input_file))
            document = conversion_result.document
            analysis = bridge.analyze_document_structure(document)
            
            metadata = {
                "source_file": args.input_file,
                "page_count": getattr(document, 'page_count', 0),
                "extraction_tool": "Docling HTML Bridge",
                "extraction_time": bridge.get_current_timestamp(),
                "structure_analysis": analysis
            }
            
            print(json.dumps(metadata, indent=2))
        else:
            if args.markdown:
                # Convert to markdown
                markdown_content = bridge.convert_to_markdown(args.input_file, args.output)
                if not args.output or args.print_json:
                    print(markdown_content)
            else:
                # Convert to structured JSON by default for better table handling
                json_content = bridge.convert_to_structured_json(args.input_file, args.output)
                if not args.output or args.print_json:
                    print(json_content)
    
    except Exception as e:
        print(f"Fatal error: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
