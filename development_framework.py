#!/usr/bin/env python3
"""
Development Framework - Using Instructor & Guardrails to build better code
This is for Claude to use when developing CHONKER & SNYFTER
"""

from pydantic import BaseModel, Field, validator
from typing import List, Optional, Dict, Any, Literal
from enum import Enum
import json


class DevelopmentPhase(str, Enum):
    UNDERSTANDING = "understanding"
    DESIGN = "design"
    IMPLEMENTATION = "implementation"
    DEBUGGING = "debugging"
    REFACTORING = "refactoring"


class UserIntent(BaseModel):
    """What the user ACTUALLY wants"""
    stated_request: str
    true_intent: str
    success_criteria: List[str]
    non_goals: List[str] = Field(default_factory=list, description="What user does NOT want")
    
    @validator('true_intent')
    def must_be_different_from_stated(cls, v, values):
        if v == values.get('stated_request'):
            raise ValueError("Must dig deeper to find true intent")
        return v


class CharacterPersonality(BaseModel):
    """Define character traits that affect code behavior"""
    name: Literal["CHONKER", "SNYFTER"]
    emoji: str
    personality_traits: List[str]
    speaking_style: str
    core_motivation: str
    implementation_implications: List[str] = Field(
        description="How personality affects code design"
    )


class CodeDecision(BaseModel):
    """Structured decision about implementation"""
    decision: str
    rationale: str
    alternatives_considered: List[str]
    character_alignment: str = Field(description="How this fits character personality")
    technical_debt_accepted: Optional[str] = None
    
    @validator('character_alignment')
    def must_reference_character(cls, v):
        if not any(char in v.upper() for char in ["CHONKER", "SNYFTER"]):
            raise ValueError("Decision must align with character")
        return v


class ImplementationPlan(BaseModel):
    """Structured plan for implementing a feature"""
    feature: str
    current_phase: DevelopmentPhase
    
    # Understanding
    user_intent: UserIntent
    character_context: CharacterPersonality
    
    # Design
    design_decisions: List[CodeDecision]
    ui_elements: Dict[str, str] = Field(description="UI element -> character-appropriate design")
    message_templates: Dict[str, List[str]] = Field(description="Action -> character messages")
    
    # Implementation
    files_to_modify: List[str]
    code_structure: Dict[str, str] = Field(description="Component -> implementation approach")
    error_handling_style: str = Field(description="How errors are presented in character")
    
    # Quality checks
    must_have_features: List[str]
    nice_to_have_features: List[str]
    bugs_to_avoid: List[str]
    
    def validate_character_consistency(self) -> bool:
        """Ensure all elements align with character"""
        character_name = self.character_context.name
        
        # Check UI elements
        for element, design in self.ui_elements.items():
            if character_name not in design:
                return False
        
        # Check messages
        for action, messages in self.message_templates.items():
            if not all(character_name in msg or self.character_context.emoji in msg for msg in messages):
                return False
        
        return True


class BugAnalysis(BaseModel):
    """Structured approach to debugging"""
    symptom: str
    user_report: str
    
    # Analysis
    likely_cause: str
    investigation_steps: List[str]
    found_issue: Optional[str] = None
    
    # Fix
    fix_approach: str
    files_affected: List[str]
    character_impact: str = Field(description="How fix maintains character personality")
    
    # Validation
    test_cases: List[str]
    regression_risks: List[str]


class DevelopmentSession(BaseModel):
    """Complete development session structure"""
    session_goal: str
    context_from_user: List[str] = Field(description="Key quotes from user")
    
    # Current understanding
    chonker_state: Dict[str, Any] = Field(description="What CHONKER can do now")
    snyfter_state: Dict[str, Any] = Field(description="What SNYFTER can do now")
    
    # Work items
    implementation_plans: List[ImplementationPlan] = Field(default_factory=list)
    bugs_found: List[BugAnalysis] = Field(default_factory=list)
    
    # Decisions made
    design_decisions: List[CodeDecision] = Field(default_factory=list)
    character_moments: List[str] = Field(
        default_factory=list,
        description="Moments where character personality shines"
    )
    
    # Quality metrics
    user_satisfaction_indicators: List[str] = Field(default_factory=list)
    technical_debt_notes: List[str] = Field(default_factory=list)
    
    def add_user_feedback(self, feedback: str):
        """Process user feedback into structured understanding"""
        self.context_from_user.append(feedback)
        
        # Analyze for satisfaction
        if any(word in feedback.lower() for word in ["perfect", "love", "exactly"]):
            self.user_satisfaction_indicators.append(f"Positive: {feedback}")
        elif any(word in feedback.lower() for word in ["no", "not", "wrong"]):
            self.user_satisfaction_indicators.append(f"Needs work: {feedback}")


# CHONKER & SNYFTER Development Profiles
CHONKER_PROFILE = CharacterPersonality(
    name="CHONKER",
    emoji="🐹",
    personality_traits=[
        "Enthusiastic about eating PDFs",
        "Makes happy hamster noises",
        "Sometimes gets indigestion from scuzzy PDFs",
        "Stores things in cheek pouches"
    ],
    speaking_style="Excited, food-focused, onomatopoeia (*nom nom*)",
    core_motivation="Turn inedible PDFs into delicious, digestible content",
    implementation_implications=[
        "Error messages should be about tummy aches, not technical errors",
        "Processing messages use eating metaphors",
        "UI should feel warm and chunky",
        "Success is celebrated with burps and satisfaction"
    ]
)

SNYFTER_PROFILE = CharacterPersonality(
    name="SNYFTER",
    emoji="🐁",
    personality_traits=[
        "Meticulous librarian",
        "Wears tiny glasses",
        "Obsessed with proper cataloging",
        "Speaks formally but kindly"
    ],
    speaking_style="Precise, formal, helpful, uses library metaphors",
    core_motivation="Preserve knowledge in perfectly organized archives",
    implementation_implications=[
        "UI should feel like a card catalog",
        "Messages reference filing, cataloging, archiving",
        "Errors are about misfiling or catalog issues",
        "Success is quiet satisfaction of proper organization"
    ]
)


# Example: Current Development Session
def create_current_session() -> DevelopmentSession:
    """Structure current development understanding"""
    
    session = DevelopmentSession(
        session_goal="Build character-driven document processing app",
        context_from_user=[
            "i just want to use it to develop the end product",
            "it is imperative that you procure the correct hamster and mouse emoji",
            "chonker is a chubby hamster... snyfter is the skinny hypothyroidal mouse"
        ],
        chonker_state={
            "working": ["PDF loading", "Basic extraction", "De-scuzzifying"],
            "personality": "Partially implemented (messages exist)",
            "missing": "Full character integration in errors"
        },
        snyfter_state={
            "working": ["Database storage", "Search function"],
            "personality": "Good messages, needs more library metaphors",
            "missing": "Card catalog UI feeling"
        }
    )
    
    # Current implementation plan
    plan = ImplementationPlan(
        feature="Character-driven error handling",
        current_phase=DevelopmentPhase.IMPLEMENTATION,
        user_intent=UserIntent(
            stated_request="make error handling better",
            true_intent="errors should feel like they come from the characters, not generic tech",
            success_criteria=[
                "CHONKER errors talk about tummy aches",
                "SNYFTER errors reference filing problems",
                "No technical jargon unless wrapped in character"
            ]
        ),
        character_context=CHONKER_PROFILE,
        design_decisions=[
            CodeDecision(
                decision="Wrap all exceptions in character-specific error classes",
                rationale="Every error should feel in-character",
                alternatives_considered=["Generic errors with character messages", "No error wrapping"],
                character_alignment="CHONKER can't digest technical errors, only PDF problems"
            )
        ],
        ui_elements={
            "error_dialog": "Looks like a hamster food bowl with error message",
            "progress_bar": "Shows CHONKER's belly getting fuller"
        },
        message_templates={
            "file_not_found": [
                "🐹 *sniff sniff* I can't find that PDF!",
                "🐹 Did someone move my food? Can't find the file!"
            ],
            "extraction_failed": [
                "🐹 *cough cough* This PDF is too scuzzy for me!",
                "🐹 Oof, my tummy hurts. This PDF needs preprocessing!"
            ]
        },
        files_to_modify=["chonker_snyfter.py"],
        code_structure={
            "ChonkerError": "Custom exception with hamster personality",
            "SnyfterError": "Custom exception with librarian personality"
        },
        error_handling_style="All technical errors translated to character terms",
        must_have_features=[
            "Character-appropriate messages",
            "No technical jargon exposed",
            "Helpful suggestions in character"
        ],
        nice_to_have_features=[
            "Animated error displays",
            "Sound effects"
        ],
        bugs_to_avoid=[
            "Mixing character voices",
            "Technical errors leaking through",
            "Breaking character in edge cases"
        ]
    )
    
    session.implementation_plans.append(plan)
    return session


# Development Guidelines
class DevelopmentGuidelines(BaseModel):
    """Rules for maintaining quality while developing"""
    
    character_consistency_rules: List[str] = [
        "Every message must sound like it comes from the character",
        "UI elements should reflect character personality",
        "Technical terms must be translated to character-appropriate language",
        "Error handling should never break character"
    ]
    
    code_quality_rules: List[str] = [
        "Each character gets their own class/module",
        "Personality traits drive implementation decisions",
        "Comments should reference character motivations",
        "Test cases should verify character consistency"
    ]
    
    user_experience_rules: List[str] = [
        "User should smile when seeing character messages",
        "Errors should be helpful despite being in-character",
        "Workflow should feel natural to each character",
        "Tab-switching should feel like visiting different personalities"
    ]


if __name__ == "__main__":
    print("=== CHONKER & SNYFTER Development Framework ===\n")
    
    session = create_current_session()
    print(f"Session Goal: {session.session_goal}\n")
    
    print("Character Profiles:")
    print(f"\n{CHONKER_PROFILE.emoji} CHONKER:")
    print(f"  Core: {CHONKER_PROFILE.core_motivation}")
    print(f"  Style: {CHONKER_PROFILE.speaking_style}")
    
    print(f"\n{SNYFTER_PROFILE.emoji} SNYFTER:")
    print(f"  Core: {SNYFTER_PROFILE.core_motivation}")
    print(f"  Style: {SNYFTER_PROFILE.speaking_style}")
    
    print("\n=== Current Implementation Plan ===")
    plan = session.implementation_plans[0]
    print(f"Feature: {plan.feature}")
    print(f"True Intent: {plan.user_intent.true_intent}")
    print("\nMessage Templates:")
    for action, messages in plan.message_templates.items():
        print(f"\n{action}:")
        for msg in messages:
            print(f"  {msg}")
    
    print("\n=== Development Guidelines ===")
    guidelines = DevelopmentGuidelines()
    print("\nCharacter Consistency:")
    for rule in guidelines.character_consistency_rules[:2]:
        print(f"  • {rule}")
    
    print("\n✅ This framework ensures every line of code serves the characters!")