
"""
SNYFTER Data Models
Pydantic models for qualitative analysis features
"""

from pydantic import BaseModel, Field, validator
from typing import List, Optional, Dict
from datetime import datetime
from enum import Enum
import uuid


class CodeColor(str, Enum):
    """Predefined code colors"""
    TURQUOISE = "#1ABC9C"
    EMERALD = "#2ECC71"
    BLUE = "#3498DB"
    PURPLE = "#9B59B6"
    YELLOW = "#F1C40F"
    ORANGE = "#E67E22"
    RED = "#E74C3C"
    GRAY = "#95A5A6"


class QualitativeCode(BaseModel):
    """Model for qualitative analysis codes"""
    code_id: str = Field(default_factory=lambda: str(uuid.uuid4()))
    name: str = Field(..., min_length=1, max_length=100)
    description: Optional[str] = Field(None, max_length=500)
    color: CodeColor = Field(default=CodeColor.TURQUOISE)
    parent_id: Optional[str] = Field(None)
    created_date: datetime = Field(default_factory=datetime.now)
    modified_date: Optional[datetime] = None
    
    @validator('name')
    def validate_name(cls, v):
        if not v.strip():
            raise ValueError('Code name cannot be empty')
        return v.strip()


class Annotation(BaseModel):
    """Model for text annotations"""
    annotation_id: str = Field(default_factory=lambda: str(uuid.uuid4()))
    document_id: str = Field(...)
    code_id: str = Field(...)
    start_pos: int = Field(..., ge=0)
    end_pos: int = Field(..., gt=0)
    highlighted_text: str = Field(...)
    memo: Optional[str] = Field(None, max_length=2000)
    created_date: datetime = Field(default_factory=datetime.now)
    created_by: str = Field(default="user")
    
    @validator('end_pos')
    def validate_positions(cls, v, values):
        if 'start_pos' in values and v <= values['start_pos']:
            raise ValueError('End position must be greater than start position')
        return v
    
    @validator('highlighted_text')
    def validate_highlighted_text(cls, v):
        if not v.strip():
            raise ValueError('Highlighted text cannot be empty')
        return v


class CodeTree(BaseModel):
    """Model for hierarchical code structure"""
    root_codes: List[QualitativeCode] = Field(default_factory=list)
    child_map: Dict[str, List[QualitativeCode]] = Field(default_factory=dict)
    
    def add_code(self, code: QualitativeCode):
        """Add a code to the tree"""
        if code.parent_id is None:
            self.root_codes.append(code)
        else:
            if code.parent_id not in self.child_map:
                self.child_map[code.parent_id] = []
            self.child_map[code.parent_id].append(code)
    
    def get_all_codes(self) -> List[QualitativeCode]:
        """Get flat list of all codes"""
        all_codes = list(self.root_codes)
        for children in self.child_map.values():
            all_codes.extend(children)
        return all_codes
    
    def get_code_by_id(self, code_id: str) -> Optional[QualitativeCode]:
        """Find a code by ID"""
        for code in self.get_all_codes():
            if code.code_id == code_id:
                return code
        return None


class AnnotatedDocument(BaseModel):
    """Model for document with annotations"""
    document_id: str = Field(...)
    document_path: str = Field(...)
    title: str = Field(...)
    annotations: List[Annotation] = Field(default_factory=list)
    last_modified: datetime = Field(default_factory=datetime.now)
    
    def add_annotation(self, annotation: Annotation):
        """Add annotation ensuring no overlaps"""
        # Check for overlaps
        for existing in self.annotations:
            if (annotation.start_pos < existing.end_pos and 
                annotation.end_pos > existing.start_pos):
                raise ValueError(f"Annotation overlaps with existing annotation {existing.annotation_id}")
        
        self.annotations.append(annotation)
        self.last_modified = datetime.now()
    
    def get_annotations_by_code(self, code_id: str) -> List[Annotation]:
        """Get all annotations for a specific code"""
        return [a for a in self.annotations if a.code_id == code_id]
    
    def remove_annotation(self, annotation_id: str) -> bool:
        """Remove an annotation by ID"""
        for i, ann in enumerate(self.annotations):
            if ann.annotation_id == annotation_id:
                self.annotations.pop(i)
                self.last_modified = datetime.now()
                return True
        return False


class SNYFTERProject(BaseModel):
    """Model for complete SNYFTER project"""
    project_id: str = Field(default_factory=lambda: str(uuid.uuid4()))
    name: str = Field(..., min_length=1, max_length=200)
    description: Optional[str] = Field(None)
    code_tree: CodeTree = Field(default_factory=CodeTree)
    documents: Dict[str, AnnotatedDocument] = Field(default_factory=dict)
    created_date: datetime = Field(default_factory=datetime.now)
    last_saved: Optional[datetime] = None
    
    def add_document(self, doc: AnnotatedDocument):
        """Add a document to the project"""
        self.documents[doc.document_id] = doc
        
    def get_all_annotations(self) -> List[Annotation]:
        """Get all annotations across all documents"""
        annotations = []
        for doc in self.documents.values():
            annotations.extend(doc.annotations)
        return annotations
    
    def export_codebook(self) -> Dict[str, Any]:
        """Export code structure as dictionary"""
        return {
            "project_name": self.name,
            "codes": [code.dict() for code in self.code_tree.get_all_codes()],
            "total_annotations": len(self.get_all_annotations()),
            "export_date": datetime.now().isoformat()
        }
