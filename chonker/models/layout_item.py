"""Layout item model for spatial positioning"""

from dataclasses import dataclass, field
from typing import Optional, Dict, Any
from .bbox import BoundingBox


@dataclass
class LayoutItem:
    """A single item in the spatial layout"""
    bbox: BoundingBox
    content: str
    item_type: str  # 'text', 'header', 'table', etc.
    level: int = 0
    page_num: int = 1
    metadata: Dict[str, Any] = field(default_factory=dict)
    style: Optional[Dict[str, Any]] = None
    
    # Layout-specific attributes
    row_group: Optional[int] = None  # Items on same row share this
    is_form_label: bool = False
    is_form_value: bool = False
    
    @property
    def is_header(self) -> bool:
        """Check if this is a header element"""
        return self.item_type in ['h1', 'h2', 'h3', 'h4', 'h5', 'h6']
    
    @property
    def is_short_text(self) -> bool:
        """Check if this is short text (potential form field)"""
        return len(self.content.strip()) < 50
    
    @property
    def ends_with_colon(self) -> bool:
        """Check if text ends with colon (potential form label)"""
        return self.content.strip().endswith(':')
    
    def calculate_font_size(self) -> float:
        """
        Calculate appropriate font size based on content and available space.
        """
        # Base font size - don't make it depend on bbox height
        # as that causes issues with varying text sizes
        base_size = 12
        
        # Adjust based on type
        if self.is_header:
            if self.item_type == 'h1':
                return 18
            elif self.item_type == 'h2':
                return 16
            elif self.item_type == 'h3':
                return 14
            else:
                return 13
        
        # Small text for short items (likely labels)
        if len(self.content) < 20 and self.ends_with_colon:
            return 11
            
        # Default size for body text
        return base_size
    
    def calculate_min_height(self, font_size: float, width: float) -> float:
        """
        Calculate minimum height needed based on content length.
        """
        # Tables need their full height - don't compress them!
        if self.item_type == 'table':
            # Tables should maintain their original height
            return self.bbox.height
            
        # Estimate characters per line (rough approximation)
        chars_per_line = max(1, int(width / (font_size * 0.6)))
        
        # Calculate number of lines needed
        content_length = len(self.content)
        estimated_lines = max(1, (content_length + chars_per_line - 1) // chars_per_line)
        
        # Height = lines * line-height * font-size
        line_height = 1.4
        min_height = estimated_lines * font_size * line_height
        
        # Add some padding
        return min_height + 8
    
    def to_html_style(self, scale: float = 1.2) -> str:
        """Generate inline CSS style for this item"""
        screen_coords = self.bbox.to_screen(scale)
        font_size = self.calculate_font_size()
        
        # Calculate proper min-height based on content
        min_height = self.calculate_min_height(font_size, screen_coords['width'])
        
        # Use the larger of calculated height or bbox height
        actual_height = max(min_height, screen_coords['height'])
        
        styles = [
            f"position: absolute",
            f"left: {screen_coords['left']:.1f}px",
            f"top: {screen_coords['top']:.1f}px",
            f"width: {screen_coords['width']:.1f}px",
            f"min-height: {actual_height:.1f}px",
            f"font-size: {font_size}px",
            f"line-height: 1.4",
            f"z-index: 2",
            f"overflow: visible",
            f"word-break: normal",
            f"overflow-wrap: break-word",
            f"hyphens: manual"
        ]
        
        # Add style overrides
        if self.style:
            if self.style.get('bold'):
                styles.append("font-weight: bold")
            if self.style.get('italic'):
                styles.append("font-style: italic")
            if self.style.get('color'):
                styles.append(f"color: {self.style['color']}")
        
        # Special styling for headers
        if self.is_header:
            styles.append("color: #1ABC9C")
            styles.append("font-weight: bold")
        
        # Form field styling
        if self.is_form_value:
            styles.append("background: rgba(26, 188, 156, 0.05)")
            styles.append("border-bottom: 1px dotted #666")
        
        return "; ".join(styles)
    
    def __repr__(self) -> str:
        return f"LayoutItem('{self.content[:30]}...', {self.bbox}, type={self.item_type})"