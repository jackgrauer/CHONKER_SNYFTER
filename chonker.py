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
import traceback
from html import escape
import time
import warnings
import pandas as pd
import re
import json
import shutil
import urllib.request
import urllib.parse
import ssl

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
    QInputDialog, QLineEdit, QListWidget, QListWidgetItem, QDialogButtonBox
)
from PyQt6.QtCore import (
    Qt, QThread, pyqtSignal, QTimer, QPointF, QObject, QEvent,
    QRect, QPropertyAnimation, QEasingCurve, QRectF, QSettings
)
from PyQt6.QtGui import (
    QAction, QKeySequence, QIcon, QPixmap, QPainter, QFont, QBrush, QColor, QTextCursor,
    QNativeGestureEvent, QTextDocument
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
# UI Constants
TEXT_ZOOM_MIN = 8
TEXT_ZOOM_MAX = 48
PDF_ZOOM_MIN = 0.25
PDF_ZOOM_MAX = 4.0
ANIMATION_INTERVAL = 500  # ms
THREAD_WAIT_TIMEOUT = 5000  # ms

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
    debug_messages: List[str] = field(default_factory=list)


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
            if not self.wait(THREAD_WAIT_TIMEOUT):  # Wait up to 5 seconds
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
                processing_time=processing_time,
                debug_messages=getattr(self, 'ocr_debug_messages', [])
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
            pipeline_options.do_table_structure = True  # Keep structure detection
            
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
        self.ocr_debug_messages = []
        self.ocr_debug_messages.append("🚀 ENTERING OCR MODE - Image preprocessing enabled!")
        self.progress.emit("🚀 ENTERING OCR MODE - Image preprocessing enabled!")
        try:
            from docling.document_converter import DocumentConverter
            from docling.datamodel.pipeline_options import PdfPipelineOptions, EasyOcrOptions, TesseractOcrOptions
            from docling.datamodel.base_models import InputFormat
            import fitz  # PyMuPDF
            import tempfile
            from PIL import Image, ImageEnhance, ImageFilter, ImageOps
            import io
            import numpy as np
            import cv2
            from scipy import ndimage
            
            # First, create a version of the PDF without text layer
            self.ocr_debug_messages.append("🔧 Preparing PDF for OCR...")
            self.progress.emit("🔧 Preparing PDF for OCR...")
            
            # Open the PDF
            pdf_doc = fitz.open(self.pdf_path)
            
            # Create a temporary PDF with only images (no text)
            with tempfile.NamedTemporaryFile(suffix='.pdf', delete=False) as tmp_pdf:
                tmp_path = tmp_pdf.name
                
            # Create new PDF with rendered pages as images
            new_doc = fitz.open()
            
            self.ocr_debug_messages.append(f"📄 Converting {len(pdf_doc)} pages to images...")
            self.ocr_debug_messages.append("🎨 Using PIL enhancements only: contrast, sharpness, edge enhance")
            
            for page_num in range(len(pdf_doc)):
                page = pdf_doc[page_num]
                self.ocr_debug_messages.append(f"📄 Processing page {page_num + 1}/{len(pdf_doc)}")
                
                # Render page as image at high resolution
                start_time = time.time()
                self.ocr_debug_messages.append(f"  🔍 Rendering page {page_num + 1} at 4x resolution...")
                mat = fitz.Matrix(4.0, 4.0)  # 4x scaling - good balance of quality and size
                pix = page.get_pixmap(matrix=mat, alpha=False)
                self.ocr_debug_messages.append(f"    ⏱️ Rendering took {time.time() - start_time:.2f}s")
                
                # Convert to PIL Image for enhancement
                self.ocr_debug_messages.append(f"  🖼️ Converting to PIL image...")
                img_data = pix.tobytes("png")
                img = Image.open(io.BytesIO(img_data))
                
                # Convert to RGB if necessary
                if img.mode != 'RGB':
                    self.ocr_debug_messages.append(f"  🎨 Converting from {img.mode} to RGB...")
                    img = img.convert('RGB')
                
                # Enhance the image for better OCR
                enhance_start = time.time()
                self.ocr_debug_messages.append(f"  🎨 Applying simple PIL enhancements...")
                
                # 1. Increase contrast
                enhancer = ImageEnhance.Contrast(img)
                img = enhancer.enhance(1.8)  # 80% more contrast
                
                # 2. Increase sharpness
                enhancer = ImageEnhance.Sharpness(img)
                img = enhancer.enhance(2.5)  # More sharpness
                
                # 3. Convert to grayscale
                img = img.convert('L')
                
                # 4. Increase brightness slightly
                enhancer = ImageEnhance.Brightness(img)
                img = enhancer.enhance(1.1)  # 10% brighter
                
                # 5. Apply edge enhance filter
                img = img.filter(ImageFilter.EDGE_ENHANCE_MORE)
                
                # 6. Final sharpen
                img = img.filter(ImageFilter.SHARPEN)
                
                self.ocr_debug_messages.append(f"    ⏱️ Enhancements took {time.time() - enhance_start:.2f}s")
                
                # Convert back to bytes
                self.ocr_debug_messages.append(f"  💾 Saving enhanced image...")
                
                # Save debug image if first page
                if page_num == 0:
                    debug_img_path = "/tmp/chonker_preprocessed_page1.png"
                    img.save(debug_img_path, format='PNG', dpi=(600, 600))
                    self.ocr_debug_messages.append(f"  🔍 Debug: Saved preprocessed image to {debug_img_path}")
                
                img_buffer = io.BytesIO()
                # Save at 600 DPI for maximum quality
                img.save(img_buffer, format='PNG', dpi=(600, 600))
                enhanced_img_data = img_buffer.getvalue()
                
                # Create new page with same dimensions as original
                self.ocr_debug_messages.append(f"  📄 Creating new PDF page...")
                new_page = new_doc.new_page(width=page.rect.width, height=page.rect.height)
                
                # Insert the enhanced image
                self.ocr_debug_messages.append(f"  📎 Inserting image into PDF...")
                img_rect = new_page.rect
                new_page.insert_image(img_rect, stream=enhanced_img_data)
                self.ocr_debug_messages.append(f"  ✅ Page {page_num + 1} complete")
            
            # Save the image-only PDF
            new_doc.save(tmp_path)
            new_doc.close()
            pdf_doc.close()
            
            # Debug: Verify the new PDF has no text
            self.ocr_debug_messages.append("🔍 Verifying image-only PDF...")
            self.progress.emit("🔍 Verifying image-only PDF...")
            verify_doc = fitz.open(tmp_path)
            for page_num in range(len(verify_doc)):
                page = verify_doc[page_num]
                text = page.get_text()
                if text.strip():
                    msg = f"⚠️ Warning: Page {page_num + 1} still has text: {text[:50]}..."
                    self.ocr_debug_messages.append(msg)
                    self.progress.emit(msg)
                else:
                    msg = f"✅ Page {page_num + 1} has no text layer"
                    self.ocr_debug_messages.append(msg)
                    self.progress.emit(msg)
            verify_doc.close()
            
            # Just use default converter - it worked before!
            converter = DocumentConverter()
            self.ocr_debug_messages.append("🔧 Using default DocumentConverter")
            
            msg1 = f"🔍 Processing image-only PDF: {tmp_path}"
            msg2 = f"📊 Temp PDF size: {os.path.getsize(tmp_path) / 1024 / 1024:.2f} MB"
            self.ocr_debug_messages.append(msg1)
            self.ocr_debug_messages.append(msg2)
            self.progress.emit(msg1)
            self.progress.emit(msg2)
            
            try:
                # Convert the image-only PDF (no text layer to interfere)
                self.ocr_debug_messages.append("🚀 Starting OCR processing with docling...")
                result = converter.convert(tmp_path)
                self.ocr_debug_messages.append("✅ OCR processing completed!")
                
                # Save a copy for debugging
                debug_path = "/tmp/chonker_debug_image_only.pdf"
                shutil.copy(tmp_path, debug_path)
                debug_msg = f"🔍 Debug: Saved image-only PDF to {debug_path}"
                self.ocr_debug_messages.append(debug_msg)
                self.progress.emit(debug_msg)
                
                # Success message
                success_msg = "✅ OCR processing completed successfully!"
                self.ocr_debug_messages.append(success_msg)
                self.progress.emit(success_msg)
                
                # Clean up temporary file
                os.unlink(tmp_path)
                
                return result
            except Exception as e:
                # Clean up on error
                if os.path.exists(tmp_path):
                    os.unlink(tmp_path)
                raise e
        except Exception as e:
            error_msg = f"❌ OCR processing failed: {str(e)}"
            self.ocr_debug_messages.append(error_msg)
            self.ocr_debug_messages.append(f"❌ Error type: {type(e).__name__}")
            self.progress.emit(error_msg)
            # Fallback to regular processing
            return self._convert_document()
    
    def _convert_document(self):
        self.progress.emit("📄 Using REGULAR extraction (no OCR preprocessing)")
        from docling.document_converter import DocumentConverter
        
        max_retries = 3
        retry_delay = 1  # seconds
        
        # Check if we should use lazy loading
        if self._should_use_lazy_loading():
            self.progress.emit(f"🔄 Large PDF detected, using chunk-based processing...")
            return self._convert_document_lazy()
        
        # Standard processing for smaller files
        # Use default DocumentConverter - it handles format analysis automatically
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
        
        # Get all items
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
        # Get text content
        text = getattr(item, 'text', str(item))
        
        return DocumentChunk(
            index=index,
            type=type(item).__name__.lower().replace('item', ''),
            content=text,
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

class ProcessBatchDialog(QDialog):
    def __init__(self, parent=None):
        super().__init__(parent)
        self.setWindowTitle("🐹 CHONKER Batch Processing")
        self.setFixedSize(600, 400)
        
        layout = QVBoxLayout()
        
        # File list
        layout.addWidget(QLabel("📁 Selected PDFs for processing:"))
        self.file_list = QListWidget()
        layout.addWidget(self.file_list)
        
        # Add/Remove buttons
        button_layout = QHBoxLayout()
        self.add_btn = QPushButton("➕ Add PDFs")
        self.add_btn.clicked.connect(self.add_files)
        self.remove_btn = QPushButton("➖ Remove Selected")
        self.remove_btn.clicked.connect(self.remove_selected)
        self.clear_btn = QPushButton("🗑️ Clear All")
        self.clear_btn.clicked.connect(self.file_list.clear)
        
        button_layout.addWidget(self.add_btn)
        button_layout.addWidget(self.remove_btn)
        button_layout.addWidget(self.clear_btn)
        button_layout.addStretch()
        layout.addLayout(button_layout)
        
        # Output format selection
        format_layout = QHBoxLayout()
        format_layout.addWidget(QLabel("📄 Output format:"))
        self.output_format = QLineEdit("parquet")
        self.output_format.setPlaceholderText("parquet, html, or markdown")
        format_layout.addWidget(self.output_format)
        layout.addLayout(format_layout)
        
        # OCR mode checkbox
        self.ocr_checkbox = QPushButton("🔍 OCR Mode: OFF")
        self.ocr_checkbox.setCheckable(True)
        self.ocr_checkbox.toggled.connect(self.toggle_ocr_mode)
        layout.addWidget(self.ocr_checkbox)
        
        # Dialog buttons
        buttons = QDialogButtonBox(
            QDialogButtonBox.StandardButton.Ok | 
            QDialogButtonBox.StandardButton.Cancel
        )
        buttons.accepted.connect(self.accept)
        buttons.rejected.connect(self.reject)
        layout.addWidget(buttons)
        
        self.setLayout(layout)
        self.selected_files = []
        
    def toggle_ocr_mode(self, checked):
        if checked:
            self.ocr_checkbox.setText("🔍 OCR Mode: ON")
        else:
            self.ocr_checkbox.setText("🔍 OCR Mode: OFF")
    
    def add_files(self):
        files, _ = QFileDialog.getOpenFileNames(
            self,
            "Select PDF files",
            "",
            "PDF Files (*.pdf)"
        )
        
        for file in files:
            if file not in self.selected_files:
                self.selected_files.append(file)
                self.file_list.addItem(os.path.basename(file))
    
    def remove_selected(self):
        for item in self.file_list.selectedItems():
            idx = self.file_list.row(item)
            self.file_list.takeItem(idx)
            if idx < len(self.selected_files):
                self.selected_files.pop(idx)
    
    def get_settings(self):
        return {
            'files': self.selected_files,
            'output_format': self.output_format.text() or 'parquet',
            'use_ocr': self.ocr_checkbox.isChecked()
        }


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
        self._temp_files = []  # Track temp files for cleanup
        
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
        try:
            file_size_mb = os.path.getsize(file_path) / (1024 * 1024)
            if file_size_mb > MAX_FILE_SIZE / (1024 * 1024):
                msg = f"Cannot {action} file: {os.path.basename(file_path)}\nSize: {file_size_mb:.1f} MB (max: {MAX_FILE_SIZE / (1024 * 1024):.0f} MB)"
                if action == "open": 
                    QMessageBox.warning(self, "File Too Large", f"File is {file_size_mb:.0f}MB (max {MAX_FILE_SIZE/(1024*1024):.0f}MB)")
                self.log(f"❌ File too large: {file_size_mb:.1f} MB")
                return None
            return file_size_mb
        except OSError as e:
            if action == "open": 
                QMessageBox.critical(self, "File Error", "Cannot access file")
            self.log(f"❌ Cannot access file: {e}")
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
        
        # Main container widget to hold top bar and search widget
        top_container = QWidget()
        top_layout = QVBoxLayout(top_container)
        top_layout.setContentsMargins(0, 0, 0, 0)
        top_layout.setSpacing(0)
        
        # Main vertical splitter for top container and content
        self.main_splitter = QSplitter(Qt.Orientation.Vertical)
        self.main_splitter.setHandleWidth(3)
        self.main_splitter.setStyleSheet("QSplitter::handle {background-color: #3A3C3E;}")
        layout.addWidget(self.main_splitter)
        
        # Add the top container to the splitter
        self.main_splitter.addWidget(top_container)
        
        # Top bar (inside container)
        self._create_top_bar(top_layout)
        
        # Create search widget (hidden by default) and add it below top bar
        self._create_search_widget()
        top_layout.addWidget(self.search_widget)
        
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
    
    def _create_search_widget(self):
        """Create the search widget with input field, buttons, and match count"""
        self.search_widget = QWidget()
        self.search_widget.setObjectName("searchWidget")
        self.search_widget.setFixedHeight(40)
        
        # Create horizontal layout
        layout = QHBoxLayout(self.search_widget)
        layout.setContentsMargins(10, 5, 10, 5)
        layout.setSpacing(10)
        
        # Search input
        self.search_input = QLineEdit()
        self.search_input.setPlaceholderText("Find in document...")
        self.search_input.setObjectName("searchInput")
        self.search_input.returnPressed.connect(lambda: self.find_text(forward=True))
        self.search_input.textChanged.connect(self.update_match_count)
        layout.addWidget(self.search_input)
        
        # Match count label
        self.match_label = QLabel("")
        self.match_label.setObjectName("matchLabel")
        self.match_label.setMinimumWidth(80)
        layout.addWidget(self.match_label)
        
        # Previous button
        self.prev_button = QPushButton("Previous")
        self.prev_button.setObjectName("prevButton")
        self.prev_button.clicked.connect(lambda: self.find_text(forward=False))
        layout.addWidget(self.prev_button)
        
        # Next button  
        self.next_button = QPushButton("Next")
        self.next_button.setObjectName("nextButton")
        self.next_button.clicked.connect(lambda: self.find_text(forward=True))
        layout.addWidget(self.next_button)
        
        # Close button
        self.close_button = QPushButton("✕")
        self.close_button.setObjectName("closeButton")
        self.close_button.setFixedWidth(30)
        self.close_button.clicked.connect(self.close_search)
        layout.addWidget(self.close_button)
        
        # Hide by default
        self.search_widget.hide()
    
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
        # Check if it's a URL from recent files
        if file_path.startswith("[URL] "):
            url = file_path[6:]  # Remove "[URL] " prefix
            try:
                downloaded_path = self._download_pdf_from_url(url)
                self.create_embedded_pdf_viewer(downloaded_path)
                self.current_pdf_path = downloaded_path
            except Exception as e:
                QMessageBox.warning(self, "Download Failed", "Download failed")
        elif os.path.exists(file_path):
            self.log(f"Opening recent: {os.path.basename(file_path)}")
            self.create_embedded_pdf_viewer(file_path)
            self.current_pdf_path = file_path
        else:
            self.recent_files.remove(file_path)
            self._update_recent_files_menu()
            QMessageBox.warning(self, "File Not Found", "File not found")
    
    
    
    
    def export_to_parquet(self):
        if not hasattr(self, '_last_processing_result') or not self._last_processing_result:
            QMessageBox.warning(self, "No Document", "Please process a document first")
            return
        
        # Get suggested filename from source PDF
        source_pdf = self.current_pdf_path if hasattr(self, 'current_pdf_path') else "unknown.pdf"
        suggested_dir = os.path.basename(source_pdf).replace('.pdf', '_parquet')
        
        # Show directory selection dialog
        dir_path = QFileDialog.getExistingDirectory(
            self,
            "Select Directory for Parquet Export",
            os.path.expanduser("~"),
            QFileDialog.Option.ShowDirsOnly
        )
        
        if not dir_path:
            return  # User cancelled
        
        # Create export directory
        export_path = os.path.join(dir_path, suggested_dir)
        os.makedirs(export_path, exist_ok=True)
        
        # Get the current content from the faithful output (which may have been edited)
        current_html = self.faithful_output.toHtml()
        
        # Extract just the body content (remove Qt's HTML wrapper)
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
        progress = QProgressDialog("Exporting content to Parquet...", None, 0, 100, self)
        progress.setWindowModality(Qt.WindowModality.WindowModal)
        progress.setMinimumDuration(0)
        progress.setCancelButton(None)  # Can't cancel
        progress.setValue(10)
        
        try:
            if not PYARROW_AVAILABLE:
                raise ImportError("PyArrow not available. Install with: uv pip install pyarrow")
            
            # Generate export ID
            timestamp = datetime.now()
            export_id = f"{os.path.basename(source_pdf).replace('.pdf', '')}_{timestamp.strftime('%Y%m%d_%H%M%S')}"
            export_id = re.sub(r'[^a-zA-Z0-9_]', '_', export_id)
            
            # Parse HTML content
            soup = BeautifulSoup(content_html, 'html.parser')
            
            progress.setValue(20)
            
            # Prepare data for different tables
            content_data = []
            style_data = []
            semantic_data = []
            export_data = []
            
            # Export metadata
            export_data.append({
                'export_id': export_id,
                'source_pdf': source_pdf,
                'export_name': os.path.basename(export_path),
                'original_html': self._last_processing_result.html_content if hasattr(self._last_processing_result, 'html_content') else content_html,
                'edited_html': content_html,
                'content_type': 'full_document',
                'content_hash': hashlib.sha256(content_html.encode()).hexdigest()[:16],
                'exported_at': timestamp,
                'qc_user': os.getenv('USER', 'user'),
                'edit_count': 1 if content_html != self._last_processing_result.html_content else 0,
                'metadata': json.dumps({'chunks': len(self._last_processing_result.chunks) if hasattr(self._last_processing_result, 'chunks') else 0})
            })
                
            progress.setValue(30)
            
            # Extract content elements
            element_order = 0
            
            for element in soup.find_all(['h1', 'h2', 'h3', 'h4', 'h5', 'h6', 'p', 'table', 'ul', 'ol', 'li', 'div']):
                if not element.get_text(strip=True):
                    continue
                
                element_id = f"{export_id}_{element_order:04d}"
                content_id = f"content_{element_id}"
                
                # Extract page number from metadata if available
                page_num = 0
                if hasattr(self._last_processing_result, 'chunks') and element_order < len(self._last_processing_result.chunks):
                    page_num = self._last_processing_result.chunks[element_order].metadata.get('page', 0)
                
                # Content data
                content_data.append({
                    'content_id': content_id,
                    'export_id': export_id,
                    'element_type': element.name,
                    'element_order': element_order,
                    'element_text': element.get_text(strip=True),
                    'element_html': str(element),
                    'element_metadata': json.dumps({
                        'level': 0,
                        'page': page_num
                    }),
                    'chunk_number': 0
                })
                
                # Extract style information
                style_data.append({
                    'element_id': element_id,
                    'style_bold': element.name in ['b', 'strong'] or bool(element.find(['b', 'strong'])),
                    'style_italic': element.name in ['i', 'em'] or bool(element.find(['i', 'em'])),
                    'style_underline': element.name == 'u' or bool(element.find('u')),
                    'font_size': element.get('style', '').split('font-size:')[-1].split(';')[0].strip() if 'font-size:' in element.get('style', '') else None,
                    'color': element.get('style', '').split('color:')[-1].split(';')[0].strip() if 'color:' in element.get('style', '') else None
                })
                
                # Semantic analysis
                text = element.get_text(strip=True).lower()
                semantic_role = 'body_text'
                
                if element.name in ['h1', 'h2', 'h3', 'h4', 'h5', 'h6']:
                    semantic_role = 'header'
                elif element.name == 'table':
                    semantic_role = 'data_table'
                elif element.name in ['ul', 'ol', 'li']:
                    semantic_role = 'list'
                elif element.name == 'p':
                    if any(term in text for term in ['revenue', 'income', 'profit', 'loss', 'cost', 'expense', '$']):
                        semantic_role = 'financial_text'
                    elif re.search(r'\b\d{4}\b|\b\d{1,2}/\d{1,2}/\d{2,4}\b', text):
                        semantic_role = 'dated_text'
                    elif text.strip().endswith('?'):
                        semantic_role = 'question'
                    elif len(text.split()) < 10:
                        semantic_role = 'caption'
                
                semantic_data.append({
                    'element_id': element_id,
                    'semantic_role': semantic_role,
                    'confidence_score': 0.95,
                    'word_count': len(element.get_text(strip=True).split()),
                    'char_count': len(element.get_text(strip=True))
                })
                
                element_order += 1
            
            progress.setValue(60)
            
            # Convert to PyArrow tables and save as Parquet
            import pyarrow as pa
            import pyarrow.parquet as pq
            
            # Export metadata table
            export_table = pa.Table.from_pandas(pd.DataFrame(export_data))
            pq.write_table(export_table, os.path.join(export_path, 'chonker_exports.parquet'))
            
            # Content table
            content_table = pa.Table.from_pandas(pd.DataFrame(content_data))
            pq.write_table(content_table, os.path.join(export_path, 'chonker_content.parquet'))
            
            progress.setValue(80)
            
            # Style table
            style_table = pa.Table.from_pandas(pd.DataFrame(style_data))
            pq.write_table(style_table, os.path.join(export_path, 'chonker_styles.parquet'))
            
            # Semantics table
            semantic_table = pa.Table.from_pandas(pd.DataFrame(semantic_data))
            pq.write_table(semantic_table, os.path.join(export_path, 'chonker_semantics.parquet'))
            
            progress.setValue(100)
            progress.close()
            
            # Show success message
            QMessageBox.information(self, "Export Successful", 
                f"Exported {len(content_data)} elements to Parquet files in:\n{export_path}")
            self.log(f"Exported to Parquet: {export_path}")
            
        except Exception as e:
            progress.close()
            QMessageBox.critical(self, "Export Failed", "Export failed")
            self.log(f"Export failed: {str(e)}")
    
    def export_to_csv(self):
        if not hasattr(self, '_last_processing_result') or not self._last_processing_result:
            QMessageBox.warning(self, "No Document", "Please process a document first")
            return
        file_path, _ = QFileDialog.getSaveFileName(self, "Export CSV", "", "CSV files (*.csv)")
        if file_path:
            # Extract content to dataframe
            soup = BeautifulSoup(self._last_processing_result, 'html.parser')
            data = []
            for i, elem in enumerate(soup.find_all(['h1', 'h2', 'h3', 'p', 'table'])):
                data.append({'type': elem.name, 'content': elem.get_text().strip()})
            df = pd.DataFrame(data)
            df.to_csv(file_path, index=False)
            self.log(f"Exported to CSV: {file_path}")
    
    def _is_url(self, text: str) -> bool:
        """Check if text is a URL"""
        return text.startswith(('http://', 'https://', 'ftp://'))
    
    def _download_pdf_from_url(self, url: str) -> str:
        """Download PDF from URL to temp file"""
        try:
            self.log(f"📥 Downloading from {urllib.parse.urlparse(url).netloc}...")
            
            # Create SSL context
            ssl_context = ssl.create_default_context()
            
            # Headers to avoid bot detection
            headers = {
                'User-Agent': 'Mozilla/5.0 (Chonker PDF Processor)',
                'Accept': 'application/pdf,*/*'
            }
            
            request = urllib.request.Request(url, headers=headers)
            
            # Create temp file
            temp_file = tempfile.NamedTemporaryFile(suffix='.pdf', delete=False, prefix='chonker_')
            self._temp_files.append(temp_file.name)
            
            # Download with progress
            with urllib.request.urlopen(request, context=ssl_context) as response:
                # Check content type
                content_type = response.headers.get('Content-Type', '')
                if 'pdf' not in content_type.lower() and not url.endswith('.pdf'):
                    self.log("⚠️ Warning: URL may not be a PDF file")
                
                total_size = int(response.headers.get('Content-Length', 0))
                downloaded = 0
                block_size = 8192
                
                while True:
                    chunk = response.read(block_size)
                    if not chunk:
                        break
                    temp_file.write(chunk)
                    downloaded += len(chunk)
                    
                    if total_size:
                        percent = (downloaded / total_size) * 100
                        mb_downloaded = downloaded / (1024 * 1024)
                        self.log(f"📥 Downloaded: {mb_downloaded:.1f}MB ({percent:.0f}%)")
                
                temp_file.close()
                self.log(f"✅ Download complete: {os.path.basename(temp_file.name)}")
                return temp_file.name
                
        except Exception as e:
            self.log(f"❌ Download failed: {str(e)}")
            if 'temp_file' in locals() and hasattr(temp_file, 'name'):
                try:
                    os.unlink(temp_file.name)
                except:
                    pass
            raise Exception(f"Failed to download PDF: {str(e)}")
    
    
    def dragEnterEvent(self, event):
        if event.mimeData().hasUrls():
            event.acceptProposedAction()
    
    def dropEvent(self, event):
        urls = event.mimeData().urls()
        
        # Check if any are web URLs
        for url in urls:
            url_string = url.toString()
            if self._is_url(url_string):
                # Handle URL drop
                try:
                    file_path = self._download_pdf_from_url(url_string)
                    self.create_embedded_pdf_viewer(file_path)
                    self.current_pdf_path = file_path
                    self._add_to_recent_files(f"[URL] {url_string}")
                    return
                except Exception as e:
                    QMessageBox.warning(self, "Download Failed", str(e))
                    return
        
        # Otherwise handle as file drops
        files = [u.toLocalFile() for u in urls]
        pdf_files = [f for f in files if f.endswith('.pdf')]
        if pdf_files:
            self.create_embedded_pdf_viewer(pdf_files[0])
            self.current_pdf_path = pdf_files[0]
    
    def open_batch_dialog(self):
        dialog = ProcessBatchDialog(self)
        if dialog.exec() == QDialog.DialogCode.Accepted:
            settings = dialog.get_settings()
            if settings['files']:
                self.process_batch(settings['files'], settings['output_format'], settings['use_ocr'])
    
    def process_batch(self, pdf_files, output_format='parquet', use_ocr=False):
        """Process multiple PDF files in batch"""
        
        # Create output directory
        timestamp = datetime.now().strftime('%Y%m%d_%H%M%S')
        output_dir = os.path.join(os.path.expanduser("~"), f"chonker_batch_{timestamp}")
        os.makedirs(output_dir, exist_ok=True)
        
        # Show progress dialog
        progress = QProgressDialog("🐹 CHONKER Batch Processing...", "Cancel", 0, len(pdf_files), self)
        progress.setWindowTitle("Batch Processing")
        progress.setWindowModality(Qt.WindowModality.WindowModal)
        progress.setMinimumWidth(400)
        
        results = []
        
        for idx, pdf_path in enumerate(pdf_files):
            if progress.wasCanceled():
                break
                
            progress.setLabelText(f"Processing {os.path.basename(pdf_path)}...")
            progress.setValue(idx)
            
            # Create processor thread
            processor = DocumentProcessor(pdf_path, use_ocr=use_ocr)
            
            # Process synchronously for batch mode
            try:
                # Convert document
                if use_ocr:
                    result = processor._convert_document_with_ocr()
                else:
                    result = processor._convert_document()
                
                # Extract content
                chunks, html_content = processor._extract_content(result)
                
                # Save based on format
                base_name = os.path.basename(pdf_path).replace('.pdf', '')
                
                if output_format == 'html':
                    output_file = os.path.join(output_dir, f"{base_name}.html")
                    with open(output_file, 'w', encoding='utf-8') as f:
                        f.write(html_content)
                
                elif output_format == 'markdown':
                    output_file = os.path.join(output_dir, f"{base_name}.md")
                    markdown_content = result.document.export_to_markdown()
                    with open(output_file, 'w', encoding='utf-8') as f:
                        f.write(markdown_content)
                
                elif output_format == 'parquet':
                    if PYARROW_AVAILABLE:
                        # Create parquet structure
                        parquet_dir = os.path.join(output_dir, f"{base_name}_parquet")
                        os.makedirs(parquet_dir, exist_ok=True)
                        
                        # Convert chunks to dataframe
                        chunk_data = []
                        for chunk in chunks:
                            chunk_data.append({
                                'index': chunk.index,
                                'type': chunk.type,
                                'content': chunk.content,
                                'page': chunk.metadata.get('page', 0),
                                'level': chunk.metadata.get('level', 0)
                            })
                        
                        df = pd.DataFrame(chunk_data)
                        
                        # Save as parquet
                        pq.write_table(
                            pa.Table.from_pandas(df),
                            os.path.join(parquet_dir, 'chunks.parquet')
                        )
                        
                        # Save metadata
                        metadata = {
                            'source_pdf': pdf_path,
                            'processing_time': datetime.now().isoformat(),
                            'num_chunks': len(chunks),
                            'ocr_mode': use_ocr
                        }
                        
                        with open(os.path.join(parquet_dir, 'metadata.json'), 'w') as f:
                            json.dump(metadata, f, indent=2)
                        
                        output_file = parquet_dir
                    else:
                        output_file = os.path.join(output_dir, f"{base_name}.html")
                        with open(output_file, 'w', encoding='utf-8') as f:
                            f.write(html_content)
                
                results.append((pdf_path, 'success', output_file))
                
            except Exception as e:
                results.append((pdf_path, 'error', str(e)))
                self.terminal.appendPlainText(f"❌ Error processing {os.path.basename(pdf_path)}: {str(e)}")
        
        progress.setValue(len(pdf_files))
        
        # Show results summary
        success_count = sum(1 for _, status, _ in results if status == 'success')
        error_count = len(results) - success_count
        
        summary = f"🐹 Batch Processing Complete!\n\n"
        summary += f"✅ Success: {success_count} files\n"
        summary += f"❌ Errors: {error_count} files\n\n"
        summary += f"📁 Output directory: {output_dir}\n\n"
        
        if error_count > 0:
            summary += "Failed files:\n"
            for pdf_path, status, error in results:
                if status == 'error':
                    summary += f"  - {os.path.basename(pdf_path)}: {error}\n"
        
        QMessageBox.information(self, "Batch Processing Complete", summary)
        
        # Log to terminal
        self.terminal.appendPlainText(f"\n{summary}")
    
    def keyPressEvent(self, event):
        if event.key() == Qt.Key.Key_Question:
            QMessageBox.information(self, "Keyboard Shortcuts",
                "Cmd+O: Open PDF from File\n"
                "Cmd+U: Open PDF from URL\n"
                "Cmd+B: Batch Process PDFs\n"
                "Cmd+P: Process Document\n"
                "Cmd+E: Export to Parquet\n"
                "Cmd+F: Toggle Search\n"
                "Cmd+Plus/Minus: Zoom\n"
                "?: This help")
        elif event.key() == Qt.Key.Key_F and event.modifiers() & Qt.KeyboardModifier.ControlModifier:
            self.simple_find()
        super().keyPressEvent(event)
    
    def simple_find(self):
        """Toggle the search bar - show if hidden, hide if visible"""
        if hasattr(self, 'search_widget'):
            if self.search_widget.isHidden():
                self.search_widget.show()
                self.search_input.setFocus()
                self.search_input.selectAll()
            else:
                self.close_search()
    
    def find_text(self, forward=True):
        """Find text with direction support"""
        search_text = self.search_input.text()
        if not search_text:
            return
        
        # Get current cursor position
        cursor = self.faithful_output.textCursor()
        
        # Set search options
        options = QTextDocument.FindFlag(0)
        if not forward:
            options = QTextDocument.FindFlag.FindBackward
        
        # Search from current position
        found_cursor = self.faithful_output.document().find(search_text, cursor, options)
        
        if found_cursor.isNull():
            # Wrap around search
            if forward:
                # Start from beginning
                cursor.movePosition(QTextCursor.MoveOperation.Start)
            else:
                # Start from end
                cursor.movePosition(QTextCursor.MoveOperation.End)
            found_cursor = self.faithful_output.document().find(search_text, cursor, options)
        
        if not found_cursor.isNull():
            self.faithful_output.setTextCursor(found_cursor)
            self.update_match_count()
        else:
            self.match_label.setText("No matches")
    
    def update_match_count(self):
        """Update the match count display"""
        search_text = self.search_input.text()
        if not search_text:
            self.match_label.setText("")
            return
        
        # Count total matches
        doc = self.faithful_output.document()
        cursor = QTextCursor(doc)
        matches = []
        
        while True:
            cursor = doc.find(search_text, cursor)
            if cursor.isNull():
                break
            matches.append(cursor.position())
        
        total_matches = len(matches)
        
        if total_matches == 0:
            self.match_label.setText("No matches")
        else:
            # Find current match index
            current_cursor = self.faithful_output.textCursor()
            current_pos = current_cursor.position()
            
            current_match = 0
            for i, pos in enumerate(matches):
                if pos <= current_pos:
                    current_match = i + 1
                else:
                    break
            
            self.match_label.setText(f"{current_match} of {total_matches}")
    
    def close_search(self):
        """Hide the search widget"""
        # Disable updates to prevent flash
        self.setUpdatesEnabled(False)
        
        # Clear selection first
        cursor = self.faithful_output.textCursor()
        cursor.clearSelection()
        self.faithful_output.setTextCursor(cursor)
        
        # Clear fields
        self.search_input.clear()
        self.match_label.clear()
        
        # Hide widget
        self.search_widget.hide()
        
        # Re-enable updates
        self.setUpdatesEnabled(True)
    
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
        
        open_url_action = QAction("Open from URL", self)
        open_url_action.setShortcut("Ctrl+U")
        open_url_action.triggered.connect(self._toggle_url_input)
        file_menu.addAction(open_url_action)
        
        file_menu.addSeparator()
        
        # Add recent files menu
        self.recent_menu = file_menu.addMenu("Recent Files")
        self._update_recent_files_menu()
        
        process_action = QAction("Process Document", self)
        # Use Ctrl+P for all platforms for consistency
        process_action.setShortcut(QKeySequence("Ctrl+P"))
        process_action.triggered.connect(self.process_current)
        file_menu.addAction(process_action)
        
        file_menu.addSeparator()
        
        export_parquet_action = QAction("Export to Parquet", self)
        export_parquet_action.setShortcut(QKeySequence("Ctrl+E"))
        export_parquet_action.triggered.connect(self.export_to_parquet)
        file_menu.addAction(export_parquet_action)
        
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
        
        # Open from File button
        open_file_btn = QPushButton("Open File")
        open_file_btn.setToolTip("Open PDF from file (Ctrl+O)")
        open_file_btn.clicked.connect(self.open_pdf)
        open_file_btn.setShortcut(QKeySequence.StandardKey.Open)
        open_file_btn.setStyleSheet(action_button_style)
        
        # Open from URL button
        open_url_btn = QPushButton("Open URL")
        open_url_btn.setToolTip("Open PDF from URL (Ctrl+U)")
        open_url_btn.clicked.connect(self._toggle_url_input)
        open_url_btn.setShortcut(QKeySequence("Ctrl+U"))
        open_url_btn.setStyleSheet(action_button_style)
        self.open_url_btn = open_url_btn
        
        # URL input field (hidden by default)
        self.url_input = QLineEdit()
        self.url_input.setPlaceholderText("Enter PDF URL and press Enter...")
        self.url_input.setMaximumWidth(300)
        self.url_input.setMinimumWidth(250)
        self.url_input.hide()
        self.url_input.returnPressed.connect(self._handle_url_input)
        self.url_input.setStyleSheet("QLineEdit{background:#2D2F31;color:#1ABC9C;border:1px solid #1ABC9C;padding:5px;font-size:11px}QLineEdit:focus{border-color:#16A085}")
        
        process_btn = QPushButton("Extract to HTML")
        process_btn.setToolTip("Process (Ctrl+P)")
        process_btn.clicked.connect(self.process_current)
        process_btn.setStyleSheet(action_button_style)
        
        export_btn = QPushButton("Export to Parquet")
        export_btn.setToolTip("Export quality-controlled content to Parquet (Cmd+E)")
        export_btn.clicked.connect(self.export_to_parquet)
        export_btn.setEnabled(False)  # Disabled until processing is done
        export_btn.setShortcut(QKeySequence("Ctrl+E"))  # Ctrl+E maps to Cmd+E on Mac
        export_btn.setStyleSheet(action_button_style)
        self.export_btn = export_btn  # Store reference
        
        # Batch processing button
        batch_btn = QPushButton("📦 Batch")
        batch_btn.setToolTip("Process multiple PDFs at once (Ctrl+B)")
        batch_btn.clicked.connect(self.open_batch_dialog)
        batch_btn.setShortcut(QKeySequence("Ctrl+B"))
        batch_btn.setStyleSheet(action_button_style)
        
        # Add action buttons directly to main layout
        layout.addWidget(open_file_btn)
        layout.addWidget(open_url_btn)
        layout.addWidget(self.url_input)
        layout.addWidget(process_btn)
        layout.addWidget(export_btn)
        layout.addWidget(batch_btn)
        
        # Add stretch to push everything left
        layout.addStretch()
        
        # Add to splitter instead of layout
        if isinstance(parent_layout, QSplitter):
            parent_layout.addWidget(top_bar)
        else:
            parent_layout.addWidget(top_bar)
    
    def _toggle_url_input(self):
        """Toggle the URL input field visibility"""
        if self.url_input.isHidden():
            self.url_input.show()
            self.url_input.setFocus()
            # Check clipboard for URL
            clipboard = QApplication.clipboard()
            clipboard_text = clipboard.text()
            if self._is_url(clipboard_text):
                self.url_input.setText(clipboard_text)
                self.url_input.selectAll()
        else:
            self.url_input.hide()
            self.url_input.clear()
    
    def _handle_url_input(self):
        """Handle URL input when Enter is pressed"""
        url = self.url_input.text().strip()
        if not url:
            return
            
        if not self._is_url(url):
            QMessageBox.warning(self, "Invalid URL", "Invalid URL")
            return
        
        try:
            # Download PDF
            file_path = self._download_pdf_from_url(url)
            
            # Open the downloaded PDF
            self.create_embedded_pdf_viewer(file_path)
            self.current_pdf_path = file_path
            
            # Add URL to recent files
            self._add_to_recent_files(f"[URL] {url}")
            
            # Hide and clear the input
            self.url_input.hide()
            self.url_input.clear()
            
        except Exception as e:
            QMessageBox.critical(self, "Download Failed", str(e))
    
    def _show_welcome(self):
        # Clear left pane
        for i in reversed(range(self.left_layout.count())): 
            widget = self.left_layout.itemAt(i).widget()
            if widget:
                widget.setParent(None)
        
        # Show shortcuts in terminal
        self.log("Cmd+O: Open File | Cmd+U: Open URL | Cmd+P: Process | Cmd+E: Export")
    
    def _update_pane_styles(self):
        pass  # Simplified for space
    
    def zoom(self, delta: int):
        if self.active_pane == 'right':
            old_zoom = self.text_zoom
            self.text_zoom = max(TEXT_ZOOM_MIN, min(TEXT_ZOOM_MAX, self.text_zoom + (2 if delta > 0 else -2)))
            self._apply_zoom()
        elif self.active_pane == 'left' and hasattr(self, 'embedded_pdf_view') and self.embedded_pdf_view:
            factor = 1.1 if delta > 0 else 0.9
            self.pdf_zoom = max(0.1, min(5.0, self.pdf_zoom * factor))
            self._apply_zoom()
    
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
                self.text_zoom = max(TEXT_ZOOM_MIN, min(TEXT_ZOOM_MAX, self.text_zoom + (1 if delta > 0 else -1)))
                self._apply_zoom()
                return True
            elif hasattr(self, 'embedded_pdf_view') and obj == self.embedded_pdf_view:
                factor = 1.1 if delta > 0 else 0.9
                new_zoom = max(0.1, min(5.0, self.pdf_zoom * factor))
                if new_zoom != self.pdf_zoom:
                    self.pdf_zoom = new_zoom
                    self._apply_zoom()
                return True
        return False
    
    def _apply_theme(self):
        # Theme colors
        bg1, bg2, bg3 = "#525659", "#3A3C3E", "#1E1E1E"
        c1, c2 = "#1ABC9C", "#16A085"
        
        # Build CSS - compact but structured
        css = f"""
        * {{color: #FFFFFF}}
        QMainWindow, QTextEdit {{background-color: {bg1}}}
        #topBar {{background-color: {bg1}; border-bottom: 1px solid {bg2}}}
        QPushButton {{background: #6B6E71; border: 1px solid #4A4C4E; border-radius: 4px; padding: 8px 16px; font-size: 14px}}
        QPushButton:hover {{background: #7B7E81; border-color: #5A5C5E}}
        QPushButton:checked {{background: {c1}; border-color: {c2}}}
        #terminal {{background: {bg3}; color: {c1}; font: 11px 'Courier New'; border: 1px solid #333; border-radius: 4px; padding: 4px}}
        QTextEdit {{border: 1px solid {bg2}; border-radius: 4px}}
        QScrollBar {{background: {bg2}}}
        QScrollBar:vertical {{width: 12px}} QScrollBar:horizontal {{height: 12px}}
        QScrollBar::handle {{background: {c1}; border: none}}
        QScrollBar::handle:vertical {{min-height: 20px}} QScrollBar::handle:horizontal {{min-width: 20px}}
        QScrollBar::handle:hover {{background: {c2}}}
        QScrollBar::add-line, QScrollBar::sub-line {{border: none; background: none; height: 0; width: 0}}
        #searchWidget {{background: {bg2}; border-bottom: 2px solid {c1}}}
        #searchInput {{background: {bg1}; border: 1px solid {c1}; border-radius: 4px; padding: 5px 10px; color: #FFF; selection-background-color: {c1}}}
        #searchInput:focus {{border-color: {c2}}}
        #matchLabel {{color: {c1}; font-size: 12px}}
        #prevButton, #nextButton {{background: {c1}; color: white; border: none; padding: 5px 15px; font-size: 12px}}
        #prevButton:hover, #nextButton:hover {{background: {c2}}}
        #closeButton {{background: transparent; color: {c1}; border: 1px solid {c1}; padding: 2px}}
        #closeButton:hover {{background: {c1}; color: white}}
        """
        self.setStyleSheet(css)
    
    
    def _handle_gesture_zoom(self, zoom_delta: float, zoom_factor: float) -> None:
        """Handle zoom gesture for active pane"""
        if self.active_pane == 'right' and abs(zoom_delta) > 0.05:
            self.text_zoom = max(TEXT_ZOOM_MIN, min(TEXT_ZOOM_MAX, int(self.text_zoom) + (1 if zoom_delta > 0 else -1)))
            self._apply_zoom()
        elif self.active_pane == 'left' and abs(zoom_delta) > 0.02:
            self.pdf_zoom = max(PDF_ZOOM_MIN, min(PDF_ZOOM_MAX, self.pdf_zoom * zoom_factor))
            self._apply_zoom()
    
    def _apply_zoom(self) -> None:
        """Apply zoom to active pane"""
        if self.active_pane == 'right' and self.faithful_output:
            if hasattr(self, '_last_processing_result'):
                self._display_in_faithful_output(self._last_processing_result)
        elif self.active_pane == 'left' and hasattr(self, 'embedded_pdf_view'):
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
        
        # Process with or without OCR based on user response
        use_ocr = reply == QMessageBox.StandardButton.Yes
        self.log("🔍 Reprocessing with OCR enabled..." if use_ocr else "📄 Processing without OCR...")
        
        # Create and start processor
        self.processor = DocumentProcessor(file_path, use_ocr=use_ocr)
        self.processor.progress.connect(self.log)
        self.processor.error.connect(lambda e: self.log(f"🐹 Error: {e}"))
        self.processor.finished.connect(self.on_processing_finished)
        self.processor.start()
        
        # Restart animation
        self.processing_timer.start(ANIMATION_INTERVAL)
    
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
            if not self.processor.wait(THREAD_WAIT_TIMEOUT):  # 5 second timeout
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
            # Create new processor - Toggle OCR with Shift key
            use_ocr = bool(QApplication.keyboardModifiers() & Qt.KeyboardModifier.ShiftModifier)
            self.log(f"🔧 OCR Mode: {'ENABLED' if use_ocr else 'DISABLED'} (hold Shift to toggle)")
            self.processor = DocumentProcessor(file_path, use_ocr=use_ocr)
            
            # Connect signals (no need to disconnect - it's a new object)
            self.processor.progress.connect(self.log)
            self.processor.error.connect(lambda e: self.log(f"🐹 Error: {e}"))
            self.processor.finished.connect(self.on_processing_finished)
            self.processor.ocr_needed.connect(lambda: self._on_ocr_needed(file_path))
            self.processor.start()
            
            # Start processing animation
            self.processing_timer.start(ANIMATION_INTERVAL)  # Update every 500ms
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
        
        html_parts = []
        html_parts.append(
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
            f'<div style="color: #FF6B6B;">OCR Mode: {"ENABLED" if hasattr(self, "processor") and self.processor.use_ocr else "DISABLED"}</div>'
            f'<hr style="border-color: #3A3C3E;">'
        )
        
        # Add debug messages if any
        if result.debug_messages:
            html_parts.append(
                f'<div style="background: #2D2F31; padding: 10px; margin: 10px 0; border: 1px solid #444;">'
                f'<h3 style="color: #FFB347;">Debug Messages:</h3>'
            )
            for msg in result.debug_messages:
                html_parts.append(f'<div style="color: #B0B0B0;">• {msg}</div>')
            html_parts.append('</div><hr style="border-color: #3A3C3E;">')
        
        html_parts.append(
            f'{result.html_content}'
            f'</body></html>'
        )
        self.faithful_output.setHtml(''.join(html_parts))
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
        
        # Clean up temporary files
        for temp_file in self._temp_files:
            try:
                if os.path.exists(temp_file):
                    os.unlink(temp_file)
                    self.log(f"Cleaned up temp file: {os.path.basename(temp_file)}")
            except Exception as e:
                print(f"Failed to clean up {temp_file}: {e}")
        
        # Remove all event filters to prevent memory leaks
        if hasattr(self, 'left_pane'):
            self.left_pane.removeEventFilter(self)
        if hasattr(self, 'faithful_output'):
            self.faithful_output.removeEventFilter(self)
        if hasattr(self, 'embedded_pdf_view') and self.embedded_pdf_view:
            self.embedded_pdf_view.removeEventFilter(self)
        
        event.accept()

def main():
    def handle_exception(exc_type, exc_value, exc_traceback):
        if issubclass(exc_type, KeyboardInterrupt):
            sys.__excepthook__(exc_type, exc_value, exc_traceback)
            return
        print("🐹 Uncaught exception:")
        traceback.print_exception(exc_type, exc_value, exc_traceback)
    sys.excepthook = handle_exception
    print("CHONKER ready with OCR image preprocessing v2!")
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
    window.raise_()
    window.activateWindow()
    sys.exit(app.exec())

if __name__ == "__main__":
    main()
