# Chonker6 Command Setup Complete! ✅

## What Was Done

1. **Created executable command** at `/Users/jack/.local/bin/chonker6`
   - Automatically sets library path for PDFium
   - Changes to correct directory
   - Handles all the complexity for you

2. **Updated .zshrc** to:
   - Add `~/.local/bin` to your PATH
   - Removed the old alias (no longer needed)
   - Kept the `chonker` alias for chonker5

3. **Installation location**:
   ```
   /Users/jack/.local/bin/chonker6  # The command
   /Users/jack/chonker6/projects/chonker6/  # The actual program
   ```

## How to Use

### After opening a new terminal (or running `source ~/.zshrc`):

Just type:
```bash
chonker6
```

That's it! No need for:
- Setting DYLD_LIBRARY_PATH
- Changing directories  
- Running special scripts
- Remembering paths

## What Happens When You Type "chonker6"

1. Shell finds `/Users/jack/.local/bin/chonker6` in your PATH
2. The script automatically:
   - Sets `DYLD_LIBRARY_PATH` to include the PDFium library
   - Changes to the project directory
   - Runs the actual chonker6 binary
   - Passes along any arguments

## Troubleshooting

### "command not found: chonker6"
Open a new terminal or run:
```bash
source ~/.zshrc
```

### Verify it's in your PATH:
```bash
which chonker6
# Should show: /Users/jack/.local/bin/chonker6
```

### Test the setup:
```bash
echo $PATH | grep .local/bin
# Should show .local/bin in the path
```

## Features Available

Once you type `chonker6`:
- **Ctrl+O**: Open PDF file browser
- **Arrow keys**: Navigate PDF pages
- **Ctrl+E**: Extract text from current page
- **Ctrl+T**: Toggle terminal output panel  
- **Ctrl+Q**: Quit

## The Old Way vs New Way

### Old (what you had before):
```bash
alias chonker6="DYLD_LIBRARY_PATH=/Users/jack/chonker6/lib /Users/jack/chonker6/target/release/chonker6"
```
Problem: Wrong paths after reorganization

### New (what you have now):
```bash
chonker6  # Just works!
```
The command at `~/.local/bin/chonker6` handles everything

## Updating Chonker6

If you make changes to the code:
```bash
cd /Users/jack/chonker6/projects/chonker6
cargo build --release
```

The `chonker6` command will automatically use the updated binary.

## Complete Reinstall (if ever needed)

```bash
rm ~/.local/bin/chonker6
cp /Users/jack/chonker6/projects/chonker6/install_chonker6.sh /tmp/
/tmp/install_chonker6.sh
```