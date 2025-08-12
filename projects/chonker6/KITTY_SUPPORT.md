# Kitty Terminal Support for Chonker6

## Overview
Chonker6 now includes native support for the Kitty terminal emulator's graphics protocol, enabling direct PDF rendering in the terminal with automatic size fitting.

## Features
- **Automatic PDF Wrapping**: PDFs automatically fit to the left pane width while maintaining aspect ratio
- **Native Image Display**: Uses Kitty's graphics protocol for crisp, clear PDF rendering
- **RGBA Support**: Full 32-bit color with transparency support
- **Dynamic Resizing**: PDF display adjusts to terminal size changes

## How It Works

### Detection
Chonker6 automatically detects Kitty terminal by checking:
1. `TERM` environment variable contains "kitty"
2. `KITTY_WINDOW_ID` environment variable is set

### Rendering Pipeline
1. PDF pages are rendered to RGBA bitmaps using PDFium
2. Bitmap dimensions are calculated to fit the left pane width
3. RGBA data is encoded as base64
4. Image is transmitted using Kitty's graphics protocol

### Protocol Details
The Kitty graphics protocol command used:
```
\x1b_Ga=T,f=32,s={width},{height},t=d;{base64_data}\x1b\\
```
- `a=T`: Transmit image data
- `f=32`: RGBA format (32-bit)
- `s={width},{height}`: Image dimensions in pixels
- `t=d`: Direct transmission (base64)

## Usage

### Running in Kitty
Simply launch chonker6 in a Kitty terminal:
```bash
./target/release/chonker6
```

### Controls
- **Ctrl+O**: Open a PDF file
- **Arrow Keys**: Navigate pages
- **Ctrl+E**: Extract text to edit mode
- **Tab**: Switch between PDF and text panels

### Testing
To test Kitty support in other terminals:
```bash
KITTY_WINDOW_ID=1 TERM=xterm-kitty ./target/release/chonker6
```

## Implementation Details

### Key Files Modified
1. **src/services/pdf_engine.rs**
   - Added `render_page_for_kitty()` method
   - Calculates optimal dimensions for display area
   - Returns RGBA data with dimensions

2. **src/app.rs**
   - Added `is_kitty` field to detect Kitty terminal
   - Implemented `initialize_kitty_mode()` for setup
   - Added `render_pdf_with_kitty()` for image display
   - Integrated Kitty rendering into main render pipeline

### Dependencies
- `pdfium-render`: PDF rendering engine
- `base64`: Image data encoding
- `image`: Image manipulation (already included)

## Advantages Over Text-Based Display
- Full visual PDF rendering
- Maintains formatting and layout
- Supports images and graphics in PDFs
- Better readability for complex documents
- Automatic aspect ratio preservation

## Future Enhancements
- [ ] Support for multiple pages in view
- [ ] Zoom controls with live preview
- [ ] Thumbnail navigation
- [ ] Support for annotations
- [ ] Caching for faster page switching

## Troubleshooting

### Image Not Displaying
1. Verify you're using Kitty terminal: `echo $TERM`
2. Check Kitty version supports graphics protocol: `kitty --version`
3. Ensure PDF file is valid and accessible

### Performance Issues
- Large PDFs may take time to render initially
- Consider reducing display resolution for very large documents
- Ensure sufficient memory for image data

### Fallback Behavior
When not running in Kitty, chonker6 automatically falls back to:
- iTerm2 image protocol (if detected)
- Text-based PDF information display