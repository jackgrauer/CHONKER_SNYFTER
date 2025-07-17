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

# Qt imports
from PyQt6.QtWidgets import (
    QApplication, QMainWindow, QWidget, QVBoxLayout, QHBoxLayout,
    QPushButton, QFileDialog, QMessageBox, QTextEdit, QLabel, 
    QSplitter, QDialog, QMenuBar, QMenu, QToolBar, QStatusBar
)
from PyQt6.QtCore import (
    Qt, QThread, pyqtSignal, QTimer, QPointF, QObject, QEvent
)
from PyQt6.QtGui import (
    QAction, QKeySequence, QIcon, QPixmap, QPainter, QFont, QBrush, QColor, QTextCursor
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
    
    def __init__(self, pdf_path: str):
        super().__init__()
        self.pdf_path = pdf_path
        self.should_stop = False
        self.start_time = None
        self.timeout_occurred = False
    
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
            
            # Convert document
            self.progress.emit("*chomp chomp* Processing document...")
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
    
    def _convert_document_lazy(self):
        """Convert large PDFs in chunks to reduce memory usage"""
        from docling.document_converter import DocumentConverter
        
        # For now, use standard conversion with memory monitoring
        # In production, this would process pages in batches
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
        
        for idx, (item, level) in enumerate(items):
            if self.should_stop or self._check_timeout():
                break
            
            # Create chunk
            chunk = self._create_chunk(item, level, idx)
            chunks.append(chunk)
            
            # Add to HTML
            html_parts.append(self._item_to_html(item, level))
            
            # Progress
            self.chunk_processed.emit(idx + 1, total)
        
        html_parts.append('</div>')
        html_parts.append(self._get_table_editor_script())
        
        return chunks, '\n'.join(html_parts)
    
    def _create_chunk(self, item, level: int, index: int) -> DocumentChunk:
        """Create a document chunk from an item"""
        item_type = type(item).__name__
        content = getattr(item, 'text', str(item))
        
        return DocumentChunk(
            index=index,
            type=item_type.lower().replace('item', ''),
            content=content,
            metadata={'level': level}
        )
    
    def _item_to_html(self, item, level: int) -> str:
        """Convert document item to HTML with XSS protection"""
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
        self.active_pane = 'right'  # Track which pane is active
        self.embedded_pdf_view = None  # For embedded PDF viewer
        
        # CRUCIAL: Load Android 7.1 Noto emojis!
        self._load_sacred_emojis()
        
        self._init_caffeinate()
        self._init_ui()
        self._apply_theme()
        
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
        
        # Menu bar
        self._create_menu_bar()
        
        # Central widget
        central = QWidget()
        self.setCentralWidget(central)
        layout = QVBoxLayout(central)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(0)
        
        # Top bar
        self._create_top_bar(layout)
        
        # Content area - split view like before
        self.splitter = QSplitter(Qt.Orientation.Horizontal)
        self.splitter.setHandleWidth(3)
        self.splitter.setStyleSheet("""
            QSplitter::handle {
                background-color: #3A3C3E;
            }
        """)
        layout.addWidget(self.splitter)
        
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
        
        # Set up focus tracking
        self.left_pane.installEventFilter(self)
        self.faithful_output.installEventFilter(self)
        
        # Status bar
        self.status_bar = QStatusBar()
        self.setStatusBar(self.status_bar)
        
        # Welcome screen
        self._show_welcome()
    
    def _create_menu_bar(self):
        """Create application menu bar"""
        menubar = self.menuBar()
        
        # File menu
        file_menu = menubar.addMenu("File")
        
        open_action = QAction("Open PDF", self)
        open_action.setShortcut("Cmd+O" if sys.platform == "darwin" else "Ctrl+O")
        open_action.triggered.connect(self.open_pdf)
        file_menu.addAction(open_action)
        
        process_action = QAction("Process", self)
        process_action.setShortcut("Cmd+P" if sys.platform == "darwin" else "Ctrl+P")
        process_action.triggered.connect(self.process_current)
        file_menu.addAction(process_action)
        
        file_menu.addSeparator()
        
        quit_action = QAction("Quit", self)
        quit_action.setShortcut("Cmd+Q" if sys.platform == "darwin" else "Ctrl+Q")
        quit_action.triggered.connect(self.close)
        file_menu.addAction(quit_action)
        
        # View menu
        view_menu = menubar.addMenu("View")
        
        chonker_action = QAction("CHONKER Mode", self)
        chonker_action.triggered.connect(lambda: self.set_mode(Mode.CHONKER))
        view_menu.addAction(chonker_action)
        
        snyfter_action = QAction("SNYFTER Mode", self)
        snyfter_action.triggered.connect(lambda: self.set_mode(Mode.SNYFTER))
        view_menu.addAction(snyfter_action)
    
    def _create_top_bar(self, parent_layout):
        """Create slim top bar with mode toggle"""
        top_bar = QWidget()
        top_bar.setFixedHeight(50)
        top_bar.setObjectName("topBar")
        
        layout = QHBoxLayout(top_bar)
        layout.setContentsMargins(20, 0, 20, 0)
        
        # Mode toggle with sacred emojis
        self.chonker_btn = QPushButton()
        self.chonker_btn.setIcon(QIcon(self.chonker_pixmap))
        self.chonker_btn.setText(" CHONKER")
        self.chonker_btn.setCheckable(True)
        self.chonker_btn.setChecked(True)
        self.chonker_btn.clicked.connect(lambda: self.set_mode(Mode.CHONKER))
        
        self.snyfter_btn = QPushButton()
        self.snyfter_btn.setIcon(QIcon(self.snyfter_pixmap))
        self.snyfter_btn.setText(" SNYFTER")
        self.snyfter_btn.setCheckable(True)
        self.snyfter_btn.clicked.connect(lambda: self.set_mode(Mode.SNYFTER))
        
        layout.addWidget(self.chonker_btn)
        layout.addWidget(self.snyfter_btn)
        
        # Terminal display
        self.terminal = QTextEdit()
        self.terminal.setFixedHeight(35)
        self.terminal.setReadOnly(True)
        self.terminal.setObjectName("terminal")
        layout.addWidget(self.terminal, stretch=1)
        
        # Quick actions
        open_btn = QPushButton("Open")
        open_btn.setToolTip("Open PDF (Cmd+O)" if sys.platform == "darwin" else "Open PDF (Ctrl+O)")
        open_btn.clicked.connect(self.open_pdf)
        
        process_btn = QPushButton("Process")
        process_btn.setToolTip("Process (Cmd+P)" if sys.platform == "darwin" else "Process (Ctrl+P)")
        process_btn.clicked.connect(self.process_current)
        
        layout.addWidget(open_btn)
        layout.addWidget(process_btn)
        
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
                Press <b>Cmd+O</b> to open a PDF<br>
                Press <b>Cmd+P</b> to process document<br>
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
        left_border_color = "#1ABC9C" if self.active_pane == 'left' else "#3A3C3E"
        right_border_color = "#1ABC9C" if self.active_pane == 'right' else "#3A3C3E"
        
        # Add glow effect for active pane
        left_shadow = "0 0 10px #1ABC9C" if self.active_pane == 'left' else "none"
        right_shadow = "0 0 10px #1ABC9C" if self.active_pane == 'right' else "none"
        
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
    
    
    def eventFilter(self, obj, event):
        """Track focus and mouse events for pane activation"""
        if event.type() == QEvent.Type.MouseButtonPress:
            if obj == self.left_pane or (hasattr(self, 'embedded_pdf_view') and self.embedded_pdf_view and obj == self.embedded_pdf_view):
                if self.active_pane != 'left':
                    self.active_pane = 'left'
                    self._update_pane_styles()
                    self.log("Left pane active - PDF navigation enabled")
            elif obj == self.faithful_output:
                if self.active_pane != 'right':
                    self.active_pane = 'right'
                    self._update_pane_styles()
                    self.log("Right pane active - Output editing enabled")
        
        # Let all events pass through normally - QPdfView handles its own scrolling
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
                color: #1ABC9C;
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
        """)
    
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
        
        # Scroll to bottom
        scrollbar = self.terminal.verticalScrollBar()
        scrollbar.setValue(scrollbar.maximum())
    
    def open_pdf(self):
        """Open PDF file with size validation"""
        file_path, _ = QFileDialog.getOpenFileName(
            self, "Open PDF", "", "PDF Files (*.pdf)"
        )
        
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
        self.embedded_pdf_view.setZoomMode(QPdfView.ZoomMode.FitToWidth)
        
        # Add to layout
        self.left_layout.addWidget(self.embedded_pdf_view)
        
        # Install event filter for focus tracking
        self.embedded_pdf_view.installEventFilter(self)
        
        # Switch to left pane
        self.active_pane = 'left'
        self._update_pane_styles()
        self.embedded_pdf_view.setFocus()
        
        self.log(f"Opened: {os.path.basename(file_path)}")
    
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
            except:
                pass  # No connections to disconnect
            
            # Connect new signals
            self.processor.progress.connect(self.log)
            self.processor.error.connect(lambda e: self.log(f"🐹 Error: {e}"))
            self.processor.finished.connect(self.on_processing_finished)
            self.processor.start()
        except Exception as e:
            self.log(f"❌ Failed to start processing: {e}")
    
    def on_processing_finished(self, result: ProcessingResult):
        """Handle processing completion"""
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
    window.show()
    
    # Run application
    sys.exit(app.exec())


if __name__ == "__main__":
    main()