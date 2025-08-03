# CHONKER TODO

## Current Issues

### 🐛 Drag-and-Drop Not Working
- **Issue**: Click-and-drag functionality implemented but not responding in QWebEngineView
- **Status**: JavaScript is loading (syntax errors fixed) but drag events not firing
- **Code Location**: `chonker.py` lines 1968-2050 (JavaScript drag implementation)

#### Potential Solutions:
1. Debug JavaScript execution in QWebEngineView console
2. Try Qt-native drag implementation instead of JavaScript
3. Add console.log statements to trace event flow
4. Check if contenteditable is interfering with mouse events

## Completed Features ✅
- Smart text wrapping based on content type
- Smart vertical spacing for compact display  
- Oversized/rotated page handling
- Clean UI without distracting borders
- Unified chrome colors (#525659)
- Text editing with contenteditable
- Table cell wrapping for long content

## Future Enhancements
- [ ] Fix drag-and-drop functionality
- [ ] Add OCR for image-based tables
- [ ] Implement proper export features
- [ ] Add undo/redo for edits and moves
- [ ] Save edited PDF layouts