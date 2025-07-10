# CHONKER Bidirectional Highlighting System

## Overview
Implemented a non-interfering bidirectional highlighting system that provides visual feedback between the PDF viewer (left pane) and table editor (right pane) without disrupting core editing functionality.

## Features Implemented

### 🎯 Edge Marker System
- **Zoom-independent markers** that appear on pane edges to show corresponding positions
- **Color-coded interactions**:
  - 🟠 **Orange markers**: Editor → PDF interactions (when clicking editor cells)
  - 🔵 **Blue markers**: PDF → Editor interactions (when clicking PDF areas)
- **Four edge positions**: left, right, top, bottom with appropriate transforms
- **Pulse animation** for newly created markers with auto-cleanup

### 🔗 Bidirectional Functionality
- **Editor to PDF**: Clicking table cells creates markers on PDF pane edges at corresponding positions
- **PDF to Editor**: Clicking PDF areas creates markers on editor pane edges and attempts to highlight corresponding table cells
- **Coordinate mapping**: Realistic PDF table bounds (15-85% vertical, 10-90% horizontal)
- **Cell highlighting**: PDF clicks within table bounds highlight specific editor cells

### 🎨 Visual System
- **Connection lines** between panes during interactions (temporary, auto-cleanup)
- **Source highlighting** for table cells with enhanced styling
- **Tooltips** showing PDF coordinates when hovering over editor cells
- **Non-interfering design** that preserves keyboard input and contenteditable functionality

### 🛠️ Technical Implementation
- **Event isolation**: PDF click handlers use capture phase and stopPropagation to avoid conflicts
- **CSS-based styling** with explicit color overrides for reliability
- **Marker containers** (.highlight-markers) in both panes for organized DOM structure
- **Automatic cleanup** of markers and connection lines after animations

## Architecture

### Core Classes
- `ChonkerTableEditor`: Main class handling both table editing and highlighting
- **Highlighting methods**:
  - `initHighlightingSystem()`: Initialize all highlighting components
  - `createEdgeMarker()`: Create colored markers on pane edges
  - `addPDFClickHandler()`: Handle PDF click events
  - `highlightCellInPDF()`: Create markers when editor cells are clicked
  - `clearAllMarkers()`: Clean up all visual indicators

### Integration Points
- **Non-intrusive**: No additional event listeners on contenteditable elements
- **Global API**: `window.chonkerHighlighting` for external control
- **Existing workflows**: Integrated with current table editing, zoom, and context menu systems

## Files Modified
- `frontend/index.html`: Added highlighting system to ChonkerTableEditor class
- Added CSS for edge markers, connection lines, and enhanced cell highlighting
- Integrated PDF click handling with existing event system

## Key Decisions
1. **Edge markers over overlay highlighting**: More robust across zoom levels and layouts
2. **Explicit color styling**: Inline styles to override any CSS conflicts
3. **Event capture**: Prevents interference with global click handlers
4. **Temporary visual feedback**: Auto-cleanup prevents UI clutter
5. **Preserved core functionality**: Table editing remains fully functional regardless of highlighting state

## Usage
The system activates automatically when the table editor initializes. Users can:
- Click any table cell to see corresponding PDF position markers
- Click anywhere on the PDF to see corresponding editor position markers
- Continue normal table editing without interference
- Use zoom controls independently on both panes

## Status
✅ **Core functionality implemented and cleaned up**  
✅ **Table editing preserved and enhanced**  
✅ **Visual feedback system operational**  
⚠️ **Color differentiation and PDF click consistency may need refinement**

## Future Improvements
- Extract actual coordinate data from PDF for precise mapping
- Add configurability for table bounds and marker styles  
- Implement more sophisticated coordinate transformation algorithms
- Add user preferences for enabling/disabling highlighting features
