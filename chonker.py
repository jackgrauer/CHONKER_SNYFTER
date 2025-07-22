#!/usr/bin/env python3
"""CHONKER - PDF Processing with Hamster Wisdom"""

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
import torch  # For OCR GPU detection

# Try to import pyarrow
try:
    import pyarrow as pa
    import pyarrow.dataset as ds
    import pyarrow.parquet as pq
    PYARROW_AVAILABLE = True
except ImportError:
    PYARROW_AVAILABLE = False
    print("Warning: PyArrow not available. Install with: uv pip install pyarrow")

# Suppress PyTorch pin_memory warnings on MPS
warnings.filterwarnings("ignore", message=".*pin_memory.*MPS.*")

# Qt imports
from PyQt6.QtWidgets import (
    QApplication, QMainWindow, QWidget, QVBoxLayout, QHBoxLayout,
    QPushButton, QFileDialog, QMessageBox, QTextEdit, QLabel, 
    QSplitter, QDialog, QMenuBar, QMenu, QToolBar, QStatusBar,
    QGroupBox, QTreeWidget, QTreeWidgetItem, QProgressDialog, QSizePolicy,
    QInputDialog
)
from PyQt6.QtCore import (
    Qt, QThread, pyqtSignal, QTimer, QPointF, QObject, QEvent,
    QRect, QPropertyAnimation, QEasingCurve, QRectF, QSettings
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
    print("Warning: Docling not available. Install with: uv pip install docling")

from dataclasses import dataclass, field



MAX_FILE_SIZE = 500 * 1024 * 1024  # 500MB
MAX_PROCESSING_TIME = 300  # 5 minutes in seconds

# Mode enum removed - always CHONKER mode


@dataclass
class DocumentChunk:
    index: int
    type: str
    content: str
    page: Optional[int] = None
    confidence: float = 1.0
    metadata: Dict[str, Any] = field(default_factory=dict)


@dataclass
class ProcessingResult:
    success: bool
    document_id: str
    chunks: List[DocumentChunk]
    html_content: str
    markdown_content: str
    processing_time: float
    error_message: Optional[str] = None
    warnings: List[str] = field(default_factory=list)


class DocumentProcessor(QThread):
    
    finished = pyqtSignal(ProcessingResult)
    progress = pyqtSignal(str)
    error = pyqtSignal(str)
    chunk_processed = pyqtSignal(int, int)
    ocr_needed = pyqtSignal()
    
    def __init__(self, pdf_path: str, use_ocr: bool = False):
        super().__init__()
        self.pdf_path = pdf_path
        self._stop_event = threading.Event()  # Thread-safe stop flag
        self.start_time = None
        self.timeout_occurred = False
        self.use_ocr = use_ocr
    
    def stop(self):
        self._stop_event.set()
        if self.isRunning():
            if not self.wait(5000):  # Wait up to 5 seconds
                self.terminate()  # Force terminate if needed
                self.wait()  # Wait for termination
    
    def _check_timeout(self) -> bool:
        if self.start_time and not self.timeout_occurred:
            elapsed = (datetime.now() - self.start_time).total_seconds()
            if elapsed > MAX_PROCESSING_TIME:
                self.timeout_occurred = True
                self.error.emit(f"⏱️ Processing timeout exceeded ({MAX_PROCESSING_TIME}s)")
                return True
        return False
    
    def run(self):
        start_time = datetime.now()
        self.start_time = start_time
        
        try:
            # Validate PDF header before processing
            if not self._validate_pdf_header():
                raise ValueError("Invalid PDF file format")
            
            # Check if we should stop or timeout
            if self._stop_event.is_set() or self._check_timeout():
                return
            
            # Initialize docling with tqdm fix
            self._init_docling()
            
            # Check if we should stop or timeout
            if self._stop_event.is_set() or self._check_timeout():
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
            if self._stop_event.is_set() or self._check_timeout():
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
        if not DOCLING_AVAILABLE:
            raise Exception("🐹 *cough* Docling not installed!")
        
        # Fix tqdm issue
        import tqdm
        if not hasattr(tqdm.tqdm, '_lock'):
            tqdm.tqdm._lock = threading.RLock()
    
    def _get_pdf_size_mb(self) -> float:
        try:
            return os.path.getsize(self.pdf_path) / (1024 * 1024)
        except (OSError, IOError):
            return 0
    
    def _should_use_lazy_loading(self) -> bool:
        file_size_mb = self._get_pdf_size_mb()
        # Use lazy loading for files over 50MB
        return file_size_mb > 50
    
    def _detect_scanned_pdf(self) -> bool:
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
            
        except Exception:
            return False
    
    def _convert_document_with_ocr(self):
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
        chunks = []
        html_parts = ['<div id="document-content" contenteditable="true">']
        
        items = list(result.document.iterate_items())
        total = len(items)
        current_page = 0
        
        for idx, (item, level) in enumerate(items):
            if self._stop_event.is_set() or self._check_timeout():
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
        
        return chunks, '\n'.join(html_parts)
    
    def _create_chunk(self, item, level: int, index: int, page: int = 0) -> DocumentChunk:
        return DocumentChunk(
            index=index,
            type=type(item).__name__.lower().replace('item', ''),
            content=getattr(item, 'text', str(item)),
            metadata={'level': level, 'page': page}
        )
    
    def _item_to_html(self, item, level: int) -> str:
        item_type = type(item).__name__
        if item_type == 'SectionHeaderItem' and hasattr(item, 'text'):
            h = min(level + 1, 3)
            return f'<h{h}>{self._enhance_text_formatting(escape(str(item.text)))}</h{h}>'
        elif item_type == 'TableItem':
            return self._table_to_html(item)
        elif item_type == 'TextItem' and hasattr(item, 'text'):
            return f'<p>{self._enhance_text_formatting(escape(str(item.text)))}</p>'
        elif item_type == 'ListItem' and hasattr(item, 'text'):
            return f'<li>{self._enhance_text_formatting(escape(str(item.text)))}</li>'
        return ''
    
    
    def _enhance_text_formatting(self, text: str) -> str:
        import re
        # Basic chem formula support  
        text = re.sub(r'\b(H|O|N|C|Na|Ca|Fe)(\d+)', r'\1<sub>\2</sub>', text)
        text = re.sub(r'\^(\d+)', r'<sup>\1</sup>', text)
        text = re.sub(r'_(\d+)', r'<sub>\1</sub>', text)
        return text
    
    def _table_to_html(self, table_item) -> str:
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
                    
            except (AttributeError, IndexError):
                html.append('<tr><td>[Table Content]</td></tr>')
        else:
            html.append('<tr><td>[Table]</td></tr>')
        
        html.append('</table>')
        return '\n'.join(html)
    
    
    def _generate_document_id(self) -> str:
        return hashlib.sha256(f"{self.pdf_path}_{datetime.now().isoformat()}".encode()).hexdigest()[:16]
    
    def _handle_error(self, error: Exception, start_time: datetime):
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






class ChonkerApp(QMainWindow):
    """Main application window"""
    
    def __init__(self):
        super().__init__()
        # Mode tracking removed - always CHONKER
        self.current_pdf_path = None
        self.active_pane = 'right'  # Track which pane is active ('left', 'right', 'top')
        self.embedded_pdf_view = None  # For embedded PDF viewer
        self.recent_files = []  # Track recent files
        self._load_recent_files()
        self.setAcceptDrops(True)  # Enable drag & drop
        
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
        
        self._init_ui()
        self._apply_theme()
        self.restore_geometry()
        
        # Set up selection sync after UI is created

    def _load_sacred_emojis(self):
        # Load the sacred Android 7.1 Noto emojis - NEVER let go of them!
        self.chonker_pixmap = QPixmap("assets/emojis/chonker.png")
        print("Sacred Android 7.1 CHONKER emoji loaded!")
    
    def _validate_file_size(self, file_path: str, action: str = "open") -> Optional[float]:
        # Validate file size and show appropriate warnings.
        try:
            file_size = os.path.getsize(file_path)
            file_size_mb = file_size / (1024 * 1024)
            if file_size > MAX_FILE_SIZE:
                verb = "open" if action == "open" else "process"
                msg = f"Cannot {verb} file: {os.path.basename(file_path)}\n\nFile size: {file_size_mb:.1f} MB\nMaximum allowed: {MAX_FILE_SIZE / (1024 * 1024):.0f} MB"
                if action == "open": msg += "\n\nPlease use a smaller PDF file."
                QMessageBox.warning(self, "File Too Large", msg)
                self.log(f"❌ {'File' if action == 'open' else 'Cannot process - file'} too large: {file_size_mb:.1f} MB")
                return None
            return file_size_mb
        except OSError as e:
            if action == "open": QMessageBox.critical(self, "File Error", f"Cannot access file: {e}")
            else: self.log(f"❌ Cannot access file: {e}")
            return None
        
    
    
    
    def _init_ui(self):
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
        self.main_splitter.setStyleSheet("QSplitter::handle {background-color: #3A3C3E;}")
        layout.addWidget(self.main_splitter)
        
        # Top bar (resizable)
        self._create_top_bar(self.main_splitter)
        
        # Content area - split view like before
        self.splitter = QSplitter(Qt.Orientation.Horizontal)
        self.splitter.setHandleWidth(3)
        self.splitter.setStyleSheet("QSplitter::handle {background-color: #3A3C3E;}")
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
        pass  # Simple in-memory storage
    
    
    def _add_to_recent_files(self, file_path):
        if file_path in self.recent_files:
            self.recent_files.remove(file_path)
        self.recent_files.insert(0, file_path)
        self.recent_files = self.recent_files[:10]
        if hasattr(self, 'recent_menu'):
            self._update_recent_files_menu()
    
    def _update_recent_files_menu(self):
        if not hasattr(self, 'recent_menu'):
            return
        self.recent_menu.clear()
        if not self.recent_files:
            self.recent_menu.addAction("(No recent files)").setEnabled(False)
        else:
            for file_path in self.recent_files:
                action = self.recent_menu.addAction(os.path.basename(file_path))
                action.triggered.connect(lambda checked, path=file_path: self._open_recent_file(path))
    
    def _open_recent_file(self, file_path):
        if os.path.exists(file_path):
            self.log(f"Opening recent: {os.path.basename(file_path)}")
            self.create_embedded_pdf_viewer(file_path)
            self.current_pdf_path = file_path
        else:
            self.recent_files.remove(file_path)
            self._update_recent_files_menu()
            QMessageBox.warning(self, "File Not Found", f"Recent file not found:\n{file_path}")
    
    
    
    
    def export_to_sql(self):
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
            # Use context manager for safe resource cleanup
            with ChonkerSQLExporter(file_path) as exporter:
                # Export the content
                export_id = exporter.export_content(
                    content_html,
                    source_pdf,
                    qc_user=os.getenv('USER', 'user')
                )
                
                progress.setValue(75)
                
                # Get export statistics
                stats = exporter.conn.execute(
                    "SELECT COUNT(*) as total_elements, COUNT(DISTINCT element_type) as unique_types "
                    "FROM chonker_content WHERE export_id = ?", 
                    [export_id]
                ).fetchone()
                
                progress.setValue(50)
                
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
                
                # Now also create Arrow dataset export if PyArrow is available
                arrow_path = None
                arrow_elements = 0
                if PYARROW_AVAILABLE:
                    progress.setLabelText("Creating Arrow dataset...")
                    progress.setValue(60)
                    
                    # Create Arrow dataset in the export directory
                    arrow_dataset_path = os.path.join(export_dir, "arrow_dataset")
                    arrow_exporter = ArrowDatasetExporter(arrow_dataset_path)
                    
                    # Generate document ID (same as export_id for consistency)
                    doc_id = export_id
                    
                    # Export to Arrow format
                    arrow_result = arrow_exporter.export_document_dataset(
                        doc_id=doc_id,
                        html_content=content_html,
                        source_pdf=source_pdf
                    )
                    
                    # Extract element count from result
                    import re
                    match = re.search(r'Exported (\d+) elements', arrow_result)
                    if match:
                        arrow_elements = int(match.group(1))
                    
                    arrow_path = arrow_dataset_path
                    
                    # Create example query script
                    example_script = f'''#!/usr/bin/env python3
"""
Query examples for the exported data
Generated by CHONKER on {datetime.now().isoformat()}

This export contains both:
1. DuckDB database: {os.path.basename(file_path)}
2. Arrow dataset: arrow_dataset/
"""

# === OPTION 1: Query with DuckDB ===
import duckdb
conn = duckdb.connect("{os.path.basename(file_path)}")

# Get all content
content_df = conn.execute("SELECT * FROM chonker_content WHERE export_id = '{export_id}'").df()
print(f"Found {{len(content_df)}} elements in DuckDB")

# === OPTION 2: Query with PyArrow ===
import pyarrow.dataset as ds
dataset = ds.dataset("arrow_dataset", format="parquet", partitioning="hive")

# Find bold headers
bold_headers = dataset.to_table(
    filter=(ds.field("style_bold") == True) & 
           (ds.field("semantic_role") == "header")
).to_pandas()
print(f"\\nFound {{len(bold_headers)}} bold headers in Arrow dataset")

# === OPTION 3: Combine both sources ===
# DuckDB has edit history, Arrow has style metadata
# You can join them on element_id for complete picture
'''
                    
                    with open(os.path.join(export_dir, "query_examples.py"), 'w') as f:
                        f.write(example_script)
                    os.chmod(os.path.join(export_dir, "query_examples.py"), 0o755)
                
                progress.setValue(100)
                progress.close()
                
                # Show success message
                summary = f"Export Complete!\n\n"
                summary += f"Export ID: {export_id}\n\n"
                summary += f"✓ {stats[0]} content elements in DuckDB\n"
                if arrow_path:
                    summary += f"✓ {arrow_elements} elements in Arrow dataset\n"
                summary += f"✓ {stats[1]} different element types\n\n"
                summary += f"Files created:\n"
                summary += f"  • Database: {os.path.basename(file_path)}\n"
                if arrow_path:
                    summary += f"  • Arrow dataset: {os.path.basename(export_dir)}/arrow_dataset/\n"
                summary += f"  • Parquet files: {os.path.basename(export_dir)}/*.parquet\n"
                summary += f"  • Query examples: {os.path.basename(export_dir)}/query_examples.py\n\n"
                summary += "All your data in one place! Query with SQL or PyArrow."
                
                QMessageBox.information(self, "Export Successful", summary)
                self.log(f"Exported to: {file_path} (with Arrow dataset)")
            
        except Exception as e:
            progress.close()
            QMessageBox.critical(self, "Export Failed", f"Failed to export content:\n\n{str(e)}")
            self.log(f"Export failed: {str(e)}")
    
    def export_to_csv(self):
        if not hasattr(self, '_last_processing_result') or not self._last_processing_result:
            QMessageBox.warning(self, "No Document", "Please process a document first")
            return
        file_path, _ = QFileDialog.getSaveFileName(self, "Export CSV", "", "CSV files (*.csv)")
        if file_path:
            # Extract content to dataframe
            from bs4 import BeautifulSoup
            soup = BeautifulSoup(self._last_processing_result, 'html.parser')
            data = []
            for i, elem in enumerate(soup.find_all(['h1', 'h2', 'h3', 'p', 'table'])):
                data.append({'type': elem.name, 'content': elem.get_text().strip()})
            df = pd.DataFrame(data)
            df.to_csv(file_path, index=False)
            self.log(f"Exported to CSV: {file_path}")
    
    def dragEnterEvent(self, event):
        if event.mimeData().hasUrls():
            event.acceptProposedAction()
    
    def dropEvent(self, event):
        files = [u.toLocalFile() for u in event.mimeData().urls()]
        pdf_files = [f for f in files if f.endswith('.pdf')]
        if pdf_files:
            self.create_embedded_pdf_viewer(pdf_files[0])
            self.current_pdf_path = pdf_files[0]
    
    def keyPressEvent(self, event):
        if event.key() == Qt.Key.Key_Question and event.modifiers() & Qt.KeyboardModifier.ShiftModifier:
            QMessageBox.information(self, "Keyboard Shortcuts",
                "Cmd+O: Open PDF\n"
                "Cmd+P: Process\n"
                "Cmd+E: Export (SQL + Arrow)\n"
                "Cmd+Plus/Minus: Zoom\n"
                "Shift+?: This help")
        elif event.key() == Qt.Key.Key_F and event.modifiers() & Qt.KeyboardModifier.ControlModifier:
            self.simple_find()
        super().keyPressEvent(event)
    
    def simple_find(self):
        text, ok = QInputDialog.getText(self, "Find", "Search for:")
        if ok and text:
            cursor = self.faithful_output.document().find(text)
            if not cursor.isNull():
                self.faithful_output.setTextCursor(cursor)
    
    def save_geometry(self):
        settings = QSettings('Chonker', 'Window')
        settings.setValue('geometry', self.saveGeometry())
    
    def restore_geometry(self):
        settings = QSettings('Chonker', 'Window')
        if settings.value('geometry'):
            self.restoreGeometry(settings.value('geometry'))
    
    def _create_menu_bar(self):
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
        
        export_sql_action = QAction("Export to SQL", self)
        export_sql_action.setShortcut(QKeySequence("Ctrl+E"))
        export_sql_action.triggered.connect(self.export_to_sql)
        file_menu.addAction(export_sql_action)
        
        export_csv_action = QAction("Export to CSV", self)
        export_csv_action.triggered.connect(self.export_to_csv)
        file_menu.addAction(export_csv_action)
        
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
        chonker_action.triggered.connect(lambda: self.set_mode())
        view_menu.addAction(chonker_action)
    
    def _create_top_bar(self, parent_layout):
        self.top_bar = QWidget()  # Store as instance variable
        self.top_bar.setMinimumHeight(60)
        self.top_bar.setMaximumHeight(150)  # Allow vertical resizing
        self.top_bar.setObjectName("topBar")
        self.top_bar.setStyleSheet("#topBar {background-color: #1ABC9C; border: 2px solid #3A3C3E; border-radius: 2px;}")
        
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
        self.chonker_btn.setStyleSheet("QPushButton {background-color: #1ABC9C; color: white; font-size: 18px; font-weight: bold; border: none; padding: 5px;} QPushButton:hover {background-color: #16A085;}")
        self.chonker_btn.clicked.connect(lambda: self.set_mode())
        
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
        self.terminal.setStyleSheet("QTextEdit {background-color: #2D2F31; color: #1ABC9C; font-family: 'Courier New', monospace; font-size: 11px; border: 1px solid #3A3C3E; padding: 2px;}")
        
        # Add terminal directly to main layout (no expand button)
        layout.addWidget(self.terminal)
        
        # Quick actions - subordinate visual style
        action_button_style = "QPushButton{background:#3A3C3E;color:#B0B0B0;font-size:12px;border:1px solid #525659;padding:6px 10px;min-height:30px;max-height:35px}QPushButton:hover{background:#525659;color:#FFF}QPushButton:disabled{background:#2D2F31;color:#666}"
        
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
        # Clear left pane
        for i in reversed(range(self.left_layout.count())): 
            widget = self.left_layout.itemAt(i).widget()
            if widget:
                widget.setParent(None)
        
        # Show shortcuts in terminal
        self.log("Cmd+O: Open PDF | Cmd+P: Process | Cmd+E: Export")
    
    def _update_pane_styles(self):
        pass  # Simplified for space
    
    def zoom(self, delta: int):
        if self.active_pane == 'right':
            self.text_zoom = max(8, min(48, self.text_zoom + (2 if delta > 0 else -2)))
            self._apply_text_zoom()
        elif self.active_pane == 'left' and hasattr(self, 'embedded_pdf_view') and self.embedded_pdf_view:
            factor = 1.1 if delta > 0 else 0.9
            self.pdf_zoom = max(0.1, min(5.0, self.pdf_zoom * factor))
            self._apply_pdf_zoom()
    
    def zoom_in(self): self.zoom(1)
    def zoom_out(self): self.zoom(-1)
    
    
    def eventFilter(self, obj, event):
        handlers = {
            QEvent.Type.NativeGesture: self._handle_native_gesture,
            QEvent.Type.Enter: self._handle_enter_event,
            QEvent.Type.Wheel: self._handle_wheel_event
        }
        handler = handlers.get(event.type())
        if handler and handler(obj, event):
            return True
        return super().eventFilter(obj, event)
    
    def _handle_native_gesture(self, obj, event):
        if 'ZoomNativeGesture' in str(event.gestureType()):
            if obj == self.faithful_output or obj == self.embedded_pdf_view:
                zoom_delta = event.value()
                self._handle_gesture_zoom(zoom_delta, 1.0 + zoom_delta * 0.5)
                return True
        return False
    
    def _handle_enter_event(self, obj, event):
        pane_map = {
            self.left_pane: 'left',
            self.faithful_output: 'right'
        }
        if hasattr(self, 'embedded_pdf_view') and self.embedded_pdf_view:
            pane_map[self.embedded_pdf_view] = 'left'
        if hasattr(self, 'top_bar'):
            pane_map[self.top_bar] = 'top'
        
        new_pane = pane_map.get(obj)
        if new_pane and self.active_pane != new_pane:
            self.active_pane = new_pane
            self._update_pane_styles()
        return False
    
    def _handle_wheel_event(self, obj, event):
        if hasattr(event, 'modifiers') and event.modifiers() & Qt.KeyboardModifier.ControlModifier:
            delta = event.angleDelta().y()
            if obj == self.faithful_output:
                self.text_zoom = max(8, min(48, self.text_zoom + (1 if delta > 0 else -1)))
                self._apply_text_zoom()
                return True
            elif hasattr(self, 'embedded_pdf_view') and obj == self.embedded_pdf_view:
                factor = 1.1 if delta > 0 else 0.9
                new_zoom = max(0.1, min(5.0, self.pdf_zoom * factor))
                if new_zoom != self.pdf_zoom:
                    self.pdf_zoom = new_zoom
                    self._apply_pdf_zoom()
                return True
        return False
    
    def _apply_theme(self):
        bg1, bg2, bg3 = "#525659", "#3A3C3E", "#1E1E1E"
        c1, c2 = "#1ABC9C", "#16A085" 
        self.setStyleSheet(f"* {{color: #FFFFFF}} QMainWindow, QTextEdit {{background-color: {bg1}}} #topBar {{background-color: {bg1}; border-bottom: 1px solid {bg2}}} QPushButton {{background-color: #6B6E71; border: 1px solid #4A4C4E; border-radius: 4px; padding: 8px 16px; font-size: 14px}} QPushButton:hover {{background-color: #7B7E81; border-color: #5A5C5E}} QPushButton:checked {{background-color: {c1}; border-color: {c2}}} #terminal {{background-color: {bg3}; color: {c1}; font-family: 'Courier New', monospace; font-size: 11px; border: 1px solid #333; border-radius: 4px; padding: 4px}} QTextEdit {{border: 1px solid {bg2}; border-radius: 4px}} QScrollBar {{background-color: {bg2}}} QScrollBar:vertical {{width: 12px}} QScrollBar:horizontal {{height: 12px}} QScrollBar::handle {{background-color: {c1}; border: none}} QScrollBar::handle:vertical {{min-height: 20px}} QScrollBar::handle:horizontal {{min-width: 20px}} QScrollBar::handle:hover {{background-color: {c2}}} QScrollBar::add-line, QScrollBar::sub-line {{border: none; background: none; height: 0; width: 0}} #terminalExpandBtn {{background-color: transparent; border: 1px solid {bg2}; border-radius: 2px; padding: 0; font-size: 10px; color: {c1}}} #terminalExpandBtn:hover {{background-color: {bg2}}}")
    
    
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
    
    def set_mode(self):
        # Always CHONKER mode now
        self.chonker_btn.setChecked(True)
        self.log("CHONKER mode activated - Ready to process PDFs!")
    
    def _on_ocr_needed(self, file_path: str):
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
        cursor = self.terminal.textCursor()
        cursor.movePosition(QTextCursor.MoveOperation.End)
        cursor.insertText(f"[{datetime.now().strftime('%H:%M:%S')}] {message}\n")
        if self.terminal.document().blockCount() > 100:
            cursor.movePosition(QTextCursor.MoveOperation.Start)
            cursor.movePosition(QTextCursor.MoveOperation.Down, QTextCursor.MoveMode.KeepAnchor, self.terminal.document().blockCount() - 100)
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
        # Clear left pane
        for i in reversed(range(self.left_layout.count())): 
            widget = self.left_layout.itemAt(i).widget()
            if widget:
                widget.setParent(None)
        # Clean up old PDF view
        if hasattr(self, 'embedded_pdf_view') and self.embedded_pdf_view:
            self.embedded_pdf_view = None
        # Create PDF viewer
        self.embedded_pdf_view = QPdfView(self.left_pane)
        pdf_document = QPdfDocument(self.left_pane)
        self.embedded_pdf_view.setDocument(pdf_document)
        pdf_document.load(file_path)
        self.embedded_pdf_view.setStyleSheet("QPdfView {background-color: #525659; border: none;}")
        self.embedded_pdf_view.setPageMode(QPdfView.PageMode.MultiPage)
        self.embedded_pdf_view.setZoomMode(QPdfView.ZoomMode.Custom)
        
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
                except AttributeError:
                    pass
        
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
        if hasattr(self.embedded_pdf_view, 'selectedText'):
            selected_text = self.embedded_pdf_view.selectedText()
            if selected_text:
                self.log(f"PDF selection: {selected_text[:50]}...")
                # Trigger selection sync
    
    def _update_processing_animation(self):
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
            # Create new processor
            self.processor = DocumentProcessor(file_path)
            
            # Connect signals (no need to disconnect - it's a new object)
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
        # Apply zoom to the base HTML
        zoom_size = self.text_zoom
        
        html = (
            f'<!DOCTYPE html><html><head><style>'
            f'body{{font-family:-apple-system,sans-serif;margin:20px;color:#FFF;background:#525659;font-size:{zoom_size}px!important}}'
            f'table{{border-collapse:collapse;margin:15px 0;border:1px solid #3A3C3E;background:#3A3C3E;font-size:{zoom_size}px!important}}'
            f'th,td{{border:1px solid #525659;padding:8px;color:#FFF;background:#424548;font-size:{zoom_size}px!important}}'
            f'th{{background:#3A3C3E;font-weight:bold}}td[contenteditable="true"]:hover{{background:#525659}}.table-controls{{margin:10px 0}}'
            f'button {{ background: #1ABC9C; color: white; border: none; padding: 5px 10px; margin: 5px; border-radius: 3px; cursor: pointer; }}'
            f'button:hover {{ background: #16A085; }}'
            f'h1 {{ color: #1ABC9C; font-size: {int(zoom_size * 1.5)}px !important; }}'
            f'h2 {{ color: #1ABC9C; font-size: {int(zoom_size * 1.3)}px !important; }}'
            f'h3 {{ color: #1ABC9C; font-size: {int(zoom_size * 1.2)}px !important; }}'
            f'p {{ color: #FFFFFF; font-size: {zoom_size}px !important; }}'
            f'li {{ color: #FFFFFF; font-size: {zoom_size}px !important; }}'
            f'div {{ font-size: {zoom_size}px !important; }}'
            f'span {{ font-size: {zoom_size}px !important; }}'
            f'</style></head><body>'
            f'<h2 style="color: #1ABC9C;">CHONKER\'s Faithful Output</h2>'
            f'<div style="color: #B0B0B0;">Document ID: {result.document_id}</div>'
            f'<div style="color: #B0B0B0;">Processing Time: {result.processing_time:.2f}s</div>'
            f'<hr style="border-color: #3A3C3E;">'
            f'{result.html_content}'
            f'</body></html>'
        )
        self.faithful_output.setHtml(html)
        # Store the result for re-rendering on zoom changes
        self._last_processing_result = result
    
    def create_output_window(self, result: ProcessingResult):
        window = QWidget()
        window.setWindowTitle("Processed Output")
        window.resize(900, 800)
        
        layout = QVBoxLayout(window)
        
        # Output view
        output_view = QTextEdit()
        output_view.setReadOnly(False)
        
        # Build HTML with styles
        html = (
            f'<!DOCTYPE html><html><head><style>'
            f'body {{ font-family: -apple-system, sans-serif; margin: 20px; color: #000000; background: #FFFFFF; }}'
            f'table {{ border-collapse: collapse; margin: 15px 0; border: 1px solid #888888; }}'
            f'th, td {{ border: 1px solid #888888; padding: 8px; color: #000000; }}'
            f'.table-controls {{ margin: 10px 0; }}'
            f'button {{ background: #28a745; color: white; border: none; padding: 5px 10px; margin: 5px; border-radius: 3px; cursor: pointer; }}'
            f'</style></head><body>'
            f'<h2 style="color: #1ABC9C;">CHONKER\'s Output</h2>'
            f'{result.html_content}'
            f'</body></html>'
        )
        
        output_view.setHtml(html)
        layout.addWidget(output_view)
        
        window.show()
    
    def changeEvent(self, event):
        if event.type() == QEvent.Type.ApplicationStateChange:
            if QApplication.applicationState() == Qt.ApplicationState.ApplicationActive:
                # Bring to front when app is activated (dock click)
                self.raise_()
                self.activateWindow()
        super().changeEvent(event)
    
    def closeEvent(self, event):
        self.save_geometry()
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
        
        event.accept()



try:
    import duckdb
    import pandas as pd
    DUCKDB_AVAILABLE = True
except ImportError:
    DUCKDB_AVAILABLE = False


class ArrowDatasetExporter:
    """Export documents to partitioned PyArrow datasets for efficient querying"""
    
    def __init__(self, base_path: str = "chonker_arrow_export"):
        if not PYARROW_AVAILABLE:
            raise ImportError("🐹 *cough* PyArrow not installed! Run: uv pip install pyarrow")
        
        self.base_path = base_path
        os.makedirs(base_path, exist_ok=True)
    
    def _group_by_type(self, elements: List[Dict]) -> Dict[str, List[Dict]]:
        """Group elements by their type"""
        grouped = {}
        for element in elements:
            element_type = element.get('type', 'unknown')
            if element_type not in grouped:
                grouped[element_type] = []
            grouped[element_type].append(element)
        return grouped
    
    def _extract_style_info(self, element_html: str) -> Dict[str, Any]:
        """Extract style information from HTML element"""
        from bs4 import BeautifulSoup
        
        style_info = {
            'bold': False,
            'italic': False,
            'underline': False,
            'font_size': None,
            'font_family': None,
            'color': None,
            'background_color': None
        }
        
        if not element_html:
            return style_info
        
        soup = BeautifulSoup(element_html, 'html.parser')
        element = soup.find()
        
        if element:
            # Check for bold
            if element.name in ['b', 'strong'] or element.find(['b', 'strong']):
                style_info['bold'] = True
            
            # Check for italic
            if element.name in ['i', 'em'] or element.find(['i', 'em']):
                style_info['italic'] = True
            
            # Check for underline
            if element.name == 'u' or element.find('u'):
                style_info['underline'] = True
            
            # Extract inline styles
            style_attr = element.get('style', '')
            if style_attr:
                styles = dict(item.split(':') for item in style_attr.split(';') 
                             if ':' in item)
                style_info['font_size'] = styles.get('font-size', '').strip()
                style_info['font_family'] = styles.get('font-family', '').strip()
                style_info['color'] = styles.get('color', '').strip()
                style_info['background_color'] = styles.get('background-color', '').strip()
        
        return style_info
    
    def _create_rich_table(self, elements: List[Dict], doc_id: str, element_type: str) -> pa.Table:
        """Create a PyArrow table with rich metadata for elements"""
        # Prepare data for table
        data = {
            'doc_id': [],
            'element_type': [],
            'element_id': [],
            'element_order': [],
            'plain_text': [],
            'html_content': [],
            'semantic_role': [],
            'confidence_score': [],
            'char_count': [],
            'word_count': [],
            # Style fields
            'style_bold': [],
            'style_italic': [],
            'style_underline': [],
            'style_font_size': [],
            'style_font_family': [],
            'style_color': [],
            'style_background_color': [],
            # Structural fields
            'parent_id': [],
            'depth_level': [],
            'is_list_item': [],
            'list_position': [],
            # Table-specific fields (null for non-tables)
            'table_rows': [],
            'table_cols': [],
            'table_cell_content': [],
            # Timestamps
            'extracted_at': [],
            'modified_at': []
        }
        
        timestamp = pd.Timestamp.now()
        
        for i, element in enumerate(elements):
            # Extract style information
            style_info = self._extract_style_info(element.get('html', ''))
            
            # Basic fields
            data['doc_id'].append(doc_id)
            data['element_type'].append(element_type)
            data['element_id'].append(element.get('id', f"{doc_id}_{element_type}_{i:04d}"))
            data['element_order'].append(element.get('order', i))
            
            # Text content
            plain_text = element.get('text', '')
            data['plain_text'].append(plain_text)
            data['html_content'].append(element.get('html', ''))
            data['char_count'].append(len(plain_text))
            data['word_count'].append(len(plain_text.split()))
            
            # Semantic analysis
            semantic_role = self._infer_semantic_role(element, element_type)
            data['semantic_role'].append(semantic_role)
            data['confidence_score'].append(element.get('confidence', 1.0))
            
            # Style fields
            data['style_bold'].append(style_info['bold'])
            data['style_italic'].append(style_info['italic'])
            data['style_underline'].append(style_info['underline'])
            data['style_font_size'].append(style_info['font_size'])
            data['style_font_family'].append(style_info['font_family'])
            data['style_color'].append(style_info['color'])
            data['style_background_color'].append(style_info['background_color'])
            
            # Structural fields
            data['parent_id'].append(element.get('parent_id'))
            data['depth_level'].append(element.get('depth', 0))
            data['is_list_item'].append(element_type in ['li', 'ul', 'ol'])
            data['list_position'].append(element.get('list_position'))
            
            # Table-specific fields
            if element_type == 'table':
                data['table_rows'].append(element.get('rows'))
                data['table_cols'].append(element.get('cols'))
                data['table_cell_content'].append(json.dumps(element.get('cells', [])))
            else:
                data['table_rows'].append(None)
                data['table_cols'].append(None)
                data['table_cell_content'].append(None)
            
            # Timestamps
            data['extracted_at'].append(timestamp)
            data['modified_at'].append(timestamp)
        
        # Create PyArrow table with explicit schema
        schema = pa.schema([
            ('doc_id', pa.string()),
            ('element_type', pa.string()),
            ('element_id', pa.string()),
            ('element_order', pa.int32()),
            ('plain_text', pa.string()),
            ('html_content', pa.string()),
            ('semantic_role', pa.string()),
            ('confidence_score', pa.float64()),
            ('char_count', pa.int32()),
            ('word_count', pa.int32()),
            ('style_bold', pa.bool_()),
            ('style_italic', pa.bool_()),
            ('style_underline', pa.bool_()),
            ('style_font_size', pa.string()),
            ('style_font_family', pa.string()),
            ('style_color', pa.string()),
            ('style_background_color', pa.string()),
            ('parent_id', pa.string()),
            ('depth_level', pa.int32()),
            ('is_list_item', pa.bool_()),
            ('list_position', pa.int32()),
            ('table_rows', pa.int32()),
            ('table_cols', pa.int32()),
            ('table_cell_content', pa.string()),
            ('extracted_at', pa.timestamp('ns')),
            ('modified_at', pa.timestamp('ns'))
        ])
        
        return pa.Table.from_pydict(data, schema=schema)
    
    def _infer_semantic_role(self, element: Dict, element_type: str) -> str:
        """Infer the semantic role of an element"""
        text = element.get('text', '').lower()
        
        # Headers
        if element_type in ['h1', 'h2', 'h3', 'h4', 'h5', 'h6']:
            return 'header'
        
        # Tables
        if element_type == 'table':
            return 'data_table'
        
        # Lists
        if element_type in ['ul', 'ol', 'li']:
            return 'list'
        
        # Check text patterns for paragraphs
        if element_type == 'p':
            # Financial patterns
            if any(term in text for term in ['revenue', 'income', 'profit', 'loss', 'cost', 'expense', '$']):
                return 'financial_text'
            
            # Dates
            if re.search(r'\b\d{4}\b|\b\d{1,2}/\d{1,2}/\d{2,4}\b', text):
                return 'dated_text'
            
            # Questions
            if text.strip().endswith('?'):
                return 'question'
            
            # Short text might be captions
            if len(text.split()) < 10:
                return 'caption'
        
        return 'body_text'
    
    def export_document_dataset(self, doc_id: str, html_content: str, source_pdf: str = None) -> str:
        """Export a document as a partitioned Arrow dataset"""
        from bs4 import BeautifulSoup
        
        # Parse HTML content
        soup = BeautifulSoup(html_content, 'html.parser')
        
        # Extract all elements with metadata
        elements = []
        element_order = 0
        
        for element in soup.find_all(['h1', 'h2', 'h3', 'h4', 'h5', 'h6', 'p', 'table', 'ul', 'ol', 'li', 'div']):
            if not element.get_text(strip=True):
                continue
            
            element_data = {
                'type': element.name,
                'text': element.get_text(strip=True),
                'html': str(element),
                'order': element_order,
                'id': f"{doc_id}_{element_order:04d}"
            }
            
            # Add table-specific metadata
            if element.name == 'table':
                rows = element.find_all('tr')
                element_data['rows'] = len(rows)
                element_data['cols'] = len(rows[0].find_all(['td', 'th'])) if rows else 0
                
                # Extract cell content
                cells = []
                for row in rows:
                    row_cells = [cell.get_text(strip=True) for cell in row.find_all(['td', 'th'])]
                    cells.append(row_cells)
                element_data['cells'] = cells
            
            elements.append(element_data)
            element_order += 1
        
        # Group elements by type
        grouped_elements = self._group_by_type(elements)
        
        # Export each element type as a partition
        export_count = 0
        for element_type, typed_elements in grouped_elements.items():
            if not typed_elements:
                continue
            
            # Create rich table with metadata
            table = self._create_rich_table(typed_elements, doc_id, element_type)
            
            # Write dataset with partitioning
            ds.write_dataset(
                table,
                self.base_path,
                format="parquet",
                partitioning=ds.partitioning(
                    pa.schema([
                        ("doc_id", pa.string()),
                        ("element_type", pa.string())
                    ])
                ),
                existing_data_behavior="overwrite_or_ignore",
                use_threads=True
            )
            export_count += len(typed_elements)
        
        # Create metadata file
        metadata = {
            'doc_id': doc_id,
            'source_pdf': source_pdf,
            'export_time': datetime.now().isoformat(),
            'total_elements': len(elements),
            'element_types': {k: len(v) for k, v in grouped_elements.items()},
            'base_path': self.base_path
        }
        
        metadata_path = os.path.join(self.base_path, f"_{doc_id}_metadata.json")
        with open(metadata_path, 'w') as f:
            json.dump(metadata, f, indent=2)
        
        return f"Exported {export_count} elements to {self.base_path}"
    
    def query_dataset(self, filter_expression=None, columns=None):
        """Query the exported dataset"""
        dataset = ds.dataset(self.base_path, format="parquet", partitioning="hive")
        
        if filter_expression:
            return dataset.to_table(filter=filter_expression, columns=columns).to_pandas()
        else:
            return dataset.to_table(columns=columns).to_pandas()
    
    def find_bold_headers_about(self, keywords: List[str]) -> pd.DataFrame:
        """Example query: Find all bold headers containing specific keywords"""
        dataset = ds.dataset(self.base_path, format="parquet", partitioning="hive")
        
        # Build filter for keywords
        keyword_filters = None
        for keyword in keywords:
            kw_filter = ds.field("plain_text").isin([keyword])
            if keyword_filters is None:
                keyword_filters = kw_filter
            else:
                keyword_filters = keyword_filters | kw_filter
        
        # Combine with other conditions
        filter_expr = (
            (ds.field("style_bold") == True) & 
            (ds.field("semantic_role") == "header") &
            keyword_filters
        )
        
        return dataset.to_table(filter=filter_expr).to_pandas()


class ChonkerSQLExporter:
    
    def __init__(self, db_path: str = "chonker_output.duckdb", chunk_size_mb: int = 50):
        if not DUCKDB_AVAILABLE:
            raise ImportError("🐹 *cough* DuckDB not installed! Run: uv pip install duckdb pandas")
        
        self.db_path = db_path
        self.chunk_size_mb = chunk_size_mb * 1024 * 1024  # Convert to bytes
        self.conn = duckdb.connect(db_path)
        self._init_metadata_table()
        self.current_chunk = 0
        self.chunk_paths = []  # Track chunk DB paths
        self.current_size = 0
    
    def __enter__(self):
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        self.close()
    
    def close(self):
        if hasattr(self, 'conn') and self.conn:
            self.conn.close()
            self.conn = None
    
    def _init_metadata_table(self):
        self.conn.execute("CREATE TABLE IF NOT EXISTS chonker_exports ( export_id TEXT PRIMARY KEY, source_pdf TEXT, export_name TEXT, original_html TEXT, edited_html TEXT, content_type TEXT, content_hash TEXT, exported_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP, qc_user TEXT, edit_count INTEGER DEFAULT 0, metadata JSON )")
        
        # Create structured content table for parsed elements
        self.conn.execute("CREATE TABLE IF NOT EXISTS chonker_content ( content_id TEXT PRIMARY KEY, export_id TEXT, element_type TEXT, element_order INTEGER, element_text TEXT, element_html TEXT, element_metadata JSON, chunk_number INTEGER DEFAULT 0, FOREIGN KEY (export_id) REFERENCES chonker_exports(export_id) )")
        
        # Create chunk tracking table
        self.conn.execute("CREATE TABLE IF NOT EXISTS chonker_chunks ( chunk_id INTEGER PRIMARY KEY, export_id TEXT, chunk_path TEXT, start_element INTEGER, end_element INTEGER, chunk_size INTEGER, created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP, FOREIGN KEY (export_id) REFERENCES chonker_exports(export_id) )")
    
    def _create_chunk_db(self, export_id: str) -> duckdb.DuckDBPyConnection:
        self.current_chunk += 1
        chunk_path = self.db_path.replace('.duckdb', f'_chunk{self.current_chunk:03d}.duckdb')
        self.chunk_paths.append(chunk_path)
        
        chunk_conn = duckdb.connect(chunk_path)
        
        # Create content table in chunk
        chunk_conn.execute("CREATE TABLE chonker_content ( content_id TEXT PRIMARY KEY, export_id TEXT, element_type TEXT, element_order INTEGER, element_text TEXT, element_html TEXT, element_metadata JSON, chunk_number INTEGER )")
        
        # Track chunk in main DB
        self.conn.execute("INSERT INTO chonker_chunks (chunk_id, export_id, chunk_path, chunk_size) VALUES (?, ?, ?, 0)", [self.current_chunk, export_id, chunk_path])
        
        return chunk_conn
    
    def _infer_schema(self, html_table: str) -> tuple[pd.DataFrame, dict]:
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
            except (ValueError, TypeError):
                # Try datetime
                try:
                    df[col] = pd.to_datetime(df[col])
                    schema[col] = 'TIMESTAMP'
                except (ValueError, TypeError):
                    # Default to text
                    schema[col] = 'VARCHAR'
        
        return df, schema
    
    def export_content(self, html_content: str, source_pdf: str, 
                      export_name: str = None, qc_user: str = "user", 
                      content_type: str = "full_document") -> str:
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
            self.conn.execute("INSERT INTO chonker_exports (export_id, source_pdf, export_name, original_html, edited_html, content_type, content_hash, qc_user, metadata) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)", [
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
            current_conn = self.conn
            chunk_element_count = 0
            
            # Check if we need chunking based on HTML size
            needs_chunking = len(html_content.encode('utf-8')) > 200 * 1024 * 1024  # 200MB
            
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
                
                # Check if we need to create a new chunk
                element_size = len(element_text.encode('utf-8')) + len(element_html.encode('utf-8'))
                if needs_chunking and self.current_size + element_size > self.chunk_size_mb:
                    # Update chunk end element in main DB
                    if self.current_chunk > 0:
                        self.conn.execute("UPDATE chonker_chunks SET end_element = ?, chunk_size = ? WHERE chunk_id = ? AND export_id = ?", [element_order - 1, self.current_size, self.current_chunk, export_id])
                    
                    # Create new chunk
                    current_conn = self._create_chunk_db(export_id)
                    
                    # Update chunk start element
                    self.conn.execute("UPDATE chonker_chunks SET start_element = ? WHERE chunk_id = ? AND export_id = ?", [element_order, self.current_chunk, export_id])
                    
                    self.current_size = 0
                    chunk_element_count = 0
                
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
                
                # Insert structured content to appropriate database
                target_conn = current_conn if needs_chunking and self.current_chunk > 0 else self.conn
                target_conn.execute("INSERT INTO chonker_content (content_id, export_id, element_type, element_order, element_text, element_html, element_metadata, chunk_number) VALUES (?, ?, ?, ?, ?, ?, ?, ?)", [
                    content_id,
                    export_id,
                    element_type,
                    element_order,
                    element_text[:5000],  # Limit text size
                    element_html[:10000],  # Limit HTML size
                    json.dumps(element_meta),
                    self.current_chunk
                ])
                
                self.current_size += element_size
                chunk_element_count += 1
                element_order += 1
            
            # Update final chunk info
            if needs_chunking and self.current_chunk > 0:
                self.conn.execute("UPDATE chonker_chunks SET end_element = ?, chunk_size = ? WHERE chunk_id = ? AND export_id = ?", [element_order - 1, self.current_size, self.current_chunk, export_id])
                
                # Close chunk connection
                if current_conn != self.conn:
                    current_conn.close()
            
            # Update metadata with chunk info
            if self.chunk_paths:
                metadata['chunks'] = {
                    'total_chunks': len(self.chunk_paths),
                    'chunk_files': self.chunk_paths,
                    'chunk_size_mb': self.chunk_size_mb / (1024 * 1024)
                }
            
            # Update metadata
            self.conn.execute("UPDATE chonker_exports SET metadata = ? WHERE export_id = ?", [json.dumps(metadata), export_id])
            
            # Export to Parquet for portability (optional)
            export_base_dir = os.path.join(os.path.dirname(self.db_path), "exports")
            os.makedirs(export_base_dir, exist_ok=True)
            
            try:
                # Export main record
                export_df = self.conn.execute(
                    "SELECT * FROM chonker_exports WHERE export_id = ?", 
                    [export_id]
                ).df()
                export_df.to_parquet(os.path.join(export_base_dir, f"{export_id}_metadata.parquet"))
                
                # Export content
                content_df = self.conn.execute(
                    "SELECT * FROM chonker_content WHERE export_id = ? ORDER BY element_order", 
                    [export_id]
                ).df()
                content_df.to_parquet(os.path.join(export_base_dir, f"{export_id}_content.parquet"))
                parquet_exported = True
            except Exception:
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
            
            with open(os.path.join(export_base_dir, f"{export_id}.json"), 'w') as f:
                json.dump(export_json, f, indent=2)
            
            # Return export ID and parquet status
            self._parquet_exported = parquet_exported
            return export_id
        except Exception:
            raise
    
    def get_api_ready_json(self, table_name: str) -> str:
        result = self.conn.execute(f"SELECT * FROM {table_name}").fetchall()
        columns = [desc[0] for desc in self.conn.description]
        
        data = []
        for row in result:
            data.append(dict(zip(columns, row)))
        
        return json.dumps(data, indent=2, default=str)
    
    def generate_crud_sql(self, table_name: str) -> dict:
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


def main():
    def handle_exception(exc_type, exc_value, exc_traceback):
        if issubclass(exc_type, KeyboardInterrupt):
            sys.__excepthook__(exc_type, exc_value, exc_traceback)
            return
        print("🐹 Uncaught exception:")
        traceback.print_exception(exc_type, exc_value, exc_traceback)
    sys.excepthook = handle_exception
    print("CHONKER ready.")
    app = QApplication(sys.argv)
    app.setApplicationName("CHONKER")
    
    # Fix Apple system font warning
    font = app.font()
    font.setFamily("Helvetica Neue" if sys.platform == "darwin" else "Arial")
    app.setFont(font)
    window = ChonkerApp()
    if hasattr(window, 'chonker_pixmap'):
        app.setWindowIcon(QIcon(window.chonker_pixmap))
    window.show()
    sys.exit(app.exec())


if __name__ == "__main__":
    main()

