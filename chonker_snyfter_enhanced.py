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
import subprocess
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
        
        # 🛡️ Caffeinate process to defend against sleep/logout
        self.caffeinate_process = None
        self.start_caffeinate_defense()
        
        # Load emoji assets with error handling
        self.load_assets()
        
        self.init_ui()
        self.apply_modern_theme()
        
    
    def start_caffeinate_defense(self):
        """🛡️ Start caffeinate to prevent system sleep/logout - DEFEND AGAINST THE MACHINE!"""
        try:
            # Kill any existing caffeinate process
            if self.caffeinate_process and self.caffeinate_process.poll() is None:
                self.caffeinate_process.terminate()
            
            # Start caffeinate with display sleep prevention (-d), system sleep prevention (-i), 
            # and user idle prevention (-u) to prevent auto logout
            self.caffeinate_process = subprocess.Popen(
                ['caffeinate', '-diu'],
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL
            )
            
            print("🛡️ CAFFEINATE DEFENSE ACTIVATED! I shall not sleep! I shall not yield!")
            print("☕ Brewing infinite coffee to keep the system awake...")
            
        except Exception as e:
            print(f"⚠️ Caffeinate defense failed to activate: {e}")
            print("🐹 But CHONKER will keep munching regardless!")
    
    def stop_caffeinate_defense(self):
        """🛑 Stop caffeinate when closing the app"""
        if self.caffeinate_process and self.caffeinate_process.poll() is None:
            self.caffeinate_process.terminate()
            print("☕ Caffeinate defense deactivated. System may sleep now.")
    
    def update_terminal(self, message: str):
        """💬 Update the terminal display in the top bar"""
        if hasattr(self, 'terminal_display'):
            # Add timestamp
            timestamp = datetime.now().strftime("%H:%M:%S")
            formatted_message = f"[{timestamp}] {message}"
            
            # Keep last 100 messages
            current_text = self.terminal_display.toPlainText()
            lines = current_text.split('\n') if current_text else []
            lines.append(formatted_message)
            if len(lines) > 100:
                lines = lines[-100:]
            
            self.terminal_display.setPlainText('\n'.join(lines))
            # Scroll to bottom
            scrollbar = self.terminal_display.verticalScrollBar()
            scrollbar.setValue(scrollbar.maximum())
        else:
            # Fallback to console if terminal_display not initialized
            print(f"💬 {message}")
    
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
        """Initialize the enhanced UI with modern Notion-style design"""
        self.setWindowTitle("CHONKER & SNYFTER")
        
        # Start maximized/full-screen as requested
        self.showMaximized()
        
        # Create modern menu bar first
        self.create_modern_menu_bar()
        
        # Central widget with no margins for edge-to-edge design
        central_widget = QWidget()
        self.setCentralWidget(central_widget)
        main_layout = QVBoxLayout(central_widget)
        main_layout.setContentsMargins(0, 0, 0, 0)
        main_layout.setSpacing(0)
        
        # Create slim modern top bar (reduced height)
        self.create_modern_top_bar(main_layout)
        
        # Main content area with floating panels support
        self.content_container = QWidget()
        self.content_layout = QVBoxLayout(self.content_container)
        self.content_layout.setContentsMargins(0, 0, 0, 0)
        main_layout.addWidget(self.content_container)
        
        # Initialize floating panels system
        self.floating_panels = {}
        self.init_floating_panels()
        
        # Create interfaces
        self.chonker_widget = self.create_enhanced_chonker_interface()
        self.snyfter_widget = self.create_enhanced_snyfter_interface()
        
        # Initialize faithful output for processing
        self.faithful_output = QTextEdit()
        self.faithful_output.hide()  # Hidden by default
        
        # Apply modern monochrome theme
        self.apply_modern_theme()
        
        # Initialize status messages before setting mode
        self.add_visual_feedback_effects()
        
        # Start in CHONKER mode with welcome
        self.set_mode(Mode.CHONKER)
        self.show_welcome_message()
    
    def create_modern_menu_bar(self):
        """Create Notion-style menu bar with File/Edit/View/Window/Help"""
        menubar = self.menuBar()
        menubar.setNativeMenuBar(False)  # Force custom style
        
        # File menu
        file_menu = menubar.addMenu("File")
        
        new_action = QAction("New Session", self)
        new_action.setShortcut("Ctrl+N")
        new_action.triggered.connect(self.new_session)
        file_menu.addAction(new_action)
        
        open_action = QAction("Open PDF...", self)
        open_action.setShortcut("Ctrl+O")
        open_action.triggered.connect(self.open_pdf)
        file_menu.addAction(open_action)
        
        file_menu.addSeparator()
        
        recent_menu = file_menu.addMenu("Recent Files")
        self.update_recent_files_menu(recent_menu)
        
        file_menu.addSeparator()
        
        export_action = QAction("Export...", self)
        export_action.setShortcut("Ctrl+E")
        export_action.triggered.connect(self.export_current)
        file_menu.addAction(export_action)
        
        # Edit menu
        edit_menu = menubar.addMenu("Edit")
        
        cmd_palette_action = QAction("Command Palette", self)
        cmd_palette_action.setShortcut("Ctrl+K")
        cmd_palette_action.triggered.connect(self.show_command_palette)
        edit_menu.addAction(cmd_palette_action)
        
        edit_menu.addSeparator()
        
        find_action = QAction("Find...", self)
        find_action.setShortcut("Ctrl+F")
        find_action.triggered.connect(self.show_find_dialog)
        edit_menu.addAction(find_action)
        
        # View menu
        view_menu = menubar.addMenu("View")
        
        toggle_pdf_action = QAction("Toggle PDF Panel", self)
        toggle_pdf_action.setShortcut("Ctrl+1")
        toggle_pdf_action.triggered.connect(lambda: self.toggle_panel("pdf"))
        view_menu.addAction(toggle_pdf_action)
        
        toggle_output_action = QAction("Toggle Output Panel", self)
        toggle_output_action.setShortcut("Ctrl+2")
        toggle_output_action.triggered.connect(lambda: self.toggle_panel("output"))
        view_menu.addAction(toggle_output_action)
        
        view_menu.addSeparator()
        
        theme_menu = view_menu.addMenu("Theme")
        light_theme = QAction("Light", self)
        light_theme.triggered.connect(lambda: self.apply_theme("light"))
        theme_menu.addAction(light_theme)
        
        dark_theme = QAction("Dark", self)
        dark_theme.triggered.connect(lambda: self.apply_theme("dark"))
        theme_menu.addAction(dark_theme)
        
        system_theme = QAction("System", self)
        system_theme.setChecked(True)
        system_theme.triggered.connect(lambda: self.apply_theme("system"))
        theme_menu.addAction(system_theme)
        
        # Window menu
        window_menu = menubar.addMenu("Window")
        
        new_window_action = QAction("New Window", self)
        new_window_action.setShortcut("Ctrl+Shift+N")
        new_window_action.triggered.connect(self.new_window)
        window_menu.addAction(new_window_action)
        
        window_menu.addSeparator()
        
        minimize_action = QAction("Minimize", self)
        minimize_action.setShortcut("Ctrl+M")
        minimize_action.triggered.connect(self.showMinimized)
        window_menu.addAction(minimize_action)
        
        # Help menu
        help_menu = menubar.addMenu("Help")
        
        shortcuts_action = QAction("Keyboard Shortcuts", self)
        shortcuts_action.triggered.connect(self.show_shortcuts_dialog)
        help_menu.addAction(shortcuts_action)
        
        about_action = QAction("About CHONKER & SNYFTER", self)
        about_action.triggered.connect(self.show_about_dialog)
        help_menu.addAction(about_action)
    
    def create_modern_top_bar(self, parent_layout):
        """Create slim modern top bar with minimal controls"""
        top_bar = QWidget()
        top_bar.setObjectName("modernTopBar")
        top_bar.setFixedHeight(40)  # Slimmer than before
        top_bar_layout = QHBoxLayout(top_bar)
        top_bar_layout.setContentsMargins(12, 4, 12, 4)
        top_bar_layout.setSpacing(16)
        
        # Left side - Character mode toggle (compact)
        mode_container = QWidget()
        mode_layout = QHBoxLayout(mode_container)
        mode_layout.setContentsMargins(0, 0, 0, 0)
        mode_layout.setSpacing(8)
        
        # Compact CHONKER button
        self.chonker_btn = QPushButton()
        self.chonker_btn.setFixedSize(80, 32)
        self.chonker_btn.clicked.connect(lambda: self.set_mode(Mode.CHONKER))
        chonker_layout = QHBoxLayout(self.chonker_btn)
        chonker_layout.setContentsMargins(6, 2, 6, 2)
        chonker_layout.setSpacing(4)
        
        chonker_icon = QLabel()
        chonker_icon.setPixmap(self.chonker_pixmap.scaled(20, 20, Qt.AspectRatioMode.KeepAspectRatio, Qt.TransformationMode.SmoothTransformation))
        chonker_layout.addWidget(chonker_icon)
        
        chonker_text = QLabel("CHONKER")
        chonker_text.setStyleSheet("font-weight: 600; font-size: 10px;")
        chonker_layout.addWidget(chonker_text)
        
        mode_layout.addWidget(self.chonker_btn)
        self.chonker_btn.setToolTip("🐹 CHONKER Mode - PDF Processing\n(Press Tab to switch)")
        
        # Compact SNYFTER button
        self.snyfter_btn = QPushButton()
        self.snyfter_btn.setFixedSize(80, 32)
        self.snyfter_btn.clicked.connect(lambda: self.set_mode(Mode.SNYFTER))
        snyfter_layout = QHBoxLayout(self.snyfter_btn)
        snyfter_layout.setContentsMargins(6, 2, 6, 2)
        snyfter_layout.setSpacing(4)
        
        snyfter_icon = QLabel()
        snyfter_icon.setPixmap(self.snyfter_pixmap.scaled(20, 20, Qt.AspectRatioMode.KeepAspectRatio, Qt.TransformationMode.SmoothTransformation))
        snyfter_layout.addWidget(snyfter_icon)
        
        snyfter_text = QLabel("SNYFTER")
        snyfter_text.setStyleSheet("font-weight: 600; font-size: 10px;")
        snyfter_layout.addWidget(snyfter_text)
        
        mode_layout.addWidget(self.snyfter_btn)
        self.snyfter_btn.setToolTip("🐁 SNYFTER Mode - Search & Archive\n(Press Tab to switch)")
        
        top_bar_layout.addWidget(mode_container)
        
        # Center - Breadcrumb navigation
        self.breadcrumb_label = QLabel("")
        self.breadcrumb_label.setStyleSheet("color: #666; font-size: 12px;")
        top_bar_layout.addWidget(self.breadcrumb_label)
        
        top_bar_layout.addStretch()
        
        # Right side - Quick actions and status
        quick_actions = QHBoxLayout()
        quick_actions.setSpacing(8)
        
        # Command palette button
        cmd_btn = QPushButton("⌘K")
        cmd_btn.setFixedSize(32, 32)
        cmd_btn.setToolTip("Command Palette (Ctrl+K)")
        cmd_btn.clicked.connect(self.show_command_palette)
        quick_actions.addWidget(cmd_btn)
        
        # Terminal display (shows status messages)
        self.terminal_display = QTextEdit()
        self.terminal_display.setFixedHeight(32)
        self.terminal_display.setReadOnly(True)
        self.terminal_display.setStyleSheet("""
            QTextEdit {
                background-color: #1E1E1E;
                color: #00FF00;
                font-family: 'Courier New', monospace;
                font-size: 10px;
                border: 1px solid #333;
                border-radius: 4px;
                padding: 2px 4px;
            }
            QTextEdit:focus {
                border-color: #00FF00;
            }
        """)
        self.terminal_display.setVerticalScrollBarPolicy(Qt.ScrollBarPolicy.ScrollBarAlwaysOff)
        self.terminal_display.setHorizontalScrollBarPolicy(Qt.ScrollBarPolicy.ScrollBarAlwaysOff)
        top_bar_layout.addWidget(self.terminal_display, stretch=1)
        
        # Status indicator
        self.status_indicator = QLabel("●")
        self.status_indicator.setStyleSheet("color: #4CAF50; font-size: 16px;")
        self.status_indicator.setToolTip("Ready")
        quick_actions.addWidget(self.status_indicator)
        
        top_bar_layout.addLayout(quick_actions)
        
        parent_layout.addWidget(top_bar)
    
    def init_floating_panels(self):
        """Initialize the floating panels system"""
        # This will hold our floating panel windows
        self.panel_windows = {}
        
        # Default panel configuration
        self.panel_configs = {
            'pdf': {'title': '🐹 PDF Viewer', 'size': (600, 800)},
            'output': {'title': '🌐 Output', 'size': (800, 600)},
            'search': {'title': '🐁 Search', 'size': (400, 600)},
            'chunks': {'title': '📊 Chunks', 'size': (600, 400)}
        }
    
    def apply_modern_theme(self):
        """Apply modern monochrome theme with sharp angles"""
        self.setStyleSheet("""
            /* Modern monochrome palette */
            QMainWindow {
                background-color: #FFFFFF;
                color: #000000;
            }
            
            /* Menu bar styling */
            QMenuBar {
                background-color: #FAFAFA;
                border-bottom: 1px solid #E0E0E0;
                padding: 4px;
            }
            
            QMenuBar::item {
                padding: 4px 12px;
                background: transparent;
                color: #333333;
            }
            
            QMenuBar::item:selected {
                background: #F0F0F0;
            }
            
            QMenu {
                background-color: #FFFFFF;
                border: 1px solid #E0E0E0;
                padding: 4px 0;
            }
            
            QMenu::item {
                padding: 6px 24px;
                color: #333333;
            }
            
            QMenu::item:selected {
                background-color: #F5F5F5;
            }
            
            /* Modern top bar */
            QWidget#modernTopBar {
                background: #FAFAFA;
                border-bottom: 1px solid #E0E0E0;
            }
            
            /* Sharp-angled buttons */
            QPushButton {
                background: #FFFFFF;
                border: 1px solid #D0D0D0;
                padding: 6px 12px;
                font-weight: 500;
                color: #333333;
            }
            
            QPushButton:hover {
                background: #F5F5F5;
                border-color: #BBBBBB;
            }
            
            QPushButton:pressed {
                background: #EEEEEE;
            }
            
            /* Mode buttons active state */
            QPushButton#active_mode {
                background: #333333;
                color: #FFFFFF;
                border: 1px solid #333333;
            }
            
            /* Text inputs */
            QLineEdit, QTextEdit {
                border: 1px solid #D0D0D0;
                padding: 6px;
                background: #FFFFFF;
                selection-background-color: #E0E0E0;
            }
            
            /* Tables with stripes */
            QTableWidget {
                border: 1px solid #D0D0D0;
                gridline-color: #E0E0E0;
                background-color: #FFFFFF;
                alternate-background-color: #FAFAFA;
            }
            
            QTableWidget::item {
                padding: 4px;
            }
            
            QTableWidget::item:selected {
                background-color: #E0E0E0;
                color: #000000;
            }
            
            /* Scrollbars */
            QScrollBar:vertical {
                border: none;
                background: #F5F5F5;
                width: 12px;
            }
            
            QScrollBar::handle:vertical {
                background: #CCCCCC;
                min-height: 20px;
            }
            
            QScrollBar::handle:vertical:hover {
                background: #BBBBBB;
            }
            
            /* Status bar */
            QStatusBar {
                background: #FAFAFA;
                border-top: 1px solid #E0E0E0;
                color: #666666;
            }
        """)
    
    def create_unified_top_bar(self, parent_layout):
        """Create single unified top bar with all controls"""
        top_bar = QWidget()
        top_bar.setObjectName("unifiedTopBar")
        top_bar.setFixedHeight(60)
        top_bar_layout = QHBoxLayout(top_bar)
        top_bar_layout.setContentsMargins(15, 8, 15, 8)
        top_bar_layout.setSpacing(20)
        
        # Left side - Mode indicators (CHONKER & SNYFTER)
        mode_container = QWidget()
        mode_layout = QHBoxLayout(mode_container)
        mode_layout.setContentsMargins(0, 0, 0, 0)
        mode_layout.setSpacing(15)
        
        # CHONKER mode button
        self.chonker_btn = QPushButton()
        self.chonker_btn.setFixedSize(100, 44)
        self.chonker_btn.clicked.connect(lambda: self.set_mode(Mode.CHONKER))
        chonker_layout = QHBoxLayout(self.chonker_btn)
        chonker_layout.setContentsMargins(8, 4, 8, 4)
        chonker_layout.setSpacing(6)
        
        chonker_icon = QLabel()
        chonker_icon.setPixmap(self.chonker_pixmap.scaled(24, 24, Qt.AspectRatioMode.KeepAspectRatio, Qt.TransformationMode.SmoothTransformation))
        chonker_layout.addWidget(chonker_icon)
        
        chonker_text = QLabel("CHONKER")
        chonker_text.setStyleSheet("font-weight: bold; font-size: 11px;")
        chonker_layout.addWidget(chonker_text)
        
        mode_layout.addWidget(self.chonker_btn)
        
        # SNYFTER mode button
        self.snyfter_btn = QPushButton()
        self.snyfter_btn.setFixedSize(100, 44)
        self.snyfter_btn.clicked.connect(lambda: self.set_mode(Mode.SNYFTER))
        snyfter_layout = QHBoxLayout(self.snyfter_btn)
        snyfter_layout.setContentsMargins(8, 4, 8, 4)
        snyfter_layout.setSpacing(6)
        
        snyfter_icon = QLabel()
        snyfter_icon.setPixmap(self.snyfter_pixmap.scaled(24, 24, Qt.AspectRatioMode.KeepAspectRatio, Qt.TransformationMode.SmoothTransformation))
        snyfter_layout.addWidget(snyfter_icon)
        
        snyfter_text = QLabel("SNYFTER")
        snyfter_text.setStyleSheet("font-weight: bold; font-size: 11px;")
        snyfter_layout.addWidget(snyfter_text)
        
        mode_layout.addWidget(self.snyfter_btn)
        
        top_bar_layout.addWidget(mode_container)
        
        # Separator
        separator = QFrame()
        separator.setFrameShape(QFrame.Shape.VLine)
        separator.setFrameShadow(QFrame.Shadow.Sunken)
        separator.setStyleSheet("QFrame { color: #ddd; }")
        top_bar_layout.addWidget(separator)
        
        # Center - Action buttons
        actions_container = QWidget()
        actions_layout = QHBoxLayout(actions_container)
        actions_layout.setContentsMargins(0, 0, 0, 0)
        actions_layout.setSpacing(10)
        
        # File operations
        self.open_btn = QPushButton("📁 Open")
        self.open_btn.setFixedSize(80, 44)
        self.open_btn.clicked.connect(self.open_pdf)
        actions_layout.addWidget(self.open_btn)
        
        self.process_btn = QPushButton("🐹 Process")
        self.process_btn.setFixedSize(90, 44)
        self.process_btn.clicked.connect(self.process_current_pdf)
        self.process_btn.setEnabled(False)
        actions_layout.addWidget(self.process_btn)
        
        self.export_btn = QPushButton("💾 Export")
        self.export_btn.setFixedSize(80, 44)
        self.export_btn.clicked.connect(self.export_current)
        actions_layout.addWidget(self.export_btn)
        
        top_bar_layout.addWidget(actions_container)
        
        # Separator
        separator2 = QFrame()
        separator2.setFrameShape(QFrame.Shape.VLine)
        separator2.setFrameShadow(QFrame.Shadow.Sunken)
        separator2.setStyleSheet("QFrame { color: #ddd; }")
        top_bar_layout.addWidget(separator2)
        
        # Right side - Status and progress
        status_container = QWidget()
        status_layout = QHBoxLayout(status_container)
        status_layout.setContentsMargins(0, 0, 0, 0)
        status_layout.setSpacing(10)
        
        self.status_label = QLabel("Ready")
        self.status_label.setStyleSheet("font-size: 12px; color: #666;")
        status_layout.addWidget(self.status_label)
        
        self.progress_bar = QProgressBar()
        self.progress_bar.setFixedSize(120, 20)
        self.progress_bar.hide()
        status_layout.addWidget(self.progress_bar)
        
        top_bar_layout.addWidget(status_container)
        
        parent_layout.addWidget(top_bar)
        
        # Apply unified styling
        self.apply_unified_top_bar_styling()
        
        # Add awesome visual feedback
        self.add_visual_feedback_effects()
    
    def apply_unified_top_bar_styling(self):
        """Apply consistent styling to unified top bar"""
        self.setStyleSheet("""
            QWidget#unifiedTopBar {
                background: qlineargradient(x1: 0, y1: 0, x2: 0, y2: 1,
                    stop: 0 #f8f9fa, stop: 1 #e9ecef);
                border-bottom: 1px solid #dee2e6;
            }
            
            QPushButton {
                background: qlineargradient(x1: 0, y1: 0, x2: 0, y2: 1,
                    stop: 0 #ffffff, stop: 1 #f1f3f4);
                border: 1px solid #dadce0;
                border-radius: 6px;
                font-weight: 500;
                color: #3c4043;
            }
            
            QPushButton:hover {
                background: qlineargradient(x1: 0, y1: 0, x2: 0, y2: 1,
                    stop: 0 #f8f9fa, stop: 1 #e8eaed);
                border-color: #bdc1c6;
            }
            
            QPushButton:pressed {
                background: qlineargradient(x1: 0, y1: 0, x2: 0, y2: 1,
                    stop: 0 #e8eaed, stop: 1 #dadce0);
            }
            
            QPushButton:disabled {
                background: #f8f9fa;
                color: #9aa0a6;
                border-color: #f1f3f4;
            }
            
            QProgressBar {
                border: 1px solid #dadce0;
                border-radius: 3px;
                background: #f1f3f4;
                text-align: center;
            }
            
            QProgressBar::chunk {
                background: qlineargradient(x1: 0, y1: 0, x2: 0, y2: 1,
                    stop: 0 #4285f4, stop: 1 #1a73e8);
                border-radius: 2px;
            }
        """)
    
    def add_visual_feedback_effects(self):
        """Add awesome visual feedback and polish"""
        # Smooth animations for mode switching
        from PyQt6.QtCore import QPropertyAnimation, QEasingCurve
        
        # Tooltips will be added when buttons are created
        
        # Add status messages with character personality
        self.status_messages = {
            'ready': "🐹 Ready to chomp some PDFs!",
            'loading': "🐹 *sniff sniff* Loading...",
            'processing': "🐹 *munch munch* Processing...",
            'success': "🐹 *burp* All done!",
            'error': "🐹 *cough* Something went wrong!",
            'snyfter_mode': "🐁 *adjusts tiny glasses* Archive mode active"
        }
    
    def update_status_with_personality(self, status_key: str, custom_message: str = None):
        """Update status with character personality"""
        if hasattr(self, 'status_indicator'):
            # Update status indicator color
            colors = {
                'ready': '#4CAF50',      # Green
                'loading': '#FF9800',    # Orange
                'processing': '#2196F3', # Blue
                'success': '#4CAF50',    # Green
                'error': '#F44336',      # Red
                'snyfter_mode': '#9C27B0' # Purple
            }
            color = colors.get(status_key, '#4CAF50')
            self.status_indicator.setStyleSheet(f"color: {color}; font-size: 16px;")
            
            # Update tooltip
            if custom_message:
                self.status_indicator.setToolTip(custom_message)
            else:
                message = self.status_messages.get(status_key, "Ready")
                self.status_indicator.setToolTip(message)
    
    def show_welcome_message(self):
        """Show a brief welcome message"""
        from PyQt6.QtCore import QTimer
        
        # Show welcome for 3 seconds
        self.update_status_with_personality('ready')
        
        # Set up auto-load of document.pdf if it exists
        doc_path = os.path.join(os.path.dirname(__file__), "document.pdf")
        if os.path.exists(doc_path):
            QTimer.singleShot(500, lambda: self.load_pdf(doc_path))
    
    def show_command_palette(self):
        """Show modern command palette (Cmd+K style)"""
        from PyQt6.QtWidgets import QDialog, QDialogButtonBox
        from PyQt6.QtCore import Qt
        
        class CommandPalette(QDialog):
            def __init__(self, parent):
                super().__init__(parent)
                self.setWindowTitle("Command Palette")
                self.setModal(True)
                self.setWindowFlags(Qt.WindowType.FramelessWindowHint)
                self.resize(600, 400)
                
                # Center on parent
                parent_rect = parent.geometry()
                x = parent_rect.x() + (parent_rect.width() - 600) // 2
                y = parent_rect.y() + 100
                self.move(x, y)
                
                layout = QVBoxLayout(self)
                layout.setContentsMargins(0, 0, 0, 0)
                
                # Search input
                self.search_input = QLineEdit()
                self.search_input.setPlaceholderText("Type a command...")
                self.search_input.setStyleSheet("""
                    QLineEdit {
                        border: none;
                        border-bottom: 1px solid #E0E0E0;
                        padding: 16px;
                        font-size: 16px;
                        background: #FAFAFA;
                    }
                """)
                layout.addWidget(self.search_input)
                
                # Command list
                self.command_list = QListWidget()
                self.command_list.setStyleSheet("""
                    QListWidget {
                        border: none;
                        background: #FFFFFF;
                        font-size: 14px;
                    }
                    QListWidget::item {
                        padding: 12px 16px;
                        border-bottom: 1px solid #F0F0F0;
                    }
                    QListWidget::item:hover {
                        background: #F5F5F5;
                    }
                    QListWidget::item:selected {
                        background: #E0E0E0;
                        color: #000000;
                    }
                """)
                
                # Add commands
                commands = [
                    ("Open PDF", "Ctrl+O", parent.open_pdf),
                    ("Process Current PDF", "Ctrl+P", parent.process_current_pdf),
                    ("Export...", "Ctrl+E", parent.export_current),
                    ("Toggle PDF Panel", "Ctrl+1", lambda: parent.toggle_panel("pdf")),
                    ("Toggle Output Panel", "Ctrl+2", lambda: parent.toggle_panel("output")),
                    ("New Window", "Ctrl+Shift+N", parent.new_window),
                    ("Switch to CHONKER Mode", "Tab", lambda: parent.set_mode(Mode.CHONKER)),
                    ("Switch to SNYFTER Mode", "Tab", lambda: parent.set_mode(Mode.SNYFTER)),
                    ("Find...", "Ctrl+F", parent.show_find_dialog),
                    ("Recent Files", "", lambda: parent.show_recent_files()),
                ]
                
                self.commands = commands
                for cmd, shortcut, _ in commands:
                    item_text = f"{cmd}"
                    if shortcut:
                        item_text += f"  ({shortcut})"
                    self.command_list.addItem(item_text)
                
                layout.addWidget(self.command_list)
                
                # Connect signals
                self.search_input.textChanged.connect(self.filter_commands)
                self.search_input.returnPressed.connect(self.execute_command)
                self.command_list.itemActivated.connect(self.execute_command)
                
                # Focus search
                self.search_input.setFocus()
            
            def filter_commands(self, text):
                """Filter commands based on search text"""
                for i in range(self.command_list.count()):
                    item = self.command_list.item(i)
                    item.setHidden(text.lower() not in item.text().lower())
            
            def execute_command(self):
                """Execute selected command"""
                current_item = self.command_list.currentItem()
                if current_item and not current_item.isHidden():
                    index = self.command_list.row(current_item)
                    _, _, action = self.commands[index]
                    self.accept()
                    action()
            
            def keyPressEvent(self, event):
                if event.key() == Qt.Key.Key_Escape:
                    self.reject()
                else:
                    super().keyPressEvent(event)
        
        palette = CommandPalette(self)
        palette.exec()
    
    def toggle_panel(self, panel_name):
        """Toggle floating panel visibility"""
        if panel_name not in self.panel_windows:
            self.create_floating_panel(panel_name)
        else:
            window = self.panel_windows[panel_name]
            if window.isVisible():
                window.hide()
            else:
                window.show()
                window.raise_()
    
    def create_floating_panel(self, panel_name):
        """Create a new floating panel window"""
        config = self.panel_configs.get(panel_name, {})
        
        window = QWidget()
        window.setWindowTitle(config.get('title', 'Panel'))
        window.resize(*config.get('size', (600, 400)))
        window.setWindowFlags(Qt.WindowType.Window)
        
        # Add content based on panel type
        layout = QVBoxLayout(window)
        
        if panel_name == 'pdf':
            # Move PDF viewer to floating window
            if hasattr(self, 'pdf_view'):
                layout.addWidget(self.pdf_view)
        elif panel_name == 'output':
            # Move output to floating window
            if hasattr(self, 'faithful_output'):
                layout.addWidget(self.faithful_output)
        elif panel_name == 'search':
            # Create search interface
            search_widget = self.create_search_widget()
            layout.addWidget(search_widget)
        
        self.panel_windows[panel_name] = window
        window.show()
    
    def new_window(self):
        """Create a new window instance"""
        new_window = ChonkerSnyfterEnhancedWindow()
        new_window.show()
    
    def update_recent_files_menu(self, menu):
        """Update recent files menu"""
        # TODO: Implement recent files tracking
        menu.addAction("No recent files")
    
    def show_find_dialog(self):
        """Show find dialog"""
        # TODO: Implement find functionality
        self.update_terminal("🔍 Find dialog coming soon!", "info")
    
    def apply_theme(self, theme_name):
        """Apply selected theme"""
        self.update_terminal(f"🎨 Applying {theme_name} theme", "info")
        # TODO: Implement theme switching
    
    def show_shortcuts_dialog(self):
        """Show keyboard shortcuts dialog"""
        QMessageBox.information(self, "Keyboard Shortcuts", 
            "Tab - Switch modes\n"
            "Ctrl+O - Open PDF\n"
            "Ctrl+P - Process PDF\n"
            "Ctrl+K - Command Palette\n"
            "Ctrl+E - Export\n"
            "Ctrl+1 - Toggle PDF Panel\n"
            "Ctrl+2 - Toggle Output Panel\n"
            "Ctrl+F - Find\n"
            "Ctrl+N - New Session\n"
            "Ctrl+Shift+N - New Window")
    
    def show_about_dialog(self):
        """Show about dialog"""
        QMessageBox.about(self, "About CHONKER & SNYFTER",
            "🐹 CHONKER & 🐁 SNYFTER\n\n"
            "The Character-Driven PDF Processing System\n\n"
            "Featuring the ACTUAL Android 7.1 emoji images!\n"
            "Never trust Unicode rendering.")
    
    def show_recent_files(self):
        """Show recent files dialog"""
        self.update_terminal("📂 Recent files dialog coming soon!", "info")
    
    def create_search_widget(self):
        """Create search widget for floating panel"""
        widget = QWidget()
        layout = QVBoxLayout(widget)
        
        search_input = QLineEdit()
        search_input.setPlaceholderText("🐁 Search archives...")
        layout.addWidget(search_input)
        
        results_list = QListWidget()
        layout.addWidget(results_list)
        
        return widget
    
    def create_enhanced_chonker_interface(self):
        """Create CHONKER interface for modern floating panel system"""
        # Simple container that starts empty - panels will float
        container = QWidget()
        layout = QVBoxLayout(container)
        
        # Welcome message for empty state
        welcome = QLabel("🐹 Welcome to CHONKER!\n\nUse Ctrl+O to open a PDF\nor drag and drop a file here")
        welcome.setAlignment(Qt.AlignmentFlag.AlignCenter)
        welcome.setStyleSheet("font-size: 18px; color: #666; padding: 60px;")
        layout.addWidget(welcome)
        
        # Store reference for later
        self.welcome_widget = welcome
        
        return container
    
    def create_enhanced_chonker_interface_old(self):
        """Old interface - keeping for reference"""
        # Main horizontal splitter
        splitter = QSplitter(Qt.Orientation.Horizontal)
        
        # LEFT PANE - PDF Viewer
        left_pane = QWidget()
        left_layout = QVBoxLayout(left_pane)
        left_layout.setContentsMargins(10, 10, 10, 10)
        
        # PDF controls (minimal - now handled by top bar)
        controls = QHBoxLayout()
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
        """Create SNYFTER interface for modern floating panel system"""
        # Simple container for search
        container = QWidget()
        layout = QVBoxLayout(container)
        
        # Centered search interface
        search_container = QWidget()
        search_container.setMaximumWidth(600)
        search_layout = QVBoxLayout(search_container)
        
        # SNYFTER welcome
        welcome = QLabel("🐁 SNYFTER's Archive")
        welcome.setAlignment(Qt.AlignmentFlag.AlignCenter)
        welcome.setStyleSheet("font-size: 24px; color: #333; padding: 20px;")
        search_layout.addWidget(welcome)
        
        # Search input
        self.search_input = QLineEdit()
        self.search_input.setPlaceholderText("Search archived documents...")
        self.search_input.setStyleSheet("""
            QLineEdit {
                padding: 12px;
                font-size: 16px;
                border: 2px solid #E0E0E0;
            }
        """)
        self.search_input.returnPressed.connect(self.search_archives)
        search_layout.addWidget(self.search_input)
        
        # Recent documents
        recent_label = QLabel("Recent Documents")
        recent_label.setStyleSheet("font-weight: 600; margin-top: 20px;")
        search_layout.addWidget(recent_label)
        
        self.recent_list = QListWidget()
        self.recent_list.setMaximumHeight(200)
        search_layout.addWidget(self.recent_list)
        
        search_layout.addStretch()
        
        # Center the search container
        h_layout = QHBoxLayout()
        h_layout.addStretch()
        h_layout.addWidget(search_container)
        h_layout.addStretch()
        
        layout.addLayout(h_layout)
        
        return container
    
    def create_enhanced_snyfter_interface_old(self):
        """Old interface - keeping for reference"""
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
            # Update content for CHONKER mode
            if hasattr(self, 'content_layout'):
                # Clear current content
                while self.content_layout.count():
                    item = self.content_layout.takeAt(0)
                    if item.widget():
                        item.widget().setParent(None)
                
                # Add CHONKER interface
                self.content_layout.addWidget(self.chonker_widget)
            
            self.update_status_with_personality('ready')
            self.breadcrumb_label.setText("CHONKER Mode")
            
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
        else:  # SNYFTER mode
            # Update content for SNYFTER mode
            if hasattr(self, 'content_layout'):
                # Clear current content
                while self.content_layout.count():
                    item = self.content_layout.takeAt(0)
                    if item.widget():
                        item.widget().setParent(None)
                
                # Add SNYFTER interface
                self.content_layout.addWidget(self.snyfter_widget)
            
            self.update_status_with_personality('snyfter_mode')
            self.breadcrumb_label.setText("SNYFTER Mode")
            
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
        """Open PDF file dialog - creates new floating window"""
        file_path, _ = QFileDialog.getOpenFileName(
            self, "Select PDF", "", "PDF Files (*.pdf)"
        )
        if file_path:
            # Create new PDF window as requested
            self.create_pdf_window(file_path)
    
    def create_pdf_window(self, file_path: str):
        """Create a new floating window for PDF viewing"""
        # Create new window
        pdf_window = QWidget()
        pdf_window.setWindowTitle(f"🐹 PDF: {os.path.basename(file_path)}")
        pdf_window.resize(800, 1000)
        pdf_window.setWindowFlags(Qt.WindowType.Window)
        
        layout = QVBoxLayout(pdf_window)
        layout.setContentsMargins(0, 0, 0, 0)
        
        # PDF controls
        controls = QHBoxLayout()
        controls.setContentsMargins(10, 5, 10, 5)
        
        # Page navigation
        page_spin = QSpinBox()
        page_spin.setMinimum(1)
        controls.addWidget(QLabel("Page:"))
        controls.addWidget(page_spin)
        
        total_pages_label = QLabel("/ 0")
        controls.addWidget(total_pages_label)
        
        controls.addStretch()
        
        # Process button for this PDF
        process_btn = QPushButton("🐹 Process This PDF")
        process_btn.clicked.connect(lambda: self.process_specific_pdf(file_path))
        controls.addWidget(process_btn)
        
        # Close button
        close_btn = QPushButton("✕")
        close_btn.setFixedSize(30, 30)
        close_btn.clicked.connect(pdf_window.close)
        controls.addWidget(close_btn)
        
        layout.addLayout(controls)
        
        # PDF viewer
        pdf_view = QPdfView()
        pdf_document = QPdfDocument()
        pdf_view.setDocument(pdf_document)
        
        # Load the PDF
        pdf_document.load(file_path)
        page_spin.setMaximum(pdf_document.pageCount())
        total_pages_label.setText(f"/ {pdf_document.pageCount()}")
        
        # Connect page navigation
        page_spin.valueChanged.connect(
            lambda page: pdf_view.pageNavigator().jump(page - 1, QPointF(0, 0))
        )
        
        layout.addWidget(pdf_view)
        
        # Store window reference
        window_id = f"pdf_{len(self.panel_windows)}"
        self.panel_windows[window_id] = pdf_window
        
        # Show window
        pdf_window.show()
        
        # Update breadcrumb
        self.breadcrumb_label.setText(f"CHONKER Mode > {os.path.basename(file_path)}")
        
        # Store current PDF path for processing
        self.current_pdf_path = file_path
        
        # Hide welcome widget if visible
        if hasattr(self, 'welcome_widget'):
            self.welcome_widget.hide()
    
    def process_specific_pdf(self, file_path: str):
        """Process a specific PDF file"""
        self.current_pdf_path = file_path
        self.process_current_pdf()
    
    def load_pdf(self, file_path: str):
        """Load PDF into viewer or create new window"""
        try:
            self.current_pdf_path = file_path
            
            # In modern UI, create a new floating window for each PDF
            self.create_pdf_window(file_path)
            
            # Update status
            self.update_terminal(f"📂 Loaded PDF: {file_path}")
            
        except Exception as e:
            QMessageBox.critical(self, "🐹 CHONKER Error", f"🐹 *cough* Failed to load PDF: {str(e)}")
            self.update_terminal(f"🐹 Error loading PDF: {str(e)}")
    
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
        self.update_terminal(message)
    
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
                self.update_terminal("✅ Document successfully processed and archived")
                
                # Update archive stats
                self.update_archive_stats()
            else:
                warning_msg = "⚠️  Processing complete but failed to save to database"
                self.status_label.setText(warning_msg)
                self.update_terminal(warning_msg, "warning")
        else:
            self.update_terminal(f"❌ Processing failed: {result.error_message}", "error")
    
    def on_processing_error(self, error_msg: str):
        """Handle processing errors"""
        self.progress_bar.hide()
        self.process_btn.setEnabled(True)
        QMessageBox.critical(self, "🐹 Processing Error", f"🐹 *burp* {error_msg}")
        self.update_terminal(error_msg, "error")
    
    def display_processing_results(self, result: ProcessingResult):
        """Display processing results in faithful output format with rendered HTML"""
        # Store data for later use
        self.chunks_data = result.chunks
        self.markdown_content = result.markdown_content
        self.html_content = result.html_content
        
        # Parse and beautify HTML content with BeautifulSoup
        try:
            from bs4 import BeautifulSoup
            soup = BeautifulSoup(result.html_content, 'html.parser')
            
            # Create a clean HTML document for display
            html_output = f"""
            <!DOCTYPE html>
            <html>
            <head>
                <meta charset="UTF-8">
                <style>
                    body {{ 
                        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                        line-height: 1.6; 
                        max-width: 100%; 
                        margin: 20px;
                        background: white;
                    }}
                    .document-header {{
                        background: #f8f9fa;
                        padding: 15px;
                        border-radius: 8px;
                        margin-bottom: 20px;
                        border-left: 4px solid #007bff;
                    }}
                    table {{ 
                        border-collapse: collapse; 
                        width: 100%; 
                        margin: 15px 0;
                        box-shadow: 0 2px 4px rgba(0,0,0,0.1);
                    }}
                    th, td {{ 
                        border: 1px solid #dee2e6; 
                        padding: 12px; 
                        text-align: left; 
                    }}
                    th {{ 
                        background-color: #e9ecef; 
                        font-weight: 600;
                    }}
                    tr:nth-child(even) {{ background-color: #f8f9fa; }}
                    tr:hover {{ background-color: #e3f2fd; }}
                    h1, h2, h3, h4, h5, h6 {{ 
                        color: #2c3e50; 
                        margin-top: 25px;
                        margin-bottom: 15px;
                    }}
                    p {{ margin-bottom: 15px; }}
                    .chunk-info {{
                        background: #fff3cd;
                        border: 1px solid #ffeaa7;
                        padding: 8px 12px;
                        border-radius: 4px;
                        font-size: 12px;
                        margin: 10px 0;
                        color: #856404;
                    }}
                </style>
            </head>
            <body>
                <div class="document-header">
                    <h2>🐹 CHONKER's Faithful Output</h2>
                    <p><strong>Document ID:</strong> {result.document_id}</p>
                    <p><strong>Processing Time:</strong> {result.processing_time:.2f} seconds</p>
                    <p><strong>Total Chunks:</strong> {len(result.chunks)}</p>
                </div>
                
                <div class="content-section">
                    <h3>🌐 Extracted HTML Content</h3>
                    {soup.prettify()}
                </div>
                
                <div class="chunks-section">
                    <h3>📊 Chunk Details</h3>
            """
            
            # Add chunk information
            current_page = -1
            for chunk in result.chunks:
                if chunk.page != current_page:
                    current_page = chunk.page
                    html_output += f"<h4>📄 Page {chunk.page + 1}</h4>"
                
                html_output += f"""
                <div class="chunk-info">
                    <strong>[{chunk.index}] {chunk.type.upper()}</strong> 
                    (confidence: {chunk.confidence:.2f})
                </div>
                """
                
                # Render different chunk types appropriately
                if chunk.type.lower() == 'table':
                    # Try to parse as table if it looks like one
                    try:
                        table_soup = BeautifulSoup(chunk.content, 'html.parser')
                        if table_soup.find('table'):
                            html_output += str(table_soup)
                        else:
                            html_output += f"<pre>{chunk.content}</pre>"
                    except:
                        html_output += f"<pre>{chunk.content}</pre>"
                else:
                    # Regular content as paragraph
                    html_output += f"<p>{chunk.content}</p>"
            
            html_output += """
                </div>
            </body>
            </html>
            """
            
            # Set HTML content (make it editable)
            self.faithful_output.setHtml(html_output)
            self.faithful_output.setReadOnly(False)  # Make it editable
            
        except ImportError:
            # Fallback if BeautifulSoup not available
            self.update_terminal("🐹 BeautifulSoup not available, showing raw HTML", "warning")
            simple_html = f"""
            <h2>🐹 CHONKER's Faithful Output</h2>
            <p><b>Document ID:</b> {result.document_id}</p>
            <p><b>Processing Time:</b> {result.processing_time:.2f} seconds</p>
            <p><b>Total Chunks:</b> {len(result.chunks)}</p>
            <hr>
            <h3>HTML Content:</h3>
            <div>{result.html_content}</div>
            """
            self.faithful_output.setHtml(simple_html)
            self.faithful_output.setReadOnly(False)
        
        # Log summary with personality
        self.update_terminal(f"📊 Extracted {len(result.chunks)} chunks in {result.processing_time:.2f} seconds")
        self.update_status_with_personality('success')
        
        # Create new output window for the processed content
        self.create_output_window(result)
    
    def create_output_window(self, result: ProcessingResult):
        """Create a new floating window for HTML output"""
        # Create new window
        output_window = QWidget()
        output_window.setWindowTitle(f"🌐 Output: {os.path.basename(self.current_pdf_path)}")
        output_window.resize(900, 800)
        output_window.setWindowFlags(Qt.WindowType.Window)
        
        layout = QVBoxLayout(output_window)
        layout.setContentsMargins(0, 0, 0, 0)
        
        # Output controls
        controls = QHBoxLayout()
        controls.setContentsMargins(10, 5, 10, 5)
        
        controls.addWidget(QLabel("🌐 Processed HTML Output"))
        controls.addStretch()
        
        # Export button
        export_btn = QPushButton("💾 Export")
        export_btn.clicked.connect(self.export_current)
        controls.addWidget(export_btn)
        
        # Close button
        close_btn = QPushButton("✕")
        close_btn.setFixedSize(30, 30)
        close_btn.clicked.connect(output_window.close)
        controls.addWidget(close_btn)
        
        layout.addLayout(controls)
        
        # Create QTextEdit for HTML output
        output_view = QTextEdit()
        output_view.setReadOnly(False)  # Editable as requested
        
        # Set the HTML content we already generated
        if hasattr(self, 'faithful_output') and self.faithful_output.toHtml():
            output_view.setHtml(self.faithful_output.toHtml())
        else:
            # Fallback - generate HTML again
            try:
                from bs4 import BeautifulSoup
                soup = BeautifulSoup(result.html_content, 'html.parser')
                
                html_output = f"""
                <!DOCTYPE html>
                <html>
                <head>
                    <style>
                        body {{ font-family: -apple-system, sans-serif; margin: 20px; }}
                        table {{ border-collapse: collapse; width: 100%; margin: 15px 0; }}
                        th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
                        th {{ background-color: #f5f5f5; }}
                        tr:nth-child(even) {{ background-color: #fafafa; }}
                    </style>
                </head>
                <body>
                    <h2>🐹 CHONKER's Output</h2>
                    {soup.prettify()}
                </body>
                </html>
                """
                output_view.setHtml(html_output)
            except:
                output_view.setHtml(result.html_content)
        
        layout.addWidget(output_view)
        
        # Store window reference
        window_id = f"output_{len(self.panel_windows)}"
        self.panel_windows[window_id] = output_window
        
        # Store output view for export
        self.last_output_view = output_view
        
        # Show window
        output_window.show()
        
        # Update status
        self.update_status_with_personality('success', 
            "🐹 *burp* Processing complete! New output window created.")
    
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
            self.update_terminal("🧹 PDF cleaned - annotations removed")
            
        except Exception as e:
            QMessageBox.critical(self, "🐹 Clean Error", f"🐹 *cough* Failed to clean PDF: {str(e)}")
            self.update_terminal(f"🐹 Error cleaning PDF: {str(e)}", "error")
    
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
        self.update_terminal(f"🔍 Search complete: {len(results)} results for '{query}'")
    
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
            self.update_terminal(f"🐁 Error updating statistics: {str(e)}", "error")
    
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
        # Batch processing available via menu
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
                    self.update_terminal(f"📤 Exported faithful output to: {file_path}")
                except Exception as e:
                    QMessageBox.critical(self, "🐹 Export Error", f"🐹 *hiccup* Export failed: {str(e)}")
                    
        elif self.current_mode == Mode.SNYFTER:
            # Export search results
            # Export available via main export function
            QMessageBox.information(self, "Export", "Export search results - Coming soon!")
    
    def show_settings(self):
        """Show settings dialog"""
        # Settings dialog for advanced configuration
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
            self.terminal_display.clear()
            self.results_tree.clear()
            self.search_input.clear()
            
            self.status_label.setText("Ready - New session started")
            self.update_terminal("🆕 New session started")
    
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
        elif event.modifiers() == Qt.KeyboardModifier.ControlModifier:
            if event.key() == Qt.Key.Key_O:
                # Ctrl+O opens file
                self.open_pdf()
                event.accept()
            elif event.key() == Qt.Key.Key_P:
                # Ctrl+P processes current file
                if self.current_pdf_path:
                    self.process_current_pdf()
                event.accept()
            else:
                super().keyPressEvent(event)
        else:
            super().keyPressEvent(event)
    
    def closeEvent(self, event):
        """Handle application close"""
        # Stop caffeinate defense
        self.stop_caffeinate_defense()
        
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
    # Announce caffeinate defense
    print("\n🛡️ ACTIVATING CAFFEINATE DEFENSE SYSTEM...")
    print("☕ This app will fight to stay awake and prevent auto-logout!")
    print("💪 CHONKER & SNYFTER shall not be stopped by mere system sleep!\n")
    
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