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
    QLineEdit, QListWidget, QSplitter, QPlainTextEdit
)
from PyQt6.QtCore import Qt, QThread, pyqtSignal, QTimer, QPointF, QObject, QEvent, QSize
from PyQt6.QtGui import QFont, QPalette, QColor, QTextCharFormat, QTextCursor, QKeyEvent, QPixmap, QIcon
from PyQt6.QtPdf import QPdfDocument
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


class ChonkerSnyfterMainWindow(QMainWindow):
    """The main window that houses both CHONKER and SNYFTER"""
    
    def __init__(self):
        super().__init__()
        self.current_mode = Mode.CHONKER
        self.snyfter_db = SnyfterDatabase()
        self.current_document = None
        
        # Load the ACTUAL Android 7.1 emoji images
        self.chonker_pixmap = QPixmap("assets/emojis/chonker.png")
        self.snyfter_pixmap = QPixmap("assets/emojis/snyfter.png")
        
        self.init_ui()
    
    def init_ui(self):
        self.setWindowTitle("🐹 CHONKER & 🐁 SNYFTER - Document Processing Duo")
        self.setGeometry(100, 100, 1400, 900)
        
        # Central widget with stacked layout
        central_widget = QWidget()
        self.setCentralWidget(central_widget)
        main_layout = QVBoxLayout(central_widget)
        
        # Mode indicator with ACTUAL emoji images
        mode_container = QWidget()
        mode_layout = QHBoxLayout(mode_container)
        mode_layout.setContentsMargins(0, 0, 0, 0)
        
        # Emoji image label
        self.emoji_label = QLabel()
        self.emoji_label.setPixmap(self.chonker_pixmap.scaled(48, 48, Qt.AspectRatioMode.KeepAspectRatio, Qt.TransformationMode.SmoothTransformation))
        
        # Mode text label
        self.mode_text_label = QLabel("CHONKER MODE")
        self.mode_text_label.setStyleSheet("font-size: 24px; font-weight: bold;")
        
        mode_layout.addStretch()
        mode_layout.addWidget(self.emoji_label)
        mode_layout.addWidget(self.mode_text_label)
        mode_layout.addStretch()
        
        # Style the container
        mode_container.setStyleSheet("""
            QWidget {
                background: #FFE4B5;
                border-radius: 10px;
                padding: 10px;
            }
        """)
        main_layout.addWidget(mode_container)
        
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
        
        # Install event filter for Tab key
        self.installEventFilter(self)
    
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
        self.html_viewer = QTextEdit()
        self.html_viewer.setReadOnly(True)
        self.content_tabs.addTab(self.html_viewer, "🐹 Digested HTML")
        
        self.chunks_table = QTableWidget()
        self.chunks_table.setColumnCount(4)
        self.chunks_table.setHorizontalHeaderLabels(["Type", "Content", "Page", "Status"])
        self.content_tabs.addTab(self.chunks_table, "🐹 Chunks")
        
        splitter.addWidget(pdf_widget)
        splitter.addWidget(self.content_tabs)
        splitter.setSizes([700, 700])
        
        layout.addWidget(splitter)
        
        return widget
    
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
                self.toggle_mode()
                return True
        return super().eventFilter(obj, event)
    
    def toggle_mode(self):
        """Switch between CHONKER and SNYFTER modes"""
        if self.current_mode == Mode.CHONKER:
            self.current_mode = Mode.SNYFTER
            self.stacked_widget.setCurrentWidget(self.snyfter_widget)
            
            # Show ACTUAL Android 7.1 SNYFTER emoji
            self.emoji_label.setPixmap(self.snyfter_pixmap.scaled(48, 48, Qt.AspectRatioMode.KeepAspectRatio, Qt.TransformationMode.SmoothTransformation))
            self.mode_text_label.setText("SNYFTER MODE")
            
            # Update container style
            self.emoji_label.parent().setStyleSheet("""
                QWidget {
                    background: #E6E6FA;
                    border-radius: 10px;
                    padding: 10px;
                }
            """)
            import random
            self.status_bar.showMessage(random.choice(SnyfterPersonality.GREETINGS))
        else:
            self.current_mode = Mode.CHONKER
            self.stacked_widget.setCurrentWidget(self.chonker_widget)
            
            # Show ACTUAL Android 7.1 CHONKER emoji
            self.emoji_label.setPixmap(self.chonker_pixmap.scaled(48, 48, Qt.AspectRatioMode.KeepAspectRatio, Qt.TransformationMode.SmoothTransformation))
            self.mode_text_label.setText("CHONKER MODE")
            
            # Update container style
            self.emoji_label.parent().setStyleSheet("""
                QWidget {
                    background: #FFE4B5;
                    border-radius: 10px;
                    padding: 10px;
                }
            """)
            import random
            self.status_bar.showMessage(random.choice(ChonkerPersonality.GREETINGS))
    
    def feed_chonker(self):
        """Feed a PDF to CHONKER"""
        file_path, _ = QFileDialog.getOpenFileName(
            self, "🐹 Select PDF for CHONKER", "", "PDF Files (*.pdf)"
        )
        
        if file_path:
            self.current_pdf_path = file_path
            self.pdf_document.load(file_path)
            self.digest_btn.setEnabled(True)
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
        
        # Populate chunks
        self.chunks_table.setRowCount(len(content['chunks']))
        for i, chunk in enumerate(content['chunks']):
            self.chunks_table.setItem(i, 0, QTableWidgetItem(chunk['type']))
            self.chunks_table.setItem(i, 1, QTableWidgetItem(chunk['content'][:50] + "..."))
            self.chunks_table.setItem(i, 2, QTableWidgetItem(str(chunk['page'])))
            self.chunks_table.setItem(i, 3, QTableWidgetItem("✓"))
        
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
        # Implementation for adding notes
        pass


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