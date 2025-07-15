#!/usr/bin/env python3
"""
🐹 CHONKER Qt - Native PDF viewer and text editor using PyQt
No web browsers, no scroll conflicts, just native widgets!
"""

import sys
import os
from pathlib import Path
from typing import Optional

from PyQt6.QtWidgets import (
    QApplication, QMainWindow, QSplitter, QTextEdit, QVBoxLayout, 
    QWidget, QPushButton, QHBoxLayout, QFileDialog, QMessageBox,
    QToolBar, QStatusBar, QLabel, QComboBox, QScrollArea,
    QTableWidget, QTableWidgetItem, QTabWidget
)
from PyQt6.QtCore import Qt, QUrl, pyqtSignal, QThread
from PyQt6.QtGui import QAction, QKeySequence, QTextDocument, QTextCursor
from PyQt6.QtPdf import QPdfDocument
from PyQt6.QtPdfWidgets import QPdfView

try:
    import fitz  # PyMuPDF for PDF optimization
    PYMUPDF_AVAILABLE = True
except ImportError:
    PYMUPDF_AVAILABLE = False

try:
    from docling.document_converter import DocumentConverter
    DOCLING_AVAILABLE = True
except ImportError:
    DOCLING_AVAILABLE = False


class DoclingThread(QThread):
    """Background thread for Docling processing"""
    finished = pyqtSignal(str, str, list)  # html, text, tables
    error = pyqtSignal(str)
    
    def __init__(self, pdf_path):
        super().__init__()
        self.pdf_path = pdf_path
    
    def run(self):
        try:
            if not DOCLING_AVAILABLE:
                self.error.emit("Docling not installed. Please run: pip install docling")
                return
            
            converter = DocumentConverter()
            result = converter.convert(self.pdf_path)
            
            text = result.document.export_to_markdown()
            html = result.document.export_to_html()
            
            # Extract tables as structured data
            tables = []
            for table in result.document.tables:
                if hasattr(table, 'export_to_dataframe'):
                    df = table.export_to_dataframe()
                    tables.append({
                        'data': df.values.tolist(),
                        'columns': df.columns.tolist(),
                        'caption': getattr(table, 'caption', '')
                    })
            
            # Clean up HTML
            if html:
                from bs4 import BeautifulSoup
                soup = BeautifulSoup(html, 'html.parser')
                for tag in soup.find_all({'html', 'head', 'body', 'meta', 'title', 'style', 'script'}):
                    tag.decompose() if tag.name in {'style', 'script'} else tag.unwrap()
                html = str(soup).strip()
            
            self.finished.emit(html or text, text, tables)
        except Exception as e:
            self.error.emit(str(e))


class ChonkerQt(QMainWindow):
    def __init__(self):
        super().__init__()
        self.current_pdf_path = None
        self.init_ui()
        
    def init_ui(self):
        self.setWindowTitle("🐹 CHONKER Qt - Document Editor")
        self.setGeometry(100, 100, 1400, 900)
        
        # Create central widget and layout
        central_widget = QWidget()
        self.setCentralWidget(central_widget)
        layout = QVBoxLayout(central_widget)
        layout.setContentsMargins(0, 0, 0, 0)
        
        # Create toolbar
        self.create_toolbar()
        
        # Create splitter for PDF and text
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
        self.zoom_combo.addItems(["50%", "75%", "100%", "125%", "150%", "200%", "300%"])
        self.zoom_combo.setCurrentText("100%")
        self.zoom_combo.setEditable(True)
        
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
        
        # Right side - Tabbed interface for text and tables
        self.tabs = QTabWidget()
        
        # Text editor tab
        self.text_edit = QTextEdit()
        self.text_edit.setAcceptRichText(True)
        
        # Set default stylesheet for better table visibility
        self.text_edit.setStyleSheet("""
            QTextEdit {
                background-color: #f8f9fa;
                color: #212529;
                font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                font-size: 14px;
                line-height: 1.6;
                padding: 20px;
            }
        """)
        
        self.tabs.addTab(self.text_edit, "📝 Text")
        
        # Tables tab (will be populated when tables are found)
        self.tables_widget = QWidget()
        self.tables_layout = QVBoxLayout(self.tables_widget)
        self.tabs.addTab(self.tables_widget, "📊 Tables")
        
        # Add to splitter
        self.splitter.addWidget(pdf_container)
        self.splitter.addWidget(self.tabs)
        self.splitter.setSizes([700, 700])
        
        # Status bar
        self.status_bar = QStatusBar()
        self.setStatusBar(self.status_bar)
        
        # Connect signals
        self.prev_button.clicked.connect(self.prev_page)
        self.next_button.clicked.connect(self.next_page)
        self.zoom_combo.currentTextChanged.connect(self.zoom_changed)
        self.pdf_document.pageCountChanged.connect(self.update_page_info)
        
        # Keyboard shortcuts
        self.setup_shortcuts()
        
    def create_toolbar(self):
        toolbar = self.addToolBar("Main")
        toolbar.setMovable(False)
        
        # Open action
        open_action = QAction("📂 Open PDF", self)
        open_action.triggered.connect(self.open_pdf)
        toolbar.addAction(open_action)
        
        # Extract text action
        extract_action = QAction("📄 Extract Text", self)
        extract_action.triggered.connect(self.extract_text)
        toolbar.addAction(extract_action)
        
        toolbar.addSeparator()
        
        # Save action
        save_action = QAction("💾 Save Text", self)
        save_action.triggered.connect(self.save_text)
        toolbar.addAction(save_action)
        
        # Export HTML action
        export_action = QAction("📤 Export HTML", self)
        export_action.triggered.connect(self.export_html)
        toolbar.addAction(export_action)
        
        toolbar.addSeparator()
        
        # Table actions
        table_action = QAction("📊 Insert Table", self)
        table_action.triggered.connect(self.insert_table)
        toolbar.addAction(table_action)
        
        format_action = QAction("🎨 Format", self)
        format_action.triggered.connect(self.show_format_menu)
        toolbar.addAction(format_action)
        
        if PYMUPDF_AVAILABLE:
            toolbar.addSeparator()
            # PDF optimization actions
            optimize_action = QAction("⚡ Optimize PDF", self)
            optimize_action.triggered.connect(self.optimize_pdf)
            toolbar.addAction(optimize_action)
            
            ocr_action = QAction("🔍 OCR PDF", self)
            ocr_action.triggered.connect(self.ocr_pdf)
            toolbar.addAction(ocr_action)
    
    def setup_shortcuts(self):
        # Ctrl+O to open
        open_shortcut = QAction(self)
        open_shortcut.setShortcut(QKeySequence.StandardKey.Open)
        open_shortcut.triggered.connect(self.open_pdf)
        self.addAction(open_shortcut)
        
        # Ctrl+S to save
        save_shortcut = QAction(self)
        save_shortcut.setShortcut(QKeySequence.StandardKey.Save)
        save_shortcut.triggered.connect(self.save_text)
        self.addAction(save_shortcut)
    
    def open_pdf(self):
        file_path, _ = QFileDialog.getOpenFileName(
            self, "Open PDF", "", "PDF Files (*.pdf)"
        )
        
        if file_path:
            self.current_pdf_path = file_path
            self.pdf_document.load(file_path)
            self.pdf_view.setZoomFactor(1.0)
            self.zoom_combo.setCurrentText("100%")
            self.update_page_info()
            self.status_bar.showMessage(f"Opened: {os.path.basename(file_path)}", 3000)
    
    def extract_text(self):
        if not self.current_pdf_path:
            QMessageBox.warning(self, "No PDF", "Please open a PDF first")
            return
        
        if DOCLING_AVAILABLE:
            # Use Docling in background thread
            self.status_bar.showMessage("Extracting text with Docling...")
            self.docling_thread = DoclingThread(self.current_pdf_path)
            self.docling_thread.finished.connect(self.on_docling_finished)
            self.docling_thread.error.connect(self.on_docling_error)
            self.docling_thread.start()
        elif PYMUPDF_AVAILABLE:
            # Fallback to PyMuPDF
            self.extract_with_pymupdf()
        else:
            QMessageBox.warning(
                self, "No Extraction Library",
                "Please install docling or pymupdf:\npip install docling pymupdf"
            )
    
    def on_docling_finished(self, html, text, tables):
        # Apply custom styling to the extracted HTML
        styled_html = self.style_extracted_html(html)
        self.text_edit.setHtml(styled_html)
        
        # Clear existing tables
        for i in reversed(range(self.tables_layout.count())): 
            self.tables_layout.itemAt(i).widget().setParent(None)
        
        # Add extracted tables to the tables tab
        if tables:
            for i, table_data in enumerate(tables):
                # Create label for table
                label = QLabel(f"Table {i+1}: {table_data.get('caption', 'Untitled')}")
                label.setStyleSheet("font-weight: bold; padding: 10px;")
                self.tables_layout.addWidget(label)
                
                # Create QTableWidget
                table_widget = QTableWidget()
                data = table_data['data']
                columns = table_data.get('columns', [])
                
                if data:
                    # Set dimensions
                    table_widget.setRowCount(len(data))
                    table_widget.setColumnCount(len(data[0]) if data else 0)
                    
                    # Set headers if available
                    if columns:
                        table_widget.setHorizontalHeaderLabels([str(col) for col in columns])
                    
                    # Populate data
                    for row_idx, row_data in enumerate(data):
                        for col_idx, cell_data in enumerate(row_data):
                            item = QTableWidgetItem(str(cell_data))
                            table_widget.setItem(row_idx, col_idx, item)
                    
                    # Style the table
                    table_widget.setAlternatingRowColors(True)
                    table_widget.setStyleSheet("""
                        QTableWidget {
                            gridline-color: #dee2e6;
                            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                        }
                        QHeaderView::section {
                            background-color: #f8f9fa;
                            padding: 8px;
                            border: 1px solid #dee2e6;
                            font-weight: bold;
                        }
                    """)
                    
                    # Make columns resize to content
                    table_widget.resizeColumnsToContents()
                    
                self.tables_layout.addWidget(table_widget)
            
            self.status_bar.showMessage(f"Text extracted with {len(tables)} tables!", 3000)
            # Switch to tables tab if tables were found
            self.tabs.setCurrentIndex(1)
        else:
            self.status_bar.showMessage("Text extracted successfully!", 3000)
    
    def style_extracted_html(self, html):
        """Apply custom styling to extracted HTML, especially for tables"""
        # Wrap in a div with custom styles
        styled = f"""
        <style>
            body {{
                font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                color: #212529;
                line-height: 1.6;
            }}
            table {{
                border-collapse: collapse;
                width: 100%;
                margin: 20px 0;
                background-color: white;
                box-shadow: 0 1px 3px rgba(0,0,0,0.1);
            }}
            th, td {{
                border: 1px solid #dee2e6;
                padding: 12px;
                text-align: left;
            }}
            th {{
                background-color: #f8f9fa;
                font-weight: 600;
                color: #495057;
            }}
            tr:nth-child(even) {{
                background-color: #f8f9fa;
            }}
            tr:hover {{
                background-color: #e9ecef;
            }}
            h1, h2, h3, h4, h5, h6 {{
                margin-top: 24px;
                margin-bottom: 16px;
                font-weight: 600;
                color: #212529;
            }}
            p {{
                margin-bottom: 16px;
            }}
            blockquote {{
                border-left: 4px solid #0d6efd;
                padding-left: 16px;
                margin: 16px 0;
                color: #6c757d;
            }}
        </style>
        <div>{html}</div>
        """
        return styled
    
    def on_docling_error(self, error):
        self.status_bar.showMessage(f"Extraction error: {error}", 5000)
        # Fallback to PyMuPDF
        if PYMUPDF_AVAILABLE:
            self.extract_with_pymupdf()
    
    def extract_with_pymupdf(self):
        try:
            doc = fitz.open(self.current_pdf_path)
            text = ""
            for page in doc:
                text += page.get_text()
            doc.close()
            
            self.text_edit.setPlainText(text)
            self.status_bar.showMessage("Text extracted with PyMuPDF", 3000)
        except Exception as e:
            QMessageBox.critical(self, "Error", f"Failed to extract text: {e}")
    
    def save_text(self):
        file_path, _ = QFileDialog.getSaveFileName(
            self, "Save Text", "", "HTML Files (*.html);;Text Files (*.txt)"
        )
        
        if file_path:
            try:
                with open(file_path, 'w', encoding='utf-8') as f:
                    if file_path.endswith('.html'):
                        f.write(self.text_edit.toHtml())
                    else:
                        f.write(self.text_edit.toPlainText())
                self.status_bar.showMessage(f"Saved: {os.path.basename(file_path)}", 3000)
            except Exception as e:
                QMessageBox.critical(self, "Error", f"Failed to save: {e}")
    
    def export_html(self):
        file_path, _ = QFileDialog.getSaveFileName(
            self, "Export HTML", "", "HTML Files (*.html)"
        )
        
        if file_path:
            try:
                # Create a full HTML document
                html = f"""<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>{os.path.basename(self.current_pdf_path or 'Document')}</title>
    <style>
        body {{ font-family: Arial, sans-serif; max-width: 800px; margin: 40px auto; padding: 20px; }}
        table {{ border-collapse: collapse; width: 100%; }}
        th, td {{ border: 1px solid #ccc; padding: 8px; }}
    </style>
</head>
<body>
{self.text_edit.toHtml()}
</body>
</html>"""
                
                with open(file_path, 'w', encoding='utf-8') as f:
                    f.write(html)
                self.status_bar.showMessage(f"Exported: {os.path.basename(file_path)}", 3000)
            except Exception as e:
                QMessageBox.critical(self, "Error", f"Failed to export: {e}")
    
    def optimize_pdf(self):
        if not self.current_pdf_path or not PYMUPDF_AVAILABLE:
            return
        
        file_path, _ = QFileDialog.getSaveFileName(
            self, "Save Optimized PDF", "", "PDF Files (*.pdf)"
        )
        
        if file_path:
            try:
                doc = fitz.open(self.current_pdf_path)
                doc.save(file_path, garbage=4, deflate=True, clean=True)
                doc.close()
                self.status_bar.showMessage("PDF optimized and saved!", 3000)
            except Exception as e:
                QMessageBox.critical(self, "Error", f"Failed to optimize: {e}")
    
    def ocr_pdf(self):
        QMessageBox.information(
            self, "OCR", 
            "OCR functionality requires additional setup.\n"
            "Install: pip install ocrmypdf"
        )
    
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
    
    def insert_table(self):
        """Insert a table at the current cursor position"""
        cursor = self.text_edit.textCursor()
        
        # Create a simple 3x3 table
        table_html = """
        <table>
            <tr>
                <th>Header 1</th>
                <th>Header 2</th>
                <th>Header 3</th>
            </tr>
            <tr>
                <td>Cell 1</td>
                <td>Cell 2</td>
                <td>Cell 3</td>
            </tr>
            <tr>
                <td>Cell 4</td>
                <td>Cell 5</td>
                <td>Cell 6</td>
            </tr>
        </table>
        """
        
        cursor.insertHtml(table_html)
        self.status_bar.showMessage("Table inserted", 2000)
    
    def show_format_menu(self):
        """Show formatting options"""
        from PyQt6.QtWidgets import QMenu, QToolButton
        
        menu = QMenu(self)
        
        # Text formatting actions
        bold_action = menu.addAction("Bold")
        bold_action.setShortcut("Ctrl+B")
        bold_action.triggered.connect(lambda: self.text_edit.setFontWeight(700))
        
        italic_action = menu.addAction("Italic")
        italic_action.setShortcut("Ctrl+I")
        italic_action.triggered.connect(lambda: self.text_edit.setFontItalic(True))
        
        menu.addSeparator()
        
        # Heading actions
        h1_action = menu.addAction("Heading 1")
        h1_action.triggered.connect(lambda: self.apply_heading(1))
        
        h2_action = menu.addAction("Heading 2")
        h2_action.triggered.connect(lambda: self.apply_heading(2))
        
        h3_action = menu.addAction("Heading 3")
        h3_action.triggered.connect(lambda: self.apply_heading(3))
        
        menu.addSeparator()
        
        # List actions
        bullet_action = menu.addAction("• Bullet List")
        bullet_action.triggered.connect(self.insert_bullet_list)
        
        number_action = menu.addAction("1. Numbered List")
        number_action.triggered.connect(self.insert_numbered_list)
        
        # Show menu at cursor position
        menu.exec(self.cursor().pos())
    
    def apply_heading(self, level):
        """Apply heading format to current line"""
        cursor = self.text_edit.textCursor()
        cursor.select(QTextCursor.SelectionType.LineUnderCursor)
        
        font_sizes = {1: 24, 2: 20, 3: 16}
        cursor.insertHtml(f'<h{level}>{cursor.selectedText()}</h{level}>')
    
    def insert_bullet_list(self):
        """Insert a bullet list"""
        cursor = self.text_edit.textCursor()
        cursor.insertHtml("""
        <ul>
            <li>Item 1</li>
            <li>Item 2</li>
            <li>Item 3</li>
        </ul>
        """)
    
    def insert_numbered_list(self):
        """Insert a numbered list"""
        cursor = self.text_edit.textCursor()
        cursor.insertHtml("""
        <ol>
            <li>First item</li>
            <li>Second item</li>
            <li>Third item</li>
        </ol>
        """)


def main():
    app = QApplication(sys.argv)
    app.setApplicationName("CHONKER Qt")
    app.setOrganizationName("CHONKER")
    
    # Set application style
    app.setStyle('Fusion')
    
    window = ChonkerQt()
    window.show()
    
    # Open file from command line if provided
    if len(sys.argv) > 1:
        window.current_pdf_path = sys.argv[1]
        window.pdf_document.load(sys.argv[1])
        window.update_page_info()
    
    sys.exit(app.exec())


if __name__ == "__main__":
    main()