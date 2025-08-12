# Fix Summary: Infinite "/" Symbol Issue

## Problem
When selecting a PDF file using Ctrl+O, the terminal would start printing infinite "/" symbols, making the application unusable.

## Root Cause
The Kitty PDF rendering function `render_pdf_with_kitty_post_frame()` was being called **EVERY FRAME** in the main loop when a PDF was loaded. This meant:
- The entire PDF was being rendered to RGBA bytes
- Converted to base64 (massive string)
- Printed to the terminal via Kitty graphics protocol
- This happened 60+ times per second!

The terminal was being flooded with base64 image data continuously, causing the display corruption.

## Solution
Added a flag-based rendering system that only renders the PDF image when needed:

### 1. Added `kitty_needs_redraw` Flag
```rust
pub struct App {
    // ... other fields ...
    kitty_needs_redraw: bool,  // New flag
}
```

### 2. Set Flag When PDF Changes
- When a PDF is loaded: `self.kitty_needs_redraw = true`
- When navigating pages: `self.kitty_needs_redraw = true`

### 3. Modified Main Loop
Changed from:
```rust
// This ran EVERY frame!
if app.is_kitty_terminal() && app.has_pdf_loaded() {
    app.render_pdf_with_kitty_post_frame(pdf_area);
}
```

To:
```rust
// Only runs when flag is set
if app.should_render_kitty_pdf() {
    app.render_pdf_with_kitty_post_frame(pdf_area);
}
```

### 4. Clear Flag After Rendering
The `should_render_kitty_pdf()` method:
```rust
pub fn should_render_kitty_pdf(&mut self) -> bool {
    if self.is_kitty && self.state.pdf.is_loaded() && self.kitty_needs_redraw {
        self.kitty_needs_redraw = false;  // Clear flag after check
        true
    } else {
        false
    }
}
```

## Result
- PDF is only rendered to Kitty when first loaded or when pages change
- No more continuous base64 flooding
- Terminal remains responsive
- File selector works properly
- Navigation works as expected

## Performance Impact
Before: Rendering massive base64 strings 60+ times per second
After: Rendering only when PDF content actually changes

This is a massive performance improvement and fixes the terminal corruption issue completely.