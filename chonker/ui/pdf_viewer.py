"""PDF viewer widget for displaying source PDFs"""

from pathlib import Path
import logging

from PyQt6.QtWidgets import QWidget, QVBoxLayout, QLabel
from PyQt6.QtCore import Qt, QUrl, QPoint
from PyQt6.QtPdf import QPdfDocument
from PyQt6.QtPdfWidgets import QPdfView


logger = logging.getLogger(__name__)


class PDFViewer(QWidget):
    """Widget for viewing PDF files"""
    
    def __init__(self):
        super().__init__()
        self.pdf_document = QPdfDocument(self)
        self.init_ui()
        
    def init_ui(self):
        """Initialize the UI"""
        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        
        # PDF view
        self.pdf_view = QPdfView(self)
        self.pdf_view.setDocument(self.pdf_document)
        self.pdf_view.setPageMode(QPdfView.PageMode.SinglePage)
        self.pdf_view.setZoomMode(QPdfView.ZoomMode.FitToWidth)
        
        # Dark theme for PDF viewer
        self.pdf_view.setStyleSheet("""
            QPdfView {
                background-color: #3C3F41;
            }
        """)
        
        layout.addWidget(self.pdf_view)
        
        # Placeholder when no PDF
        self.placeholder = QLabel("No PDF loaded")
        self.placeholder.setAlignment(Qt.AlignmentFlag.AlignCenter)
        self.placeholder.setStyleSheet("""
            QLabel {
                color: #808080;
                font-size: 16px;
                background-color: #3C3F41;
            }
        """)
        layout.addWidget(self.placeholder)
        
        # Initially show placeholder
        self.pdf_view.hide()
        
    def load_pdf(self, file_path: str):
        """Load a PDF file"""
        try:
            # Load PDF
            self.pdf_document.load(file_path)
            
            if self.pdf_document.status() == QPdfDocument.Status.Ready:
                # Show PDF view
                self.placeholder.hide()
                self.pdf_view.show()
                
                # Zoom to fit
                self.pdf_view.setZoomMode(QPdfView.ZoomMode.FitToWidth)
                
                logger.info(f"Loaded PDF: {Path(file_path).name}")
            else:
                # Show error
                self.placeholder.setText(f"Failed to load PDF: {Path(file_path).name}")
                self.placeholder.show()
                self.pdf_view.hide()
                
        except Exception as e:
            logger.error(f"Error loading PDF: {e}")
            self.placeholder.setText(f"Error: {str(e)}")
            self.placeholder.show()
            self.pdf_view.hide()
            
    def clear(self):
        """Clear the current PDF"""
        self.pdf_document.close()
        self.pdf_view.hide()
        self.placeholder.setText("No PDF loaded")
        self.placeholder.show()
        
    def next_page(self):
        """Go to next page"""
        nav = self.pdf_view.pageNavigator()
        if nav.currentPage() < self.pdf_document.pageCount() - 1:
            nav.jump(nav.currentPage() + 1, QPoint())
            
    def previous_page(self):
        """Go to previous page"""
        nav = self.pdf_view.pageNavigator()
        if nav.currentPage() > 0:
            nav.jump(nav.currentPage() - 1, QPoint())
            
    def zoom_in(self):
        """Zoom in"""
        self.pdf_view.setZoomFactor(self.pdf_view.zoomFactor() * 1.2)
        
    def zoom_out(self):
        """Zoom out"""
        self.pdf_view.setZoomFactor(self.pdf_view.zoomFactor() / 1.2)
        
    def fit_width(self):
        """Fit to width"""
        self.pdf_view.setZoomMode(QPdfView.ZoomMode.FitToWidth)
        
    def fit_page(self):
        """Fit entire page"""
        self.pdf_view.setZoomMode(QPdfView.ZoomMode.FitInView)