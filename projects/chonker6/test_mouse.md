# Mouse Click Functionality Test

## Features Added

### 1. File Selector Mouse Support
- **Click on files/folders**: Click any file or folder in the file selector to select and open it
- **Single click navigation**: Click on directories to enter them, click on PDF files to open them
- **Visual feedback**: The clicked item becomes selected before opening

### 2. Text Editor Mouse Support
- **Click to position cursor**: Click anywhere in the text matrix to position the cursor at that location
- **Click and drag to select**: Click and drag to select text
- **Block selection with ALT**: Hold ALT while clicking and dragging for block/rectangular selection
- **Visual feedback**: Cursor position updates immediately on click

## How to Test

1. **File Selector Click Test**:
   - Press `Ctrl+O` to open file selector
   - Click on any PDF file to open it directly
   - Click on folders to navigate into them
   - Click on ".." to go to parent directory

2. **Text Editor Click Test**:
   - Open a PDF and extract text (`Ctrl+E`)
   - Press `i` to enter edit mode
   - Click anywhere in the text to position cursor
   - Click and drag to select text
   - Hold ALT and drag for block selection

## Implementation Details

- File selector tracks click position and maps it to the file list
- Text editor converts screen coordinates to matrix positions
- Single clicks position cursor without starting selection
- Drag operations create selections
- ALT modifier enables block selection mode