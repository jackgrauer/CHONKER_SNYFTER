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
        Calculate appropriate font size based on bounding box height.
        Assumes typical line height is ~1.4x font size.
        """
        # Base calculation
        font_size = self.bbox.height / 1.4
        
        # Apply limits
        min_size = 8 if self.is_header else 6
        max_size = 24
        
        return max(min_size, min(font_size, max_size))
    
    def to_html_style(self, scale: float = 1.2) -> str:
        """Generate inline CSS style for this item"""
        screen_coords = self.bbox.to_screen(scale)
        font_size = self.calculate_font_size()
        
        styles = [
            f"position: absolute",
            f"left: {screen_coords['left']:.1f}px",
            f"top: {screen_coords['top']:.1f}px",
            f"width: {screen_coords['width']:.1f}px",
            f"min-height: {screen_coords['height']:.1f}px",
            f"font-size: {font_size:.1f}px",
            f"line-height: 1.2",
            f"z-index: 2"
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