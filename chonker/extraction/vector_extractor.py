"""Extract vector graphics (lines, rectangles) from PDFs using PyMuPDF"""

import logging
from typing import List, Dict, Tuple, Optional
import fitz  # PyMuPDF

from ..models.bbox import BoundingBox
from ..models.layout_item import LayoutItem

logger = logging.getLogger(__name__)


class VectorGraphicsExtractor:
    """Extract lines and rectangles that form tables in PDFs"""
    
    def __init__(self, pdf_path: str):
        """Initialize with PDF document"""
        self.pdf_path = pdf_path
        self.doc = fitz.open(pdf_path)
        
    def extract_table_graphics(self, page_num: int, table_bbox: BoundingBox) -> Optional[Dict]:
        """
        Extract vector graphics within a table bounding box.
        
        Returns:
            Dict containing horizontal_lines, vertical_lines, and cells
        """
        if page_num < 1 or page_num > len(self.doc):
            return None
            
        page = self.doc[page_num - 1]  # 0-indexed
        page_height = page.rect.height
        
        # Convert our bbox to PyMuPDF rect (with PDF coords)
        pdf_bbox = fitz.Rect(
            table_bbox.left,
            page_height - table_bbox.bottom,  # Convert from top-left to bottom-left origin
            table_bbox.right,
            page_height - table_bbox.top
        )
        
        # Extract drawings from the page
        drawings = page.get_drawings()
        logger.info(f"🐹 Found {len(drawings)} drawings on page {page_num}")
        
        # Also try to get paths if drawings are empty
        if not drawings:
            logger.info(f"🐹 No drawings found, trying alternative extraction methods...")
            # Try getting the display list
            try:
                paths = page.get_paths()
                logger.info(f"🐹 Found {len(paths)} paths on page")
                # Convert paths to drawings format
                for path in paths:
                    drawing = {"items": []}
                    for item in path["items"]:
                        drawing["items"].append(item)
                    drawings.append(drawing)
            except Exception as e:
                logger.warning(f"🐹 Could not extract paths: {e}")
        
        horizontal_lines = []
        vertical_lines = []
        rectangles = []
        
        for drawing in drawings:
            # Check if drawing intersects with our table bbox
            for item in drawing["items"]:
                if item[0] == "l":  # Line
                    p1 = item[1]
                    p2 = item[2]
                    
                    # Check if line is within table bounds
                    if self._point_in_rect(p1, pdf_bbox) or self._point_in_rect(p2, pdf_bbox):
                        # Determine if horizontal or vertical
                        if abs(p1.y - p2.y) < 2:  # Horizontal line
                            horizontal_lines.append({
                                'y': page_height - p1.y,  # Convert to top-left coords
                                'x1': p1.x,
                                'x2': p2.x
                            })
                        elif abs(p1.x - p2.x) < 2:  # Vertical line
                            vertical_lines.append({
                                'x': p1.x,
                                'y1': page_height - p1.y,  # Convert to top-left coords
                                'y2': page_height - p2.y
                            })
                            
                elif item[0] == "re":  # Rectangle
                    rect = item[1]
                    if rect.intersects(pdf_bbox):
                        rectangles.append({
                            'left': rect.x0,
                            'top': page_height - rect.y1,  # Convert to top-left coords
                            'right': rect.x1,
                            'bottom': page_height - rect.y0,
                            'width': rect.width,
                            'height': rect.height
                        })
        
        # Sort lines for easier processing
        horizontal_lines.sort(key=lambda l: l['y'])
        vertical_lines.sort(key=lambda l: l['x'])
        
        logger.info(f"🐹 Table bbox: {table_bbox}")
        logger.info(f"🐹 Found {len(horizontal_lines)} h-lines, {len(vertical_lines)} v-lines, {len(rectangles)} rectangles")
        
        # If we don't have enough lines but have rectangles, use those
        if len(horizontal_lines) < 2 or len(vertical_lines) < 2:
            if rectangles:
                logger.info(f"🐹 Using rectangles to infer table structure")
                # Extract lines from rectangles
                for rect in rectangles:
                    # Add horizontal lines from rectangle edges
                    if not any(abs(l['y'] - rect['top']) < 2 for l in horizontal_lines):
                        horizontal_lines.append({'y': rect['top'], 'x1': rect['left'], 'x2': rect['right']})
                    if not any(abs(l['y'] - rect['bottom']) < 2 for l in horizontal_lines):
                        horizontal_lines.append({'y': rect['bottom'], 'x1': rect['left'], 'x2': rect['right']})
                    
                    # Add vertical lines from rectangle edges
                    if not any(abs(l['x'] - rect['left']) < 2 for l in vertical_lines):
                        vertical_lines.append({'x': rect['left'], 'y1': rect['top'], 'y2': rect['bottom']})
                    if not any(abs(l['x'] - rect['right']) < 2 for l in vertical_lines):
                        vertical_lines.append({'x': rect['right'], 'y1': rect['top'], 'y2': rect['bottom']})
                
                # Re-sort after adding new lines
                horizontal_lines.sort(key=lambda l: l['y'])
                vertical_lines.sort(key=lambda l: l['x'])
        
        # Detect grid cells from line intersections
        cells = self._detect_cells(horizontal_lines, vertical_lines, table_bbox)
        
        return {
            'horizontal_lines': horizontal_lines,
            'vertical_lines': vertical_lines,
            'rectangles': rectangles,
            'cells': cells,
            'bbox': table_bbox
        }
    
    def _point_in_rect(self, point: fitz.Point, rect: fitz.Rect) -> bool:
        """Check if point is within rectangle"""
        return (rect.x0 <= point.x <= rect.x1 and 
                rect.y0 <= point.y <= rect.y1)
    
    def _detect_cells(self, h_lines: List[Dict], v_lines: List[Dict], 
                     table_bbox: BoundingBox) -> List[Dict]:
        """Detect table cells from intersecting lines"""
        cells = []
        
        # If we have at least 2 lines in each direction, create cells
        if len(h_lines) >= 2 and len(v_lines) >= 2:
            # Create cells from adjacent line pairs
            for i in range(len(h_lines) - 1):
                for j in range(len(v_lines) - 1):
                    cell = {
                        'row': i,
                        'col': j,
                        'left': v_lines[j]['x'],
                        'top': h_lines[i]['y'],
                        'right': v_lines[j + 1]['x'],
                        'bottom': h_lines[i + 1]['y'],
                        'width': v_lines[j + 1]['x'] - v_lines[j]['x'],
                        'height': h_lines[i + 1]['y'] - h_lines[i]['y']
                    }
                    
                    # Only include cells that are reasonably sized
                    if cell['width'] > 5 and cell['height'] > 5:
                        cells.append(cell)
        else:
            # Fallback: create a single cell for the entire table
            logger.warning(f"🐹 Not enough lines for grid detection, creating single cell")
            cells = [{
                'row': 0,
                'col': 0,
                'left': table_bbox.left,
                'top': table_bbox.top,
                'right': table_bbox.right,
                'bottom': table_bbox.bottom,
                'width': table_bbox.width,
                'height': table_bbox.height
            }]
        
        logger.info(f"🐹 Detected {len(cells)} cells")
        return cells
    
    def create_table_structure_html(self, table_graphics: Dict) -> str:
        """
        Create HTML representation of table structure.
        """
        if not table_graphics or not table_graphics.get('cells'):
            return ""
            
        cells = table_graphics['cells']
        if not cells:
            return ""
            
        # Group cells by row
        rows = {}
        for cell in cells:
            row_idx = cell['row']
            if row_idx not in rows:
                rows[row_idx] = []
            rows[row_idx].append(cell)
        
        # Build HTML table
        html_parts = ['<table style="width: 100%; border-collapse: collapse;">']
        
        for row_idx in sorted(rows.keys()):
            html_parts.append('<tr>')
            row_cells = sorted(rows[row_idx], key=lambda c: c['col'])
            
            for cell in row_cells:
                style = (
                    f"border: 1px solid #666; "
                    f"padding: 4px; "
                    f"min-height: {cell['height']}px; "
                    f"min-width: {cell['width']}px; "
                    f"position: relative;"
                )
                html_parts.append(f'<td style="{style}">&nbsp;</td>')
                
            html_parts.append('</tr>')
            
        html_parts.append('</table>')
        
        return '\n'.join(html_parts)
    
    def __del__(self):
        """Clean up PDF document"""
        if hasattr(self, 'doc'):
            self.doc.close()