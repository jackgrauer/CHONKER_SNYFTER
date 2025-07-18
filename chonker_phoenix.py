#!/usr/bin/env python3
"""
🐹 CHONKER Phoenix - The Focused PDF-to-SQL Hamster
A svelte PDF chomper that extracts tables for human review and SQL export.

Workflow: PDF → Docling → Editable HTML → Human QA → SQL Export
Target: Under 1,800 lines of maintainable code
"""

import sys
import os
import re
from pathlib import Path
from typing import List, Dict, Optional, Tuple
from dataclasses import dataclass
from datetime import datetime
import html

# PyQt6 imports - minimal set
from PyQt6.QtWidgets import (
    QApplication, QMainWindow, QWidget, QVBoxLayout, QHBoxLayout,
    QPushButton, QFileDialog, QMessageBox, QTextEdit, QLabel,
    QSplitter, QMenuBar, QMenu, QStatusBar, QProgressBar,
    QDialog, QDialogButtonBox, QTableWidget, QComboBox
)
from PyQt6.QtCore import (
    Qt, QThread, pyqtSignal, QTimer, QObject, QEvent
)
from PyQt6.QtGui import (
    QAction, QKeySequence, QFont, QTextCursor, QIcon
)

# Security and performance limits - CRITICAL DEFENSES
MAX_FILE_SIZE = 500 * 1024 * 1024  # 500MB protection
MAX_PROCESSING_TIME = 300  # 5 minute timeout defense

# Optional imports with graceful fallbacks
try:
    from docling.document_converter import DocumentConverter
    DOCLING_AVAILABLE = True
except ImportError:
    DOCLING_AVAILABLE = False
    print("⚠️  Docling not available. Install with: pip install docling")
    
# Caffeinate defense
try:
    import subprocess
    CAFFEINATE_AVAILABLE = True
except ImportError:
    CAFFEINATE_AVAILABLE = False

# ============================================================================
# DATA MODELS - Pydantic Bot's Domain
# ============================================================================

@dataclass
class ExtractedTable:
    """A table extracted from PDF"""
    table_id: str
    headers: List[str]
    rows: List[List[str]]
    confidence: float = 1.0
    page_number: int = 0
    
@dataclass
class ProcessingResult:
    """Result of PDF processing"""
    success: bool
    tables: List[ExtractedTable]
    text_content: str
    html_content: str
    error_message: Optional[str] = None
    processing_time: float = 0.0

@dataclass
class SQLExportConfig:
    """Configuration for SQL export"""
    table_name: str
    column_types: Dict[str, str]  # column -> SQL type
    primary_key: Optional[str] = None
    skip_empty_rows: bool = True

# ============================================================================
# PDF PROCESSOR - Instructor Bot's Extracted Core
# ============================================================================

class ChonkerProcessor(QThread):
    """Lean PDF processor - just the essentials"""
    
    progress_update = pyqtSignal(int, str)
    processing_complete = pyqtSignal(ProcessingResult)
    
    def __init__(self):
        super().__init__()
        self.file_path = None
        self.converter = None
        self.should_stop = False
        self.start_time = None
        self.timeout_occurred = False
        
    def chomp_pdf(self, file_path: str):
        """Start chomping a PDF"""
        self.file_path = file_path
        self.start()
        
    def run(self):
        """Process PDF in background thread with full defensive armor"""
        self.start_time = datetime.now()
        
        try:
            # 🛡️ DEFENSE 1: Validate PDF file 
            if not self._validate_pdf_file():
                return
                
            # 🛡️ DEFENSE 2: Check file size
            file_size = os.path.getsize(self.file_path)
            if file_size > MAX_FILE_SIZE:
                raise ValueError(f"File too large: {file_size/(1024*1024):.1f}MB (max: {MAX_FILE_SIZE/(1024*1024):.0f}MB)")
                
            if not DOCLING_AVAILABLE:
                raise ImportError("Docling is required for PDF processing")
                
            self.progress_update.emit(10, "Initializing Docling...")
            
            # 🛡️ DEFENSE 3: Timeout check
            if self._check_timeout():
                return
                
            # Initialize converter if needed
            if not self.converter:
                self.converter = DocumentConverter()
                
            self.progress_update.emit(30, "Chomping PDF...")
            
            # 🛡️ DEFENSE 4: Timeout check before heavy processing
            if self._check_timeout():
                return
            
            # Process with Docling
            result = self.converter.convert(self.file_path)
            
            self.progress_update.emit(60, "Extracting tables...")
            
            # 🛡️ DEFENSE 5: Timeout check during extraction
            if self._check_timeout():
                return
            
            # Extract tables and content
            tables = self._extract_tables(result)
            text_content = self._extract_text(result)
            html_content = self._generate_html(tables, text_content)
            
            self.progress_update.emit(90, "Preparing for display...")
            
            processing_time = (datetime.now() - self.start_time).total_seconds()
            
            self.processing_complete.emit(ProcessingResult(
                success=True,
                tables=tables,
                text_content=text_content,
                html_content=html_content,
                processing_time=processing_time
            ))
            
        except Exception as e:
            self.processing_complete.emit(ProcessingResult(
                success=False,
                tables=[],
                text_content="",
                html_content="",
                error_message=str(e)
            ))
    
    def _validate_pdf_file(self) -> bool:
        """🛡️ Validate PDF file header"""
        try:
            with open(self.file_path, 'rb') as f:
                header = f.read(8)
                if not header.startswith(b'%PDF-'):
                    raise ValueError("Invalid PDF file format")
            return True
        except Exception as e:
            self.processing_complete.emit(ProcessingResult(
                success=False, tables=[], text_content="", html_content="",
                error_message=f"PDF validation failed: {str(e)}"
            ))
            return False
    
    def _check_timeout(self) -> bool:
        """🛡️ Check if processing has exceeded timeout"""
        if self.start_time and not self.timeout_occurred:
            elapsed = (datetime.now() - self.start_time).total_seconds()
            if elapsed > MAX_PROCESSING_TIME:
                self.timeout_occurred = True
                self.processing_complete.emit(ProcessingResult(
                    success=False, tables=[], text_content="", html_content="",
                    error_message=f"⏱️ Processing timeout exceeded ({MAX_PROCESSING_TIME}s)"
                ))
                return True
        return False
    
    def stop_chomping(self):
        """🛡️ Emergency stop"""
        self.should_stop = True
        self.timeout_occurred = True
            
    def _extract_tables(self, docling_result) -> List[ExtractedTable]:
        """Extract tables from Docling result"""
        tables = []
        page_num = 0
        
        for item in docling_result.document.iterate_items():
            if hasattr(item, 'page_no'):
                page_num = item.page_no
                
            if hasattr(item, 'table') and item.table:
                try:
                    # Convert to pandas DataFrame for easier handling
                    import pandas as pd
                    df = item.table.to_pandas()
                    
                    if not df.empty:
                        # Extract headers and rows
                        headers = df.columns.tolist()
                        rows = df.values.tolist()
                        
                        # Create ExtractedTable
                        table = ExtractedTable(
                            table_id=f"table_{page_num}_{len(tables)}",
                            headers=[str(h) for h in headers],
                            rows=[[str(cell) for cell in row] for row in rows],
                            confidence=0.95,  # Default high confidence
                            page_number=page_num
                        )
                        tables.append(table)
                        
                except Exception as e:
                    self.progress_update.emit(50, f"Table extraction warning: {str(e)}")
                    
        return tables
        
    def _extract_text(self, docling_result) -> str:
        """Extract plain text from Docling result"""
        text_parts = []
        
        for item in docling_result.document.iterate_items():
            if hasattr(item, 'text') and item.text:
                text_parts.append(item.text)
                
        return "\n\n".join(text_parts)
        
    def _generate_html(self, tables: List[ExtractedTable], text: str) -> str:
        """Generate editable HTML with tables"""
        
        # Generate table HTML
        tables_html = ""
        for table in tables:
            table_html = f"""
            <div class="table-container" data-table-id="{table.table_id}">
                <h3>Table from Page {table.page_number}</h3>
                <table id="{table.table_id}">
                    <thead>
                        <tr>
                            {"".join(f'<th contenteditable="true">{html.escape(h)}</th>' for h in table.headers)}
                            <th class="controls">Actions</th>
                        </tr>
                    </thead>
                    <tbody>
            """
            
            for row in table.rows:
                row_html = "<tr>"
                for cell in row:
                    row_html += f'<td contenteditable="true">{html.escape(str(cell))}</td>'
                row_html += '<td class="controls"><button class="delete-row">Delete</button></td>'
                row_html += "</tr>"
                table_html += row_html
                
            table_html += """
                    </tbody>
                </table>
                <div class="table-controls">
                    <button class="add-row">Add Row</button>
                    <button class="add-column">Add Column</button>
                </div>
            </div>
            """
            tables_html += table_html
        
        # Build complete HTML
        html_content = f"""
        <!DOCTYPE html>
        <html>
        <head>
            <meta charset="utf-8">
            <style>
                body {{
                    font-family: -apple-system, sans-serif;
                    margin: 20px;
                    color: #FFFFFF;
                    background: #2D2E30;
                    line-height: 1.6;
                }}
                table {{
                    border-collapse: collapse;
                    margin: 15px 0;
                    width: 100%;
                    background: #1E1E1E;
                }}
                th, td {{
                    border: 1px solid #525659;
                    padding: 8px;
                    text-align: left;
                }}
                th {{
                    background: #3A3C3E;
                    font-weight: bold;
                }}
                td[contenteditable="true"] {{
                    cursor: text;
                    background: #2D2E30;
                }}
                td[contenteditable="true"]:hover {{
                    background-color: #3A3C3E;
                }}
                td[contenteditable="true"]:focus {{
                    background-color: #525659;
                    outline: 2px solid #1ABC9C;
                }}
                .controls {{
                    width: 100px;
                    text-align: center;
                }}
                button {{
                    margin: 2px;
                    padding: 5px 10px;
                    background: #1ABC9C;
                    color: white;
                    border: none;
                    cursor: pointer;
                    border-radius: 3px;
                }}
                button:hover {{
                    background: #16A085;
                }}
                .delete-row {{
                    background: #E74C3C;
                }}
                .delete-row:hover {{
                    background: #C0392B;
                }}
                .table-container {{
                    margin: 30px 0;
                    padding: 20px;
                    background: #1E1E1E;
                    border-radius: 5px;
                }}
                h3 {{
                    color: #1ABC9C;
                    margin-top: 0;
                }}
                .summary {{
                    background: #3A3C3E;
                    padding: 15px;
                    border-radius: 5px;
                    margin-bottom: 20px;
                }}
            </style>
            <script>
                // Add row functionality
                document.addEventListener('click', function(e) {{
                    if (e.target.classList.contains('add-row')) {{
                        const table = e.target.closest('.table-container').querySelector('tbody');
                        const columnCount = table.querySelector('tr').querySelectorAll('td').length - 1;
                        const newRow = table.insertRow();
                        
                        for (let i = 0; i < columnCount; i++) {{
                            const cell = newRow.insertCell();
                            cell.contentEditable = true;
                            cell.textContent = '';
                        }}
                        
                        const controlCell = newRow.insertCell();
                        controlCell.className = 'controls';
                        controlCell.innerHTML = '<button class="delete-row">Delete</button>';
                    }}
                    
                    // Delete row functionality
                    if (e.target.classList.contains('delete-row')) {{
                        const row = e.target.closest('tr');
                        row.remove();
                    }}
                    
                    // Add column functionality
                    if (e.target.classList.contains('add-column')) {{
                        const table = e.target.closest('.table-container').querySelector('table');
                        const headerRow = table.querySelector('thead tr');
                        const newHeader = document.createElement('th');
                        newHeader.contentEditable = true;
                        newHeader.textContent = 'New Column';
                        headerRow.insertBefore(newHeader, headerRow.lastElementChild);
                        
                        const rows = table.querySelectorAll('tbody tr');
                        rows.forEach(row => {{
                            const newCell = row.insertCell(row.cells.length - 1);
                            newCell.contentEditable = true;
                            newCell.textContent = '';
                        }});
                    }}
                }});
                
                // Track edits
                document.addEventListener('input', function(e) {{
                    if (e.target.contentEditable === 'true') {{
                        e.target.dataset.edited = 'true';
                        e.target.style.backgroundColor = '#525659';
                    }}
                }});
            </script>
        </head>
        <body>
            <h1>🐹 CHONKER's Belly Contents</h1>
            
            <div class="summary">
                <h2>Extraction Summary</h2>
                <p>Found {len(tables)} table(s) in the PDF</p>
                <p>Text preview: {html.escape(text[:200])}...</p>
            </div>
            
            {tables_html}
            
            <div class="summary">
                <h3>Instructions</h3>
                <ul>
                    <li>Click any cell to edit its contents</li>
                    <li>Use "Add Row" to add new rows to a table</li>
                    <li>Use "Add Column" to add new columns</li>
                    <li>Click "Delete" to remove a row</li>
                    <li>Edited cells will be highlighted</li>
                    <li>When ready, export to SQL from the menu</li>
                </ul>
            </div>
        </body>
        </html>
        """
        return html_content

# ============================================================================
# HTML EDITOR - The X-Ray Viewer
# ============================================================================

class ChonkerBellyViewer(QTextEdit):
    """View and edit CHONKER's belly contents"""
    
    def __init__(self):
        super().__init__()
        self.setup_editor()
        self.current_zoom = 12
        
    def setup_editor(self):
        """Configure the HTML editor"""
        self.setReadOnly(False)
        self.setAcceptRichText(True)
        
        # Dark theme styling
        self.setStyleSheet("""
            QTextEdit {
                background-color: #2D2E30;
                color: #E0E0E0;
                border: 1px solid #3A3C3E;
                font-size: 12px;
            }
        """)
        
    def display_chomped_content(self, result: ProcessingResult):
        """Display the processed PDF content"""
        # Apply zoom to HTML before displaying
        html = self._apply_zoom_to_html(result.html_content)
        self.setHtml(html)
        self._last_result = result  # Store for re-rendering on zoom
        
    def _apply_zoom_to_html(self, html: str) -> str:
        """Apply current zoom level to HTML"""
        # Insert zoom CSS into existing HTML
        zoom_css = f"""
        <style>
            body {{ font-size: {self.current_zoom}px !important; }}
            p {{ font-size: {self.current_zoom}px !important; }}
            td, th {{ font-size: {self.current_zoom}px !important; }}
        </style>
        """
        
        # Find insertion point
        if '</head>' in html:
            html = html.replace('</head>', f'{zoom_css}</head>')
        elif '<body>' in html:
            html = html.replace('<body>', f'{zoom_css}<body>')
        else:
            html = zoom_css + html
            
        return html
        
    def zoom_in(self):
        """Increase zoom"""
        self.current_zoom = min(48, self.current_zoom + 2)
        if hasattr(self, '_last_result'):
            self.display_chomped_content(self._last_result)
            
    def zoom_out(self):
        """Decrease zoom"""
        self.current_zoom = max(8, self.current_zoom - 2)
        if hasattr(self, '_last_result'):
            self.display_chomped_content(self._last_result)

# ============================================================================
# SQL EXPORTER - The SQL Pooper
# ============================================================================

class SQLExporter:
    """Convert edited HTML tables to SQL"""
    
    @staticmethod
    def parse_html_tables(html_content: str) -> List[Dict]:
        """Extract table data from edited HTML"""
        from html.parser import HTMLParser
        
        class TableParser(HTMLParser):
            def __init__(self):
                super().__init__()
                self.tables = []
                self.current_table = None
                self.current_row = None
                self.current_cell = None
                self.in_header = False
                self.in_cell = False
                
            def handle_starttag(self, tag, attrs):
                attrs_dict = dict(attrs)
                
                if tag == 'table' and 'id' in attrs_dict:
                    self.current_table = {
                        'id': attrs_dict['id'],
                        'headers': [],
                        'rows': []
                    }
                elif tag == 'thead':
                    self.in_header = True
                elif tag == 'tbody':
                    self.in_header = False
                elif tag == 'tr':
                    self.current_row = []
                elif tag in ['th', 'td'] and attrs_dict.get('contenteditable') == 'true':
                    self.in_cell = True
                    self.current_cell = ""
                    
            def handle_endtag(self, tag):
                if tag == 'table' and self.current_table:
                    self.tables.append(self.current_table)
                    self.current_table = None
                elif tag == 'tr' and self.current_row is not None:
                    if self.in_header and self.current_table:
                        self.current_table['headers'] = self.current_row
                    elif self.current_table:
                        self.current_table['rows'].append(self.current_row)
                    self.current_row = None
                elif tag in ['th', 'td']:
                    if self.in_cell and self.current_row is not None:
                        self.current_row.append(self.current_cell)
                    self.in_cell = False
                    
            def handle_data(self, data):
                if self.in_cell:
                    self.current_cell += data.strip()
        
        parser = TableParser()
        parser.feed(html_content)
        return parser.tables
    
    @staticmethod
    def infer_column_type(values: List[str]) -> str:
        """Infer SQL column type from values"""
        if not values:
            return "TEXT"
            
        # Check if all values are integers
        try:
            all(int(v) for v in values if v)
            return "INTEGER"
        except:
            pass
            
        # Check if all values are floats
        try:
            all(float(v) for v in values if v)
            return "REAL"
        except:
            pass
            
        # Check if values look like dates
        date_patterns = [
            r'^\d{4}-\d{2}-\d{2}$',
            r'^\d{2}/\d{2}/\d{4}$',
            r'^\d{2}-\d{2}-\d{4}$'
        ]
        for pattern in date_patterns:
            if all(re.match(pattern, v) for v in values if v):
                return "DATE"
                
        # Default to TEXT
        return "TEXT"
    
    @staticmethod
    def generate_sql(html_content: str, config: SQLExportConfig) -> str:
        """Generate SQL from edited HTML content"""
        tables = SQLExporter.parse_html_tables(html_content)
        
        if not tables:
            return "-- No tables found in HTML content"
            
        sql_parts = [
            f"-- Generated by CHONKER Phoenix 🐹",
            f"-- Generated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}",
            f"-- Source tables: {len(tables)}",
            ""
        ]
        
        for i, table in enumerate(tables):
            table_name = f"{config.table_name}_{i+1}" if len(tables) > 1 else config.table_name
            
            # Clean headers for SQL column names
            headers = []
            for h in table['headers']:
                clean_h = re.sub(r'[^a-zA-Z0-9_]', '_', h.lower())
                clean_h = re.sub(r'_+', '_', clean_h).strip('_')
                headers.append(clean_h or f'column_{len(headers)+1}')
            
            # Infer column types if enabled
            column_types = {}
            if config.infer_types and table['rows']:
                for i, header in enumerate(headers):
                    column_values = [row[i] for row in table['rows'] if i < len(row)]
                    column_types[header] = SQLExporter.infer_column_type(column_values)
            else:
                column_types = {h: "TEXT" for h in headers}
                
            # Override with config types if provided
            for col, typ in config.column_types.items():
                if col in headers:
                    column_types[col] = typ
            
            # Generate CREATE TABLE
            sql_parts.append(f"-- Table: {table_name}")
            sql_parts.append(f"CREATE TABLE IF NOT EXISTS {table_name} (")
            
            columns = []
            if config.primary_key and config.primary_key not in headers:
                columns.append(f"    {config.primary_key} INTEGER PRIMARY KEY AUTOINCREMENT")
                
            for header in headers:
                col_type = column_types.get(header, "TEXT")
                col_def = f"    {header} {col_type}"
                if header == config.primary_key:
                    col_def += " PRIMARY KEY"
                columns.append(col_def)
                
            sql_parts.append(",\n".join(columns))
            sql_parts.append(");")
            sql_parts.append("")
            
            # Generate INSERT statements
            if table['rows']:
                sql_parts.append(f"-- Data for {table_name}")
                for row in table['rows']:
                    if config.skip_empty_rows and all(not cell for cell in row):
                        continue
                        
                    values = []
                    for i, cell in enumerate(row[:len(headers)]):
                        if config.escape_strings:
                            # Escape single quotes
                            escaped = cell.replace("'", "''")
                            values.append(f"'{escaped}'")
                        else:
                            values.append(f"'{cell}'")
                            
                    sql_parts.append(
                        f"INSERT INTO {table_name} ({', '.join(headers)}) "
                        f"VALUES ({', '.join(values)});"
                    )
                sql_parts.append("")
                
        return "\n".join(sql_parts)

# ============================================================================
# MAIN WINDOW - OpenHands Bot's Assembly
# ============================================================================

class ChonkerPhoenix(QMainWindow):
    """The reborn CHONKER - lean and focused"""
    
    def __init__(self):
        super().__init__()
        self.processor = ChonkerProcessor()
        self.current_file = None
        self.caffeinate_process = None
        self._init_caffeinate()
        self.setup_ui()
        self.setup_connections()
        
    def _init_caffeinate(self):
        """🛡️ Initialize caffeinate defense against sleep/logout"""
        if CAFFEINATE_AVAILABLE:
            try:
                self.caffeinate_process = subprocess.Popen(
                    ['caffeinate', '-diu'],
                    stdout=subprocess.DEVNULL,
                    stderr=subprocess.DEVNULL
                )
                print("🛡️ Caffeinate defense activated!")
            except:
                print("⚠️ Warning: Caffeinate not available")
        
    def setup_ui(self):
        """Build the minimal UI"""
        self.setWindowTitle("🐹 CHONKER - PDF to SQL")
        self.setGeometry(100, 100, 1200, 800)
        
        # Central widget
        central = QWidget()
        self.setCentralWidget(central)
        layout = QVBoxLayout(central)
        
        # Main content area
        splitter = QSplitter(Qt.Orientation.Horizontal)
        
        # Left: PDF info panel
        info_panel = QWidget()
        info_layout = QVBoxLayout(info_panel)
        
        self.info_label = QLabel("No PDF loaded")
        self.info_label.setWordWrap(True)
        info_layout.addWidget(self.info_label)
        
        open_btn = QPushButton("🐹 Feed Me a PDF")
        open_btn.clicked.connect(self.open_pdf)
        info_layout.addWidget(open_btn)
        
        info_layout.addStretch()
        
        # Right: Belly viewer (HTML editor)
        self.belly_viewer = ChonkerBellyViewer()
        
        splitter.addWidget(info_panel)
        splitter.addWidget(self.belly_viewer)
        splitter.setStretchFactor(0, 1)
        splitter.setStretchFactor(1, 3)
        
        layout.addWidget(splitter)
        
        # Status bar
        self.status_bar = QStatusBar()
        self.setStatusBar(self.status_bar)
        
        self.progress_bar = QProgressBar()
        self.progress_bar.setVisible(False)
        self.status_bar.addPermanentWidget(self.progress_bar)
        
        # Load sacred hamster emoji - HIGHEST PRIORITY
        self.load_sacred_hamster()
        
        # Menu bar (after belly_viewer is created)
        self.create_menu_bar()
        
        # Apply dark theme
        self.setStyleSheet("""
            QMainWindow {
                background-color: #1E1E1E;
            }
            QWidget {
                background-color: #2D2E30;
                color: #E0E0E0;
            }
            QPushButton {
                background-color: #3A3C3E;
                border: 1px solid #525659;
                padding: 8px;
                border-radius: 4px;
            }
            QPushButton:hover {
                background-color: #525659;
            }
            QLabel {
                padding: 10px;
            }
        """)
        
    def load_sacred_hamster(self):
        """Load the SACRED Android 7.1 Noto Color Emoji hamster - DO NOT LOSE THIS"""
        try:
            # The holy path
            hamster_path = Path("icons/hamster_android7.png")
            
            if hamster_path.exists():
                from PyQt6.QtGui import QPixmap
                self.hamster_pixmap = QPixmap(str(hamster_path))
                self.setWindowIcon(QIcon(self.hamster_pixmap))
                print("🐹 Sacred Android 7.1 CHONKER emoji loaded!")
            else:
                print("⚠️  CRITICAL: Sacred hamster emoji missing at icons/hamster_android7.png")
                print("   This is the HIGHEST DIRECTIVE - please restore immediately!")
                
                # Create fallback hamster
                from PyQt6.QtGui import QPainter, QColor
                self.hamster_pixmap = QPixmap(256, 256)
                self.hamster_pixmap.fill(Qt.GlobalColor.transparent)
                
                painter = QPainter(self.hamster_pixmap)
                painter.setBrush(QColor("#D2691E"))  # Hamster brown
                painter.setPen(Qt.PenStyle.NoPen)
                painter.drawEllipse(64, 64, 128, 128)
                
                # Draw tiny eyes
                painter.setBrush(QColor("#000000"))
                painter.drawEllipse(90, 100, 10, 10)
                painter.drawEllipse(156, 100, 10, 10)
                
                # Draw tiny nose
                painter.drawEllipse(123, 130, 10, 10)
                painter.end()
                
                self.setWindowIcon(QIcon(self.hamster_pixmap))
                print("🐹 Using fallback hamster (but please restore the sacred one!)")
                
        except Exception as e:
            print(f"🐹 Hamster loading error (THIS IS CRITICAL): {e}")
    
    def create_menu_bar(self):
        """Create minimal menu bar"""
        menubar = self.menuBar()
        
        # File menu
        file_menu = menubar.addMenu("File")
        
        open_action = QAction("Open PDF", self)
        open_action.setShortcut(QKeySequence.StandardKey.Open)
        open_action.triggered.connect(self.open_pdf)
        file_menu.addAction(open_action)
        
        file_menu.addSeparator()
        
        exit_action = QAction("Exit", self)
        exit_action.setShortcut(QKeySequence.StandardKey.Quit)
        exit_action.triggered.connect(self.close)
        file_menu.addAction(exit_action)
        
        # Export menu
        export_menu = menubar.addMenu("Export")
        
        export_sql_action = QAction("Export to SQL", self)
        export_sql_action.setShortcut("Ctrl+E")
        export_sql_action.triggered.connect(self.export_to_sql)
        export_menu.addAction(export_sql_action)
        
        # View menu
        view_menu = menubar.addMenu("View")
        
        zoom_in_action = QAction("Zoom In", self)
        zoom_in_action.setShortcut(QKeySequence.StandardKey.ZoomIn)
        zoom_in_action.triggered.connect(self.belly_viewer.zoom_in)
        view_menu.addAction(zoom_in_action)
        
        zoom_out_action = QAction("Zoom Out", self)
        zoom_out_action.setShortcut(QKeySequence.StandardKey.ZoomOut)
        zoom_out_action.triggered.connect(self.belly_viewer.zoom_out)
        view_menu.addAction(zoom_out_action)
        
    def setup_connections(self):
        """Connect signals and slots"""
        self.processor.progress_update.connect(self.update_progress)
        self.processor.processing_complete.connect(self.processing_complete)
        
    def open_pdf(self):
        """Open a PDF for chomping with full validation"""
        file_path, _ = QFileDialog.getOpenFileName(
            self,
            "Select PDF to Chomp",
            "",
            "PDF Files (*.pdf)"
        )
        
        if file_path:
            # 🛡️ DEFENSE: Validate file size before processing
            try:
                file_size = os.path.getsize(file_path)
                file_size_mb = file_size / (1024 * 1024)
                
                if file_size > MAX_FILE_SIZE:
                    QMessageBox.warning(
                        self,
                        "File Too Large",
                        f"Cannot process file: {Path(file_path).name}\n\n"
                        f"File size: {file_size_mb:.1f} MB\n"
                        f"Maximum allowed: {MAX_FILE_SIZE / (1024 * 1024):.0f} MB\n\n"
                        f"Please use a smaller PDF file."
                    )
                    self.status_bar.showMessage(f"❌ File too large: {file_size_mb:.1f} MB")
                    return
                
                self.current_file = file_path
                self.info_label.setText(
                    f"📄 {Path(file_path).name}\n"
                    f"📏 Size: {file_size_mb:.1f} MB\n"
                    f"🔄 Ready to chomp!"
                )
                
                self.progress_bar.setVisible(True)
                self.status_bar.showMessage(f"Starting to chomp {file_size_mb:.1f}MB PDF...")
                self.processor.chomp_pdf(file_path)
                
            except OSError as e:
                QMessageBox.critical(
                    self,
                    "File Access Error", 
                    f"Cannot access file: {str(e)}"
                )
                self.status_bar.showMessage("❌ File access error")
            
    def update_progress(self, value: int, message: str):
        """Update progress bar"""
        self.progress_bar.setValue(value)
        self.status_bar.showMessage(message)
        
    def processing_complete(self, result: ProcessingResult):
        """Handle completed processing"""
        self.progress_bar.setVisible(False)
        
        if result.success:
            self.belly_viewer.display_chomped_content(result)
            self.status_bar.showMessage(
                f"Chomped in {result.processing_time:.1f}s - "
                f"Found {len(result.tables)} tables"
            )
        else:
            QMessageBox.critical(
                self,
                "Chomping Failed",
                f"Failed to chomp PDF:\n{result.error_message}"
            )
            
    def export_to_sql(self):
        """Export edited content to SQL"""
        if not hasattr(self.belly_viewer, '_last_result'):
            QMessageBox.warning(
                self,
                "Nothing to Export",
                "Please chomp a PDF first!"
            )
            return
            
        # Get edited HTML content
        html_content = self.belly_viewer.toHtml()
        
        # TODO: Show export dialog to configure table name, types, etc.
        
        # For now, use default config
        config = SQLExportConfig(
            table_name="extracted_data",
            column_types={"id": "INTEGER", "data": "TEXT"}
        )
        
        # Generate SQL
        sql = SQLExporter.generate_sql(html_content, config)
        
        # Save to file
        file_path, _ = QFileDialog.getSaveFileName(
            self,
            "Save SQL",
            f"{Path(self.current_file).stem}.sql",
            "SQL Files (*.sql)"
        )
        
        if file_path:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(sql)
            self.status_bar.showMessage(f"Pooped SQL to {Path(file_path).name}")
    
    def closeEvent(self, event):
        """🛡️ Clean shutdown with defensive cleanup"""
        # Stop any processing
        if self.processor and self.processor.isRunning():
            self.processor.stop_chomping()
            if not self.processor.wait(5000):  # 5 second timeout
                self.processor.terminate()
                
        # Stop caffeinate
        if self.caffeinate_process:
            try:
                self.caffeinate_process.terminate()
                print("🛡️ Caffeinate defense deactivated")
            except:
                pass
                
        event.accept()

# ============================================================================
# ENTRY POINT
# ============================================================================

def main():
    """Launch the Phoenix"""
    app = QApplication(sys.argv)
    app.setApplicationName("CHONKER Phoenix")
    
    # Create and show window
    window = ChonkerPhoenix()
    window.show()
    
    sys.exit(app.exec())

if __name__ == "__main__":
    main()