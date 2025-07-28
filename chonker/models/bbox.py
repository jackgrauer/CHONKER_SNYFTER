"""Bounding box model with coordinate normalization"""

from dataclasses import dataclass
from typing import Tuple, Dict, Optional


@dataclass
class BoundingBox:
    """
    Normalized bounding box using top-left coordinate system.
    All coordinates are stored in PDF points (72 DPI).
    """
    left: float
    top: float
    right: float
    bottom: float
    
    def __post_init__(self):
        """Ensure coordinates are valid"""
        if self.right < self.left:
            self.left, self.right = self.right, self.left
        if self.bottom < self.top:
            self.top, self.bottom = self.bottom, self.top
    
    @classmethod
    def from_pdf_coords(cls, l: float, t: float, r: float, b: float, page_height: float) -> 'BoundingBox':
        """
        Convert PDF coordinates (bottom-left origin) to normalized top-left coordinates.
        
        Args:
            l, t, r, b: Left, top, right, bottom in PDF coordinates
            page_height: Height of the PDF page in points
        """
        # In PDF coords, 'top' is distance from bottom, so we flip it
        return cls(
            left=l,
            top=page_height - t,  # Convert to distance from top
            right=r,
            bottom=page_height - b
        )
    
    @classmethod
    def from_docling_bbox(cls, docling_bbox, page_height: float = 792) -> 'BoundingBox':
        """Create from Docling bbox object"""
        if hasattr(docling_bbox, 'coord_origin') and str(docling_bbox.coord_origin) == 'CoordOrigin.BOTTOMLEFT':
            # Docling uses bottom-left origin
            return cls.from_pdf_coords(
                docling_bbox.l,
                docling_bbox.t,
                docling_bbox.r,
                docling_bbox.b,
                page_height
            )
        else:
            # Already in top-left
            return cls(
                left=docling_bbox.l,
                top=docling_bbox.t,
                right=docling_bbox.r,
                bottom=docling_bbox.b
            )
    
    @property
    def width(self) -> float:
        """Width of the bounding box"""
        return self.right - self.left
    
    @property
    def height(self) -> float:
        """Height of the bounding box"""
        return self.bottom - self.top
    
    @property
    def center_x(self) -> float:
        """X coordinate of center"""
        return (self.left + self.right) / 2
    
    @property
    def center_y(self) -> float:
        """Y coordinate of center"""
        return (self.top + self.bottom) / 2
    
    def to_screen(self, scale: float = 1.2) -> Dict[str, float]:
        """
        Convert to screen coordinates for HTML rendering.
        
        Args:
            scale: Scale factor (default 1.2 for better readability)
            
        Returns:
            Dictionary with 'left', 'top', 'width', 'height' in pixels
        """
        return {
            'left': self.left * scale,
            'top': self.top * scale,
            'width': self.width * scale,
            'height': self.height * scale
        }
    
    def overlaps(self, other: 'BoundingBox', tolerance: float = 0) -> bool:
        """
        Check if this bbox overlaps with another.
        
        Args:
            other: Another bounding box
            tolerance: Additional spacing to consider (negative for requiring gap)
        """
        return not (
            self.right + tolerance < other.left or
            other.right + tolerance < self.left or
            self.bottom + tolerance < other.top or
            other.bottom + tolerance < self.top
        )
    
    def vertical_overlap(self, other: 'BoundingBox') -> float:
        """
        Calculate vertical overlap percentage with another bbox.
        
        Returns:
            0.0 = no overlap, 1.0 = complete overlap
        """
        overlap_top = max(self.top, other.top)
        overlap_bottom = min(self.bottom, other.bottom)
        
        if overlap_bottom <= overlap_top:
            return 0.0
            
        overlap_height = overlap_bottom - overlap_top
        min_height = min(self.height, other.height)
        
        return overlap_height / min_height if min_height > 0 else 0.0
    
    def horizontal_overlap(self, other: 'BoundingBox') -> float:
        """
        Calculate horizontal overlap percentage with another bbox.
        
        Returns:
            0.0 = no overlap, 1.0 = complete overlap
        """
        overlap_left = max(self.left, other.left)
        overlap_right = min(self.right, other.right)
        
        if overlap_right <= overlap_left:
            return 0.0
            
        overlap_width = overlap_right - overlap_left
        min_width = min(self.width, other.width)
        
        return overlap_width / min_width if min_width > 0 else 0.0
    
    def expand(self, padding: float) -> 'BoundingBox':
        """Return new bbox expanded by padding on all sides"""
        return BoundingBox(
            left=self.left - padding,
            top=self.top - padding,
            right=self.right + padding,
            bottom=self.bottom + padding
        )
    
    def __repr__(self) -> str:
        return f"BBox(l={self.left:.1f}, t={self.top:.1f}, r={self.right:.1f}, b={self.bottom:.1f})"