#!/usr/bin/env python3
"""
Lean PDF Table Extractor
Extract tables from PDFs using docling with editable HTML output
"""

import os
import hashlib
from typing import Dict, List, Optional, Tuple, Any
from datetime import datetime
from html import escape
from dataclasses import dataclass, field
import warnings

# Suppress warnings
warnings.filterwarnings("ignore", message=".*pin_memory.*MPS.*")

# Try importing required libraries
try:
    from docling.document_converter import DocumentConverter
    from docling.datamodel.pipeline_options import PdfPipelineOptions
    from docling.datamodel.base_models import InputFormat
    DOCLING_AVAILABLE = True
except ImportError:
    DOCLING_AVAILABLE = False
    print("Error: Docling not available. Install with: pip install docling")

try:
    import torch
    TORCH_AVAILABLE = True
except ImportError:
    TORCH_AVAILABLE = False


@dataclass
class DocumentChunk:
    """Single chunk of processed document"""
    index: int
    type: str
    content: str
    page: Optional[int] = None
    metadata: Dict[str, Any] = field(default_factory=dict)


@dataclass
class ProcessingResult:
    """Result from document processing"""
    success: bool
    document_id: str
    chunks: List[DocumentChunk]
    html_content: str
    markdown_content: str
    processing_time: float
    error_message: Optional[str] = None


class PDFTableExtractor:
    """Extract tables from PDFs with HTML output"""
    
    def __init__(self, use_ocr: bool = False):
        self.use_ocr = use_ocr
        if not DOCLING_AVAILABLE:
            raise ImportError("Docling is required. Install with: pip install docling")
    
    def process_pdf(self, pdf_path: str) -> ProcessingResult:
        """Process PDF and extract tables"""
        start_time = datetime.now()
        
        try:
            # Validate PDF
            if not self._validate_pdf(pdf_path):
                raise ValueError("Invalid PDF file")
            
            # Convert document
            print(f"Processing {os.path.basename(pdf_path)}...")
            result = self._convert_document(pdf_path)
            
            # Extract content
            chunks, html_content = self._extract_content(result)
            
            # Build result
            processing_time = (datetime.now() - start_time).total_seconds()
            
            return ProcessingResult(
                success=True,
                document_id=self._generate_document_id(pdf_path),
                chunks=chunks,
                html_content=html_content,
                markdown_content=result.document.export_to_markdown(),
                processing_time=processing_time,
                error_message=None
            )
            
        except Exception as e:
            processing_time = (datetime.now() - start_time).total_seconds()
            return ProcessingResult(
                success=False,
                document_id="",
                chunks=[],
                html_content="",
                markdown_content="",
                processing_time=processing_time,
                error_message=str(e)
            )
    
    def _validate_pdf(self, pdf_path: str) -> bool:
        """Validate PDF file"""
        try:
            with open(pdf_path, 'rb') as f:
                header = f.read(1024)
                return header.startswith(b'%PDF-')
        except Exception:
            return False
    
    def _convert_document(self, pdf_path: str):
        """Convert PDF using docling"""
        if self.use_ocr and TORCH_AVAILABLE:
            # OCR configuration
            pipeline_options = PdfPipelineOptions()
            pipeline_options.do_ocr = True
            pipeline_options.ocr_options = {
                "use_gpu": torch.cuda.is_available() or torch.backends.mps.is_available()
            }
            
            converter = DocumentConverter(
                format_options={InputFormat.PDF: pipeline_options}
            )
        else:
            # Standard conversion
            converter = DocumentConverter()
        
        return converter.convert(pdf_path)
    
    def _extract_content(self, result) -> Tuple[List[DocumentChunk], str]:
        """Extract chunks and HTML from document"""
        chunks = []
        html_parts = ['<div id="document-content">']
        
        current_page = 0
        
        for idx, (item, level) in enumerate(result.document.iterate_items()):
            # Get page number
            item_page = getattr(item, 'page_number', 0) or getattr(item, 'page', 0) or 0
            
            # Add page break if needed
            if item_page > current_page and current_page > 0:
                html_parts.append(self._page_break_html(item_page))
            current_page = max(current_page, item_page)
            
            # Create chunk
            chunk = self._create_chunk(item, level, idx, item_page)
            chunks.append(chunk)
            
            # Add to HTML
            html_parts.append(self._item_to_html(item, level))
        
        html_parts.append('</div>')
        html_parts.append(self._get_table_editor_script())
        
        return chunks, '\n'.join(html_parts)
    
    def _create_chunk(self, item, level: int, index: int, page: int = 0) -> DocumentChunk:
        """Create a document chunk"""
        item_type = type(item).__name__
        content = getattr(item, 'text', str(item))
        
        return DocumentChunk(
            index=index,
            type=item_type.lower().replace('item', ''),
            content=content,
            page=page,
            metadata={'level': level}
        )
    
    def _item_to_html(self, item, level: int) -> str:
        """Convert document item to HTML"""
        item_type = type(item).__name__
        
        if item_type == 'SectionHeaderItem' and hasattr(item, 'text'):
            heading_level = min(level + 1, 3)
            safe_text = escape(str(item.text))
            return f'<h{heading_level}>{safe_text}</h{heading_level}>'
        
        elif item_type == 'TableItem':
            return self._table_to_html(item)
        
        elif item_type == 'TextItem' and hasattr(item, 'text'):
            safe_text = escape(str(item.text))
            return f'<p>{safe_text}</p>'
        
        elif item_type == 'ListItem' and hasattr(item, 'text'):
            safe_text = escape(str(item.text))
            return f'<li>{safe_text}</li>'
        
        return ''
    
    def _table_to_html(self, table_item) -> str:
        """Convert table to editable HTML"""
        html = ['<div class="table-container">']
        html.append('<table class="editable-table" border="1">')
        
        if hasattr(table_item, 'export_to_dataframe'):
            try:
                df = table_item.export_to_dataframe()
                
                # Headers
                html.append('<thead><tr>')
                for col in df.columns:
                    safe_col = escape(str(col))
                    html.append(f'<th contenteditable="true">{safe_col}</th>')
                html.append('</tr></thead>')
                
                # Data rows
                html.append('<tbody>')
                for _, row in df.iterrows():
                    html.append('<tr>')
                    for value in row:
                        safe_value = escape(str(value))
                        html.append(f'<td contenteditable="true">{safe_value}</td>')
                    html.append('</tr>')
                html.append('</tbody>')
                
            except Exception as e:
                html.append(f'<tr><td>Error extracting table: {e}</td></tr>')
        else:
            html.append('<tr><td>Table data not available</td></tr>')
        
        html.append('</table>')
        html.append('<div class="table-controls">')
        html.append('<button onclick="addRow(this)">+ Add Row</button>')
        html.append('<button onclick="addColumn(this)">+ Add Column</button>')
        html.append('<button onclick="deleteRow(this)">- Delete Row</button>')
        html.append('<button onclick="deleteColumn(this)">- Delete Column</button>')
        html.append('</div>')
        html.append('</div>')
        
        return '\n'.join(html)
    
    def _page_break_html(self, page_num: int) -> str:
        """Generate page break HTML"""
        return f'''
        <div class="page-break">
            <hr>
            <span>Page {page_num}</span>
        </div>
        '''
    
    def _get_table_editor_script(self) -> str:
        """JavaScript for table editing"""
        return '''
        <style>
        .table-container {
            margin: 20px 0;
            overflow-x: auto;
        }
        .editable-table {
            border-collapse: collapse;
            width: 100%;
            margin: 10px 0;
        }
        .editable-table th, .editable-table td {
            border: 1px solid #ddd;
            padding: 8px;
            text-align: left;
        }
        .editable-table th {
            background-color: #f4f4f4;
            font-weight: bold;
        }
        .table-controls {
            margin: 10px 0;
        }
        .table-controls button {
            margin-right: 10px;
            padding: 5px 10px;
            background: #4CAF50;
            color: white;
            border: none;
            border-radius: 4px;
            cursor: pointer;
        }
        .table-controls button:hover {
            background: #45a049;
        }
        .page-break {
            margin: 30px 0;
            text-align: center;
            color: #666;
        }
        .page-break hr {
            border: none;
            border-top: 2px dashed #ccc;
        }
        .page-break span {
            background: white;
            padding: 0 15px;
            position: relative;
            top: -12px;
        }
        </style>
        
        <script>
        function addRow(btn) {
            const table = btn.parentElement.previousElementSibling.querySelector('tbody');
            const row = table.insertRow(-1);
            const cellCount = table.rows[0].cells.length;
            for (let i = 0; i < cellCount; i++) {
                const cell = row.insertCell(i);
                cell.setAttribute('contenteditable', 'true');
                cell.innerHTML = '&nbsp;';
            }
        }
        
        function addColumn(btn) {
            const table = btn.parentElement.previousElementSibling;
            const thead = table.querySelector('thead');
            const tbody = table.querySelector('tbody');
            
            // Add header
            const th = document.createElement('th');
            th.setAttribute('contenteditable', 'true');
            th.innerHTML = 'New Column';
            thead.rows[0].appendChild(th);
            
            // Add cells
            Array.from(tbody.rows).forEach(row => {
                const cell = row.insertCell(-1);
                cell.setAttribute('contenteditable', 'true');
                cell.innerHTML = '&nbsp;';
            });
        }
        
        function deleteRow(btn) {
            const tbody = btn.parentElement.previousElementSibling.querySelector('tbody');
            if (tbody.rows.length > 1) {
                tbody.deleteRow(-1);
            }
        }
        
        function deleteColumn(btn) {
            const table = btn.parentElement.previousElementSibling;
            const thead = table.querySelector('thead');
            const tbody = table.querySelector('tbody');
            
            if (thead.rows[0].cells.length > 1) {
                // Remove header
                thead.rows[0].deleteCell(-1);
                
                // Remove cells
                Array.from(tbody.rows).forEach(row => {
                    row.deleteCell(-1);
                });
            }
        }
        </script>
        '''
    
    def _generate_document_id(self, pdf_path: str) -> str:
        """Generate unique document ID"""
        timestamp = datetime.now().isoformat()
        content = f"{pdf_path}_{timestamp}"
        return hashlib.sha256(content.encode()).hexdigest()[:16]


def extract_tables_from_pdf(pdf_path: str, use_ocr: bool = False) -> ProcessingResult:
    """Simple function to extract tables from a PDF"""
    extractor = PDFTableExtractor(use_ocr=use_ocr)
    return extractor.process_pdf(pdf_path)


def save_html_output(result: ProcessingResult, output_path: str):
    """Save the HTML output to a file"""
    html_template = f'''
    <!DOCTYPE html>
    <html>
    <head>
        <meta charset="utf-8">
        <title>PDF Table Extraction</title>
    </head>
    <body>
        <h1>PDF Table Extraction Results</h1>
        <p>Document ID: {result.document_id}</p>
        <p>Processing Time: {result.processing_time:.2f} seconds</p>
        <p>Tables Found: {sum(1 for chunk in result.chunks if chunk.type == 'table')}</p>
        <hr>
        {result.html_content}
    </body>
    </html>
    '''
    
    with open(output_path, 'w', encoding='utf-8') as f:
        f.write(html_template)


# Example usage
if __name__ == "__main__":
    import sys
    
    if len(sys.argv) < 2:
        print("Usage: python pdf_table_extractor.py <pdf_file> [--ocr]")
        sys.exit(1)
    
    pdf_file = sys.argv[1]
    use_ocr = "--ocr" in sys.argv
    
    if not os.path.exists(pdf_file):
        print(f"Error: File '{pdf_file}' not found")
        sys.exit(1)
    
    print(f"Extracting tables from: {pdf_file}")
    if use_ocr:
        print("OCR mode enabled")
    
    result = extract_tables_from_pdf(pdf_file, use_ocr)
    
    if result.success:
        print(f"\nExtraction completed in {result.processing_time:.2f} seconds")
        print(f"Found {len(result.chunks)} content chunks")
        print(f"Tables found: {sum(1 for chunk in result.chunks if chunk.type == 'table')}")
        
        # Save HTML output
        output_file = pdf_file.replace('.pdf', '_tables.html')
        save_html_output(result, output_file)
        print(f"\nHTML output saved to: {output_file}")
    else:
        print(f"\nExtraction failed: {result.error_message}")