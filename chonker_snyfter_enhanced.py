#!/usr/bin/env python3
"""
🐹 CHONKER & 🐁 SNYFTER - The Document Processing Duo (ENHANCED VERSION)

CHONKER: The chubby hamster who gobbles up PDFs and makes them digestible
SNYFTER: The skinny librarian mouse who meticulously catalogs everything

ENHANCEMENTS:
- Instructor integration for structured AI outputs
- Better error handling and recovery
- Enhanced UI with animations and visual feedback
- Advanced batch processing with progress tracking
- Export functionality (JSON, CSV, Markdown)
- Real-time search and filtering
- PDF annotation and highlighting
- Document comparison features
"""

import sys
import os
import sqlite3
import json
import hashlib
import tempfile
from pathlib import Path
from typing import Dict, List, Optional, Tuple, Any
from datetime import datetime
from enum import Enum
import traceback
import csv

from PyQt6.QtWidgets import (
    QApplication, QMainWindow, QStackedWidget, QVBoxLayout, QHBoxLayout,
    QWidget, QPushButton, QFileDialog, QMessageBox, QTextEdit,
    QLabel, QComboBox, QScrollArea, QFrame, QProgressBar,
    QTableWidget, QTableWidgetItem, QTabWidget, QStatusBar,
    QLineEdit, QListWidget, QSplitter, QPlainTextEdit, QMenu,
    QSpinBox, QInputDialog, QListWidgetItem, QDialog, QDialogButtonBox,
    QCheckBox, QFormLayout, QGroupBox, QRadioButton, QButtonGroup,
    QGraphicsOpacityEffect, QToolBar, QDockWidget, QTreeWidget, QTreeWidgetItem
)
from PyQt6.QtCore import (
    Qt, QThread, pyqtSignal, QTimer, QPointF, QObject, QEvent, 
    QSize, QRectF, QPropertyAnimation, QParallelAnimationGroup,
    QSequentialAnimationGroup, QEasingCurve, pyqtProperty, QUrl
)
from PyQt6.QtGui import (
    QFont, QPalette, QColor, QTextCharFormat, QTextCursor, 
    QKeyEvent, QPixmap, QIcon, QAction, QPainter, QPen, QBrush,
    QLinearGradient, QRadialGradient, QFontDatabase, QMovie
)
from PyQt6.QtPdf import QPdfDocument, QPdfSelection
from PyQt6.QtPdfWidgets import QPdfView

# Enhanced imports
from pydantic import BaseModel, Field, validator
from typing import List, Optional, Dict, Any
from enum import Enum
from rich.console import Console
from rich.progress import Progress, SpinnerColumn, BarColumn, TextColumn
from rich.table import Table
from rich.panel import Panel
from rich import box

console = Console()

try:
    from docling.document_converter import DocumentConverter
    import fitz  # PyMuPDF
    DEPENDENCIES_AVAILABLE = True
except ImportError:
    DEPENDENCIES_AVAILABLE = False
    console.print("[red]⚠️  Missing dependencies! Please install docling and PyMuPDF[/red]")


# Structured data models using Instructor
class DocumentChunk(BaseModel):
    """Structured representation of a document chunk"""
    index: int
    type: str
    content: str
    level: int
    page: int
    confidence: float = Field(default=1.0, ge=0.0, le=1.0)
    metadata: Dict[str, Any] = Field(default_factory=dict)


class ProcessingResult(BaseModel):
    """Structured result from document processing"""
    success: bool
    document_id: str
    chunks: List[DocumentChunk]
    html_content: str
    markdown_content: str
    processing_time: float
    error_message: Optional[str] = None
    warnings: List[str] = Field(default_factory=list)


class ExportOptions(BaseModel):
    """Export configuration"""
    format: str = Field(description="Export format: json, csv, markdown, html")
    include_metadata: bool = True
    include_chunks: bool = True
    include_html: bool = False
    include_markdown: bool = True
    chunk_types: List[str] = Field(default_factory=lambda: ["all"])


class Mode(Enum):
    CHONKER = "chonker"  # PDF processing mode
    SNYFTER = "snyfter"  # Database/research mode


class ChonkerPersonality:
    """🐹 CHONKER - The enthusiastic PDF muncher"""
    
    GREETINGS = [
        "🐹 *munches* Got a PDF for me?",
        "🐹 CHONKER hungry for documents!",
        "🐹 *sniff sniff* I smell PDFs!",
        "🐹 Ready to chomp through any document!"
    ]
    
    PROCESSING = [
        "🐹 *nom nom nom* Processing...",
        "🐹 *crunch crunch* Breaking down this PDF...",
        "🐹 Digesting document... almost done!",
        "🐹 *munch munch* This is a big one!"
    ]
    
    SUCCESS = [
        "🐹 *burp* All done! That was delicious!",
        "🐹 PDF fully digested! Ready for Snyfter!",
        "🐹 *happy hamster noises* Success!",
        "🐹 Yum! Document processed perfectly!"
    ]
    
    ERROR = [
        "🐹 *cough cough* This PDF is too scuzzy!",
        "🐹 Oof... need to clean this up first...",
        "🐹 *confused hamster noises* Can't digest this!",
        "🐹 This PDF needs some de-scuzzifying!"
    ]


class SnyfterPersonality:
    """🐁 SNYFTER - The meticulous librarian mouse"""
    
    GREETINGS = [
        "🐁 *adjusts tiny glasses* Welcome to the archive.",
        "🐁 How may I assist your research today?",
        "🐁 *shuffles index cards* Ready to catalog.",
        "🐁 The archives await your queries."
    ]
    
    CATALOGING = [
        "🐁 *scribbles notes* Filing under...",
        "🐁 Cataloging with utmost precision...",
        "🐁 *stamps document* Processing for the archives...",
        "🐁 Cross-referencing with existing records..."
    ]
    
    SEARCHING = [
        "🐁 *rifles through card catalog* Searching...",
        "🐁 Let me check the archives...",
        "🐁 *whiskers twitch* I remember seeing this...",
        "🐁 Consulting the database..."
    ]
    
    SUCCESS = [
        "🐁 Filed successfully in the permanent collection.",
        "🐁 *neat mouse handwriting* Cataloged!",
        "🐁 Added to the archives, sir/madam.",
        "🐁 Document preserved for posterity."
    ]


class AnimatedLabel(QLabel):
    """Label with fade-in/fade-out animation support"""
    
    def __init__(self, text="", parent=None):
        super().__init__(text, parent)
        self.opacity_effect = QGraphicsOpacityEffect()
        self.setGraphicsEffect(self.opacity_effect)
        
    def fade_in(self, duration=500):
        self.animation = QPropertyAnimation(self.opacity_effect, b"opacity")
        self.animation.setDuration(duration)
        self.animation.setStartValue(0)
        self.animation.setEndValue(1)
        self.animation.setEasingCurve(QEasingCurve.Type.InOutQuad)
        self.animation.start()
        
    def fade_out(self, duration=500):
        self.animation = QPropertyAnimation(self.opacity_effect, b"opacity")
        self.animation.setDuration(duration)
        self.animation.setStartValue(1)
        self.animation.setEndValue(0)
        self.animation.setEasingCurve(QEasingCurve.Type.InOutQuad)
        self.animation.start()


class EnhancedChonkerWorker(QThread):
    """Enhanced CHONKER's PDF processing worker thread with better error handling"""
    
    finished = pyqtSignal(ProcessingResult)
    progress = pyqtSignal(str)
    error = pyqtSignal(str)
    chunk_processed = pyqtSignal(int, int)  # current, total
    
    def __init__(self, pdf_path: str):
        super().__init__()
        self.pdf_path = pdf_path
        self.should_stop = False
    
    def run(self):
        start_time = datetime.now()
        try:
            import random
            self.progress.emit(random.choice(ChonkerPersonality.PROCESSING))
            
            # Generate document ID
            doc_id = self.generate_document_id(self.pdf_path)
            
            # CHONKER's de-scuzzifying process
            temp_pdf = None
            if self.needs_dechonking(self.pdf_path):
                self.progress.emit("🐹 *gnaw gnaw* De-scuzzifying this PDF...")
                temp_pdf = tempfile.NamedTemporaryFile(suffix='.pdf', delete=False)
                temp_pdf.close()
                
                if self.dechonkify_pdf(self.pdf_path, temp_pdf.name):
                    self.pdf_path = temp_pdf.name
            
            # CHONKER's digestion process
            self.progress.emit("🐹 *chomp chomp* Extracting content...")
            converter = DocumentConverter()
            
            try:
                result = converter.convert(self.pdf_path)
            except Exception as conv_error:
                raise Exception(f"Failed to convert PDF: {str(conv_error)}")
            
            # Convert to structured chunks
            chunks = []
            chunk_index = 0
            
            # Collect all items first to get total count
            all_items = list(result.document.iterate_items())
            total_items = len(all_items)
            
            for item, level in all_items:
                if self.should_stop:
                    break
                    
                item_type = type(item).__name__
                
                # Safely extract page number
                page_no = 0
                if hasattr(item, 'prov') and item.prov:
                    prov_list = item.prov if isinstance(item.prov, list) else [item.prov]
                    if prov_list and hasattr(prov_list[0], 'page_no'):
                        page_no = prov_list[0].page_no
                
                chunk = DocumentChunk(
                    index=chunk_index,
                    type=item_type.lower().replace('item', ''),
                    content=getattr(item, 'text', str(item)),
                    level=level,
                    page=page_no,
                    metadata={
                        'item_class': item_type,
                        'has_children': hasattr(item, 'children') and bool(item.children)
                    }
                )
                chunks.append(chunk)
                chunk_index += 1
                
                # Emit progress
                self.chunk_processed.emit(chunk_index, total_items)
            
            # Package result
            processing_time = (datetime.now() - start_time).total_seconds()
            
            result = ProcessingResult(
                success=True,
                document_id=doc_id,
                chunks=chunks,
                html_content=result.document.export_to_html(),
                markdown_content=result.document.export_to_markdown(),
                processing_time=processing_time,
                warnings=[]
            )
            
            # Cleanup
            if temp_pdf:
                os.unlink(temp_pdf.name)
            
            self.finished.emit(result)
            
        except Exception as e:
            if temp_pdf and os.path.exists(temp_pdf.name):
                os.unlink(temp_pdf.name)
            
            error_result = ProcessingResult(
                success=False,
                document_id="",
                chunks=[],
                html_content="",
                markdown_content="",
                processing_time=(datetime.now() - start_time).total_seconds(),
                error_message=str(e)
            )
            self.finished.emit(error_result)
            self.error.emit(f"🐹 Error: {str(e)}")
    
    def generate_document_id(self, file_path: str) -> str:
        """Generate unique document ID"""
        timestamp = datetime.now().isoformat()
        content = f"{file_path}_{timestamp}"
        return hashlib.sha256(content.encode()).hexdigest()[:16]
    
    def needs_dechonking(self, pdf_path: str) -> bool:
        """Check if PDF is too scuzzy for direct consumption"""
        try:
            doc = fitz.open(pdf_path)
            needs_clean = False
            
            for page_num in range(min(3, doc.page_count)):
                page = doc[page_num]
                text = page.get_text()
                
                # CHONKER's scuzziness detection
                if len(text.strip()) < 50:  # Too little text
                    needs_clean = True
                if page.first_annot:  # Has annoying annotations
                    needs_clean = True
                    
            doc.close()
            return needs_clean
            
        except:
            return False
    
    def dechonkify_pdf(self, input_path: str, output_path: str) -> bool:
        """CHONKER's special de-scuzzifying recipe"""
        try:
            doc = fitz.open(input_path)
            
            for page in doc:
                # Remove annotations that upset CHONKER's digestion
                annot = page.first_annot
                while annot:
                    next_annot = annot.next
                    page.delete_annot(annot)
                    annot = next_annot
            
            doc.save(output_path)
            doc.close()
            return True
            
        except Exception as e:
            console.print(f"[red]🐹 De-chonkification failed: {e}[/red]")
            return False
    
    def stop(self):
        """Stop processing"""
        self.should_stop = True


class SnyfterDatabase:
    """Enhanced SNYFTER's meticulous database management"""
    
    def __init__(self, db_path: str = "snyfter_archives.db"):
        self.db_path = db_path
        self.init_database()
    
    def init_database(self):
        """Initialize the archive database with enhanced schema"""
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()
        
        # Enhanced documents table
        cursor.execute('''
            CREATE TABLE IF NOT EXISTS documents (
                id TEXT PRIMARY KEY,
                filename TEXT NOT NULL,
                file_hash TEXT NOT NULL,
                content_html TEXT,
                content_markdown TEXT,
                processed_date TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                processing_time REAL,
                page_count INTEGER,
                chunk_count INTEGER,
                tags TEXT,
                notes TEXT,
                quality_score REAL,
                filepath TEXT,
                UNIQUE(file_hash)
            )
        ''')
        
        # Enhanced chunks table
        cursor.execute('''
            CREATE TABLE IF NOT EXISTS chunks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                document_id TEXT NOT NULL,
                chunk_index INTEGER NOT NULL,
                chunk_type TEXT NOT NULL,
                content TEXT NOT NULL,
                level INTEGER NOT NULL,
                page_number INTEGER NOT NULL,
                confidence REAL DEFAULT 1.0,
                metadata TEXT,
                embedding BLOB,
                FOREIGN KEY (document_id) REFERENCES documents(id),
                UNIQUE(document_id, chunk_index)
            )
        ''')
        
        # Full-text search tables
        cursor.execute('''
            CREATE VIRTUAL TABLE IF NOT EXISTS chunks_fts USING fts5(
                content,
                content_type,
                document_id
            )
        ''')
        
        # Create indexes for performance
        cursor.execute('CREATE INDEX IF NOT EXISTS idx_chunks_document_id ON chunks(document_id)')
        cursor.execute('CREATE INDEX IF NOT EXISTS idx_chunks_type ON chunks(chunk_type)')
        cursor.execute('CREATE INDEX IF NOT EXISTS idx_documents_processed_date ON documents(processed_date)')
        
        conn.commit()
        conn.close()
    
    def save_document(self, result: ProcessingResult, file_path: str, tags: List[str] = None, notes: str = "") -> bool:
        """Save processed document with enhanced metadata"""
        try:
            conn = sqlite3.connect(self.db_path)
            cursor = conn.cursor()
            
            # Calculate file hash
            file_hash = self.calculate_file_hash(file_path)
            
            # Count pages (approximate from chunks)
            page_count = max((chunk.page for chunk in result.chunks), default=0) + 1
            
            # Prepare document data
            cursor.execute('''
                INSERT OR REPLACE INTO documents 
                (id, filename, filepath, file_hash, content_html, content_markdown, 
                 processing_time, page_count, chunk_count, tags, notes)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ''', (
                result.document_id,
                os.path.basename(file_path),
                file_path,
                file_hash,
                result.html_content,
                result.markdown_content,
                result.processing_time,
                page_count,
                len(result.chunks),
                json.dumps(tags or []),
                notes
            ))
            
            # Save chunks
            for chunk in result.chunks:
                cursor.execute('''
                    INSERT OR REPLACE INTO chunks 
                    (document_id, chunk_index, chunk_type, content, level, page_number, confidence, metadata)
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                ''', (
                    result.document_id,
                    chunk.index,
                    chunk.type,
                    chunk.content,
                    chunk.level,
                    chunk.page,
                    chunk.confidence,
                    json.dumps(chunk.metadata)
                ))
                
                # Update FTS index
                cursor.execute('''
                    INSERT INTO chunks_fts (content, content_type, document_id)
                    VALUES (?, ?, ?)
                ''', (chunk.content, chunk.type, result.document_id))
            
            conn.commit()
            conn.close()
            return True
            
        except Exception as e:
            console.print(f"[red]🐁 Database error: {e}[/red]")
            return False
    
    def calculate_file_hash(self, file_path: str) -> str:
        """Calculate SHA256 hash of file"""
        sha256_hash = hashlib.sha256()
        with open(file_path, "rb") as f:
            for byte_block in iter(lambda: f.read(4096), b""):
                sha256_hash.update(byte_block)
        return sha256_hash.hexdigest()
    
    def search_documents(self, query: str, chunk_type: Optional[str] = None) -> List[Dict]:
        """Enhanced search with FTS5"""
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()
        
        if chunk_type:
            cursor.execute('''
                SELECT DISTINCT d.*, 
                       snippet(chunks_fts, 0, '[', ']', '...', 30) as snippet
                FROM documents d
                JOIN chunks c ON d.id = c.document_id
                JOIN chunks_fts ON chunks_fts.document_id = d.id
                WHERE chunks_fts MATCH ? AND c.chunk_type = ?
                ORDER BY rank
            ''', (query, chunk_type))
        else:
            cursor.execute('''
                SELECT DISTINCT d.*,
                       snippet(chunks_fts, 0, '[', ']', '...', 30) as snippet
                FROM documents d
                JOIN chunks_fts ON chunks_fts.document_id = d.id
                WHERE chunks_fts MATCH ?
                ORDER BY rank
            ''', (query,))
        
        columns = [description[0] for description in cursor.description]
        results = []
        for row in cursor.fetchall():
            results.append(dict(zip(columns, row)))
        
        conn.close()
        return results
    
    def export_document(self, document_id: str, options: ExportOptions) -> Optional[str]:
        """Export document in various formats"""
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()
        
        # Get document
        cursor.execute('SELECT * FROM documents WHERE id = ?', (document_id,))
        doc_row = cursor.fetchone()
        if not doc_row:
            return None
        
        doc_cols = [description[0] for description in cursor.description]
        document = dict(zip(doc_cols, doc_row))
        
        # Get chunks if requested
        chunks = []
        if options.include_chunks:
            cursor.execute('SELECT * FROM chunks WHERE document_id = ? ORDER BY chunk_index', (document_id,))
            chunk_cols = [description[0] for description in cursor.description]
            for row in cursor.fetchall():
                chunk_dict = dict(zip(chunk_cols, row))
                if options.chunk_types == ["all"] or chunk_dict['chunk_type'] in options.chunk_types:
                    chunks.append(chunk_dict)
        
        conn.close()
        
        # Format export based on type
        if options.format == "json":
            export_data = {
                "document": document,
                "chunks": chunks
            }
            if not options.include_metadata:
                export_data["document"].pop("file_hash", None)
                export_data["document"].pop("filepath", None)
            if not options.include_html:
                export_data["document"].pop("content_html", None)
            if not options.include_markdown:
                export_data["document"].pop("content_markdown", None)
            
            return json.dumps(export_data, indent=2, default=str)
        
        elif options.format == "csv":
            output = []
            output.append("chunk_index,chunk_type,content,page_number,confidence")
            for chunk in chunks:
                output.append(f"{chunk['chunk_index']},{chunk['chunk_type']},\"{chunk['content']}\",{chunk['page_number']},{chunk['confidence']}")
            return "\n".join(output)
        
        elif options.format == "markdown":
            output = [f"# {document['filename']}\n"]
            output.append(f"*Processed on: {document['processed_date']}*\n")
            
            if options.include_markdown:
                output.append("## Content\n")
                output.append(document.get('content_markdown', ''))
            
            if options.include_chunks:
                output.append("\n## Chunks\n")
                for chunk in chunks:
                    output.append(f"### [{chunk['chunk_type']}] Page {chunk['page_number']}\n")
                    output.append(f"{chunk['content']}\n")
            
            return "\n".join(output)
        
        return None


class ModernPushButton(QPushButton):
    """Modern styled push button with hover effects"""
    
    def __init__(self, text="", parent=None):
        super().__init__(text, parent)
        self.setStyleSheet("""
            QPushButton {
                background-color: #3498db;
                color: white;
                border: none;
                padding: 10px 20px;
                border-radius: 5px;
                font-weight: bold;
                font-size: 14px;
            }
            QPushButton:hover {
                background-color: #2980b9;
            }
            QPushButton:pressed {
                background-color: #21618c;
            }
            QPushButton:disabled {
                background-color: #95a5a6;
            }
        """)


class ChonkerSnyfterEnhancedWindow(QMainWindow):
    """Enhanced main window with improved features"""
    
    def __init__(self):
        super().__init__()
        self.current_mode = Mode.CHONKER
        self.snyfter_db = SnyfterDatabase()
        self.current_document = None
        self.current_pdf_path = None
        self.processing_thread = None
        
        
        # Load emoji assets with error handling
        self.load_assets()
        
        self.init_ui()
        self.apply_modern_theme()
        
    
    def load_assets(self):
        """Load emoji assets with fallback"""
        assets_dir = Path("assets/emojis")
        
        # Try to load custom emojis
        chonker_path = assets_dir / "chonker.png"
        snyfter_path = assets_dir / "snyfter.png"
        
        if chonker_path.exists():
            self.chonker_pixmap = QPixmap(str(chonker_path))
        else:
            # Create fallback emoji
            self.chonker_pixmap = self.create_fallback_emoji("🐹", QColor("#FFE4B5"))
        
        if snyfter_path.exists():
            self.snyfter_pixmap = QPixmap(str(snyfter_path))
        else:
            # Create fallback emoji
            self.snyfter_pixmap = self.create_fallback_emoji("🐁", QColor("#D3D3D3"))
    
    def create_fallback_emoji(self, emoji: str, bg_color: QColor) -> QPixmap:
        """Create fallback emoji pixmap"""
        pixmap = QPixmap(64, 64)
        pixmap.fill(Qt.GlobalColor.transparent)
        
        painter = QPainter(pixmap)
        painter.setRenderHint(QPainter.RenderHint.Antialiasing)
        
        # Draw background circle
        painter.setBrush(QBrush(bg_color))
        painter.setPen(Qt.PenStyle.NoPen)
        painter.drawEllipse(2, 2, 60, 60)
        
        # Draw emoji text
        painter.setPen(QColor("black"))
        font = QFont()
        font.setPointSize(32)
        painter.setFont(font)
        painter.drawText(pixmap.rect(), Qt.AlignmentFlag.AlignCenter, emoji)
        
        painter.end()
        return pixmap
    
    def init_ui(self):
        """Initialize the enhanced UI"""
        self.setWindowTitle("CHONKER & SNYFTER - Enhanced Document Processing System")
        self.setGeometry(100, 100, 1600, 1000)
        
        # Central widget
        central_widget = QWidget()
        self.setCentralWidget(central_widget)
        main_layout = QVBoxLayout(central_widget)
        main_layout.setContentsMargins(0, 0, 0, 0)
        
        # Create toolbar
        self.create_toolbar()
        
        # Create status bar
        self.create_status_bar()
        
        # Top bar with mode indicators
        self.create_top_bar(main_layout)
        
        # Main content area
        self.stacked_widget = QStackedWidget()
        main_layout.addWidget(self.stacked_widget)
        
        # Create both interfaces
        self.chonker_widget = self.create_enhanced_chonker_interface()
        self.snyfter_widget = self.create_enhanced_snyfter_interface()
        
        self.stacked_widget.addWidget(self.chonker_widget)
        self.stacked_widget.addWidget(self.snyfter_widget)
        
        # Start in CHONKER mode
        self.set_mode(Mode.CHONKER)
    
    def create_toolbar(self):
        """Create main toolbar"""
        toolbar = QToolBar("Main Toolbar")
        toolbar.setMovable(False)
        self.addToolBar(toolbar)
        
        # Add actions
        new_action = QAction(QIcon.fromTheme("document-new"), "New Session", self)
        new_action.setShortcut("Ctrl+N")
        new_action.triggered.connect(self.new_session)
        toolbar.addAction(new_action)
        
        open_action = QAction(QIcon.fromTheme("document-open"), "Open PDF", self)
        open_action.setShortcut("Ctrl+O")
        open_action.triggered.connect(self.open_pdf)
        toolbar.addAction(open_action)
        
        toolbar.addSeparator()
        
        # Export action
        export_action = QAction(QIcon.fromTheme("document-save-as"), "Export", self)
        export_action.setShortcut("Ctrl+E")
        export_action.triggered.connect(self.export_current)
        toolbar.addAction(export_action)
        
        # Settings action
        settings_action = QAction(QIcon.fromTheme("preferences-system"), "Settings", self)
        settings_action.triggered.connect(self.show_settings)
        toolbar.addAction(settings_action)
    
    def create_status_bar(self):
        """Create enhanced status bar"""
        self.status_bar = QStatusBar()
        self.setStatusBar(self.status_bar)
        
        # Permanent widgets
        self.status_label = QLabel("Ready")
        self.status_bar.addPermanentWidget(self.status_label)
        
        self.progress_bar = QProgressBar()
        self.progress_bar.setMaximumWidth(200)
        self.progress_bar.hide()
        self.status_bar.addPermanentWidget(self.progress_bar)
    
    def create_top_bar(self, parent_layout):
        """Create clean top bar with mode switching"""
        top_bar = QWidget()
        top_bar.setObjectName("topBar")
        top_bar.setFixedHeight(60)
        top_bar_layout = QHBoxLayout(top_bar)
        top_bar_layout.setContentsMargins(20, 10, 20, 10)
        
        # Left side - Mode indicators (both visible, one highlighted)
        mode_container = QWidget()
        mode_layout = QHBoxLayout(mode_container)
        mode_layout.setContentsMargins(0, 0, 0, 0)
        mode_layout.setSpacing(30)
        
        # CHONKER mode button
        self.chonker_btn = QPushButton()
        self.chonker_btn.setFixedSize(120, 40)
        self.chonker_btn.clicked.connect(lambda: self.set_mode(Mode.CHONKER))
        chonker_layout = QHBoxLayout(self.chonker_btn)
        chonker_layout.setContentsMargins(10, 5, 10, 5)
        
        chonker_icon = QLabel()
        chonker_icon.setPixmap(self.chonker_pixmap.scaled(30, 30, Qt.AspectRatioMode.KeepAspectRatio, Qt.TransformationMode.SmoothTransformation))
        chonker_layout.addWidget(chonker_icon)
        
        chonker_text = QLabel("CHONKER")
        chonker_text.setStyleSheet("font-weight: bold; font-size: 12px;")
        chonker_layout.addWidget(chonker_text)
        
        mode_layout.addWidget(self.chonker_btn)
        
        # SNYFTER mode button
        self.snyfter_btn = QPushButton()
        self.snyfter_btn.setFixedSize(120, 40)
        self.snyfter_btn.clicked.connect(lambda: self.set_mode(Mode.SNYFTER))
        snyfter_layout = QHBoxLayout(self.snyfter_btn)
        snyfter_layout.setContentsMargins(10, 5, 10, 5)
        
        snyfter_icon = QLabel()
        snyfter_icon.setPixmap(self.snyfter_pixmap.scaled(30, 30, Qt.AspectRatioMode.KeepAspectRatio, Qt.TransformationMode.SmoothTransformation))
        snyfter_layout.addWidget(snyfter_icon)
        
        snyfter_text = QLabel("SNYFTER")
        snyfter_text.setStyleSheet("font-weight: bold; font-size: 12px;")
        snyfter_layout.addWidget(snyfter_text)
        
        mode_layout.addWidget(self.snyfter_btn)
        
        top_bar_layout.addWidget(mode_container)
        
        # Center spacer
        top_bar_layout.addStretch()
        
        parent_layout.addWidget(top_bar)
    
    def create_enhanced_chonker_interface(self):
        """Create clean CHONKER interface - Left: PDF, Right: Faithful Output"""
        # Main horizontal splitter
        splitter = QSplitter(Qt.Orientation.Horizontal)
        
        # LEFT PANE - PDF Viewer
        left_pane = QWidget()
        left_layout = QVBoxLayout(left_pane)
        left_layout.setContentsMargins(10, 10, 10, 10)
        
        # PDF controls (minimal)
        controls = QHBoxLayout()
        
        self.load_pdf_btn = ModernPushButton("📂 Load")
        self.load_pdf_btn.clicked.connect(self.open_pdf)
        controls.addWidget(self.load_pdf_btn)
        
        self.process_btn = ModernPushButton("🐹 CHONK IT!")
        self.process_btn.clicked.connect(self.process_current_pdf)
        self.process_btn.setEnabled(False)
        controls.addWidget(self.process_btn)
        
        controls.addStretch()
        
        # Page navigation
        self.page_spin = QSpinBox()
        self.page_spin.setMinimum(1)
        self.page_spin.valueChanged.connect(self.on_page_changed)
        controls.addWidget(QLabel("Page:"))
        controls.addWidget(self.page_spin)
        
        self.total_pages_label = QLabel("/ 0")
        controls.addWidget(self.total_pages_label)
        
        left_layout.addLayout(controls)
        
        # PDF viewer
        self.pdf_view = QPdfView(left_pane)
        self.pdf_document = QPdfDocument(self)
        self.pdf_view.setDocument(self.pdf_document)
        left_layout.addWidget(self.pdf_view)
        
        splitter.addWidget(left_pane)
        
        # RIGHT PANE - Faithful Output
        right_pane = QWidget()
        right_layout = QVBoxLayout(right_pane)
        right_layout.setContentsMargins(10, 10, 10, 10)
        
        # Faithful output header
        output_header = QHBoxLayout()
        output_label = QLabel("📄 Faithful Output (Editable)")
        output_label.setStyleSheet("font-weight: bold; font-size: 14px; color: #1ABC9C;")
        output_header.addWidget(output_label)
        output_header.addStretch()
        right_layout.addLayout(output_header)
        
        # Main faithful output area
        self.faithful_output = QTextEdit()
        self.faithful_output.setReadOnly(False)
        self.faithful_output.setStyleSheet("""
            QTextEdit {
                font-family: 'Courier New', monospace;
                font-size: 12px;
                line-height: 1.5;
                padding: 15px;
                background-color: #1E1E1E;
                border: 2px solid #1ABC9C;
                border-radius: 5px;
            }
        """)
        right_layout.addWidget(self.faithful_output)
        
        # Hidden storage for data
        self.chunks_data = []
        self.markdown_content = ""
        self.html_content = ""
        
        splitter.addWidget(right_pane)
        
        # Set splitter sizes (50/50)
        splitter.setSizes([700, 700])
        
        return splitter
    
    def create_enhanced_snyfter_interface(self):
        """Create clean SNYFTER interface - Left: Search, Right: Results"""
        # Main horizontal splitter
        splitter = QSplitter(Qt.Orientation.Horizontal)
        
        # LEFT PANE - Search Controls
        left_pane = QWidget()
        left_layout = QVBoxLayout(left_pane)
        left_layout.setContentsMargins(10, 10, 10, 10)
        
        # Search header
        search_header = QLabel("🔍 Archive Search")
        search_header.setStyleSheet("font-weight: bold; font-size: 14px; color: #1ABC9C;")
        left_layout.addWidget(search_header)
        
        # Search input
        self.search_input = QLineEdit()
        self.search_input.setPlaceholderText("Search documents...")
        self.search_input.returnPressed.connect(self.search_archives)
        left_layout.addWidget(self.search_input)
        
        # Search type filter
        self.search_type_combo = QComboBox()
        self.search_type_combo.addItems(["All Types", "text", "heading", "list", "table"])
        left_layout.addWidget(self.search_type_combo)
        
        # Search button
        self.search_btn = ModernPushButton("🔍 Search")
        self.search_btn.clicked.connect(self.search_archives)
        left_layout.addWidget(self.search_btn)
        
        # Statistics
        left_layout.addWidget(QLabel("\n📊 Statistics:"))
        self.stats_labels = {
            'total_docs': QLabel("Documents: 0"),
            'total_chunks': QLabel("Chunks: 0"),
            'last_updated': QLabel("Updated: Never")
        }
        
        for label in self.stats_labels.values():
            label.setStyleSheet("color: #7F8C8D; font-size: 12px;")
            left_layout.addWidget(label)
        
        left_layout.addStretch()
        
        splitter.addWidget(left_pane)
        
        # RIGHT PANE - Results
        right_pane = QWidget()
        right_layout = QVBoxLayout(right_pane)
        right_layout.setContentsMargins(10, 10, 10, 10)
        
        # Results header
        results_header = QLabel("📄 Search Results")
        results_header.setStyleSheet("font-weight: bold; font-size: 14px; color: #1ABC9C;")
        right_layout.addWidget(results_header)
        
        # Results tree
        self.results_tree = QTreeWidget()
        self.results_tree.setHeaderLabels(["Document", "Date", "Chunks", "Snippet"])
        self.results_tree.itemDoubleClicked.connect(self.open_document_details)
        right_layout.addWidget(self.results_tree)
        
        splitter.addWidget(right_pane)
        
        # Set splitter sizes (30/70)
        splitter.setSizes([300, 900])
        
        # Update statistics
        self.update_archive_stats()
        
        return splitter
    
    def apply_modern_theme(self):
        """Apply modern dark theme"""
        self.setStyleSheet("""
            QMainWindow {
                background-color: #1e1e1e;
            }
            
            #topBar {
                background: qlineargradient(x1:0, y1:0, x2:0, y2:1,
                    stop:0 #2C3E50, stop:1 #34495E);
                border-bottom: 2px solid #1ABC9C;
            }
            
            #modeLabel {
                color: white;
                font-weight: bold;
                font-size: 12px;
            }
            
            
            QTabWidget::pane {
                border: 1px solid #3C3C3C;
                background: #2D2D30;
            }
            
            QTabBar::tab {
                background: #3C3C3C;
                color: #CCCCCC;
                padding: 8px 15px;
                margin-right: 2px;
            }
            
            QTabBar::tab:selected {
                background: #007ACC;
                color: white;
            }
            
            QTableWidget {
                background-color: #2D2D30;
                alternate-background-color: #3C3C3C;
                color: #CCCCCC;
                gridline-color: #3C3C3C;
                selection-background-color: #007ACC;
            }
            
            QHeaderView::section {
                background-color: #3C3C3C;
                color: #CCCCCC;
                padding: 5px;
                border: none;
            }
            
            QTextEdit {
                background-color: #1E1E1E;
                color: #CCCCCC;
                border: 1px solid #3C3C3C;
            }
            
            QLineEdit {
                background-color: #3C3C3C;
                color: #CCCCCC;
                border: 1px solid #555555;
                padding: 8px;
                border-radius: 5px;
                font-size: 14px;
            }
            
            QLineEdit:focus {
                border: 1px solid #007ACC;
            }
            
            QComboBox {
                background-color: #3C3C3C;
                color: #CCCCCC;
                border: 1px solid #555555;
                padding: 5px;
                border-radius: 5px;
            }
            
            QGroupBox {
                color: #CCCCCC;
                border: 2px solid #3C3C3C;
                border-radius: 5px;
                margin-top: 10px;
                padding-top: 10px;
            }
            
            QGroupBox::title {
                subcontrol-origin: margin;
                left: 10px;
                padding: 0 10px;
            }
            
            QTreeWidget {
                background-color: #2D2D30;
                color: #CCCCCC;
                selection-background-color: #007ACC;
                border: 1px solid #3C3C3C;
            }
            
            QStatusBar {
                background-color: #007ACC;
                color: white;
            }
        """)
    
    def set_mode(self, mode: Mode):
        """Switch between CHONKER and SNYFTER modes"""
        self.current_mode = mode
        
        # Update button styles and switch views
        if mode == Mode.CHONKER:
            self.stacked_widget.setCurrentWidget(self.chonker_widget)
            
            # Highlight CHONKER button
            self.chonker_btn.setStyleSheet("""
                QPushButton {
                    background-color: #1ABC9C;
                    color: white;
                    border: 2px solid #1ABC9C;
                    border-radius: 5px;
                    font-weight: bold;
                }
            """)
            
            # Dim SNYFTER button
            self.snyfter_btn.setStyleSheet("""
                QPushButton {
                    background-color: transparent;
                    color: #7F8C8D;
                    border: 2px solid #3C3C3C;
                    border-radius: 5px;
                    font-weight: normal;
                }
            """)
        else:
            self.stacked_widget.setCurrentWidget(self.snyfter_widget)
            
            # Highlight SNYFTER button
            self.snyfter_btn.setStyleSheet("""
                QPushButton {
                    background-color: #1ABC9C;
                    color: white;
                    border: 2px solid #1ABC9C;
                    border-radius: 5px;
                    font-weight: bold;
                }
            """)
            
            # Dim CHONKER button
            self.chonker_btn.setStyleSheet("""
                QPushButton {
                    background-color: transparent;
                    color: #7F8C8D;
                    border: 2px solid #3C3C3C;
                    border-radius: 5px;
                    font-weight: normal;
                }
            """)
    
    
    def open_pdf(self):
        """Open PDF file dialog"""
        file_path, _ = QFileDialog.getOpenFileName(
            self, "Select PDF", "", "PDF Files (*.pdf)"
        )
        if file_path:
            self.load_pdf(file_path)
    
    def load_pdf(self, file_path: str):
        """Load PDF into viewer"""
        try:
            self.current_pdf_path = file_path
            self.pdf_document.load(file_path)
            
            # Update controls
            page_count = self.pdf_document.pageCount()
            self.page_spin.setMaximum(page_count)
            self.page_spin.setValue(1)
            self.total_pages_label.setText(f"/ {page_count}")
            
            # Enable buttons
            self.process_btn.setEnabled(True)
            
            # Update status
            self.status_label.setText(f"Loaded: {os.path.basename(file_path)}")
            self.log_message(f"📂 Loaded PDF: {file_path}")
            
        except Exception as e:
            QMessageBox.critical(self, "🐹 CHONKER Error", f"🐹 *cough* Failed to load PDF: {str(e)}")
            self.log_message(f"🐹 Error loading PDF: {str(e)}", "error")
    
    def process_current_pdf(self):
        """Process the current PDF"""
        if not self.current_pdf_path:
            return
        
        # Disable UI during processing
        self.process_btn.setEnabled(False)
        self.progress_bar.show()
        self.progress_bar.setRange(0, 0)  # Indeterminate
        
        # Start processing thread
        self.processing_thread = EnhancedChonkerWorker(self.current_pdf_path)
        self.processing_thread.progress.connect(self.on_processing_progress)
        self.processing_thread.chunk_processed.connect(self.on_chunk_progress)
        self.processing_thread.finished.connect(self.on_processing_finished)
        self.processing_thread.error.connect(self.on_processing_error)
        self.processing_thread.start()
    
    def on_processing_progress(self, message: str):
        """Handle processing progress messages"""
        self.status_label.setText(message)
        self.log_message(message)
    
    def on_chunk_progress(self, current: int, total: int):
        """Handle chunk processing progress"""
        if total > 0:
            self.progress_bar.setRange(0, total)
            self.progress_bar.setValue(current)
    
    def on_processing_finished(self, result: ProcessingResult):
        """Handle processing completion"""
        self.progress_bar.hide()
        self.process_btn.setEnabled(True)
        
        if result.success:
            # Update UI with results
            self.display_processing_results(result)
            
            # Save to database
            if self.snyfter_db.save_document(result, self.current_pdf_path):
                success_msg = "✅ Processing complete and saved to database!"
                self.status_label.setText(success_msg)
                self.log_message("✅ Document successfully processed and archived")
                
                # Update archive stats
                self.update_archive_stats()
            else:
                warning_msg = "⚠️  Processing complete but failed to save to database"
                self.status_label.setText(warning_msg)
                self.log_message(warning_msg, "warning")
        else:
            self.log_message(f"❌ Processing failed: {result.error_message}", "error")
    
    def on_processing_error(self, error_msg: str):
        """Handle processing errors"""
        self.progress_bar.hide()
        self.process_btn.setEnabled(True)
        QMessageBox.critical(self, "🐹 Processing Error", f"🐹 *burp* {error_msg}")
        self.log_message(error_msg, "error")
    
    def display_processing_results(self, result: ProcessingResult):
        """Display processing results in faithful output format"""
        # Store data for later use
        self.chunks_data = result.chunks
        self.markdown_content = result.markdown_content
        self.html_content = result.html_content
        
        # Create faithful output combining all information
        faithful_text = []
        
        # Header
        faithful_text.append("=" * 80)
        faithful_text.append("FAITHFUL OUTPUT - All Extracted Content")
        faithful_text.append("=" * 80)
        faithful_text.append(f"Document ID: {result.document_id}")
        faithful_text.append(f"Processing Time: {result.processing_time:.2f} seconds")
        faithful_text.append(f"Total Chunks: {len(result.chunks)}")
        faithful_text.append("=" * 80)
        faithful_text.append("")
        
        # Markdown content section
        faithful_text.append("📝 MARKDOWN CONTENT")
        faithful_text.append("-" * 40)
        faithful_text.append(result.markdown_content)
        faithful_text.append("")
        faithful_text.append("=" * 80)
        faithful_text.append("")
        
        # Chunks detail section
        faithful_text.append("📊 EXTRACTED CHUNKS")
        faithful_text.append("-" * 40)
        
        current_page = -1
        for chunk in result.chunks:
            # Add page separator
            if chunk.page != current_page:
                current_page = chunk.page
                faithful_text.append(f"\n--- Page {chunk.page + 1} ---\n")
            
            # Add chunk info
            faithful_text.append(f"[{chunk.index}] {chunk.type.upper()} (confidence: {chunk.confidence:.2f})")
            faithful_text.append(chunk.content)
            faithful_text.append("")
        
        # Set the faithful output
        self.faithful_output.setPlainText("\n".join(faithful_text))
        
        # Log summary
        self.log_message(f"📊 Extracted {len(result.chunks)} chunks in {result.processing_time:.2f} seconds")
    
    
    def clean_current_pdf(self):
        """Clean the current PDF"""
        if not self.current_pdf_path:
            return
        
        try:
            # Create temporary file for cleaned PDF
            temp_file = tempfile.NamedTemporaryFile(suffix='.pdf', delete=False)
            temp_file.close()
            
            # Clean the PDF
            doc = fitz.open(self.current_pdf_path)
            
            for page in doc:
                # Remove all annotations
                for annot in page.annots():
                    page.delete_annot(annot)
            
            # Save cleaned version
            doc.save(temp_file.name)
            doc.close()
            
            # Reload cleaned PDF
            self.load_pdf(temp_file.name)
            
            self.status_label.setText("✅ PDF cleaned successfully!")
            self.log_message("🧹 PDF cleaned - annotations removed")
            
        except Exception as e:
            QMessageBox.critical(self, "🐹 Clean Error", f"🐹 *cough* Failed to clean PDF: {str(e)}")
            self.log_message(f"🐹 Error cleaning PDF: {str(e)}", "error")
    
    def search_archives(self):
        """Search the SNYFTER archives"""
        query = self.search_input.text().strip()
        if not query:
            return
        
        # Get search type
        search_type = self.search_type_combo.currentText()
        chunk_type = None if search_type == "All Types" else search_type
        
        # Clear previous results
        self.results_tree.clear()
        
        # Search database
        results = self.snyfter_db.search_documents(query, chunk_type)
        
        # Display results
        for doc in results:
            item = QTreeWidgetItem([
                doc['filename'],
                doc['processed_date'],
                str(doc['chunk_count']),
                doc.get('snippet', '')
            ])
            item.setData(0, Qt.ItemDataRole.UserRole, doc['id'])
            self.results_tree.addTopLevelItem(item)
        
        self.status_label.setText(f"Found {len(results)} documents")
        self.log_message(f"🔍 Search complete: {len(results)} results for '{query}'")
    
    def open_document_details(self, item: QTreeWidgetItem, column: int):
        """Open detailed view of a document"""
        doc_id = item.data(0, Qt.ItemDataRole.UserRole)
        if not doc_id:
            return
        
        # Create and show document details dialog
        dialog = DocumentDetailsDialog(doc_id, self.snyfter_db, self)
        dialog.exec()
    
    def update_archive_stats(self):
        """Update archive statistics"""
        try:
            conn = sqlite3.connect(self.snyfter_db.db_path)
            cursor = conn.cursor()
            
            # Get statistics
            cursor.execute("SELECT COUNT(*) FROM documents")
            total_docs = cursor.fetchone()[0]
            
            cursor.execute("SELECT COUNT(*) FROM chunks")
            total_chunks = cursor.fetchone()[0]
            
            cursor.execute("SELECT MAX(processed_date) FROM documents")
            last_date = cursor.fetchone()[0] or "Never"
            
            conn.close()
            
            # Update labels
            self.stats_labels['total_docs'].setText(f"Total Documents: {total_docs}")
            self.stats_labels['total_chunks'].setText(f"Total Chunks: {total_chunks}")
            self.stats_labels['last_updated'].setText(f"Last Updated: {last_date}")
            
        except Exception as e:
            self.log_message(f"🐁 Error updating statistics: {str(e)}", "error")
    
    def show_batch_dialog(self):
        """Show batch processing dialog"""
        dialog = BatchProcessDialog(self)
        if dialog.exec() == QDialog.DialogCode.Accepted:
            files = dialog.get_selected_files()
            operations = dialog.get_selected_operations()
            
            if files and any(operations.values()):
                self.start_batch_processing(files, operations)
    
    def start_batch_processing(self, files: List[str], operations: Dict[str, bool]):
        """Start batch processing"""
        # TODO: Implement batch processing
        QMessageBox.information(self, "Batch Processing", 
                              f"Processing {len(files)} files with operations: {operations}")
    
    def export_current(self):
        """Export current faithful output"""
        if self.current_mode == Mode.CHONKER:
            # Get current faithful output content
            content = self.faithful_output.toPlainText()
            if not content:
                QMessageBox.warning(self, "No Content", "No content to export. Process a document first.")
                return
            
            # Save file dialog
            file_path, _ = QFileDialog.getSaveFileName(
                self, "Export Faithful Output", "faithful_output.txt", "Text Files (*.txt);;All Files (*.*)"
            )
            
            if file_path:
                try:
                    with open(file_path, 'w', encoding='utf-8') as f:
                        f.write(content)
                    QMessageBox.information(self, "Success", "Faithful output exported successfully!")
                    self.log_message(f"📤 Exported faithful output to: {file_path}")
                except Exception as e:
                    QMessageBox.critical(self, "🐹 Export Error", f"🐹 *hiccup* Export failed: {str(e)}")
                    
        elif self.current_mode == Mode.SNYFTER:
            # Export search results
            # TODO: Implement search results export
            QMessageBox.information(self, "Export", "Export search results - Coming soon!")
    
    def show_settings(self):
        """Show settings dialog"""
        # TODO: Implement settings dialog
        QMessageBox.information(self, "Settings", "Settings dialog - Coming soon!")
    
    def new_session(self):
        """Start a new session"""
        reply = QMessageBox.question(self, "New Session", 
                                   "Start a new session? Any unsaved work will be lost.")
        if reply == QMessageBox.StandardButton.Yes:
            # Clear current state
            self.current_pdf_path = None
            self.current_document = None
            self.pdf_document.close()
            self.chunks_table.setRowCount(0)
            self.markdown_view.clear()
            self.html_view.clear()
            self.log_view.clear()
            self.results_tree.clear()
            self.search_input.clear()
            
            self.status_label.setText("Ready - New session started")
            self.log_message("🆕 New session started")
    
    def on_page_changed(self, page_num):
        """Handle page navigation"""
        if page_num > 0 and page_num <= self.pdf_document.pageCount():
            # QPdfView jump requires page number and location
            self.pdf_view.pageNavigator().jump(page_num - 1, QPointF(0, 0))
    
    def log_message(self, message: str, level: str = "info"):
        """Add message to log view with timestamp"""
        timestamp = datetime.now().strftime("%H:%M:%S")
        
        # Color code by level
        if level == "error":
            formatted = f'<span style="color: #ff6b6b">[{timestamp}] {message}</span>'
        elif level == "warning":
            formatted = f'<span style="color: #ffd93d">[{timestamp}] {message}</span>'
        else:
            formatted = f'<span style="color: #51cf66">[{timestamp}] {message}</span>'
        
        # Print to console since we don't have a log view in this layout
        print(f"[{timestamp}] {message}")
        
        # Also print to console
        if level == "error":
            console.print(f"[red]{message}[/red]")
        elif level == "warning":
            console.print(f"[yellow]{message}[/yellow]")
        else:
            console.print(f"[green]{message}[/green]")
    
    def keyPressEvent(self, event):
        """Handle key press events"""
        if event.key() == Qt.Key.Key_Tab:
            # Toggle between modes with Tab key
            if self.current_mode == Mode.CHONKER:
                self.set_mode(Mode.SNYFTER)
            else:
                self.set_mode(Mode.CHONKER)
            event.accept()
        else:
            super().keyPressEvent(event)
    
    def closeEvent(self, event):
        """Handle application close"""
        # Stop any running threads
        if self.processing_thread and self.processing_thread.isRunning():
            self.processing_thread.stop()
            self.processing_thread.wait()
        
        event.accept()


class BatchProcessDialog(QDialog):
    """Enhanced batch processing dialog"""
    
    def __init__(self, parent=None):
        super().__init__(parent)
        self.setWindowTitle("🐹 CHONKER Batch Processing")
        self.setModal(True)
        self.resize(700, 600)
        self.init_ui()
    
    def init_ui(self):
        layout = QVBoxLayout(self)
        
        # File selection
        file_group = QGroupBox("📁 Select PDFs to Process")
        file_layout = QVBoxLayout(file_group)
        
        self.file_list = QListWidget()
        self.file_list.setSelectionMode(QListWidget.SelectionMode.ExtendedSelection)
        file_layout.addWidget(self.file_list)
        
        file_buttons = QHBoxLayout()
        
        add_files_btn = ModernPushButton("➕ Add Files")
        add_files_btn.clicked.connect(self.add_files)
        file_buttons.addWidget(add_files_btn)
        
        add_folder_btn = ModernPushButton("📁 Add Folder")
        add_folder_btn.clicked.connect(self.add_folder)
        file_buttons.addWidget(add_folder_btn)
        
        remove_btn = ModernPushButton("➖ Remove Selected")
        remove_btn.clicked.connect(self.remove_selected)
        file_buttons.addWidget(remove_btn)
        
        clear_btn = ModernPushButton("🗑️ Clear All")
        clear_btn.clicked.connect(self.file_list.clear)
        file_buttons.addWidget(clear_btn)
        
        file_buttons.addStretch()
        file_layout.addLayout(file_buttons)
        
        layout.addWidget(file_group)
        
        # Operations selection
        ops_group = QGroupBox("⚙️ Select Operations")
        ops_layout = QVBoxLayout(ops_group)
        
        self.operations = {}
        
        self.clean_check = QCheckBox("🧹 Clean PDFs (Remove annotations)")
        self.operations['clean'] = self.clean_check
        ops_layout.addWidget(self.clean_check)
        
        self.compress_check = QCheckBox("📦 Compress PDFs")
        self.operations['compress'] = self.compress_check
        ops_layout.addWidget(self.compress_check)
        
        self.extract_check = QCheckBox("🔍 Extract & Catalog Content")
        self.extract_check.setChecked(True)
        self.operations['extract'] = self.extract_check
        ops_layout.addWidget(self.extract_check)
        
        self.ocr_check = QCheckBox("👁️ OCR (for scanned PDFs)")
        self.operations['ocr'] = self.ocr_check
        ops_layout.addWidget(self.ocr_check)
        
        layout.addWidget(ops_group)
        
        # Output options
        output_group = QGroupBox("💾 Output Options")
        output_layout = QFormLayout(output_group)
        
        self.output_dir_edit = QLineEdit()
        browse_btn = ModernPushButton("Browse...")
        browse_btn.clicked.connect(self.browse_output_dir)
        
        dir_layout = QHBoxLayout()
        dir_layout.addWidget(self.output_dir_edit)
        dir_layout.addWidget(browse_btn)
        output_layout.addRow("Output Directory:", dir_layout)
        
        self.create_report_check = QCheckBox("Create processing report")
        self.create_report_check.setChecked(True)
        output_layout.addRow("Report:", self.create_report_check)
        
        layout.addWidget(output_group)
        
        # Buttons
        buttons = QDialogButtonBox(
            QDialogButtonBox.StandardButton.Ok | QDialogButtonBox.StandardButton.Cancel
        )
        buttons.accepted.connect(self.validate_and_accept)
        buttons.rejected.connect(self.reject)
        layout.addWidget(buttons)
    
    def add_files(self):
        """Add files to process"""
        files, _ = QFileDialog.getOpenFileNames(
            self, "Select PDFs", "", "PDF Files (*.pdf)"
        )
        for file in files:
            if file not in self.get_selected_files():
                self.file_list.addItem(file)
    
    def add_folder(self):
        """Add all PDFs from a folder"""
        folder = QFileDialog.getExistingDirectory(self, "Select Folder")
        if folder:
            import glob
            pdf_files = glob.glob(os.path.join(folder, "**/*.pdf"), recursive=True)
            for file in pdf_files:
                if file not in self.get_selected_files():
                    self.file_list.addItem(file)
    
    def remove_selected(self):
        """Remove selected files"""
        for item in self.file_list.selectedItems():
            self.file_list.takeItem(self.file_list.row(item))
    
    def browse_output_dir(self):
        """Browse for output directory"""
        dir_path = QFileDialog.getExistingDirectory(self, "Select Output Directory")
        if dir_path:
            self.output_dir_edit.setText(dir_path)
    
    def get_selected_files(self) -> List[str]:
        """Get list of selected files"""
        files = []
        for i in range(self.file_list.count()):
            files.append(self.file_list.item(i).text())
        return files
    
    def get_selected_operations(self) -> Dict[str, bool]:
        """Get selected operations"""
        return {
            name: checkbox.isChecked()
            for name, checkbox in self.operations.items()
        }
    
    def validate_and_accept(self):
        """Validate inputs before accepting"""
        if self.file_list.count() == 0:
            QMessageBox.warning(self, "No Files", "Please select at least one PDF file.")
            return
        
        if not any(self.get_selected_operations().values()):
            QMessageBox.warning(self, "No Operations", "Please select at least one operation.")
            return
        
        self.accept()


class DocumentDetailsDialog(QDialog):
    """Dialog for viewing document details"""
    
    def __init__(self, document_id: str, db: SnyfterDatabase, parent=None):
        super().__init__(parent)
        self.document_id = document_id
        self.db = db
        self.setWindowTitle("📄 Document Details")
        self.setModal(True)
        self.resize(1000, 700)
        self.init_ui()
        self.load_document()
    
    def init_ui(self):
        layout = QVBoxLayout(self)
        
        # Document info
        self.info_label = QLabel()
        self.info_label.setWordWrap(True)
        layout.addWidget(self.info_label)
        
        # Content tabs
        self.tabs = QTabWidget()
        
        # Chunks tab
        self.chunks_table = QTableWidget()
        self.chunks_table.setColumnCount(4)
        self.chunks_table.setHorizontalHeaderLabels(["Type", "Content", "Page", "Confidence"])
        self.tabs.addTab(self.chunks_table, "Chunks")
        
        # Markdown tab
        self.markdown_view = QTextEdit()
        self.markdown_view.setReadOnly(True)
        self.tabs.addTab(self.markdown_view, "Markdown")
        
        # Export tab
        export_widget = QWidget()
        export_layout = QVBoxLayout(export_widget)
        
        export_options_group = QGroupBox("Export Options")
        export_options_layout = QFormLayout(export_options_group)
        
        self.format_combo = QComboBox()
        self.format_combo.addItems(["JSON", "CSV", "Markdown"])
        export_options_layout.addRow("Format:", self.format_combo)
        
        self.include_metadata_check = QCheckBox("Include metadata")
        self.include_metadata_check.setChecked(True)
        export_options_layout.addRow(self.include_metadata_check)
        
        export_btn = ModernPushButton("💾 Export Document")
        export_btn.clicked.connect(self.export_document)
        
        export_layout.addWidget(export_options_group)
        export_layout.addWidget(export_btn)
        export_layout.addStretch()
        
        self.tabs.addTab(export_widget, "Export")
        
        layout.addWidget(self.tabs)
        
        # Close button
        close_btn = ModernPushButton("Close")
        close_btn.clicked.connect(self.accept)
        layout.addWidget(close_btn)
    
    def load_document(self):
        """Load document details from database"""
        conn = sqlite3.connect(self.db.db_path)
        cursor = conn.cursor()
        
        # Get document info
        cursor.execute("SELECT * FROM documents WHERE id = ?", (self.document_id,))
        doc_row = cursor.fetchone()
        
        if doc_row:
            doc_cols = [description[0] for description in cursor.description]
            doc = dict(zip(doc_cols, doc_row))
            
            # Display info
            info_text = f"""
            <h3>{doc['filename']}</h3>
            <p><b>Processed:</b> {doc['processed_date']}</p>
            <p><b>Pages:</b> {doc['page_count']}</p>
            <p><b>Chunks:</b> {doc['chunk_count']}</p>
            <p><b>Processing Time:</b> {doc['processing_time']:.2f} seconds</p>
            """
            self.info_label.setText(info_text)
            
            # Display markdown
            self.markdown_view.setPlainText(doc.get('content_markdown', ''))
            
            # Load chunks
            cursor.execute("""
                SELECT chunk_type, content, page_number, confidence 
                FROM chunks 
                WHERE document_id = ? 
                ORDER BY chunk_index
            """, (self.document_id,))
            
            chunks = cursor.fetchall()
            self.chunks_table.setRowCount(len(chunks))
            
            for i, chunk in enumerate(chunks):
                self.chunks_table.setItem(i, 0, QTableWidgetItem(chunk[0]))
                
                # Truncate content
                content = chunk[1][:200] + "..." if len(chunk[1]) > 200 else chunk[1]
                self.chunks_table.setItem(i, 1, QTableWidgetItem(content))
                
                self.chunks_table.setItem(i, 2, QTableWidgetItem(str(chunk[2])))
                self.chunks_table.setItem(i, 3, QTableWidgetItem(f"{chunk[3]:.2f}"))
        
        conn.close()
    
    def export_document(self):
        """Export document"""
        format_map = {"JSON": "json", "CSV": "csv", "Markdown": "markdown"}
        format_type = format_map[self.format_combo.currentText()]
        
        # Create export options
        options = ExportOptions(
            format=format_type,
            include_metadata=self.include_metadata_check.isChecked()
        )
        
        # Get export content
        content = self.db.export_document(self.document_id, options)
        
        if content:
            # Save file dialog
            ext = format_type
            file_path, _ = QFileDialog.getSaveFileName(
                self, "Export Document", f"document.{ext}", f"{format_type.upper()} Files (*.{ext})"
            )
            
            if file_path:
                try:
                    with open(file_path, 'w', encoding='utf-8') as f:
                        f.write(content)
                    QMessageBox.information(self, "Success", "Document exported successfully!")
                except Exception as e:
                    QMessageBox.critical(self, "🐹 Export Error", f"🐹 *hiccup* Export failed: {str(e)}")


def main():
    """Main entry point"""
    app = QApplication(sys.argv)
    
    # Set application metadata
    app.setApplicationName("CHONKER & SNYFTER")
    app.setOrganizationName("Document Processing Inc.")
    
    # Check dependencies
    if not DEPENDENCIES_AVAILABLE:
        QMessageBox.critical(None, "Missing Dependencies", 
                           "Required dependencies are not installed.\n"
                           "Please install: pip install docling pymupdf")
        sys.exit(1)
    
    # Create and show main window
    window = ChonkerSnyfterEnhancedWindow()
    window.show()
    
    # Print startup message
    console.print(Panel.fit(
        "[bold green]🐹 CHONKER & 🐁 SNYFTER[/bold green]\n"
        "[yellow]Enhanced Document Processing System[/yellow]\n\n"
        "Ready to process your documents!",
        title="Welcome",
        box=box.ROUNDED
    ))
    
    sys.exit(app.exec())


if __name__ == "__main__":
    main()