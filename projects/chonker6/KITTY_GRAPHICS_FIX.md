# Kitty Graphics Protocol Implementation

## What Was Fixed

### Previous Issue
- The PDF panel showed "PDF VIEWER (Kitty) Loading..." text and never displayed the actual PDF
- The placeholder text was always rendered in the frame buffer

### Solution Implemented

1. **Removed placeholder text**: The render function now only clears the background for Kitty terminals, no "Loading..." text
2. **Added page info bar**: Shows "Page X/Y | ←/→: Navigate | +/-: Zoom" at the bottom of the PDF panel
3. **Proper image overlay**: The Kitty graphics protocol sends the PDF image after the frame is rendered
4. **Terminal logging**: Added detailed logging to the terminal panel showing:
   - When a PDF is being rendered
   - Success/failure of rendering
   - Image dimensions

## How It Works

### Rendering Pipeline
1. **Frame rendering** (`app.render()`):
   - Clears PDF panel background
   - Shows page navigation info at bottom
   - Does NOT show any placeholder text

2. **Post-frame rendering** (`render_pdf_with_kitty_post_frame()`):
   - Called only when `kitty_needs_redraw` flag is true
   - Renders PDF page to RGBA bitmap
   - Encodes as base64
   - Sends to terminal using Kitty graphics protocol
   - Chunks large images for protocol compliance

3. **Trigger points**:
   - PDF loaded: Sets `kitty_needs_redraw = true`
   - Page navigation (Left/Right arrows): Sets flag
   - Zoom changes: Sets flag

## Kitty Graphics Protocol Details

### Escape Sequences Used
- `\x1b_Ga=d\x1b\\` - Delete all existing images
- `\x1b_Ga=T,f=32,s=W,H;DATA\x1b\\` - Send image (single chunk)
- `\x1b_Ga=T,f=32,s=W,H,m=1;DATA\x1b\\` - First chunk of multi-part image
- `\x1b_Gm=1;DATA\x1b\\` - Middle chunks
- `\x1b_G;DATA\x1b\\` - Final chunk

### Parameters
- `a=T`: Transmit and display image
- `f=32`: 32-bit RGBA format
- `s=W,H`: Image dimensions in pixels
- `m=1`: More chunks follow (for chunked transfer)

## Testing

### Verify Kitty Support
Run the test script to verify your terminal supports Kitty graphics:
```bash
./test_kitty_graphics.sh
```
You should see a small red square if graphics are working.

### In chonker6
1. Open a PDF with Ctrl+O
2. Check terminal panel (Ctrl+T) for rendering logs
3. Navigate pages with arrow keys
4. Each navigation should show "Rendering PDF page X with Kitty graphics..."

## Troubleshooting

### PDF Not Showing
1. **Check terminal**: Ensure you're using Kitty terminal (not iTerm2, Terminal.app, etc.)
2. **Check logs**: Open terminal panel (Ctrl+T) to see rendering status
3. **Verify TERM**: Should be "xterm-kitty" 
4. **Check KITTY_WINDOW_ID**: Should be set in Kitty

### Common Issues
- **Still showing placeholder**: Old binary - rebuild with `cargo build --release`
- **No image appears**: Check if Kitty graphics are enabled in your Kitty config
- **Partial rendering**: Terminal panel logs will show error details

## Performance Notes
- Images are only rendered when needed (not every frame)
- Large PDFs are chunked to comply with protocol limits (4KB chunks)
- Rendering happens after UI frame to avoid blocking