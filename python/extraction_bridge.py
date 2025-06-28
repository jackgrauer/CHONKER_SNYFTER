#!/Library/Frameworks/Python.framework/Versions/3.12/bin/python3
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

# Docling v2 imports
try:
    from docling.document_converter import DocumentConverter
    from docling.datamodel.base_models import InputFormat
    from docling.datamodel.pipeline_options import PdfPipelineOptions
    from docling.document_converter import PdfFormatOption
    DOCLING_AVAILABLE = True
except ImportError:
    DOCLING_AVAILABLE = False

def extract_with_magic_pdf(pdf_path: str, page_num: Optional[int] = None) -> Dict[str, Any]:
    """
    Extract content using Magic-PDF via CLI interface (more reliable than direct API)
    """
    try:
        import subprocess
        import tempfile
        import os
        
        # Create a temporary output directory
        with tempfile.TemporaryDirectory() as temp_dir:
            output_dir = Path(temp_dir) / "magic_output"
            output_dir.mkdir(exist_ok=True)
            
            # Use magic-pdf CLI command (simplified)
            cmd = [
                "magic-pdf",
                "-p", str(pdf_path),
                "-o", str(output_dir),
                "-m", "auto"  # auto mode
            ]
            
            try:
                result = subprocess.run(
                    cmd, 
                    capture_output=True, 
                    text=True, 
                    timeout=60,  # 60 second timeout
                    check=False
                )
                
                # Check if CLI ran successfully
                if result.returncode != 0:
                    # CLI failed, try PyMuPDF-based extraction instead
                    return extract_with_pymupdf(pdf_path, page_num)
                
                # Try to find and parse output files
                text_files = list(output_dir.glob("**/*.txt"))
                json_files = list(output_dir.glob("**/*.json"))
                
                extractions = []
                
                if text_files:
                    # Process text output
                    for text_file in text_files:
                        with open(text_file, 'r', encoding='utf-8') as f:
                            text_content = f.read().strip()
                            
                        page_content = {
                            'page_number': 1,  # Magic-PDF CLI output doesn't always separate pages
                            'text': text_content,
                            'tables': [],
                            'figures': [],
                            'formulas': [],
                            'confidence': 0.8,
                            'layout_boxes': [],
                            'tool': 'magic-pdf'
                        }
                        extractions.append(page_content)
                        break  # Use first text file found
                
                if not extractions:
                    # No usable output, fall back to PyMuPDF
                    return extract_with_pymupdf(pdf_path, page_num)
                
                return {
                    'success': True,
                    'tool': 'magic-pdf',
                    'extractions': extractions,
                    'metadata': {
                        'total_pages': len(extractions),
                        'processing_time': 0
                    }
                }
                
            except subprocess.TimeoutExpired:
                return {
                    'success': False,
                    'error': 'Magic-PDF extraction timed out (60s)',
                    'tool': 'magic-pdf'
                }
            except subprocess.SubprocessError as e:
                return {
                    'success': False,
                    'error': f'Magic-PDF CLI error: {str(e)}',
                    'tool': 'magic-pdf'
                }
        
    except ImportError as e:
        return {
            'success': False,
            'error': f'Magic-PDF not properly installed: {str(e)}',
            'tool': 'magic-pdf'
        }
    except FileNotFoundError as e:
        if 'magic-pdf.json' in str(e):
            return {
                'success': False,
                'error': 'Magic-PDF configuration error - try updating the package',
                'tool': 'magic-pdf'
            }
        else:
            return {
                'success': False,
                'error': f'PDF file not found: {str(e)}',
                'tool': 'magic-pdf'
            }
    except Exception as e:
        return {
            'success': False,
            'error': f'Magic-PDF extraction failed: {str(e)}',
            'traceback': traceback.format_exc(),
            'tool': 'magic-pdf'
        }

def extract_with_pymupdf(pdf_path: str, page_num: Optional[int] = None) -> Dict[str, Any]:
    """
    Extract content using PyMuPDF (used as fallback for Magic-PDF)
    """
    try:
        import fitz  # PyMuPDF
        
        doc = fitz.open(pdf_path)
        extractions = []
        
        pages_to_process = [page_num - 1] if page_num else range(len(doc))
        
        for page_idx in pages_to_process:
            if page_idx < len(doc):
                page = doc[page_idx]
                text = page.get_text()
                
                page_content = {
                    'page_number': page_idx + 1,
                    'text': text,
                    'tables': [],
                    'figures': [],
                    'formulas': [],
                    'confidence': 0.7,  # PyMuPDF generally good for text
                    'layout_boxes': [],
                    'tool': 'pymupdf'
                }
                extractions.append(page_content)
        
        doc.close()
        
        return {
            'success': True,
            'tool': 'pymupdf',
            'extractions': extractions,
            'metadata': {
                'total_pages': len(doc),
                'processing_time': 0
            }
        }
        
    except ImportError:
        # Fall back to PyPDF2 if PyMuPDF not available
        return fallback_extraction(pdf_path, page_num)
    except Exception as e:
        return {
            'success': False,
            'error': f'PyMuPDF extraction failed: {str(e)}',
            'tool': 'pymupdf'
        }

def extract_with_docling(pdf_path: str, page_num: Optional[int] = None) -> Dict[str, Any]:
    """
    Extract content using Docling v2 API
    """
    if not DOCLING_AVAILABLE:
        return {
            'success': False,
            'error': 'Docling not installed. Run: pip install docling',
            'tool': 'docling'
        }
    
    try:
        # Configure pipeline options for Docling v2
        pipeline_options = PdfPipelineOptions()
        pipeline_options.do_ocr = True  # Enable OCR for scanned PDFs
        pipeline_options.do_table_structure = True  # Enable table recognition
        
        # Create converter with format-specific options
        converter = DocumentConverter(
            format_options={
                InputFormat.PDF: PdfFormatOption(pipeline_options=pipeline_options)
            }
        )
        
        # Convert the document
        if page_num is not None:
            # Docling v2 supports page limiting
            result = converter.convert(pdf_path, max_num_pages=1, start_page=page_num)
        else:
            result = converter.convert(pdf_path)
        
        # Export to markdown (Docling v2 feature)
        markdown_content = result.document.export_to_markdown()
        
        # Convert to our expected format
        extractions = []
        
        # Simple approach: treat the entire markdown as one extraction
        # In practice, you might want to parse pages separately
        page_count = len(result.document.pages) if hasattr(result.document, 'pages') else 1
        tables_found = len(result.document.tables) if hasattr(result.document, 'tables') else 0
        
        extraction = {
            'page_number': 1,
            'text': markdown_content,
            'tables': [],  # Tables are embedded in markdown
            'figures': [],  # Figures are embedded in markdown
            'formulas': [],  # Formulas are embedded in markdown
            'confidence': 0.95,  # Docling v2 is generally high confidence
            'layout_boxes': [],
            'tool': 'docling_v2'
        }
        extractions.append(extraction)
        
        return {
            'success': True,
            'tool': 'docling_v2',
            'extractions': extractions,
            'metadata': {
                'total_pages': page_count,
                'tables_found': tables_found,
                'processing_time': 0
            }
        }
        
    except Exception as e:
        return {
            'success': False,
            'error': f'Docling v2 extraction failed: {str(e)}',
            'traceback': traceback.format_exc(),
            'tool': 'docling_v2'
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
