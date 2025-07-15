#!/Users/jack/chonksnyft-env/bin/python
"""
🐹 CHONKER Qt Mixed - Full mixed content rendering with WebEngine
"""

import sys
import os
import json
from pathlib import Path
from typing import Optional, List, Dict, Any

from PyQt6.QtWidgets import (
    QApplication, QMainWindow, QSplitter, QVBoxLayout, 
    QWidget, QPushButton, QHBoxLayout, QFileDialog, QMessageBox,
    QToolBar, QStatusBar, QLabel, QComboBox
)
from PyQt6.QtCore import Qt, QUrl, pyqtSignal, QThread, QPointF
from PyQt6.QtGui import QAction, QKeySequence
from PyQt6.QtPdf import QPdfDocument
from PyQt6.QtPdfWidgets import QPdfView
from PyQt6.QtWebEngineWidgets import QWebEngineView

try:
    from docling.document_converter import DocumentConverter
    DOCLING_AVAILABLE = True
except ImportError:
    DOCLING_AVAILABLE = False


class DocumentBridge(QThread):
    """Bridge between Python and JavaScript"""
    contentReady = pyqtSignal(str)  # Sends HTML to JS
    
    def __init__(self):
        super().__init__()
        self.content = ""
    
    def setContent(self, html_content):
        """Called from Python to update content"""
        self.content = html_content
        self.contentReady.emit(html_content)
    
    def getContent(self):
        """Called from JS to get current content"""
        return self.content
    
    def saveTable(self, table_id, table_data):
        """Called from JS when table is edited"""
        print(f"Table {table_id} updated:", table_data)


class DoclingExtractionThread(QThread):
    """Extract PDF content using Docling"""
    finished = pyqtSignal(str)  # HTML content
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
            
            # Build rich HTML with proper structure
            html_parts = []
            html_parts.append("""
                <div id="document-content" contenteditable="true">
            """)
            
            table_id = 0
            
            # Process document items
            for item, level in result.document.iterate_items():
                item_type = type(item).__name__
                
                if item_type == 'SectionHeaderItem' and hasattr(item, 'text'):
                    heading_level = min(level, 3)
                    html_parts.append(f'<h{heading_level}>{item.text}</h{heading_level}>')
                    
                elif item_type == 'TableItem':
                    if hasattr(item, 'export_to_dataframe'):
                        df = item.export_to_dataframe()
                        table_id += 1
                        
                        # Create container for Handsontable with data attribute
                        table_data = {
                            'data': df.values.tolist(),
                            'columns': df.columns.tolist()
                        }
                        html_parts.append(f'''
                            <div class="table-container">
                                <div id="table-{table_id}" 
                                     class="handsontable-container" 
                                     data-table='{json.dumps(table_data)}'>
                                </div>
                            </div>
                        ''')
                        
                elif item_type == 'TextItem' and hasattr(item, 'text'):
                    html_parts.append(f'<p>{item.text}</p>')
                    
                elif item_type == 'ListItem' and hasattr(item, 'text'):
                    html_parts.append(f'<li>{item.text}</li>')
            
            html_parts.append('</div>')
            
            # If no items found, try markdown export
            if len(html_parts) <= 2:
                text = result.document.export_to_markdown()
                if text:
                    html_parts = ['<div id="document-content" contenteditable="true">']
                    for para in text.split('\n\n'):
                        para = para.strip()
                        if para:
                            if para.startswith('# '):
                                html_parts.append(f'<h1>{para[2:]}</h1>')
                            elif para.startswith('## '):
                                html_parts.append(f'<h2>{para[3:]}</h2>')
                            elif para.startswith('### '):
                                html_parts.append(f'<h3>{para[4:]}</h3>')
                            else:
                                html_parts.append(f'<p>{para}</p>')
                    html_parts.append('</div>')
            
            self.finished.emit('\n'.join(html_parts))
            
        except Exception as e:
            self.error.emit(str(e))


class ChonkerQtMixed(QMainWindow):
    def __init__(self):
        super().__init__()
        self.current_pdf_path = None
        self.bridge = DocumentBridge()
        self.init_ui()
        
    def init_ui(self):
        self.setWindowTitle("🐹 CHONKER Qt Mixed - Advanced Document Editor")
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
        
        # Right side - Web-based mixed content editor
        self.web_view = QWebEngineView()
        self.setup_web_content()
        
        # Add to splitter
        self.splitter.addWidget(pdf_container)
        self.splitter.addWidget(self.web_view)
        self.splitter.setSizes([700, 700])
        
        # Status bar
        self.status_bar = QStatusBar()
        self.setStatusBar(self.status_bar)
        
        # Connect signals
        self.prev_button.clicked.connect(self.prev_page)
        self.next_button.clicked.connect(self.next_page)
        self.zoom_combo.currentTextChanged.connect(self.zoom_changed)
    
    def setup_web_content(self):
        """Set up the web view with mixed content editing capabilities"""
        html = """
        <!DOCTYPE html>
        <html>
        <head>
            <meta charset="UTF-8">
            <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/handsontable/dist/handsontable.full.min.css">
            <link href="https://cdn.quilljs.com/1.3.6/quill.snow.css" rel="stylesheet">
            <style>
                body {
                    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                    max-width: 900px;
                    margin: 0 auto;
                    padding: 20px;
                    line-height: 1.6;
                }
                h1, h2, h3 { 
                    color: #333; 
                    margin: 20px 0 10px 0;
                }
                p { 
                    margin: 10px 0;
                }
                .table-container {
                    margin: 20px 0;
                    border: 1px solid #ddd;
                    border-radius: 4px;
                    padding: 10px;
                    background: #f9f9f9;
                }
                .handsontable-container {
                    overflow: hidden;
                }
                #document-content {
                    min-height: 500px;
                    outline: none;
                }
                #document-content:focus {
                    outline: none;
                }
                .status {
                    position: fixed;
                    bottom: 10px;
                    right: 10px;
                    background: #28a745;
                    color: white;
                    padding: 5px 15px;
                    border-radius: 4px;
                    display: none;
                }
                .toolbar {
                    position: sticky;
                    top: 0;
                    background: white;
                    border-bottom: 1px solid #ddd;
                    padding: 10px;
                    margin: -20px -20px 20px -20px;
                    z-index: 100;
                }
            </style>
        </head>
        <body>
            <div class="toolbar">
                <button onclick="formatText('bold')">Bold</button>
                <button onclick="formatText('italic')">Italic</button>
                <button onclick="formatText('underline')">Underline</button>
                <select onchange="formatBlock(this.value)">
                    <option value="">Normal</option>
                    <option value="h1">Heading 1</option>
                    <option value="h2">Heading 2</option>
                    <option value="h3">Heading 3</option>
                </select>
                <button onclick="insertTable()">Insert Table</button>
                <button onclick="saveDocument()">Save</button>
            </div>
            
            <div id="document-content" contenteditable="true">
                <h1>🐹 CHONKER Mixed Content Editor</h1>
                <p>Open a PDF to begin extraction with full mixed content support.</p>
            </div>
            
            <div class="status" id="status">Saved!</div>
            
            <script src="https://cdn.jsdelivr.net/npm/handsontable/dist/handsontable.full.min.js"></script>
            <script>
                let bridge = null;
                let tables = {};
                
                // Simpler approach - expose functions to Python
                window.updateContent = function(html) {
                    document.getElementById('document-content').innerHTML = html;
                    // Small delay to ensure DOM is ready
                    setTimeout(initializeTables, 100);
                }
                
                window.getContent = function() {
                    return document.getElementById('document-content').innerHTML;
                }
                
                function initializeTables() {
                    // Find all table containers and initialize Handsontable
                    const containers = document.querySelectorAll('.handsontable-container');
                    containers.forEach(container => {
                        const tableId = container.id;
                        if (!tables[tableId] && container.dataset.table) {
                            try {
                                const tableData = JSON.parse(container.dataset.table);
                                tables[tableId] = new Handsontable(container, {
                                    data: tableData.data,
                                    colHeaders: tableData.columns || true,
                                    rowHeaders: true,
                                    height: 'auto',
                                    width: '100%',
                                    stretchH: 'all',
                                    contextMenu: true,
                                    manualColumnResize: true,
                                    manualRowResize: true,
                                    licenseKey: 'non-commercial-and-evaluation'
                                });
                            } catch (e) {
                                console.error('Error initializing table:', e);
                            }
                        }
                    });
                }
                
                function formatText(command) {
                    document.execCommand(command, false, null);
                }
                
                function formatBlock(tag) {
                    if (tag) {
                        document.execCommand('formatBlock', false, tag);
                    }
                }
                
                function insertTable() {
                    const tableId = 'table-' + Date.now();
                    const tableHtml = `
                        <div class="table-container">
                            <div id="${tableId}" class="handsontable-container"></div>
                        </div>
                    `;
                    document.execCommand('insertHTML', false, tableHtml);
                    
                    // Initialize the new table
                    setTimeout(() => {
                        const container = document.getElementById(tableId);
                        tables[tableId] = new Handsontable(container, {
                            data: [['', '', ''], ['', '', ''], ['', '', '']],
                            colHeaders: ['Column 1', 'Column 2', 'Column 3'],
                            rowHeaders: true,
                            height: 'auto',
                            width: '100%',
                            stretchH: 'all',
                            contextMenu: true,
                            licenseKey: 'non-commercial-and-evaluation'
                        });
                    }, 100);
                }
                
                function saveDocument() {
                    const content = document.getElementById('document-content').innerHTML;
                    // In real implementation, send to Python
                    document.getElementById('status').style.display = 'block';
                    setTimeout(() => {
                        document.getElementById('status').style.display = 'none';
                    }, 2000);
                }
                
                // Handle paste to clean up formatting
                document.getElementById('document-content').addEventListener('paste', function(e) {
                    e.preventDefault();
                    const text = e.clipboardData.getData('text/plain');
                    document.execCommand('insertText', false, text);
                });
            </script>
        </body>
        </html>
        """
        
        self.web_view.setHtml(html)
    
    def create_toolbar(self):
        toolbar = self.addToolBar("Main")
        toolbar.setMovable(False)
        
        # Open action
        open_action = QAction("📂 Open PDF", self)
        open_action.setShortcut(QKeySequence("Ctrl+O"))
        open_action.triggered.connect(self.open_pdf)
        toolbar.addAction(open_action)
        
        # Extract action
        extract_action = QAction("📄 Extract Mixed", self)
        extract_action.setShortcut(QKeySequence("Ctrl+P"))
        extract_action.triggered.connect(self.extract_mixed)
        toolbar.addAction(extract_action)
        
        toolbar.addSeparator()
        
        # Save action
        save_action = QAction("💾 Save Document", self)
        save_action.triggered.connect(self.save_document)
        toolbar.addAction(save_action)
        
        # Export action
        export_action = QAction("📤 Export HTML", self)
        export_action.triggered.connect(self.export_html)
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
    
    def extract_mixed(self):
        if not self.current_pdf_path:
            QMessageBox.warning(self, "No PDF", "Please open a PDF first")
            return
        
        self.status_bar.showMessage("Extracting with mixed content support...")
        self.extraction_thread = DoclingExtractionThread(self.current_pdf_path)
        self.extraction_thread.finished.connect(self.on_extraction_finished)
        self.extraction_thread.error.connect(self.on_extraction_error)
        self.extraction_thread.start()
    
    def on_extraction_finished(self, html_content):
        # Send content to web view using JavaScript
        escaped_html = html_content.replace('\\', '\\\\').replace("'", "\\'").replace('\n', '\\n')
        self.web_view.page().runJavaScript(f"updateContent('{escaped_html}')")
        self.status_bar.showMessage("Extraction complete!", 3000)
    
    def on_extraction_error(self, error):
        QMessageBox.critical(self, "Extraction Error", error)
    
    def save_document(self):
        # Execute JavaScript to get current content
        self.web_view.page().runJavaScript(
            "document.getElementById('document-content').innerHTML",
            lambda html: self._save_html(html)
        )
    
    def _save_html(self, html):
        file_path, _ = QFileDialog.getSaveFileName(
            self, "Save Document", "", "HTML Files (*.html)"
        )
        if file_path:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(f"""
                <!DOCTYPE html>
                <html>
                <head>
                    <meta charset="UTF-8">
                    <style>
                        body {{ 
                            font-family: Arial, sans-serif; 
                            max-width: 800px; 
                            margin: 40px auto;
                            padding: 20px;
                        }}
                        table {{ 
                            border-collapse: collapse; 
                            width: 100%; 
                            margin: 20px 0;
                        }}
                        th, td {{ 
                            border: 1px solid #ddd; 
                            padding: 8px; 
                        }}
                    </style>
                </head>
                <body>
                    {html}
                </body>
                </html>
                """)
            self.status_bar.showMessage(f"Saved: {os.path.basename(file_path)}", 3000)
    
    def export_html(self):
        self.save_document()
    
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
    app.setApplicationName("CHONKER Qt Mixed")
    app.setStyle('Fusion')
    
    window = ChonkerQtMixed()
    window.show()
    
    sys.exit(app.exec())


if __name__ == "__main__":
    main()