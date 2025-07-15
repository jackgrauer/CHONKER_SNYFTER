#!/usr/bin/env python3
"""
🐹 CHONKER - PDF Document Extraction & Quality Control System
A focused tool for extracting content from PDFs with human-in-the-loop QC and database storage.
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

from PyQt6.QtWidgets import (
    QApplication, QMainWindow, QSplitter, QVBoxLayout, QHBoxLayout,
    QWidget, QPushButton, QFileDialog, QMessageBox, QTextEdit,
    QLabel, QComboBox, QScrollArea, QFrame, QProgressBar,
    QTableWidget, QTableWidgetItem, QTabWidget, QStatusBar, QTextBrowser
)
from PyQt6.QtCore import Qt, QThread, pyqtSignal, QTimer, QPointF, QRectF, QObject, QEvent
from PyQt6.QtGui import QFont, QPalette, QColor, QTextCharFormat, QTextCursor, QPainter, QPen, QBrush
from PyQt6.QtPdf import QPdfDocument, QPdfSelection
from PyQt6.QtPdfWidgets import QPdfView

try:
    from docling.document_converter import DocumentConverter
    import fitz  # PyMuPDF
    DEPENDENCIES_AVAILABLE = True
except ImportError:
    DEPENDENCIES_AVAILABLE = False


class DatabaseManager:
    """Handles all database operations for document storage and retrieval."""
    
    def __init__(self, db_path: str = "chonker_documents.db"):
        self.db_path = db_path
        self.init_database()
    
    def init_database(self):
        """Initialize the database with schema."""
        # Check if database exists
        db_exists = os.path.exists(self.db_path)
        
        if not db_exists:
            # Read and execute schema for new database
            schema_path = Path(__file__).parent / "database_schema.sql"
            if schema_path.exists():
                with open(schema_path, 'r') as f:
                    schema = f.read()
                
                with sqlite3.connect(self.db_path) as conn:
                    conn.executescript(schema)
            else:
                # Fallback minimal schema
                with sqlite3.connect(self.db_path) as conn:
                    conn.execute("""
                        CREATE TABLE IF NOT EXISTS documents (
                            id INTEGER PRIMARY KEY AUTOINCREMENT,
                            filename TEXT NOT NULL,
                            file_hash TEXT UNIQUE NOT NULL,
                            processed_date TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                            status TEXT DEFAULT 'pending'
                        )
                    """)
    
    def add_document(self, filename: str, file_path: str, file_hash: str) -> int:
        """Add a new document to the database."""
        with sqlite3.connect(self.db_path) as conn:
            cursor = conn.execute(
                "INSERT INTO documents (filename, file_path, file_hash) VALUES (?, ?, ?)",
                (filename, file_path, file_hash)
            )
            return cursor.lastrowid
    
    def get_document_by_hash(self, file_hash: str) -> Optional[Dict]:
        """Check if document already exists."""
        with sqlite3.connect(self.db_path) as conn:
            conn.row_factory = sqlite3.Row
            cursor = conn.execute(
                "SELECT * FROM documents WHERE file_hash = ?", (file_hash,)
            )
            row = cursor.fetchone()
            return dict(row) if row else None
    
    def save_extracted_content(self, document_id: int, content_type: str, content: str):
        """Save extracted content to database."""
        with sqlite3.connect(self.db_path) as conn:
            conn.execute(
                "INSERT INTO extracted_content (document_id, content_type, content) VALUES (?, ?, ?)",
                (document_id, content_type, content)
            )
    
    def get_recent_documents(self, limit: int = 20) -> List[Dict]:
        """Get recently processed documents."""
        with sqlite3.connect(self.db_path) as conn:
            conn.row_factory = sqlite3.Row
            cursor = conn.execute(
                "SELECT * FROM documents ORDER BY processed_date DESC LIMIT ?", (limit,)
            )
            return [dict(row) for row in cursor.fetchall()]


class PDFPreprocessor:
    """Handles PDF cleaning and preprocessing for better extraction."""
    
    @staticmethod
    def clean_pdf(input_path: str, output_path: str) -> bool:
        """Clean PDF to improve extraction quality."""
        try:
            doc = fitz.open(input_path)
            
            # Basic cleaning operations
            for page in doc:
                # Remove annotations that might interfere
                annot = page.first_annot
                while annot:
                    next_annot = annot.next
                    page.delete_annot(annot)
                    annot = next_annot
                
                # Ensure text is selectable (basic check)
                text = page.get_text()
                if not text.strip():
                    # Page might be image-based, could add OCR here
                    pass
            
            doc.save(output_path)
            doc.close()
            return True
            
        except Exception as e:
            print(f"PDF preprocessing failed: {e}")
            return False


class BidirectionalSelector(QObject):
    """Handles bidirectional selection between PDF and extracted content."""
    
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
        """Update chunk mapping when new content is extracted."""
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
        """Handle mouse clicks on PDF view."""
        if obj == self.pdf_view.viewport() and event.type() == QEvent.Type.MouseButtonPress:
            if event.button() == Qt.MouseButton.LeftButton:
                self.on_pdf_click(event.pos())
        return False
    
    def on_pdf_click(self, pos):
        """Handle click on PDF - find and highlight corresponding chunk."""
        # Get current page
        nav = self.pdf_view.pageNavigator()
        if not nav:
            return
            
        current_page = nav.currentPage()
        
        # For now, just cycle through chunks on the current page when clicked
        # (Full coordinate mapping would require more complex PDF rendering info)
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
    
    def point_in_bbox(self, point: QPointF, bbox: Dict) -> bool:
        """Check if point is within bounding box."""
        if not bbox or not all(k in bbox for k in ['x', 'y', 'width', 'height']):
            return False
        
        x, y = point.x(), point.y()
        return (bbox['x'] <= x <= bbox['x'] + bbox['width'] and
                bbox['y'] <= y <= bbox['y'] + bbox['height'])
    
    def on_chunk_selected(self):
        """Handle selection in chunks table."""
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
        """Highlight chunk in both views."""
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
        
        # Markdown viewer  
        elif self.content_tabs.currentIndex() == 1:  # Markdown tab
            md_viewer = self.content_tabs.widget(1)
            if isinstance(md_viewer, QTextEdit):
                self.highlight_text_in_viewer(md_viewer, content)
        
        # Store current highlight for clearing
        self.current_highlight = chunk_id
    
    def highlight_text_in_viewer(self, viewer: QTextEdit, text: str):
        """Highlight text in a QTextEdit viewer."""
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
    
    def clear_highlights(self):
        """Clear all highlights."""
        self.current_highlight = None
        # Clear table selection
        self.chunks_table.clearSelection()


class ExtractionWorker(QThread):
    """Background thread for PDF content extraction."""
    
    finished = pyqtSignal(dict)  # Emits extraction results
    progress = pyqtSignal(str)   # Status updates
    error = pyqtSignal(str)      # Error messages
    
    def __init__(self, pdf_path: str):
        super().__init__()
        self.pdf_path = pdf_path
    
    def run(self):
        try:
            self.progress.emit("Starting extraction...")
            
            # Preprocess PDF if needed
            temp_pdf = None
            original_path = self.pdf_path
            
            # Check if PDF needs cleaning
            if self.needs_preprocessing(self.pdf_path):
                self.progress.emit("Preprocessing PDF...")
                temp_pdf = tempfile.NamedTemporaryFile(suffix='.pdf', delete=False)
                temp_pdf.close()
                
                if PDFPreprocessor.clean_pdf(self.pdf_path, temp_pdf.name):
                    self.pdf_path = temp_pdf.name
                else:
                    self.progress.emit("Preprocessing failed, using original...")
            
            # Extract content using Docling
            self.progress.emit("Extracting content...")
            converter = DocumentConverter()
            result = converter.convert(self.pdf_path)
            
            # Process extraction results
            content = {
                'html': result.document.export_to_html(),
                'markdown': result.document.export_to_markdown(),
                'chunks': [],
                'tables': []
            }
            
            # Extract structured chunks with positioning
            chunk_index = 0
            for item, level in result.document.iterate_items():
                item_type = type(item).__name__
                
                chunk = {
                    'index': chunk_index,
                    'type': item_type.lower().replace('item', ''),
                    'content': getattr(item, 'text', str(item)),
                    'level': level,
                    'page': getattr(item, 'prov', [{}])[0].get('page_no', 0) if hasattr(item, 'prov') else 0,
                    'bbox': getattr(item, 'prov', [{}])[0].get('bbox', {}) if hasattr(item, 'prov') else {}
                }
                content['chunks'].append(chunk)
                chunk_index += 1
                
                # Extract tables separately
                if item_type == 'TableItem' and hasattr(item, 'export_to_dataframe'):
                    try:
                        df = item.export_to_dataframe()
                        table_data = {
                            'index': len(content['tables']),
                            'data': df.values.tolist(),
                            'headers': df.columns.tolist(),
                            'page': chunk['page'],
                            'bbox': chunk['bbox']
                        }
                        content['tables'].append(table_data)
                    except:
                        pass
            
            # Cleanup
            if temp_pdf:
                os.unlink(temp_pdf.name)
            
            self.finished.emit(content)
            
        except Exception as e:
            if temp_pdf:
                os.unlink(temp_pdf.name)
            self.error.emit(str(e))
    
    def needs_preprocessing(self, pdf_path: str) -> bool:
        """Check if PDF needs preprocessing."""
        try:
            doc = fitz.open(pdf_path)
            needs_clean = False
            
            # Check first few pages for quality issues
            for page_num in range(min(3, doc.page_count)):
                page = doc[page_num]
                text = page.get_text()
                
                # Heuristics for determining if preprocessing is needed
                if len(text.strip()) < 50:  # Very little text
                    needs_clean = True
                if page.first_annot:  # Has annotations
                    needs_clean = True
                    
            doc.close()
            return needs_clean
            
        except:
            return False


class ChonkerMainWindow(QMainWindow):
    """Main application window with PDF viewer and content editor."""
    
    def __init__(self):
        super().__init__()
        self.db = DatabaseManager()
        self.current_document_id = None
        self.extracted_content = None
        self.bidirectional_selector = None
        self.init_ui()
    
    def init_ui(self):
        self.setWindowTitle("🐹 CHONKER - PDF Document Extraction & QC")
        self.setGeometry(100, 100, 1600, 1000)
        
        # Central widget
        central_widget = QWidget()
        self.setCentralWidget(central_widget)
        layout = QVBoxLayout(central_widget)
        
        # Top toolbar
        toolbar_layout = QHBoxLayout()
        
        self.open_btn = QPushButton("📂 Open PDF")
        self.open_btn.clicked.connect(self.open_pdf)
        toolbar_layout.addWidget(self.open_btn)
        
        self.extract_btn = QPushButton("🔍 Extract Content")
        self.extract_btn.clicked.connect(self.extract_content)
        self.extract_btn.setEnabled(False)
        toolbar_layout.addWidget(self.extract_btn)
        
        self.save_db_btn = QPushButton("💾 Save to Database")
        self.save_db_btn.clicked.connect(self.save_to_database)
        self.save_db_btn.setEnabled(False)
        toolbar_layout.addWidget(self.save_db_btn)
        
        self.recent_btn = QPushButton("📋 Recent Documents")
        self.recent_btn.clicked.connect(self.show_recent_documents)
        toolbar_layout.addWidget(self.recent_btn)
        
        toolbar_layout.addStretch()
        
        # Quality indicator
        self.quality_label = QLabel("Quality: Not assessed")
        toolbar_layout.addWidget(self.quality_label)
        
        layout.addLayout(toolbar_layout)
        
        # Progress bar
        self.progress_bar = QProgressBar()
        self.progress_bar.setVisible(False)
        layout.addWidget(self.progress_bar)
        
        # Main splitter
        self.splitter = QSplitter(Qt.Orientation.Horizontal)
        layout.addWidget(self.splitter)
        
        # Left side - PDF viewer
        self.setup_pdf_viewer()
        
        # Right side - Extracted content tabs
        self.setup_content_viewer()
        
        # Status bar
        self.status_bar = QStatusBar()
        self.setStatusBar(self.status_bar)
        self.status_bar.showMessage("Ready - Open a PDF to begin")
    
    def setup_pdf_viewer(self):
        """Set up the PDF viewer pane."""
        pdf_widget = QWidget()
        pdf_layout = QVBoxLayout(pdf_widget)
        
        # PDF controls
        controls_layout = QHBoxLayout()
        self.prev_page_btn = QPushButton("← Previous")
        self.prev_page_btn.clicked.connect(self.prev_page)
        self.next_page_btn = QPushButton("Next →")
        self.next_page_btn.clicked.connect(self.next_page)
        self.page_label = QLabel("Page 1 of 1")
        
        controls_layout.addWidget(self.prev_page_btn)
        controls_layout.addWidget(self.page_label)
        controls_layout.addWidget(self.next_page_btn)
        controls_layout.addStretch()
        
        pdf_layout.addLayout(controls_layout)
        
        # PDF viewer
        self.pdf_view = QPdfView(pdf_widget)
        self.pdf_document = QPdfDocument(self)
        self.pdf_view.setDocument(self.pdf_document)
        pdf_layout.addWidget(self.pdf_view)
        
        self.splitter.addWidget(pdf_widget)
    
    def setup_content_viewer(self):
        """Set up the extracted content viewer with tabs."""
        self.content_tabs = QTabWidget()
        
        # HTML content tab
        self.html_viewer = QTextEdit()
        self.html_viewer.setReadOnly(True)
        self.content_tabs.addTab(self.html_viewer, "Extracted HTML")
        
        # Markdown content tab
        self.markdown_viewer = QTextEdit()
        self.markdown_viewer.setReadOnly(True)
        self.content_tabs.addTab(self.markdown_viewer, "Markdown")
        
        # Chunks tab for detailed view
        self.chunks_table = QTableWidget()
        self.chunks_table.setColumnCount(5)
        self.chunks_table.setHorizontalHeaderLabels(["Type", "Content", "Page", "Level", "Confidence"])
        self.content_tabs.addTab(self.chunks_table, "Content Chunks")
        
        # Tables tab
        self.tables_widget = QWidget()
        self.content_tabs.addTab(self.tables_widget, "Extracted Tables")
        
        self.splitter.addWidget(self.content_tabs)
        self.splitter.setSizes([800, 800])
        
        # Initialize bidirectional selector after UI is created
        self.bidirectional_selector = BidirectionalSelector(
            self.pdf_view, self.content_tabs, self.chunks_table
        )
    
    def open_pdf(self):
        """Open a PDF file."""
        file_path, _ = QFileDialog.getOpenFileName(
            self, "Open PDF", "", "PDF Files (*.pdf)"
        )
        
        if file_path:
            self.current_pdf_path = file_path
            self.pdf_document.load(file_path)
            self.extract_btn.setEnabled(True)
            self.update_page_label()
            self.status_bar.showMessage(f"Loaded: {os.path.basename(file_path)}")
            
            # Check if already in database
            file_hash = self.calculate_file_hash(file_path)
            existing_doc = self.db.get_document_by_hash(file_hash)
            if existing_doc:
                self.status_bar.showMessage(f"Document already in database (ID: {existing_doc['id']})")
    
    def extract_content(self):
        """Extract content from the loaded PDF."""
        if not hasattr(self, 'current_pdf_path'):
            return
        
        self.progress_bar.setVisible(True)
        self.progress_bar.setRange(0, 0)  # Indeterminate
        self.extract_btn.setEnabled(False)
        
        # Start extraction in background
        self.extraction_worker = ExtractionWorker(self.current_pdf_path)
        self.extraction_worker.finished.connect(self.on_extraction_finished)
        self.extraction_worker.progress.connect(self.status_bar.showMessage)
        self.extraction_worker.error.connect(self.on_extraction_error)
        self.extraction_worker.start()
    
    def on_extraction_finished(self, content: dict):
        """Handle completed extraction."""
        self.extracted_content = content
        self.progress_bar.setVisible(False)
        self.extract_btn.setEnabled(True)
        self.save_db_btn.setEnabled(True)
        
        # Populate content viewers
        self.html_viewer.setHtml(content['html'])
        self.markdown_viewer.setPlainText(content['markdown'])
        
        # Populate chunks table
        self.populate_chunks_table(content['chunks'])
        
        # Update bidirectional selector with chunk data
        if self.bidirectional_selector:
            self.bidirectional_selector.set_chunks(content['chunks'])
        
        self.status_bar.showMessage("Extraction completed - Review content for quality")
    
    def on_extraction_error(self, error: str):
        """Handle extraction errors."""
        self.progress_bar.setVisible(False)
        self.extract_btn.setEnabled(True)
        QMessageBox.critical(self, "Extraction Error", f"Failed to extract content:\n{error}")
        self.status_bar.showMessage("Extraction failed")
    
    def populate_chunks_table(self, chunks: List[Dict]):
        """Populate the chunks table with extracted content."""
        self.chunks_table.setRowCount(len(chunks))
        
        for i, chunk in enumerate(chunks):
            self.chunks_table.setItem(i, 0, QTableWidgetItem(chunk['type']))
            content_preview = chunk['content'][:100] + "..." if len(chunk['content']) > 100 else chunk['content']
            self.chunks_table.setItem(i, 1, QTableWidgetItem(content_preview))
            self.chunks_table.setItem(i, 2, QTableWidgetItem(str(chunk['page'])))
            self.chunks_table.setItem(i, 3, QTableWidgetItem(str(chunk['level'])))
            self.chunks_table.setItem(i, 4, QTableWidgetItem("N/A"))  # Confidence placeholder
        
        self.chunks_table.resizeColumnsToContents()
    
    def save_to_database(self):
        """Save extracted content to database."""
        if not self.extracted_content:
            return
        
        try:
            # Calculate file hash
            file_hash = self.calculate_file_hash(self.current_pdf_path)
            filename = os.path.basename(self.current_pdf_path)
            
            # Check if already exists
            existing_doc = self.db.get_document_by_hash(file_hash)
            if existing_doc:
                reply = QMessageBox.question(
                    self, "Document Exists", 
                    "This document is already in the database. Update it?",
                    QMessageBox.StandardButton.Yes | QMessageBox.StandardButton.No
                )
                if reply == QMessageBox.StandardButton.No:
                    return
                document_id = existing_doc['id']
            else:
                # Add new document
                document_id = self.db.add_document(filename, self.current_pdf_path, file_hash)
            
            # Save extracted content
            self.db.save_extracted_content(document_id, 'html', self.extracted_content['html'])
            self.db.save_extracted_content(document_id, 'markdown', self.extracted_content['markdown'])
            
            self.current_document_id = document_id
            self.status_bar.showMessage(f"Saved to database (ID: {document_id})")
            QMessageBox.information(self, "Success", f"Document saved to database with ID: {document_id}")
            
        except Exception as e:
            QMessageBox.critical(self, "Database Error", f"Failed to save to database:\n{str(e)}")
    
    def show_recent_documents(self):
        """Show recently processed documents."""
        recent_docs = self.db.get_recent_documents()
        
        # Simple dialog showing recent documents
        msg = "Recent Documents:\n\n"
        for doc in recent_docs[:10]:
            msg += f"ID {doc['id']}: {doc['filename']} ({doc['processed_date']})\n"
        
        QMessageBox.information(self, "Recent Documents", msg)
    
    def calculate_file_hash(self, file_path: str) -> str:
        """Calculate SHA-256 hash of file."""
        hasher = hashlib.sha256()
        with open(file_path, 'rb') as f:
            for chunk in iter(lambda: f.read(4096), b""):
                hasher.update(chunk)
        return hasher.hexdigest()
    
    def prev_page(self):
        """Navigate to previous page."""
        nav = self.pdf_view.pageNavigator()
        if nav and nav.currentPage() > 0:
            nav.jump(nav.currentPage() - 1, QPointF())
            self.update_page_label()
    
    def next_page(self):
        """Navigate to next page."""
        nav = self.pdf_view.pageNavigator()
        if nav and nav.currentPage() < self.pdf_document.pageCount() - 1:
            nav.jump(nav.currentPage() + 1, QPointF())
            self.update_page_label()
    
    def update_page_label(self):
        """Update page navigation label."""
        if self.pdf_document.pageCount() > 0:
            nav = self.pdf_view.pageNavigator()
            current = nav.currentPage() + 1 if nav else 1
            total = self.pdf_document.pageCount()
            self.page_label.setText(f"Page {current} of {total}")


def main():
    if not DEPENDENCIES_AVAILABLE:
        print("🐹 Error: CHONKER needs dependencies! Please install:")
        print("  pip install docling PyMuPDF")
        sys.exit(1)
    
    app = QApplication(sys.argv)
    app.setApplicationName("CHONKER")
    
    window = ChonkerMainWindow()
    window.show()
    
    sys.exit(app.exec())


if __name__ == "__main__":
    main()