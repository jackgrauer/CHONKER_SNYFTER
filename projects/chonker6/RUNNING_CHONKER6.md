# How to Run Chonker6

## Quick Start

### From the project directory:
```bash
cd /Users/jack/chonker6/projects/chonker6
./run_chonker6.sh
```

### From anywhere:
```bash
/Users/jack/chonker6/chonker6
```

## Why the Special Script?

Chonker6 uses PDFium library for PDF rendering. The library (`libpdfium.dylib`) is bundled in the `lib/` directory, but macOS needs to know where to find it.

## The PDF Engine Error

If you see "PDF engine not initialized" or "PDF engine not available", it means PDFium couldn't be loaded.

### Fix Option 1: Use the run script
```bash
./run_chonker6.sh
```
This script automatically sets `DYLD_LIBRARY_PATH` to include the lib directory.

### Fix Option 2: Set library path manually
```bash
export DYLD_LIBRARY_PATH=/Users/jack/chonker6/projects/chonker6/lib:$DYLD_LIBRARY_PATH
./target/release/chonker6
```

### Fix Option 3: Run from project directory
```bash
cd /Users/jack/chonker6/projects/chonker6
DYLD_LIBRARY_PATH=./lib ./target/release/chonker6
```

## Troubleshooting

### "PDF engine not initialized"
- **Cause**: PDFium library not found
- **Solution**: Use `run_chonker6.sh` or set `DYLD_LIBRARY_PATH`

### Library Security Warning on macOS
If macOS blocks the library for security:
```bash
# Remove quarantine attribute
xattr -d com.apple.quarantine lib/libpdfium.dylib
```

### Verify Library is Present
```bash
ls -la /Users/jack/chonker6/projects/chonker6/lib/libpdfium.dylib
# Should show: -rw-r--r--@ 1 jack staff 5760688 ...
```

## Terminal Requirements

### For PDF Rendering
- **Kitty Terminal**: Full graphical PDF rendering
  - Install: `brew install kitty`
  - Run chonker6 inside Kitty
  
- **Other Terminals**: Text-only PDF info
  - Will show page count and navigation
  - No actual PDF image

### Verify Kitty Support
```bash
echo $TERM
# Should show: xterm-kitty

echo $KITTY_WINDOW_ID  
# Should show a number (e.g., 1)
```

## Complete Setup from Scratch

1. **Build the project**:
   ```bash
   cd /Users/jack/chonker6/projects/chonker6
   cargo build --release
   ```

2. **Make scripts executable**:
   ```bash
   chmod +x run_chonker6.sh
   chmod +x /Users/jack/chonker6/chonker6
   ```

3. **Run with proper library path**:
   ```bash
   ./run_chonker6.sh
   ```

## Features Available

Once running properly with PDF engine initialized:
- **Ctrl+O**: Open PDF file browser
- **←/→**: Navigate pages
- **Ctrl+E**: Extract text from current page
- **Ctrl+T**: Toggle terminal output panel
- **Ctrl+Q**: Quit

## Creating an Alias

Add to your `~/.zshrc` or `~/.bashrc`:
```bash
alias chonker6='/Users/jack/chonker6/chonker6'
```

Then you can run from anywhere:
```bash
chonker6
```