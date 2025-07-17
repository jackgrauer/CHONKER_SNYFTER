"""
TASK-002: INSTRUCTOR CODE ANALYSIS
Structured analysis of zoom functionality issues using Pydantic models
"""

from typing import List, Dict, Any, Optional
from pydantic import BaseModel, Field
from datetime import datetime
import uuid

# Import our zoom models
from zoom_models import (
    ZoomState, ZoomConfig, GestureEvent, ZoomOperation, UIComponent,
    ZoomManagerState, ZoomDebugInfo, PaneType, ZoomMethod, GestureType, 
    InputSource, create_zoom_manager_state
)


class CodeIssue(BaseModel):
    """Model for individual code issues"""
    issue_id: str = Field(default_factory=lambda: str(uuid.uuid4()))
    line_number: int = Field(description="Line number where issue occurs")
    severity: str = Field(description="Issue severity: critical, high, medium, low")
    category: str = Field(description="Issue category: logic, integration, performance, etc.")
    description: str = Field(description="Detailed issue description")
    code_snippet: str = Field(description="Relevant code snippet")
    root_cause: str = Field(description="Root cause analysis")
    impact: str = Field(description="Impact on zoom functionality")
    
    class Config:
        extra = "forbid"


class FixRecommendation(BaseModel):
    """Model for fix recommendations"""
    fix_id: str = Field(default_factory=lambda: str(uuid.uuid4()))
    issue_id: str = Field(description="ID of the issue this fix addresses")
    priority: str = Field(description="Priority: critical, high, medium, low")
    title: str = Field(description="Short title of the fix")
    implementation_approach: str = Field(description="How to implement the fix")
    code_changes: str = Field(description="Specific code changes needed")
    integration_notes: str = Field(description="How to integrate with Pydantic models")
    risk_assessment: str = Field(description="Risk assessment for this change")
    testing_requirements: str = Field(description="Testing requirements")
    
    class Config:
        extra = "forbid"


class TestCase(BaseModel):
    """Model for test cases"""
    test_id: str = Field(default_factory=lambda: str(uuid.uuid4()))
    test_type: str = Field(description="Type: unit, integration, ui, edge_case")
    test_name: str = Field(description="Name of the test")
    description: str = Field(description="Test description")
    test_steps: List[str] = Field(description="Steps to execute test")
    expected_result: str = Field(description="Expected result")
    preconditions: List[str] = Field(description="Preconditions for test")
    
    class Config:
        extra = "forbid"


class CodeAnalysisResult(BaseModel):
    """Complete code analysis result"""
    analysis_id: str = Field(default_factory=lambda: str(uuid.uuid4()))
    analysis_date: datetime = Field(default_factory=datetime.now)
    target_file: str = Field(description="File being analyzed")
    issues: List[CodeIssue] = Field(description="List of identified issues")
    summary: str = Field(description="Overall analysis summary")
    interaction_conflicts: List[str] = Field(description="Conflicts between zoom methods")
    
    class Config:
        extra = "forbid"


class FixRecommendations(BaseModel):
    """Complete fix recommendations"""
    recommendations_id: str = Field(default_factory=lambda: str(uuid.uuid4()))
    analysis_id: str = Field(description="ID of the analysis this addresses")
    fixes: List[FixRecommendation] = Field(description="List of fix recommendations")
    implementation_order: List[str] = Field(description="Order of implementation by fix_id")
    
    class Config:
        extra = "forbid"


class TestPlan(BaseModel):
    """Complete test plan"""
    plan_id: str = Field(default_factory=lambda: str(uuid.uuid4()))
    analysis_id: str = Field(description="ID of the analysis this tests")
    test_cases: List[TestCase] = Field(description="List of test cases")
    test_execution_order: List[str] = Field(description="Order of test execution by test_id")
    
    class Config:
        extra = "forbid"


# Analysis Functions
def analyze_zoom_code() -> CodeAnalysisResult:
    """Analyze the zoom functionality code and identify issues"""
    
    issues = []
    
    # Issue 1: Conflicting zoom methods in gesture handling
    issues.append(CodeIssue(
        line_number=1655,
        severity="critical",
        category="logic",
        description="Conflicting zoom methods for HTML content in gesture handling",
        code_snippet="""
# For HTML content, use zoomIn/zoomOut
if zoom_delta > 0:
    self.faithful_output.zoomIn()
else:
    self.faithful_output.zoomOut()
""",
        root_cause="Uses QTextEdit.zoomIn()/zoomOut() methods which only affect the document zoom, not the font size. This conflicts with font-based zoom tracking in self.text_zoom variable.",
        impact="Pinch gestures are detected but produce no visible zoom effect because document zoom and font size are not synchronized."
    ))
    
    # Issue 2: Inconsistent zoom implementation between keyboard and gesture
    issues.append(CodeIssue(
        line_number=1835,
        severity="high",
        category="integration",
        description="Keyboard shortcuts use hybrid zoom approach while gestures use document zoom only",
        code_snippet="""
# In zoom_in() method:
font = self.faithful_output.font()
font.setPointSize(self.text_zoom)
self.faithful_output.setFont(font)
self.faithful_output.zoomIn(2)  # Additional zoom for HTML content

# In gesture handler:
if zoom_delta > 0:
    self.faithful_output.zoomIn()
else:
    self.faithful_output.zoomOut()
""",
        root_cause="Keyboard shortcuts combine font size changes with document zoom, while gestures only use document zoom. This creates inconsistent zoom behavior.",
        impact="Keyboard shortcuts work but gestures don't produce visible results, leading to user confusion."
    ))
    
    # Issue 3: Missing zoom state synchronization
    issues.append(CodeIssue(
        line_number=1653,
        severity="high",
        category="logic",
        description="Font size tracking variable not synchronized with actual zoom implementation",
        code_snippet="""
self.text_zoom = new_size
self.log(f"Text zoom: {old_size} → {new_size}")
# For HTML content, use zoomIn/zoomOut
if zoom_delta > 0:
    self.faithful_output.zoomIn()
else:
    self.faithful_output.zoomOut()
""",
        root_cause="The self.text_zoom variable is updated but the actual zoom implementation doesn't use this value. The font size remains unchanged.",
        impact="Internal state tracking becomes inconsistent with actual visual zoom level."
    ))
    
    # Issue 4: Wheel event handling inconsistency
    issues.append(CodeIssue(
        line_number=1697,
        severity="medium",
        category="integration",
        description="Wheel event (Ctrl+scroll) correctly updates font but gestures don't",
        code_snippet="""
# Wheel event handling:
font = self.faithful_output.font()
font.setPointSize(self.text_zoom)
self.faithful_output.setFont(font)
return True
""",
        root_cause="Wheel events properly update the font size after changing self.text_zoom, but gesture events don't apply the same logic.",
        impact="Ctrl+scroll zoom works correctly while pinch gestures don't, creating inconsistent user experience."
    ))
    
    # Issue 5: Event filter performance and complexity
    issues.append(CodeIssue(
        line_number=1617,
        severity="medium",
        category="performance",
        description="Event filter handles multiple zoom methods in a single complex function",
        code_snippet="""
def eventFilter(self, obj, event):
    # Handle native gesture events (pinch zoom on trackpad)
    if event.type() == QEvent.Type.NativeGesture:
        # ... 50+ lines of gesture handling
    elif event.type() == QEvent.Type.Wheel and hasattr(event, 'modifiers'):
        # ... 20+ lines of wheel handling
    # ... more event handling
""",
        root_cause="Single eventFilter method handles multiple zoom approaches without proper separation of concerns.",
        impact="Code is difficult to maintain and debug, increases risk of conflicts between zoom methods."
    ))
    
    # Issue 6: Missing error handling and validation
    issues.append(CodeIssue(
        line_number=1645,
        severity="medium",
        category="logic",
        description="No error handling for zoom operations or validation of zoom ranges",
        code_snippet="""
if abs(zoom_delta) > 0.05:  # Threshold for triggering zoom
    if zoom_delta > 0:
        new_size = min(48, int(self.text_zoom) + 1)
    else:
        new_size = max(8, int(self.text_zoom) - 1)
""",
        root_cause="No validation that zoom operations succeed or that the widget is in a valid state for zooming.",
        impact="Potential for silent failures or unexpected behavior when zoom operations fail."
    ))
    
    interaction_conflicts = [
        "Document zoom (zoomIn/zoomOut) vs Font size zoom (setFont/setPointSize) - gestures use document zoom while keyboard uses font size",
        "self.text_zoom variable tracking vs actual zoom implementation - variable updated but not applied in gestures",
        "Gesture threshold (0.05) vs wheel event threshold (no threshold) - different sensitivity for different input methods",
        "PDF zoom uses factor-based scaling while text zoom uses point-size increments - inconsistent zoom behavior"
    ]
    
    return CodeAnalysisResult(
        target_file="/Users/jack/CHONKER_SNYFTER/chonker_snyfter_elegant_v2.py",
        issues=issues,
        summary="Critical zoom functionality issues identified. Pinch gestures are detected but don't produce visual zoom due to conflicting zoom methods. Keyboard shortcuts work because they use font-based zoom, while gestures use document zoom which doesn't affect HTML content visibility. The code needs refactoring to use consistent zoom methods and integrate with the Pydantic state management models.",
        interaction_conflicts=interaction_conflicts
    )


def generate_fix_recommendations(analysis: CodeAnalysisResult) -> FixRecommendations:
    """Generate prioritized fix recommendations based on analysis"""
    
    fixes = []
    
    # Fix 1: Unify zoom methods (Critical)
    fixes.append(FixRecommendation(
        issue_id=analysis.issues[0].issue_id,  # Conflicting zoom methods
        priority="critical",
        title="Unify zoom methods to use font-based approach for HTML content",
        implementation_approach="Replace document zoom (zoomIn/zoomOut) with font-based zoom in gesture handling. Use the same approach as keyboard shortcuts.",
        code_changes="""
# Replace lines 1655-1659 in eventFilter:
if new_size != int(self.text_zoom):
    old_size = int(self.text_zoom)
    self.text_zoom = new_size
    self.log(f"Text zoom: {old_size} → {new_size}")
    # Apply font-based zoom consistently
    font = self.faithful_output.font()
    font.setPointSize(self.text_zoom)
    self.faithful_output.setFont(font)
""",
        integration_notes="Integrate with ZoomManagerState to track zoom operations and maintain consistency with Pydantic models. Use ZoomOperation to log each zoom change.",
        risk_assessment="Low risk - this aligns gesture behavior with working keyboard shortcuts. No breaking changes to existing functionality.",
        testing_requirements="Test pinch gestures produce visible zoom, verify consistency with keyboard shortcuts, test zoom limits."
    ))
    
    # Fix 2: Implement Pydantic-based zoom manager (High)
    fixes.append(FixRecommendation(
        issue_id=analysis.issues[2].issue_id,  # Missing zoom state synchronization
        priority="high",
        title="Implement ZoomManagerState integration for state tracking",
        implementation_approach="Add ZoomManagerState instance to main application class and use it to track all zoom operations consistently.",
        code_changes="""
# Add to __init__ method:
from zoom_models import create_zoom_manager_state
self.zoom_manager = create_zoom_manager_state()

# Replace direct zoom variable access with manager:
def update_zoom_state(self, pane: PaneType, new_zoom: float, method: ZoomMethod):
    operation = self.zoom_manager.update_zoom(pane, new_zoom, method)
    self.log(f"Zoom operation: {operation.operation_id} - {operation.old_zoom} → {operation.new_zoom}")
    return operation
""",
        integration_notes="Replace self.text_zoom and self.pdf_zoom with zoom_manager.zoom_state properties. This provides structured state management and operation logging.",
        risk_assessment="Medium risk - requires refactoring existing zoom variable access. Need to ensure all zoom operations go through the manager.",
        testing_requirements="Test state consistency, verify zoom operations are logged, test zoom limits are enforced."
    ))
    
    # Fix 3: Separate zoom handling into dedicated methods (High)
    fixes.append(FixRecommendation(
        issue_id=analysis.issues[4].issue_id,  # Event filter complexity
        priority="high",
        title="Refactor eventFilter to use dedicated zoom handling methods",
        implementation_approach="Extract zoom handling logic into separate methods for better maintainability and testing.",
        code_changes="""
def handle_gesture_zoom(self, obj, gesture_event):
    \"\"\"Handle pinch zoom gestures\"\"\"
    zoom_delta = gesture_event.value()
    if abs(zoom_delta) > 0.05:
        if self.active_pane == 'right':
            self._apply_text_zoom(zoom_delta)
        elif self.active_pane == 'left':
            self._apply_pdf_zoom(zoom_delta)

def _apply_text_zoom(self, zoom_delta):
    \"\"\"Apply zoom to text pane using font-based approach\"\"\"
    current_size = int(self.zoom_manager.zoom_state.right_pane_zoom)
    new_size = current_size + (1 if zoom_delta > 0 else -1)
    new_size = max(8, min(48, new_size))
    
    if new_size != current_size:
        self.zoom_manager.update_zoom(PaneType.RIGHT, new_size, ZoomMethod.FONT_SIZE)
        font = self.faithful_output.font()
        font.setPointSize(new_size)
        self.faithful_output.setFont(font)
""",
        integration_notes="Use GestureEvent model to structure event data and ZoomOperation to track results. This provides better separation of concerns.",
        risk_assessment="Low risk - improves code maintainability without changing core functionality. Easier to test and debug.",
        testing_requirements="Test gesture handling isolation, verify zoom operations work correctly, test error handling."
    ))
    
    # Fix 4: Add comprehensive error handling (Medium)
    fixes.append(FixRecommendation(
        issue_id=analysis.issues[5].issue_id,  # Missing error handling
        priority="medium",
        title="Add error handling and validation for zoom operations",
        implementation_approach="Add try-catch blocks around zoom operations and validate widget state before applying zoom.",
        code_changes="""
def _apply_text_zoom(self, zoom_delta):
    \"\"\"Apply zoom to text pane with error handling\"\"\"
    try:
        if not self.faithful_output or not hasattr(self.faithful_output, 'font'):
            raise ValueError("Text widget not available for zoom")
            
        current_size = int(self.zoom_manager.zoom_state.right_pane_zoom)
        new_size = max(8, min(48, current_size + (1 if zoom_delta > 0 else -1)))
        
        if new_size != current_size:
            font = self.faithful_output.font()
            font.setPointSize(new_size)
            self.faithful_output.setFont(font)
            
            operation = self.zoom_manager.update_zoom(PaneType.RIGHT, new_size, ZoomMethod.FONT_SIZE)
            operation.success = True
            self.log(f"Text zoom applied: {current_size} → {new_size}")
            
    except Exception as e:
        self.log(f"Error applying text zoom: {e}")
        # Create failed operation record
        operation = ZoomOperation(
            operation_id=f"zoom_error_{datetime.now().isoformat()}",
            target_pane=PaneType.RIGHT,
            zoom_type=GestureType.ZOOM_IN if zoom_delta > 0 else GestureType.ZOOM_OUT,
            input_source=InputSource.GESTURE,
            old_zoom=current_size,
            new_zoom=current_size,
            method_used=ZoomMethod.FONT_SIZE,
            success=False,
            error_message=str(e)
        )
        self.zoom_manager.operation_history.append(operation)
""",
        integration_notes="Use ZoomOperation model to track both successful and failed operations. This provides better debugging capabilities.",
        risk_assessment="Low risk - adds robustness without changing core functionality. Improves user experience by handling edge cases.",
        testing_requirements="Test error scenarios, verify failed operations are logged, test recovery from errors."
    ))
    
    # Fix 5: Implement zoom method configuration (Medium)
    fixes.append(FixRecommendation(
        issue_id=analysis.issues[1].issue_id,  # Inconsistent zoom implementation
        priority="medium",
        title="Implement configurable zoom methods using ZoomConfig",
        implementation_approach="Use ZoomConfig to define preferred zoom methods and allow runtime configuration of zoom behavior.",
        code_changes="""
# In zoom handling methods:
def _get_zoom_method_for_pane(self, pane: PaneType) -> ZoomMethod:
    \"\"\"Get the preferred zoom method for a pane\"\"\"
    if pane == PaneType.RIGHT:
        return self.zoom_manager.config.preferred_text_method
    else:
        return self.zoom_manager.config.preferred_pdf_method

def _apply_zoom_by_method(self, pane: PaneType, zoom_delta: float):
    \"\"\"Apply zoom using the configured method for the pane\"\"\"
    method = self._get_zoom_method_for_pane(pane)
    
    if method == ZoomMethod.FONT_SIZE and pane == PaneType.RIGHT:
        self._apply_text_zoom(zoom_delta)
    elif method == ZoomMethod.WIDGET_ZOOM and pane == PaneType.LEFT:
        self._apply_pdf_zoom(zoom_delta)
    elif method == ZoomMethod.HYBRID and pane == PaneType.RIGHT:
        self._apply_hybrid_text_zoom(zoom_delta)
""",
        integration_notes="Use ZoomConfig model to make zoom behavior configurable. This allows users to choose their preferred zoom method.",
        risk_assessment="Medium risk - changes default behavior. Need to ensure backward compatibility and proper configuration management.",
        testing_requirements="Test all zoom methods work correctly, verify configuration persistence, test method switching."
    ))
    
    implementation_order = [
        fixes[0].fix_id,  # Unify zoom methods (Critical)
        fixes[1].fix_id,  # Implement Pydantic-based zoom manager (High)
        fixes[2].fix_id,  # Separate zoom handling (High)
        fixes[3].fix_id,  # Add error handling (Medium)
        fixes[4].fix_id,  # Implement zoom method configuration (Medium)
    ]
    
    return FixRecommendations(
        analysis_id=analysis.analysis_id,
        fixes=fixes,
        implementation_order=implementation_order
    )


def create_test_plan(analysis: CodeAnalysisResult) -> TestPlan:
    """Create comprehensive test plan for zoom functionality"""
    
    test_cases = []
    
    # Unit Tests
    test_cases.append(TestCase(
        test_type="unit",
        test_name="test_font_based_zoom_calculation",
        description="Test font size calculation for zoom operations",
        test_steps=[
            "Create ZoomManagerState instance",
            "Set initial text zoom to 12pt",
            "Call zoom_in operation",
            "Verify new zoom is 14pt",
            "Call zoom_out operation",
            "Verify zoom returns to 12pt"
        ],
        expected_result="Font size calculations are correct and within valid range (8-48pt)",
        preconditions=["ZoomManagerState initialized", "Text pane active"]
    ))
    
    test_cases.append(TestCase(
        test_type="unit",
        test_name="test_zoom_state_tracking",
        description="Test zoom state is properly tracked and updated",
        test_steps=[
            "Initialize ZoomManagerState",
            "Perform zoom operation",
            "Verify operation is logged in history",
            "Verify state is updated correctly",
            "Check timestamp is recorded"
        ],
        expected_result="All zoom operations are tracked with correct state changes",
        preconditions=["ZoomManagerState initialized"]
    ))
    
    # Integration Tests
    test_cases.append(TestCase(
        test_type="integration",
        test_name="test_gesture_to_zoom_pipeline",
        description="Test complete pipeline from gesture detection to zoom application",
        test_steps=[
            "Simulate pinch gesture event",
            "Verify gesture is detected in eventFilter",
            "Verify zoom delta is calculated correctly",
            "Verify font size is applied to widget",
            "Verify zoom state is updated"
        ],
        expected_result="Pinch gestures produce visible zoom changes in text content",
        preconditions=["Application running", "Text pane active", "HTML content loaded"]
    ))
    
    test_cases.append(TestCase(
        test_type="integration",
        test_name="test_keyboard_gesture_consistency",
        description="Test keyboard shortcuts and gestures produce same zoom results",
        test_steps=[
            "Set text zoom to 12pt",
            "Use Ctrl+Plus to zoom in",
            "Record resulting font size",
            "Reset to 12pt",
            "Use pinch gesture to zoom in",
            "Compare font sizes"
        ],
        expected_result="Keyboard shortcuts and gestures produce identical zoom results",
        preconditions=["Application running", "Text pane active"]
    ))
    
    # UI Tests
    test_cases.append(TestCase(
        test_type="ui",
        test_name="test_visual_zoom_verification",
        description="Test zoom changes are visually apparent in UI",
        test_steps=[
            "Load HTML content in text pane",
            "Capture screenshot at default zoom",
            "Perform zoom in operation",
            "Capture screenshot at zoomed state",
            "Compare screenshots for size differences"
        ],
        expected_result="Text content is visually larger/smaller after zoom operations",
        preconditions=["Application running", "HTML content loaded"]
    ))
    
    test_cases.append(TestCase(
        test_type="ui",
        test_name="test_pane_focus_zoom_targeting",
        description="Test zoom operations target the correct pane based on focus",
        test_steps=[
            "Focus on left pane (PDF)",
            "Perform zoom operation",
            "Verify only PDF zoom changes",
            "Focus on right pane (text)",
            "Perform zoom operation",
            "Verify only text zoom changes"
        ],
        expected_result="Zoom operations only affect the active pane",
        preconditions=["Application running", "Both panes with content"]
    ))
    
    # Edge Case Tests
    test_cases.append(TestCase(
        test_type="edge_case",
        test_name="test_zoom_limits_enforcement",
        description="Test zoom operations respect minimum and maximum limits",
        test_steps=[
            "Set text zoom to minimum (8pt)",
            "Attempt to zoom out further",
            "Verify zoom stays at 8pt",
            "Set text zoom to maximum (48pt)",
            "Attempt to zoom in further",
            "Verify zoom stays at 48pt"
        ],
        expected_result="Zoom operations respect configured limits and don't exceed bounds",
        preconditions=["ZoomConfig with proper limits"]
    ))
    
    test_cases.append(TestCase(
        test_type="edge_case",
        test_name="test_error_recovery",
        description="Test system handles zoom errors gracefully",
        test_steps=[
            "Simulate widget not available error",
            "Attempt zoom operation",
            "Verify error is logged",
            "Verify operation marked as failed",
            "Verify system continues to function"
        ],
        expected_result="Zoom errors are handled gracefully without crashing",
        preconditions=["Error simulation capability"]
    ))
    
    test_cases.append(TestCase(
        test_type="edge_case",
        test_name="test_rapid_zoom_operations",
        description="Test system handles rapid successive zoom operations",
        test_steps=[
            "Perform 10 rapid zoom in operations",
            "Verify all operations are processed",
            "Verify final zoom state is correct",
            "Verify no operations are lost",
            "Verify UI remains responsive"
        ],
        expected_result="Rapid zoom operations are handled correctly without loss or corruption",
        preconditions=["Application running", "Performance monitoring enabled"]
    ))
    
    execution_order = [
        test_cases[0].test_id,  # Unit: Font calculation
        test_cases[1].test_id,  # Unit: State tracking
        test_cases[2].test_id,  # Integration: Gesture pipeline
        test_cases[3].test_id,  # Integration: Keyboard/gesture consistency
        test_cases[4].test_id,  # UI: Visual verification
        test_cases[5].test_id,  # UI: Pane targeting
        test_cases[6].test_id,  # Edge: Zoom limits
        test_cases[7].test_id,  # Edge: Error recovery
        test_cases[8].test_id,  # Edge: Rapid operations
    ]
    
    return TestPlan(
        analysis_id=analysis.analysis_id,
        test_cases=test_cases,
        test_execution_order=execution_order
    )


if __name__ == "__main__":
    print("=" * 80)
    print("INSTRUCTOR CODE ANALYSIS - ZOOM FUNCTIONALITY")
    print("=" * 80)
    
    # Perform analysis
    analysis = analyze_zoom_code()
    print(f"\nAnalysis ID: {analysis.analysis_id}")
    print(f"Target File: {analysis.target_file}")
    print(f"Issues Found: {len(analysis.issues)}")
    
    print("\n" + "=" * 60)
    print("CRITICAL ISSUES SUMMARY")
    print("=" * 60)
    
    for issue in analysis.issues:
        if issue.severity == "critical":
            print(f"\n🔴 {issue.category.upper()}: {issue.description}")
            print(f"   Line {issue.line_number}: {issue.root_cause}")
            print(f"   Impact: {issue.impact}")
    
    print("\n" + "=" * 60)
    print("INTERACTION CONFLICTS")
    print("=" * 60)
    
    for i, conflict in enumerate(analysis.interaction_conflicts, 1):
        print(f"{i}. {conflict}")
    
    # Generate recommendations
    recommendations = generate_fix_recommendations(analysis)
    print(f"\n" + "=" * 60)
    print("PRIORITY FIX RECOMMENDATIONS")
    print("=" * 60)
    
    for fix in recommendations.fixes:
        if fix.priority in ["critical", "high"]:
            print(f"\n🔥 {fix.priority.upper()}: {fix.title}")
            print(f"   Implementation: {fix.implementation_approach}")
            print(f"   Risk: {fix.risk_assessment}")
    
    # Create test plan
    test_plan = create_test_plan(analysis)
    print(f"\n" + "=" * 60)
    print("TEST PLAN SUMMARY")
    print("=" * 60)
    
    test_counts = {}
    for test in test_plan.test_cases:
        test_counts[test.test_type] = test_counts.get(test.test_type, 0) + 1
    
    for test_type, count in test_counts.items():
        print(f"{test_type.title()} Tests: {count}")
    
    print(f"\nTotal Test Cases: {len(test_plan.test_cases)}")
    
    print("\n" + "=" * 80)
    print("NEXT STEPS")
    print("=" * 80)
    print("1. Implement CRITICAL fix: Unify zoom methods to use font-based approach")
    print("2. Integrate ZoomManagerState for structured state management")
    print("3. Refactor eventFilter to use dedicated zoom handling methods")
    print("4. Run comprehensive test suite to verify fixes")
    print("5. Add error handling and validation for robust zoom operations")
    print("\n" + "=" * 80)