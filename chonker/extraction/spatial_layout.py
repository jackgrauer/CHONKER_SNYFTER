"""Spatial layout engine for managing item positioning and preventing overlaps"""

from typing import List, Dict, Tuple, Optional
from collections import defaultdict
import logging

from ..models.bbox import BoundingBox
from ..models.layout_item import LayoutItem


logger = logging.getLogger(__name__)


class SpatialLayoutEngine:
    """
    Manages spatial layout of items, preventing overlaps and organizing form-like structures.
    """
    
    def __init__(self, grid_size: int = 5, min_spacing: int = 10):
        """
        Initialize the layout engine.
        
        Args:
            grid_size: Snap items to this grid size for alignment
            min_spacing: Minimum spacing between items
        """
        self.grid_size = grid_size
        self.min_spacing = min_spacing
        self.pages: Dict[int, List[LayoutItem]] = defaultdict(list)
        
    def add_item(self, item: LayoutItem) -> LayoutItem:
        """
        Add an item to the layout, resolving any overlaps.
        
        Args:
            item: The layout item to add
            
        Returns:
            The item with potentially adjusted position
        """
        # Snap to grid for better alignment
        item = self._snap_to_grid(item)
        
        # Get items on the same page
        page_items = self.pages[item.page_num]
        
        # Check for overlaps and resolve them
        for existing in page_items:
            if self._items_overlap(item, existing):
                logger.debug(f"Overlap detected: '{item.content[:30]}' with '{existing.content[:30]}'")
                item = self._resolve_overlap(item, existing, page_items)
        
        # Detect row grouping
        item.row_group = self._find_row_group(item, page_items)
        
        # Detect form patterns
        self._detect_form_pattern(item, page_items)
        
        page_items.append(item)
        return item
    
    def _snap_to_grid(self, item: LayoutItem) -> LayoutItem:
        """Snap item position to grid for better alignment"""
        # Only snap Y coordinate to help with row alignment
        snapped_top = round(item.bbox.top / self.grid_size) * self.grid_size
        
        if abs(snapped_top - item.bbox.top) < self.grid_size / 2:
            # Adjust bbox with snapped position
            height = item.bbox.height
            item.bbox.top = snapped_top
            item.bbox.bottom = snapped_top + height
            
        return item
    
    def _items_overlap(self, item1: LayoutItem, item2: LayoutItem) -> bool:
        """Check if two items overlap significantly"""
        # Check with negative tolerance to require small gap
        return item1.bbox.overlaps(item2.bbox, tolerance=-2)
    
    def _resolve_overlap(self, new_item: LayoutItem, existing_item: LayoutItem, 
                        page_items: List[LayoutItem]) -> LayoutItem:
        """
        Resolve overlap between items intelligently.
        """
        v_overlap = new_item.bbox.vertical_overlap(existing_item.bbox)
        h_overlap = new_item.bbox.horizontal_overlap(existing_item.bbox)
        
        # If items are mostly on the same line (high vertical overlap)
        if v_overlap > 0.7:
            # Shift horizontally
            new_item.bbox.left = existing_item.bbox.right + self.min_spacing
            new_item.bbox.right = new_item.bbox.left + new_item.bbox.width
            
            # Check if we pushed it into another item
            for other in page_items:
                if other != existing_item and self._items_overlap(new_item, other):
                    # Try shifting down instead
                    new_item.bbox.top = existing_item.bbox.bottom + self.grid_size
                    new_item.bbox.bottom = new_item.bbox.top + new_item.bbox.height
                    # Reset horizontal position
                    new_item.bbox.left = new_item.bbox.left - (existing_item.bbox.right + self.min_spacing) + new_item.bbox.left
                    new_item.bbox.right = new_item.bbox.left + new_item.bbox.width
                    break
        else:
            # Shift vertically
            new_item.bbox.top = existing_item.bbox.bottom + self.grid_size
            new_item.bbox.bottom = new_item.bbox.top + new_item.bbox.height
        
        return new_item
    
    def _find_row_group(self, item: LayoutItem, page_items: List[LayoutItem]) -> int:
        """Find or assign a row group for items on the same line"""
        tolerance = self.grid_size * 2  # Allow some vertical variance
        
        for existing in page_items:
            if abs(existing.bbox.center_y - item.bbox.center_y) < tolerance:
                if existing.row_group is not None:
                    return existing.row_group
        
        # Assign new row group
        max_group = max((i.row_group or 0 for i in page_items), default=0)
        return max_group + 1
    
    def _detect_form_pattern(self, item: LayoutItem, page_items: List[LayoutItem]) -> None:
        """Detect if this item is part of a form label-value pattern"""
        if item.ends_with_colon and item.is_short_text:
            # This looks like a form label
            item.is_form_label = True
            
            # Look for items to the right that might be values
            same_row_items = [
                i for i in page_items 
                if i.row_group == item.row_group and i.bbox.left > item.bbox.right
            ]
            
            if same_row_items:
                # Mark the next item as a form value
                next_item = min(same_row_items, key=lambda i: i.bbox.left)
                next_item.is_form_value = True
    
    def get_rows(self, page_num: int) -> List[List[LayoutItem]]:
        """
        Get items organized by rows for a specific page.
        
        Returns:
            List of rows, each row is a list of items sorted by X position
        """
        page_items = self.pages[page_num]
        
        # Group by row
        rows = defaultdict(list)
        for item in page_items:
            if item.row_group:
                rows[item.row_group].append(item)
            else:
                # Items without row group go in their own row
                rows[f"single_{id(item)}"].append(item)
        
        # Sort each row by X position and return as list
        sorted_rows = []
        for row_key in sorted(rows.keys(), key=lambda k: min(i.bbox.top for i in rows[k])):
            row_items = sorted(rows[row_key], key=lambda i: i.bbox.left)
            sorted_rows.append(row_items)
            
        return sorted_rows
    
    def get_all_items(self, page_num: Optional[int] = None) -> List[LayoutItem]:
        """Get all items, optionally filtered by page"""
        if page_num is not None:
            return list(self.pages[page_num])
        
        all_items = []
        for page_items in self.pages.values():
            all_items.extend(page_items)
        return all_items
    
    def analyze_layout(self, page_num: int) -> Dict[str, any]:
        """Analyze the layout structure of a page"""
        items = self.pages[page_num]
        rows = self.get_rows(page_num)
        
        return {
            'total_items': len(items),
            'total_rows': len(rows),
            'form_labels': sum(1 for i in items if i.is_form_label),
            'form_values': sum(1 for i in items if i.is_form_value),
            'headers': sum(1 for i in items if i.is_header),
            'avg_items_per_row': len(items) / len(rows) if rows else 0
        }