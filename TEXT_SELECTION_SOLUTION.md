# Text Selection in Terminal UIs

## The Problem

**Terminal text selection will ALWAYS bleed across panes** in any TUI application. This is a fundamental limitation of how terminal emulators work:

- The terminal sees your TUI as a **flat grid of characters**
- It has **no concept of "panes" or "windows"**
- Selection naturally flows from right edge to left edge of next line
- **No amount of separators, borders, or gaps can stop this**

## Why This Happens

```
Your TUI shows:          Terminal sees:
┌─────────┬─────────┐    PDF text...Markdown text...
│ PDF     │ Markdown│    more PDF...more Markdown...
│ text... │ text... │    
└─────────┴─────────┘
```

When you select "text..." in the PDF pane, the terminal continues into the Markdown pane because it doesn't understand the visual boundary.

## The Solution That Actually Works

### ✅ Zoom Mode (Press 'z')

Press `z` to cycle through view modes:
1. **Split view** (selection bleeds)
2. **Left pane only** (clean selection) 
3. **Right pane only** (clean selection)

In single-pane mode, there's **nothing to bleed into** - problem solved!

### This is the Professional Solution

- **tmux**: Press `C-b z` to zoom current pane
- **vim**: Use splits with `:only` to focus one pane
- **lazygit**: Dedicated view modes for each panel
- **k9s**: Single-panel views for clean selection

## Other Attempted Solutions (That Don't Work)

❌ **Mouse capture**: Can intercept events but can't prevent terminal selection  
❌ **Wide separators**: Terminal still sees continuous text  
❌ **Different background colors**: Visual only, doesn't affect selection  
❌ **Borders and margins**: Terminal ignores TUI layout  

## Usage in CHONKER

- **'z'**: Toggle between Split → Left Only → Right Only
- **'m'**: Toggle mouse mode (for educational purposes)
- **'c'**: Copy current chunk (works in any mode)

The zoom mode gives you the **same functionality** as split view but with **perfect text selection**.

## Technical Details

This limitation exists because:

1. **Terminal emulators** handle text selection at the character grid level
2. **TUI frameworks** (ratatui, ncurses, etc.) just draw characters
3. **No communication** exists between TUI layout and terminal selection
4. This is **by design** - terminals are simple, universal interfaces

The zoom solution works because it removes the fundamental cause: having multiple content areas in the same terminal view.
