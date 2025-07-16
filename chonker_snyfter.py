#!/usr/bin/env python3
"""
🐹 CHONKER & 🐁 SNYFTER - The Document Processing Duo

CHONKER: The chubby hamster who gobbles up PDFs and makes them digestible
SNYFTER: The skinny librarian mouse who meticulously catalogs everything
"""

import sys
import os
import sqlite3
import json
import hashlib
import tempfile
from pathlib import Path
from typing import Dict, List, Optional, Tuple
from datetime import datetime
from enum import Enum

from PyQt6.QtWidgets import (
    QApplication, QMainWindow, QStackedWidget, QVBoxLayout, QHBoxLayout,
    QWidget, QPushButton, QFileDialog, QMessageBox, QTextEdit,
    QLabel, QComboBox, QScrollArea, QFrame, QProgressBar,
    QTableWidget, QTableWidgetItem, QTabWidget, QStatusBar,
    QLineEdit, QListWidget, QSplitter, QPlainTextEdit, QMenu,
    QSpinBox, QInputDialog, QListWidgetItem, QDialog, QDialogButtonBox,
    QCheckBox, QFormLayout
)
from PyQt6.QtCore import Qt, QThread, pyqtSignal, QTimer, QPointF, QObject, QEvent, QSize, QRectF
from PyQt6.QtGui import QFont, QPalette, QColor, QTextCharFormat, QTextCursor, QKeyEvent, QPixmap, QIcon, QAction, QPainter, QPen, QBrush
from PyQt6.QtPdf import QPdfDocument, QPdfSelection
from PyQt6.QtPdfWidgets import QPdfView

try:
    from docling.document_converter import DocumentConverter
    import fitz  # PyMuPDF
    DEPENDENCIES_AVAILABLE = True
except ImportError:
    DEPENDENCIES_AVAILABLE = False


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


class ChonkerWorker(QThread):
    """CHONKER's PDF processing worker thread"""
    
    finished = pyqtSignal(dict)
    progress = pyqtSignal(str)
    error = pyqtSignal(str)
    
    def __init__(self, pdf_path: str):
        super().__init__()
        self.pdf_path = pdf_path
    
    def run(self):
        try:
            import random
            self.progress.emit(random.choice(ChonkerPersonality.PROCESSING))
            
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
            result = converter.convert(self.pdf_path)
            
            # Package for Snyfter
            content = {
                'html': result.document.export_to_html(),
                'markdown': result.document.export_to_markdown(),
                'chunks': [],
                'tables': [],
                'chonker_notes': "🐹 Processed with love and hamster cheeks!"
            }
            
            # Extract chunks for Snyfter's cataloging
            chunk_index = 0
            for item, level in result.document.iterate_items():
                item_type = type(item).__name__
                
                # Safely extract page number
                page_no = 0
                if hasattr(item, 'prov') and item.prov:
                    prov_list = item.prov if isinstance(item.prov, list) else [item.prov]
                    if prov_list and hasattr(prov_list[0], 'page_no'):
                        page_no = prov_list[0].page_no
                
                chunk = {
                    'index': chunk_index,
                    'type': item_type.lower().replace('item', ''),
                    'content': getattr(item, 'text', str(item)),
                    'level': level,
                    'page': page_no,
                }
                content['chunks'].append(chunk)
                chunk_index += 1
            
            # Cleanup
            if temp_pdf:
                os.unlink(temp_pdf.name)
            
            self.finished.emit(content)
            
        except Exception as e:
            if temp_pdf:
                os.unlink(temp_pdf.name)
            self.error.emit(f"🐹 Error: {str(e)}")
    
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
            print(f"🐹 De-chonkification failed: {e}")
            return False


class BatchProcessor(QThread):
    """CHONKER's batch processing worker"""
    
    progress = pyqtSignal(int, int, str)  # current, total, message
    file_completed = pyqtSignal(str, bool, str)  # filepath, success, message
    all_completed = pyqtSignal()
    
    def __init__(self, files: List[str], operations: Dict[str, bool]):
        super().__init__()
        self.files = files
        self.operations = operations
        self.should_stop = False
    
    def run(self):
        total_files = len(self.files)
        
        for i, file_path in enumerate(self.files):
            if self.should_stop:
                break
                
            self.progress.emit(i + 1, total_files, f"🐹 Processing {os.path.basename(file_path)}...")
            
            try:
                # Process each file
                success = self.process_file(file_path)
                if success:
                    self.file_completed.emit(file_path, True, "🐹 *burp* Success!")
                else:
                    self.file_completed.emit(file_path, False, "🐹 *cough* Failed!")
            except Exception as e:
                self.file_completed.emit(file_path, False, f"🐹 Error: {str(e)}")
        
        self.all_completed.emit()
    
    def process_file(self, file_path: str) -> bool:
        """Process a single file with selected operations"""
        try:
            doc = fitz.open(file_path)
            modified = False
            
            # Apply operations
            if self.operations.get('clean', False):
                for page in doc:
                    annot = page.first_annot
                    while annot:
                        next_annot = annot.next
                        page.delete_annot(annot)
                        annot = next_annot
                modified = True
            
            if self.operations.get('compress', False):
                # Save with compression
                temp_path = file_path + '.tmp'
                doc.save(temp_path, garbage=4, clean=True, deflate=True)
                doc.close()
                doc = fitz.open(temp_path)
                os.remove(temp_path)
                modified = True
            
            if self.operations.get('rotate', False):
                for page in doc:
                    page.set_rotation(page.rotation + 90)
                modified = True
            
            # Save if modified
            if modified:
                output_dir = os.path.dirname(file_path)
                basename = os.path.splitext(os.path.basename(file_path))[0]
                output_path = os.path.join(output_dir, f"{basename}_processed.pdf")
                doc.save(output_path)
            
            doc.close()
            return True
            
        except Exception as e:
            print(f"🐹 Batch processing error: {e}")
            return False
    
    def stop(self):
        self.should_stop = True


class SnyfterDatabase:
    """SNYFTER's meticulous filing system"""
    
    def __init__(self, db_path: str = "snyfter_archives.db"):
        self.db_path = db_path
        self.init_archives()
    
    def init_archives(self):
        """Set up Snyfter's card catalog system"""
        with sqlite3.connect(self.db_path) as conn:
            # Snyfter's main catalog
            conn.execute("""
                CREATE TABLE IF NOT EXISTS documents (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    filename TEXT NOT NULL,
                    file_hash TEXT UNIQUE NOT NULL,
                    processed_date TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    chonker_notes TEXT,
                    snyfter_classification TEXT,
                    review_status TEXT DEFAULT 'pending'
                )
            """)
            
            # Snyfter's content cards
            conn.execute("""
                CREATE TABLE IF NOT EXISTS content_cards (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    document_id INTEGER NOT NULL,
                    content_type TEXT NOT NULL,
                    content TEXT NOT NULL,
                    snyfter_notes TEXT,
                    FOREIGN KEY (document_id) REFERENCES documents(id)
                )
            """)
            
            # Snyfter's research notes
            conn.execute("""
                CREATE TABLE IF NOT EXISTS research_notes (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    document_id INTEGER NOT NULL,
                    note TEXT NOT NULL,
                    created_date TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    FOREIGN KEY (document_id) REFERENCES documents(id)
                )
            """)
    
    def file_document(self, filename: str, file_hash: str, chonker_notes: str) -> int:
        """File a new document in Snyfter's archives"""
        with sqlite3.connect(self.db_path) as conn:
            cursor = conn.execute(
                """INSERT INTO documents 
                   (filename, file_hash, chonker_notes, snyfter_classification) 
                   VALUES (?, ?, ?, ?)""",
                (filename, file_hash, chonker_notes, "🐁 Awaiting classification")
            )
            return cursor.lastrowid
    
    def search_archives(self, query: str) -> List[Dict]:
        """Snyfter searches through the card catalog"""
        with sqlite3.connect(self.db_path) as conn:
            conn.row_factory = sqlite3.Row
            
            # Snyfter's thorough search
            cursor = conn.execute("""
                SELECT d.*, COUNT(c.id) as content_count
                FROM documents d
                LEFT JOIN content_cards c ON d.id = c.document_id
                WHERE d.filename LIKE ? OR d.chonker_notes LIKE ?
                   OR d.snyfter_classification LIKE ?
                GROUP BY d.id
                ORDER BY d.processed_date DESC
                LIMIT 20
            """, (f"%{query}%", f"%{query}%", f"%{query}%"))
            
            return [dict(row) for row in cursor.fetchall()]


class BidirectionalSelector(QObject):
    """Handles bidirectional selection between PDF and extracted content"""
    
    def __init__(self, pdf_view: QPdfView, content_tabs: QTabWidget, chunks_table: QTableWidget):
        super().__init__()
        self.pdf_view = pdf_view
        self.content_tabs = content_tabs
        self.chunks_table = chunks_table
        self.chunk_map = {}  # chunk_id -> {page, bbox, content}
        self.current_highlight = None
        
        # Install event filter for PDF clicks
        self.pdf_view.viewport().installEventFilter(self)
        
        # Connect table selection
        self.chunks_table.itemSelectionChanged.connect(self.on_chunk_selected)
    
    def set_chunks(self, chunks: List[Dict]):
        """Update chunk mapping when new content is extracted"""
        self.chunk_map.clear()
        for chunk in chunks:
            chunk_id = chunk['index']
            self.chunk_map[chunk_id] = {
                'page': chunk.get('page', 0),
                'bbox': chunk.get('bbox', {}),
                'content': chunk.get('content', ''),
                'type': chunk.get('type', '')
            }
    
    def eventFilter(self, obj, event):
        """Handle mouse clicks on PDF view"""
        try:
            if obj == self.pdf_view.viewport() and event.type() == QEvent.Type.MouseButtonPress:
                if event.button() == Qt.MouseButton.LeftButton:
                    self.on_pdf_click(event.pos())
        except RuntimeError:
            # PDF view was deleted
            pass
        return False
    
    def on_pdf_click(self, pos):
        """Handle click on PDF - find and highlight corresponding chunk"""
        # Get current page
        nav = self.pdf_view.pageNavigator()
        if not nav:
            return
            
        current_page = nav.currentPage()
        
        # For now, just cycle through chunks on the current page when clicked
        page_chunks = []
        for chunk_id, chunk_data in self.chunk_map.items():
            if chunk_data['page'] == current_page:
                page_chunks.append(chunk_id)
        
        if page_chunks:
            # Find next chunk to highlight
            if self.current_highlight in page_chunks:
                current_idx = page_chunks.index(self.current_highlight)
                next_idx = (current_idx + 1) % len(page_chunks)
            else:
                next_idx = 0
            
            self.highlight_chunk(page_chunks[next_idx])
    
    def on_chunk_selected(self):
        """Handle selection in chunks table"""
        selected_items = self.chunks_table.selectedItems()
        if selected_items:
            # Get row of first selected item
            row = selected_items[0].row()
            self.highlight_chunk(row)
            
            # Scroll PDF to show the chunk
            chunk_data = self.chunk_map.get(row, {})
            if chunk_data and 'page' in chunk_data:
                nav = self.pdf_view.pageNavigator()
                if nav and chunk_data['page'] != nav.currentPage():
                    nav.jump(chunk_data['page'], QPointF())
    
    def highlight_chunk(self, chunk_id: int):
        """Highlight chunk in both views"""
        chunk_data = self.chunk_map.get(chunk_id)
        if not chunk_data:
            return
        
        # Highlight in chunks table
        self.chunks_table.selectRow(chunk_id)
        
        # Highlight in content views
        content = chunk_data['content']
        
        # HTML viewer
        if self.content_tabs.currentIndex() == 0:  # HTML tab
            html_viewer = self.content_tabs.widget(0)
            if isinstance(html_viewer, QTextEdit):
                self.highlight_text_in_viewer(html_viewer, content)
        
        # Store current highlight
        self.current_highlight = chunk_id
    
    def highlight_text_in_viewer(self, viewer: QTextEdit, text: str):
        """Highlight text in a QTextEdit viewer"""
        if not text:
            return
            
        # Clear previous highlights
        cursor = viewer.textCursor()
        cursor.select(QTextCursor.SelectionType.Document)
        cursor.setCharFormat(QTextCharFormat())
        
        # Find and highlight text
        cursor = viewer.document().find(text[:50])  # Use first 50 chars to find
        if not cursor.isNull():
            # Highlight format
            fmt = QTextCharFormat()
            fmt.setBackground(QColor(255, 255, 0, 100))  # Light yellow
            cursor.mergeCharFormat(fmt)
            
            # Scroll to position
            viewer.setTextCursor(cursor)
            viewer.ensureCursorVisible()


class BatchProcessDialog(QDialog):
    """CHONKER's batch processing configuration dialog"""
    
    def __init__(self, parent=None):
        super().__init__(parent)
        self.setWindowTitle("🐹 CHONKER Batch Processing")
        self.setModal(True)
        self.resize(600, 500)
        
        layout = QVBoxLayout(self)
        
        # File selection
        layout.addWidget(QLabel("🐹 Select PDFs to process:"))
        
        self.file_list = QListWidget()
        layout.addWidget(self.file_list)
        
        file_buttons = QHBoxLayout()
        add_files_btn = QPushButton("➕ Add Files")
        add_files_btn.clicked.connect(self.add_files)
        file_buttons.addWidget(add_files_btn)
        
        add_folder_btn = QPushButton("📁 Add Folder")
        add_folder_btn.clicked.connect(self.add_folder)
        file_buttons.addWidget(add_folder_btn)
        
        remove_btn = QPushButton("➖ Remove Selected")
        remove_btn.clicked.connect(self.remove_selected)
        file_buttons.addWidget(remove_btn)
        
        file_buttons.addStretch()
        layout.addLayout(file_buttons)
        
        # Operations selection
        layout.addWidget(QLabel("\n🐹 Select operations to perform:"))
        
        self.operations = {}
        
        self.clean_check = QCheckBox("🧹 Clean PDFs (Remove annotations)")
        self.operations['clean'] = self.clean_check
        layout.addWidget(self.clean_check)
        
        self.compress_check = QCheckBox("📦 Compress PDFs")
        self.operations['compress'] = self.compress_check
        layout.addWidget(self.compress_check)
        
        self.rotate_check = QCheckBox("🔄 Rotate 90° Right")
        self.operations['rotate'] = self.rotate_check
        layout.addWidget(self.rotate_check)
        
        self.extract_check = QCheckBox("🔍 Extract & Catalog (Process with Docling)")
        self.operations['extract'] = self.extract_check
        layout.addWidget(self.extract_check)
        
        layout.addStretch()
        
        # Buttons
        buttons = QDialogButtonBox(
            QDialogButtonBox.StandardButton.Ok | QDialogButtonBox.StandardButton.Cancel
        )
        buttons.accepted.connect(self.accept)
        buttons.rejected.connect(self.reject)
        layout.addWidget(buttons)
    
    def add_files(self):
        files, _ = QFileDialog.getOpenFileNames(
            self, "🐹 Select PDFs", "", "PDF Files (*.pdf)"
        )
        for file in files:
            if file not in self.get_selected_files():
                self.file_list.addItem(file)
    
    def add_folder(self):
        folder = QFileDialog.getExistingDirectory(self, "🐹 Select Folder")
        if folder:
            import glob
            pdf_files = glob.glob(os.path.join(folder, "*.pdf"))
            for file in pdf_files:
                if file not in self.get_selected_files():
                    self.file_list.addItem(file)
    
    def remove_selected(self):
        for item in self.file_list.selectedItems():
            self.file_list.takeItem(self.file_list.row(item))
    
    def get_selected_files(self) -> List[str]:
        files = []
        for i in range(self.file_list.count()):
            files.append(self.file_list.item(i).text())
        return files
    
    def get_selected_operations(self) -> Dict[str, bool]:
        return {
            name: checkbox.isChecked()
            for name, checkbox in self.operations.items()
        }


class ChonkerSnyfterMainWindow(QMainWindow):
    """The main window that houses both CHONKER and SNYFTER"""
    
    def __init__(self):
        super().__init__()
        self.current_mode = Mode.CHONKER
        self.snyfter_db = SnyfterDatabase()
        self.current_document = None
        self.bidirectional_selector = None
        self.current_pdf_path = None
        self.original_pdf_path = None
        self.temp_pdf_path = None
        self.pdf_modified = False
        
        # Load the ACTUAL Android 7.1 emoji images with error handling
        chonker_path = os.path.join(os.path.dirname(__file__), "assets/emojis/chonker.png")
        snyfter_path = os.path.join(os.path.dirname(__file__), "assets/emojis/snyfter.png")
        
        self.chonker_pixmap = QPixmap(chonker_path) if os.path.exists(chonker_path) else QPixmap()
        self.snyfter_pixmap = QPixmap(snyfter_path) if os.path.exists(snyfter_path) else QPixmap()
        
        # Create fallback pixmaps if files don't exist
        if self.chonker_pixmap.isNull():
            self.chonker_pixmap = self.create_fallback_pixmap("🐹", QColor("#FFE4B5"))
        if self.snyfter_pixmap.isNull():
            self.snyfter_pixmap = self.create_fallback_pixmap("🐁", QColor("#D3D3D3"))
        
        self.init_ui()
    
    def create_fallback_pixmap(self, emoji: str, bg_color: QColor) -> QPixmap:
        """Create fallback pixmap when emoji images are not available"""
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
        font.setPointSize(28)
        painter.setFont(font)
        painter.drawText(pixmap.rect(), Qt.AlignmentFlag.AlignCenter, emoji)
        
        painter.end()
        return pixmap
    
    def init_ui(self):
        self.setWindowTitle("CHONKER & SNYFTER - Document Processing Duo")
        self.setGeometry(100, 100, 1400, 900)
        
        # Central widget with stacked layout
        central_widget = QWidget()
        self.setCentralWidget(central_widget)
        main_layout = QVBoxLayout(central_widget)
        
        # Top bar with both emojis and terminal feedback
        top_bar = QWidget()
        top_bar_layout = QHBoxLayout(top_bar)
        top_bar_layout.setContentsMargins(10, 5, 10, 5)
        
        # CHONKER emoji (always visible)
        self.chonker_label = QLabel()
        self.chonker_label.setPixmap(self.chonker_pixmap.scaled(32, 32, Qt.AspectRatioMode.KeepAspectRatio, Qt.TransformationMode.SmoothTransformation))
        self.chonker_label.setStyleSheet("""
            QLabel {
                padding: 5px;
                background: #FFE4B5;
                border: 2px solid #FFE4B5;
                border-radius: 5px;
            }
        """)
        top_bar_layout.addWidget(self.chonker_label)
        
        # SNYFTER emoji (always visible)
        self.snyfter_label = QLabel()
        self.snyfter_label.setPixmap(self.snyfter_pixmap.scaled(32, 32, Qt.AspectRatioMode.KeepAspectRatio, Qt.TransformationMode.SmoothTransformation))
        self.snyfter_label.setStyleSheet("""
            QLabel {
                padding: 5px;
                background: transparent;
                border: 2px solid transparent;
                border-radius: 5px;
            }
        """)
        top_bar_layout.addWidget(self.snyfter_label)
        
        # Terminal-style feedback area
        self.terminal_feedback = QLabel("CHONKER MODE ACTIVE")
        self.terminal_feedback.setStyleSheet("""
            QLabel {
                font-family: 'Courier New', monospace;
                font-size: 14px;
                color: #00FF00;
                background: #000000;
                padding: 5px 10px;
                border-radius: 3px;
            }
        """)
        top_bar_layout.addWidget(self.terminal_feedback, 1)  # Stretch to fill
        
        # Style the top bar
        top_bar.setStyleSheet("""
            QWidget {
                background: #2C3E50;
                border-bottom: 2px solid #34495E;
            }
        """)
        main_layout.addWidget(top_bar)
        
        # Stacked widget for modes
        self.stacked_widget = QStackedWidget()
        main_layout.addWidget(self.stacked_widget)
        
        # Create both interfaces
        self.chonker_widget = self.create_chonker_interface()
        self.snyfter_widget = self.create_snyfter_interface()
        
        self.stacked_widget.addWidget(self.chonker_widget)
        self.stacked_widget.addWidget(self.snyfter_widget)
        
        # Status bar
        self.status_bar = QStatusBar()
        self.setStatusBar(self.status_bar)
        self.status_bar.showMessage(ChonkerPersonality.GREETINGS[0])
        
        # Install event filter for Tab key on the app to catch it globally
        QApplication.instance().installEventFilter(self)
    
    def create_chonker_interface(self) -> QWidget:
        """Create CHONKER's PDF munching interface"""
        widget = QWidget()
        layout = QVBoxLayout(widget)
        
        # Chonker's tools
        toolbar = QHBoxLayout()
        
        self.feed_btn = QPushButton("Feed PDF to CHONKER")
        self.feed_btn.setIcon(QIcon(self.chonker_pixmap))
        self.feed_btn.setIconSize(QSize(24, 24))
        self.feed_btn.clicked.connect(self.feed_chonker)
        toolbar.addWidget(self.feed_btn)
        
        # PDF Tools menu (CHONKER's utensils)
        self.pdf_tools_btn = QPushButton("PDF Tools")
        self.pdf_tools_btn.setIcon(QIcon(self.chonker_pixmap))
        self.pdf_tools_btn.setIconSize(QSize(24, 24))
        self.pdf_tools_menu = QMenu()
        
        # Organize tools by category
        rotate_menu = self.pdf_tools_menu.addMenu("Rotate Pages")
        rotate_menu.addAction("Rotate 90° Right", lambda: self.rotate_pdf(90))
        rotate_menu.addAction("Rotate 90° Left", lambda: self.rotate_pdf(-90))
        rotate_menu.addAction("Rotate 180°", lambda: self.rotate_pdf(180))
        rotate_menu.addAction("Set to 0° (Original)", lambda: self.rotate_pdf(0))
        
        self.pdf_tools_menu.addSeparator()
        
        page_menu = self.pdf_tools_menu.addMenu("Page Operations")
        page_menu.addAction("Split PDF", self.split_pdf)
        page_menu.addAction("Merge PDFs", self.merge_pdfs)
        page_menu.addAction("Extract Pages", self.extract_pages)
        page_menu.addAction("Insert Pages", self.insert_pages)
        page_menu.addAction("Delete Pages", self.delete_pages)
        
        self.pdf_tools_menu.addSeparator()
        
        clean_menu = self.pdf_tools_menu.addMenu("Clean & Optimize")
        clean_menu.addAction("Clean PDF (Remove Annotations)", self.clean_pdf)
        clean_menu.addAction("Compress PDF", self.compress_pdf)
        clean_menu.addAction("Optimize for Extraction", self.optimize_for_extraction)
        
        self.pdf_tools_menu.addSeparator()
        self.pdf_tools_menu.addAction("Save Modified PDF", self.save_modified_pdf)
        self.pdf_tools_menu.addAction("Revert Changes", self.revert_changes)
        
        self.pdf_tools_btn.setMenu(self.pdf_tools_menu)
        self.pdf_tools_btn.setEnabled(False)
        toolbar.addWidget(self.pdf_tools_btn)
        
        # Batch processing button
        self.batch_btn = QPushButton("Batch Process")
        self.batch_btn.setIcon(QIcon(self.chonker_pixmap))
        self.batch_btn.setIconSize(QSize(24, 24))
        self.batch_btn.clicked.connect(self.show_batch_dialog)
        toolbar.addWidget(self.batch_btn)
        
        self.digest_btn = QPushButton("Digest & Extract")
        self.digest_btn.setIcon(QIcon(self.chonker_pixmap))
        self.digest_btn.setIconSize(QSize(24, 24))
        self.digest_btn.clicked.connect(self.digest_pdf)
        self.digest_btn.setEnabled(False)
        toolbar.addWidget(self.digest_btn)
        
        self.pass_to_snyfter_btn = QPushButton("Pass to SNYFTER")
        self.pass_to_snyfter_btn.setIcon(QIcon(self.snyfter_pixmap))
        self.pass_to_snyfter_btn.setIconSize(QSize(24, 24))
        self.pass_to_snyfter_btn.clicked.connect(self.pass_to_snyfter)
        self.pass_to_snyfter_btn.setEnabled(False)
        toolbar.addWidget(self.pass_to_snyfter_btn)
        
        toolbar.addStretch()
        layout.addLayout(toolbar)
        
        # Progress bar (CHONKER's digestion indicator)
        self.chonker_progress = QProgressBar()
        self.chonker_progress.setVisible(False)
        layout.addWidget(self.chonker_progress)
        
        # Splitter for PDF and content
        splitter = QSplitter(Qt.Orientation.Horizontal)
        
        # PDF viewer (CHONKER's meal viewer)
        pdf_widget = QWidget()
        pdf_layout = QVBoxLayout(pdf_widget)
        
        self.pdf_view = QPdfView(pdf_widget)
        self.pdf_document = QPdfDocument(self)
        self.pdf_view.setDocument(self.pdf_document)
        pdf_layout.addWidget(self.pdf_view)
        
        # Content viewer (CHONKER's digestion results)
        self.content_tabs = QTabWidget()
        
        # Editable extraction view
        self.extraction_widget = QWidget()
        extraction_layout = QVBoxLayout(self.extraction_widget)
        
        # Toolbar for editing operations
        edit_toolbar = QHBoxLayout()
        self.add_table_btn = QPushButton("Add Table")
        self.add_table_btn.clicked.connect(self.add_table_chunk)
        edit_toolbar.addWidget(self.add_table_btn)
        
        self.convert_table_btn = QPushButton("Convert to Table")
        self.convert_table_btn.clicked.connect(self.convert_to_table)
        edit_toolbar.addWidget(self.convert_table_btn)
        
        self.merge_chunks_btn = QPushButton("Merge Selected")
        self.merge_chunks_btn.clicked.connect(self.merge_chunks)
        edit_toolbar.addWidget(self.merge_chunks_btn)
        
        self.split_chunk_btn = QPushButton("Split Chunk")
        self.split_chunk_btn.clicked.connect(self.split_chunk)
        edit_toolbar.addWidget(self.split_chunk_btn)
        
        edit_toolbar.addStretch()
        extraction_layout.addLayout(edit_toolbar)
        
        # Editable chunks list
        self.chunks_widget = QListWidget()
        self.chunks_widget.setDragDropMode(QListWidget.DragDropMode.InternalMove)
        extraction_layout.addWidget(self.chunks_widget)
        
        self.content_tabs.addTab(self.extraction_widget, "🐹 Fix Extraction")
        
        # Original HTML view (read-only)
        self.html_viewer = QTextEdit()
        self.html_viewer.setReadOnly(True)
        self.content_tabs.addTab(self.html_viewer, "🐹 Original HTML")
        
        # Chunks table for reference
        self.chunks_table = QTableWidget()
        self.chunks_table.setColumnCount(4)
        self.chunks_table.setHorizontalHeaderLabels(["Type", "Content", "Page", "Status"])
        self.content_tabs.addTab(self.chunks_table, "🐹 Chunks Table")
        
        splitter.addWidget(pdf_widget)
        splitter.addWidget(self.content_tabs)
        splitter.setSizes([700, 700])
        
        layout.addWidget(splitter)
        
        # Initialize bidirectional selector after UI is created
        QTimer.singleShot(100, self.setup_bidirectional_selector)
        
        return widget
    
    def setup_bidirectional_selector(self):
        """Set up bidirectional selection after UI is fully created"""
        if hasattr(self, 'pdf_view') and hasattr(self, 'content_tabs') and hasattr(self, 'chunks_table'):
            self.bidirectional_selector = BidirectionalSelector(
                self.pdf_view, self.content_tabs, self.chunks_table
            )
    
    def add_table_chunk(self):
        """Add a new table chunk at current position"""
        from PyQt6.QtWidgets import QDialog, QSpinBox, QFormLayout
        
        dialog = QDialog(self)
        dialog.setWindowTitle("🐹 Add Table")
        layout = QFormLayout(dialog)
        
        rows_spin = QSpinBox()
        rows_spin.setRange(1, 50)
        rows_spin.setValue(3)
        layout.addRow("Rows:", rows_spin)
        
        cols_spin = QSpinBox()
        cols_spin.setRange(1, 20)
        cols_spin.setValue(3)
        layout.addRow("Columns:", cols_spin)
        
        page_spin = QSpinBox()
        page_spin.setRange(1, 9999)
        nav = self.pdf_view.pageNavigator()
        if nav:
            page_spin.setValue(nav.currentPage() + 1)
        layout.addRow("From Page:", page_spin)
        
        buttons = QDialogButtonBox(
            QDialogButtonBox.StandardButton.Ok | QDialogButtonBox.StandardButton.Cancel
        )
        buttons.accepted.connect(dialog.accept)
        buttons.rejected.connect(dialog.reject)
        layout.addRow(buttons)
        
        if dialog.exec() == QDialog.DialogCode.Accepted:
            # Create table HTML
            table_html = "<table border='1' data-page='" + str(page_spin.value()) + "'>"
            for r in range(rows_spin.value()):
                table_html += "<tr>"
                for c in range(cols_spin.value()):
                    table_html += f"<td contenteditable='true'>Cell {r+1},{c+1}</td>"
                table_html += "</tr>"
            table_html += "</table>"
            
            # Add to chunks
            item_text = f"[TABLE - Page {page_spin.value()}] {rows_spin.value()}x{cols_spin.value()} table"
            self.chunks_widget.addItem(item_text)
            
            self.status_bar.showMessage(f"🐹 Added {rows_spin.value()}x{cols_spin.value()} table!")
    
    def convert_to_table(self):
        """Convert selected text chunk to a table"""
        current_item = self.chunks_widget.currentItem()
        if not current_item:
            QMessageBox.information(self, "🐹 CHONKER", "Select a chunk to convert!")
            return
        
        # Simple conversion - split by whitespace and newlines
        text = current_item.text()
        lines = text.split('\\n')
        
        if len(lines) < 2:
            QMessageBox.warning(self, "🐹 CHONKER", "Not enough data for a table!")
            return
        
        # Estimate columns by first line
        cols = len(lines[0].split())
        
        # Build table representation
        table_text = f"[TABLE - Converted] {len(lines)}x{cols} table"
        current_row = self.chunks_widget.row(current_item)
        self.chunks_widget.takeItem(current_row)
        self.chunks_widget.insertItem(current_row, table_text)
        
        self.status_bar.showMessage("🐹 *munch munch* Converted to table!")
    
    def merge_chunks(self):
        """Merge selected chunks into one"""
        selected_items = self.chunks_widget.selectedItems()
        if len(selected_items) < 2:
            QMessageBox.information(self, "🐹 CHONKER", "Select multiple chunks to merge!")
            return
        
        # Get all selected text and pages
        merged_text = []
        pages = set()
        
        for item in selected_items:
            merged_text.append(item.text())
            # Extract page number if present
            if "Page " in item.text():
                try:
                    page = int(item.text().split("Page ")[1].split("]")[0])
                    pages.add(page)
                except:
                    pass
        
        # Remove old items
        for item in selected_items:
            self.chunks_widget.takeItem(self.chunks_widget.row(item))
        
        # Add merged item
        page_str = f"Pages {min(pages)}-{max(pages)}" if len(pages) > 1 else f"Page {min(pages)}"
        merged_item = f"[MERGED - {page_str}] " + " | ".join(merged_text)
        self.chunks_widget.addItem(merged_item)
        
        self.status_bar.showMessage(f"🐹 *gulp* Merged {len(selected_items)} chunks!")
    
    def split_chunk(self):
        """Split a chunk at cursor position"""
        current_item = self.chunks_widget.currentItem()
        if not current_item:
            QMessageBox.information(self, "🐹 CHONKER", "Select a chunk to split!")
            return
        
        text = current_item.text()
        
        # Simple split dialog
        split_pos, ok = QInputDialog.getInt(
            self, "🐹 Split Position",
            f"Split at character (1-{len(text)}):",
            value=len(text)//2, min=1, max=len(text)
        )
        
        if ok:
            part1 = text[:split_pos]
            part2 = text[split_pos:]
            
            current_row = self.chunks_widget.row(current_item)
            self.chunks_widget.takeItem(current_row)
            
            self.chunks_widget.insertItem(current_row, part1)
            self.chunks_widget.insertItem(current_row + 1, part2)
            
            self.status_bar.showMessage("🐹 *chomp* Split chunk!")
    
    def rotate_pdf(self, angle: int):
        """CHONKER rotates the PDF pages"""
        if not self.current_pdf_path:
            return
        
        try:
            # Create temp file if first modification
            if not self.pdf_modified:
                self.create_temp_pdf()
            
            doc = fitz.open(self.temp_pdf_path or self.current_pdf_path)
            
            # Get selected pages or all pages
            nav = self.pdf_view.pageNavigator()
            current_page = nav.currentPage() if nav else 0
            
            # Ask if rotate all or just current page
            reply = QMessageBox.question(
                self, "🐹 CHONKER Asks",
                f"Rotate all pages or just page {current_page + 1}?",
                QMessageBox.StandardButton.Yes | QMessageBox.StandardButton.No | QMessageBox.StandardButton.Cancel
            )
            
            if reply == QMessageBox.StandardButton.Cancel:
                doc.close()
                return
            
            if reply == QMessageBox.StandardButton.Yes:  # All pages
                for page in doc:
                    if angle == 0:
                        page.set_rotation(0)  # Reset to original
                    else:
                        page.set_rotation(page.rotation + angle)
            else:  # Current page only
                page = doc[current_page]
                if angle == 0:
                    page.set_rotation(0)
                else:
                    page.set_rotation(page.rotation + angle)
            
            doc.save(self.temp_pdf_path)
            doc.close()
            
            # Reload in viewer
            self.pdf_document.load(self.temp_pdf_path)
            self.pdf_modified = True
            self.status_bar.showMessage(f"🐹 *spin spin* Rotated {angle}°!")
            
        except Exception as e:
            QMessageBox.critical(self, "🐹 CHONKER Error", f"Rotation failed: {str(e)}")
    
    def split_pdf(self):
        """CHONKER splits the PDF into individual pages"""
        if not self.current_pdf_path:
            return
        
        try:
            output_dir = QFileDialog.getExistingDirectory(self, "🐹 Where to save split pages?")
            if not output_dir:
                return
            
            doc = fitz.open(self.current_pdf_path)
            basename = os.path.splitext(os.path.basename(self.current_pdf_path))[0]
            
            for i, page in enumerate(doc):
                new_doc = fitz.open()
                new_doc.insert_pdf(doc, from_page=i, to_page=i)
                output_path = os.path.join(output_dir, f"{basename}_page_{i+1}.pdf")
                new_doc.save(output_path)
                new_doc.close()
            
            doc.close()
            self.status_bar.showMessage(f"🐹 *chomp chomp* Split into {doc.page_count} pages!")
            QMessageBox.information(self, "🐹 Success", f"Split PDF into {doc.page_count} individual files!")
            
        except Exception as e:
            QMessageBox.critical(self, "🐹 CHONKER Error", f"Split failed: {str(e)}")
    
    def merge_pdfs(self):
        """CHONKER merges multiple PDFs"""
        files, _ = QFileDialog.getOpenFileNames(
            self, "🐹 Select PDFs to merge", "", "PDF Files (*.pdf)"
        )
        
        if len(files) < 2:
            return
        
        try:
            merged_doc = fitz.open()
            
            for file_path in files:
                doc = fitz.open(file_path)
                merged_doc.insert_pdf(doc)
                doc.close()
            
            output_path, _ = QFileDialog.getSaveFileName(
                self, "🐹 Save merged PDF as", "", "PDF Files (*.pdf)"
            )
            
            if output_path:
                merged_doc.save(output_path)
                self.status_bar.showMessage("🐹 *nom nom* PDFs merged successfully!")
                QMessageBox.information(self, "🐹 Success", f"Merged {len(files)} PDFs!")
            
            merged_doc.close()
            
        except Exception as e:
            QMessageBox.critical(self, "🐹 CHONKER Error", f"Merge failed: {str(e)}")
    
    def extract_pages(self):
        """CHONKER extracts specific pages"""
        if not self.current_pdf_path:
            return
        
        doc = fitz.open(self.current_pdf_path)
        total_pages = doc.page_count
        doc.close()
        
        pages_str, ok = QInputDialog.getText(
            self, "🐹 Extract Pages",
            f"Enter page numbers to extract (1-{total_pages}):\\nExamples: 1,3,5 or 1-5 or 1-3,7,9-11"
        )
        
        if not ok or not pages_str:
            return
        
        try:
            # Parse page numbers
            pages = []
            for part in pages_str.split(','):
                if '-' in part:
                    start, end = map(int, part.split('-'))
                    pages.extend(range(start-1, end))
                else:
                    pages.append(int(part)-1)
            
            # Extract pages
            doc = fitz.open(self.current_pdf_path)
            new_doc = fitz.open()
            
            for page_num in pages:
                if 0 <= page_num < doc.page_count:
                    new_doc.insert_pdf(doc, from_page=page_num, to_page=page_num)
            
            output_path, _ = QFileDialog.getSaveFileName(
                self, "🐹 Save extracted pages as", "", "PDF Files (*.pdf)"
            )
            
            if output_path:
                new_doc.save(output_path)
                self.status_bar.showMessage(f"🐹 *munch* Extracted {len(pages)} pages!")
                QMessageBox.information(self, "🐹 Success", f"Extracted {len(pages)} pages!")
            
            new_doc.close()
            doc.close()
            
        except Exception as e:
            QMessageBox.critical(self, "🐹 CHONKER Error", f"Extract failed: {str(e)}")
    
    def insert_pages(self):
        """CHONKER inserts pages from another PDF"""
        if not self.current_pdf_path:
            return
        
        insert_file, _ = QFileDialog.getOpenFileName(
            self, "🐹 Select PDF to insert", "", "PDF Files (*.pdf)"
        )
        
        if not insert_file:
            return
        
        try:
            # Create temp file if first modification
            if not self.pdf_modified:
                self.create_temp_pdf()
            
            doc = fitz.open(self.temp_pdf_path or self.current_pdf_path)
            insert_doc = fitz.open(insert_file)
            
            # Get position
            position, ok = QInputDialog.getInt(
                self, "🐹 Insert Position",
                f"Insert after page (0 for beginning, {doc.page_count} for end):",
                value=doc.page_count, min=0, max=doc.page_count
            )
            
            if ok:
                doc.insert_pdf(insert_doc, start_at=position)
                doc.save(self.temp_pdf_path)
                
                # Reload
                self.pdf_document.load(self.temp_pdf_path)
                self.pdf_modified = True
                self.status_bar.showMessage(f"🐹 *gulp* Inserted {insert_doc.page_count} pages!")
            
            doc.close()
            insert_doc.close()
            
        except Exception as e:
            QMessageBox.critical(self, "🐹 CHONKER Error", f"Insert failed: {str(e)}")
    
    def delete_pages(self):
        """CHONKER removes pages"""
        if not self.current_pdf_path:
            return
        
        # Create temp file if first modification
        if not self.pdf_modified:
            self.create_temp_pdf()
        
        doc = fitz.open(self.temp_pdf_path or self.current_pdf_path)
        total_pages = doc.page_count
        
        pages_str, ok = QInputDialog.getText(
            self, "🐹 Delete Pages",
            f"Enter page numbers to delete (1-{total_pages}):\\nExamples: 1,3,5 or 1-5"
        )
        
        if not ok or not pages_str:
            doc.close()
            return
        
        try:
            # Parse page numbers
            pages_to_delete = []
            for part in pages_str.split(','):
                if '-' in part:
                    start, end = map(int, part.split('-'))
                    pages_to_delete.extend(range(start-1, end))
                else:
                    pages_to_delete.append(int(part)-1)
            
            # Delete in reverse order
            for page_num in sorted(pages_to_delete, reverse=True):
                if 0 <= page_num < doc.page_count:
                    doc.delete_page(page_num)
            
            doc.save(self.temp_pdf_path)
            doc.close()
            
            # Reload
            self.pdf_document.load(self.temp_pdf_path)
            self.pdf_modified = True
            self.status_bar.showMessage(f"🐹 *chomp* Deleted {len(pages_to_delete)} pages!")
            
        except Exception as e:
            doc.close()
            QMessageBox.critical(self, "🐹 CHONKER Error", f"Delete failed: {str(e)}")
    
    def clean_pdf(self):
        """CHONKER cleans the PDF"""
        if not self.current_pdf_path:
            return
        
        try:
            # Create temp file if first modification
            if not self.pdf_modified:
                self.create_temp_pdf()
            
            doc = fitz.open(self.temp_pdf_path or self.current_pdf_path)
            
            cleaned_items = 0
            for page in doc:
                # Remove annotations
                annot = page.first_annot
                while annot:
                    next_annot = annot.next
                    page.delete_annot(annot)
                    annot = next_annot
                    cleaned_items += 1
            
            doc.save(self.temp_pdf_path)
            doc.close()
            
            # Reload
            self.pdf_document.load(self.temp_pdf_path)
            self.pdf_modified = True
            self.status_bar.showMessage(f"🐹 *scrub scrub* Cleaned {cleaned_items} annotations!")
            
        except Exception as e:
            QMessageBox.critical(self, "🐹 CHONKER Error", f"Clean failed: {str(e)}")
    
    def compress_pdf(self):
        """CHONKER compresses the PDF"""
        if not self.current_pdf_path:
            return
        
        try:
            # Create temp file if first modification
            if not self.pdf_modified:
                self.create_temp_pdf()
            
            doc = fitz.open(self.temp_pdf_path or self.current_pdf_path)
            
            # PyMuPDF compression options
            doc.save(
                self.temp_pdf_path,
                garbage=4,  # Maximum garbage collection
                clean=True,  # Clean up PDF
                deflate=True,  # Compress streams
                deflate_images=True,  # Compress images
                deflate_fonts=True  # Compress fonts
            )
            doc.close()
            
            # Check size reduction
            original_size = os.path.getsize(self.current_pdf_path)
            new_size = os.path.getsize(self.temp_pdf_path)
            reduction = (1 - new_size/original_size) * 100
            
            # Reload
            self.pdf_document.load(self.temp_pdf_path)
            self.pdf_modified = True
            self.status_bar.showMessage(f"🐹 *squeeze* Compressed by {reduction:.1f}%!")
            
        except Exception as e:
            QMessageBox.critical(self, "🐹 CHONKER Error", f"Compress failed: {str(e)}")
    
    def optimize_for_extraction(self):
        """CHONKER optimizes PDF for better extraction"""
        if not self.current_pdf_path:
            return
        
        try:
            # Create temp file if first modification
            if not self.pdf_modified:
                self.create_temp_pdf()
            
            doc = fitz.open(self.temp_pdf_path or self.current_pdf_path)
            
            # Optimization steps
            for page in doc:
                # Remove annotations
                annot = page.first_annot
                while annot:
                    next_annot = annot.next
                    page.delete_annot(annot)
                    annot = next_annot
                
                # Clean up page
                page.clean_contents()
            
            # Save with optimization
            doc.save(
                self.temp_pdf_path,
                garbage=4,
                clean=True,
                deflate=True,
                ascii=True,  # ASCII encoding for better compatibility
                expand=True  # Expand compressed objects
            )
            doc.close()
            
            # Reload
            self.pdf_document.load(self.temp_pdf_path)
            self.pdf_modified = True
            self.status_bar.showMessage("🐹 *polish polish* Optimized for extraction!")
            
        except Exception as e:
            QMessageBox.critical(self, "🐹 CHONKER Error", f"Optimize failed: {str(e)}")
    
    def create_temp_pdf(self):
        """Create a temporary copy of the PDF for modifications"""
        if not self.temp_pdf_path:
            temp_file = tempfile.NamedTemporaryFile(suffix='.pdf', delete=False)
            self.temp_pdf_path = temp_file.name
            temp_file.close()
            
            # Copy original to temp
            import shutil
            shutil.copy2(self.current_pdf_path, self.temp_pdf_path)
    
    def save_modified_pdf(self):
        """Save the modified PDF"""
        if not self.pdf_modified or not self.temp_pdf_path:
            QMessageBox.information(self, "🐹 CHONKER", "No modifications to save!")
            return
        
        output_path, _ = QFileDialog.getSaveFileName(
            self, "🐹 Save modified PDF as", "", "PDF Files (*.pdf)"
        )
        
        if output_path:
            try:
                import shutil
                shutil.copy2(self.temp_pdf_path, output_path)
                self.status_bar.showMessage("🐹 *happy dance* Saved modified PDF!")
                QMessageBox.information(self, "🐹 Success", "Modified PDF saved successfully!")
            except Exception as e:
                QMessageBox.critical(self, "🐹 CHONKER Error", f"Save failed: {str(e)}")
    
    def revert_changes(self):
        """Revert all modifications"""
        if not self.pdf_modified:
            return
        
        reply = QMessageBox.question(
            self, "🐹 CHONKER Asks",
            "Revert all changes to the original PDF?",
            QMessageBox.StandardButton.Yes | QMessageBox.StandardButton.No
        )
        
        if reply == QMessageBox.StandardButton.Yes:
            if self.temp_pdf_path and os.path.exists(self.temp_pdf_path):
                os.unlink(self.temp_pdf_path)
            self.temp_pdf_path = None
            self.pdf_modified = False
            self.pdf_document.load(self.current_pdf_path)
            self.status_bar.showMessage("🐹 *shake shake* Reverted to original!")
    
    def show_batch_dialog(self):
        """Show batch processing dialog"""
        dialog = BatchProcessDialog(self)
        if dialog.exec() == QDialog.DialogCode.Accepted:
            files = dialog.get_selected_files()
            operations = dialog.get_selected_operations()
            
            if files and operations:
                self.start_batch_processing(files, operations)
    
    def start_batch_processing(self, files: List[str], operations: Dict[str, bool]):
        """Start batch processing with CHONKER"""
        # Create progress dialog
        progress_dialog = QDialog(self)
        progress_dialog.setWindowTitle("🐹 CHONKER Batch Processing")
        progress_dialog.setModal(True)
        progress_dialog.resize(600, 400)
        
        layout = QVBoxLayout(progress_dialog)
        
        # Overall progress
        overall_label = QLabel("🐹 *munch munch* Processing files...")
        layout.addWidget(overall_label)
        
        overall_progress = QProgressBar()
        layout.addWidget(overall_progress)
        
        # Results list
        results_list = QListWidget()
        layout.addWidget(results_list)
        
        # Cancel button
        cancel_btn = QPushButton("🐹 Stop Processing")
        layout.addWidget(cancel_btn)
        
        # Start worker
        worker = BatchProcessor(files, operations)
        
        def update_progress(current, total, message):
            overall_progress.setRange(0, total)
            overall_progress.setValue(current)
            overall_label.setText(message)
        
        def file_completed(filepath, success, message):
            icon = "✅" if success else "❌"
            results_list.addItem(f"{icon} {os.path.basename(filepath)}: {message}")
        
        def all_completed():
            overall_label.setText("🐹 *burp* All done!")
            cancel_btn.setText("Close")
            cancel_btn.clicked.disconnect()
            cancel_btn.clicked.connect(progress_dialog.accept)
        
        worker.progress.connect(update_progress)
        worker.file_completed.connect(file_completed)
        worker.all_completed.connect(all_completed)
        
        cancel_btn.clicked.connect(worker.stop)
        
        worker.start()
        progress_dialog.exec()
    
    def create_snyfter_interface(self) -> QWidget:
        """Create SNYFTER's archival interface"""
        widget = QWidget()
        layout = QVBoxLayout(widget)
        
        # Snyfter's tools
        toolbar = QHBoxLayout()
        
        self.search_input = QLineEdit()
        self.search_input.setPlaceholderText("🐁 Search the archives...")
        self.search_input.returnPressed.connect(self.search_archives)
        toolbar.addWidget(self.search_input)
        
        self.search_btn = QPushButton("🐁 Search")
        self.search_btn.clicked.connect(self.search_archives)
        toolbar.addWidget(self.search_btn)
        
        self.catalog_btn = QPushButton("🐁 View Recent")
        self.catalog_btn.clicked.connect(self.view_recent)
        toolbar.addWidget(self.catalog_btn)
        
        toolbar.addStretch()
        layout.addLayout(toolbar)
        
        # Splitter for search results and document view
        splitter = QSplitter(Qt.Orientation.Horizontal)
        
        # Search results (Snyfter's card catalog)
        self.results_list = QListWidget()
        self.results_list.itemClicked.connect(self.load_document)
        splitter.addWidget(self.results_list)
        
        # Document viewer (Snyfter's reading desk)
        self.document_viewer = QTextEdit()
        self.document_viewer.setReadOnly(True)
        splitter.addWidget(self.document_viewer)
        
        splitter.setSizes([400, 1000])
        layout.addWidget(splitter)
        
        # Snyfter's notes section
        notes_layout = QHBoxLayout()
        notes_layout.addWidget(QLabel("🐁 Research Notes:"))
        self.notes_input = QLineEdit()
        self.notes_input.setPlaceholderText("Add notes about this document...")
        notes_layout.addWidget(self.notes_input)
        
        self.add_note_btn = QPushButton("📝 Add Note")
        self.add_note_btn.clicked.connect(self.add_research_note)
        notes_layout.addWidget(self.add_note_btn)
        
        layout.addLayout(notes_layout)
        
        return widget
    
    def eventFilter(self, obj, event):
        """Handle Tab key to switch between CHONKER and SNYFTER"""
        if event.type() == QEvent.Type.KeyPress:
            key_event = event
            if key_event.key() == Qt.Key.Key_Tab and not key_event.modifiers():
                # Don't switch if we're in a text input field
                if not isinstance(obj, (QLineEdit, QTextEdit, QPlainTextEdit)):
                    self.toggle_mode()
                    return True
        return super().eventFilter(obj, event)
    
    def toggle_mode(self):
        """Switch between CHONKER and SNYFTER modes"""
        import random
        
        if self.current_mode == Mode.CHONKER:
            self.current_mode = Mode.SNYFTER
            self.stacked_widget.setCurrentWidget(self.snyfter_widget)
            
            # Update highlighting - SNYFTER active
            self.chonker_label.setStyleSheet("""
                QLabel {
                    padding: 5px;
                    background: transparent;
                    border: 2px solid transparent;
                    border-radius: 5px;
                }
            """)
            self.snyfter_label.setStyleSheet("""
                QLabel {
                    padding: 5px;
                    background: #E6E6FA;
                    border: 2px solid #E6E6FA;
                    border-radius: 5px;
                }
            """)
            
            # Update terminal feedback
            self.terminal_feedback.setText("SNYFTER MODE ACTIVE")
            greeting = random.choice(SnyfterPersonality.GREETINGS).replace("🐁 ", "")
            self.status_bar.showMessage(greeting)
        else:
            self.current_mode = Mode.CHONKER
            self.stacked_widget.setCurrentWidget(self.chonker_widget)
            
            # Update highlighting - CHONKER active
            self.chonker_label.setStyleSheet("""
                QLabel {
                    padding: 5px;
                    background: #FFE4B5;
                    border: 2px solid #FFE4B5;
                    border-radius: 5px;
                }
            """)
            self.snyfter_label.setStyleSheet("""
                QLabel {
                    padding: 5px;
                    background: transparent;
                    border: 2px solid transparent;
                    border-radius: 5px;
                }
            """)
            
            # Update terminal feedback
            self.terminal_feedback.setText("CHONKER MODE ACTIVE")
            greeting = random.choice(ChonkerPersonality.GREETINGS).replace("🐹 ", "")
            self.status_bar.showMessage(greeting)
    
    def feed_chonker(self):
        """Feed a PDF to CHONKER"""
        file_path, _ = QFileDialog.getOpenFileName(
            self, "🐹 Select PDF for CHONKER", "", "PDF Files (*.pdf)"
        )
        
        if file_path:
            self.current_pdf_path = file_path
            self.original_pdf_path = file_path
            self.pdf_document.load(file_path)
            self.digest_btn.setEnabled(True)
            self.pdf_tools_btn.setEnabled(True)
            self.pdf_modified = False
            self.status_bar.showMessage(f"🐹 *sniff sniff* Examining {os.path.basename(file_path)}...")
    
    def digest_pdf(self):
        """CHONKER digests the PDF"""
        if not hasattr(self, 'current_pdf_path'):
            return
        
        self.chonker_progress.setVisible(True)
        self.chonker_progress.setRange(0, 0)
        self.digest_btn.setEnabled(False)
        
        # Start CHONKER's digestion
        self.chonker_worker = ChonkerWorker(self.current_pdf_path)
        self.chonker_worker.finished.connect(self.on_digestion_complete)
        self.chonker_worker.progress.connect(self.status_bar.showMessage)
        self.chonker_worker.error.connect(self.on_digestion_error)
        self.chonker_worker.start()
    
    def on_digestion_complete(self, content: dict):
        """CHONKER finished digesting"""
        self.current_document = content
        self.chonker_progress.setVisible(False)
        self.digest_btn.setEnabled(True)
        self.pass_to_snyfter_btn.setEnabled(True)
        
        # Show results
        self.html_viewer.setHtml(content['html'])
        
        # Populate chunks table
        self.chunks_table.setRowCount(len(content['chunks']))
        for i, chunk in enumerate(content['chunks']):
            self.chunks_table.setItem(i, 0, QTableWidgetItem(chunk['type']))
            self.chunks_table.setItem(i, 1, QTableWidgetItem(chunk['content'][:50] + "..."))
            self.chunks_table.setItem(i, 2, QTableWidgetItem(str(chunk['page'])))
            self.chunks_table.setItem(i, 3, QTableWidgetItem("✓"))
        
        # Populate editable chunks list
        self.chunks_widget.clear()
        for chunk in content['chunks']:
            # Format: [TYPE - Page X] content...
            item_text = f"[{chunk['type'].upper()} - Page {chunk['page']}] {chunk['content'][:100]}..."
            self.chunks_widget.addItem(item_text)
        
        # Update bidirectional selector with chunk data
        if self.bidirectional_selector:
            self.bidirectional_selector.set_chunks(content['chunks'])
        
        import random
        self.status_bar.showMessage(random.choice(ChonkerPersonality.SUCCESS))
    
    def on_digestion_error(self, error: str):
        """CHONKER had trouble digesting"""
        self.chonker_progress.setVisible(False)
        self.digest_btn.setEnabled(True)
        QMessageBox.warning(self, "🐹 CHONKER Error", error)
        import random
        self.status_bar.showMessage(random.choice(ChonkerPersonality.ERROR))
    
    def pass_to_snyfter(self):
        """Pass the digested document to SNYFTER for filing"""
        if not self.current_document:
            return
        
        # Calculate file hash
        hasher = hashlib.sha256()
        with open(self.current_pdf_path, 'rb') as f:
            hasher.update(f.read())
        file_hash = hasher.hexdigest()
        
        # File in Snyfter's archives
        try:
            doc_id = self.snyfter_db.file_document(
                os.path.basename(self.current_pdf_path),
                file_hash,
                self.current_document.get('chonker_notes', '')
            )
            
            # Save content
            with sqlite3.connect(self.snyfter_db.db_path) as conn:
                conn.execute(
                    "INSERT INTO content_cards (document_id, content_type, content) VALUES (?, ?, ?)",
                    (doc_id, 'html', self.current_document['html'])
                )
                conn.execute(
                    "INSERT INTO content_cards (document_id, content_type, content) VALUES (?, ?, ?)",
                    (doc_id, 'markdown', self.current_document['markdown'])
                )
            
            import random
            self.status_bar.showMessage(random.choice(SnyfterPersonality.SUCCESS))
            QMessageBox.information(self, "🐁 SNYFTER", f"Document filed in archives as #{doc_id}")
            
        except Exception as e:
            QMessageBox.critical(self, "🐁 SNYFTER Error", f"Filing failed: {str(e)}")
    
    def search_archives(self):
        """SNYFTER searches the archives"""
        query = self.search_input.text()
        if not query:
            return
        
        import random
        self.status_bar.showMessage(random.choice(SnyfterPersonality.SEARCHING))
        
        results = self.snyfter_db.search_archives(query)
        
        self.results_list.clear()
        for doc in results:
            item_text = f"#{doc['id']}: {doc['filename']} ({doc['processed_date'][:10]})"
            self.results_list.addItem(item_text)
        
        if results:
            self.status_bar.showMessage(f"🐁 Found {len(results)} documents")
        else:
            self.status_bar.showMessage("🐁 No documents found in archives")
    
    def view_recent(self):
        """View recent documents in SNYFTER's archives"""
        self.search_input.setText("")
        self.search_archives()
    
    def load_document(self, item):
        """Load a document from SNYFTER's archives"""
        # Extract document ID from item text
        doc_id = int(item.text().split(':')[0][1:])
        
        with sqlite3.connect(self.snyfter_db.db_path) as conn:
            conn.row_factory = sqlite3.Row
            cursor = conn.execute(
                "SELECT * FROM content_cards WHERE document_id = ? AND content_type = 'html'",
                (doc_id,)
            )
            row = cursor.fetchone()
            
            if row:
                self.document_viewer.setHtml(row['content'])
                self.status_bar.showMessage(f"🐁 Viewing document #{doc_id}")
    
    def add_research_note(self):
        """Add a research note to the current document"""
        note_text = self.notes_input.text().strip()
        if not note_text:
            return
        
        # Get current document ID from selected item
        selected_items = self.results_list.selectedItems()
        if not selected_items:
            QMessageBox.information(self, "🐁 SNYFTER", "Please select a document first!")
            return
        
        try:
            # Extract document ID
            doc_id = int(selected_items[0].text().split(':')[0][1:])
            
            # Add note to database
            with sqlite3.connect(self.snyfter_db.db_path) as conn:
                conn.execute(
                    "INSERT INTO research_notes (document_id, note) VALUES (?, ?)",
                    (doc_id, note_text)
                )
            
            self.notes_input.clear()
            self.status_bar.showMessage("🐁 *scribbles* Note added to archives!")
            
        except Exception as e:
            QMessageBox.critical(self, "🐁 SNYFTER Error", f"Failed to add note: {str(e)}")


def main():
    if not DEPENDENCIES_AVAILABLE:
        print("🐹🐁 Error: CHONKER & SNYFTER need their dependencies! Please install:")
        print("  pip install docling PyMuPDF")
        sys.exit(1)
    
    app = QApplication(sys.argv)
    app.setApplicationName("CHONKER & SNYFTER")
    
    window = ChonkerSnyfterMainWindow()
    window.show()
    
    sys.exit(app.exec())


if __name__ == "__main__":
    main()