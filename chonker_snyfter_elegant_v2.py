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
    QAction, QKeySequence, QIcon, QPixmap, QPainter, QFont, QBrush, QColor
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
    
    def stop(self):
        """Stop processing thread safely"""
        self.should_stop = True
        if self.isRunning():
            if not self.wait(5000):  # Wait up to 5 seconds
                self.terminate()  # Force terminate if needed
                self.wait()  # Wait for termination
    
    def run(self):
        """Process document with comprehensive error handling"""
        start_time = datetime.now()
        
        try:
            # Initialize docling with tqdm fix
            self._init_docling()
            
            # Convert document
            self.progress.emit("*chomp chomp* Processing document...")
            result = self._convert_document()
            
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
    
    def _init_docling(self):
        """Initialize docling with tqdm workaround"""
        if not DOCLING_AVAILABLE:
            raise Exception("🐹 *cough* Docling not installed!")
        
        # Fix tqdm issue
        import tqdm
        if not hasattr(tqdm.tqdm, '_lock'):
            tqdm.tqdm._lock = threading.RLock()
    
    def _convert_document(self):
        """Convert PDF using docling"""
        from docling.document_converter import DocumentConverter
        converter = DocumentConverter()
        return converter.convert(self.pdf_path)
    
    def _extract_content(self, result) -> Tuple[List[DocumentChunk], str]:
        """Extract chunks and HTML from document"""
        chunks = []
        html_parts = ['<div id="document-content" contenteditable="true">']
        
        items = list(result.document.iterate_items())
        total = len(items)
        
        for idx, (item, level) in enumerate(items):
            if self.should_stop:
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
# DATABASE
# ============================================================================

class DocumentDatabase:
    """Clean database interface for document storage"""
    
    def __init__(self, db_path: str = "snyfter_archive.db"):
        self.db_path = db_path
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
    
    def save_document(self, result: ProcessingResult, file_path: str) -> bool:
        """Save processed document to database"""
        try:
            conn = sqlite3.connect(self.db_path)
            
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
            conn.close()
            return True
            
        except Exception as e:
            print(f"🐁 Database error: {e}")
            return False
    
    def search(self, query: str) -> List[Dict]:
        """Search documents using FTS with SQL injection protection"""
        if not query or len(query) > 1000:
            return []
        
        # Validate FTS5 query syntax - only allow safe characters
        import re
        if not re.match(r'^[a-zA-Z0-9\s\-_"\']+$', query):
            print(f"🐁 Invalid search query: {query}")
            return []
        
        conn = sqlite3.connect(self.db_path)
        conn.row_factory = sqlite3.Row  # Fix for dict conversion
        results = []
        
        try:
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
        
        conn.close()
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
        self.terminal.append(f"[{timestamp}] {message}")
        
        # Keep only last 100 lines
        text = self.terminal.toPlainText()
        lines = text.split('\n')
        if len(lines) > 100:
            self.terminal.setPlainText('\n'.join(lines[-100:]))
        
        # Scroll to bottom
        scrollbar = self.terminal.verticalScrollBar()
        scrollbar.setValue(scrollbar.maximum())
    
    def open_pdf(self):
        """Open PDF file"""
        file_path, _ = QFileDialog.getOpenFileName(
            self, "Open PDF", "", "PDF Files (*.pdf)"
        )
        
        if file_path:
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
        # Stop any existing processor
        if hasattr(self, 'processor') and self.processor.isRunning():
            self.log("Stopping previous processing...")
            self.processor.stop()
        
        # Check if we have a PDF loaded
        if not self.current_pdf_path:
            QMessageBox.warning(self, "No PDF", "Please open a PDF first")
            return
        
        file_path = self.current_pdf_path
        
        # Start processing
        self.processor = DocumentProcessor(file_path)
        self.processor.progress.connect(self.log)
        self.processor.error.connect(lambda e: self.log(f"🐹 Error: {e}"))
        self.processor.finished.connect(self.on_processing_finished)
        self.processor.start()
    
    def on_processing_finished(self, result: ProcessingResult):
        """Handle processing completion"""
        if result.success:
            # Save to database
            self.db.save_document(result, self.current_pdf_path)
            
            # Display in faithful output (RIGHT PANE!)
            self._display_in_faithful_output(result)
            
            # Also create floating output window
            self.create_output_window(result)
            
            self.log(f"Processing complete! {len(result.chunks)} chunks extracted")
        else:
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