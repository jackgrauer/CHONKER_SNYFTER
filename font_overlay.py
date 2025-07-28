#!/usr/bin/env python3
"""
Font Overlay - Extract font information using PyMuPDF and match to Docling items by position
"""

import fitz  # PyMuPDF
from dataclasses import dataclass
from typing import Dict, List, Tuple, Optional
import sys

@dataclass
class FontInfo:
    """Font information from PyMuPDF"""
    font_name: str
    font_size: float
    flags: int  # Bold, italic, etc.
    color: int
    bbox: Tuple[float, float, float, float]  # (x0, y0, x1, y1)
    text: str
    
    @property
    def is_bold(self) -> bool:
        return bool(self.flags & 2**4)  # Bit 4 indicates bold
    
    @property
    def is_italic(self) -> bool:
        return bool(self.flags & 2**1)  # Bit 1 indicates italic

def extract_font_info(pdf_path: str) -> Dict[int, List[FontInfo]]:
    """Extract font information from PDF using PyMuPDF"""
    doc = fitz.open(pdf_path)
    font_data = {}
    
    for page_num in range(len(doc)):
        page = doc[page_num]
        font_data[page_num] = []
        
        # Extract text with detailed font info
        blocks = page.get_text("dict")
        
        for block in blocks.get("blocks", []):
            if block.get("type") == 0:  # Text block
                for line in block.get("lines", []):
                    for span in line.get("spans", []):
                        font_info = FontInfo(
                            font_name=span.get("font", ""),
                            font_size=span.get("size", 0),
                            flags=span.get("flags", 0),
                            color=span.get("color", 0),
                            bbox=(span["bbox"][0], span["bbox"][1], 
                                  span["bbox"][2], span["bbox"][3]),
                            text=span.get("text", "")
                        )
                        font_data[page_num].append(font_info)
    
    doc.close()
    return font_data

def match_font_to_docling_bbox(font_bbox: Tuple[float, float, float, float],
                               docling_bbox: Dict,
                               page_height: float = 792) -> float:
    """
    Calculate overlap between PyMuPDF bbox and Docling bbox
    Returns overlap ratio (0-1)
    """
    # Convert Docling bbox (bottom-left origin) to top-left origin
    docling_x0 = docling_bbox.get('l', 0)
    docling_x1 = docling_bbox.get('r', 0)
    docling_y0_bottom = docling_bbox.get('b', 0)
    docling_y1_bottom = docling_bbox.get('t', 0)
    
    # Convert to top-left origin
    docling_y0 = page_height - docling_y1_bottom
    docling_y1 = page_height - docling_y0_bottom
    
    # Calculate intersection
    x_overlap = max(0, min(font_bbox[2], docling_x1) - max(font_bbox[0], docling_x0))
    y_overlap = max(0, min(font_bbox[3], docling_y1) - max(font_bbox[1], docling_y0))
    
    intersection = x_overlap * y_overlap
    
    # Calculate union
    font_area = (font_bbox[2] - font_bbox[0]) * (font_bbox[3] - font_bbox[1])
    docling_area = (docling_x1 - docling_x0) * (docling_y1 - docling_y0)
    union = font_area + docling_area - intersection
    
    return intersection / union if union > 0 else 0

def get_dominant_font_for_bbox(font_infos: List[FontInfo], 
                              docling_bbox: Dict,
                              threshold: float = 0.5) -> Optional[FontInfo]:
    """Find the dominant font within a Docling bounding box"""
    matching_fonts = []
    
    for font_info in font_infos:
        overlap = match_font_to_docling_bbox(font_info.bbox, docling_bbox)
        if overlap > threshold:
            matching_fonts.append(font_info)
    
    if not matching_fonts:
        return None
    
    # Find most common font size
    font_sizes = {}
    for font in matching_fonts:
        key = (font.font_name, font.font_size, font.is_bold, font.is_italic)
        font_sizes[key] = font_sizes.get(key, 0) + len(font.text)
    
    # Return the font style with the most characters
    dominant = max(font_sizes.items(), key=lambda x: x[1])
    font_name, font_size, is_bold, is_italic = dominant[0]
    
    # Find a matching font info to return
    for font in matching_fonts:
        if (font.font_name == font_name and font.font_size == font_size and
            font.is_bold == is_bold and font.is_italic == is_italic):
            return font
    
    return matching_fonts[0]  # Fallback

# Example integration with Chonker
def enhance_docling_with_fonts(docling_result, pdf_path: str):
    """Add font information to Docling items"""
    font_data = extract_font_info(pdf_path)
    
    items = list(docling_result.document.iterate_items())
    for idx, (item, level) in enumerate(items):
        if hasattr(item, 'prov') and item.prov:
            prov = item.prov[0]
            if hasattr(prov, 'bbox') and hasattr(prov, 'page_no'):
                page_num = prov.page_no - 1  # Convert to 0-based
                bbox_dict = {
                    'l': prov.bbox.l,
                    'r': prov.bbox.r,
                    't': prov.bbox.t,
                    'b': prov.bbox.b
                }
                
                if page_num in font_data:
                    dominant_font = get_dominant_font_for_bbox(
                        font_data[page_num], 
                        bbox_dict
                    )
                    
                    if dominant_font:
                        # Add font info to item (this is a hack - better to use proper structure)
                        item._font_info = {
                            'font_name': dominant_font.font_name,
                            'font_size': dominant_font.font_size,
                            'is_bold': dominant_font.is_bold,
                            'is_italic': dominant_font.is_italic
                        }
                        print(f"Added font info to item {idx}: {item._font_info}")

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python font_overlay.py <pdf_path>")
        sys.exit(1)
    
    pdf_path = sys.argv[1]
    font_data = extract_font_info(pdf_path)
    
    print(f"Extracted font data for {len(font_data)} pages")
    for page_num, fonts in font_data.items():
        print(f"\nPage {page_num + 1}: {len(fonts)} text spans")
        # Show unique font styles
        unique_styles = set()
        for font in fonts:
            style = f"{font.font_name} {font.font_size}pt"
            if font.is_bold:
                style += " Bold"
            if font.is_italic:
                style += " Italic"
            unique_styles.add(style)
        
        print("Unique font styles:")
        for style in sorted(unique_styles):
            print(f"  - {style}")