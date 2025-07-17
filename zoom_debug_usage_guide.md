# OpenHands Zoom Debugging Functions Usage Guide

## Overview

The `zoom_debug_functions.py` file contains systematic debugging functions for the zoom functionality in CHONKER_SNYFTER. These functions use the Pydantic models and provide actionable debugging output with structured results.

## Functions Available

### 1. `debug_gesture_flow(app_instance=None)`
**Purpose**: Trace complete gesture event pipeline to identify where events get lost or misrouted.

**Key Features**:
- Tests event filter installation
- Validates active pane detection logic
- Simulates gesture event processing
- Detects conflicting zoom implementations

**Usage**:
```python
from zoom_debug_functions import debug_gesture_flow

# Test without real app instance (mock mode)
results = debug_gesture_flow()

# Test with real app instance
results = debug_gesture_flow(app_instance=my_app)
```

**Returns**: Structured dictionary with event flow analysis, detected issues, and recommendations.

### 2. `test_zoom_methods(app_instance=None)`
**Purpose**: Test all zoom methods (font_size, widget_zoom, css_scaling) and compare effectiveness.

**Key Features**:
- Tests effectiveness on HTML vs plain text
- Measures visual impact of each method
- Provides method-specific recommendations
- Compares consistency across content types

**Usage**:
```python
from zoom_debug_functions import test_zoom_methods

results = test_zoom_methods(app_instance=my_app)
```

**Returns**: Dictionary with test results for each zoom method and content type.

### 3. `validate_html_zoom(app_instance=None, test_html=None)`
**Purpose**: Specifically test HTML content zoom behavior.

**Key Features**:
- Compares QTextEdit.zoomIn() vs font.setPointSize()
- Tests zoom persistence across content changes
- Validates HTML structure preservation
- Measures visual effectiveness

**Usage**:
```python
from zoom_debug_functions import validate_html_zoom

# Use default test HTML
results = validate_html_zoom(app_instance=my_app)

# Use custom HTML
custom_html = "<h1>Test</h1><p>Content</p>"
results = validate_html_zoom(app_instance=my_app, test_html=custom_html)
```

**Returns**: Dictionary with HTML zoom validation results and recommendations.

### 4. `fix_zoom_conflicts(app_instance=None, auto_fix=False)`
**Purpose**: Detect and fix conflicting zoom implementations.

**Key Features**:
- Automatically detects conflicting zoom implementations
- Suggests unified zoom approach
- Validates state consistency with ZoomManagerState
- Optionally applies fixes (when auto_fix=True)

**Usage**:
```python
from zoom_debug_functions import fix_zoom_conflicts

# Analysis only
results = fix_zoom_conflicts(app_instance=my_app)

# Analysis with auto-fix
results = fix_zoom_conflicts(app_instance=my_app, auto_fix=True)
```

**Returns**: Dictionary with conflict analysis and fix recommendations.

### 5. `run_comprehensive_zoom_debug(app_instance=None)`
**Purpose**: Run all debugging functions in sequence for complete analysis.

**Key Features**:
- Runs all 4 debugging functions
- Aggregates critical issues
- Provides comprehensive summary
- Generates phase-based action plan

**Usage**:
```python
from zoom_debug_functions import run_comprehensive_zoom_debug

results = run_comprehensive_zoom_debug(app_instance=my_app)
```

**Returns**: Comprehensive analysis with all function results and action plan.

## Key Findings from Current Analysis

Based on the Instructor analysis and debugging functions, the main issues are:

### Critical Issues Detected:
1. **Conflicting zoom implementations** (Lines 1655-1659 vs 1835)
   - Gesture handler uses document zoom (`zoomIn()/zoomOut()`)
   - Keyboard handler uses font size (`font.setPointSize()`)
   - This causes inconsistent behavior

2. **Poor HTML zoom effectiveness**
   - `QTextEdit.zoomIn/zoomOut()` has minimal effect on HTML content
   - Font size method is much more effective for HTML

3. **State inconsistency**
   - No unified zoom state management
   - Different zoom methods can conflict

### Recommended Solutions:

#### Immediate Actions:
1. **Unify zoom methods**: Replace all `zoomIn()/zoomOut()` calls with `font.setPointSize()`
2. **Remove redundant zoom calls**: Eliminate duplicate zoom operations in keyboard handlers
3. **Add zoom bounds validation**: Ensure zoom values stay within valid ranges (8-48 for text)

#### Code Changes Needed:

**In gesture handler (lines 1655-1659):**
```python
# BEFORE
if zoom_delta > 0:
    self.faithful_output.zoomIn()
else:
    self.faithful_output.zoomOut()

# AFTER
font = self.faithful_output.font()
delta = 2 if zoom_delta > 0 else -2
new_size = max(8, min(48, font.pointSize() + delta))
font.setPointSize(new_size)
self.faithful_output.setFont(font)
```

**In keyboard handlers (lines 1835, 1850):**
```python
# BEFORE
font = self.faithful_output.font()
font.setPointSize(self.text_zoom)
self.faithful_output.setFont(font)
self.faithful_output.zoomIn(2)  # Remove this line

# AFTER
font = self.faithful_output.font()
font.setPointSize(self.text_zoom)
self.faithful_output.setFont(font)
# Remove redundant zoomIn/zoomOut calls
```

## Integration with Pydantic Models

The debugging functions integrate with the Pydantic models from `zoom_models.py`:

- **ZoomManagerState**: Manages complete zoom state
- **ZoomOperation**: Tracks zoom operations
- **GestureEvent**: Handles gesture events
- **ZoomConfig**: Configuration settings

## Testing and Validation

The functions work in two modes:

1. **Mock Mode**: When Pydantic models aren't available, uses mock implementations
2. **Full Mode**: When Pydantic models are available, uses complete validation

## Claude Tool Calling

These functions are designed for Claude tool calling with the OpenHands format:

```python
# Example Claude tool call
result = debug_gesture_flow(app_instance=app)
if result["status"] == "completed":
    print(f"Found {len(result['issues_detected'])} issues")
    for issue in result["issues_detected"]:
        print(f"- {issue['issue']}: {issue['description']}")
```

## Next Steps

1. **Run comprehensive debug**: Use `run_comprehensive_zoom_debug()` to get full analysis
2. **Apply fixes**: Implement the recommended code changes
3. **Test validation**: Re-run functions to verify fixes
4. **Monitor zoom behavior**: Use functions for ongoing zoom behavior validation

The debugging functions provide a systematic approach to identifying and fixing zoom-related issues in the CHONKER_SNYFTER application.