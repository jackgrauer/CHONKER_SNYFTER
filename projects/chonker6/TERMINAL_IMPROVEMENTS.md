# Terminal Panel Improvements

## Changes Made

### 1. Fixed Slice Panic Bug
- **Issue**: Terminal panel would panic with "slice index starts at 3 but ends at 1" when selecting text
- **Fix**: Added proper bounds checking and index swapping in `get_selected_text()` to ensure start <= end

### 2. Full History Scrolling
- **Previous**: Only showed latest 5 lines
- **Now**: Can scroll through entire 1000-line history
- **Scroll Methods**:
  - PageUp/PageDown keys
  - Mouse wheel when hovering over terminal panel
  - Maintains position when new content is added (only auto-scrolls if already at bottom)

### 3. Theme Integration
- **Previous**: Classic green terminal colors
- **Now**: Matches app's color scheme:
  - Background: Dark blue-gray (30, 34, 42)
  - Text: Light purple (180, 180, 200)
  - Selection: Teal highlight (22, 160, 133) - same as main editor
  - Border: Subtle gray matching focused panels

### 4. Removed Auto-Copy
- **Previous**: Automatically copied text on mouse release
- **Now**: Manual copy with Ctrl+C (standard behavior)
- Selection remains visible until manually cleared

### 5. Scroll Position Indicator
- **Title Bar**: Shows current position like "Terminal Output [25/150]"
- **Visual Hints**: Shows "▼ More below ▼" or "▲ Scroll for more ▲" when content extends beyond view

### 6. Enhanced Mouse Support
- **Mouse Wheel**: Scrolls terminal content when hovering over terminal panel
- **Click and Drag**: Select multiple lines of text
- **Ctrl+C**: Copy selected text to clipboard
- **File Selector**: Mouse wheel scrolls file list
- **PDF Panel**: Mouse wheel navigates pages

## Terminal Panel Controls

| Action | Shortcut | Description |
|--------|----------|-------------|
| Toggle Panel | Ctrl+T | Show/hide terminal panel |
| Scroll Up | PageUp or Mouse Wheel Up | Scroll up through history |
| Scroll Down | PageDown or Mouse Wheel Down | Scroll down through history |
| Clear Output | Ctrl+Shift+K | Clear all terminal output |
| Select Text | Click+Drag | Select lines in terminal |
| Copy Selection | Ctrl+C | Copy selected text to clipboard |

## Benefits

1. **Better Navigation**: Full scrolling through entire history instead of just recent lines
2. **Visual Consistency**: Terminal panel matches app theme instead of jarring green-on-black
3. **Standard Behavior**: Manual copy (Ctrl+C) instead of auto-copy follows standard UI conventions
4. **Clear Feedback**: Scroll position indicator shows where you are in the history
5. **Mouse Integration**: Consistent mouse wheel behavior across all panels