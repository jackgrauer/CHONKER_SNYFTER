"""HTML generation for spatial layout display"""

from typing import List, Dict, Optional
import html
import logging

from ..extraction.spatial_layout import SpatialLayoutEngine
from ..models.layout_item import LayoutItem

logger = logging.getLogger(__name__)


class HTMLGenerator:
    """Generate HTML from spatial layout"""
    
    def __init__(self, layout_engine: SpatialLayoutEngine, scale: float = 1.2):
        """
        Initialize HTML generator.
        
        Args:
            layout_engine: The spatial layout engine with positioned items
            scale: Scale factor for display
        """
        self.layout = layout_engine
        self.scale = scale
        
    def generate(self, page_nums: Optional[List[int]] = None) -> str:
        """
        Generate complete HTML document.
        
        Args:
            page_nums: List of page numbers to include (None = all pages)
        """
        if page_nums is None:
            page_nums = sorted(self.layout.pages.keys())
            
        html_parts = [
            self._generate_header(),
            '<div id="document-content" contenteditable="true">'
        ]
        
        for page_num in page_nums:
            if page_num > min(page_nums):
                html_parts.append(self._generate_page_break(page_num))
            html_parts.append(self._generate_page(page_num))
            
        html_parts.append('</div>')
        
        return '\n'.join(html_parts)
    
    def _generate_header(self) -> str:
        """Generate CSS styles for the document"""
        return '''
<style>
    #document-content {
        font-family: Arial, sans-serif;
        color: #E0E0E0;
        background: #525659;
        padding: 20px;
        min-height: 100vh;
        /* Sacred hamster background */
    }
    
    .spatial-page {
        position: relative;
        margin-bottom: 40px;
        border: 2px solid #1ABC9C;
        background: #525659;
        overflow: visible;
    }
    
    .spatial-item {
        position: absolute !important;
        padding: 4px 6px;
        margin: 2px;
        overflow: visible;
        cursor: text;
        transition: all 0.2s;
        box-sizing: border-box;
    }
    
    /* Table styles for vector graphics */
    .spatial-item table {
        border-collapse: collapse;
        width: 100%;
        height: 100%;
    }
    
    .spatial-item table td {
        border: 1px solid #666;
        padding: 2px 4px;
        vertical-align: top;
        color: #E0E0E0;
    }
    
    .spatial-item:hover {
        background: rgba(26, 188, 156, 0.1);
        z-index: 100 !important;
    }
    
    /* Special styling for tables */
    .spatial-item.table-item {
        background: rgba(26, 188, 156, 0.02);
        border: 1px solid rgba(26, 188, 156, 0.3);
    }
    
    .spatial-item.table-item:hover {
        background: rgba(26, 188, 156, 0.05);
    }
    
    /* Missing table placeholder */
    .spatial-item.missing-table-item {
        background: rgba(230, 126, 34, 0.1);
        border: 2px dashed rgba(230, 126, 34, 0.5);
        color: #E67E22;
        font-style: italic;
        padding: 20px;
        text-align: center;
    }
    
    .spatial-item:focus {
        outline: 2px solid #1ABC9C;
        outline-offset: 2px;
        z-index: 101 !important;
    }
    
    .header-item {
        color: #1ABC9C;
        font-weight: bold;
    }
    
    .form-label {
        color: #FFB84D;
        font-weight: 500;
    }
    
    .form-value {
        background: rgba(26, 188, 156, 0.05);
        border-bottom: 1px dotted #666;
    }
    
    .form-row-hint {
        position: absolute;
        left: 0;
        right: 0;
        height: 1px;
        background: rgba(26, 188, 156, 0.2);
        pointer-events: none;
        z-index: 0;
    }
    
    .page-break {
        margin: 30px 0;
        text-align: center;
        color: #999;
    }
    
    .page-break hr {
        border: none;
        border-top: 2px dashed #666;
        margin: 10px 0;
    }
    
    .debug-info {
        position: absolute;
        right: 5px;
        top: 5px;
        font-size: 10px;
        color: #666;
        background: rgba(0,0,0,0.5);
        padding: 2px 5px;
        border-radius: 3px;
        display: none;
    }
    
    .spatial-page.debug-mode .debug-info {
        display: block;
    }
</style>
'''
    
    def _generate_page_break(self, page_num: int) -> str:
        """Generate page break indicator"""
        return f'''
<div class="page-break">
    <hr>
    <span style="background: #525659; padding: 0 15px; position: relative; top: -12px;">
        Page {page_num}
    </span>
</div>
'''
    
    def _generate_page(self, page_num: int) -> str:
        """Generate a single page with all its items"""
        items = self.layout.get_all_items(page_num)
        if not items:
            return ''
            
        # Calculate page dimensions
        max_right = max(item.bbox.right for item in items) * self.scale + 50
        max_bottom = max(item.bbox.bottom for item in items) * self.scale + 50
        
        html_parts = [
            f'<div class="spatial-page" data-page="{page_num}" '
            f'style="width: {max_right}px; min-height: {max_bottom}px;">'
        ]
        
        # Add debug info
        layout_info = self.layout.analyze_layout(page_num)
        html_parts.append(f'''
            <div class="debug-info">
                Items: {layout_info["total_items"]} | 
                Rows: {layout_info["total_rows"]} | 
                Forms: {layout_info["form_labels"]}
            </div>
        ''')
        
        # Render form row hints (disabled for now)
        # rows = self.layout.get_rows(page_num)
        # for row in rows:
        #     if any(item.is_form_label or item.is_form_value for item in row):
        #         # Add subtle line to show form row
        #         avg_y = sum(item.bbox.center_y for item in row) / len(row) * self.scale
        #         html_parts.append(
        #             f'<div class="form-row-hint" style="top: {avg_y}px;"></div>'
        #         )
        
        # Render all items
        table_count = sum(1 for item in items if item.item_type == 'table')
        if table_count > 0:
            logger.info(f"🐹 Rendering page with {table_count} tables out of {len(items)} total items")
        
        for item in items:
            if item.item_type == 'table':
                logger.debug(f"🐹 Rendering table at {item.bbox}")
            html_parts.append(self._generate_item(item))
            
        html_parts.append('</div>')
        
        return '\n'.join(html_parts)
    
    def _generate_item(self, item: LayoutItem) -> str:
        """Generate HTML for a single item"""
        # Determine CSS classes
        classes = ['spatial-item']
        if item.is_header:
            classes.append('header-item')
        if item.is_form_label:
            classes.append('form-label')
        if item.is_form_value:
            classes.append('form-value')
        if item.item_type == 'table':
            classes.append('table-item')
        if item.item_type == 'missing_table':
            classes.append('missing-table-item')
            
        # Generate attributes
        attrs = [
            f'class="{" ".join(classes)}"',
            f'style="{item.to_html_style(self.scale)}"',
            f'data-type="{item.item_type}"',
            f'data-page="{item.page_num}"'
        ]
        
        # Add debug tooltip
        debug_info = (
            f"Type: {item.item_type} | "
            f"Pos: ({item.bbox.left:.0f},{item.bbox.top:.0f}) | "
            f"Size: {item.bbox.width:.0f}x{item.bbox.height:.0f}"
        )
        attrs.append(f'title="{html.escape(debug_info)}"')
        
        # Escape content
        content = html.escape(item.content)
        
        # Handle different item types
        if item.item_type == 'table':
            # Check if we have vector graphics
            if item.metadata.get('has_vector_graphics'):
                # Content already contains HTML table structure
                content = item.content
            else:
                # For image tables, show them in a special way
                content = self._format_image_table(item)
        elif item.item_type == 'missing_table':
            # Don't escape content for missing tables - it has formatting
            content = item.content.replace('\n', '<br>')
            
        return f'<div {" ".join(attrs)}>{content}</div>'
    
    def _format_table_content(self, content: str) -> str:
        """Format table content for display"""
        # Simple table formatting - could be enhanced
        lines = content.strip().split('\n')
        if len(lines) > 1:
            # Wrap in pre tag to preserve formatting
            return f'<pre style="margin: 0; font-family: inherit;">{content}</pre>'
        return content
    
    def _format_image_table(self, item: LayoutItem) -> str:
        """Format tables that are images"""
        content = item.content
        
        # Don't double-escape if content is already escaped
        if '&lt;' not in content and '&gt;' not in content and '<' in content:
            # Already has HTML, don't escape
            pass
        elif 'table_cells=' in content:
            # It's the object repr that wasn't parsed - shouldn't happen now
            content = "[Table parsing failed - see PDF for details]"
        else:
            content = html.escape(content)
        
        # Always show tables in a consistent way
        return f'''<div style="width: 100%; height: 100%; overflow: auto; background: #2a2a2a; border-radius: 4px; padding: 8px;">
            <div style="color: #1ABC9C; font-size: 11px; margin-bottom: 6px; font-weight: bold;">📊 Table</div>
            <pre style="margin: 0; font-family: 'Courier New', monospace; font-size: 11px; color: #E0E0E0; white-space: pre-wrap; word-break: break-word;">{content}</pre>
        </div>'''