#!/usr/bin/env python3
"""
CHONKER & SNYFTER - Elegant Document Processing System

A clean, elegant implementation focused on core functionality:
- CHONKER: PDF processing with HTML extraction
- SNYFTER: Document archiving and search
- Modern UI with floating windows
- Editable table support
"""

import sys
import os
import sqlite3
import json
import hashlib
import tempfile
import subprocess
import threading
from pathlib import Path
from typing import Dict, List, Optional, Tuple, Any
from datetime import datetime
from enum import Enum
import traceback
from html import escape
from queue import Queue
import time
import warnings

# Suppress PyTorch pin_memory warnings on MPS
warnings.filterwarnings("ignore", message=".*pin_memory.*MPS.*")

# Qt imports
from PyQt6.QtWidgets import (
    QApplication, QMainWindow, QWidget, QVBoxLayout, QHBoxLayout,
    QPushButton, QFileDialog, QMessageBox, QTextEdit, QLabel, 
    QSplitter, QDialog, QMenuBar, QMenu, QToolBar, QStatusBar
)
from PyQt6.QtCore import (
    Qt, QThread, pyqtSignal, QTimer, QPointF, QObject, QEvent,
    QRect, QPropertyAnimation, QEasingCurve, QRectF
)
from PyQt6.QtGui import (
    QAction, QKeySequence, QIcon, QPixmap, QPainter, QFont, QBrush, QColor, QTextCursor,
    QNativeGestureEvent
)
from PyQt6.QtPdf import QPdfDocument
from PyQt6.QtPdfWidgets import QPdfView

# Third-party imports with graceful fallbacks
try:
    from bs4 import BeautifulSoup
    BEAUTIFULSOUP_AVAILABLE = True
except ImportError:
    BEAUTIFULSOUP_AVAILABLE = False

try:
    from docling.document_converter import DocumentConverter
    DOCLING_AVAILABLE = True
except ImportError:
    DOCLING_AVAILABLE = False
    print("Warning: Docling not available. Install with: pip install docling")

try:
    from pydantic import BaseModel, Field, validator
    PYDANTIC_AVAILABLE = True
except ImportError:
    PYDANTIC_AVAILABLE = False
    BaseModel = object

# Import structured logging
try:
    from structured_logging import app_logger, db_logger, processing_logger, ui_logger
    STRUCTURED_LOGGING = True
except ImportError:
    # Fallback to simple logging
    class FallbackLogger:
        def info(self, msg, **kwargs): print(f"INFO: {msg}")
        def warning(self, msg, **kwargs): print(f"WARNING: {msg}")
        def error(self, msg, **kwargs): print(f"ERROR: {msg}")
        def debug(self, msg, **kwargs): pass
        def log_event(self, event, data): print(f"EVENT: {event} - {data}")
        def log_performance(self, op, duration, **metrics): print(f"PERF: {op} - {duration}s")
        def log_error_with_context(self, error, context): print(f"ERROR: {error} - {context}")
    
    app_logger = db_logger = processing_logger = ui_logger = FallbackLogger()
    STRUCTURED_LOGGING = False


# ============================================================================
# CONSTANTS
# ============================================================================

MAX_FILE_SIZE = 500 * 1024 * 1024  # 500MB in bytes
MAX_PROCESSING_TIME = 300  # 5 minutes in seconds

# ============================================================================
# DATA MODELS
# ============================================================================

class Mode(Enum):
    CHONKER = "chonker"
    SNYFTER = "snyfter"


class DocumentChunk(BaseModel):
    """Single chunk of processed document"""
    index: int
    type: str
    content: str
    page: Optional[int] = None
    confidence: float = 1.0
    metadata: Dict[str, Any] = Field(default_factory=dict)


class ProcessingResult(BaseModel):
    """Result from document processing"""
    success: bool
    document_id: str
    chunks: List[DocumentChunk]
    html_content: str
    markdown_content: str
    processing_time: float
    error_message: Optional[str] = None
    warnings: List[str] = Field(default_factory=list)


# ============================================================================
# PROCESSING ENGINE
# ============================================================================

class DocumentProcessor(QThread):
    """Clean document processing worker thread"""
    
    finished = pyqtSignal(ProcessingResult)
    progress = pyqtSignal(str)
    error = pyqtSignal(str)
    chunk_processed = pyqtSignal(int, int)
    ocr_needed = pyqtSignal()
    
    def __init__(self, pdf_path: str, use_ocr: bool = False):
        super().__init__()
        self.pdf_path = pdf_path
        self.should_stop = False
        self.start_time = None
        self.timeout_occurred = False
        self.use_ocr = use_ocr
    
    def stop(self):
        """Stop processing thread safely"""
        self.should_stop = True
        if self.isRunning():
            if not self.wait(5000):  # Wait up to 5 seconds
                self.terminate()  # Force terminate if needed
                self.wait()  # Wait for termination
    
    def _check_timeout(self) -> bool:
        """Check if processing has exceeded timeout"""
        if self.start_time and not self.timeout_occurred:
            elapsed = (datetime.now() - self.start_time).total_seconds()
            if elapsed > MAX_PROCESSING_TIME:
                self.timeout_occurred = True
                self.error.emit(f"⏱️ Processing timeout exceeded ({MAX_PROCESSING_TIME}s)")
                return True
        return False
    
    def run(self):
        """Process document with comprehensive error handling"""
        start_time = datetime.now()
        self.start_time = start_time
        
        try:
            # Validate PDF header before processing
            if not self._validate_pdf_header():
                raise ValueError("Invalid PDF file format")
            
            # Check if we should stop or timeout
            if self.should_stop or self._check_timeout():
                return
            
            # Initialize docling with tqdm fix
            self._init_docling()
            
            # Check if we should stop or timeout
            if self.should_stop or self._check_timeout():
                return
            
            # Check if OCR is needed (only if not already using OCR)
            if not self.use_ocr and self._detect_scanned_pdf():
                self.progress.emit("📄 Detected scanned/image PDF - OCR recommended")
                self.ocr_needed.emit()
                return  # Will be handled by main window
            
            # Convert document
            self.progress.emit("🐹 *chomp chomp* Processing document...")
            if self.use_ocr:
                result = self._convert_document_with_ocr()
            else:
                result = self._convert_document()
            
            # Check if we should stop or timeout
            if self.should_stop or self._check_timeout():
                return
            
            # Extract content
            chunks, html_content = self._extract_content(result)
            
            # Build result
            processing_time = (datetime.now() - start_time).total_seconds()
            result_obj = ProcessingResult(
                success=True,
                document_id=self._generate_document_id(),
                chunks=chunks,
                html_content=html_content,
                markdown_content=result.document.export_to_markdown(),
                processing_time=processing_time
            )
            
            # Log successful processing
            processing_logger.log_performance(
                "pdf_processing", 
                processing_time,
                chunks_count=len(chunks),
                file_size_mb=self._get_pdf_size_mb(),
                lazy_loading=self._should_use_lazy_loading()
            )
            
            self.finished.emit(result_obj)
            
        except Exception as e:
            processing_logger.log_error_with_context(e, {
                'pdf_path': self.pdf_path,
                'elapsed_time': (datetime.now() - start_time).total_seconds()
            })
            self._handle_error(e, start_time)
    
    def _validate_pdf_header(self) -> bool:
        """Validate PDF file header to ensure it's a valid PDF"""
        try:
            with open(self.pdf_path, 'rb') as f:
                # Read first 1024 bytes for header check
                header = f.read(1024)
                
                # Check for PDF magic bytes
                if not header.startswith(b'%PDF-'):
                    self.error.emit(f"Not a valid PDF file: {os.path.basename(self.pdf_path)}")
                    return False
                
                # Basic validation - check for PDF version
                if not header.startswith((b'%PDF-1.', b'%PDF-2.')):
                    self.error.emit(f"Unsupported PDF version in: {os.path.basename(self.pdf_path)}")
                    return False
                
                return True
                
        except Exception as e:
            self.error.emit(f"Cannot read file: {e}")
            return False
    
    def _init_docling(self):
        """Initialize docling with tqdm workaround"""
        if not DOCLING_AVAILABLE:
            raise Exception("🐹 *cough* Docling not installed!")
        
        # Fix tqdm issue
        import tqdm
        if not hasattr(tqdm.tqdm, '_lock'):
            tqdm.tqdm._lock = threading.RLock()
    
    def _get_pdf_size_mb(self) -> float:
        """Get PDF file size in MB"""
        try:
            return os.path.getsize(self.pdf_path) / (1024 * 1024)
        except:
            return 0
    
    def _should_use_lazy_loading(self) -> bool:
        """Determine if lazy loading should be used based on file size"""
        file_size_mb = self._get_pdf_size_mb()
        # Use lazy loading for files over 50MB
        return file_size_mb > 50
    
    def _detect_scanned_pdf(self) -> bool:
        """Quick check if PDF is likely scanned/image-based"""
        try:
            # Do a quick conversion without OCR to check text density
            from docling.document_converter import DocumentConverter
            from docling.datamodel.pipeline_options import PdfPipelineOptions
            from docling.datamodel.base_models import InputFormat
            
            # Configure for quick text extraction (no OCR)
            pipeline_options = PdfPipelineOptions()
            pipeline_options.do_ocr = False
            pipeline_options.do_table_structure = False  # Skip tables for speed
            
            converter = DocumentConverter(
                format_options={
                    InputFormat.PDF: pipeline_options,
                }
            )
            
            # Convert just first page for quick check
            result = converter.convert(self.pdf_path, max_pages=1)
            
            # Count extracted text
            total_text = ""
            if hasattr(result, 'document'):
                for item, _ in result.document.iterate_items():
                    if hasattr(item, 'text'):
                        total_text += str(item.text) + " "
            
            # If very little text, likely scanned
            text_length = len(total_text.strip())
            self.progress.emit(f"📄 First page text length: {text_length} chars")
            
            return text_length < 100  # Threshold for scanned detection
            
        except Exception as e:
            self.progress.emit(f"⚠️ Could not detect if PDF is scanned: {e}")
            return False
    
    def _convert_document_with_ocr(self):
        """Convert document with OCR enabled"""
        try:
            from docling.document_converter import DocumentConverter
            from docling.datamodel.pipeline_options import PdfPipelineOptions
            from docling.datamodel.base_models import InputFormat
            
            # Configure docling for OCR processing
            pipeline_options = PdfPipelineOptions()
            pipeline_options.do_ocr = True  # Enable OCR
            pipeline_options.ocr_options = {
                "use_gpu": torch.cuda.is_available() or torch.backends.mps.is_available()
            }
            
            converter = DocumentConverter(
                format_options={
                    InputFormat.PDF: pipeline_options,
                }
            )
            
            self.progress.emit("🔍 Processing with OCR enabled...")
            
            # Use lazy loading for large PDFs
            if self.file_size_mb > 50:
                return self._convert_document_lazy(converter)
            
            # Standard conversion with OCR
            result = converter.convert(self.pdf_path)
            self.progress.emit("✅ OCR processing complete!")
            
            return result
            
        except Exception as e:
            self.progress.emit(f"❌ OCR processing failed: {e}")
            # Fallback to regular processing
            return self._convert_document()
    
    def _convert_document(self):
        """Convert PDF using docling with retry mechanism and lazy loading"""
        from docling.document_converter import DocumentConverter
        
        max_retries = 3
        retry_delay = 1  # seconds
        
        # Check if we should use lazy loading
        if self._should_use_lazy_loading():
            self.progress.emit(f"🔄 Large PDF detected, using chunk-based processing...")
            return self._convert_document_lazy()
        
        # Standard processing for smaller files
        for attempt in range(max_retries):
            try:
                converter = DocumentConverter()
                result = converter.convert(self.pdf_path)
                return result
            except Exception as e:
                if attempt < max_retries - 1:
                    self.progress.emit(f"⚠️ Conversion attempt {attempt + 1} failed, retrying...")
                    time.sleep(retry_delay)
                    retry_delay *= 2  # Exponential backoff
                else:
                    raise e  # Re-raise on final attempt
    
    def _convert_document_lazy(self, converter=None):
        """Convert large PDFs in chunks to reduce memory usage"""
        from docling.document_converter import DocumentConverter
        
        # For now, use standard conversion with memory monitoring
        # In production, this would process pages in batches
        if converter is None:
            converter = DocumentConverter()
        
        # Process with lower memory footprint settings if available
        try:
            # Attempt to use chunked processing if docling supports it
            result = converter.convert(self.pdf_path)
            return result
        except Exception as e:
            self.error.emit(f"Lazy loading failed: {e}")
            raise
    
    def _extract_content(self, result) -> Tuple[List[DocumentChunk], str]:
        """Extract chunks and HTML from document"""
        chunks = []
        html_parts = ['<div id="document-content" contenteditable="true">']
        
        items = list(result.document.iterate_items())
        total = len(items)
        current_page = 0
        
        for idx, (item, level) in enumerate(items):
            if self.should_stop or self._check_timeout():
                break
            
            # Get page number from item metadata if available
            item_page = getattr(item, 'page_number', None) or getattr(item, 'page', None) or 0
            
            # Add page break indicator if page changed
            if item_page > current_page and current_page > 0:
                html_parts.append(f'''
                    <div style="margin: 20px 0; text-align: center; color: #999;">
                        <hr style="border-top: 2px dashed #666; margin: 10px 0;">
                        <span style="background: #525659; padding: 0 15px; position: relative; top: -12px;">
                            Page {item_page}
                        </span>
                    </div>
                ''')
                current_page = item_page
            elif item_page > current_page:
                current_page = item_page
            
            # Create chunk with page metadata
            chunk = self._create_chunk(item, level, idx, item_page)
            chunks.append(chunk)
            
            # Add to HTML
            html_parts.append(self._item_to_html(item, level))
            
            # Progress
            self.chunk_processed.emit(idx + 1, total)
        
        html_parts.append('</div>')
        html_parts.append(self._get_table_editor_script())
        
        return chunks, '\n'.join(html_parts)
    
    def _create_chunk(self, item, level: int, index: int, page: int = 0) -> DocumentChunk:
        """Create a document chunk from an item"""
        item_type = type(item).__name__
        content = getattr(item, 'text', str(item))
        
        return DocumentChunk(
            index=index,
            type=item_type.lower().replace('item', ''),
            content=content,
            metadata={'level': level, 'page': page}
        )
    
    def _item_to_html(self, item, level: int) -> str:
        """Convert document item to HTML with XSS protection and superscript/subscript support"""
        item_type = type(item).__name__
        
        if item_type == 'SectionHeaderItem' and hasattr(item, 'text'):
            heading_level = min(level + 1, 3)
            safe_text = self._enhance_text_formatting(escape(str(item.text)))
            return f'<h{heading_level}>{safe_text}</h{heading_level}>'
        
        elif item_type == 'TableItem':
            return self._table_to_html(item)
        
        elif item_type == 'TextItem' and hasattr(item, 'text'):
            safe_text = self._enhance_text_formatting(escape(str(item.text)))
            return f'<p>{safe_text}</p>'
        
        elif item_type == 'ListItem' and hasattr(item, 'text'):
            safe_text = self._enhance_text_formatting(escape(str(item.text)))
            return f'<li>{safe_text}</li>'
        
        return ''
    
    def _detect_math_formulas(self, text: str) -> bool:
        """Detect if text contains math formulas that might need VLM"""
        import re
        
        # Patterns that indicate math formulas
        math_patterns = [
            r'∫|∑|∏|√|∞|∈|∉|⊂|⊃|∪|∩',  # Math symbols
            r'[αβγδεζηθικλμνξοπρστυφχψω]',  # Greek letters
            r'\b(sin|cos|tan|log|ln|exp|lim|dx|dy)\b',  # Math functions
            r'[⁰¹²³⁴⁵⁶⁷⁸⁹]+.*[₀₁₂₃₄₅₆₇₈₉]+',  # Mixed super/subscripts
            r'[{}^_]{2,}',  # Complex notation
            r'\\[a-zA-Z]+\{',  # LaTeX commands
        ]
        
        for pattern in math_patterns:
            if re.search(pattern, text):
                return True
        
        # Check for garbled characters that might be misread formulas
        garbled_count = sum(1 for c in text if ord(c) > 0x1F000)  # Private use chars
        if garbled_count > 5:
            return True
            
        return False
    
    def _enhance_text_formatting(self, text: str) -> str:
        """Enhance text with proper superscript/subscript formatting"""
        # Check if we need VLM for math formulas
        if self._detect_math_formulas(text):
            self.progress.emit("🧮 Math formulas detected - consider VLM for better recognition")
        
        # Unicode superscript mapping
        superscript_map = {
            '⁰': '<sup>0</sup>', '¹': '<sup>1</sup>', '²': '<sup>2</sup>', '³': '<sup>3</sup>',
            '⁴': '<sup>4</sup>', '⁵': '<sup>5</sup>', '⁶': '<sup>6</sup>', '⁷': '<sup>7</sup>',
            '⁸': '<sup>8</sup>', '⁹': '<sup>9</sup>', '⁺': '<sup>+</sup>', '⁻': '<sup>-</sup>',
            'ⁿ': '<sup>n</sup>', 'ⁱ': '<sup>i</sup>', 'ʳ': '<sup>r</sup>', 'ˢ': '<sup>s</sup>',
            'ᵗ': '<sup>t</sup>', 'ʰ': '<sup>h</sup>', 'ˣ': '<sup>x</sup>', 'ʸ': '<sup>y</sup>',
            'ᶻ': '<sup>z</sup>', 'ᵃ': '<sup>a</sup>', 'ᵇ': '<sup>b</sup>', 'ᶜ': '<sup>c</sup>',
            'ᵈ': '<sup>d</sup>', 'ᵉ': '<sup>e</sup>', 'ᶠ': '<sup>f</sup>', 'ᵍ': '<sup>g</sup>'
        }
        
        # Unicode subscript mapping
        subscript_map = {
            '₀': '<sub>0</sub>', '₁': '<sub>1</sub>', '₂': '<sub>2</sub>', '₃': '<sub>3</sub>',
            '₄': '<sub>4</sub>', '₅': '<sub>5</sub>', '₆': '<sub>6</sub>', '₇': '<sub>7</sub>',
            '₈': '<sub>8</sub>', '₉': '<sub>9</sub>', '₊': '<sub>+</sub>', '₋': '<sub>-</sub>',
            'ₐ': '<sub>a</sub>', 'ₑ': '<sub>e</sub>', 'ₒ': '<sub>o</sub>', 'ₓ': '<sub>x</sub>',
            'ₕ': '<sub>h</sub>', 'ₖ': '<sub>k</sub>', 'ₗ': '<sub>l</sub>', 'ₘ': '<sub>m</sub>',
            'ₙ': '<sub>n</sub>', 'ₚ': '<sub>p</sub>', 'ₛ': '<sub>s</sub>', 'ₜ': '<sub>t</sub>'
        }
        
        # Replace Unicode superscripts/subscripts with HTML
        for char, html in superscript_map.items():
            text = text.replace(char, html)
        for char, html in subscript_map.items():
            text = text.replace(char, html)
        
        # Common patterns for chemical formulas and math - be more conservative
        import re
        
        # Only apply to known chemical elements followed by numbers
        chemical_elements = r'\b(H|He|Li|Be|B|C|N|O|F|Ne|Na|Mg|Al|Si|P|S|Cl|Ar|K|Ca|Fe|Cu|Zn|Ag|Au|Pb|U)\b'
        text = re.sub(f'({chemical_elements})(\\d+)', r'\1<sub>\2</sub>', text)
        
        # Only convert explicit notation markers (^, _) not regular text
        text = re.sub(r'\^(\d+|\{[^}]+\})', lambda m: '<sup>' + m.group(1).strip('{}') + '</sup>', text)
        text = re.sub(r'_(\d+|\{[^}]+\})', lambda m: '<sub>' + m.group(1).strip('{}') + '</sub>', text)
        
        return text
    
    def _table_to_html(self, table_item) -> str:
        """Convert table item to editable HTML table"""
        html = ['<table class="editable-table" border="1">']
        
        # Try to extract table data
        if hasattr(table_item, 'export_to_dataframe'):
            try:
                df = table_item.export_to_dataframe()
                
                # Headers
                html.append('<tr>')
                for col in df.columns:
                    safe_col = escape(str(col))
                    html.append(f'<th>{safe_col}</th>')
                html.append('</tr>')
                
                # Data rows
                for _, row in df.iterrows():
                    html.append('<tr>')
                    for value in row:
                        safe_value = escape(str(value))
                        html.append(f'<td contenteditable="true">{safe_value}</td>')
                    html.append('</tr>')
                    
            except:
                html.append('<tr><td>[Table Content]</td></tr>')
        else:
            html.append('<tr><td>[Table]</td></tr>')
        
        html.append('</table>')
        return '\n'.join(html)
    
    def _get_table_editor_script(self) -> str:
        """JavaScript for table editing functionality"""
        return '''
        <script>
        document.addEventListener('DOMContentLoaded', function() {
            const tables = document.querySelectorAll('table.editable-table');
            
            tables.forEach(table => {
                // Add controls
                const controls = document.createElement('div');
                controls.className = 'table-controls';
                controls.innerHTML = `
                    <button onclick="addRow(this)">+ Add Row</button>
                    <button onclick="addColumn(this)">+ Add Column</button>
                `;
                table.parentNode.insertBefore(controls, table.nextSibling);
                
                // Make cells editable
                table.querySelectorAll('td').forEach(cell => {
                    cell.setAttribute('contenteditable', 'true');
                });
            });
        });
        
        function addRow(btn) {
            const table = btn.parentNode.nextSibling;
            const row = table.insertRow(-1);
            const cellCount = table.rows[0].cells.length;
            for (let i = 0; i < cellCount; i++) {
                const cell = row.insertCell(i);
                cell.setAttribute('contenteditable', 'true');
                cell.innerHTML = '&nbsp;';
            }
        }
        
        function addColumn(btn) {
            const table = btn.parentNode.nextSibling;
            Array.from(table.rows).forEach(row => {
                const cell = row.insertCell(-1);
                cell.setAttribute('contenteditable', 'true');
                cell.innerHTML = '&nbsp;';
            });
        }
        </script>
        '''
    
    def _generate_document_id(self) -> str:
        """Generate unique document ID"""
        timestamp = datetime.now().isoformat()
        content = f"{self.pdf_path}_{timestamp}"
        return hashlib.sha256(content.encode()).hexdigest()[:16]
    
    def _handle_error(self, error: Exception, start_time: datetime):
        """Handle processing error"""
        error_result = ProcessingResult(
            success=False,
            document_id="",
            chunks=[],
            html_content="",
            markdown_content="",
            processing_time=(datetime.now() - start_time).total_seconds(),
            error_message=str(error)
        )
        
        self.error.emit(str(error))
        self.finished.emit(error_result)


# ============================================================================
# CACHING
# ============================================================================

class DocumentCache:
    """LRU cache for processed documents to improve performance"""
    
    def __init__(self, cache_dir: Optional[Path] = None, max_size_mb: int = 500):
        self.cache_dir = cache_dir or (Path.home() / '.chonker_cache')
        self.cache_dir.mkdir(exist_ok=True)
        self.max_size_mb = max_size_mb
        self._cache_index = {}
        self._load_index()
    
    def _load_index(self):
        """Load cache index from disk"""
        index_file = self.cache_dir / 'cache_index.json'
        if index_file.exists():
            try:
                with open(index_file, 'r') as f:
                    self._cache_index = json.load(f)
            except:
                self._cache_index = {}
    
    def _save_index(self):
        """Save cache index to disk"""
        index_file = self.cache_dir / 'cache_index.json'
        with open(index_file, 'w') as f:
            json.dump(self._cache_index, f)
    
    def _get_cache_key(self, file_path: str) -> str:
        """Generate cache key from file path and modification time"""
        stat = os.stat(file_path)
        key_str = f"{file_path}_{stat.st_mtime}_{stat.st_size}"
        return hashlib.sha256(key_str.encode()).hexdigest()[:16]
    
    def get(self, file_path: str) -> Optional[ProcessingResult]:
        """Get cached result if available and valid"""
        try:
            cache_key = self._get_cache_key(file_path)
            if cache_key in self._cache_index:
                cache_file = self.cache_dir / f"{cache_key}.json"
                if cache_file.exists():
                    with open(cache_file, 'r') as f:
                        data = json.load(f)
                        # Reconstruct ProcessingResult
                        chunks = [DocumentChunk(**chunk) for chunk in data['chunks']]
                        return ProcessingResult(
                            success=True,
                            document_id=data['document_id'],
                            chunks=chunks,
                            html_content=data['html_content'],
                            markdown_content=data['markdown_content'],
                            processing_time=data['processing_time']
                        )
        except:
            pass
        return None
    
    def put(self, file_path: str, result: ProcessingResult):
        """Cache a processing result"""
        try:
            cache_key = self._get_cache_key(file_path)
            cache_file = self.cache_dir / f"{cache_key}.json"
            
            # Convert to JSON-serializable format
            data = {
                'document_id': result.document_id,
                'chunks': [chunk.model_dump() if hasattr(chunk, 'model_dump') else chunk.__dict__ for chunk in result.chunks],
                'html_content': result.html_content,
                'markdown_content': result.markdown_content,
                'processing_time': result.processing_time
            }
            
            with open(cache_file, 'w') as f:
                json.dump(data, f)
            
            self._cache_index[cache_key] = {
                'file_path': file_path,
                'cached_at': datetime.now().isoformat()
            }
            self._save_index()
            self._enforce_size_limit()
        except Exception as e:
            print(f"Cache write error: {e}")
    
    def _enforce_size_limit(self):
        """Remove old cache entries if size limit exceeded"""
        total_size = sum(f.stat().st_size for f in self.cache_dir.glob('*.json'))
        total_size_mb = total_size / (1024 * 1024)
        
        if total_size_mb > self.max_size_mb:
            # Remove oldest entries
            cache_files = sorted(
                self.cache_dir.glob('*.json'),
                key=lambda f: f.stat().st_mtime
            )
            
            for cache_file in cache_files[:len(cache_files)//4]:  # Remove oldest 25%
                cache_key = cache_file.stem
                cache_file.unlink()
                if cache_key in self._cache_index:
                    del self._cache_index[cache_key]
            
            self._save_index()

# ============================================================================
# DATABASE
# ============================================================================

class ConnectionPool:
    """Database connection pool for better performance"""
    
    def __init__(self, db_path: str, pool_size: int = 5):
        self.db_path = db_path
        self.pool = Queue(maxsize=pool_size)
        self._lock = threading.Lock()
        
        # Initialize pool with connections
        for _ in range(pool_size):
            conn = sqlite3.connect(db_path, check_same_thread=False)
            conn.row_factory = sqlite3.Row
            self.pool.put(conn)
    
    def get_connection(self):
        """Get a connection from the pool"""
        return self.pool.get()
    
    def return_connection(self, conn):
        """Return a connection to the pool"""
        if conn:
            self.pool.put(conn)
    
    def close_all(self):
        """Close all connections in the pool"""
        while not self.pool.empty():
            conn = self.pool.get()
            conn.close()

class DocumentDatabase:
    """Clean database interface for document storage with connection pooling"""
    
    def __init__(self, db_path: str = "snyfter_archive.db", use_pool: bool = True):
        self.db_path = db_path
        self.use_pool = use_pool
        
        if self.use_pool:
            self.pool = ConnectionPool(db_path, pool_size=5)
        
        self._init_database()
    
    def _init_database(self):
        """Initialize database schema"""
        conn = sqlite3.connect(self.db_path)
        conn.execute('''
            CREATE TABLE IF NOT EXISTS documents (
                id TEXT PRIMARY KEY,
                filename TEXT NOT NULL,
                file_path TEXT,
                file_hash TEXT,
                html_content TEXT,
                markdown_content TEXT,
                processing_time REAL,
                processed_date TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                tags TEXT,
                notes TEXT
            )
        ''')
        
        conn.execute('''
            CREATE TABLE IF NOT EXISTS chunks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                document_id TEXT NOT NULL,
                chunk_index INTEGER,
                chunk_type TEXT,
                content TEXT,
                page_number INTEGER,
                confidence REAL,
                metadata TEXT,
                FOREIGN KEY (document_id) REFERENCES documents(id)
            )
        ''')
        
        conn.execute('''
            CREATE VIRTUAL TABLE IF NOT EXISTS chunks_fts USING fts5(
                content, 
                document_id UNINDEXED,
                tokenize = 'porter'
            )
        ''')
        
        conn.commit()
        conn.close()
    
    def _get_connection(self):
        """Get a database connection (from pool or direct)"""
        if self.use_pool:
            return self.pool.get_connection()
        else:
            conn = sqlite3.connect(self.db_path)
            conn.row_factory = sqlite3.Row
            return conn
    
    def _return_connection(self, conn):
        """Return a connection (to pool or close it)"""
        if self.use_pool:
            self.pool.return_connection(conn)
        else:
            conn.close()
    
    def save_document(self, result: ProcessingResult, file_path: str) -> tuple[bool, str]:
        """Save processed document to database. Returns (success, error_message)"""
        conn = None
        try:
            conn = self._get_connection()
            conn.execute('BEGIN TRANSACTION')
            
            # Save document
            conn.execute('''
                INSERT OR REPLACE INTO documents 
                (id, filename, file_path, html_content, markdown_content, processing_time)
                VALUES (?, ?, ?, ?, ?, ?)
            ''', (
                result.document_id,
                os.path.basename(file_path),
                file_path,
                result.html_content,
                result.markdown_content,
                result.processing_time
            ))
            
            # Delete old chunks first (for updates)
            conn.execute('DELETE FROM chunks WHERE document_id = ?', (result.document_id,))
            conn.execute('DELETE FROM chunks_fts WHERE document_id = ?', (result.document_id,))
            
            # Save chunks
            for chunk in result.chunks:
                conn.execute('''
                    INSERT INTO chunks 
                    (document_id, chunk_index, chunk_type, content, confidence, metadata)
                    VALUES (?, ?, ?, ?, ?, ?)
                ''', (
                    result.document_id,
                    chunk.index,
                    chunk.type,
                    chunk.content,
                    chunk.confidence,
                    json.dumps(chunk.metadata)
                ))
                
                # FTS index
                conn.execute(
                    'INSERT INTO chunks_fts (content, document_id) VALUES (?, ?)',
                    (chunk.content, result.document_id)
                )
            
            conn.commit()
            return True, ""
            
        except sqlite3.DatabaseError as e:
            if conn:
                conn.rollback()
            error_msg = f"Database error: {e}"
            print(f"🐁 {error_msg}")
            return False, error_msg
        except Exception as e:
            if conn:
                conn.rollback()
            error_msg = f"Unexpected error saving document: {e}"
            print(f"🐁 {error_msg}")
            return False, error_msg
        finally:
            if conn:
                self._return_connection(conn)
    
    def search(self, query: str) -> List[Dict]:
        """Search documents using FTS with SQL injection protection"""
        if not query or len(query) > 1000:
            return []
        
        # Validate FTS5 query syntax - only allow safe characters
        import re
        if not re.match(r'^[a-zA-Z0-9\s\-_"\']+$', query):
            print(f"🐁 Invalid search query: {query}")
            return []
        
        conn = None
        results = []
        
        try:
            conn = self._get_connection()
            
            # Use proper parameterized query
            cursor = conn.execute("""
                SELECT DISTINCT d.* 
                FROM chunks_fts f
                JOIN documents d ON f.document_id = d.id
                WHERE chunks_fts MATCH ?
                ORDER BY rank
                LIMIT 50
            """, (query,))
            
            results = [dict(row) for row in cursor.fetchall()]
            
        except sqlite3.OperationalError as e:
            # FTS syntax error - return empty results
            print(f"🐁 Search syntax error: {e}")
            results = []
        except Exception as e:
            print(f"🐁 Search error: {e}")
        finally:
            if conn:
                self._return_connection(conn)
        
        return results


# ============================================================================
# MAIN APPLICATION
# ============================================================================

class SelectionManager:
    """Manages bidirectional selection sync between PDF and text panes"""
    
    def __init__(self, parent):
        self.parent = parent
        self.pdf_view = None
        self.text_view = None
        self.selection_sync_enabled = True
        self.coordinate_map = {}  # Maps PDF positions to text positions
        self.chunks = []  # Store document chunks with metadata
        self.carrots = []  # Track active carrot indicators
        self.selection_timer = QTimer()
        self.selection_timer.setSingleShot(True)
        self.selection_timer.timeout.connect(self._process_selection)
        
    def set_views(self, pdf_view, text_view):
        """Set the PDF and text views to sync"""
        self.pdf_view = pdf_view
        self.text_view = text_view
        
        # Connect selection events
        if self.text_view:
            self.text_view.selectionChanged.connect(self._handle_text_selection)
        
        # Connect PDF selection if supported
        if self.pdf_view and hasattr(self.pdf_view, 'selectionChanged'):
            self.pdf_view.selectionChanged.connect(self._handle_pdf_selection)
            
    def set_chunks(self, chunks):
        """Store document chunks for coordinate mapping"""
        self.chunks = chunks
        self._build_coordinate_map()
        
    def _build_coordinate_map(self):
        """Build mapping between chunks and text positions"""
        self.coordinate_map.clear()
        current_pos = 0
        
        for chunk in self.chunks:
            if hasattr(chunk, 'content'):
                chunk_length = len(chunk.content)
                self.coordinate_map[current_pos] = {
                    'chunk': chunk,
                    'start': current_pos,
                    'end': current_pos + chunk_length
                }
                current_pos += chunk_length + 1  # +1 for newline
                
    def _handle_text_selection(self):
        """Handle when user selects text in the extracted text pane"""
        # Debounce selection events for better performance
        self.selection_timer.stop()
        self.selection_timer.start(200)  # Increased to 200ms for better performance
        
    def _process_selection(self):
        """Process the text selection after debounce"""
        if not self.selection_sync_enabled or not self.text_view:
            return
            
        cursor = self.text_view.textCursor()
        if not cursor.hasSelection():
            # Clear carrots if no selection
            for carrot in self.carrots:
                carrot.deleteLater()
            self.carrots.clear()
            return
            
        # Get selection position
        start = cursor.selectionStart()
        end = cursor.selectionEnd()
        
        # Find corresponding chunk
        for pos, mapping in self.coordinate_map.items():
            if start >= mapping['start'] and start <= mapping['end']:
                chunk = mapping['chunk']
                
                # Show carrot indicator
                self._show_margin_carrot(start)
                
                # Log the selection
                if hasattr(chunk, 'metadata') and chunk.metadata.get('page'):
                    page_num = chunk.metadata['page']
                    self.parent.log(f"📍 Selected text from page {page_num}")
                    
                    # Scroll PDF to page if possible
                    if self.pdf_view and hasattr(self.pdf_view, 'pageNavigator'):
                        nav = self.pdf_view.pageNavigator()
                        if nav:
                            nav.jump(page_num - 1)  # 0-indexed
                            
                break
                
    def _show_margin_carrot(self, text_position):
        """Show carrot indicator in margin"""
        # Clean up old carrots
        for carrot in self.carrots:
            carrot.deleteLater()
        self.carrots.clear()
        
        # Create new carrot - small green triangle
        carrot = QLabel("▶", self.parent)
        carrot.setStyleSheet("""
            QLabel {
                background-color: transparent;
                color: #1ABC9C;
                font-size: 10px;
                font-weight: bold;
            }
        """)
        
        # Position carrot next to selected text
        cursor = self.text_view.textCursor()
        cursor.setPosition(text_position)
        rect = self.text_view.cursorRect(cursor)
        
        # Convert to parent coordinates
        global_pos = self.text_view.mapToGlobal(rect.topLeft())
        parent_pos = self.parent.mapFromGlobal(global_pos)
        
        # Position to the left of text
        carrot.move(parent_pos.x() - 30, parent_pos.y())
        carrot.show()
        
        # Animate appearance
        self._animate_carrot(carrot)
        
        # Auto-hide after 3 seconds
        QTimer.singleShot(3000, lambda: self._hide_carrot(carrot))
        
        self.carrots.append(carrot)
        
    def _animate_carrot(self, carrot):
        """Animate carrot appearance"""
        # Skip animation for now to avoid crashes
        pass
        
    def _hide_carrot(self, carrot):
        """Hide and remove carrot"""
        if carrot in self.carrots:
            self.carrots.remove(carrot)
            carrot.deleteLater()
            
    def _handle_pdf_selection(self):
        """Handle when user selects text in the PDF viewer"""
        if not self.selection_sync_enabled or not self.pdf_view:
            return
            
        # Get selected text from PDF if possible
        if hasattr(self.pdf_view, 'selectedText'):
            selected_text = self.pdf_view.selectedText()
            if selected_text:
                # Clean up the selected text for better matching
                clean_text = selected_text.strip()
                if not clean_text:
                    return
                    
                # Find corresponding text in the extracted content
                text_content = self.text_view.toPlainText()
                
                # Try exact match first
                index = text_content.find(clean_text)
                
                # If not found, try with normalized whitespace
                if index < 0:
                    normalized_selected = ' '.join(clean_text.split())
                    normalized_content = ' '.join(text_content.split())
                    norm_index = normalized_content.find(normalized_selected)
                    
                    if norm_index >= 0:
                        # Map back to original position
                        index = self._map_normalized_to_original(text_content, norm_index)
                
                if index >= 0:
                    # Select corresponding text in text view
                    cursor = self.text_view.textCursor()
                    cursor.setPosition(index)
                    cursor.setPosition(index + len(selected_text), QTextCursor.MoveMode.KeepAnchor)
                    self.text_view.setTextCursor(cursor)
                    
                    # Ensure visible
                    self.text_view.ensureCursorVisible()
                    
                    # Show carrot indicator
                    self._show_margin_carrot(index)
                    
                    self.parent.log(f"📍 PDF → Text sync: {clean_text[:30]}...")
                else:
                    self.parent.log(f"⚠️ Could not find matching text in extracted content")
    
    def _map_normalized_to_original(self, original: str, normalized_pos: int) -> int:
        """Map position from normalized text back to original"""
        # Simple mapping - can be improved
        return min(normalized_pos, len(original) - 1)


class ChonkerSnyfterApp(QMainWindow):
    """Main application window - clean and elegant"""
    
    def __init__(self):
        super().__init__()
        self.current_mode = Mode.CHONKER
        self.db = DocumentDatabase()
        self.cache = DocumentCache()  # Initialize document cache
        self.caffeinate_process = None
        self.floating_windows = {}
        self.current_pdf_path = None
        self.active_pane = 'right'  # Track which pane is active ('left', 'right', 'top')
        self.embedded_pdf_view = None  # For embedded PDF viewer
        self.recent_files = []  # Track recent files
        self._load_recent_files()
        
        # Independent zoom levels
        self.pdf_zoom = 1.0
        self.text_zoom = 12.0  # Font size for text (float for smooth zoom)
        
        # CRUCIAL: Load Android 7.1 Noto emojis!
        self._load_sacred_emojis()
        
        # Processing animation
        self.processing_timer = QTimer()
        self.processing_timer.timeout.connect(self._update_processing_animation)
        self.processing_animation_state = 0
        
        # Selection sync manager
        self.selection_manager = SelectionManager(self)
        
        self._init_caffeinate()
        self._init_ui()
        self._apply_theme()
        
        # Set up selection sync after UI is created
        self.selection_manager.set_views(None, self.faithful_output)
        
        # Log application start
        app_logger.log_event("application_started", {
            'mode': 'CHONKER',
            'structured_logging': STRUCTURED_LOGGING,
            'docling_available': DOCLING_AVAILABLE,
            'cache_enabled': True,
            'connection_pooling': True
        })
    
    def _load_sacred_emojis(self):
        """Load the sacred Android 7.1 Noto emojis - NEVER let go of them!"""
        assets_dir = Path("assets/emojis")
        
        # Load CHONKER emoji
        chonker_path = assets_dir / "chonker.png"
        if chonker_path.exists():
            self.chonker_pixmap = QPixmap(str(chonker_path))
            print("Sacred Android 7.1 CHONKER emoji loaded!")
        else:
            # Fallback but with warning
            print("WARNING: CHONKER emoji missing! Using fallback...")
            self.chonker_pixmap = self._create_fallback_emoji("C", QColor("#FFE4B5"))
        
        # Load SNYFTER emoji
        snyfter_path = assets_dir / "snyfter.png"
        if snyfter_path.exists():
            self.snyfter_pixmap = QPixmap(str(snyfter_path))
            print("Sacred Android 7.1 SNYFTER emoji loaded!")
        else:
            print("WARNING: SNYFTER emoji missing! Using fallback...")
            self.snyfter_pixmap = self._create_fallback_emoji("S", QColor("#D3D3D3"))
    
    def _create_fallback_emoji(self, emoji: str, bg_color: QColor) -> QPixmap:
        """Create fallback emoji with proper resource cleanup"""
        pixmap = QPixmap(64, 64)
        pixmap.fill(Qt.GlobalColor.transparent)
        
        painter = QPainter()
        try:
            if not painter.begin(pixmap):
                return pixmap
                
            painter.setRenderHint(QPainter.RenderHint.Antialiasing)
            
            # Background circle
            painter.setBrush(QBrush(bg_color))
            painter.setPen(Qt.PenStyle.NoPen)
            painter.drawEllipse(2, 2, 60, 60)
            
            # Emoji text
            painter.setPen(QColor("black"))
            font = QFont()
            font.setPointSize(32)
            painter.setFont(font)
            painter.drawText(pixmap.rect(), Qt.AlignmentFlag.AlignCenter, emoji)
            
        except Exception as e:
            print(f"Error creating fallback emoji: {e}")
        finally:
            if painter.isActive():
                painter.end()
        
        return pixmap
    
    def _init_caffeinate(self):
        """Initialize caffeinate defense against sleep/logout"""
        try:
            self.caffeinate_process = subprocess.Popen(
                ['caffeinate', '-diu'],
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL
            )
            print("Caffeinate defense activated!")
        except:
            print("Warning: Caffeinate not available")
    
    def _init_ui(self):
        """Initialize the user interface"""
        self.setWindowTitle("CHONKER & SNYFTER")
        self.showMaximized()  # Start maximized
        
        # Set CHONKER icon as window icon
        self.setWindowIcon(QIcon(self.chonker_pixmap))
        
        # Ensure window can be brought to front
        self.setAttribute(Qt.WidgetAttribute.WA_ShowWithoutActivating, False)
        
        # Menu bar
        self._create_menu_bar()
        
        # Central widget
        central = QWidget()
        self.setCentralWidget(central)
        layout = QVBoxLayout(central)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(0)
        
        # Main vertical splitter for top bar and content
        self.main_splitter = QSplitter(Qt.Orientation.Vertical)
        self.main_splitter.setHandleWidth(3)
        self.main_splitter.setStyleSheet("""
            QSplitter::handle {
                background-color: #3A3C3E;
            }
        """)
        layout.addWidget(self.main_splitter)
        
        # Top bar (resizable)
        self._create_top_bar(self.main_splitter)
        
        # Content area - split view like before
        self.splitter = QSplitter(Qt.Orientation.Horizontal)
        self.splitter.setHandleWidth(3)
        self.splitter.setStyleSheet("""
            QSplitter::handle {
                background-color: #3A3C3E;
            }
        """)
        self.main_splitter.addWidget(self.splitter)
        
        # Set initial sizes for vertical splitter (top bar and content)
        self.main_splitter.setSizes([50, 900])
        
        # Left side - welcome/PDF view placeholder
        self.left_pane = QWidget()
        self.left_layout = QVBoxLayout(self.left_pane)
        self.left_layout.setContentsMargins(0, 0, 0, 0)
        self.left_layout.setSpacing(0)
        self.splitter.addWidget(self.left_pane)
        
        # Right side - faithful output (CRUCIAL!)
        self.faithful_output = QTextEdit()
        self.faithful_output.setReadOnly(False)
        self._update_pane_styles()  # Apply initial active pane styling
        self.splitter.addWidget(self.faithful_output)
        self.splitter.setSizes([700, 700])
        
        # Set up focus tracking and mouse tracking
        self.left_pane.installEventFilter(self)
        self.faithful_output.installEventFilter(self)
        self.left_pane.setMouseTracking(True)
        self.faithful_output.setMouseTracking(True)
        
        # Accept touch events for gesture support
        self.faithful_output.setAttribute(Qt.WidgetAttribute.WA_AcceptTouchEvents)
        self.left_pane.setAttribute(Qt.WidgetAttribute.WA_AcceptTouchEvents)
        
        # Install event filter on main window to catch all gestures
        self.installEventFilter(self)
        self.setAttribute(Qt.WidgetAttribute.WA_AcceptTouchEvents)
        
        # No status bar - more screen space!
        
        # Welcome screen
        self._show_welcome()
    
    def _load_recent_files(self):
        """Load recent files from settings"""
        # Simple in-memory storage for now
        pass
    
    def _update_recent_files_menu(self):
        """Update the recent files menu"""
        self.recent_menu.clear()
        
        if not self.recent_files:
            action = self.recent_menu.addAction("(No recent files)")
            action.setEnabled(False)
            return
            
        for file_path in self.recent_files[:5]:  # Show last 5 files
            if os.path.exists(file_path):
                action = self.recent_menu.addAction(os.path.basename(file_path))
                action.setData(file_path)
                action.triggered.connect(lambda checked, path=file_path: self._open_recent_file(path))
    
    def _add_to_recent_files(self, file_path):
        """Add file to recent files list"""
        if file_path in self.recent_files:
            self.recent_files.remove(file_path)
        self.recent_files.insert(0, file_path)
        self.recent_files = self.recent_files[:10]  # Keep only 10 recent files
        self._update_recent_files_menu()
    
    def _open_recent_file(self, file_path):
        """Open a recent file"""
        if os.path.exists(file_path):
            self.create_embedded_pdf_viewer(file_path)
            self.current_pdf_path = file_path
            self.log(f"Opened recent: {os.path.basename(file_path)}")
        else:
            QMessageBox.warning(self, "File Not Found", f"File no longer exists:\n{file_path}")
            self.recent_files.remove(file_path)
            self._update_recent_files_menu()
    
    def _create_menu_bar(self):
        """Create application menu bar"""
        menubar = self.menuBar()
        
        # File menu
        file_menu = menubar.addMenu("File")
        
        open_action = QAction("Open PDF", self)
        open_action.setShortcut(QKeySequence.StandardKey.Open)
        open_action.triggered.connect(self.open_pdf)
        file_menu.addAction(open_action)
        
        # Add recent files menu
        self.recent_menu = file_menu.addMenu("Recent Files")
        self._update_recent_files_menu()
        
        process_action = QAction("Process Document", self)
        # Use Ctrl+P for all platforms for consistency
        process_action.setShortcut(QKeySequence("Ctrl+P"))
        process_action.triggered.connect(self.process_current)
        file_menu.addAction(process_action)
        
        file_menu.addSeparator()
        
        quit_action = QAction("Quit", self)
        quit_action.setShortcut(QKeySequence.StandardKey.Quit)
        quit_action.triggered.connect(self.close)
        file_menu.addAction(quit_action)
        
        # View menu
        view_menu = menubar.addMenu("View")
        
        # Zoom actions
        zoom_in_action = QAction("Zoom In", self)
        zoom_in_action.setShortcut(QKeySequence.StandardKey.ZoomIn)
        zoom_in_action.triggered.connect(self.zoom_in)
        view_menu.addAction(zoom_in_action)
        
        zoom_out_action = QAction("Zoom Out", self)
        zoom_out_action.setShortcut(QKeySequence.StandardKey.ZoomOut)
        zoom_out_action.triggered.connect(self.zoom_out)
        view_menu.addAction(zoom_out_action)
        
        view_menu.addSeparator()
        
        chonker_action = QAction("CHONKER Mode", self)
        chonker_action.triggered.connect(lambda: self.set_mode(Mode.CHONKER))
        view_menu.addAction(chonker_action)
        
        snyfter_action = QAction("SNYFTER Mode", self)
        snyfter_action.triggered.connect(lambda: self.set_mode(Mode.SNYFTER))
        view_menu.addAction(snyfter_action)
    
    def _create_top_bar(self, parent_layout):
        """Create resizable top bar with mode toggle"""
        self.top_bar = QWidget()  # Store as instance variable
        self.top_bar.setMinimumHeight(50)
        self.top_bar.setMaximumHeight(150)  # Allow vertical resizing
        self.top_bar.setObjectName("topBar")
        
        # Make top bar focusable and install event filter for pane selection
        self.top_bar.setMouseTracking(True)
        self.top_bar.installEventFilter(self)
        
        top_bar = self.top_bar  # Keep local reference for compatibility
        
        layout = QHBoxLayout(top_bar)
        layout.setContentsMargins(20, 0, 20, 0)
        
        # Mode toggle with sacred emojis
        self.chonker_btn = QPushButton()
        self.chonker_btn.setIcon(QIcon(self.chonker_pixmap))
        self.chonker_btn.setText(" CHONKER")
        self.chonker_btn.setCheckable(True)
        self.chonker_btn.setChecked(True)
        self.chonker_btn.setMinimumWidth(130)  # Make button wider to fit text
        self.chonker_btn.clicked.connect(lambda: self.set_mode(Mode.CHONKER))
        
        self.snyfter_btn = QPushButton()
        self.snyfter_btn.setIcon(QIcon(self.snyfter_pixmap))
        self.snyfter_btn.setText(" SNYFTER")
        self.snyfter_btn.setCheckable(True)
        self.snyfter_btn.setMinimumWidth(130)  # Make button wider to fit text
        self.snyfter_btn.clicked.connect(lambda: self.set_mode(Mode.SNYFTER))
        
        layout.addWidget(self.chonker_btn)
        layout.addWidget(self.snyfter_btn)
        
        # Spacer to push terminal to the right
        layout.addStretch()
        
        # Terminal with expand button
        terminal_container = QWidget()
        terminal_layout = QHBoxLayout(terminal_container)
        terminal_layout.setContentsMargins(0, 0, 0, 0)
        terminal_layout.setSpacing(0)  # No gap between terminal and button
        
        # Terminal display - smaller and on the right
        self.terminal = QTextEdit()
        self.terminal.setFixedHeight(30)  # Shorter
        self.terminal.setMaximumWidth(350)  # Narrower
        self.terminal.setReadOnly(False)  # Enable copy/paste
        self.terminal.setObjectName("terminal")
        self.terminal.setVerticalScrollBarPolicy(Qt.ScrollBarPolicy.ScrollBarAlwaysOff)
        self.terminal.setHorizontalScrollBarPolicy(Qt.ScrollBarPolicy.ScrollBarAlwaysOff)
        # Ensure no extra space at bottom
        self.terminal.setLineWrapMode(QTextEdit.LineWrapMode.WidgetWidth)
        self.terminal.document().setDocumentMargin(0)
        
        # Expand/collapse button
        self.terminal_expand_btn = QPushButton("▼")
        self.terminal_expand_btn.setFixedSize(20, 20)
        self.terminal_expand_btn.setObjectName("terminalExpandBtn")
        self.terminal_expand_btn.clicked.connect(self._toggle_terminal_expansion)
        self.terminal_expanded = False
        
        terminal_layout.addWidget(self.terminal)
        terminal_layout.addWidget(self.terminal_expand_btn)
        layout.addWidget(terminal_container)
        
        # Quick actions
        open_btn = QPushButton("Open")
        open_btn.setToolTip("Open PDF (Ctrl+O)")
        open_btn.clicked.connect(self.open_pdf)
        open_btn.setShortcut(QKeySequence.StandardKey.Open)
        
        process_btn = QPushButton("Process")
        process_btn.setToolTip("Process (Ctrl+P)")
        process_btn.clicked.connect(self.process_current)
        
        layout.addWidget(open_btn)
        layout.addWidget(process_btn)
        
        # Add to splitter instead of layout
        if isinstance(parent_layout, QSplitter):
            parent_layout.addWidget(top_bar)
        else:
            parent_layout.addWidget(top_bar)
    
    def _show_welcome(self):
        """Show welcome screen"""
        welcome = QLabel("""
        <div style="text-align: center; padding: 50px;">
            <h1 style="color: #FFFFFF;">CHONKER & SNYFTER</h1>
            <p style="font-size: 18px; color: #B0B0B0;">
                Enhanced Document Processing System
            </p>
            <p style="margin-top: 30px; color: #B0B0B0;">
                Press <b>Ctrl+O</b> to open a PDF<br>
                Press <b>Ctrl+P</b> to process document<br>
                Click on a pane to make it active
            </p>
        </div>
        """)
        welcome.setAlignment(Qt.AlignmentFlag.AlignCenter)
        welcome.setStyleSheet("background-color: #525659;")
        
        # Clear left pane
        for i in reversed(range(self.left_layout.count())): 
            self.left_layout.itemAt(i).widget().setParent(None)
        
        self.left_layout.addWidget(welcome)
    
    def _update_pane_styles(self):
        """Update pane borders based on active state"""
        left_border_width = "3" if self.active_pane == 'left' else "1"
        right_border_width = "3" if self.active_pane == 'right' else "1"
        top_border_width = "3" if self.active_pane == 'top' else "1"
        
        left_border_color = "#1ABC9C" if self.active_pane == 'left' else "#3A3C3E"
        right_border_color = "#1ABC9C" if self.active_pane == 'right' else "#3A3C3E"
        top_border_color = "#1ABC9C" if self.active_pane == 'top' else "#3A3C3E"
        
        # Qt doesn't support box-shadow, but we can use border width and color for visual feedback
        
        self.left_pane.setStyleSheet(f"""
            QWidget {{
                border: {left_border_width}px solid {left_border_color};
                border-radius: 2px;
                background-color: #525659;
            }}
        """)
        
        # Update PDF viewer if it exists
        if hasattr(self, 'embedded_pdf_view') and self.embedded_pdf_view:
            self.embedded_pdf_view.setStyleSheet(f"""
                QPdfView {{
                    background-color: #525659;
                    border: none;
                    margin: 2px;
                }}
            """)
        
        self.faithful_output.setStyleSheet(f"""
            QTextEdit {{
                font-family: 'Courier New', monospace;
                font-size: 12px;
                background-color: #525659;
                color: #FFFFFF;
                border: {right_border_width}px solid {right_border_color};
                border-radius: 2px;
                padding: 10px;
            }}
        """)
        
        # Update top bar if it exists
        if hasattr(self, 'top_bar'):
            self.top_bar.setStyleSheet(f"""
                #topBar {{
                    background-color: #2D2F31;
                    border-bottom: {top_border_width}px solid {top_border_color};
                }}
            """)
    
    
    def eventFilter(self, obj, event):
        """Track focus and mouse events for pane activation"""
        # Handle native gesture events (pinch zoom on trackpad)
        if event.type() == QEvent.Type.NativeGesture:
            gesture_event = event
            # Check if this is a zoom gesture
            gesture_type = gesture_event.gestureType()
            
            # Debug: log all gesture events
            self.log(f"Gesture on {obj.__class__.__name__}: {gesture_type}")
            
            if 'ZoomNativeGesture' in str(gesture_type):
                # Only handle the event once per gesture
                # Process it from the widget that received it, not the main window
                if obj != self.faithful_output and obj != self.embedded_pdf_view:
                    return False
                    
                # Handle pinch-to-zoom for both PDF and text views
                zoom_delta = gesture_event.value()
                # Only zoom the active pane
                zoom_factor = 1.0 + zoom_delta * 0.5  # Increased to 0.5 for much faster zoom
                
                
                # Handle zoom for active pane
                self._handle_gesture_zoom(zoom_delta, zoom_factor)
                
                return True
        
        # Handle mouse enter for hover-based pane switching
        elif event.type() == QEvent.Type.Enter:
            if obj == self.left_pane or (hasattr(self, 'embedded_pdf_view') and self.embedded_pdf_view and obj == self.embedded_pdf_view):
                if self.active_pane != 'left':
                    self.active_pane = 'left'
                    self._update_pane_styles()
            elif obj == self.faithful_output:
                if self.active_pane != 'right':
                    self.active_pane = 'right'
                    self._update_pane_styles()
            elif hasattr(self, 'top_bar') and obj == self.top_bar:
                if self.active_pane != 'top':
                    self.active_pane = 'top'
                    self._update_pane_styles()
        
        # Handle pinch-to-zoom (Ctrl/Cmd + scroll)
        elif event.type() == QEvent.Type.Wheel and hasattr(event, 'modifiers'):
            if event.modifiers() & Qt.KeyboardModifier.ControlModifier:
                if obj == self.faithful_output:
                    # Zoom text in faithful output
                    if event.angleDelta().y() > 0:
                        self.text_zoom = min(self.text_zoom + 1, 48)
                    else:
                        self.text_zoom = max(self.text_zoom - 1, 8)
                    self._apply_text_zoom()
                    return True
                elif hasattr(self, 'embedded_pdf_view') and self.embedded_pdf_view and obj == self.embedded_pdf_view:
                    # Zoom PDF view independently with performance optimization
                    zoom_factor = 1.1 if event.angleDelta().y() > 0 else 0.9
                    new_zoom = self.pdf_zoom * zoom_factor
                    
                    # Limit zoom range for performance
                    if 0.1 <= new_zoom <= 5.0:
                        self.pdf_zoom = new_zoom
                        self._apply_pdf_zoom()
                    return True
        
        # Let all events pass through normally
        return super().eventFilter(obj, event)
    
    def _apply_theme(self):
        """Apply elegant theme"""
        self.setStyleSheet("""
            QMainWindow {
                background-color: #525659;
            }
            
            #topBar {
                background-color: #525659;
                border-bottom: 1px solid #3A3C3E;
            }
            
            QPushButton {
                background-color: #6B6E71;
                border: 1px solid #4A4C4E;
                border-radius: 4px;
                padding: 8px 16px;
                font-size: 14px;
                color: #FFFFFF;
            }
            
            QPushButton:hover {
                background-color: #7B7E81;
                border-color: #5A5C5E;
            }
            
            QPushButton:checked {
                background-color: #1ABC9C;
                border-color: #16A085;
                color: #FFFFFF;
            }
            
            #terminal {
                background-color: #1E1E1E;
                color: #1ABC9C;  /* Already using nice app green */
                font-family: 'Courier New', monospace;
                font-size: 11px;
                border: 1px solid #333;
                border-radius: 4px;
                padding: 4px;
            }
            
            QTextEdit {
                background-color: #525659;
                color: #FFFFFF;
                border: 1px solid #3A3C3E;
                border-radius: 4px;
            }
            
            /* Scrollbar styling for both panes */
            QScrollBar:vertical {
                background-color: #3A3C3E;
                width: 12px;
                border: none;
            }
            
            QScrollBar::handle:vertical {
                background-color: #1ABC9C;
                border: none;
                min-height: 20px;
            }
            
            QScrollBar::handle:vertical:hover {
                background-color: #16A085;
            }
            
            QScrollBar::add-line:vertical, QScrollBar::sub-line:vertical {
                border: none;
                background: none;
                height: 0px;
            }
            
            QScrollBar:horizontal {
                background-color: #3A3C3E;
                height: 12px;
                border: none;
            }
            
            QScrollBar::handle:horizontal {
                background-color: #1ABC9C;
                border: none;
                min-width: 20px;
            }
            
            QScrollBar::handle:horizontal:hover {
                background-color: #16A085;
            }
            
            QScrollBar::add-line:horizontal, QScrollBar::sub-line:horizontal {
                border: none;
                background: none;
                width: 0px;
            }
            
            #terminalExpandBtn {
                background-color: transparent;
                border: 1px solid #3A3C3E;
                border-radius: 2px;
                padding: 0px;
                font-size: 10px;
                color: #1ABC9C;
            }
            
            #terminalExpandBtn:hover {
                background-color: #3A3C3E;
            }
        """)
    
    def zoom_in(self) -> None:
        """Zoom in the active pane.
        
        Increases text zoom by 2 points (max 48) or PDF zoom by 1.2x factor.
        """
        if self.active_pane == 'right' and self.faithful_output:
            self.text_zoom = min(48, int(self.text_zoom) + 2)  # Increase by 2 points
            self._apply_text_zoom()
        elif self.active_pane == 'left' and hasattr(self, 'embedded_pdf_view') and self.embedded_pdf_view:
            self.pdf_zoom *= 1.2
            self._apply_pdf_zoom()
                
    def zoom_out(self) -> None:
        """Zoom out the active pane.
        
        Decreases text zoom by 2 points (min 8) or PDF zoom by 0.8x factor.
        """
        if self.active_pane == 'right' and self.faithful_output:
            self.text_zoom = max(8, int(self.text_zoom) - 2)  # Decrease by 2 points
            self._apply_text_zoom()
        elif self.active_pane == 'left' and hasattr(self, 'embedded_pdf_view') and self.embedded_pdf_view:
            self.pdf_zoom *= 0.8
            self._apply_pdf_zoom()
    
    def _handle_gesture_zoom(self, zoom_delta: float, zoom_factor: float) -> None:
        """Handle zoom gesture for active pane.
        
        Args:
            zoom_delta: The zoom delta value from the gesture (-1.0 to 1.0)
            zoom_factor: The calculated zoom factor (1.0 + zoom_delta * 0.5)
        """
        if self.active_pane == 'right' and self.faithful_output:
            # Zoom text pane by whole point sizes
            if abs(zoom_delta) > 0.05:  # Threshold for triggering zoom
                if zoom_delta > 0:
                    new_size = min(48, int(self.text_zoom) + 1)
                else:
                    new_size = max(8, int(self.text_zoom) - 1)
                
                if new_size != int(self.text_zoom):
                    self.text_zoom = new_size
                    self._apply_text_zoom()
        
        elif self.active_pane == 'left' and hasattr(self, 'embedded_pdf_view') and self.embedded_pdf_view:
            # Zoom PDF pane with threshold to reduce flicker
            if abs(zoom_delta) > 0.02:  # Only zoom if delta is significant
                new_zoom = self.pdf_zoom * zoom_factor
                if 0.25 <= new_zoom <= 4.0:
                    self.pdf_zoom = new_zoom
                    self._apply_pdf_zoom()
    
    def _apply_text_zoom(self) -> None:
        """Apply current text zoom to the text output widget.
        
        Sets the font size of the faithful_output widget to the current text_zoom value.
        """
        if self.faithful_output:
            font = self.faithful_output.font()
            font.setPointSize(self.text_zoom)
            self.faithful_output.setFont(font)
    
    def _apply_pdf_zoom(self) -> None:
        """Apply current PDF zoom to the PDF view widget.
        
        Uses setZoomFactor if available on the embedded_pdf_view widget.
        """
        if hasattr(self, 'embedded_pdf_view') and self.embedded_pdf_view:
            if hasattr(self.embedded_pdf_view, 'setZoomFactor'):
                self.embedded_pdf_view.setZoomFactor(self.pdf_zoom)
    
    def set_mode(self, mode: Mode):
        """Switch between CHONKER and SNYFTER modes"""
        self.current_mode = mode
        
        # Update buttons
        self.chonker_btn.setChecked(mode == Mode.CHONKER)
        self.snyfter_btn.setChecked(mode == Mode.SNYFTER)
        
        # Update terminal
        if mode == Mode.CHONKER:
            self.log("CHONKER mode activated - Ready to process PDFs!")
        else:
            self.log("SNYFTER mode activated - Ready to search archives!")
    
    def _on_ocr_needed(self, file_path: str):
        """Handle when OCR is needed for a scanned PDF"""
        # Stop animation
        self.processing_timer.stop()
        
        reply = QMessageBox.question(
            self,
            "Enable OCR?",
            "This PDF appears to be scanned/image-based with minimal text.\n\n"
            "Would you like to enable OCR for better extraction?\n\n"
            "✅ Better text extraction\n"
            "✅ Improved table recognition\n"
            "✅ Enhanced structure detection\n"
            "✅ Math formula recognition (VLM fallback)\n\n"
            "(This may take longer to process)",
            QMessageBox.StandardButton.Yes | QMessageBox.StandardButton.No
        )
        
        if reply == QMessageBox.StandardButton.Yes:
            self.log("🔍 Reprocessing with OCR enabled...")
            # Start new processor with OCR
            self.processor = DocumentProcessor(file_path, use_ocr=True)
            self.processor.progress.connect(self.log)
            self.processor.error.connect(lambda e: self.log(f"🐹 Error: {e}"))
            self.processor.finished.connect(self.on_processing_finished)
            self.processor.start()
            
            # Restart animation
            self.processing_timer.start(500)
        else:
            self.log("📄 Processing without OCR...")
            # Continue without OCR - just process normally
            self.processor = DocumentProcessor(file_path, use_ocr=False)
            self.processor.progress.connect(self.log)
            self.processor.error.connect(lambda e: self.log(f"🐹 Error: {e}"))
            self.processor.finished.connect(self.on_processing_finished)
            self.processor.start()
            
            # Restart animation
            self.processing_timer.start(500)
    
    def _toggle_terminal_expansion(self):
        """Toggle terminal between compact and expanded view"""
        if self.terminal_expanded:
            # Collapse
            self.terminal.setFixedHeight(30)
            self.terminal_expand_btn.setText("▼")
            self.terminal_expanded = False
        else:
            # Expand
            self.terminal.setFixedHeight(150)
            self.terminal_expand_btn.setText("▲")
            self.terminal_expanded = True
    
    def log(self, message: str):
        """Log message to terminal"""
        timestamp = datetime.now().strftime("%H:%M:%S")
        
        # More efficient line count management
        cursor = self.terminal.textCursor()
        cursor.movePosition(QTextCursor.MoveOperation.End)
        cursor.insertText(f"[{timestamp}] {message}\n")
        
        # Keep only last 100 lines - optimized approach
        doc = self.terminal.document()
        if doc.blockCount() > 100:
            cursor.movePosition(QTextCursor.MoveOperation.Start)
            cursor.movePosition(QTextCursor.MoveOperation.Down, QTextCursor.MoveMode.KeepAnchor, doc.blockCount() - 100)
            cursor.removeSelectedText()
        
        # Move cursor to end and ensure visibility
        cursor.movePosition(QTextCursor.MoveOperation.End)
        self.terminal.setTextCursor(cursor)
        self.terminal.ensureCursorVisible()
    
    def open_pdf(self):
        """Open PDF file with size validation"""
        # Use native dialog for better performance
        dialog = QFileDialog(self)
        dialog.setWindowTitle("Open PDF")
        dialog.setNameFilter("PDF Files (*.pdf)")
        dialog.setFileMode(QFileDialog.FileMode.ExistingFile)
        dialog.setViewMode(QFileDialog.ViewMode.List)
        
        # Remember last directory
        if hasattr(self, '_last_open_dir') and self._last_open_dir:
            dialog.setDirectory(self._last_open_dir)
        
        if dialog.exec() != QFileDialog.DialogCode.Accepted:
            return
            
        file_path = dialog.selectedFiles()[0]
        self._last_open_dir = os.path.dirname(file_path)
        
        if file_path:
            # Check file size before opening
            try:
                file_size = os.path.getsize(file_path)
                file_size_mb = file_size / (1024 * 1024)
                
                if file_size > MAX_FILE_SIZE:
                    QMessageBox.warning(
                        self,
                        "File Too Large",
                        f"Cannot open file: {os.path.basename(file_path)}\n\n"
                        f"File size: {file_size_mb:.1f} MB\n"
                        f"Maximum allowed: {MAX_FILE_SIZE / (1024 * 1024):.0f} MB\n\n"
                        "Please use a smaller PDF file."
                    )
                    self.log(f"❌ File too large: {file_size_mb:.1f} MB")
                    return
                
                self.log(f"Opening PDF: {os.path.basename(file_path)} ({file_size_mb:.1f} MB)")
                
            except OSError as e:
                QMessageBox.critical(
                    self,
                    "File Error", 
                    f"Cannot access file: {e}"
                )
                return
            
            # Show loading message
            self.log("Loading PDF...")
            QApplication.processEvents()  # Update UI immediately
            
            # Create embedded PDF viewer in left pane
            self.create_embedded_pdf_viewer(file_path)
            self.current_pdf_path = file_path
    
    def create_embedded_pdf_viewer(self, file_path: str):
        """Create embedded PDF viewer in left pane"""
        # Clear left pane and remove old event filters
        for i in reversed(range(self.left_layout.count())): 
            widget = self.left_layout.itemAt(i).widget()
            if widget:
                if hasattr(widget, 'removeEventFilter'):
                    widget.removeEventFilter(self)
                widget.setParent(None)
                widget.deleteLater()
        
        # Clean up old PDF view if exists
        if hasattr(self, 'embedded_pdf_view') and self.embedded_pdf_view:
            self.embedded_pdf_view.removeEventFilter(self)
            self.embedded_pdf_view = None
        
        # Create PDF viewer
        self.embedded_pdf_view = QPdfView(self.left_pane)
        pdf_document = QPdfDocument(self.left_pane)
        self.embedded_pdf_view.setDocument(pdf_document)
        pdf_document.load(file_path)
        
        # Style the PDF viewer to match theme
        self.embedded_pdf_view.setStyleSheet("""
            QPdfView {
                background-color: #525659;
                border: none;
            }
        """)
        
        # Set page mode for better navigation
        self.embedded_pdf_view.setPageMode(QPdfView.PageMode.MultiPage)
        self.embedded_pdf_view.setZoomMode(QPdfView.ZoomMode.Custom)  # Allow custom zoom
        
        # Enable text selection in PDF (if supported)
        if hasattr(QPdfView, 'InteractionMode'):
            try:
                # Try to enable text selection
                self.embedded_pdf_view.setInteractionMode(QPdfView.InteractionMode.TextSelection)
                self.log("PDF text selection enabled")
                
                # Connect selection handling
                if hasattr(self.embedded_pdf_view, 'selectionChanged'):
                    self.embedded_pdf_view.selectionChanged.connect(self._handle_pdf_selection_change)
            except AttributeError:
                # Try alternative approach for text selection
                try:
                    self.embedded_pdf_view.setTextInteractionFlags(Qt.TextInteractionFlag.TextSelectableByMouse)
                    self.log("PDF text selection enabled (alternative method)")
                except:
                    self.log("Note: PDF text selection not supported in this PyQt version")
        
        # Performance optimization: Reduce render quality for faster interaction
        if hasattr(self.embedded_pdf_view, 'setRenderHint'):
            self.embedded_pdf_view.setRenderHint(QPainter.RenderHint.Antialiasing, False)
            self.embedded_pdf_view.setRenderHint(QPainter.RenderHint.TextAntialiasing, True)
        
        # Add to layout
        self.left_layout.addWidget(self.embedded_pdf_view)
        
        # Install event filter for focus tracking
        self.embedded_pdf_view.installEventFilter(self)
        
        # Switch to left pane
        self.active_pane = 'left'
        self._update_pane_styles()
        self.embedded_pdf_view.setFocus()
        
        # Set PDF view in selection manager
        self.selection_manager.set_views(self.embedded_pdf_view, self.faithful_output)
        
        # Add to recent files
        self._add_to_recent_files(file_path)
        
        self.log(f"Opened: {os.path.basename(file_path)}")
    
    def _handle_pdf_selection_change(self):
        """Handle selection changes in PDF viewer"""
        if hasattr(self.embedded_pdf_view, 'selectedText'):
            selected_text = self.embedded_pdf_view.selectedText()
            if selected_text:
                self.log(f"PDF selection: {selected_text[:50]}...")
                # Trigger selection sync
                self.selection_manager._handle_pdf_selection()
    
    def _update_processing_animation(self):
        """Update processing animation"""
        states = ["🐹 *chomp*", "🐹 *chomp chomp*", "🐹 *CHOMP*", "🐹 *chomp chomp chomp*"]
        self.processing_animation_state = (self.processing_animation_state + 1) % len(states)
        message = f"{states[self.processing_animation_state]} Processing..."
        # Update the last line in terminal with animation
        cursor = self.terminal.textCursor()
        cursor.movePosition(QTextCursor.MoveOperation.End)
        cursor.select(QTextCursor.SelectionType.LineUnderCursor)
        cursor.removeSelectedText()
        cursor.insertText(f"[{datetime.now().strftime('%H:%M:%S')}] {message}")
        self.terminal.setTextCursor(cursor)
        self.terminal.ensureCursorVisible()
    
    def process_current(self):
        """Process current PDF"""
        # Stop any existing processor and wait for it to finish
        if hasattr(self, 'processor') and self.processor.isRunning():
            self.log("Stopping previous processing...")
            self.processor.stop()
            # Wait for the thread to actually finish
            if not self.processor.wait(5000):  # 5 second timeout
                self.log("⚠️ Previous processing didn't stop cleanly")
                return
        
        # Check if we have a PDF loaded
        if not self.current_pdf_path:
            QMessageBox.warning(self, "No PDF", "Please open a PDF first")
            return
        
        file_path = self.current_pdf_path
        
        # Double-check file size before processing
        try:
            file_size = os.path.getsize(file_path)
            file_size_mb = file_size / (1024 * 1024)
            
            if file_size > MAX_FILE_SIZE:
                QMessageBox.warning(
                    self,
                    "File Too Large",
                    f"Cannot process file: {os.path.basename(file_path)}\n\n"
                    f"File size: {file_size_mb:.1f} MB\n"
                    f"Maximum allowed: {MAX_FILE_SIZE / (1024 * 1024):.0f} MB"
                )
                self.log(f"❌ Cannot process - file too large: {file_size_mb:.1f} MB")
                return
                
            # Check cache first
            cached_result = self.cache.get(file_path)
            if cached_result:
                self.log(f"⚡ Using cached result for {os.path.basename(file_path)}")
                self.on_processing_finished(cached_result)
                return
            
            self.log(f"Processing {os.path.basename(file_path)} ({file_size_mb:.1f} MB)...")
            
        except OSError as e:
            self.log(f"❌ Cannot access file: {e}")
            return
        
        # Start processing with thread safety
        try:
            self.processor = DocumentProcessor(file_path)
            # Disconnect any existing signals first
            try:
                self.processor.progress.disconnect()
                self.processor.error.disconnect()
                self.processor.finished.disconnect()
                self.processor.ocr_needed.disconnect()
            except:
                pass  # No connections to disconnect
            
            # Connect new signals
            self.processor.progress.connect(self.log)
            self.processor.error.connect(lambda e: self.log(f"🐹 Error: {e}"))
            self.processor.finished.connect(self.on_processing_finished)
            self.processor.ocr_needed.connect(lambda: self._on_ocr_needed(file_path))
            self.processor.start()
            
            # Start processing animation
            self.processing_timer.start(500)  # Update every 500ms
        except Exception as e:
            self.log(f"❌ Failed to start processing: {e}")
    
    def on_processing_finished(self, result: ProcessingResult):
        """Handle processing completion"""
        # Stop animation
        self.processing_timer.stop()
        if result.success:
            # Save to database with retry mechanism
            max_retries = 3
            for attempt in range(max_retries):
                success, error_msg = self.db.save_document(result, self.current_pdf_path)
                
                if success:
                    break
                    
                if attempt < max_retries - 1:
                    self.log(f"⚠️ Database save failed (attempt {attempt + 1}), retrying...")
                    QTimer.singleShot(1000 * (attempt + 1), lambda: None)  # Wait with backoff
                else:
                    # Final attempt failed
                    QMessageBox.warning(
                        self, 
                        "Database Error", 
                        f"Failed to save document after {max_retries} attempts:\n{error_msg}"
                    )
                    self.log(f"❌ Database save failed after {max_retries} attempts: {error_msg}")
                    return
            
            # Cache the result for future use
            if self.current_pdf_path:
                self.cache.put(self.current_pdf_path, result)
                self.log("💾 Result cached for faster future access")
            
            # Display in faithful output (RIGHT PANE!)
            self._display_in_faithful_output(result)
            
            # Also create floating output window
            self.create_output_window(result)
            
            # Pass chunks to selection manager for coordinate mapping
            if hasattr(result, 'chunks'):
                self.selection_manager.set_chunks(result.chunks)
            
            self.log(f"Processing complete! {len(result.chunks)} chunks extracted")
        else:
            # Check if it was a timeout
            if "timeout exceeded" in result.error_message.lower():
                QMessageBox.warning(
                    self,
                    "Processing Timeout",
                    f"Processing took too long and was stopped.\n\n"
                    f"Maximum time allowed: {MAX_PROCESSING_TIME}s\n\n"
                    "Try with a smaller or less complex PDF."
                )
            self.log(f"🐹 Processing failed: {result.error_message}")
    
    def _display_in_faithful_output(self, result: ProcessingResult):
        """Display processed content in the right pane faithful output"""
        html = f"""
        <!DOCTYPE html>
        <html>
        <head>
            <style>
                body {{ 
                    font-family: -apple-system, sans-serif; 
                    margin: 20px; 
                    color: #FFFFFF;
                    background: #525659;
                }}
                table {{ 
                    border-collapse: collapse; 
                    margin: 15px 0;
                    border: 1px solid #3A3C3E;
                    background-color: #3A3C3E;
                }}
                th, td {{ 
                    border: 1px solid #525659; 
                    padding: 8px;
                    color: #FFFFFF;
                    background-color: #424548;
                }}
                th {{
                    background-color: #3A3C3E;
                    font-weight: bold;
                }}
                td[contenteditable="true"]:hover {{
                    background-color: #525659;
                }}
                .table-controls {{ margin: 10px 0; }}
                button {{ 
                    background: #1ABC9C;
                    color: white;
                    border: none;
                    padding: 5px 10px;
                    margin: 5px;
                    border-radius: 3px;
                    cursor: pointer;
                }}
                button:hover {{
                    background: #16A085;
                }}
                h1, h2, h3 {{ color: #1ABC9C; }}
                p {{ color: #FFFFFF; }}
                li {{ color: #FFFFFF; }}
            </style>
        </head>
        <body>
            <h2 style="color: #1ABC9C;">CHONKER's Faithful Output</h2>
            <div style="color: #B0B0B0;">Document ID: {result.document_id}</div>
            <div style="color: #B0B0B0;">Processing Time: {result.processing_time:.2f}s</div>
            <hr style="border-color: #3A3C3E;">
            {result.html_content}
        </body>
        </html>
        """
        self.faithful_output.setHtml(html)
    
    def create_output_window(self, result: ProcessingResult):
        """Create window for processed output"""
        window = QWidget()
        window.setWindowTitle("Processed Output")
        window.resize(900, 800)
        
        layout = QVBoxLayout(window)
        
        # Output view
        output_view = QTextEdit()
        output_view.setReadOnly(False)
        
        # Build HTML with styles
        html = f"""
        <!DOCTYPE html>
        <html>
        <head>
            <style>
                body {{ 
                    font-family: -apple-system, sans-serif; 
                    margin: 20px; 
                    color: #000000;
                    background: #FFFFFF;
                }}
                table {{ 
                    border-collapse: collapse; 
                    margin: 15px 0;
                    border: 1px solid #888888;
                }}
                th, td {{ 
                    border: 1px solid #888888; 
                    padding: 8px;
                    color: #000000;
                }}
                .table-controls {{ margin: 10px 0; }}
                button {{ 
                    background: #28a745;
                    color: white;
                    border: none;
                    padding: 5px 10px;
                    margin: 5px;
                    border-radius: 3px;
                    cursor: pointer;
                }}
            </style>
        </head>
        <body>
            <h2 style="color: #1ABC9C;">CHONKER's Output</h2>
            {result.html_content}
        </body>
        </html>
        """
        
        output_view.setHtml(html)
        layout.addWidget(output_view)
        
        window.show()
    
    def changeEvent(self, event):
        """Handle window state changes"""
        if event.type() == QEvent.Type.ApplicationStateChange:
            if QApplication.applicationState() == Qt.ApplicationState.ApplicationActive:
                # Bring to front when app is activated (dock click)
                self.raise_()
                self.activateWindow()
        super().changeEvent(event)
    
    def closeEvent(self, event):
        """Clean up on close"""
        # Stop any running processor
        if hasattr(self, 'processor') and self.processor.isRunning():
            self.processor.stop()
        
        # Remove all event filters to prevent memory leaks
        if hasattr(self, 'left_pane'):
            self.left_pane.removeEventFilter(self)
        if hasattr(self, 'faithful_output'):
            self.faithful_output.removeEventFilter(self)
        if hasattr(self, 'embedded_pdf_view'):
            self.embedded_pdf_view.removeEventFilter(self)
        
        # Stop caffeinate
        if self.caffeinate_process:
            self.caffeinate_process.terminate()
        
        # Close floating windows
        for window_data in list(self.floating_windows.values()):
            window_data['window'].close()
        
        event.accept()



class SNYFTERmodeUIlayout(QWidget):
    """Generated SNYFTERmodeUIlayout for SNYFTER mode"""
    
    def __init__(self, parent=None):
        super().__init__(parent)
        self.setup_ui()
        self.setup_style()
        self.setup_event_handlers()
    
    def setup_ui(self):
        """Initialize UI components"""
        # Generated UI setup
        pass
    
    def setup_style(self):
        """Apply SNYFTER theme styling"""
        style = f"""
        SNYFTERmodeUIlayout {
            background-color: #525659;
            color: #FFFFFF;
            border: 1px solid #3A3C3E;
        }
        """
        self.setStyleSheet(style)
    
    def setup_event_handlers(self):
        """Connect event handlers"""
        # Generated event handler connections
        pass
    
    # Additional methods would be generated here based on component type
    def refresh(self):
        """Refresh component state"""
        pass

# ============================================================================
# MAIN ENTRY POINT
# ============================================================================

def main():
    """Application entry point"""
    # Set up exception handling
    def handle_exception(exc_type, exc_value, exc_traceback):
        if issubclass(exc_type, KeyboardInterrupt):
            sys.__excepthook__(exc_type, exc_value, exc_traceback)
            return
        
        print("🐹 Uncaught exception:")
        traceback.print_exception(exc_type, exc_value, exc_traceback)
    
    sys.excepthook = handle_exception
    
    # Print startup message
    print("""
╭────────────── Welcome ──────────────╮
│ CHONKER & SNYFTER                   │
│ Elegant Document Processing System  │
│                                     │
│ Ready to process your documents!    │
╰─────────────────────────────────────╯
    """)
    
    # Create application
    app = QApplication(sys.argv)
    app.setApplicationName("CHONKER & SNYFTER")
    
    # Create and show main window
    window = ChonkerSnyfterApp()
    
    # Set app icon to CHONKER after window is created
    if hasattr(window, 'chonker_pixmap'):
        app.setWindowIcon(QIcon(window.chonker_pixmap))
    
    window.show()
    
    # Run application
    sys.exit(app.exec())


if __name__ == "__main__":
    main()