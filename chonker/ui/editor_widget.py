"""Editor widget for displaying and editing extracted content"""

from typing import Optional
import logging

from PyQt6.QtWidgets import QWidget, QVBoxLayout, QLineEdit, QPushButton, QHBoxLayout
from PyQt6.QtCore import Qt, pyqtSignal, QUrl
try:
    from PyQt6.QtWebEngineWidgets import QWebEngineView
    from PyQt6.QtWebEngineCore import QWebEnginePage
    WEBENGINE_AVAILABLE = True
except ImportError:
    WEBENGINE_AVAILABLE = False
    QWebEngineView = None
    QWebEnginePage = None

from ..models.document import Document


logger = logging.getLogger(__name__)


class EditorPage(QWebEnginePage):
    """Custom page to handle JavaScript console messages"""
    
    def javaScriptConsoleMessage(self, level, message, line, source):
        """Log JavaScript console messages"""
        if "Edit:" in message:
            # This is our edit message, don't log it
            return
        logger.debug(f"JS Console: {message}")


class EditorWidget(QWidget):
    """Web-based editor for spatial content"""
    
    content_changed = pyqtSignal(int, int, str)  # page_num, item_index, new_content
    
    def __init__(self):
        super().__init__()
        self.document: Optional[Document] = None
        self.init_ui()
        
    def init_ui(self):
        """Initialize the UI"""
        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        
        # Search bar
        self.search_widget = self._create_search_widget()
        self.search_widget.hide()
        layout.addWidget(self.search_widget)
        
        if WEBENGINE_AVAILABLE:
            # Web view
            self.web_view = QWebEngineView()
            self.web_page = EditorPage()
            self.web_view.setPage(self.web_page)
            layout.addWidget(self.web_view)
            
            # Set dark background
            self.web_view.setStyleSheet("QWebEngineView { background-color: #525659; }")
            self.web_view.page().setBackgroundColor(Qt.GlobalColor.transparent)
            
            # Inject JavaScript for handling edits
            self.web_view.loadFinished.connect(self._inject_edit_handler)
        else:
            # Fallback to text widget
            from PyQt6.QtWidgets import QTextEdit
            self.web_view = QTextEdit()
            self.web_view.setStyleSheet("""
                QTextEdit {
                    background-color: #525659;
                    color: #E0E0E0;
                    font-family: monospace;
                }
            """)
            layout.addWidget(self.web_view)
            logger.warning("PyQt6-WebEngine not available, using text fallback")
        
    def _create_search_widget(self) -> QWidget:
        """Create search widget"""
        widget = QWidget()
        layout = QHBoxLayout(widget)
        layout.setContentsMargins(5, 5, 5, 5)
        
        # Search input
        self.search_input = QLineEdit()
        self.search_input.setPlaceholderText("Find in document...")
        self.search_input.returnPressed.connect(self.find_next)
        layout.addWidget(self.search_input)
        
        # Buttons
        self.prev_button = QPushButton("Previous")
        self.prev_button.clicked.connect(self.find_previous)
        layout.addWidget(self.prev_button)
        
        self.next_button = QPushButton("Next")
        self.next_button.clicked.connect(self.find_next)
        layout.addWidget(self.next_button)
        
        # Close button
        close_button = QPushButton("×")
        close_button.setMaximumWidth(30)
        close_button.clicked.connect(self.hide_search)
        layout.addWidget(close_button)
        
        return widget
        
    def _inject_edit_handler(self):
        """Inject JavaScript to handle content edits"""
        js_code = """
        // Track edits to spatial items
        document.addEventListener('input', function(e) {
            if (e.target.classList.contains('spatial-item')) {
                const pageNum = parseInt(e.target.dataset.page || '1');
                const itemType = e.target.dataset.type;
                const content = e.target.innerText;
                
                // Find item index by counting previous siblings
                let index = 0;
                let sibling = e.target.previousElementSibling;
                while (sibling) {
                    if (sibling.classList.contains('spatial-item') && 
                        sibling.dataset.page === e.target.dataset.page) {
                        index++;
                    }
                    sibling = sibling.previousElementSibling;
                }
                
                // Send edit notification
                console.log(`Edit:${pageNum}:${index}:${content}`);
            }
        });
        
        // Prevent line breaks in spatial items
        document.addEventListener('keydown', function(e) {
            if (e.key === 'Enter' && e.target.classList.contains('spatial-item')) {
                e.preventDefault();
            }
        });
        """
        self.web_view.page().runJavaScript(js_code)
        
    def set_document(self, doc: Document):
        """Set the document being edited"""
        self.document = doc
        
    def set_content(self, html: str):
        """Set HTML content"""
        if WEBENGINE_AVAILABLE:
            # Wrap in basic HTML structure
            full_html = f"""
            <!DOCTYPE html>
            <html>
            <head>
                <meta charset="UTF-8">
                <style>
                    body {{
                        margin: 0;
                        padding: 0;
                        background: #525659;
                        font-family: Arial, sans-serif;
                    }}
                </style>
            </head>
            <body>
                {html}
            </body>
            </html>
            """
            
            self.web_view.setHtml(full_html)
        else:
            # For text edit, just set the HTML as text
            self.web_view.setHtml(html)
            # Ensure background color
            self.web_view.setStyleSheet("""
                QTextEdit {
                    background-color: #525659;
                    color: #E0E0E0;
                    font-family: monospace;
                }
            """)
        
    def get_content(self) -> str:
        """Get current HTML content"""
        # This would need to be implemented with JavaScript
        # For now, return empty string
        return ""
        
    def toggle_search(self):
        """Toggle search widget visibility"""
        if self.search_widget.isVisible():
            self.hide_search()
        else:
            self.show_search()
            
    def show_search(self):
        """Show search widget"""
        self.search_widget.show()
        self.search_input.setFocus()
        self.search_input.selectAll()
        
    def hide_search(self):
        """Hide search widget"""
        self.search_widget.hide()
        if WEBENGINE_AVAILABLE:
            self.web_view.findText("")  # Clear highlighting
        
    def find_next(self):
        """Find next occurrence"""
        text = self.search_input.text()
        if text and WEBENGINE_AVAILABLE:
            self.web_view.findText(text)
            
    def find_previous(self):
        """Find previous occurrence"""
        text = self.search_input.text()
        if text and WEBENGINE_AVAILABLE:
            self.web_view.findText(text, QWebEnginePage.FindFlag.FindBackward)
            
    def handle_console_message(self, message: str):
        """Handle console message from JavaScript"""
        if message.startswith("Edit:"):
            # Parse edit message
            parts = message.split(":", 3)
            if len(parts) == 4:
                try:
                    page_num = int(parts[1])
                    item_index = int(parts[2])
                    new_content = parts[3]
                    
                    # Emit signal
                    self.content_changed.emit(page_num, item_index, new_content)
                except ValueError:
                    logger.error(f"Invalid edit message: {message}")