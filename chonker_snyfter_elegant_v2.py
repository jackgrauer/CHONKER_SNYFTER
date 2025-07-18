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
import time
import warnings
import duckdb
import pandas as pd
import re
import json
import shutil

# Suppress PyTorch pin_memory warnings on MPS
warnings.filterwarnings("ignore", message=".*pin_memory.*MPS.*")

# Qt imports
from PyQt6.QtWidgets import (
    QApplication, QMainWindow, QWidget, QVBoxLayout, QHBoxLayout,
    QPushButton, QFileDialog, QMessageBox, QTextEdit, QLabel, 
    QSplitter, QDialog, QMenuBar, QMenu, QToolBar, QStatusBar,
    QGroupBox, QTreeWidget, QTreeWidgetItem, QProgressDialog, QSizePolicy
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
            
            
            self.finished.emit(result_obj)
            
        except Exception as e:
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
            if self._get_pdf_size_mb() > 50:
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
    
    
    def _enhance_text_formatting(self, text: str) -> str:
        """Enhance text with proper superscript/subscript formatting"""
        
        # Unicode superscript mapping using comprehensions
        sup_digits = {chr(0x2070 + i): f'<sup>{i}</sup>' for i in range(10) if i != 1 and i != 2 and i != 3}
        sup_digits.update({chr(0x00B9): '<sup>1</sup>', chr(0x00B2): '<sup>2</sup>', chr(0x00B3): '<sup>3</sup>'})
        sup_chars = {chr(c): f'<sup>{ch}</sup>' for c, ch in [
            (0x207A, '+'), (0x207B, '-'), (0x207F, 'n'), (0x2071, 'i'), 
            (0x02B3, 'r'), (0x02E2, 's'), (0x1D57, 't'), (0x02B0, 'h'),
            (0x02E3, 'x'), (0x02B8, 'y'), (0x1DBB, 'z'), (0x1D43, 'a'),
            (0x1D47, 'b'), (0x1D9C, 'c'), (0x1D48, 'd'), (0x1D49, 'e'),
            (0x1DA0, 'f'), (0x1D4D, 'g')
        ]}
        superscript_map = {**sup_digits, **sup_chars}
        
        # Unicode subscript mapping using comprehensions
        sub_digits = {chr(0x2080 + i): f'<sub>{i}</sub>' for i in range(10)}
        sub_chars = {chr(c): f'<sub>{ch}</sub>' for c, ch in [
            (0x208A, '+'), (0x208B, '-'), (0x2090, 'a'), (0x2091, 'e'),
            (0x2092, 'o'), (0x2093, 'x'), (0x2095, 'h'), (0x2096, 'k'),
            (0x2097, 'l'), (0x2098, 'm'), (0x2099, 'n'), (0x209A, 'p'),
            (0x209B, 's'), (0x209C, 't')
        ]}
        subscript_map = {**sub_digits, **sub_chars}
        
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
                    <button onclick="exportTable(this)" style="background: #1ABC9C;">📤 Export to SQL</button>
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
        
        function exportTable(btn) {
            const table = btn.parentNode.nextSibling;
            const tableHtml = table.outerHTML;
            const pageNum = table.getAttribute('data-page') || '1';
            
            // Send to Python backend via Qt channel
            if (window.qt && window.qt.webChannelTransport) {
                new QWebChannel(qt.webChannelTransport, function(channel) {
                    channel.objects.backend.exportTable(tableHtml, pageNum);
                });
            } else {
                alert('Export feature requires Qt WebChannel');
            }
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
# MAIN APPLICATION
# ============================================================================



class ChonkerSnyfterApp(QMainWindow):
    """Main application window - clean and elegant"""
    
    def __init__(self):
        super().__init__()
        self.current_mode = Mode.CHONKER
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
        
        self._init_caffeinate()
        self._init_ui()
        self._apply_theme()
        
        # Set up selection sync after UI is created
        
    
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
    
    def _validate_file_size(self, file_path: str, action: str = "open") -> Optional[float]:
        """Validate file size and show appropriate warnings.
        
        Args:
            file_path: Path to the file to validate
            action: Action being performed ("open" or "process")
            
        Returns:
            File size in MB if valid, None if invalid or error occurred
        """
        try:
            file_size = os.path.getsize(file_path)
            file_size_mb = file_size / (1024 * 1024)
            
            if file_size > MAX_FILE_SIZE:
                # Different messages for different actions
                if action == "open":
                    QMessageBox.warning(
                        self,
                        "File Too Large",
                        f"Cannot open file: {os.path.basename(file_path)}\n\n"
                        f"File size: {file_size_mb:.1f} MB\n"
                        f"Maximum allowed: {MAX_FILE_SIZE / (1024 * 1024):.0f} MB\n\n"
                        "Please use a smaller PDF file."
                    )
                    self.log(f"❌ File too large: {file_size_mb:.1f} MB")
                else:  # process
                    QMessageBox.warning(
                        self,
                        "File Too Large",
                        f"Cannot process file: {os.path.basename(file_path)}\n\n"
                        f"File size: {file_size_mb:.1f} MB\n"
                        f"Maximum allowed: {MAX_FILE_SIZE / (1024 * 1024):.0f} MB"
                    )
                    self.log(f"❌ Cannot process - file too large: {file_size_mb:.1f} MB")
                return None
                
            return file_size_mb
            
        except OSError as e:
            if action == "open":
                QMessageBox.critical(
                    self,
                    "File Error", 
                    f"Cannot access file: {e}"
                )
            else:  # process
                self.log(f"❌ Cannot access file: {e}")
            return None
        
    
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
        self.setWindowTitle("")  # No title
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
        
        # Add custom context menu for export
        self.faithful_output.setContextMenuPolicy(Qt.ContextMenuPolicy.CustomContextMenu)
        self.faithful_output.customContextMenuRequested.connect(self._show_export_menu)
        
        # Initialize SQL exporter
        self.sql_exporter = ChonkerSQLExporter()
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
    
    def _show_export_menu(self, pos):
        """Show context menu for exporting tables"""
        cursor = self.faithful_output.textCursor()
        cursor.setPosition(self.faithful_output.cursorForPosition(pos).position())
        
        # Check if cursor is near a table
        cursor.movePosition(cursor.MoveOperation.StartOfBlock)
        cursor.movePosition(cursor.MoveOperation.EndOfBlock, cursor.MoveMode.KeepAnchor)
        block_text = cursor.selectedText()
        
        if '<table' in block_text or cursor.charFormat().anchorHref():
            menu = QMenu(self)
            export_action = menu.addAction("Export Content to SQL")
            export_action.triggered.connect(lambda: self._export_table_at_cursor(cursor))
            menu.exec(self.faithful_output.mapToGlobal(pos))
    
    def _export_table_at_cursor(self, cursor):
        """Export the table at the current cursor position"""
        # Find the table HTML
        doc = self.faithful_output.document()
        html = doc.toHtml()
        
        # Extract table near cursor position
        cursor_pos = cursor.position()
        
        # Simple table extraction - find nearest table tags
        import re
        tables = list(re.finditer(r'<table[^>]*>.*?</table>', html, re.DOTALL | re.IGNORECASE))
        
        if not tables:
            QMessageBox.information(self, "No Table Found", "No table found at cursor position")
            return
        
        # Find the table closest to cursor
        # This is simplified - in production you'd want more robust table detection
        table_html = tables[0].group(0) if tables else ""
        
        if table_html:
            # Export using the SQL exporter
            try:
                source_pdf = self.current_pdf_path if hasattr(self, 'current_pdf_path') else "unknown.pdf"
                export_path = self.sql_exporter.export_table(
                    table_html, 
                    source_pdf, 
                    1,  # Page number - simplified
                    qc_user=os.getenv('USER', 'user')
                )
                
                QMessageBox.information(
                    self, 
                    "Export Successful", 
                    f"Table exported to:\n{export_path}\n\nParquet file saved for efficient querying!"
                )
            except Exception as e:
                QMessageBox.critical(self, "Export Failed", f"Failed to export table: {str(e)}")
    
    def export_to_sql(self):
        """Export the quality-controlled content to SQL database"""
        if not hasattr(self, '_last_processing_result') or not self._last_processing_result:
            QMessageBox.warning(self, "No Document", "Please process a document first")
            return
        
        # Get suggested filename from source PDF
        source_pdf = self.current_pdf_path if hasattr(self, 'current_pdf_path') else "unknown.pdf"
        suggested_name = os.path.basename(source_pdf).replace('.pdf', '_export.duckdb')
        
        # Show native file save dialog
        file_path, _ = QFileDialog.getSaveFileName(
            self,
            "Export to DuckDB",
            suggested_name,
            "DuckDB Database (*.duckdb);;All Files (*)"
        )
        
        if not file_path:
            return  # User cancelled
        
        # Ensure .duckdb extension
        if not file_path.endswith('.duckdb'):
            file_path += '.duckdb'
        
        # Get the current content from the faithful output (which may have been edited)
        current_html = self.faithful_output.toHtml()
        
        # Extract just the body content (remove Qt's HTML wrapper)
        from bs4 import BeautifulSoup
        soup = BeautifulSoup(current_html, 'html.parser')
        body = soup.find('body')
        if body:
            # Get all content after the header info
            hr = body.find('hr')
            if hr:
                content_html = ''.join(str(sibling) for sibling in hr.find_next_siblings())
            else:
                content_html = str(body)
        else:
            content_html = current_html
        
        # Show progress
        progress = QProgressDialog("Exporting content to SQL...", None, 0, 100, self)
        progress.setWindowModality(Qt.WindowModality.WindowModal)
        progress.setMinimumDuration(0)
        progress.setCancelButton(None)  # Can't cancel
        progress.setValue(25)
        
        try:
            # Create exporter with chosen file path
            exporter = ChonkerSQLExporter(file_path)
            
            # Export the content
            export_id = exporter.export_content(
                content_html,
                source_pdf,
                qc_user=os.getenv('USER', 'user')
            )
            
            progress.setValue(75)
            
            # Get export statistics
            stats = exporter.conn.execute("""
                SELECT 
                    COUNT(*) as total_elements,
                    COUNT(DISTINCT element_type) as unique_types
                FROM chonker_content 
                WHERE export_id = ?
            """, [export_id]).fetchone()
            
            progress.setValue(100)
            progress.close()
            
            # Create export directory next to the database file
            export_dir = os.path.join(os.path.dirname(file_path), f"{export_id}_files")
            os.makedirs(export_dir, exist_ok=True)
            
            # Move/copy exported files to the chosen location
            if os.path.exists("exports"):
                import shutil
                for file in os.listdir("exports"):
                    if file.startswith(export_id):
                        shutil.move(
                            os.path.join("exports", file),
                            os.path.join(export_dir, file)
                        )
            
            # Show success message
            summary = f"Export Complete!\n\n"
            summary += f"Export ID: {export_id}\n\n"
            summary += f"✓ {stats[0]} content elements exported\n"
            summary += f"✓ {stats[1]} different element types\n\n"
            summary += f"Files created:\n"
            summary += f"  • Database: {os.path.basename(file_path)}\n"
            summary += f"  • Export files: {os.path.basename(export_dir)}/\n\n"
            summary += "The content is now ready for downstream applications!"
            
            QMessageBox.information(self, "Export Successful", summary)
            self.log(f"Exported to: {file_path}")
            
            # Close the exporter connection
            exporter.conn.close()
            
        except Exception as e:
            progress.close()
            QMessageBox.critical(self, "Export Failed", f"Failed to export content:\n\n{str(e)}")
            self.log(f"Export failed: {str(e)}")
    
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
    
    def _create_top_bar(self, parent_layout):
        """Create resizable top bar with mode toggle"""
        self.top_bar = QWidget()  # Store as instance variable
        self.top_bar.setMinimumHeight(60)
        self.top_bar.setMaximumHeight(150)  # Allow vertical resizing
        self.top_bar.setObjectName("topBar")
        self.top_bar.setStyleSheet("""
            #topBar {
                background-color: #1ABC9C;
                border: 2px solid #3A3C3E;
                border-radius: 2px;
            }
        """)
        
        # Make top bar focusable and install event filter for pane selection
        self.top_bar.setMouseTracking(True)
        self.top_bar.installEventFilter(self)
        
        top_bar = self.top_bar  # Keep local reference for compatibility
        
        # Main layout - everything directly in here for proper alignment
        layout = QHBoxLayout(top_bar)
        layout.setContentsMargins(5, 8, 5, 8)  # Consistent margin from border
        layout.setSpacing(5)  # Small consistent spacing between ALL elements
        layout.setAlignment(Qt.AlignmentFlag.AlignLeft)
        
        # Now add buttons to the container
        # Mode toggle with sacred emojis - FULL HEIGHT PRIMARY VISUAL ELEMENT
        self.chonker_btn = QPushButton()
        # Scale hamster emoji larger
        scaled_hamster = self.chonker_pixmap.scaled(48, 48, Qt.AspectRatioMode.KeepAspectRatio, Qt.TransformationMode.SmoothTransformation)
        self.chonker_btn.setIcon(QIcon(scaled_hamster))
        self.chonker_btn.setIconSize(scaled_hamster.size())
        self.chonker_btn.setText(" CHONKER")
        self.chonker_btn.setCheckable(True)
        self.chonker_btn.setChecked(True)
        self.chonker_btn.setMinimumWidth(180)  # Make button wider
        self.chonker_btn.setSizePolicy(QSizePolicy.Policy.Preferred, QSizePolicy.Policy.Expanding)
        self.chonker_btn.setStyleSheet("""
            QPushButton {
                background-color: #1ABC9C;
                color: white;
                font-size: 18px;
                font-weight: bold;
                border: none;
                padding: 5px;
            }
            QPushButton:hover {
                background-color: #16A085;
            }
        """)
        self.chonker_btn.clicked.connect(lambda: self.set_mode(Mode.CHONKER))
        
        layout.addWidget(self.chonker_btn)
        
        # Remove terminal container - add directly to main layout
        
        # Terminal display - smaller and on the right
        self.terminal = QTextEdit()
        self.terminal.setMinimumHeight(40)  # Match button heights
        self.terminal.setMaximumHeight(50)  # Allow slight flexibility
        self.terminal.setMaximumWidth(250)  # Even narrower for tighter layout
        self.terminal.setReadOnly(True)  # READ-ONLY for display only
        self.terminal.setObjectName("terminal")
        self.terminal.setVerticalScrollBarPolicy(Qt.ScrollBarPolicy.ScrollBarAlwaysOff)
        self.terminal.setHorizontalScrollBarPolicy(Qt.ScrollBarPolicy.ScrollBarAlwaysOff)
        # Ensure no extra space at bottom
        self.terminal.setLineWrapMode(QTextEdit.LineWrapMode.WidgetWidth)  # Wrap within widget
        self.terminal.document().setDocumentMargin(2)  # Small margin
        # Set font metrics to ensure full lines are visible
        font = self.terminal.font()
        font.setFamily('Courier New')
        font.setPixelSize(11)
        self.terminal.setFont(font)
        self.terminal.setStyleSheet("""
            QTextEdit {
                background-color: #2D2F31;
                color: #1ABC9C;
                font-family: 'Courier New', monospace;
                font-size: 11px;
                line-height: 14px;
                border: 1px solid #3A3C3E;
                padding: 2px;
            }
        """)
        
        # Add terminal directly to main layout (no expand button)
        layout.addWidget(self.terminal)
        
        # Quick actions - subordinate visual style
        action_button_style = """
            QPushButton {
                background-color: #3A3C3E;
                color: #B0B0B0;
                font-size: 12px;
                border: 1px solid #525659;
                padding: 6px 10px;
                min-height: 30px;
                max-height: 35px;
            }
            QPushButton:hover {
                background-color: #525659;
                color: #FFFFFF;
            }
            QPushButton:disabled {
                background-color: #2D2F31;
                color: #666666;
            }
        """
        
        open_btn = QPushButton("Open")
        open_btn.setToolTip("Open PDF (Ctrl+O)")
        open_btn.clicked.connect(self.open_pdf)
        open_btn.setShortcut(QKeySequence.StandardKey.Open)
        open_btn.setStyleSheet(action_button_style)
        
        process_btn = QPushButton("Process")
        process_btn.setToolTip("Process (Ctrl+P)")
        process_btn.clicked.connect(self.process_current)
        process_btn.setStyleSheet(action_button_style)
        
        export_btn = QPushButton("Export")
        export_btn.setToolTip("Export quality-controlled content to SQL (Cmd+E)")
        export_btn.clicked.connect(self.export_to_sql)
        export_btn.setEnabled(False)  # Disabled until processing is done
        export_btn.setShortcut(QKeySequence("Ctrl+E"))  # Ctrl+E maps to Cmd+E on Mac
        export_btn.setStyleSheet(action_button_style)
        self.export_btn = export_btn  # Store reference
        
        # Add action buttons directly to main layout
        layout.addWidget(open_btn)
        layout.addWidget(process_btn)
        layout.addWidget(export_btn)
        
        # Add stretch to push everything left
        layout.addStretch()
        
        # Add to splitter instead of layout
        if isinstance(parent_layout, QSplitter):
            parent_layout.addWidget(top_bar)
        else:
            parent_layout.addWidget(top_bar)
    
    def _show_welcome(self):
        """Clear the left pane and show shortcuts in terminal"""
        # Clear left pane
        for i in reversed(range(self.left_layout.count())): 
            widget = self.left_layout.itemAt(i).widget()
            if widget:
                widget.setParent(None)
        
        # Show shortcuts in terminal
        self.log("Cmd+O: Open PDF | Cmd+P: Process | Cmd+E: Export")
    
    def _update_pane_styles(self):
        """Update pane borders based on active state"""
        # Keep border width fixed to prevent layout shifts
        border_width = "2"
        
        left_border_color = "#1ABC9C" if self.active_pane == 'left' else "#3A3C3E"
        right_border_color = "#1ABC9C" if self.active_pane == 'right' else "#3A3C3E"
        top_border_color = "#1ABC9C" if self.active_pane == 'top' else "#3A3C3E"
        
        # Qt doesn't support box-shadow, but we can use border width and color for visual feedback
        
        self.left_pane.setStyleSheet(f"""
            QWidget {{
                border: {border_width}px solid {left_border_color};
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
                border: {border_width}px solid {right_border_color};
                border-radius: 2px;
                padding: 10px;
            }}
        """)
        
        # Update top bar if it exists
        if hasattr(self, 'top_bar'):
            self.top_bar.setStyleSheet(f"""
                #topBar {{
                    background-color: #1ABC9C;
                    border: {border_width}px solid {top_border_color};
                    border-radius: 2px;
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
            # Gesture detected
            
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
        """Apply current text zoom by re-rendering the HTML content"""
        if not self.faithful_output:
            return
            
        # Store the current content
        current_html = self.faithful_output.toHtml()
        
        # Check if we have the processing result to re-render properly
        if hasattr(self, '_last_processing_result'):
            # Re-display with new zoom level
            self._display_in_faithful_output(self._last_processing_result)
            # Zoom applied via re-render
        else:
            # Simple approach: just log that we need content first
            pass  # Zoom will apply when content loads
    
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
        
        # Always CHONKER mode now
        self.log("CHONKER mode activated - Ready to process PDFs!")
    
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
        
        # Move cursor to end
        cursor.movePosition(QTextCursor.MoveOperation.End)
        self.terminal.setTextCursor(cursor)
        
        # Ensure we're showing complete lines - scroll to show the last complete line
        scrollbar = self.terminal.verticalScrollBar()
        if scrollbar:
            # Scroll to bottom minus a small offset to ensure complete line visibility
            max_value = scrollbar.maximum()
            scrollbar.setValue(max_value)
    
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
            file_size_mb = self._validate_file_size(file_path, "open")
            if file_size_mb is None:
                return
                
            self.log(f"Opening PDF: {os.path.basename(file_path)} ({file_size_mb:.1f} MB)")
            
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
        file_size_mb = self._validate_file_size(file_path, "process")
        if file_size_mb is None:
            return
            
        self.log(f"Processing {os.path.basename(file_path)} ({file_size_mb:.1f} MB)...")
        
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
            # Display in faithful output (RIGHT PANE!)
            self._display_in_faithful_output(result)
            
            # Also create floating output window
            self.create_output_window(result)
            
            # Enable export button and store result
            if hasattr(self, 'export_btn'):
                self.export_btn.setEnabled(True)
            self._last_processing_result = result
            
            # Log completion
            if hasattr(result, 'chunks'):
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
        # Apply zoom to the base HTML
        zoom_size = self.text_zoom
        
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
                    font-size: {zoom_size}px !important;
                }}
                table {{ 
                    border-collapse: collapse; 
                    margin: 15px 0;
                    border: 1px solid #3A3C3E;
                    background-color: #3A3C3E;
                    font-size: {zoom_size}px !important;
                }}
                th, td {{ 
                    border: 1px solid #525659; 
                    padding: 8px;
                    color: #FFFFFF;
                    background-color: #424548;
                    font-size: {zoom_size}px !important;
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
                h1 {{ color: #1ABC9C; font-size: {int(zoom_size * 1.5)}px !important; }}
                h2 {{ color: #1ABC9C; font-size: {int(zoom_size * 1.3)}px !important; }}
                h3 {{ color: #1ABC9C; font-size: {int(zoom_size * 1.2)}px !important; }}
                p {{ color: #FFFFFF; font-size: {zoom_size}px !important; }}
                li {{ color: #FFFFFF; font-size: {zoom_size}px !important; }}
                div {{ font-size: {zoom_size}px !important; }}
                span {{ font-size: {zoom_size}px !important; }}
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
        # Store the result for re-rendering on zoom changes
        self._last_processing_result = result
    
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
        if hasattr(self, 'embedded_pdf_view') and self.embedded_pdf_view:
            self.embedded_pdf_view.removeEventFilter(self)
        
        # Stop caffeinate
        if self.caffeinate_process:
            self.caffeinate_process.terminate()
        
        # Close floating windows
        for window_data in list(self.floating_windows.values()):
            window_data['window'].close()
        
        event.accept()



# ============================================================================
# SQL EXPORT - CUTTING EDGE DATABASE INTEGRATION 🚀
# ============================================================================

try:
    import duckdb
    import pandas as pd
    DUCKDB_AVAILABLE = True
except ImportError:
    DUCKDB_AVAILABLE = False

class ChonkerSQLExporter:
    """Export quality-controlled HTML content to DuckDB for downstream apps"""
    
    def __init__(self, db_path: str = "chonker_output.duckdb"):
        if not DUCKDB_AVAILABLE:
            raise ImportError("🐹 *cough* DuckDB not installed! Run: pip install duckdb pandas")
        
        self.db_path = db_path
        self.conn = duckdb.connect(db_path)
        self._init_metadata_table()
    
    def _init_metadata_table(self):
        """Initialize metadata tracking table"""
        self.conn.execute("""
            CREATE TABLE IF NOT EXISTS chonker_exports (
                export_id TEXT PRIMARY KEY,
                source_pdf TEXT,
                export_name TEXT,
                original_html TEXT,
                edited_html TEXT,
                content_type TEXT,  -- 'full_document', 'table', 'section', etc.
                content_hash TEXT,
                exported_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                qc_user TEXT,
                edit_count INTEGER DEFAULT 0,
                metadata JSON
            )
        """)
        
        # Create structured content table for parsed elements
        self.conn.execute("""
            CREATE TABLE IF NOT EXISTS chonker_content (
                content_id TEXT PRIMARY KEY,
                export_id TEXT,
                element_type TEXT,  -- 'heading', 'paragraph', 'table', 'list', etc.
                element_order INTEGER,
                element_text TEXT,
                element_html TEXT,
                element_metadata JSON,
                FOREIGN KEY (export_id) REFERENCES chonker_exports(export_id)
            )
        """)
    
    def _infer_schema(self, html_table: str) -> tuple[pd.DataFrame, dict]:
        """Smart schema inference using pandas"""
        # Parse HTML table
        dfs = pd.read_html(html_table)
        if not dfs:
            raise ValueError("No tables found in HTML")
        
        df = dfs[0]
        
        # Infer types
        schema = {}
        for col in df.columns:
            # Try numeric first
            try:
                df[col] = pd.to_numeric(df[col])
                if df[col].dtype == 'int64':
                    schema[col] = 'INTEGER'
                else:
                    schema[col] = 'DOUBLE'
            except:
                # Try datetime
                try:
                    df[col] = pd.to_datetime(df[col])
                    schema[col] = 'TIMESTAMP'
                except:
                    # Default to text
                    schema[col] = 'VARCHAR'
        
        return df, schema
    
    def export_content(self, html_content: str, source_pdf: str, 
                      export_name: str = None, qc_user: str = "user", 
                      content_type: str = "full_document") -> str:
        """Export quality-controlled HTML content to DuckDB"""
        try:
            # Generate export ID
            timestamp = datetime.now()
            export_id = f"{os.path.basename(source_pdf).replace('.pdf', '')}_{timestamp.strftime('%Y%m%d_%H%M%S')}"
            export_id = re.sub(r'[^a-zA-Z0-9_]', '_', export_id)
            
            # Generate export name if not provided
            if not export_name:
                export_name = f"export_{export_id}"
            
            # Parse HTML to extract structured content
            from bs4 import BeautifulSoup
            soup = BeautifulSoup(html_content, 'html.parser')
            
            # Calculate content hash for tracking edits
            content_hash = hashlib.sha256(html_content.encode()).hexdigest()[:16]
            
            # Check if this exact content was already exported
            existing = self.conn.execute(
                "SELECT export_id FROM chonker_exports WHERE content_hash = ?",
                [content_hash]
            ).fetchone()
            
            if existing:
                return existing[0]
            
            # Extract metadata
            metadata = {
                'source_pdf': source_pdf,
                'export_time': timestamp.isoformat(),
                'total_elements': 0,
                'element_types': {}
            }
            
            # Insert main export record
            self.conn.execute("""
                INSERT INTO chonker_exports 
                (export_id, source_pdf, export_name, original_html, edited_html, 
                 content_type, content_hash, qc_user, metadata)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            """, [
                export_id,
                source_pdf,
                export_name,
                html_content[:10000],  # Store first 10KB for reference
                html_content[:10000],  # Edited version (same initially)
                content_type,
                content_hash,
                qc_user,
                json.dumps(metadata)
            ])
            
            # Parse and store structured content
            element_order = 0
            
            # Process all elements in order
            for element in soup.find_all(['h1', 'h2', 'h3', 'h4', 'h5', 'h6', 'p', 'table', 'ul', 'ol', 'div']):
                element_type = element.name
                element_text = element.get_text(strip=True)
                element_html = str(element)
                
                # Skip empty elements
                if not element_text:
                    continue
                
                # Track element types
                metadata['element_types'][element_type] = metadata['element_types'].get(element_type, 0) + 1
                metadata['total_elements'] += 1
                
                # Generate content ID
                content_id = f"{export_id}_{element_order:04d}"
                
                # Store element metadata
                element_meta = {
                    'tag': element_type,
                    'classes': element.get('class', []),
                    'id': element.get('id'),
                    'char_count': len(element_text)
                }
                
                # Special handling for tables
                if element_type == 'table':
                    rows = element.find_all('tr')
                    cols = len(rows[0].find_all(['td', 'th'])) if rows else 0
                    element_meta['rows'] = len(rows)
                    element_meta['cols'] = cols
                
                # Insert structured content
                self.conn.execute("""
                    INSERT INTO chonker_content
                    (content_id, export_id, element_type, element_order, 
                     element_text, element_html, element_metadata)
                    VALUES (?, ?, ?, ?, ?, ?, ?)
                """, [
                    content_id,
                    export_id,
                    element_type,
                    element_order,
                    element_text[:5000],  # Limit text size
                    element_html[:10000],  # Limit HTML size
                    json.dumps(element_meta)
                ])
                
                element_order += 1
            
            # Update metadata
            self.conn.execute("""
                UPDATE chonker_exports 
                SET metadata = ? 
                WHERE export_id = ?
            """, [json.dumps(metadata), export_id])
            
            # Export to Parquet for portability (optional)
            os.makedirs("exports", exist_ok=True)
            
            try:
                # Export main record
                export_df = self.conn.execute(
                    "SELECT * FROM chonker_exports WHERE export_id = ?", 
                    [export_id]
                ).df()
                export_df.to_parquet(os.path.join("exports", f"{export_id}_metadata.parquet"))
                
                # Export content
                content_df = self.conn.execute(
                    "SELECT * FROM chonker_content WHERE export_id = ? ORDER BY element_order", 
                    [export_id]
                ).df()
                content_df.to_parquet(os.path.join("exports", f"{export_id}_content.parquet"))
                parquet_exported = True
            except Exception as parquet_error:
                print(f"Warning: Could not export Parquet files: {parquet_error}")
                content_df = self.conn.execute(
                    "SELECT * FROM chonker_content WHERE export_id = ? ORDER BY element_order", 
                    [export_id]
                ).df()
                parquet_exported = False
            
            # Also export as single JSON for easy consumption
            export_json = {
                'export_id': export_id,
                'metadata': metadata,
                'content': []
            }
            
            for _, row in content_df.iterrows():
                export_json['content'].append({
                    'type': row['element_type'],
                    'text': row['element_text'],
                    'html': row['element_html'],
                    'metadata': json.loads(row['element_metadata'])
                })
            
            with open(os.path.join("exports", f"{export_id}.json"), 'w') as f:
                json.dump(export_json, f, indent=2)
            
            # Return export ID and parquet status
            self._parquet_exported = parquet_exported
            return export_id
            
        except Exception as e:
            print(f"Export error: {str(e)}")
            raise
    
    def get_api_ready_json(self, table_name: str) -> str:
        """Generate JSON export for REST APIs"""
        result = self.conn.execute(f"SELECT * FROM {table_name}").fetchall()
        columns = [desc[0] for desc in self.conn.description]
        
        data = []
        for row in result:
            data.append(dict(zip(columns, row)))
        
        return json.dumps(data, indent=2, default=str)
    
    def generate_crud_sql(self, table_name: str) -> dict:
        """Generate CRUD SQL statements for other apps"""
        schema = self.conn.execute(f"DESCRIBE {table_name}").fetchall()
        
        columns = [col[0] for col in schema]
        placeholders = ['?' for _ in columns]
        
        return {
            'create': f"INSERT INTO {table_name} ({', '.join(columns)}) VALUES ({', '.join(placeholders)})",
            'read': f"SELECT * FROM {table_name} WHERE id = ?",
            'update': f"UPDATE {table_name} SET {', '.join([f'{col} = ?' for col in columns])} WHERE id = ?",
            'delete': f"DELETE FROM {table_name} WHERE id = ?",
            'schema': schema
        }


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
    
    # Simple startup message
    print("CHONKER ready.")
    
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

