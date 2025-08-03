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
# Removed pyarrow imports - not ready for export features yet

# Try to import pdfplumber for table extraction
try:
    import pdfplumber
    PDFPLUMBER_AVAILABLE = True
except ImportError:
    PDFPLUMBER_AVAILABLE = False
    print("Warning: pdfplumber not available. Install with: pip install pdfplumber")

# Suppress PyTorch pin_memory warnings on MPS
warnings.filterwarnings("ignore", message=".*pin_memory.*MPS.*")

# Qt imports
from PyQt6.QtWidgets import (
    QApplication, QMainWindow, QWidget, QVBoxLayout, QHBoxLayout,
    QPushButton, QFileDialog, QMessageBox, QTextEdit, QLabel, 
    QSplitter, QDialog, QMenuBar, QMenu, QToolBar, QStatusBar,
    QGroupBox, QTreeWidget, QTreeWidgetItem, QProgressDialog, QSizePolicy,
    QInputDialog, QLineEdit
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
try:
    from PyQt6.QtWebEngineWidgets import QWebEngineView
    WEBENGINE_AVAILABLE = True
except ImportError:
    WEBENGINE_AVAILABLE = False
    print("⚠️ QtWebEngine not available - spatial layout will be limited")
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
    
    def __init__(self, pdf_path: str, force_spatial: bool = False):
        super().__init__()
        self.pdf_path = pdf_path
        self._stop_event = threading.Event()  # Thread-safe stop flag
        self.start_time = None
        self.timeout_occurred = False
        self.force_spatial = force_spatial
        self.debug_messages = []
    
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
            
            # Convert document
            self.progress.emit("🐹 *chomp chomp* Processing document...")
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
                markdown_content="",  # Removed markdown export
                processing_time=processing_time,
                debug_messages=self.debug_messages
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
    
    def _convert_document(self):
        self.progress.emit("📄 Processing document...")
        from docling.document_converter import DocumentConverter
        
        max_retries = 3
        retry_delay = 1  # seconds
        
        
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
    
    def _extract_content(self, result) -> Tuple[List[DocumentChunk], str]:
        chunks = []
        
        # Check if we should use spatial layout mode
        # TEMPORARY: Default to spatial layout for testing
        use_spatial_layout = True  # self.force_spatial or self._should_use_spatial_layout(result)
        
        if use_spatial_layout:
            html_content = self._extract_with_spatial_layout(result, chunks)
        else:
            html_content = self._extract_linear(result, chunks)
        
        return chunks, html_content
    
    def _should_use_spatial_layout(self, result) -> bool:
        """Detect if document looks like a form based on content patterns"""
        items = list(result.document.iterate_items())
        if len(items) < 10:
            return False
            
        # Count short text items that might be form fields
        short_texts = 0
        total_texts = 0
        
        for item, _ in items[:50]:  # Check first 50 items
            if hasattr(item, 'text'):
                text = str(item.text).strip()
                if text:
                    total_texts += 1
                    if len(text) < 50:  # Short text likely form label/field
                        short_texts += 1
        
        # Log detection info
        form_ratio = (short_texts / total_texts) if total_texts > 0 else 0
        self.progress.emit(f"📊 Form detection: {short_texts}/{total_texts} short texts ({form_ratio:.1%})")
        
        # Check for form keywords too
        form_keywords = ['name:', 'date:', 'phone:', 'address:', 'title:', 'permit', 'form', 'id:', 'no.', 'tel:']
        keyword_found = False
        for item, _ in items[:20]:
            if hasattr(item, 'text'):
                text_lower = str(item.text).lower()
                if any(kw in text_lower for kw in form_keywords):
                    keyword_found = True
                    break
        
        # Use spatial layout if high ratio of short texts OR form keywords found
        use_spatial = (total_texts > 0 and form_ratio > 0.6) or keyword_found
        self.progress.emit(f"🗺️ Spatial layout: {'ENABLED' if use_spatial else 'DISABLED'} (keywords: {keyword_found})")
        
        return use_spatial
    
    def _extract_with_spatial_layout(self, result, chunks: List[DocumentChunk]) -> str:
        """Extract content preserving spatial layout using bounding boxes"""
        self.progress.emit("🗺️ SPATIAL LAYOUT MODE ACTIVE")
        print("🗺️ SPATIAL LAYOUT MODE ACTIVE - Console check")
        
        html_parts = [
            '<div id="document-content" contenteditable="true">',
            '<h2 style="color: #1ABC9C;">SPATIAL LAYOUT MODE</h2>'
        ]
        
        # Group items by page
        pages = {}
        items = list(result.document.iterate_items())
        
        # Debug: Check if ANY items have bbox
        items_with_bbox = 0
        items_without_bbox = 0
        
        for idx, (item, level) in enumerate(items):
            if self._stop_event.is_set() or self._check_timeout():
                break
                
            # Get page number and bbox
            page_no = 0
            bbox = None
            if hasattr(item, 'prov') and item.prov:
                if len(item.prov) > 0:
                    prov = item.prov[0]
                    page_no = getattr(prov, 'page_no', 0)
                    bbox = getattr(prov, 'bbox', None)
                    
            if bbox:
                items_with_bbox += 1
            else:
                items_without_bbox += 1
            
            if page_no not in pages:
                pages[page_no] = []
            pages[page_no].append((item, level, idx, bbox))
            
            # Create chunk
            chunk = self._create_chunk(item, level, idx, page_no)
            chunks.append(chunk)
            
            # Progress
            self.chunk_processed.emit(idx + 1, len(items))
        
        # Debug report
        self.progress.emit(f"📦 Items with bbox: {items_with_bbox}, without: {items_without_bbox}")
        
        # Add debug info to HTML output
        html_parts.append(f'<div style="background: #1ABC9C; color: #000; padding: 10px; margin: 10px 0;">')
        html_parts.append(f'<strong>DEBUG: Spatial Layout Stats</strong><br>')
        html_parts.append(f'Total items: {len(items)}<br>')
        html_parts.append(f'Items WITH bbox data: {items_with_bbox}<br>')
        html_parts.append(f'Items WITHOUT bbox: {items_without_bbox}<br>')
        if not WEBENGINE_AVAILABLE:
            html_parts.append(f'<br><strong>⚠️ Note:</strong> Install PyQt6-WebEngine for proper spatial rendering.<br>')
        html_parts.append(f'</div>')
        
        # Render each page with spatial layout
        for page_no in sorted(pages.keys()):
            if page_no > 0:
                html_parts.append(f'<h3 style="text-align: center; color: #1ABC9C;">Page {page_no}</h3>')
            
            # Get page dimensions from items FIRST
            scale = 1.2  # Same scale factor as coordinates
            
            # Find actual page bounds
            min_y = float('inf')
            max_y = 0
            max_x = 0
            
            for item, level, idx, bbox in pages[page_no]:
                if bbox and hasattr(bbox, 't') and hasattr(bbox, 'r'):
                    # For BOTTOMLEFT origin, find the maximum Y coordinate
                    if hasattr(bbox, 'coord_origin') and str(bbox.coord_origin) == 'CoordOrigin.BOTTOMLEFT':
                        max_y = max(max_y, bbox.t, bbox.b if hasattr(bbox, 'b') else bbox.t)
                        min_y = min(min_y, bbox.t, bbox.b if hasattr(bbox, 'b') else bbox.t)
                    max_x = max(max_x, bbox.r)
            
            # Calculate actual page dimensions
            if max_y > 0:
                # Add some padding
                page_height = (max_y + 50) * scale
                page_width = (max_x + 50) * scale
            else:
                # Fallback to standard letter size
                page_height = 1100  
                page_width = 850 * scale
            
            # NOW create the page div with correct height
            html_parts.append(f'<div class="spatial-page" data-page="{page_no}" style="min-height: {page_height}px;">')
            
            # Sort items by vertical position for gap detection
            sorted_items = sorted(
                [(item, level, idx, bbox) for item, level, idx, bbox in pages[page_no] if bbox],
                key=lambda x: x[3].t if hasattr(x[3], 't') else 0
            )
            
            # Render items with absolute positioning
            for item, level, idx, bbox in pages[page_no]:
                if bbox and hasattr(bbox, 'l') and hasattr(bbox, 't'):
                    # Convert coordinates handling different origins
                    # Scale factor: PDF points (72 DPI) to screen pixels
                    scale = 1.2  # Adjust for better visual spacing
                    
                    left = bbox.l * scale
                    width = abs(bbox.r - bbox.l) * scale if hasattr(bbox, 'r') else 200
                    height = abs(bbox.b - bbox.t) * scale if hasattr(bbox, 'b') else 20
                    
                    # Add minimum dimensions and extra padding
                    width = max(width, 50)
                    height = max(height, 20) + 5  # Add 5px vertical padding
                    
                    # Handle BOTTOMLEFT origin (PDF standard)
                    if hasattr(bbox, 'coord_origin') and 'BOTTOMLEFT' in str(bbox.coord_origin):
                        # Convert from bottom-left to top-left origin
                        # In BOTTOMLEFT: t is top edge distance from bottom, b is bottom edge distance from bottom
                        # We need distance from top of page
                        top = (max_y - bbox.t) * scale
                        # Also ensure height is calculated correctly
                        if hasattr(bbox, 'b'):
                            actual_height = abs(bbox.t - bbox.b)
                            height = max(actual_height, height)
                    else:
                        top = bbox.t * scale
                    
                    # Debug first few items
                    if idx < 3:
                        print(f"Item {idx}: bbox l={bbox.l:.1f}, t={bbox.t:.1f}, r={bbox.r:.1f}, b={bbox.b:.1f}")
                        print(f"  Converted: left={left:.1f}, top={top:.1f}, width={width:.1f}, height={height:.1f}")
                        print(f"  Text: {getattr(item, 'text', 'NO TEXT')[:50]}")
                    
                    # Detect if this looks like a form field
                    is_form_field = self._is_form_field(item, width, height)
                    
                    # Debug: show coordinates in the HTML
                    debug_info = f' title="pos:({left:.0f},{top:.0f}) size:({width:.0f}x{height:.0f})"'
                    
                    html_parts.append(
                        f'<div class="spatial-item{" form-field" if is_form_field else ""}" '
                        f'style="left: {left}px; top: {top}px; width: {width}px; min-height: {height}px;"'
                        f'{debug_info}>'
                    )
                    html_parts.append(self._item_to_html_spatial(item, level, page_no))
                    html_parts.append('</div>')
                else:
                    # Fallback for items without bbox - use flow layout
                    text = getattr(item, 'text', '')
                    if text:
                        html_parts.append(f'<div class="spatial-item" style="position: static; margin: 5px; display: inline-block;">')
                        html_parts.append(self._item_to_html_spatial(item, level, page_no))
                        html_parts.append('</div>')
            
            # After rendering all items, check for tables with pdfplumber
            print(f"🐹 DEBUG: Checking page {page_no + 1} with pdfplumber. PDFPLUMBER_AVAILABLE={PDFPLUMBER_AVAILABLE}, pdf_path={getattr(self, 'pdf_path', 'NO PATH')}")
            if PDFPLUMBER_AVAILABLE and hasattr(self, 'pdf_path'):
                try:
                    with pdfplumber.open(self.pdf_path) as pdf:
                        if page_no < len(pdf.pages):
                            page = pdf.pages[page_no]
                            tables = page.extract_tables()
                            print(f"🐹 pdfplumber scan complete: found {len(tables) if tables else 0} tables on page {page_no + 1}")
                            if tables:
                                print(f"🐹 Table details: {[f'{len(t)}x{len(t[0]) if t else 0}' for t in tables]}")
                                # Add any tables that Docling missed
                                for i, table in enumerate(tables):
                                    if table and len(table) > 0:  # Valid table (even if just headers)
                                        print(f"🐹 Table {i+1}: {len(table)} rows x {len(table[0]) if table else 0} columns")
                                        # Debug: print first few rows
                                        for j, row in enumerate(table[:3]):
                                            print(f"   Row {j}: {row}")
                                        # Place tables at bottom of page content
                                        top_position = page_height - 200 - (i * 200)  # Stack from bottom up
                                        html_parts.append(
                                            f'<div class="spatial-item" style="left: 50px; top: {top_position}px; '
                                            f'width: 90%; background: rgba(26, 188, 156, 0.1); '
                                            f'border: 2px solid #1ABC9C; padding: 10px; z-index: 10;">'
                                        )
                                        html_parts.append('<table style="width: 100%; border-collapse: collapse; border: 1px solid #666;">')
                                        
                                        # Header row
                                        if len(table) > 0:
                                            html_parts.append('<tr>')
                                            for cell in table[0]:
                                                cell_content = str(cell) if cell is not None else ""
                                                if not cell_content.strip():
                                                    cell_content = "&nbsp;"
                                                else:
                                                    cell_content = escape(cell_content)
                                                html_parts.append(f'<th style="padding: 5px; background: #1ABC9C; color: black; border: 1px solid #333;">{cell_content}</th>')
                                            html_parts.append('</tr>')
                                        
                                        # Data rows
                                        for row in table[1:]:
                                            html_parts.append('<tr>')
                                            for cell in row:
                                                # Preserve empty cells with non-breaking space
                                                cell_content = str(cell) if cell is not None else ""
                                                if not cell_content.strip():
                                                    cell_content = "&nbsp;"  # Non-breaking space for empty cells
                                                else:
                                                    cell_content = escape(cell_content)
                                                html_parts.append(f'<td style="padding: 5px; border: 1px solid #666;">{cell_content}</td>')
                                            html_parts.append('</tr>')
                                        
                                        html_parts.append('</table>')
                                        html_parts.append('</div>')
                                        print(f"🐹 Added table {i+1} with {len(table)} rows")
                except Exception as e:
                    print(f"🐹 pdfplumber page scan failed: {e}")
                    import traceback
                    traceback.print_exc()
            
            html_parts.append('</div>')
        
        html_parts.append('</div>')
        
        return '\n'.join(html_parts)
    
    def _is_form_field(self, item, width: float, height: float) -> bool:
        """Detect if item is likely a form field based on dimensions and content"""
        if not hasattr(item, 'text'):
            return False
            
        text = str(item.text).strip()
        # Form fields are typically small boxes with short text
        return (
            width < 300 and height < 40 and len(text) < 50
            and not any(text.lower().endswith(x) for x in ['.', ':', '?', '!'])
        )
    
    def _item_to_html_spatial(self, item, level: int, page_no: int = 0) -> str:
        """Convert item to HTML without wrapper elements for spatial layout"""
        item_type = type(item).__name__
        if hasattr(item, 'text'):
            text = self._enhance_text_formatting(escape(str(item.text)))
            if item_type == 'SectionHeaderItem':
                return f'<span style="color: #1ABC9C; margin: 0;">{text}</span>'
            else:
                return f'<span>{text}</span>'
        elif item_type == 'TableItem':
            # Pass PDF path and page number for pdfplumber extraction
            print(f"🐹 DEBUG: Found TableItem on page {page_no}")
            return self._table_to_html(item, self.pdf_path, page_no)
        return ''
    
    def _extract_linear(self, result, chunks: List[DocumentChunk]) -> str:
        """Original linear extraction method"""
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
            html_parts.append(self._item_to_html(item, level, item_page))
            
            # Progress
            self.chunk_processed.emit(idx + 1, total)
        
        html_parts.append('</div>')
        
        return '\n'.join(html_parts)
    
    
    def _create_chunk(self, item, level: int, index: int, page: int = 0) -> DocumentChunk:
        # Get text content
        text = getattr(item, 'text', str(item))
        
        return DocumentChunk(
            index=index,
            type=type(item).__name__.lower().replace('item', ''),
            content=text,
            metadata={'level': level, 'page': page}
        )
    
    
    
    def _item_to_html(self, item, level: int, page_no: int = 0) -> str:
        item_type = type(item).__name__
        if item_type == 'SectionHeaderItem' and hasattr(item, 'text'):
            h = min(level + 1, 3)
            return f'<h{h}>{self._enhance_text_formatting(escape(str(item.text)))}</h{h}>'
        elif item_type == 'TableItem':
            return self._table_to_html(item, self.pdf_path, page_no)
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
    
    def _table_to_html(self, table_item, pdf_path=None, page_num=None) -> str:
        html = ['<table class="editable-table" border="1">']
        
        # Try multiple methods to extract table data
        table_extracted = False
        
        # Method 0: Try pdfplumber first if available
        if PDFPLUMBER_AVAILABLE and pdf_path and page_num is not None:
            try:
                with pdfplumber.open(pdf_path) as pdf:
                    if page_num < len(pdf.pages):
                        page = pdf.pages[page_num]
                        # Try to find tables on this page
                        tables = page.extract_tables()
                        if tables:
                            # Use the first table found (could be improved with position matching)
                            table = tables[0]
                            if table and len(table) > 0:
                                # First row as headers
                                if len(table) > 0:
                                    html.append('<tr>')
                                    for cell in table[0]:
                                        html.append(f'<th>{escape(str(cell or ""))}</th>')
                                    html.append('</tr>')
                                # Rest as data
                                for row in table[1:]:
                                    html.append('<tr>')
                                    for cell in row:
                                        html.append(f'<td contenteditable="true">{escape(str(cell or ""))}</td>')
                                    html.append('</tr>')
                                table_extracted = True
                                print(f"🐹 Table extracted with pdfplumber from page {page_num + 1}")
                                print(f"🐹 Table has {len(table)} rows and {len(table[0]) if table else 0} columns")
            except Exception as e:
                print(f"🐹 pdfplumber failed: {e}")
        
        # Method 1: Try export_to_markdown first (best for display)
        if hasattr(table_item, 'export_to_markdown'):
            try:
                text = table_item.export_to_markdown()
                if text and text.strip() and not text.startswith('table_cells=') and '|' in text:
                    # Parse markdown table
                    lines = text.strip().split('\n')
                    for line in lines:
                        if '---' in line:
                            continue  # Skip separator lines
                        cells = line.split('|')
                        html.append('<tr>')
                        for cell in cells:
                            if cell.strip():
                                html.append(f'<td contenteditable="true">{escape(cell.strip())}</td>')
                        html.append('</tr>')
                    table_extracted = True
            except:
                pass
        
        # Method 2: Try export_to_dataframe
        if not table_extracted and hasattr(table_item, 'export_to_dataframe'):
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
        self.left_pane.setStyleSheet("QWidget { background-color: #525659; }")
        self.left_layout = QVBoxLayout(self.left_pane)
        self.left_layout.setContentsMargins(0, 0, 0, 0)
        self.left_layout.setSpacing(0)
        self.splitter.addWidget(self.left_pane)
        
        # Right side - faithful output (CRUCIAL!)
        if WEBENGINE_AVAILABLE:
            self.faithful_output = QWebEngineView()
            self.faithful_output.setContextMenuPolicy(Qt.ContextMenuPolicy.NoContextMenu)
            # Set initial HTML with dark background
            initial_html = '''<!DOCTYPE html>
            <html>
            <head>
                <style>
                    body { 
                        background-color: #525659; 
                        color: #FFFFFF; 
                        font-family: Arial, Helvetica, sans-serif;
                        margin: 20px;
                        font-size: 14px;
                    }
                </style>
            </head>
            <body>
            </body>
            </html>'''
            self.faithful_output.setHtml(initial_html)
        else:
            self.faithful_output = QTextEdit()
            self.faithful_output.setReadOnly(False)
            self.faithful_output.setStyleSheet("QTextEdit {background-color: #525659; color: #FFFFFF;}")
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
    
    
    
    
    # Removed export_to_parquet function - not ready yet
    
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
    
    def process_multiple_files(self, pdf_files):
        """Process multiple PDF files and display all results in the right pane"""
        modifiers = QApplication.keyboardModifiers()
        
        # Clear the right pane
        if WEBENGINE_AVAILABLE and isinstance(self.faithful_output, QWebEngineView):
            self.faithful_output.setHtml('')
        else:
            self.faithful_output.clear()
        
        # Show progress dialog
        progress = QProgressDialog("Processing multiple PDFs...", "Cancel", 0, len(pdf_files), self)
        progress.setWindowTitle("Processing")
        progress.setWindowModality(Qt.WindowModality.WindowModal)
        progress.setMinimumWidth(400)
        
        all_html = []
        
        for idx, pdf_path in enumerate(pdf_files):
            if progress.wasCanceled():
                break
                
            file_name = os.path.basename(pdf_path)
            progress.setLabelText(f"Processing {file_name}...")
            progress.setValue(idx)
            
            # Check file size
            file_size_mb = self._validate_file_size(pdf_path, "process")
            if file_size_mb is None:
                all_html.append(f'<h2 style="color: #e74c3c;">❌ {file_name} - File too large</h2><hr>')
                continue
            
            # Process the file
            try:
                self.log(f"Processing {file_name} ({file_size_mb:.1f} MB)...")
                
                # Create processor
                processor = DocumentProcessor(pdf_path)
                
                # Process synchronously for multi-file mode
                result = processor._convert_document()
                
                # Extract content
                chunks, html_content = processor._extract_content(result)
                
                # Add file header and content
                file_header = f'''
                <div style="background: #2c3e50; color: #ecf0f1; padding: 15px; margin: 20px 0 10px 0; border-radius: 5px;">
                    <h2 style="margin: 0;">📄 {file_name}</h2>
                    <p style="margin: 5px 0 0 0; font-size: 0.9em; color: #bdc3c7;">
                        Processed: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')} | 
                        Chunks: {len(chunks)} | 
                        Size: {file_size_mb:.1f} MB |
                        Mode: Standard
                    </p>
                </div>
                '''
                
                all_html.append(file_header)
                all_html.append(html_content)
                all_html.append('<div style="border-bottom: 3px solid #34495e; margin: 40px 0;"></div>')
                
                self.log(f"✅ Completed {file_name}")
                
            except Exception as e:
                error_html = f'''
                <h2 style="color: #e74c3c;">❌ {file_name} - Processing Error</h2>
                <p>{str(e)}</p>
                <hr>
                '''
                all_html.append(error_html)
                self.log(f"❌ Error processing {file_name}: {str(e)}")
        
        progress.setValue(len(pdf_files))
        
        # Display all results
        combined_html = '\n'.join(all_html)
        self.faithful_output.setHtml(combined_html)
        
        # Enable export button (for exporting all as one file)
        # self.export_btn.setEnabled(True)  # Removed export
        
        # Store the file list for reference
        self.current_pdf_path = pdf_files[0] if pdf_files else None
        self._last_processing_result = ProcessingResult(
            success=True,
            document_id="multi_" + datetime.now().strftime('%Y%m%d_%H%M%S'),
            chunks=[],  # Combined chunks would go here if needed
            html_content=combined_html,
            markdown_content="",
            processing_time=0
        )
        
        # Show first PDF in left pane if exists
        if pdf_files and len(pdf_files) > 0:
            self.create_embedded_pdf_viewer(pdf_files[0])
        
        self.log(f"✅ Processed {len(pdf_files)} files")
    
    
    def keyPressEvent(self, event):
        if event.key() == Qt.Key.Key_Question:
            QMessageBox.information(self, "Keyboard Shortcuts",
                "Cmd+O: Open PDF from File (multi-select supported)\n"
                "Cmd+U: Open PDF from URL\n"
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
        
        # Removed export actions - not ready yet
        
        
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
        self.chonker_btn.clicked.connect(lambda: self.log("🐹 CHONKER ready!"))
        
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
        process_btn.setToolTip("Process (Ctrl+P)\nHold Alt/Option: Force spatial layout for forms")
        process_btn.clicked.connect(self.process_current)
        process_btn.setStyleSheet(action_button_style)
        
        # Removed export button - not ready yet
        # export_btn.setStyleSheet(action_button_style)
        # self.export_btn = export_btn  # Store reference
        
        
        # Add action buttons directly to main layout
        layout.addWidget(open_file_btn)
        layout.addWidget(open_url_btn)
        layout.addWidget(self.url_input)
        layout.addWidget(process_btn)
        # layout.addWidget(export_btn)  # Removed export
        
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
        dialog.setWindowTitle("Open PDF(s) - Hold Shift for multiple selection")
        dialog.setNameFilter("PDF Files (*.pdf)")
        dialog.setFileMode(QFileDialog.FileMode.ExistingFiles)  # Allow multiple files
        dialog.setViewMode(QFileDialog.ViewMode.List)
        
        # Remember last directory
        if hasattr(self, '_last_open_dir') and self._last_open_dir:
            dialog.setDirectory(self._last_open_dir)
        
        if dialog.exec() != QFileDialog.DialogCode.Accepted:
            return
            
        file_paths = dialog.selectedFiles()
        if file_paths:
            self._last_open_dir = os.path.dirname(file_paths[0])
            
            # Check if multiple files selected
            if len(file_paths) > 1:
                self.process_multiple_files(file_paths)
            else:
                # Single file - original behavior
                file_path = file_paths[0]
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
            # Create new processor - Alt forces spatial
            modifiers = QApplication.keyboardModifiers()
            force_spatial = bool(modifiers & Qt.KeyboardModifier.AltModifier)
            
            if force_spatial:
                self.log("🗺️ Forcing spatial layout mode (Alt key held)")
            self.processor = DocumentProcessor(file_path, force_spatial=force_spatial)
            
            # Connect signals (no need to disconnect - it's a new object)
            self.processor.progress.connect(self.log)
            self.processor.error.connect(lambda e: self.log(f"🐹 Error: {e}"))
            self.processor.finished.connect(self.on_processing_finished)
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
            # if hasattr(self, 'export_btn'):  # Removed export
            #     self.export_btn.setEnabled(True)
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
            f'body{{font-family:Arial,Helvetica,sans-serif;margin:20px;color:#FFF;background:#525659;font-size:{zoom_size}px!important}}'
            f'table{{border-collapse:collapse;margin:15px 0;border:1px solid #3A3C3E;background:#3A3C3E;font-size:{zoom_size}px!important}}'
            f'th,td{{border:1px solid #525659;padding:8px;color:#FFF;background:#424548;font-size:{zoom_size}px!important}}'
            f'th{{background:#3A3C3E;font-weight:bold}}td[contenteditable="true"]:hover{{background:#525659}}.table-controls{{margin:10px 0}}'
            f'button {{ background: #1ABC9C; color: white; border: none; padding: 5px 10px; margin: 5px; border-radius: 3px; cursor: pointer; }}'
            f'button:hover {{ background: #16A085; }}'
            f'h1 {{ color: #1ABC9C; font-size: {zoom_size}px !important; }}'
            f'h2 {{ color: #1ABC9C; font-size: {zoom_size}px !important; }}'
            f'h3 {{ color: #1ABC9C; font-size: {zoom_size}px !important; }}'
            f'p {{ color: #FFFFFF; font-size: {zoom_size}px !important; }}'
            f'li {{ color: #FFFFFF; font-size: {zoom_size}px !important; }}'
            f'div {{ font-size: {zoom_size}px !important; }}'
            f'span {{ font-size: {zoom_size}px !important; }}'
            # Include spatial layout styles
            f'.spatial-page {{ position: relative; width: 100%; min-height: 1000px; margin-bottom: 20px; border: 2px solid #1ABC9C; background: #525659; overflow: visible; }}'
            f'.spatial-item {{ position: absolute !important; border: none; padding: 3px 5px; color: #FFF; background: transparent; font-size: {zoom_size * 0.5}px !important; line-height: 1.1; overflow: hidden; }}'
            f'.spatial-item:hover {{ color: #1ABC9C; z-index: 100; }}'
            f'.form-field {{ background: transparent !important; border: none !important; color: #FFF; }}'
            f'</style></head><body>'
            f'<h2 style="color: #1ABC9C;">CHONKER\'s Faithful Output</h2>'
            f'<div style="color: #B0B0B0;">Document ID: {result.document_id}</div>'
            f'<div style="color: #B0B0B0;">Processing Time: {result.processing_time:.2f}s</div>'
            f'<div style="color: #FF6B6B;">Processing Mode: Standard</div>'
            f'<hr style="border-color: #3A3C3E;">'
        )
        
        # Add debug messages if any
        if result.debug_messages:
            html_parts.append(
                f'<div style="background: #525659; padding: 10px; margin: 10px 0; border: 1px solid #666;">'
                f'<h3 style="color: #FFB347;">Debug Messages:</h3>'
            )
            for msg in result.debug_messages:
                html_parts.append(f'<div style="color: #B0B0B0;">• {msg}</div>')
            html_parts.append('</div><hr style="border-color: #3A3C3E;">')
        
        html_parts.append(
            f'{result.html_content}'
            f'</body></html>'
        )
        if WEBENGINE_AVAILABLE and isinstance(self.faithful_output, QWebEngineView):
            self.faithful_output.setHtml(''.join(html_parts))
        else:
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
            f'body {{ font-family: Arial, Helvetica, sans-serif; margin: 20px; color: #FFFFFF; background: #525659; }}'
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
    print("CHONKER ready!")
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
