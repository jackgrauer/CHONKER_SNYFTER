"""Document model with edit tracking"""

from dataclasses import dataclass, field
from datetime import datetime
from typing import List, Dict, Optional, Any, Tuple
import hashlib
import json

from .layout_item import LayoutItem


@dataclass
class Edit:
    """Represents a single edit to the document"""
    timestamp: datetime
    page_num: int
    item_index: int
    old_content: str
    new_content: str
    user: str = "user"
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary for serialization"""
        return {
            'timestamp': self.timestamp.isoformat(),
            'page_num': self.page_num,
            'item_index': self.item_index,
            'old_content': self.old_content,
            'new_content': self.new_content,
            'user': self.user
        }


@dataclass 
class Page:
    """Represents a single page in the document"""
    page_num: int
    items: List[LayoutItem] = field(default_factory=list)
    metadata: Dict[str, Any] = field(default_factory=dict)
    
    def get_item(self, index: int) -> Optional[LayoutItem]:
        """Get item by index"""
        if 0 <= index < len(self.items):
            return self.items[index]
        return None
        
    def update_item(self, index: int, new_content: str) -> bool:
        """Update item content by index"""
        if 0 <= index < len(self.items):
            self.items[index].content = new_content
            return True
        return False
        
    def find_item_by_position(self, x: float, y: float) -> Optional[Tuple[int, LayoutItem]]:
        """Find item containing the given position"""
        for idx, item in enumerate(self.items):
            if (item.bbox.left <= x <= item.bbox.right and
                item.bbox.top <= y <= item.bbox.bottom):
                return idx, item
        return None


class Document:
    """Main document model with pages and edit history"""
    
    def __init__(self, source_path: str = "", metadata: Optional[Dict[str, Any]] = None):
        """Initialize document"""
        self.source_path = source_path
        self.metadata = metadata or {}
        self.pages: Dict[int, Page] = {}
        self.edit_history: List[Edit] = []
        self.created_at = datetime.now()
        self.modified_at = datetime.now()
        
    def add_page(self, page: Page) -> None:
        """Add a page to the document"""
        self.pages[page.page_num] = page
        
    def get_page(self, page_num: int) -> Optional[Page]:
        """Get page by number"""
        return self.pages.get(page_num)
        
    def add_item(self, page_num: int, item: LayoutItem) -> None:
        """Add item to a specific page"""
        if page_num not in self.pages:
            self.pages[page_num] = Page(page_num)
        self.pages[page_num].items.append(item)
        
    def apply_edit(self, page_num: int, item_index: int, new_content: str, user: str = "user") -> bool:
        """
        Apply an edit to a specific item.
        
        Returns:
            True if edit was successful, False otherwise
        """
        page = self.get_page(page_num)
        if not page:
            return False
            
        item = page.get_item(item_index)
        if not item:
            return False
            
        # Record edit
        edit = Edit(
            timestamp=datetime.now(),
            page_num=page_num,
            item_index=item_index,
            old_content=item.content,
            new_content=new_content,
            user=user
        )
        self.edit_history.append(edit)
        
        # Apply edit
        page.update_item(item_index, new_content)
        self.modified_at = datetime.now()
        
        return True
        
    def get_edit_count(self) -> int:
        """Get total number of edits"""
        return len(self.edit_history)
        
    def get_recent_edits(self, limit: int = 10) -> List[Edit]:
        """Get most recent edits"""
        return self.edit_history[-limit:] if self.edit_history else []
        
    def has_edits(self) -> bool:
        """Check if document has been edited"""
        return len(self.edit_history) > 0
        
    def generate_export_id(self) -> str:
        """Generate unique export ID"""
        content = f"{self.source_path}_{self.modified_at.isoformat()}"
        return hashlib.sha256(content.encode()).hexdigest()[:16]
        
    def to_chunks(self) -> List[Dict[str, Any]]:
        """Convert all items to chunk format for export"""
        chunks = []
        
        for page_num in sorted(self.pages.keys()):
            page = self.pages[page_num]
            for idx, item in enumerate(page.items):
                chunk = {
                    'content_id': f"{page_num}_{idx}",
                    'page': page_num,
                    'index': idx,
                    'text': item.content,
                    'type': item.item_type,
                    'bbox_left': item.bbox.left,
                    'bbox_top': item.bbox.top,
                    'bbox_right': item.bbox.right,
                    'bbox_bottom': item.bbox.bottom,
                    'metadata': json.dumps(item.metadata),
                    'is_edited': any(
                        e.page_num == page_num and e.item_index == idx 
                        for e in self.edit_history
                    )
                }
                
                # Add style info if available
                if item.style:
                    chunk.update({
                        'font_size': item.style.get('font_size'),
                        'font_name': item.style.get('font_name'),
                        'is_bold': item.style.get('bold', False),
                        'is_italic': item.style.get('italic', False)
                    })
                    
                chunks.append(chunk)
                
        return chunks
        
    def get_statistics(self) -> Dict[str, Any]:
        """Get document statistics"""
        total_items = sum(len(page.items) for page in self.pages.values())
        item_types = {}
        
        for page in self.pages.values():
            for item in page.items:
                item_types[item.item_type] = item_types.get(item.item_type, 0) + 1
                
        return {
            'page_count': len(self.pages),
            'total_items': total_items,
            'edit_count': len(self.edit_history),
            'item_types': item_types,
            'created_at': self.created_at.isoformat(),
            'modified_at': self.modified_at.isoformat()
        }