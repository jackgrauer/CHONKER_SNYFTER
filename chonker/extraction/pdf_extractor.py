"""PDF extraction using Docling with spatial layout support"""

import logging
from typing import Optional, Tuple, List, Dict
from pathlib import Path

from docling.document_converter import DocumentConverter, FormatOption
from docling_core.types.doc import ImageRefMode, PictureItem, TableItem
from docling.datamodel.base_models import InputFormat

from ..models.document import Document, Page
from ..models.layout_item import LayoutItem  
from ..models.bbox import BoundingBox
from .spatial_layout import SpatialLayoutEngine


logger = logging.getLogger(__name__)


class PDFExtractor:
    """Extract content from PDFs using Docling"""
    
    def __init__(self, progress_callback=None):
        """
        Initialize the PDF extractor.
        
        Args:
            progress_callback: Optional callback for progress updates
        """
        self.progress_callback = progress_callback
        self._init_converter()
        
    def _init_converter(self):
        """Initialize Docling converter with optimal settings"""
        try:
            # Try to use table detection
            from docling.datamodel.pipeline_options import TableFormerMode
            from docling.document_converter import FormatOption
            
            # Create format options with table detection for PDF format
            format_options = {
                InputFormat.PDF: FormatOption(
                    table_former_mode=TableFormerMode.ACCURATE
                )
            }
            
            self.converter = DocumentConverter(
                format_options=format_options
            )
            logger.info("🐹 Initialized with ACCURATE table detection")
            
        except Exception as e:
            # Fallback to basic converter
            logger.warning(f"Could not initialize with table options: {e}")
            self.converter = DocumentConverter()
            logger.info("🐹 Using basic converter")
            
    def extract(self, pdf_path: str) -> Tuple[Document, SpatialLayoutEngine]:
        """
        Extract content from PDF with spatial layout.
        
        Args:
            pdf_path: Path to the PDF file
            
        Returns:
            Tuple of (Document, SpatialLayoutEngine)
        """
        self._emit_progress("🐹 *chomp chomp* Processing document...")
        
        # Convert PDF
        try:
            result = self.converter.convert(pdf_path)
        except Exception as e:
            logger.error(f"Failed to convert PDF: {e}")
            raise
            
        # Create document and layout engine
        doc = Document(source_path=pdf_path)
        layout = SpatialLayoutEngine()
        
        # Extract metadata
        doc.metadata = {
            'title': getattr(result.document, 'title', Path(pdf_path).stem),
            'page_count': getattr(result.document, 'page_count', 0)
        }
        
        # Process items
        items = list(result.document.iterate_items())
        total_items = len(items)
        
        self._emit_progress(f"🐹 Found {total_items} tasty items to process...")
        
        for idx, (item, level) in enumerate(items):
            # Progress update
            if idx % 10 == 0:
                self._emit_progress(f"🐹 *chomp* {idx}/{total_items}")
                
            # Extract item info
            layout_item = self._process_item(item, level, idx)
            
            if layout_item:
                # Add to layout engine (handles overlap resolution)
                layout_item = layout.add_item(layout_item)
                
                # Add to document
                doc.add_item(layout_item.page_num, layout_item)
                
        self._emit_progress("🐹 *burp* Extraction complete!")
        
        return doc, layout
        
    def _process_item(self, item, level: int, index: int) -> Optional[LayoutItem]:
        """Process a single Docling item into a LayoutItem"""
        # Get text content
        if hasattr(item, 'text'):
            content = str(item.text)
        elif isinstance(item, TableItem):
            content = self._extract_table_text(item)
        elif isinstance(item, PictureItem):
            content = f"[Image: {getattr(item, 'caption', 'No caption')}]"
        else:
            content = str(item)
            
        if not content.strip():
            return None
            
        # Get position info
        page_num = 1  # Default
        bbox = None
        
        if hasattr(item, 'prov') and item.prov:
            prov = item.prov[0] if isinstance(item.prov, list) else item.prov
            
            # Extract page number
            page_num = getattr(prov, 'page_no', 1)
            
            # Extract bounding box
            if hasattr(prov, 'bbox'):
                bbox = BoundingBox.from_docling_bbox(prov.bbox)
                
        # If no bbox, create a dummy one
        if not bbox:
            # Estimate position based on index
            y_offset = index * 20
            bbox = BoundingBox(
                left=50,
                top=50 + y_offset,
                right=500,
                bottom=70 + y_offset
            )
            
        # Determine item type
        item_type = self._get_item_type(item)
        
        # Extract style info
        style = self._extract_style(item)
        
        # Create layout item
        return LayoutItem(
            bbox=bbox,
            content=content,
            item_type=item_type,
            level=level,
            page_num=page_num,
            metadata={'index': index},
            style=style
        )
        
    def _get_item_type(self, item) -> str:
        """Determine the type of a Docling item"""
        type_name = type(item).__name__
        
        # Map Docling types to our types
        type_map = {
            'TextItem': 'text',
            'Title': 'h1',
            'SectionHeader': 'h2', 
            'Paragraph': 'p',
            'TableItem': 'table',
            'PictureItem': 'image',
            'ListItem': 'li'
        }
        
        # Check for heading levels
        if hasattr(item, 'level'):
            level = getattr(item, 'level', 0)
            if level > 0 and type_name in ['Title', 'SectionHeader']:
                return f'h{min(level, 6)}'
                
        return type_map.get(type_name, 'text')
        
    def _extract_style(self, item) -> Optional[Dict]:
        """Extract style information from item"""
        style = {}
        
        # Check for formatting attribute
        if hasattr(item, 'formatting') and item.formatting:
            fmt = item.formatting
            if hasattr(fmt, 'font_size'):
                style['font_size'] = fmt.font_size
            if hasattr(fmt, 'font_name'):
                style['font_name'] = fmt.font_name
            if hasattr(fmt, 'bold'):
                style['bold'] = fmt.bold
            if hasattr(fmt, 'italic'):
                style['italic'] = fmt.italic
                
        return style if style else None
        
    def _extract_table_text(self, table_item) -> str:
        """Extract text from a table item"""
        if hasattr(table_item, 'table') and hasattr(table_item.table, 'to_text'):
            return table_item.table.to_text()
        elif hasattr(table_item, 'text'):
            return str(table_item.text)
        else:
            return "[Table]"
            
    def _emit_progress(self, message: str):
        """Emit progress message"""
        logger.info(message)
        if self.progress_callback:
            self.progress_callback(message)