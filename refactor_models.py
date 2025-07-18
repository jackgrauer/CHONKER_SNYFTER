#!/usr/bin/env python3
"""
Pydantic models for tracking CHONKER refactoring
"""

from pydantic import BaseModel, Field
from typing import List, Optional
from enum import Enum


class RefactorConfidence(str, Enum):
    HIGH = "high"
    MEDIUM = "medium"
    LOW = "low"


class RefactorArea(str, Enum):
    FILE_VALIDATION = "file_validation"
    MAPPINGS = "mappings"
    ZOOM_METHODS = "zoom_methods"
    CSS_STYLES = "css_styles"
    EVENT_FILTERS = "event_filters"
    ANIMATIONS = "animations"


class RefactorTarget(BaseModel):
    """Track a specific refactoring target"""
    area: RefactorArea
    description: str
    estimated_lines_saved: int
    confidence: RefactorConfidence
    branch_name: str = Field(default="")
    completed: bool = False
    test_passed: bool = False
    
    def get_branch_name(self) -> str:
        """Generate branch name with timestamp"""
        from datetime import datetime
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        return f"refactor_{self.area.value}_{timestamp}"


class SafetyProtocol(BaseModel):
    """Safety measures for refactoring"""
    test_before_changes: bool = True
    use_branches: bool = True
    ready_to_rollback: bool = True
    main_branch: str = "main"
    test_command: str = "source venv/bin/activate && python -m pytest tests/ -x"
    

class RefactorPlan(BaseModel):
    """Complete refactoring plan"""
    targets: List[RefactorTarget] = [
        RefactorTarget(
            area=RefactorArea.FILE_VALIDATION,
            description="Consolidate repetitive file size validation into single method",
            estimated_lines_saved=30,
            confidence=RefactorConfidence.HIGH
        ),
        RefactorTarget(
            area=RefactorArea.MAPPINGS,
            description="Convert verbose superscript/subscript mappings to comprehensions",
            estimated_lines_saved=30,
            confidence=RefactorConfidence.HIGH
        ),
        RefactorTarget(
            area=RefactorArea.EVENT_FILTERS,
            description="Replace conditional chains with handler dictionary lookup",
            estimated_lines_saved=40,
            confidence=RefactorConfidence.HIGH
        ),
        RefactorTarget(
            area=RefactorArea.ZOOM_METHODS,
            description="Consolidate zoom_in/zoom_out into single zoom method",
            estimated_lines_saved=15,
            confidence=RefactorConfidence.MEDIUM
        ),
        RefactorTarget(
            area=RefactorArea.CSS_STYLES,
            description="Use CSS inheritance to reduce style repetition",
            estimated_lines_saved=20,
            confidence=RefactorConfidence.MEDIUM
        ),
        RefactorTarget(
            area=RefactorArea.ANIMATIONS,
            description="Simplify processing animation updates",
            estimated_lines_saved=10,
            confidence=RefactorConfidence.LOW
        )
    ]
    safety_protocol: SafetyProtocol = SafetyProtocol()
    total_estimated_savings: int = Field(default=0)
    
    def __init__(self, **data):
        super().__init__(**data)
        self.total_estimated_savings = sum(t.estimated_lines_saved for t in self.targets)
    
    def get_next_target(self) -> Optional[RefactorTarget]:
        """Get next uncompleted target"""
        for target in self.targets:
            if not target.completed:
                return target
        return None
    
    def mark_complete(self, area: RefactorArea, test_passed: bool):
        """Mark a target as complete"""
        for target in self.targets:
            if target.area == area:
                target.completed = True
                target.test_passed = test_passed
                break
    
    def get_status_report(self) -> str:
        """Get current refactoring status"""
        completed = sum(1 for t in self.targets if t.completed)
        saved = sum(t.estimated_lines_saved for t in self.targets if t.completed and t.test_passed)
        return f"Refactoring Progress: {completed}/{len(self.targets)} targets, {saved} lines saved"