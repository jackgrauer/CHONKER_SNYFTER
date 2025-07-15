#!/Users/jack/chonksnyft-env/bin/python
"""
🐹 CHONKER Qt Faithful - Chunk-based document editor that preserves structure
Each chunk (paragraph, table, heading, etc.) is a separate widget in sequence
"""

import sys
import os
from pathlib import Path
from typing import Optional, List, Dict, Any
from enum import Enum

from PyQt6.QtWidgets import (
    QApplication, QMainWindow, QSplitter, QTextEdit, QVBoxLayout, 
    QWidget, QPushButton, QHBoxLayout, QFileDialog, QMessageBox,
    QToolBar, QStatusBar, QLabel, QComboBox, QScrollArea,
    QTableWidget, QTableWidgetItem, QFrame, QPlainTextEdit
)
from PyQt6.QtCore import Qt, QUrl, pyqtSignal, QThread, QPointF
from PyQt6.QtGui import QAction, QKeySequence, QFont
from PyQt6.QtPdf import QPdfDocument
from PyQt6.QtPdfWidgets import QPdfView

try:
    import fitz  # PyMuPDF
    PYMUPDF_AVAILABLE = True
except ImportError:
    PYMUPDF_AVAILABLE = False

try:
    from docling.document_converter import DocumentConverter
    from docling.datamodel.base_models import InputFormat
    from docling.datamodel.document import DoclingDocument, DocItem
    DOCLING_AVAILABLE = True
except ImportError:
    DOCLING_AVAILABLE = False


class ChunkType(Enum):
    HEADING1 = "heading1"
    HEADING2 = "heading2"
    HEADING3 = "heading3"
    PARAGRAPH = "paragraph"
    TABLE = "table"
    LIST = "list"
    CODE = "code"
    IMAGE = "image"
    QUOTE = "quote"


class ChunkWidget(QFrame):
    """Base class for all chunk widgets"""
    def __init__(self, chunk_type: ChunkType, content: Any):
        super().__init__()
        self.chunk_type = chunk_type
        self.content = content
        
    def get_content(self) -> Any:
        """Get the current content of this chunk"""
        return self.content


class TextChunkWidget(ChunkWidget):
    """Widget for text chunks (paragraphs, headings, etc.)"""
    def __init__(self, chunk_type: ChunkType, text: str):
        super().__init__(chunk_type, text)
        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        
        self.text_edit = QTextEdit()
        
        # Convert to HTML based on chunk type
        if chunk_type == ChunkType.HEADING1:
            self.text_edit.setHtml(f"<h1>{text}</h1>")
        elif chunk_type == ChunkType.HEADING2:
            self.text_edit.setHtml(f"<h2>{text}</h2>")
        elif chunk_type == ChunkType.HEADING3:
            self.text_edit.setHtml(f"<h3>{text}</h3>")
        elif chunk_type == ChunkType.CODE:
            self.text_edit.setHtml(f"<pre><code>{text}</code></pre>")
        elif chunk_type == ChunkType.QUOTE:
            self.text_edit.setHtml(f"<blockquote>{text}</blockquote>")
        else:
            self.text_edit.setHtml(f"<p>{text}</p>")
        
        layout.addWidget(self.text_edit)
    
    def get_content(self) -> str:
        return self.text_edit.toPlainText()


class TableChunkWidget(ChunkWidget):
    """Widget for table chunks"""
    def __init__(self, table_data: Dict[str, Any]):
        super().__init__(ChunkType.TABLE, table_data)
        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        
        # Create table widget
        self.table = QTableWidget()
        data = table_data.get('data', [])
        columns = table_data.get('columns', [])
        
        if data:
            self.table.setRowCount(len(data))
            self.table.setColumnCount(len(data[0]) if data else 0)
            
            if columns:
                self.table.setHorizontalHeaderLabels([str(col) for col in columns])
            
            for row_idx, row_data in enumerate(data):
                for col_idx, cell_data in enumerate(row_data):
                    item = QTableWidgetItem(str(cell_data))
                    self.table.setItem(row_idx, col_idx, item)
            
            self.table.resizeColumnsToContents()
            
            # Set height based on content
            height = self.table.horizontalHeader().height()
            height += self.table.verticalHeader().length()
            height += 20  # padding
            self.table.setFixedHeight(height)
        
        self.table.setAlternatingRowColors(True)
        layout.addWidget(self.table)
    
    def get_content(self) -> Dict[str, Any]:
        """Extract current table data"""
        data = []
        for row in range(self.table.rowCount()):
            row_data = []
            for col in range(self.table.columnCount()):
                item = self.table.item(row, col)
                row_data.append(item.text() if item else "")
            data.append(row_data)
        
        columns = []
        for col in range(self.table.columnCount()):
            columns.append(self.table.horizontalHeaderItem(col).text() if self.table.horizontalHeaderItem(col) else f"Col{col}")
        
        return {'data': data, 'columns': columns}


class DoclingFaithfulThread(QThread):
    """Extract document preserving structure"""
    finished = pyqtSignal(list)  # List of chunks
    error = pyqtSignal(str)
    
    def __init__(self, pdf_path):
        super().__init__()
        self.pdf_path = pdf_path
    
    def run(self):
        try:
            if not DOCLING_AVAILABLE:
                self.error.emit("Docling not installed")
                return
            
            converter = DocumentConverter()
            result = converter.convert(self.pdf_path)
            
            chunks = []
            
            # Process document items in order
            for item, level in result.document.iterate_items():
                chunk = self.process_item(item, level)
                if chunk:
                    chunks.append(chunk)
            
            # If no elements, fall back to basic extraction
            if not chunks:
                # Try to get structured content
                text = result.document.export_to_markdown()
                if text:
                    # Simple paragraph splitting
                    for para in text.split('\n\n'):
                        para = para.strip()
                        if para:
                            # Detect headings
                            if para.startswith('# '):
                                chunks.append({
                                    'type': ChunkType.HEADING1,
                                    'content': para[2:]
                                })
                            elif para.startswith('## '):
                                chunks.append({
                                    'type': ChunkType.HEADING2,
                                    'content': para[3:]
                                })
                            elif para.startswith('### '):
                                chunks.append({
                                    'type': ChunkType.HEADING3,
                                    'content': para[4:]
                                })
                            else:
                                chunks.append({
                                    'type': ChunkType.PARAGRAPH,
                                    'content': para
                                })
                
                # Extract tables separately
                for table in result.document.tables:
                    if hasattr(table, 'export_to_dataframe'):
                        df = table.export_to_dataframe()
                        chunks.append({
                            'type': ChunkType.TABLE,
                            'content': {
                                'data': df.values.tolist(),
                                'columns': df.columns.tolist(),
                                'caption': getattr(table, 'caption', '')
                            }
                        })
            
            self.finished.emit(chunks)
            
        except Exception as e:
            self.error.emit(str(e))
    
    def process_item(self, item, level) -> Optional[Dict]:
        """Process a document item into a chunk"""
        try:
            item_type = type(item).__name__
            
            # Get page number if available
            page_num = None
            if hasattr(item, 'prov') and hasattr(item.prov, 'page'):
                page_num = item.prov.page
            
            if item_type == 'SectionHeaderItem':
                # Determine heading level based on text formatting or level
                if hasattr(item, 'text'):
                    # You could analyze font size or other attributes here
                    # For now, use level parameter
                    heading_level = min(level, 3)  # Cap at H3
                    return {
                        'type': getattr(ChunkType, f'HEADING{heading_level}', ChunkType.HEADING1),
                        'content': item.text,
                        'page': page_num
                    }
            
            elif item_type == 'TableItem':
                # Export table to dataframe for structured data
                if hasattr(item, 'export_to_dataframe'):
                    df = item.export_to_dataframe()
                    return {
                        'type': ChunkType.TABLE,
                        'content': {
                            'data': df.values.tolist(),
                            'columns': df.columns.tolist(),
                            'caption': getattr(item, 'text', '')
                        }
                    }
            
            elif item_type == 'TextItem':
                if hasattr(item, 'text'):
                    return {
                        'type': ChunkType.PARAGRAPH,
                        'content': item.text
                    }
            
            elif item_type == 'ListItem':
                if hasattr(item, 'text'):
                    return {
                        'type': ChunkType.LIST,
                        'content': item.text
                    }
            
            elif item_type == 'PictureItem':
                # Handle images if needed
                return {
                    'type': ChunkType.IMAGE,
                    'content': getattr(item, 'text', 'Image')
                }
            
            # Add more item types as needed
            
        except Exception as e:
            print(f"Error processing item: {e}")
        
        return None


class ChonkerQtFaithful(QMainWindow):
    def __init__(self):
        super().__init__()
        self.current_pdf_path = None
        self.init_ui()
        
    def init_ui(self):
        self.setWindowTitle("🐹 CHONKER Qt Faithful - Document Structure Editor")
        self.setGeometry(100, 100, 1400, 900)
        
        # Create central widget
        central_widget = QWidget()
        self.setCentralWidget(central_widget)
        layout = QVBoxLayout(central_widget)
        layout.setContentsMargins(0, 0, 0, 0)
        
        # Create toolbar
        self.create_toolbar()
        
        # Create splitter
        self.splitter = QSplitter(Qt.Orientation.Horizontal)
        layout.addWidget(self.splitter)
        
        # Left side - PDF viewer
        pdf_container = QWidget()
        pdf_layout = QVBoxLayout(pdf_container)
        pdf_layout.setContentsMargins(0, 0, 0, 0)
        
        # PDF controls
        pdf_controls = QHBoxLayout()
        self.prev_button = QPushButton("← Previous")
        self.next_button = QPushButton("Next →")
        self.page_label = QLabel("Page 1 of 1")
        self.zoom_combo = QComboBox()
        self.zoom_combo.addItems(["50%", "75%", "100%", "125%", "150%", "200%"])
        self.zoom_combo.setCurrentText("100%")
        
        pdf_controls.addWidget(self.prev_button)
        pdf_controls.addWidget(self.page_label)
        pdf_controls.addWidget(self.next_button)
        pdf_controls.addStretch()
        pdf_controls.addWidget(QLabel("Zoom:"))
        pdf_controls.addWidget(self.zoom_combo)
        
        pdf_layout.addLayout(pdf_controls)
        
        # PDF viewer
        self.pdf_view = QPdfView(pdf_container)
        self.pdf_document = QPdfDocument(self)
        self.pdf_view.setDocument(self.pdf_document)
        self.pdf_view.setPageMode(QPdfView.PageMode.SinglePage)
        pdf_layout.addWidget(self.pdf_view)
        
        # Right side - Chunk editor
        right_container = QWidget()
        right_layout = QVBoxLayout(right_container)
        right_layout.setContentsMargins(0, 0, 0, 0)
        
        # Chunk controls
        chunk_controls = QHBoxLayout()
        chunk_controls.addWidget(QLabel("📄 Document Chunks"))
        chunk_controls.addStretch()
        add_text_btn = QPushButton("+ Text")
        add_table_btn = QPushButton("+ Table")
        add_heading_btn = QPushButton("+ Heading")
        chunk_controls.addWidget(add_text_btn)
        chunk_controls.addWidget(add_table_btn)
        chunk_controls.addWidget(add_heading_btn)
        
        right_layout.addLayout(chunk_controls)
        
        # Single text editor for all content
        self.content_editor = QTextEdit()
        self.content_editor.setAcceptRichText(True)
        # Ensure smooth scrolling
        self.content_editor.setVerticalScrollBarPolicy(Qt.ScrollBarPolicy.ScrollBarAsNeeded)
        self.content_editor.setHorizontalScrollBarPolicy(Qt.ScrollBarPolicy.ScrollBarAsNeeded)
        right_layout.addWidget(self.content_editor)
        
        # Add to splitter
        self.splitter.addWidget(pdf_container)
        self.splitter.addWidget(right_container)
        self.splitter.setSizes([700, 700])
        
        # Status bar
        self.status_bar = QStatusBar()
        self.setStatusBar(self.status_bar)
        
        # Connect signals
        self.prev_button.clicked.connect(self.prev_page)
        self.next_button.clicked.connect(self.next_page)
        self.zoom_combo.currentTextChanged.connect(self.zoom_changed)
    
    def create_toolbar(self):
        toolbar = self.addToolBar("Main")
        toolbar.setMovable(False)
        
        # Open action
        open_action = QAction("📂 Open PDF", self)
        open_action.setShortcut(QKeySequence("Ctrl+O"))
        open_action.triggered.connect(self.open_pdf)
        toolbar.addAction(open_action)
        
        # Extract action
        extract_action = QAction("📄 Extract Faithful", self)
        extract_action.setShortcut(QKeySequence("Ctrl+P"))
        extract_action.triggered.connect(self.extract_faithful)
        toolbar.addAction(extract_action)
        
        toolbar.addSeparator()
        
        # Save action
        save_action = QAction("💾 Save Document", self)
        save_action.triggered.connect(self.save_document)
        toolbar.addAction(save_action)
        
        # Export action
        export_action = QAction("📤 Export", self)
        export_action.triggered.connect(self.export_document)
        toolbar.addAction(export_action)
    
    def open_pdf(self):
        file_path, _ = QFileDialog.getOpenFileName(
            self, "Open PDF", "", "PDF Files (*.pdf)"
        )
        
        if file_path:
            self.current_pdf_path = file_path
            self.pdf_document.load(file_path)
            self.pdf_view.setZoomFactor(1.0)
            self.update_page_info()
            self.status_bar.showMessage(f"Opened: {os.path.basename(file_path)}", 3000)
    
    def extract_faithful(self):
        if not self.current_pdf_path:
            QMessageBox.warning(self, "No PDF", "Please open a PDF first")
            return
        
        self.status_bar.showMessage("Extracting document structure...")
        self.docling_thread = DoclingFaithfulThread(self.current_pdf_path)
        self.docling_thread.finished.connect(self.on_extraction_finished)
        self.docling_thread.error.connect(self.on_extraction_error)
        self.docling_thread.start()
    
    def on_extraction_finished(self, chunks):
        # Build HTML from chunks
        html = "<html><body style='font-family: -apple-system, BlinkMacSystemFont, \"Segoe UI\", Roboto, sans-serif; line-height: 1.6; max-width: 800px; margin: 0 auto; padding: 20px;'>"
        
        for chunk_data in chunks:
            chunk_type = chunk_data['type']
            content = chunk_data['content']
            
            if chunk_type == ChunkType.TABLE:
                # Build HTML table
                html += "<table style='border-collapse: collapse; width: 100%; margin: 20px 0;'>"
                if 'columns' in content:
                    html += "<thead><tr>"
                    for col in content['columns']:
                        html += f"<th style='border: 1px solid #ddd; padding: 8px; background-color: #f5f5f5;'>{col}</th>"
                    html += "</tr></thead>"
                html += "<tbody>"
                for row in content.get('data', []):
                    html += "<tr>"
                    for cell in row:
                        html += f"<td style='border: 1px solid #ddd; padding: 8px;'>{cell}</td>"
                    html += "</tr>"
                html += "</tbody></table>"
            elif chunk_type == ChunkType.HEADING1:
                html += f"<h1>{content}</h1>"
            elif chunk_type == ChunkType.HEADING2:
                html += f"<h2>{content}</h2>"
            elif chunk_type == ChunkType.HEADING3:
                html += f"<h3>{content}</h3>"
            elif chunk_type == ChunkType.CODE:
                html += f"<pre style='background: #f5f5f5; padding: 10px; overflow-x: auto;'><code>{content}</code></pre>"
            elif chunk_type == ChunkType.QUOTE:
                html += f"<blockquote style='border-left: 4px solid #ddd; padding-left: 20px; margin: 20px 0; color: #666;'>{content}</blockquote>"
            else:
                html += f"<p>{content}</p>"
        
        html += "</body></html>"
        
        self.content_editor.setHtml(html)
        self.status_bar.showMessage(f"Extracted {len(chunks)} chunks", 3000)
    
    def on_extraction_error(self, error):
        QMessageBox.critical(self, "Extraction Error", error)
    
    
    def save_document(self):
        """Save the document"""
        file_path, _ = QFileDialog.getSaveFileName(
            self, "Save Document", "", "HTML Files (*.html);;Text Files (*.txt)"
        )
        
        if file_path:
            with open(file_path, 'w', encoding='utf-8') as f:
                if file_path.endswith('.html'):
                    f.write(self.content_editor.toHtml())
                else:
                    f.write(self.content_editor.toPlainText())
            self.status_bar.showMessage(f"Saved: {os.path.basename(file_path)}", 3000)
    
    def export_document(self):
        """Export as HTML"""
        file_path, _ = QFileDialog.getSaveFileName(
            self, "Export HTML", "", "HTML Files (*.html)"
        )
        
        if file_path:
            html = self.content_editor.toHtml()
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(html)
            self.status_bar.showMessage(f"Exported: {os.path.basename(file_path)}", 3000)
    
    def prev_page(self):
        nav = self.pdf_view.pageNavigator()
        if nav.currentPage() > 0:
            nav.jump(nav.currentPage() - 1, QPointF())
            self.update_page_info()
    
    def next_page(self):
        nav = self.pdf_view.pageNavigator()
        if nav.currentPage() < self.pdf_document.pageCount() - 1:
            nav.jump(nav.currentPage() + 1, QPointF())
            self.update_page_info()
    
    def zoom_changed(self, text):
        try:
            zoom = float(text.rstrip('%')) / 100
            self.pdf_view.setZoomFactor(zoom)
        except ValueError:
            pass
    
    def update_page_info(self):
        if self.pdf_document.pageCount() > 0:
            current = self.pdf_view.pageNavigator().currentPage() + 1
            total = self.pdf_document.pageCount()
            self.page_label.setText(f"Page {current} of {total}")


def main():
    app = QApplication(sys.argv)
    app.setApplicationName("CHONKER Qt Faithful")
    app.setStyle('Fusion')
    
    window = ChonkerQtFaithful()
    window.show()
    
    sys.exit(app.exec())


if __name__ == "__main__":
    main()