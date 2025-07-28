"""Export document data to Parquet format"""

import pandas as pd
from pathlib import Path
from datetime import datetime
from typing import Dict, Any, Optional
import json

from ..models.document import Document
from ..models.layout_item import LayoutItem
from ..extraction.spatial_layout import SpatialLayoutEngine


class ParquetExporter:
    """Export document and layout data to Parquet files"""
    
    def __init__(self, document: Document, layout: SpatialLayoutEngine, 
                 original_html: str = "", edited_html: str = ""):
        """
        Initialize the exporter.
        
        Args:
            document: The document to export
            layout: The spatial layout engine
            original_html: Original HTML before edits
            edited_html: HTML after edits
        """
        self.document = document
        self.layout = layout
        self.original_html = original_html
        self.edited_html = edited_html
        
    def export(self, output_dir: str, qc_user: str = "user") -> Dict[str, Path]:
        """
        Export to Parquet files.
        
        Args:
            output_dir: Directory to save Parquet files
            qc_user: User who performed QC
            
        Returns:
            Dictionary mapping table names to file paths
        """
        output_path = Path(output_dir)
        output_path.mkdir(parents=True, exist_ok=True)
        
        # Generate export ID
        export_id = self.document.generate_export_id()
        
        # Export each table
        files = {}
        
        # 1. Export metadata
        files['exports'] = self._export_metadata(output_path, export_id, qc_user)
        
        # 2. Export content
        files['content'] = self._export_content(output_path, export_id)
        
        # 3. Export styles
        files['styles'] = self._export_styles(output_path, export_id)
        
        # 4. Export semantics
        files['semantics'] = self._export_semantics(output_path, export_id)
        
        return files
        
    def _export_metadata(self, output_path: Path, export_id: str, qc_user: str) -> Path:
        """Export metadata table"""
        metadata = {
            'export_id': [export_id],
            'export_timestamp': [datetime.now()],
            'source_pdf': [self.document.source_path],
            'original_html': [self.original_html],
            'edited_html': [self.edited_html],
            'qc_user': [qc_user],
            'edit_count': [self.document.get_edit_count()],
            'page_count': [len(self.document.pages)],
            'total_items': [sum(len(p.items) for p in self.document.pages.values())]
        }
        
        df = pd.DataFrame(metadata)
        file_path = output_path / 'chonker_exports.parquet'
        df.to_parquet(file_path, index=False, compression='snappy')
        
        return file_path
        
    def _export_content(self, output_path: Path, export_id: str) -> Path:
        """Export content table"""
        chunks = self.document.to_chunks()
        
        # Add export ID and layout info
        for chunk in chunks:
            chunk['export_id'] = export_id
            
            # Get layout info for this item
            page_num = chunk['page']
            item_idx = chunk['index']
            
            if page_num in self.layout.pages:
                items = self.layout.pages[page_num]
                if item_idx < len(items):
                    item = items[item_idx]
                    chunk['row_group'] = item.row_group
                    chunk['is_form_label'] = item.is_form_label
                    chunk['is_form_value'] = item.is_form_value
                    
        df = pd.DataFrame(chunks)
        
        # Ensure proper column types
        numeric_cols = ['bbox_left', 'bbox_top', 'bbox_right', 'bbox_bottom']
        for col in numeric_cols:
            if col in df.columns:
                df[col] = pd.to_numeric(df[col], errors='coerce')
                
        file_path = output_path / 'chonker_content.parquet'
        df.to_parquet(file_path, index=False, compression='snappy')
        
        return file_path
        
    def _export_styles(self, output_path: Path, export_id: str) -> Path:
        """Export styles table"""
        style_records = []
        
        for page in self.document.pages.values():
            for idx, item in enumerate(page.items):
                if item.style:
                    record = {
                        'export_id': export_id,
                        'element_id': f"{page.page_num}_{idx}",
                        'page': page.page_num,
                        'style_bold': item.style.get('bold', False),
                        'style_italic': item.style.get('italic', False),
                        'font_size': item.style.get('font_size'),
                        'font_name': item.style.get('font_name'),
                        'color': item.style.get('color')
                    }
                    style_records.append(record)
                    
        # Create at least one record if no styles found
        if not style_records:
            style_records.append({
                'export_id': export_id,
                'element_id': 'none',
                'page': 0,
                'style_bold': False,
                'style_italic': False,
                'font_size': None,
                'font_name': None,
                'color': None
            })
            
        df = pd.DataFrame(style_records)
        file_path = output_path / 'chonker_styles.parquet'
        df.to_parquet(file_path, index=False, compression='snappy')
        
        return file_path
        
    def _export_semantics(self, output_path: Path, export_id: str) -> Path:
        """Export semantic analysis table"""
        semantic_records = []
        
        for page in self.document.pages.values():
            for idx, item in enumerate(page.items):
                # Determine semantic role
                semantic_role = self._get_semantic_role(item)
                
                record = {
                    'export_id': export_id,
                    'element_id': f"{page.page_num}_{idx}",
                    'page': page.page_num,
                    'semantic_role': semantic_role,
                    'word_count': len(item.content.split()),
                    'char_count': len(item.content),
                    'confidence': 0.9 if semantic_role != 'unknown' else 0.5,
                    'is_header': item.is_header,
                    'is_form_field': item.is_form_label or item.is_form_value
                }
                semantic_records.append(record)
                
        df = pd.DataFrame(semantic_records)
        file_path = output_path / 'chonker_semantics.parquet'
        df.to_parquet(file_path, index=False, compression='snappy')
        
        return file_path
        
    def _get_semantic_role(self, item: LayoutItem) -> str:
        """Determine semantic role of an item"""
        content_lower = item.content.lower()
        
        # Headers
        if item.is_header:
            return 'header'
            
        # Form fields
        if item.is_form_label:
            return 'form_label'
        if item.is_form_value:
            return 'form_value'
            
        # Financial text detection
        financial_keywords = ['$', 'usd', 'revenue', 'income', 'expense', 'profit', 
                            'loss', 'total', 'amount', 'balance', 'cost']
        if any(kw in content_lower for kw in financial_keywords):
            return 'financial_text'
            
        # Tables
        if item.item_type == 'table':
            return 'data_table'
            
        # Default
        return 'body_text' if len(item.content) > 50 else 'short_text'