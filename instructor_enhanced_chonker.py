#!/usr/bin/env python3
"""
Instructor-Enhanced CHONKER - Using instructor for structured LLM outputs
"""

import instructor
from openai import OpenAI
from pydantic import BaseModel, Field, validator
from typing import List, Optional, Dict, Any, Literal
from enum import Enum
import json


# Initialize instructor with OpenAI
client = instructor.from_openai(OpenAI())


class DocumentQuality(str, Enum):
    PRISTINE = "pristine"
    GOOD = "good"
    SCUZZY = "scuzzy"
    VERY_SCUZZY = "very_scuzzy"


class ChonkerAnalysis(BaseModel):
    """What CHONKER thinks about a PDF"""
    
    quality: DocumentQuality
    issues_found: List[str] = Field(description="Specific issues CHONKER detected")
    preprocessing_needed: List[str] = Field(description="Steps CHONKER will take to de-scuzzify")
    hamster_opinion: str = Field(description="CHONKER's personal take on this PDF")
    digestibility_score: int = Field(ge=1, le=10, description="1=inedible, 10=delicious")
    
    @validator('hamster_opinion')
    def must_be_hamster_like(cls, v):
        if not any(word in v.lower() for word in ['nom', 'munch', 'tasty', 'yum', 'chewy', 'crunchy']):
            raise ValueError("CHONKER must express opinion in hamster terms!")
        return v


class SnyfterCataloging(BaseModel):
    """How SNYFTER will catalog a document"""
    
    classification: str = Field(description="Dewey Decimal-style classification")
    keywords: List[str] = Field(max_items=10, description="Key terms for the card catalog")
    cross_references: List[str] = Field(description="Related documents in the archive")
    librarian_notes: str = Field(description="SNYFTER's meticulous observations")
    storage_location: Literal["main_stacks", "rare_books", "reference", "periodicals"]
    
    @validator('librarian_notes')
    def must_be_precise(cls, v):
        if len(v) < 20:
            raise ValueError("SNYFTER requires more detailed notes!")
        return v


class ExtractionStrategy(BaseModel):
    """Structured extraction plan"""
    
    extraction_method: Literal["docling", "pymupdf", "ocr", "hybrid"]
    confidence_areas: List[Dict[str, Any]] = Field(
        description="Areas where extraction will be reliable"
    )
    problem_areas: List[Dict[str, Any]] = Field(
        description="Areas that need human review"
    )
    recommended_qc_focus: List[str] = Field(
        description="What the human should pay attention to"
    )


def analyze_pdf_with_chonker(pdf_path: str) -> ChonkerAnalysis:
    """Let CHONKER analyze a PDF using instructor-structured output"""
    
    # In a real implementation, we'd analyze the PDF and send to LLM
    # For demo, showing what instructor would return:
    
    analysis = client.chat.completions.create(
        model="gpt-4",
        response_model=ChonkerAnalysis,
        messages=[
            {
                "role": "system", 
                "content": "You are CHONKER, a chubby hamster who loves eating PDFs. Analyze this PDF like a hamster would."
            },
            {
                "role": "user",
                "content": f"Analyze this PDF: {pdf_path}"
            }
        ]
    )
    
    # Instructor ensures the response ALWAYS matches our schema
    return analysis


def snyfter_classification(content: str) -> SnyfterCataloging:
    """SNYFTER classifies content with instructor validation"""
    
    cataloging = client.chat.completions.create(
        model="gpt-4",
        response_model=SnyfterCataloging,
        messages=[
            {
                "role": "system",
                "content": "You are SNYFTER, a meticulous librarian mouse. Classify this document."
            },
            {
                "role": "user",
                "content": f"Catalog this content: {content[:500]}..."
            }
        ]
    )
    
    return cataloging


class ChonkerSnyfterPipeline(BaseModel):
    """Complete processing pipeline with validation"""
    
    # Input validation
    pdf_path: str
    pdf_size_mb: float = Field(le=100.0, description="Max 100MB for CHONKER's tummy")
    
    # CHONKER's analysis
    chonker_analysis: Optional[ChonkerAnalysis] = None
    preprocessing_applied: List[str] = Field(default_factory=list)
    
    # Extraction results
    extraction_strategy: Optional[ExtractionStrategy] = None
    extracted_chunks: int = Field(default=0, ge=0)
    extraction_time_seconds: float = Field(default=0.0, ge=0.0)
    
    # SNYFTER's cataloging
    snyfter_cataloging: Optional[SnyfterCataloging] = None
    database_id: Optional[int] = None
    
    # Quality checks
    human_qc_required: bool = Field(default=True)
    qc_notes: List[str] = Field(default_factory=list)
    
    @validator('pdf_size_mb')
    def chonker_can_handle(cls, v):
        if v > 50:
            raise ValueError("🐹 CHONKER says: That's too big for my cheeks!")
        return v
    
    def add_qc_note(self, note: str):
        """Add a QC note with validation"""
        if len(note) < 10:
            raise ValueError("QC notes must be descriptive")
        self.qc_notes.append(note)
    
    def mark_ready_for_archive(self) -> bool:
        """Check if ready for SNYFTER's permanent collection"""
        return all([
            self.chonker_analysis is not None,
            self.extraction_strategy is not None,
            self.snyfter_cataloging is not None,
            self.extracted_chunks > 0,
            len(self.qc_notes) > 0
        ])


# Example of using guardrails for code generation
def generate_extraction_code(strategy: ExtractionStrategy) -> str:
    """Generate extraction code with guardrails validation"""
    
    from guardrails import Guard
    from guardrails.validators import (
        ValidPython,
        NoImportStatements,
        ContainsSubstring
    )
    
    # Define guardrails for generated code
    guard = Guard.from_pydantic(
        output_class=str,
        validators=[
            ValidPython(on_fail="fix"),  # Ensure valid Python
            ContainsSubstring(
                substring="try:", 
                on_fail="fix"
            ),  # Must have error handling
            NoImportStatements(on_fail="filter")  # No new imports
        ]
    )
    
    # Generate code with guardrails
    code = guard(
        client.chat.completions.create,
        model="gpt-4",
        messages=[{
            "role": "user",
            "content": f"Generate extraction code for strategy: {strategy.model_dump_json()}"
        }]
    )
    
    return code


# Structured error handling
class ChonkerError(BaseModel):
    """Structured error for CHONKER issues"""
    
    error_type: Literal["indigestion", "too_scuzzy", "unknown_format", "too_big"]
    hamster_message: str
    technical_details: str
    suggested_remedy: str
    
    def display(self) -> str:
        return f"🐹 {self.hamster_message}\n💻 {self.technical_details}\n💡 {self.suggested_remedy}"


class SnyfterError(BaseModel):
    """Structured error for SNYFTER issues"""
    
    error_type: Literal["duplicate", "invalid_metadata", "storage_full", "classification_unclear"]
    mouse_message: str
    catalog_reference: Optional[str] = None
    remediation_steps: List[str]
    
    def display(self) -> str:
        steps = "\n".join(f"  {i+1}. {step}" for i, step in enumerate(self.remediation_steps))
        return f"🐁 {self.mouse_message}\n📚 Reference: {self.catalog_reference or 'N/A'}\n📝 Steps:\n{steps}"


# Example usage showing the power of instructor
def process_document_with_instructor(pdf_path: str):
    """Complete document processing with structured outputs"""
    
    # Initialize pipeline with validation
    pipeline = ChonkerSnyfterPipeline(
        pdf_path=pdf_path,
        pdf_size_mb=os.path.getsize(pdf_path) / 1024 / 1024
    )
    
    try:
        # CHONKER analyzes with structured output
        pipeline.chonker_analysis = analyze_pdf_with_chonker(pdf_path)
        
        # Generate extraction strategy
        pipeline.extraction_strategy = client.chat.completions.create(
            model="gpt-4",
            response_model=ExtractionStrategy,
            messages=[{
                "role": "user",
                "content": f"Plan extraction for: {pipeline.chonker_analysis.model_dump_json()}"
            }]
        )
        
        # Generate and validate extraction code
        extraction_code = generate_extraction_code(pipeline.extraction_strategy)
        
        # Execute extraction (simplified)
        # ... extraction happens here ...
        pipeline.extracted_chunks = 42
        
        # SNYFTER catalogs with validation
        pipeline.snyfter_cataloging = snyfter_classification("extracted content here")
        
        # Structured output for human QC
        qc_guidance = client.chat.completions.create(
            model="gpt-4",
            response_model=List[str],
            messages=[{
                "role": "user",
                "content": f"What should human check? {pipeline.model_dump_json()}"
            }]
        )
        
        return pipeline
        
    except Exception as e:
        # Even errors are structured!
        if "chonker" in str(e).lower():
            error = ChonkerError(
                error_type="indigestion",
                hamster_message="*cough* This PDF gave me a tummy ache!",
                technical_details=str(e),
                suggested_remedy="Try preprocessing with PyMuPDF first"
            )
        else:
            error = SnyfterError(
                error_type="classification_unclear",
                mouse_message="*adjusts glasses nervously* Cannot classify this document",
                catalog_reference="ERROR-2024-001",
                remediation_steps=[
                    "Check document metadata",
                    "Verify extraction completed",
                    "Consult senior librarian mouse"
                ]
            )
        
        raise error


if __name__ == "__main__":
    # Demo output showing structured thinking
    print("=== INSTRUCTOR-ENHANCED CHONKER & SNYFTER ===\n")
    
    # Show what structured outputs look like
    demo_analysis = ChonkerAnalysis(
        quality=DocumentQuality.SCUZZY,
        issues_found=["Annotations everywhere", "Weird encoding", "Upside down pages"],
        preprocessing_needed=["Remove annotations", "Fix encoding", "Rotate pages"],
        hamster_opinion="This PDF is pretty chewy but I can nom through it!",
        digestibility_score=6
    )
    
    print("CHONKER's Structured Analysis:")
    print(json.dumps(demo_analysis.model_dump(), indent=2))
    
    demo_cataloging = SnyfterCataloging(
        classification="004.5/PDF-2024",
        keywords=["technical", "documentation", "scuzzy", "needs-qc"],
        cross_references=["DOC-2024-001", "DOC-2024-002"],
        librarian_notes="Document shows signs of heavy annotation. Extraction confidence moderate. Filed in technical section pending human review.",
        storage_location="main_stacks"
    )
    
    print("\n\nSNYFTER's Structured Cataloging:")
    print(json.dumps(demo_cataloging.model_dump(), indent=2))