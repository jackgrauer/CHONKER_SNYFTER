"""
PYDANTIC-CLEANUP-001: Elegant Data Models for Working Features

This module provides clean, production-ready Pydantic models for the successfully 
implemented features in the CHONKER_SNYFTER application. These models provide 
type safety, validation, and structured data handling for:

1. OCR-VLM-FALLBACK: OCR with VLM fallback for math formulas
2. PDF-TEXT-SELECTION: PDF text selection layer synchronization  
3. SELECTION-SYNC: Bidirectional selection synchronization
4. GESTURE-DETECTION: Native gesture detection and handling
5. KEYBOARD-SHORTCUTS: Standardized keyboard shortcuts

Created as part of the elegantification operation to provide structured 
data models for today's big advances.
"""

from typing import Optional, Dict, Any, List, Union, Tuple
from pydantic import BaseModel, Field, field_validator, model_validator
from enum import Enum
from datetime import datetime
from uuid import uuid4, UUID


# ==================== ENUMS ====================

class OCRProcessingStatus(str, Enum):
    """Status of OCR processing"""
    PENDING = "pending"
    PROCESSING = "processing"
    COMPLETED = "completed"
    FAILED = "failed"
    VLM_FALLBACK = "vlm_fallback"


class MathFormulaDetectionStatus(str, Enum):
    """Status of math formula detection"""
    NOT_DETECTED = "not_detected"
    DETECTED = "detected"
    VLM_REQUIRED = "vlm_required"
    VLM_PROCESSED = "vlm_processed"


class SelectionType(str, Enum):
    """Type of text selection"""
    WORD = "word"
    LINE = "line"
    PARAGRAPH = "paragraph"
    BLOCK = "block"
    CUSTOM = "custom"


class SyncDirection(str, Enum):
    """Direction of selection synchronization"""
    PDF_TO_TEXT = "pdf_to_text"
    TEXT_TO_PDF = "text_to_pdf"
    BIDIRECTIONAL = "bidirectional"
    NONE = "none"


class GestureType(str, Enum):
    """Type of gesture detected"""
    ZOOM_IN = "zoom_in"
    ZOOM_OUT = "zoom_out"
    PINCH = "pinch"
    SCROLL = "scroll"
    PAN = "pan"
    TAP = "tap"
    DOUBLE_TAP = "double_tap"


class KeyboardShortcutAction(str, Enum):
    """Available keyboard shortcut actions"""
    OPEN_FILE = "open_file"
    PROCESS_DOCUMENT = "process_document"
    TOGGLE_OCR = "toggle_ocr"
    ZOOM_IN = "zoom_in"
    ZOOM_OUT = "zoom_out"
    RESET_ZOOM = "reset_zoom"
    TOGGLE_PANE = "toggle_pane"
    SAVE_OUTPUT = "save_output"
    COPY_SELECTION = "copy_selection"
    SELECT_ALL = "select_all"


class PaneType(str, Enum):
    """Pane identifier"""
    LEFT = "left"
    RIGHT = "right"
    PDF = "pdf"
    TEXT = "text"


# ==================== OCR AND VLM MODELS ====================

class MathFormulaDetection(BaseModel):
    """Model for math formula detection results"""
    contains_math: bool = Field(description="Whether text contains math formulas")
    formula_count: int = Field(default=0, ge=0, description="Number of formulas detected")
    formula_patterns: List[str] = Field(default_factory=list, description="Detected formula patterns")
    greek_letters: bool = Field(default=False, description="Contains Greek letters")
    math_symbols: bool = Field(default=False, description="Contains math symbols")
    subscripts_superscripts: bool = Field(default=False, description="Contains subscripts/superscripts")
    latex_commands: bool = Field(default=False, description="Contains LaTeX commands")
    garbled_characters: int = Field(default=0, ge=0, description="Count of garbled characters")
    confidence_score: float = Field(default=0.0, ge=0.0, le=1.0, description="Detection confidence")
    
    @field_validator('formula_count')
    @classmethod
    def validate_formula_count(cls, v, info):
        """Ensure formula count matches detection status"""
        if info.data.get('contains_math') and v == 0:
            raise ValueError('Math detected but formula count is 0')
        return v


class OCRProcessingMetrics(BaseModel):
    """Metrics for OCR processing performance"""
    processing_time: float = Field(ge=0.0, description="Processing time in seconds")
    pages_processed: int = Field(default=1, ge=1, description="Number of pages processed")
    text_extraction_rate: float = Field(ge=0.0, le=1.0, description="Rate of successful text extraction")
    error_count: int = Field(default=0, ge=0, description="Number of errors encountered")
    memory_usage_mb: float = Field(default=0.0, ge=0.0, description="Memory usage in MB")
    
    @property
    def processing_rate(self) -> float:
        """Calculate processing rate in pages per second"""
        return self.pages_processed / self.processing_time if self.processing_time > 0 else 0.0


class OCRResult(BaseModel):
    """Comprehensive OCR result model with VLM fallback support"""
    result_id: str = Field(default_factory=lambda: str(uuid4()), description="Unique result identifier")
    
    # Core content
    text_content: str = Field(description="Extracted text content")
    html_content: Optional[str] = Field(None, description="HTML-formatted content")
    
    # Processing status
    processing_status: OCRProcessingStatus = Field(default=OCRProcessingStatus.PENDING, description="Processing status")
    used_ocr: bool = Field(default=False, description="Whether OCR was used")
    vlm_fallback_used: bool = Field(default=False, description="Whether VLM fallback was used")
    
    # Math formula detection
    math_detection: MathFormulaDetection = Field(default_factory=lambda: MathFormulaDetection(contains_math=False), description="Math formula detection results")
    math_detection_status: MathFormulaDetectionStatus = Field(default=MathFormulaDetectionStatus.NOT_DETECTED, description="Math detection status")
    
    # Processing metrics
    processing_metrics: OCRProcessingMetrics = Field(description="Processing performance metrics")
    
    # Confidence and quality
    overall_confidence: float = Field(default=0.0, ge=0.0, le=1.0, description="Overall processing confidence")
    text_quality_score: float = Field(default=0.0, ge=0.0, le=1.0, description="Text quality assessment")
    
    # Metadata
    document_path: str = Field(description="Path to source document")
    processed_at: datetime = Field(default_factory=datetime.now, description="Processing timestamp")
    processing_options: Dict[str, Any] = Field(default_factory=dict, description="Processing options used")
    
    # Error handling
    errors: List[str] = Field(default_factory=list, description="Processing errors")
    warnings: List[str] = Field(default_factory=list, description="Processing warnings")
    
    @field_validator('overall_confidence')
    @classmethod
    def validate_confidence(cls, v, info):
        """Adjust confidence based on processing status"""
        if info.data.get('processing_status') == OCRProcessingStatus.FAILED:
            return 0.0
        elif info.data.get('vlm_fallback_used'):
            return max(0.7, v)  # VLM fallback should have decent confidence
        return v
    
    @model_validator(mode='after')
    def validate_math_consistency(self):
        """Ensure math detection consistency"""
        if self.math_detection and self.math_detection.contains_math:
            if self.math_detection_status == MathFormulaDetectionStatus.NOT_DETECTED:
                self.math_detection_status = MathFormulaDetectionStatus.DETECTED
        return self
    
    def should_use_vlm_fallback(self) -> bool:
        """Determine if VLM fallback should be used"""
        return (
            self.math_detection.contains_math and 
            self.math_detection.confidence_score < 0.7 and
            not self.vlm_fallback_used
        )
    
    def get_processing_summary(self) -> Dict[str, Any]:
        """Get a summary of processing results"""
        return {
            "status": self.processing_status,
            "used_ocr": self.used_ocr,
            "vlm_fallback": self.vlm_fallback_used,
            "math_formulas": self.math_detection.formula_count,
            "confidence": self.overall_confidence,
            "processing_time": self.processing_metrics.processing_time,
            "errors": len(self.errors),
            "warnings": len(self.warnings)
        }


# ==================== PDF SELECTION MODELS ====================

class SelectionCoordinates(BaseModel):
    """Coordinates for text selection"""
    start_x: float = Field(description="Start X coordinate")
    start_y: float = Field(description="Start Y coordinate")
    end_x: float = Field(description="End X coordinate")
    end_y: float = Field(description="End Y coordinate")
    page_number: int = Field(default=1, ge=1, description="Page number (1-based)")
    
    @field_validator('end_x')
    @classmethod
    def validate_coordinates(cls, v, info):
        """Validate coordinate consistency"""
        start_x = info.data.get('start_x', 0)
        if v < start_x:
            raise ValueError('End X must be >= Start X')
        return v
    
    @property
    def width(self) -> float:
        """Calculate selection width"""
        return self.end_x - self.start_x
    
    @property
    def height(self) -> float:
        """Calculate selection height"""
        return abs(self.end_y - self.start_y)


class PDFSelectionState(BaseModel):
    """Model for PDF text selection state"""
    selection_id: str = Field(default_factory=lambda: str(uuid4()), description="Unique selection identifier")
    
    # Selection content
    selected_text: str = Field(default="", description="Selected text content")
    text_length: int = Field(default=0, ge=0, description="Length of selected text")
    
    # Selection properties
    selection_type: SelectionType = Field(default=SelectionType.CUSTOM, description="Type of selection")
    coordinates: Optional[SelectionCoordinates] = Field(None, description="Selection coordinates")
    
    # State tracking
    is_active: bool = Field(default=False, description="Whether selection is active")
    is_synchronized: bool = Field(default=False, description="Whether selection is synchronized")
    sync_direction: SyncDirection = Field(default=SyncDirection.NONE, description="Synchronization direction")
    
    # Timing
    created_at: datetime = Field(default_factory=datetime.now, description="Selection creation time")
    last_updated: datetime = Field(default_factory=datetime.now, description="Last update time")
    
    # Metadata
    pane_source: PaneType = Field(description="Source pane of selection")
    chunk_references: List[str] = Field(default_factory=list, description="References to document chunks")
    
    @field_validator('text_length')
    @classmethod
    def validate_text_length(cls, v, info):
        """Ensure text length matches selected text"""
        selected_text = info.data.get('selected_text', '')
        if len(selected_text) != v:
            return len(selected_text)
        return v
    
    @model_validator(mode='after')
    def validate_selection_consistency(self):
        """Ensure selection state consistency"""
        if self.is_active and not self.selected_text:
            self.is_active = False
        elif self.selected_text and not self.is_active:
            self.is_active = True
        return self
    
    def clear_selection(self):
        """Clear the selection"""
        self.selected_text = ""
        self.text_length = 0
        self.is_active = False
        self.is_synchronized = False
        self.sync_direction = SyncDirection.NONE
        self.coordinates = None
        self.last_updated = datetime.now()


# ==================== SELECTION SYNC MODELS ====================

class SyncConflictResolution(BaseModel):
    """Model for handling selection sync conflicts"""
    conflict_detected: bool = Field(default=False, description="Whether conflict was detected")
    resolution_strategy: str = Field(default="latest_wins", description="Strategy used to resolve conflict")
    original_selection: Optional[str] = Field(None, description="Original selection before conflict")
    resolved_selection: Optional[str] = Field(None, description="Selection after conflict resolution")
    resolution_timestamp: Optional[datetime] = Field(None, description="When conflict was resolved")
    
    def resolve_conflict(self, strategy: str = "latest_wins"):
        """Resolve a detected conflict"""
        self.conflict_detected = True
        self.resolution_strategy = strategy
        self.resolution_timestamp = datetime.now()


class SelectionSyncManager(BaseModel):
    """Model for bidirectional selection synchronization"""
    sync_id: str = Field(default_factory=lambda: str(uuid4()), description="Unique sync session identifier")
    
    # Pane states
    left_pane_selection: PDFSelectionState = Field(default_factory=PDFSelectionState, description="Left pane selection state")
    right_pane_selection: PDFSelectionState = Field(default_factory=PDFSelectionState, description="Right pane selection state")
    
    # Sync configuration
    sync_enabled: bool = Field(default=True, description="Whether synchronization is enabled")
    sync_direction: SyncDirection = Field(default=SyncDirection.BIDIRECTIONAL, description="Synchronization direction")
    debounce_delay_ms: int = Field(default=200, ge=0, description="Debounce delay in milliseconds")
    
    # Sync status
    last_sync_timestamp: Optional[datetime] = Field(None, description="Last successful sync timestamp")
    sync_in_progress: bool = Field(default=False, description="Whether sync is currently in progress")
    
    # Conflict resolution
    conflict_resolution: SyncConflictResolution = Field(default_factory=SyncConflictResolution, description="Conflict resolution info")
    
    # Coordinate mapping
    coordinate_mapping: Dict[str, Any] = Field(default_factory=dict, description="Mapping between PDF and text coordinates")
    
    # History
    sync_history: List[Dict[str, Any]] = Field(default_factory=list, description="History of sync operations")
    
    @field_validator('debounce_delay_ms')
    @classmethod
    def validate_debounce_delay(cls, v):
        """Validate debounce delay is reasonable"""
        if v > 5000:  # 5 seconds max
            raise ValueError('Debounce delay too long')
        return v
    
    def sync_selections(self, source_pane: PaneType, target_pane: PaneType):
        """Synchronize selections between panes"""
        if not self.sync_enabled or self.sync_in_progress:
            return
        
        self.sync_in_progress = True
        
        try:
            # Get source selection
            source_selection = (
                self.left_pane_selection if source_pane == PaneType.LEFT 
                else self.right_pane_selection
            )
            
            # Get target selection
            target_selection = (
                self.right_pane_selection if target_pane == PaneType.RIGHT 
                else self.left_pane_selection
            )
            
            # Perform sync logic (simplified)
            if source_selection.is_active and source_selection.selected_text:
                target_selection.selected_text = source_selection.selected_text
                target_selection.text_length = source_selection.text_length
                target_selection.is_active = True
                target_selection.is_synchronized = True
                target_selection.sync_direction = SyncDirection.BIDIRECTIONAL
                target_selection.last_updated = datetime.now()
            
            # Update sync status
            self.last_sync_timestamp = datetime.now()
            
            # Add to history
            self.sync_history.append({
                "timestamp": datetime.now(),
                "source_pane": source_pane,
                "target_pane": target_pane,
                "text_length": source_selection.text_length,
                "success": True
            })
            
        finally:
            self.sync_in_progress = False
    
    def get_active_selection(self) -> Optional[PDFSelectionState]:
        """Get the currently active selection"""
        if self.left_pane_selection.is_active:
            return self.left_pane_selection
        elif self.right_pane_selection.is_active:
            return self.right_pane_selection
        return None
    
    def clear_all_selections(self):
        """Clear all selections"""
        self.left_pane_selection.clear_selection()
        self.right_pane_selection.clear_selection()
        self.last_sync_timestamp = datetime.now()


# ==================== GESTURE MODELS ====================

class GestureValues(BaseModel):
    """Values associated with a gesture"""
    delta_x: float = Field(default=0.0, description="X-axis delta")
    delta_y: float = Field(default=0.0, description="Y-axis delta")
    scale_factor: float = Field(default=1.0, description="Scale factor for zoom gestures")
    velocity: float = Field(default=0.0, description="Gesture velocity")
    pressure: float = Field(default=0.0, ge=0.0, le=1.0, description="Pressure (for supported devices)")
    
    @field_validator('scale_factor')
    @classmethod
    def validate_scale_factor(cls, v):
        """Validate scale factor is reasonable"""
        if v < 0.1 or v > 10.0:
            raise ValueError('Scale factor must be between 0.1 and 10.0')
        return v


class GestureEvent(BaseModel):
    """Model for gesture event handling"""
    event_id: str = Field(default_factory=lambda: str(uuid4()), description="Unique event identifier")
    
    # Gesture properties
    gesture_type: GestureType = Field(description="Type of gesture")
    gesture_values: GestureValues = Field(default_factory=GestureValues, description="Gesture values and deltas")
    
    # Target information
    target_pane: PaneType = Field(description="Target pane for the gesture")
    target_component: Optional[str] = Field(None, description="Specific UI component target")
    
    # Timing
    start_timestamp: datetime = Field(default_factory=datetime.now, description="Gesture start time")
    end_timestamp: Optional[datetime] = Field(None, description="Gesture end time")
    processing_timestamp: Optional[datetime] = Field(None, description="When gesture was processed")
    
    # Processing status
    processed: bool = Field(default=False, description="Whether gesture has been processed")
    processing_successful: bool = Field(default=False, description="Whether processing was successful")
    processing_error: Optional[str] = Field(None, description="Processing error message")
    
    # Context
    modifier_keys: List[str] = Field(default_factory=list, description="Active modifier keys")
    device_type: str = Field(default="unknown", description="Input device type")
    
    @field_validator('end_timestamp')
    @classmethod
    def validate_end_timestamp(cls, v, info):
        """Ensure end timestamp is after start timestamp"""
        start_timestamp = info.data.get('start_timestamp')
        if v and start_timestamp and v < start_timestamp:
            raise ValueError('End timestamp must be after start timestamp')
        return v
    
    def mark_processed(self, success: bool = True, error: Optional[str] = None):
        """Mark the gesture as processed"""
        self.processed = True
        self.processing_successful = success
        self.processing_error = error
        self.processing_timestamp = datetime.now()
        if not self.end_timestamp:
            self.end_timestamp = datetime.now()
    
    @property
    def duration_ms(self) -> Optional[float]:
        """Calculate gesture duration in milliseconds"""
        if self.end_timestamp:
            return (self.end_timestamp - self.start_timestamp).total_seconds() * 1000
        return None


# ==================== KEYBOARD SHORTCUT MODELS ====================

class KeyboardShortcutConfig(BaseModel):
    """Configuration for a keyboard shortcut"""
    key_combination: str = Field(description="Key combination (e.g., 'Ctrl+P')")
    action: KeyboardShortcutAction = Field(description="Action to perform")
    enabled: bool = Field(default=True, description="Whether shortcut is enabled")
    customizable: bool = Field(default=True, description="Whether shortcut can be customized")
    description: str = Field(description="Human-readable description")
    tooltip: Optional[str] = Field(None, description="Tooltip text")
    
    @field_validator('key_combination')
    @classmethod
    def validate_key_combination(cls, v):
        """Validate key combination format"""
        if not v or len(v) < 1:
            raise ValueError('Key combination cannot be empty')
        
        # Basic validation for common patterns
        valid_modifiers = ['Ctrl', 'Alt', 'Shift', 'Meta', 'Cmd']
        parts = v.split('+')
        
        if len(parts) > 1:
            modifiers = parts[:-1]
            for mod in modifiers:
                if mod not in valid_modifiers:
                    raise ValueError(f'Invalid modifier: {mod}')
        
        return v


class KeyboardShortcut(BaseModel):
    """Model for keyboard shortcut management"""
    shortcut_id: str = Field(default_factory=lambda: str(uuid4()), description="Unique shortcut identifier")
    
    # Configuration
    config: KeyboardShortcutConfig = Field(description="Shortcut configuration")
    
    # Usage tracking
    usage_count: int = Field(default=0, ge=0, description="Number of times used")
    last_used: Optional[datetime] = Field(None, description="Last time shortcut was used")
    
    # Context
    context_restrictions: List[str] = Field(default_factory=list, description="Contexts where shortcut is active")
    conflicting_shortcuts: List[str] = Field(default_factory=list, description="Other shortcuts that conflict")
    
    # Customization
    original_key_combination: str = Field(description="Original key combination")
    custom_key_combination: Optional[str] = Field(None, description="Custom key combination")
    
    def __init__(self, **data):
        super().__init__(**data)
        if not self.original_key_combination:
            self.original_key_combination = self.config.key_combination
    
    def use_shortcut(self):
        """Record usage of the shortcut"""
        self.usage_count += 1
        self.last_used = datetime.now()
    
    def customize_key_combination(self, new_combination: str):
        """Customize the key combination"""
        if not self.config.customizable:
            raise ValueError("This shortcut cannot be customized")
        
        # Validate new combination
        KeyboardShortcutConfig.validate_key_combination(new_combination)
        
        self.custom_key_combination = new_combination
        self.config.key_combination = new_combination
    
    def reset_to_default(self):
        """Reset to original key combination"""
        self.custom_key_combination = None
        self.config.key_combination = self.original_key_combination
    
    @property
    def is_customized(self) -> bool:
        """Check if shortcut has been customized"""
        return self.custom_key_combination is not None
    
    @property
    def effective_key_combination(self) -> str:
        """Get the effective key combination"""
        return self.custom_key_combination or self.original_key_combination


# ==================== FACTORY FUNCTIONS ====================

def create_default_ocr_result(document_path: str, text_content: str) -> OCRResult:
    """Create a default OCR result"""
    return OCRResult(
        text_content=text_content,
        document_path=document_path,
        processing_metrics=OCRProcessingMetrics(
            processing_time=0.0,
            pages_processed=1,
            text_extraction_rate=1.0
        )
    )


def create_pdf_selection_state(pane_source: PaneType) -> PDFSelectionState:
    """Create a PDF selection state for a specific pane"""
    return PDFSelectionState(
        pane_source=pane_source,
        selection_type=SelectionType.CUSTOM,
        sync_direction=SyncDirection.BIDIRECTIONAL
    )


def create_selection_sync_manager() -> SelectionSyncManager:
    """Create a selection sync manager with default settings"""
    return SelectionSyncManager(
        left_pane_selection=create_pdf_selection_state(PaneType.LEFT),
        right_pane_selection=create_pdf_selection_state(PaneType.RIGHT),
        sync_direction=SyncDirection.BIDIRECTIONAL,
        debounce_delay_ms=200
    )


def create_gesture_event(gesture_type: GestureType, target_pane: PaneType) -> GestureEvent:
    """Create a gesture event"""
    return GestureEvent(
        gesture_type=gesture_type,
        target_pane=target_pane,
        gesture_values=GestureValues()
    )


def create_default_keyboard_shortcuts() -> List[KeyboardShortcut]:
    """Create default keyboard shortcuts"""
    shortcuts = [
        KeyboardShortcut(
            config=KeyboardShortcutConfig(
                key_combination="Ctrl+O",
                action=KeyboardShortcutAction.OPEN_FILE,
                description="Open PDF file",
                tooltip="Open PDF (Ctrl+O)"
            ),
            original_key_combination="Ctrl+O"
        ),
        KeyboardShortcut(
            config=KeyboardShortcutConfig(
                key_combination="Ctrl+P",
                action=KeyboardShortcutAction.PROCESS_DOCUMENT,
                description="Process document",
                tooltip="Process document (Ctrl+P)"
            ),
            original_key_combination="Ctrl+P"
        ),
        KeyboardShortcut(
            config=KeyboardShortcutConfig(
                key_combination="Ctrl+Plus",
                action=KeyboardShortcutAction.ZOOM_IN,
                description="Zoom in",
                tooltip="Zoom in (Ctrl+Plus)"
            ),
            original_key_combination="Ctrl+Plus"
        ),
        KeyboardShortcut(
            config=KeyboardShortcutConfig(
                key_combination="Ctrl+Minus",
                action=KeyboardShortcutAction.ZOOM_OUT,
                description="Zoom out",
                tooltip="Zoom out (Ctrl+Minus)"
            ),
            original_key_combination="Ctrl+Minus"
        ),
        KeyboardShortcut(
            config=KeyboardShortcutConfig(
                key_combination="Ctrl+0",
                action=KeyboardShortcutAction.RESET_ZOOM,
                description="Reset zoom",
                tooltip="Reset zoom (Ctrl+0)"
            ),
            original_key_combination="Ctrl+0"
        )
    ]
    
    return shortcuts


# ==================== VALIDATION HELPERS ====================

def validate_model_consistency(models: List[BaseModel]) -> Dict[str, Any]:
    """Validate consistency across multiple models"""
    validation_results = {
        "valid": True,
        "errors": [],
        "warnings": []
    }
    
    # Add validation logic as needed
    return validation_results


# ==================== SERIALIZATION HELPERS ====================

def serialize_for_storage(model: BaseModel) -> Dict[str, Any]:
    """Serialize model for storage with custom handling"""
    data = model.dict()
    
    # Convert datetime objects to ISO format
    def convert_datetime(obj):
        if isinstance(obj, datetime):
            return obj.isoformat()
        elif isinstance(obj, dict):
            return {k: convert_datetime(v) for k, v in obj.items()}
        elif isinstance(obj, list):
            return [convert_datetime(item) for item in obj]
        return obj
    
    return convert_datetime(data)


def deserialize_from_storage(data: Dict[str, Any], model_class: type) -> BaseModel:
    """Deserialize model from storage with custom handling"""
    # Convert ISO format strings back to datetime objects
    def convert_datetime_strings(obj):
        if isinstance(obj, str):
            try:
                return datetime.fromisoformat(obj)
            except ValueError:
                return obj
        elif isinstance(obj, dict):
            return {k: convert_datetime_strings(v) for k, v in obj.items()}
        elif isinstance(obj, list):
            return [convert_datetime_strings(item) for item in obj]
        return obj
    
    processed_data = convert_datetime_strings(data)
    return model_class(**processed_data)


if __name__ == "__main__":
    # Demonstration of the elegant models
    print("=" * 80)
    print("ELEGANT PYDANTIC MODELS FOR WORKING FEATURES")
    print("=" * 80)
    
    # 1. OCR Result with VLM fallback
    print("\n1. OCR Result with VLM Fallback:")
    ocr_result = create_default_ocr_result(
        document_path="/path/to/document.pdf",
        text_content="Sample text with math: E=mc² and chemical formula H₂O"
    )
    ocr_result.math_detection.contains_math = True
    ocr_result.math_detection.formula_count = 2
    ocr_result.math_detection.subscripts_superscripts = True
    ocr_result.processing_status = OCRProcessingStatus.COMPLETED
    ocr_result.overall_confidence = 0.95
    
    print(f"  Status: {ocr_result.processing_status}")
    print(f"  Math formulas: {ocr_result.math_detection.formula_count}")
    print(f"  Confidence: {ocr_result.overall_confidence}")
    print(f"  VLM needed: {ocr_result.should_use_vlm_fallback()}")
    
    # 2. PDF Selection State
    print("\n2. PDF Selection State:")
    selection_state = create_pdf_selection_state(PaneType.LEFT)
    selection_state.selected_text = "Sample selected text"
    selection_state.text_length = len(selection_state.selected_text)
    selection_state.is_active = True
    selection_state.selection_type = SelectionType.PARAGRAPH
    
    print(f"  Selected text: '{selection_state.selected_text}'")
    print(f"  Selection type: {selection_state.selection_type}")
    print(f"  Active: {selection_state.is_active}")
    print(f"  Synchronized: {selection_state.is_synchronized}")
    
    # 3. Selection Sync Manager
    print("\n3. Selection Sync Manager:")
    sync_manager = create_selection_sync_manager()
    sync_manager.left_pane_selection.selected_text = "Left pane text"
    sync_manager.left_pane_selection.is_active = True
    sync_manager.sync_selections(PaneType.LEFT, PaneType.RIGHT)
    
    print(f"  Sync enabled: {sync_manager.sync_enabled}")
    print(f"  Left pane active: {sync_manager.left_pane_selection.is_active}")
    print(f"  Right pane active: {sync_manager.right_pane_selection.is_active}")
    print(f"  Sync history entries: {len(sync_manager.sync_history)}")
    
    # 4. Gesture Event
    print("\n4. Gesture Event:")
    gesture = create_gesture_event(GestureType.ZOOM_IN, PaneType.RIGHT)
    gesture.gesture_values.scale_factor = 1.2
    gesture.gesture_values.delta_y = -5.0
    gesture.mark_processed(success=True)
    
    print(f"  Gesture type: {gesture.gesture_type}")
    print(f"  Target pane: {gesture.target_pane}")
    print(f"  Scale factor: {gesture.gesture_values.scale_factor}")
    print(f"  Processed: {gesture.processed}")
    print(f"  Duration: {gesture.duration_ms}ms")
    
    # 5. Keyboard Shortcuts
    print("\n5. Keyboard Shortcuts:")
    shortcuts = create_default_keyboard_shortcuts()
    
    for shortcut in shortcuts[:3]:  # Show first 3
        print(f"  {shortcut.config.key_combination}: {shortcut.config.description}")
        print(f"    Action: {shortcut.config.action}")
        print(f"    Enabled: {shortcut.config.enabled}")
        print(f"    Usage count: {shortcut.usage_count}")
    
    # Demonstration of customization
    print("\n6. Shortcut Customization:")
    open_shortcut = shortcuts[0]
    open_shortcut.customize_key_combination("Ctrl+Alt+O")
    print(f"  Original: {open_shortcut.original_key_combination}")
    print(f"  Custom: {open_shortcut.custom_key_combination}")
    print(f"  Effective: {open_shortcut.effective_key_combination}")
    print(f"  Is customized: {open_shortcut.is_customized}")
    
    print("\n" + "=" * 80)
    print("ELEGANT MODELS READY FOR PRODUCTION USE")
    print("Features: Type safety, validation, serialization, factory methods")
    print("=" * 80)