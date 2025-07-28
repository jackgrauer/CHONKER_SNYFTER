"""Main window for CHONKER application"""

import os
import sys
from pathlib import Path
from typing import Optional
import logging

from PyQt6.QtWidgets import (
    QMainWindow, QWidget, QVBoxLayout, QHBoxLayout, 
    QSplitter, QStatusBar, QFileDialog, QMessageBox,
    QPushButton, QTextEdit, QLineEdit, QSizePolicy
)
from PyQt6.QtCore import Qt, QThread, pyqtSignal, QUrl, QTimer
from PyQt6.QtGui import QAction, QKeySequence, QIcon, QPixmap

from .editor_widget import EditorWidget
from .pdf_viewer import PDFViewer
from ..extraction.pdf_extractor import PDFExtractor
from ..extraction.spatial_layout import SpatialLayoutEngine
from ..export.html_generator import HTMLGenerator
from ..export.parquet_exporter import ParquetExporter
from ..models.document import Document


logger = logging.getLogger(__name__)


class ExtractionThread(QThread):
    """Background thread for PDF extraction"""
    progress = pyqtSignal(str)
    finished = pyqtSignal(object, object)  # Document, SpatialLayoutEngine
    error = pyqtSignal(str)
    
    def __init__(self, pdf_path: str):
        super().__init__()
        self.pdf_path = pdf_path
        
    def run(self):
        """Run extraction in background"""
        try:
            extractor = PDFExtractor(progress_callback=self.progress.emit)
            doc, layout = extractor.extract(self.pdf_path)
            self.finished.emit(doc, layout)
        except Exception as e:
            logger.error(f"Extraction failed: {e}")
            self.error.emit(str(e))


class MainWindow(QMainWindow):
    """Main application window with hamster wisdom"""
    
    def __init__(self):
        super().__init__()
        self.current_document: Optional[Document] = None
        self.current_layout: Optional[SpatialLayoutEngine] = None
        self.current_pdf_path: Optional[str] = None
        self.extraction_thread: Optional[ExtractionThread] = None
        
        # Load sacred hamster emoji
        self.load_sacred_hamster()
        
        # Animation state
        self.processing_animation_state = 0
        self.animation_timer = QTimer()
        self.animation_timer.timeout.connect(self._update_processing_animation)
        
        self.init_ui()
        self.create_actions()
        self.create_menus()
        self.apply_hamster_theme()
        
    def init_ui(self):
        """Initialize the hamster interface"""
        self.setWindowTitle("CHONKER - Elegant PDF Processing with Hamster Wisdom 🐹")
        self.setGeometry(100, 100, 1400, 900)
        
        # Sacred hamster geometry
        self.resize(1400, 900)
        self.setMinimumSize(1200, 700)
        
        # Create central widget
        central = QWidget()
        self.setCentralWidget(central)
        
        # Main vertical layout
        main_layout = QVBoxLayout(central)
        main_layout.setContentsMargins(0, 0, 0, 0)
        main_layout.setSpacing(0)
        
        # Top bar with CHONKER button and terminal
        self.top_bar = self._create_top_bar()
        main_layout.addWidget(self.top_bar)
        
        # Create main splitter (just two panes, no third pane!)
        self.splitter = QSplitter(Qt.Orientation.Horizontal)
        self.splitter.setStyleSheet("QSplitter::handle {background-color: #3A3C3E;}")
        main_layout.addWidget(self.splitter)
        
        # Left pane - PDF viewer
        self.pdf_viewer = PDFViewer()
        self.pdf_viewer.setStyleSheet("QWidget { background-color: #525659; }")
        self.splitter.addWidget(self.pdf_viewer)
        
        # Right pane - Editor
        self.editor = EditorWidget()
        self.editor.setStyleSheet("QWidget { background-color: #525659; }")
        self.splitter.addWidget(self.editor)
        
        # Set initial sizes (equal split)
        self.splitter.setSizes([700, 700])
        
        # Connect signals
        self.editor.content_changed.connect(self.on_content_changed)
        
        # Initial hamster message
        self.log("🐹 CHONKER ready!")
        
    def load_sacred_hamster(self):
        """Load the sacred Android 7.1 hamster emoji"""
        # Load the sacred Android 7.1 Noto emojis - NEVER let go of them!
        hamster_path = Path(__file__).parent.parent.parent / "assets" / "emojis" / "chonker.png"
        if hamster_path.exists():
            self.chonker_pixmap = QPixmap(str(hamster_path))
            print("Sacred Android 7.1 CHONKER emoji loaded!")
        else:
            self.chonker_pixmap = None
            print(f"Warning: Sacred hamster not found at {hamster_path}")
    
    def _create_top_bar(self) -> QWidget:
        """Create the top bar with CHONKER button, controls, and terminal"""
        container = QWidget()
        container.setObjectName("topBar")
        container.setStyleSheet("#topBar {background-color: #1ABC9C; border: 2px solid #3A3C3E; border-radius: 2px;}")
        container.setMaximumHeight(60)
        
        layout = QHBoxLayout(container)
        layout.setContentsMargins(5, 5, 5, 5)
        layout.setSpacing(10)
        
        # CHONKER button with sacred emoji
        self.chonker_btn = QPushButton()
        if self.chonker_pixmap:
            scaled_hamster = self.chonker_pixmap.scaled(48, 48, Qt.AspectRatioMode.KeepAspectRatio, Qt.TransformationMode.SmoothTransformation)
            self.chonker_btn.setIcon(QIcon(scaled_hamster))
            self.chonker_btn.setIconSize(scaled_hamster.size())
        self.chonker_btn.setText(" CHONKER")
        self.chonker_btn.setCheckable(True)
        self.chonker_btn.setChecked(True)
        self.chonker_btn.setMinimumWidth(180)
        self.chonker_btn.setSizePolicy(QSizePolicy.Policy.Preferred, QSizePolicy.Policy.Expanding)
        self.chonker_btn.setStyleSheet("QPushButton {background-color: #1ABC9C; color: white; font-size: 18px; font-weight: bold; border: none; padding: 5px;} QPushButton:hover {background-color: #16A085;}")
        self.chonker_btn.clicked.connect(lambda: self.log("🐹 CHONKER ready!"))
        layout.addWidget(self.chonker_btn)
        
        # Terminal display
        self.terminal = QTextEdit()
        self.terminal.setMinimumHeight(40)
        self.terminal.setMaximumHeight(50)
        self.terminal.setMaximumWidth(250)
        self.terminal.setReadOnly(True)
        self.terminal.setVerticalScrollBarPolicy(Qt.ScrollBarPolicy.ScrollBarAlwaysOff)
        self.terminal.setStyleSheet("QTextEdit {background-color: #2D2F31; color: #1ABC9C; font-family: 'Courier New', monospace; font-size: 11px; border: 1px solid #3A3C3E; padding: 2px;}")
        layout.addWidget(self.terminal)
        
        # Action button style
        action_button_style = "QPushButton{background:#3A3C3E;color:#B0B0B0;font-size:12px;border:1px solid #525659;padding:6px 10px;min-height:30px;max-height:35px}QPushButton:hover{background:#525659;color:#FFF}QPushButton:disabled{background:#2D2F31;color:#666}"
        
        # Open file button
        open_file_btn = QPushButton("Open PDF")
        open_file_btn.setStyleSheet(action_button_style)
        open_file_btn.clicked.connect(self.open_pdf)
        layout.addWidget(open_file_btn)
        
        # Export button
        self.export_btn = QPushButton("Export")
        self.export_btn.setStyleSheet(action_button_style)
        self.export_btn.clicked.connect(self.export_parquet)
        self.export_btn.setEnabled(False)
        layout.addWidget(self.export_btn)
        
        return container
    
    
    def log(self, message: str):
        """Add message to terminal"""
        self.terminal.append(message)
        cursor = self.terminal.textCursor()
        cursor.movePosition(cursor.MoveOperation.End)
        self.terminal.setTextCursor(cursor)
    
    def apply_hamster_theme(self):
        """Apply the sacred hamster theme"""
        # Colors from the sacred theme
        bg1, bg2, bg3 = "#525659", "#3A3C3E", "#1E1E1E"
        c1, c2, c3 = "#1ABC9C", "#16A085", "#FFB84D"
        
        css = f"""
        QMainWindow, QTextEdit {{background-color: {bg1}}}
        #topBar {{background-color: {bg1}; border-bottom: 1px solid {bg2}}}
        QScrollBar:vertical {{border: none; background: {bg2}; width: 10px; border-radius: 5px}}
        QScrollBar::handle:vertical {{background: {bg3}; min-height: 30px; border-radius: 5px}}
        QScrollBar::handle:vertical:hover {{background: #2B2D30}}
        QScrollBar:horizontal {{border: none; background: {bg2}; height: 10px; border-radius: 5px}}
        QScrollBar::handle:horizontal {{background: {bg3}; min-width: 30px; border-radius: 5px}}
        QScrollBar::handle:horizontal:hover {{background: #2B2D30}}
        QScrollBar::add-line, QScrollBar::sub-line {{border: none; background: none}}
        QStatusBar {{background-color: {bg3}; color: {c3}; border-top: 1px solid {bg2}}}
        """
        self.setStyleSheet(css)
    
    def create_actions(self):
        """Create menu actions"""
        # File actions
        self.open_action = QAction("&Open PDF", self)
        self.open_action.setShortcut(QKeySequence.StandardKey.Open)
        self.open_action.triggered.connect(self.open_pdf)
        
        self.export_action = QAction("&Export to Parquet", self)
        self.export_action.setShortcut(QKeySequence("Cmd+E"))
        self.export_action.triggered.connect(self.export_parquet)
        self.export_action.setEnabled(False)
        
        self.quit_action = QAction("&Quit", self)
        self.quit_action.setShortcut(QKeySequence.StandardKey.Quit)
        self.quit_action.triggered.connect(self.close)
        
        # Edit actions
        self.find_action = QAction("&Find", self)
        self.find_action.setShortcut(QKeySequence.StandardKey.Find)
        self.find_action.triggered.connect(self.editor.toggle_search)
        
    def create_menus(self):
        """Create menu bar"""
        menubar = self.menuBar()
        
        # File menu
        file_menu = menubar.addMenu("&File")
        file_menu.addAction(self.open_action)
        file_menu.addSeparator()
        file_menu.addAction(self.export_action)
        file_menu.addSeparator()
        file_menu.addAction(self.quit_action)
        
        # Edit menu
        edit_menu = menubar.addMenu("&Edit")
        edit_menu.addAction(self.find_action)
        
    def open_pdf(self):
        """Open a PDF file"""
        file_path, _ = QFileDialog.getOpenFileName(
            self,
            "Open PDF",
            "",
            "PDF Files (*.pdf);;All Files (*.*)"
        )
        
        if file_path:
            self.load_pdf(file_path)
            
    def load_pdf(self, file_path: str):
        """Load a PDF file"""
        self.current_pdf_path = file_path
        
        # Load in PDF viewer
        self.pdf_viewer.load_pdf(file_path)
        
        # Start extraction  
        self.editor.set_content("<p>Extracting content...</p>")
        
        # Stop any existing extraction
        if self.extraction_thread and self.extraction_thread.isRunning():
            self.extraction_thread.terminate()
            self.extraction_thread.wait()
            
        # Start new extraction
        self.extraction_thread = ExtractionThread(file_path)
        self.extraction_thread.progress.connect(self.on_extraction_progress)
        self.extraction_thread.finished.connect(self.on_extraction_finished)
        self.extraction_thread.error.connect(self.on_extraction_error)
        self.extraction_thread.start()
        
    def on_extraction_progress(self, message: str):
        """Handle extraction progress"""
        self.log(message)
        
    def _update_processing_animation(self):
        """Update hamster chomp animation"""
        states = ["🐹 *chomp*", "🐹 *chomp chomp*", "🐹 *CHOMP*", "🐹 *chomp chomp chomp*"]
        self.processing_animation_state = (self.processing_animation_state + 1) % len(states)
        message = f"{states[self.processing_animation_state]} Processing..."
        self.log(message)
        
    def on_extraction_finished(self, doc: Document, layout: SpatialLayoutEngine):
        """Handle extraction completion"""
        self.current_document = doc
        self.current_layout = layout
        
        # Generate HTML
        generator = HTMLGenerator(layout)
        html = generator.generate()
        
        # Display in editor
        self.editor.set_content(html)
        self.editor.set_document(doc)
        
        # Update status
        stats = doc.get_statistics()
        
        # Enable export
        self.export_action.setEnabled(True)
        self.export_btn.setEnabled(True)
        
    def on_extraction_error(self, error_msg: str):
        """Handle extraction error"""
        QMessageBox.critical(self, "Extraction Error", error_msg)
        
    def on_content_changed(self, page_num: int, item_index: int, new_content: str):
        """Handle content changes from editor"""
        if self.current_document:
            self.current_document.apply_edit(page_num, item_index, new_content)
            
            # Update status
            edit_count = self.current_document.get_edit_count()
            
    def export_parquet(self):
        """Export to Parquet format"""
        if not self.current_document or not self.current_layout:
            return
            
        # Get output directory
        default_name = Path(self.current_pdf_path).stem + "_export"
        output_dir = QFileDialog.getExistingDirectory(
            self,
            "Select Export Directory",
            str(Path.home() / default_name)
        )
        
        if not output_dir:
            return
            
        try:
            # Get current HTML
            edited_html = self.editor.get_content()
            
            # Create exporter
            exporter = ParquetExporter(
                self.current_document,
                self.current_layout,
                original_html="",  # Could store original
                edited_html=edited_html
            )
            
            # Export
            files = exporter.export(output_dir)
            
            # Show success
            file_list = "\n".join(f"- {f.name}" for f in files.values())
            QMessageBox.information(
                self,
                "Export Complete",
                f"Exported to:\n{output_dir}\n\nFiles created:\n{file_list}"
            )
            
            self.log("🐹 Export complete!")
            
        except Exception as e:
            logger.error(f"Export failed: {e}")
            self.log(f"🐹 Export failed: {e}")
            QMessageBox.critical(self, "Export Error", str(e))
            
    def closeEvent(self, event):
        """Handle window close"""
        if self.current_document and self.current_document.has_edits():
            reply = QMessageBox.question(
                self,
                "Unsaved Changes",
                "You have unsaved edits. Are you sure you want to quit?",
                QMessageBox.StandardButton.Yes | QMessageBox.StandardButton.No,
                QMessageBox.StandardButton.No
            )
            
            if reply == QMessageBox.StandardButton.No:
                event.ignore()
                return
                
        event.accept()