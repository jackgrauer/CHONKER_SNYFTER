#!/bin/bash
# Launcher script for Chonker5 TUI - requires Kitty terminal

cd /Users/jack/chonker5

# Check if Kitty is installed
if ! command -v kitty &> /dev/null; then
    echo "Error: Kitty terminal is not installed"
    echo "Install with: brew install kitty (macOS)"
    echo "Or visit: https://sw.kovidgoyal.net/kitty/"
    exit 1
fi

# Check if we're already running in Kitty
if [[ "$TERM" == *"kitty"* ]] || [[ "$TERM_PROGRAM" == "kitty" ]]; then
    # Already in Kitty, run directly
    echo "Running Chonker5 TUI in Kitty..."
    DYLD_LIBRARY_PATH=/Users/jack/chonker5/lib ./target/release/chonker5-tui "$@"
else
    # Launch in Kitty with optimal settings
    echo "Launching Chonker5 TUI in Kitty terminal..."
    kitty --single-instance \
          --title "Chonker5 TUI" \
          --override font_size=12 \
          --override cursor_shape=block \
          --override cursor_blink_interval=0.5 \
          --override remember_window_size=yes \
          --override window_padding_width=2 \
          bash -c "cd /Users/jack/chonker5 && DYLD_LIBRARY_PATH=/Users/jack/chonker5/lib ./target/release/chonker5-tui $*"
fi