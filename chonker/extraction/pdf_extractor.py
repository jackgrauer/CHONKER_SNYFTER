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
from .vector_extractor import VectorGraphicsExtractor


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
        
        # Initialize vector graphics extractor for tables
        vector_extractor = VectorGraphicsExtractor(pdf_path)
        
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
                
            # Debug logging for what Docling found
            item_type = type(item).__name__
            logger.debug(f"🐹 Item {idx}: {item_type}, level={level}")
            if hasattr(item, 'text'):
                logger.debug(f"   Text preview: {str(item.text)[:50]}...")
                
            # Extract item info
            layout_item = self._process_item(item, level, idx)
            
            if layout_item:
                # Add to layout engine (handles overlap resolution)
                layout_item = layout.add_item(layout_item)
                
                # If it's a table, try to extract vector graphics
                if layout_item.item_type == 'table':
                    logger.info(f"🐹 Found table at page {layout_item.page_num}, bbox: {layout_item.bbox}")
                    # Skip vector extraction entirely - these are image tables
                    logger.info(f"🐹 Skipping vector extraction for image-based table")
                
                # Add to document
                doc.add_item(layout_item.page_num, layout_item)
                
        # Look for gaps between items that might be tables
        # self._detect_missing_tables(doc, layout)
        
        self._emit_progress("🐹 *burp* Extraction complete!")
        
        return doc, layout
    
    def _create_fallback_table(self, layout_item: LayoutItem) -> str:
        """Create a simple table structure when vector extraction fails"""
        # Extract any existing table text
        table_text = layout_item.content
        
        # Log what we're working with
        logger.info(f"🐹 Creating fallback table with content: '{table_text[:100]}...' ({len(table_text)} chars)")
        
        # If content is just "[Table]", try to get more info
        if table_text == "[Table]" or not table_text.strip():
            table_text = "[Table content not extracted - likely an image]"
        
        # Since tables are images, let's style it appropriately
        import html
        escaped_text = html.escape(table_text)
        
        return f'''<div style="border: 2px solid #1ABC9C; padding: 8px; width: 100%; height: 100%; box-sizing: border-box; background: rgba(26, 188, 156, 0.05);">
            <div style="color: #1ABC9C; font-size: 11px; margin-bottom: 6px; font-weight: bold;">📊 Table (Image)</div>
            <div style="color: #E0E0E0; font-family: monospace; font-size: 11px; white-space: pre-wrap;">{escaped_text}</div>
        </div>'''
        
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
                # Get page height for coordinate conversion
                page_height = 792  # Default letter size
                if hasattr(prov, 'page_height'):
                    page_height = prov.page_height
                    
                # Debug coordinate system
                if hasattr(prov.bbox, 'coord_origin'):
                    logger.debug(f"   Bbox coord origin: {prov.bbox.coord_origin}")
                logger.debug(f"   Raw bbox: l={prov.bbox.l}, t={prov.bbox.t}, r={prov.bbox.r}, b={prov.bbox.b}")
                    
                bbox = BoundingBox.from_docling_bbox(prov.bbox, page_height)
                
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
        logger.info(f"🐹 Extracting table text from {type(table_item).__name__}")
        
        # Try export_to_markdown first (best for display)
        if hasattr(table_item, 'export_to_markdown'):
            try:
                text = table_item.export_to_markdown()
                logger.info(f"🐹 Got text from export_to_markdown(): {len(text)} chars")
                
                # Check if it's actual markdown or just object repr
                if text and text.strip() and not text.startswith('table_cells=') and '|' in text:
                    # Looks like actual markdown
                    return text
                elif 'table_cells=' in text or text.startswith('table_cells='):
                    # It's the object representation
                    logger.warning(f"🐹 export_to_markdown returned object representation")
                    return self._parse_table_repr(text)
                elif not text.strip():
                    logger.warning(f"🐹 export_to_markdown returned empty string")
            except Exception as e:
                logger.warning(f"🐹 export_to_markdown failed: {e}")
        
        # Try export_to_html 
        if hasattr(table_item, 'export_to_html'):
            try:
                text = table_item.export_to_html()
                logger.info(f"🐹 Got HTML from export_to_html(): {len(text)} chars")
                if text and text.strip():
                    return text
            except Exception as e:
                logger.warning(f"🐹 export_to_html failed: {e}")
                
        # Try data attribute
        if hasattr(table_item, 'data'):
            try:
                data = table_item.data
                logger.info(f"🐹 Got data attribute: {type(data)}")
                if data:
                    data_str = str(data)
                    # Check if it's the object representation
                    if 'table_cells=' in data_str:
                        logger.info(f"🐹 TableData contains object representation")
                        return self._parse_table_repr(data_str)
                    else:
                        return data_str
            except Exception as e:
                logger.warning(f"🐹 data access failed: {e}")
                
        # Try to extract from table cells if available
        if hasattr(table_item, 'table_cells'):
            try:
                cells = table_item.table_cells
                logger.info(f"🐹 Found {len(cells)} table cells")
                # Build a simple text representation
                rows = {}
                for cell in cells:
                    row = cell.start_row_offset_idx
                    col = cell.start_col_offset_idx
                    if row not in rows:
                        rows[row] = {}
                    rows[row][col] = cell.text
                
                # Convert to markdown-like format
                lines = []
                for row_idx in sorted(rows.keys()):
                    row = rows[row_idx]
                    cells_text = [row.get(col, '') for col in sorted(row.keys())]
                    lines.append(' | '.join(cells_text))
                    if row_idx == 0:  # Add separator after header
                        lines.append(' | '.join(['---'] * len(cells_text)))
                
                return '\n'.join(lines)
            except Exception as e:
                logger.warning(f"🐹 table_cells extraction failed: {e}")
        
        # Try older methods
        if hasattr(table_item, 'table'):
            logger.info(f"🐹 Table item has 'table' attribute")
            if hasattr(table_item.table, 'to_text'):
                text = table_item.table.to_text()
                logger.info(f"🐹 Got text from table.to_text(): {len(text)} chars")
                return text
                
        if hasattr(table_item, 'text'):
            text = str(table_item.text)
            logger.info(f"🐹 Got text from item.text: {len(text)} chars")
            return text
            
        # Log what attributes the table item has
        attrs = [attr for attr in dir(table_item) if not attr.startswith('_')]
        logger.warning(f"🐹 Table item attributes: {attrs}")
        
        return "[Table - No text extracted]"
            
    def _parse_table_repr(self, repr_text: str) -> str:
        """Parse table object representation to extract text"""
        import re
        
        # Extract text from each cell using regex
        cell_texts = re.findall(r"text='([^']*)'", repr_text)
        
        # Extract row and column indices
        cells_data = []
        for match in re.finditer(r"start_row_offset_idx=(\d+).*?start_col_offset_idx=(\d+).*?text='([^']*)'", repr_text):
            row, col, text = match.groups()
            cells_data.append((int(row), int(col), text))
        
        if not cells_data:
            logger.warning(f"🐹 Could not parse table representation")
            return "[Table]"
        
        # Build markdown table
        rows = {}
        for row_idx, col_idx, text in cells_data:
            if row_idx not in rows:
                rows[row_idx] = {}
            rows[row_idx][col_idx] = text
        
        # Convert to markdown
        lines = []
        for row_idx in sorted(rows.keys()):
            row = rows[row_idx]
            max_col = max(row.keys()) if row else 0
            cells = [row.get(col, '') for col in range(max_col + 1)]
            lines.append(' | '.join(cells))
            if row_idx == 0:  # Add separator after header
                lines.append(' | '.join(['---'] * len(cells)))
        
        result = '\n'.join(lines)
        logger.info(f"🐹 Parsed table repr to markdown: {len(result)} chars")
        return result
    
    def _detect_missing_tables(self, doc: Document, layout: SpatialLayoutEngine):
        """Detect gaps between text that might be missing tables"""
        logger.info("🐹 Checking for gaps that might be missing tables...")
        
        for page_num, page in doc.pages.items():
            items = sorted(page.items, key=lambda x: x.bbox.top)
            
            # Log item positions to debug overlaps
            logger.debug(f"🐹 Page {page_num} has {len(items)} items:")
            for idx, item in enumerate(items[:10]):  # First 10 items
                logger.debug(f"  Item {idx}: top={item.bbox.top:.1f}, bottom={item.bbox.bottom:.1f}, height={item.bbox.height:.1f}, content='{item.content[:30]}...'")
            
            for i in range(len(items) - 1):
                curr_item = items[i]
                next_item = items[i + 1]
                
                # Calculate gap between items
                gap = next_item.bbox.top - curr_item.bbox.bottom
                
                # Skip if items overlap (negative gap)
                if gap < 0:
                    logger.debug(f"🐹 Items overlap by {-gap:.1f}px, skipping gap check")
                    continue
                    
                # If there's a significant gap (more than ~100 pixels)
                if gap > 100:
                    logger.warning(f"🐹 Large gap detected on page {page_num}: {gap:.1f}px between items")
                    logger.warning(f"   After: '{curr_item.content[:50]}...' (bottom: {curr_item.bbox.bottom:.1f})")
                    logger.warning(f"   Before: '{next_item.content[:50]}...' (top: {next_item.bbox.top:.1f})")
                    
                    # Create a placeholder for potential missing table
                    placeholder = LayoutItem(
                        bbox=BoundingBox(
                            left=curr_item.bbox.left,
                            top=curr_item.bbox.bottom + 20,
                            right=curr_item.bbox.right,
                            bottom=next_item.bbox.top - 20
                        ),
                        content="[📄 Potential missing table - Tables between text chunks may not be detected]\n\n" +
                                "This PDF contains image-based tables that Docling couldn't detect.\n" +
                                "The table should appear in this location.",
                        item_type='missing_table',
                        page_num=page_num,
                        metadata={'gap_size': gap},
                        style={'color': '#E67E22', 'italic': True}
                    )
                    
                    # Add to layout engine
                    placeholder = layout.add_item(placeholder)
                    doc.add_item(page_num, placeholder)
                    
    def _emit_progress(self, message: str):
        """Emit progress message"""
        logger.info(message)
        if self.progress_callback:
            self.progress_callback(message)