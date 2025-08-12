#!/bin/bash

# Test script for chonker6 with Kitty terminal emulation

echo "Testing chonker6 with Kitty terminal settings..."

# Set environment to simulate Kitty
export TERM="xterm-kitty"
export KITTY_WINDOW_ID="1"

# Run chonker6
./target/release/chonker6

# Reset terminal on exit
echo "Test completed."