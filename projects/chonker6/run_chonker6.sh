#!/bin/bash

# Run chonker6 with proper library path
# This ensures PDFium library can be found

# Get the directory where this script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# Set library path for PDFium
export DYLD_LIBRARY_PATH="$SCRIPT_DIR/lib:$DYLD_LIBRARY_PATH"

# Change to script directory so relative paths work
cd "$SCRIPT_DIR"

# Check if binary exists
if [ ! -f "target/release/chonker6" ]; then
    echo "Building chonker6..."
    cargo build --release
fi

# Run chonker6
echo "Starting chonker6 with PDFium support..."
echo "Library path: $SCRIPT_DIR/lib"
./target/release/chonker6 "$@"