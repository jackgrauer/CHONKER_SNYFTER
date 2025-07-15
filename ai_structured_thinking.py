#!/usr/bin/env python3
"""
AI Structured Thinking Framework using Instructor and Guardrails
"""

import instructor
from pydantic import BaseModel, Field
from typing import List, Optional, Dict, Any
from enum import Enum
import openai
from guardrails import Guard
from guardrails.hub import DetectPII, ToxicLanguage
import json


class TaskPriority(str, Enum):
    CRITICAL = "critical"
    HIGH = "high"
    MEDIUM = "medium"
    LOW = "low"


class CodeQuality(str, Enum):
    PRODUCTION = "production"
    PROTOTYPE = "prototype"
    EXPERIMENTAL = "experimental"


class DesignDecision(BaseModel):
    """Structured representation of a design decision"""
    decision: str = Field(description="The decision being made")
    rationale: str = Field(description="Why this decision was made")
    alternatives_considered: List[str] = Field(description="Other options that were considered")
    trade_offs: List[str] = Field(description="Trade-offs of this decision")


class SystemComponent(BaseModel):
    """Structured representation of a system component"""
    name: str = Field(description="Component name")
    purpose: str = Field(description="What this component does")
    dependencies: List[str] = Field(description="What this component depends on")
    interfaces: List[str] = Field(description="How other components interact with this")
    implementation_notes: str = Field(description="Key implementation details")


class UserRequirement(BaseModel):
    """Structured parsing of user requirements"""
    core_need: str = Field(description="The fundamental need expressed")
    functional_requirements: List[str] = Field(description="What the system must do")
    non_functional_requirements: List[str] = Field(description="How the system should behave")
    constraints: List[str] = Field(description="Limitations or boundaries")
    success_criteria: List[str] = Field(description="How to measure success")


class ProjectAnalysis(BaseModel):
    """Complete structured analysis of a project"""
    project_name: str = Field(description="Name of the project")
    project_goal: str = Field(description="Primary objective")
    user_requirements: UserRequirement
    system_components: List[SystemComponent]
    design_decisions: List[DesignDecision]
    implementation_plan: List[str] = Field(description="Ordered steps to implement")
    risks: List[str] = Field(description="Potential risks and challenges")
    quality_target: CodeQuality


class ChonkerAnalysis(BaseModel):
    """Specific analysis for the CHONKER project"""
    current_state: str = Field(description="What CHONKER is right now")
    desired_state: str = Field(description="What the user wants CHONKER to be")
    gaps: List[str] = Field(description="Differences between current and desired state")
    core_features: List[str] = Field(description="Essential features that must work")
    nice_to_haves: List[str] = Field(description="Features that would be good but not critical")
    technical_approach: str = Field(description="Overall technical strategy")
    
    # Specific to CHONKER's mission
    pdf_processing_strategy: str = Field(description="How to handle PDF extraction")
    database_design: str = Field(description="How to structure the database")
    ui_approach: str = Field(description="User interface strategy")
    bidirectional_selection_plan: str = Field(description="How to implement PDF<->content selection")


class ImplementationTask(BaseModel):
    """Structured task for implementation"""
    task_id: str
    description: str
    priority: TaskPriority
    dependencies: List[str] = Field(default_factory=list)
    acceptance_criteria: List[str]
    estimated_complexity: int = Field(ge=1, le=10, description="1=trivial, 10=very complex")
    implementation_notes: str = Field(default="")


class QualityCheck(BaseModel):
    """Structured quality assessment"""
    aspect: str = Field(description="What aspect is being checked")
    status: str = Field(description="Current status")
    issues: List[str] = Field(description="Problems found")
    recommendations: List[str] = Field(description="How to fix issues")


def analyze_chonker_requirements() -> ChonkerAnalysis:
    """Analyze CHONKER requirements with structured thinking"""
    
    # Based on user's stated requirements:
    # 1. PDF-to-editable-document converter
    # 2. Extract content and edit in WYSIWYG
    # 3. Save to extensible SQL database
    # 4. PDF viewing alongside extracted content
    # 5. Bidirectional selection (click PDF -> highlight HTML)
    # 6. Handle "scuzzy" PDFs with preprocessing
    # 7. Human-in-the-loop QC workflow
    # 8. Open PDF -> Extract -> QC -> Export to DB -> Recall later
    
    analysis = ChonkerAnalysis(
        current_state="Basic Qt app with PDF viewer and content extraction, saves to SQLite",
        desired_state="Robust PDF extraction system with bidirectional selection and quality control",
        gaps=[
            "Missing bidirectional selection between PDF and extracted content",
            "No visual highlighting/synchronization between views",
            "Limited PDF preprocessing capabilities",
            "Basic QC interface needs enhancement",
            "No recall/search functionality for database",
            "No confidence scoring for extraction quality"
        ],
        core_features=[
            "PDF viewing with page navigation",
            "Content extraction using Docling",
            "SQLite database storage",
            "Side-by-side PDF and content views",
            "Human review before database save"
        ],
        nice_to_haves=[
            "OCR for image-based PDFs",
            "Batch processing multiple PDFs",
            "Export to multiple formats",
            "Advanced search in database",
            "Extraction confidence visualization"
        ],
        technical_approach="PyQt6 desktop app with modular components for PDF processing, extraction, and database management",
        pdf_processing_strategy="Use PyMuPDF for preprocessing (remove annotations, check quality) then Docling for extraction",
        database_design="Normalized schema with documents, content, chunks, and tables; uses JSON for flexible metadata",
        ui_approach="Split-pane interface with PDF on left, extracted content on right, tabs for different views",
        bidirectional_selection_plan="Track bounding boxes from Docling, implement click handlers in PDF view to find corresponding chunks, use highlighting in both views"
    )
    
    return analysis


def generate_implementation_plan(analysis: ChonkerAnalysis) -> List[ImplementationTask]:
    """Generate structured implementation tasks"""
    
    tasks = [
        ImplementationTask(
            task_id="BIDIR-001",
            description="Implement PDF click detection and coordinate mapping",
            priority=TaskPriority.HIGH,
            dependencies=[],
            acceptance_criteria=[
                "Can detect mouse clicks on PDF view",
                "Can convert click coordinates to PDF page coordinates",
                "Can identify which text/element was clicked"
            ],
            estimated_complexity=7,
            implementation_notes="Use QPdfView's mouse events and coordinate transformation"
        ),
        ImplementationTask(
            task_id="BIDIR-002", 
            description="Add chunk highlighting in extracted content view",
            priority=TaskPriority.HIGH,
            dependencies=["BIDIR-001"],
            acceptance_criteria=[
                "Can highlight specific chunks in HTML/Markdown views",
                "Highlight is visually distinct",
                "Can clear previous highlights"
            ],
            estimated_complexity=5,
            implementation_notes="Use QTextEdit's formatting capabilities or custom HTML/CSS"
        ),
        ImplementationTask(
            task_id="BIDIR-003",
            description="Connect PDF selection to content highlighting",
            priority=TaskPriority.HIGH,
            dependencies=["BIDIR-001", "BIDIR-002"],
            acceptance_criteria=[
                "Clicking PDF highlights corresponding content",
                "Clicking content highlights PDF region",
                "Smooth scrolling to highlighted content"
            ],
            estimated_complexity=8,
            implementation_notes="Map Docling bbox data to PDF coordinates"
        ),
        ImplementationTask(
            task_id="QC-001",
            description="Enhanced QC interface with confidence scoring",
            priority=TaskPriority.MEDIUM,
            dependencies=[],
            acceptance_criteria=[
                "Show extraction confidence per chunk",
                "Allow user to mark chunks as correct/incorrect",
                "Calculate overall document quality score"
            ],
            estimated_complexity=6
        ),
        ImplementationTask(
            task_id="PDF-001",
            description="Advanced PDF preprocessing for difficult documents",
            priority=TaskPriority.MEDIUM,
            dependencies=[],
            acceptance_criteria=[
                "Detect and fix common PDF issues",
                "Option to run OCR on image-based pages",
                "Preview preprocessing results"
            ],
            estimated_complexity=7
        ),
        ImplementationTask(
            task_id="DB-001",
            description="Database recall and search functionality",
            priority=TaskPriority.MEDIUM,
            dependencies=[],
            acceptance_criteria=[
                "Search documents by filename, content, date",
                "Load previous extractions for review",
                "Show extraction history"
            ],
            estimated_complexity=4
        )
    ]
    
    return tasks


def create_quality_guards():
    """Create guardrails for code quality"""
    
    # Define validation rules
    code_quality_guard = Guard.from_string(
        validators=[
            # Could add custom validators here
        ],
        description="Ensure code quality and safety"
    )
    
    return code_quality_guard


def structured_approach_summary():
    """Generate a structured summary of the approach"""
    
    summary = {
        "mission": "Create a robust PDF extraction and QC system",
        "approach": {
            "phase_1": "Fix current implementation gaps",
            "phase_2": "Add bidirectional selection",
            "phase_3": "Enhance QC workflow",
            "phase_4": "Improve PDF preprocessing"
        },
        "key_principles": [
            "Keep it focused on core mission",
            "Ensure data integrity with proper database design",
            "Make QC process efficient for humans",
            "Handle edge cases in PDF processing"
        ],
        "success_metrics": [
            "Can extract content from 95%+ of PDFs",
            "Bidirectional selection works smoothly",
            "Human QC takes < 2 min per document",
            "All extractions stored in searchable database"
        ]
    }
    
    return summary


if __name__ == "__main__":
    # Demonstrate structured thinking
    print("=== CHONKER Project Analysis ===\n")
    
    analysis = analyze_chonker_requirements()
    print("Current State:", analysis.current_state)
    print("\nDesired State:", analysis.desired_state)
    print("\nGaps to Address:")
    for gap in analysis.gaps:
        print(f"  - {gap}")
    
    print("\n=== Implementation Plan ===\n")
    tasks = generate_implementation_plan(analysis)
    for task in tasks:
        print(f"{task.task_id}: {task.description}")
        print(f"  Priority: {task.priority.value}")
        print(f"  Complexity: {task.estimated_complexity}/10")
        print()
    
    print("\n=== Structured Approach ===")
    summary = structured_approach_summary()
    print(json.dumps(summary, indent=2))