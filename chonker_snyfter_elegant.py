#!/usr/bin/env python3
"""
🐹 CHONKER & 🐁 SNYFTER - Elegant Document Processing System

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
    print("⚠️ Docling not available. Install with: pip install docling")

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
    
    def run(self):
        """Process document with comprehensive error handling"""
        start_time = datetime.now()
        
        try:
            # Initialize docling with tqdm fix
            self._init_docling()
            
            # Convert document
            self.progress.emit("🐹 *chomp chomp* Processing document...")
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
        """Convert document item to HTML"""
        item_type = type(item).__name__
        
        if item_type == 'SectionHeaderItem' and hasattr(item, 'text'):
            heading_level = min(level + 1, 3)
            return f'<h{heading_level}>{item.text}</h{heading_level}>'
        
        elif item_type == 'TableItem':
            return self._table_to_html(item)
        
        elif item_type == 'TextItem' and hasattr(item, 'text'):
            return f'<p>{item.text}</p>'
        
        elif item_type == 'ListItem' and hasattr(item, 'text'):
            return f'<li>{item.text}</li>'
        
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
                    html.append(f'<th>{col}</th>')
                html.append('</tr>')
                
                # Data rows
                for _, row in df.iterrows():
                    html.append('<tr>')
                    for value in row:
                        html.append(f'<td contenteditable="true">{value}</td>')
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
        """Search documents using FTS"""
        conn = sqlite3.connect(self.db_path)
        results = []
        
        try:
            cursor = conn.execute('''
                SELECT DISTINCT d.* 
                FROM chunks_fts f
                JOIN documents d ON f.document_id = d.id
                WHERE chunks_fts MATCH ?
                ORDER BY rank
                LIMIT 50
            ''', (query,))
            
            results = [dict(row) for row in cursor.fetchall()]
            
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
            print("🐹 Sacred Android 7.1 CHONKER emoji loaded!")
        else:
            # Fallback but with warning
            print("⚠️ CHONKER emoji missing! Using fallback...")
            self.chonker_pixmap = self._create_fallback_emoji("🐹", QColor("#FFE4B5"))
        
        # Load SNYFTER emoji
        snyfter_path = assets_dir / "snyfter.png"
        if snyfter_path.exists():
            self.snyfter_pixmap = QPixmap(str(snyfter_path))
            print("🐁 Sacred Android 7.1 SNYFTER emoji loaded!")
        else:
            print("⚠️ SNYFTER emoji missing! Using fallback...")
            self.snyfter_pixmap = self._create_fallback_emoji("🐁", QColor("#D3D3D3"))
    
    def _create_fallback_emoji(self, emoji: str, bg_color: QColor) -> QPixmap:
        """Create fallback emoji (but we should never need this!)"""
        pixmap = QPixmap(64, 64)
        pixmap.fill(Qt.GlobalColor.transparent)
        
        painter = QPainter(pixmap)
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
            print("🛡️ Caffeinate defense activated!")
        except:
            print("⚠️ Caffeinate not available")
    
    def _init_ui(self):
        """Initialize the user interface"""
        self.setWindowTitle("🐹 CHONKER & 🐁 SNYFTER")
        self.setGeometry(100, 100, 1400, 900)
        
        # Menu bar
        self._create_menu_bar()
        
        # Central widget
        central = QWidget()
        self.setCentralWidget(central)
        layout = QVBoxLayout(central)
        layout.setContentsMargins(0, 0, 0, 0)
        
        # Top bar
        self._create_top_bar(layout)
        
        # Content area - split view like before
        self.splitter = QSplitter(Qt.Orientation.Horizontal)
        layout.addWidget(self.splitter)
        
        # Left side - welcome/PDF view placeholder
        self.left_pane = QWidget()
        self.left_layout = QVBoxLayout(self.left_pane)
        self.splitter.addWidget(self.left_pane)
        
        # Right side - faithful output (CRUCIAL!)
        self.faithful_output = QTextEdit()
        self.faithful_output.setReadOnly(False)
        self.faithful_output.setStyleSheet("""
            QTextEdit {
                font-family: 'Courier New', monospace;
                font-size: 12px;
                background-color: #FFFFFF;
                color: #000000;
                border: 2px solid #1ABC9C;
                border-radius: 5px;
                padding: 15px;
            }
        """)
        self.splitter.addWidget(self.faithful_output)
        self.splitter.setSizes([700, 700])
        
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
        open_action.setShortcut("Ctrl+O")
        open_action.triggered.connect(self.open_pdf)
        file_menu.addAction(open_action)
        
        process_action = QAction("Process", self)
        process_action.setShortcut("Ctrl+P")
        process_action.triggered.connect(self.process_current)
        file_menu.addAction(process_action)
        
        file_menu.addSeparator()
        
        quit_action = QAction("Quit", self)
        quit_action.setShortcut("Ctrl+Q")
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
        open_btn = QPushButton("📂")
        open_btn.setToolTip("Open PDF (Ctrl+O)")
        open_btn.clicked.connect(self.open_pdf)
        
        process_btn = QPushButton("⚙️")
        process_btn.setToolTip("Process (Ctrl+P)")
        process_btn.clicked.connect(self.process_current)
        
        layout.addWidget(open_btn)
        layout.addWidget(process_btn)
        
        parent_layout.addWidget(top_bar)
    
    def _show_welcome(self):
        """Show welcome screen"""
        welcome = QLabel("""
        <div style="text-align: center; padding: 50px;">
            <h1>🐹 CHONKER & 🐁 SNYFTER</h1>
            <p style="font-size: 18px; color: #666;">
                Enhanced Document Processing System
            </p>
            <p style="margin-top: 30px;">
                Press <b>Ctrl+O</b> to open a PDF<br>
                Press <b>Tab</b> to switch modes
            </p>
        </div>
        """)
        welcome.setAlignment(Qt.AlignmentFlag.AlignCenter)
        
        # Clear left pane
        for i in reversed(range(self.left_layout.count())): 
            self.left_layout.itemAt(i).widget().setParent(None)
        
        self.left_layout.addWidget(welcome)
    
    def _apply_theme(self):
        """Apply elegant theme"""
        self.setStyleSheet("""
            QMainWindow {
                background-color: #FAFAFA;
            }
            
            #topBar {
                background-color: #FFFFFF;
                border-bottom: 1px solid #E0E0E0;
            }
            
            QPushButton {
                background-color: #FFFFFF;
                border: 1px solid #D0D0D0;
                border-radius: 4px;
                padding: 8px 16px;
                font-size: 14px;
                color: #333333;
            }
            
            QPushButton:hover {
                background-color: #F5F5F5;
                border-color: #B0B0B0;
            }
            
            QPushButton:checked {
                background-color: #E3F2FD;
                border-color: #2196F3;
                color: #1976D2;
            }
            
            #terminal {
                background-color: #1E1E1E;
                color: #00FF00;
                font-family: 'Courier New', monospace;
                font-size: 11px;
                border: 1px solid #333;
                border-radius: 4px;
                padding: 4px;
            }
            
            QTextEdit {
                background-color: #FFFFFF;
                color: #000000;
                border: 1px solid #D0D0D0;
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
            self.log("🐹 CHONKER mode activated - Ready to process PDFs!")
        else:
            self.log("🐁 SNYFTER mode activated - Ready to search archives!")
    
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
            # Create floating PDF window
            self.create_pdf_window(file_path)
    
    def create_pdf_window(self, file_path: str):
        """Create floating window for PDF"""
        window = QWidget()
        window.setWindowTitle(f"📄 {os.path.basename(file_path)}")
        window.resize(800, 1000)
        window.setAttribute(Qt.WidgetAttribute.WA_DeleteOnClose)
        
        layout = QVBoxLayout(window)
        
        # PDF viewer
        pdf_view = QPdfView(window)
        pdf_document = QPdfDocument(window)
        pdf_view.setDocument(pdf_document)
        pdf_document.load(file_path)
        
        layout.addWidget(pdf_view)
        
        # Store reference
        window_id = f"pdf_{len(self.floating_windows)}"
        self.floating_windows[window_id] = {
            'window': window,
            'path': file_path,
            'document': pdf_document
        }
        
        # Connect close event
        window.destroyed.connect(lambda: self.floating_windows.pop(window_id, None))
        
        window.show()
        self.log(f"📂 Opened: {os.path.basename(file_path)}")
    
    def process_current(self):
        """Process current PDF"""
        # Find most recent PDF window
        if not self.floating_windows:
            QMessageBox.warning(self, "No PDF", "Please open a PDF first")
            return
        
        # Get last opened PDF
        window_data = list(self.floating_windows.values())[-1]
        file_path = window_data['path']
        self.current_pdf_path = file_path  # Store for database save
        
        # Start processing
        self.processor = DocumentProcessor(file_path)
        self.processor.progress.connect(self.log)
        self.processor.error.connect(lambda e: self.log(f"❌ Error: {e}"))
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
            
            self.log(f"✅ Processing complete! {len(result.chunks)} chunks extracted")
        else:
            self.log(f"❌ Processing failed: {result.error_message}")
    
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
                td[contenteditable="true"]:hover {{
                    background-color: #f0f0f0;
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
            <h2>🐹 CHONKER's Faithful Output</h2>
            <div>Document ID: {result.document_id}</div>
            <div>Processing Time: {result.processing_time:.2f}s</div>
            <hr>
            {result.html_content}
        </body>
        </html>
        """
        self.faithful_output.setHtml(html)
    
    def create_output_window(self, result: ProcessingResult):
        """Create window for processed output"""
        window = QWidget()
        window.setWindowTitle("🌐 Processed Output")
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
            <h2>🐹 CHONKER's Output</h2>
            {result.html_content}
        </body>
        </html>
        """
        
        output_view.setHtml(html)
        layout.addWidget(output_view)
        
        window.show()
    
    def closeEvent(self, event):
        """Clean up on close"""
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
        
        print("❌ Uncaught exception:")
        traceback.print_exception(exc_type, exc_value, exc_traceback)
    
    sys.excepthook = handle_exception
    
    # Print startup message
    print("""
╭────────────── Welcome ──────────────╮
│ 🐹 CHONKER & 🐁 SNYFTER             │
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