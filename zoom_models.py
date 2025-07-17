"""
TASK-001: Pydantic Data Models for Zoom Functionality
Structured data models for zoom state management, gesture events, and UI components
"""

from typing import Optional, Dict, Any, List
from pydantic import BaseModel, Field, validator
from enum import Enum
from datetime import datetime


class PaneType(str, Enum):
    """Enum for pane identification"""
    LEFT = "left"
    RIGHT = "right"


class ZoomMethod(str, Enum):
    """Enum for zoom method types"""
    FONT_SIZE = "font_size"
    WIDGET_ZOOM = "widget_zoom"
    CSS_SCALING = "css_scaling"
    HYBRID = "hybrid"


class GestureType(str, Enum):
    """Enum for gesture types"""
    ZOOM_IN = "zoom_in"
    ZOOM_OUT = "zoom_out"
    PINCH = "pinch"
    SCROLL = "scroll"


class InputSource(str, Enum):
    """Enum for input source types"""
    KEYBOARD = "keyboard"
    GESTURE = "gesture"
    MENU = "menu"


class ZoomState(BaseModel):
    """Model for tracking zoom state of both panes"""
    left_pane_zoom: float = Field(default=1.0, ge=0.1, le=10.0, description="PDF pane zoom level")
    right_pane_zoom: float = Field(default=12.0, ge=8.0, le=48.0, description="Text pane font size")
    active_pane: PaneType = Field(default=PaneType.RIGHT, description="Currently active pane")
    zoom_method: ZoomMethod = Field(default=ZoomMethod.FONT_SIZE, description="Current zoom method")
    last_updated: datetime = Field(default_factory=datetime.now, description="Last zoom change timestamp")
    
    @validator('right_pane_zoom')
    def validate_text_zoom(cls, v):
        """Ensure text zoom is in valid range and whole numbers"""
        if v < 8 or v > 48:
            raise ValueError('Text zoom must be between 8 and 48')
        return float(int(v))  # Ensure whole numbers for visibility
    
    @validator('left_pane_zoom')
    def validate_pdf_zoom(cls, v):
        """Ensure PDF zoom is in valid range"""
        if v < 0.1 or v > 10.0:
            raise ValueError('PDF zoom must be between 0.1 and 10.0')
        return v


class GestureEvent(BaseModel):
    """Model for structured gesture event handling"""
    event_type: GestureType = Field(description="Type of gesture detected")
    input_source: InputSource = Field(description="Source of the zoom command")
    target_pane: PaneType = Field(description="Pane that should receive the zoom")
    zoom_delta: float = Field(description="Amount of zoom change")
    timestamp: datetime = Field(default_factory=datetime.now, description="When gesture occurred")
    processed: bool = Field(default=False, description="Whether event has been processed")
    
    @validator('zoom_delta')
    def validate_zoom_delta(cls, v, values):
        """Validate zoom delta based on event type"""
        if values.get('event_type') in [GestureType.ZOOM_IN, GestureType.ZOOM_OUT]:
            if abs(v) > 10:
                raise ValueError('Zoom delta too large')
        return v


class UIComponent(BaseModel):
    """Model for UI component state management"""
    component_id: str = Field(description="Unique identifier for UI component")
    component_type: str = Field(description="Type of component (QTextEdit, QPdfView, etc.)")
    is_active: bool = Field(default=False, description="Whether component is currently active")
    supports_zoom: bool = Field(default=True, description="Whether component supports zoom")
    current_zoom: float = Field(description="Current zoom level")
    zoom_method: ZoomMethod = Field(description="Zoom method used by this component")
    last_zoom_change: datetime = Field(default_factory=datetime.now, description="Last zoom change")
    
    class Config:
        extra = "forbid"


class ZoomConfig(BaseModel):
    """Model for zoom configuration settings"""
    # Text pane settings
    text_min_size: float = Field(default=8.0, description="Minimum text size")
    text_max_size: float = Field(default=48.0, description="Maximum text size") 
    text_zoom_step: float = Field(default=2.0, description="Text zoom increment")
    
    # PDF pane settings
    pdf_min_zoom: float = Field(default=0.1, description="Minimum PDF zoom")
    pdf_max_zoom: float = Field(default=10.0, description="Maximum PDF zoom")
    pdf_zoom_factor: float = Field(default=1.2, description="PDF zoom multiplier")
    
    # Gesture settings
    gesture_sensitivity: float = Field(default=1.0, description="Gesture sensitivity multiplier")
    gesture_threshold: float = Field(default=0.1, description="Minimum gesture delta to process")
    
    # Method preferences
    preferred_text_method: ZoomMethod = Field(default=ZoomMethod.FONT_SIZE, description="Preferred text zoom method")
    preferred_pdf_method: ZoomMethod = Field(default=ZoomMethod.WIDGET_ZOOM, description="Preferred PDF zoom method")
    
    @validator('text_zoom_step')
    def validate_text_step(cls, v):
        """Ensure text zoom step is reasonable"""
        if v < 1.0 or v > 5.0:
            raise ValueError('Text zoom step must be between 1.0 and 5.0')
        return v


class ZoomOperation(BaseModel):
    """Model for zoom operation execution"""
    operation_id: str = Field(description="Unique operation identifier")
    target_pane: PaneType = Field(description="Pane to zoom")
    zoom_type: GestureType = Field(description="Type of zoom operation")
    input_source: InputSource = Field(description="Source of zoom command")
    old_zoom: float = Field(description="Zoom level before operation")
    new_zoom: float = Field(description="Zoom level after operation")
    method_used: ZoomMethod = Field(description="Zoom method that was used")
    success: bool = Field(default=False, description="Whether operation succeeded")
    error_message: Optional[str] = Field(None, description="Error message if failed")
    execution_time: datetime = Field(default_factory=datetime.now, description="When operation executed")
    
    @validator('new_zoom')
    def validate_zoom_change(cls, v, values):
        """Validate that zoom change is reasonable"""
        old_zoom = values.get('old_zoom')
        if old_zoom and abs(v - old_zoom) > 20:
            raise ValueError('Zoom change too large')
        return v


class ZoomDebugInfo(BaseModel):
    """Model for zoom debugging information"""
    debug_id: str = Field(description="Debug session identifier")
    current_state: ZoomState = Field(description="Current zoom state")
    recent_events: List[GestureEvent] = Field(description="Recent gesture events")
    recent_operations: List[ZoomOperation] = Field(description="Recent zoom operations")
    active_component: UIComponent = Field(description="Currently active UI component")
    config: ZoomConfig = Field(description="Current zoom configuration")
    issues_detected: List[str] = Field(default_factory=list, description="Detected issues")
    
    def add_issue(self, issue: str):
        """Add a detected issue to the debug info"""
        self.issues_detected.append(f"[{datetime.now()}] {issue}")
    
    def get_summary(self) -> Dict[str, Any]:
        """Get a summary of the debug state"""
        return {
            "active_pane": self.current_state.active_pane,
            "zoom_levels": {
                "left": self.current_state.left_pane_zoom,
                "right": self.current_state.right_pane_zoom
            },
            "recent_events_count": len(self.recent_events),
            "recent_operations_count": len(self.recent_operations),
            "issues_count": len(self.issues_detected),
            "last_zoom_method": self.current_state.zoom_method
        }


class ZoomManagerState(BaseModel):
    """Complete state model for zoom manager"""
    zoom_state: ZoomState = Field(description="Current zoom state")
    config: ZoomConfig = Field(description="Zoom configuration")
    ui_components: Dict[str, UIComponent] = Field(description="UI components state")
    event_history: List[GestureEvent] = Field(default_factory=list, description="Event history")
    operation_history: List[ZoomOperation] = Field(default_factory=list, description="Operation history")
    
    def get_active_component(self) -> Optional[UIComponent]:
        """Get the currently active UI component"""
        for component in self.ui_components.values():
            if component.is_active:
                return component
        return None
    
    def update_zoom(self, pane: PaneType, new_zoom: float, method: ZoomMethod) -> ZoomOperation:
        """Update zoom state and create operation record"""
        operation_id = f"zoom_{pane.value}_{datetime.now().isoformat()}"
        
        if pane == PaneType.LEFT:
            old_zoom = self.zoom_state.left_pane_zoom
            self.zoom_state.left_pane_zoom = new_zoom
        else:
            old_zoom = self.zoom_state.right_pane_zoom
            self.zoom_state.right_pane_zoom = new_zoom
        
        self.zoom_state.zoom_method = method
        self.zoom_state.last_updated = datetime.now()
        
        operation = ZoomOperation(
            operation_id=operation_id,
            target_pane=pane,
            zoom_type=GestureType.ZOOM_IN if new_zoom > old_zoom else GestureType.ZOOM_OUT,
            input_source=InputSource.GESTURE,  # Default, can be overridden
            old_zoom=old_zoom,
            new_zoom=new_zoom,
            method_used=method,
            success=True
        )
        
        self.operation_history.append(operation)
        return operation


# Factory functions for creating default states
def create_default_zoom_state() -> ZoomState:
    """Create a default zoom state"""
    return ZoomState(
        left_pane_zoom=1.0,
        right_pane_zoom=12.0,
        active_pane=PaneType.RIGHT,
        zoom_method=ZoomMethod.FONT_SIZE
    )


def create_default_config() -> ZoomConfig:
    """Create a default zoom configuration"""
    return ZoomConfig(
        text_min_size=8.0,
        text_max_size=48.0,
        text_zoom_step=2.0,
        pdf_min_zoom=0.1,
        pdf_max_zoom=10.0,
        pdf_zoom_factor=1.2,
        gesture_sensitivity=1.0,
        gesture_threshold=0.1,
        preferred_text_method=ZoomMethod.FONT_SIZE,
        preferred_pdf_method=ZoomMethod.WIDGET_ZOOM
    )


def create_ui_component(component_id: str, component_type: str, zoom_method: ZoomMethod) -> UIComponent:
    """Create a UI component model"""
    return UIComponent(
        component_id=component_id,
        component_type=component_type,
        is_active=False,
        supports_zoom=True,
        current_zoom=12.0 if component_type == "QTextEdit" else 1.0,
        zoom_method=zoom_method
    )


def create_zoom_manager_state() -> ZoomManagerState:
    """Create a complete zoom manager state"""
    return ZoomManagerState(
        zoom_state=create_default_zoom_state(),
        config=create_default_config(),
        ui_components={
            "faithful_output": create_ui_component("faithful_output", "QTextEdit", ZoomMethod.FONT_SIZE),
            "embedded_pdf_view": create_ui_component("embedded_pdf_view", "QPdfView", ZoomMethod.WIDGET_ZOOM)
        }
    )


if __name__ == "__main__":
    # Demonstrate the models
    print("=" * 60)
    print("PYDANTIC ZOOM MODELS DEMONSTRATION")
    print("=" * 60)
    
    # Create default state
    manager_state = create_zoom_manager_state()
    print("Default Zoom Manager State:")
    print(f"  Active Pane: {manager_state.zoom_state.active_pane}")
    print(f"  Left Zoom: {manager_state.zoom_state.left_pane_zoom}")
    print(f"  Right Zoom: {manager_state.zoom_state.right_pane_zoom}")
    print(f"  Zoom Method: {manager_state.zoom_state.zoom_method}")
    print()
    
    # Create and process a gesture event
    gesture = GestureEvent(
        event_type=GestureType.ZOOM_IN,
        input_source=InputSource.GESTURE,
        target_pane=PaneType.RIGHT,
        zoom_delta=2.0
    )
    print("Sample Gesture Event:")
    print(f"  Type: {gesture.event_type}")
    print(f"  Target: {gesture.target_pane}")
    print(f"  Delta: {gesture.zoom_delta}")
    print()
    
    # Update zoom state
    operation = manager_state.update_zoom(PaneType.RIGHT, 14.0, ZoomMethod.FONT_SIZE)
    print("Zoom Operation Result:")
    print(f"  Operation ID: {operation.operation_id}")
    print(f"  Old Zoom: {operation.old_zoom}")
    print(f"  New Zoom: {operation.new_zoom}")
    print(f"  Method: {operation.method_used}")
    print(f"  Success: {operation.success}")
    print()
    
    # Create debug info
    debug_info = ZoomDebugInfo(
        debug_id="debug_001",
        current_state=manager_state.zoom_state,
        recent_events=[gesture],
        recent_operations=[operation],
        active_component=manager_state.get_active_component() or create_ui_component("default", "QTextEdit", ZoomMethod.FONT_SIZE),
        config=manager_state.config
    )
    
    debug_info.add_issue("Pinch gesture detected but no visual zoom")
    debug_info.add_issue("Conflicting zoom methods detected")
    
    print("Debug Summary:")
    summary = debug_info.get_summary()
    for key, value in summary.items():
        print(f"  {key}: {value}")
    
    print("\nDetected Issues:")
    for issue in debug_info.issues_detected:
        print(f"  • {issue}")
    
    print("\n" + "=" * 60)
    print("PYDANTIC MODELS READY FOR INSTRUCTOR & OPENHANDS")
    print("=" * 60)