#!/usr/bin/env python3
"""
CHONKER Extraction Bridge
Wraps Magic-PDF and Docling for high-quality PDF extraction
"""

import json
import sys
import argparse
import tempfile
from pathlib import Path
from typing import Dict, List, Any, Optional
import traceback

def extract_with_magic_pdf(pdf_path: str, page_num: Optional[int] = None) -> Dict[str, Any]:
    """
    Extract content using Magic-PDF
    """
    try:
        # Import Magic-PDF (install with: pip install magic-pdf[full])
        from magic_pdf.pipe.UNIPipe import UNIPipe
        from magic_pdf.pipe.OCRPipe import OCRPipe
        from magic_pdf.pipe.TXTMode import TXTMode
        
        # Configure extraction pipeline
        pipe = UNIPipe()
        
        # Extract content
        with open(pdf_path, 'rb') as f:
            pdf_bytes = f.read()
        
        result = pipe.pipe_analyze(pdf_bytes)
        
        # Process results
        extractions = []
        for page_idx, page_data in enumerate(result.get('pages', [])):
            if page_num is not None and page_idx != page_num - 1:
                continue
                
            page_content = {
                'page_number': page_idx + 1,
                'text': page_data.get('text', ''),
                'tables': page_data.get('tables', []),
                'figures': page_data.get('figures', []),
                'formulas': page_data.get('formulas', []),
                'confidence': page_data.get('confidence', 0.8),
                'layout_boxes': page_data.get('layout_boxes', []),
                'tool': 'magic-pdf'
            }
            extractions.append(page_content)
        
        return {
            'success': True,
            'tool': 'magic-pdf',
            'extractions': extractions,
            'metadata': {
                'total_pages': len(result.get('pages', [])),
                'processing_time': result.get('processing_time', 0)
            }
        }
        
    except ImportError:
        return {
            'success': False,
            'error': 'Magic-PDF not installed. Run: pip install magic-pdf[full]',
            'tool': 'magic-pdf'
        }
    except Exception as e:
        return {
            'success': False,
            'error': str(e),
            'traceback': traceback.format_exc(),
            'tool': 'magic-pdf'
        }

def extract_with_docling(pdf_path: str, page_num: Optional[int] = None) -> Dict[str, Any]:
    """
    Extract content using Docling
    """
    try:
        # Import Docling (install with: pip install docling)
        from docling.document_converter import DocumentConverter
        from docling.datamodel.base_models import InputFormat
        from docling.datamodel.pipeline_options import PdfPipelineOptions
        
        # Configure pipeline
        pipeline_options = PdfPipelineOptions()
        pipeline_options.do_ocr = True
        pipeline_options.do_table_structure = True
        
        converter = DocumentConverter(
            format_options={
                InputFormat.PDF: pipeline_options
            }
        )
        
        # Extract content
        result = converter.convert(pdf_path)
        
        # Process results
        extractions = []
        for page_idx, page_data in enumerate(result.document.pages):
            if page_num is not None and page_idx != page_num - 1:
                continue
                
            page_content = {
                'page_number': page_idx + 1,
                'text': page_data.text,
                'tables': [table.to_dict() for table in page_data.tables],
                'figures': [fig.to_dict() for fig in page_data.figures],
                'formulas': [formula.to_dict() for formula in page_data.formulas],
                'confidence': getattr(page_data, 'confidence', 0.85),
                'layout_boxes': [box.to_dict() for box in page_data.layout_boxes],
                'tool': 'docling'
            }
            extractions.append(page_content)
        
        return {
            'success': True,
            'tool': 'docling',
            'extractions': extractions,
            'metadata': {
                'total_pages': len(result.document.pages),
                'processing_time': getattr(result, 'processing_time', 0)
            }
        }
        
    except ImportError:
        return {
            'success': False,
            'error': 'Docling not installed. Run: pip install docling',
            'tool': 'docling'
        }
    except Exception as e:
        return {
            'success': False,
            'error': str(e),
            'traceback': traceback.format_exc(),
            'tool': 'docling'
        }

def fallback_extraction(pdf_path: str, page_num: Optional[int] = None) -> Dict[str, Any]:
    """
    Fallback extraction using basic PDF text extraction
    """
    try:
        import PyPDF2
        
        extractions = []
        with open(pdf_path, 'rb') as f:
            reader = PyPDF2.PdfReader(f)
            
            pages_to_process = [page_num - 1] if page_num else range(len(reader.pages))
            
            for page_idx in pages_to_process:
                if page_idx < len(reader.pages):
                    page = reader.pages[page_idx]
                    text = page.extract_text()
                    
                    page_content = {
                        'page_number': page_idx + 1,
                        'text': text,
                        'tables': [],
                        'figures': [],
                        'formulas': [],
                        'confidence': 0.6,  # Lower confidence for fallback
                        'layout_boxes': [],
                        'tool': 'pypdf2-fallback'
                    }
                    extractions.append(page_content)
        
        return {
            'success': True,
            'tool': 'pypdf2-fallback',
            'extractions': extractions,
            'metadata': {
                'total_pages': len(reader.pages),
                'processing_time': 0
            }
        }
        
    except Exception as e:
        return {
            'success': False,
            'error': str(e),
            'traceback': traceback.format_exc(),
            'tool': 'pypdf2-fallback'
        }

def smart_extract(pdf_path: str, page_num: Optional[int] = None, preferred_tool: str = 'auto') -> Dict[str, Any]:
    """
    Smart extraction that tries different tools in order of preference
    """
    # Determine extraction strategy
    if preferred_tool == 'magic-pdf':
        tools = ['magic-pdf', 'docling', 'fallback']
    elif preferred_tool == 'docling':
        tools = ['docling', 'magic-pdf', 'fallback']
    else:  # auto
        # Check file size to determine best tool
        file_size = Path(pdf_path).stat().st_size
        if file_size > 50 * 1024 * 1024:  # 50MB+
            tools = ['docling', 'magic-pdf', 'fallback']  # Docling better for large files
        else:
            tools = ['magic-pdf', 'docling', 'fallback']  # Magic-PDF better for smaller files
    
    last_error = None
    
    for tool in tools:
        try:
            if tool == 'magic-pdf':
                result = extract_with_magic_pdf(pdf_path, page_num)
            elif tool == 'docling':
                result = extract_with_docling(pdf_path, page_num)
            else:  # fallback
                result = fallback_extraction(pdf_path, page_num)
            
            if result['success']:
                return result
            else:
                last_error = result
                
        except Exception as e:
            last_error = {
                'success': False,
                'error': str(e),
                'tool': tool
            }
    
    # If all tools failed, return the last error
    return last_error or {
        'success': False,
        'error': 'All extraction tools failed',
        'tool': 'unknown'
    }

def main():
    parser = argparse.ArgumentParser(description='CHONKER PDF Extraction Bridge')
    parser.add_argument('pdf_path', help='Path to PDF file')
    parser.add_argument('--page', type=int, help='Specific page number to extract (1-indexed)')
    parser.add_argument('--tool', choices=['auto', 'magic-pdf', 'docling'], default='auto',
                        help='Preferred extraction tool')
    parser.add_argument('--output', help='Output JSON file (default: stdout)')
    
    args = parser.parse_args()
    
    # Validate input
    pdf_path = Path(args.pdf_path)
    if not pdf_path.exists():
        result = {
            'success': False,
            'error': f'PDF file not found: {pdf_path}',
            'tool': 'none'
        }
    else:
        # Perform extraction
        result = smart_extract(str(pdf_path), args.page, args.tool)
    
    # Output results
    output_json = json.dumps(result, indent=2, ensure_ascii=False)
    
    if args.output:
        with open(args.output, 'w', encoding='utf-8') as f:
            f.write(output_json)
    else:
        print(output_json)

if __name__ == '__main__':
    main()
