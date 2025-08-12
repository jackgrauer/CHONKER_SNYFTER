#!/bin/bash

# Install chonker6 command system-wide

echo "Installing chonker6 command..."

# Create the launcher script
cat > /tmp/chonker6 << 'EOF'
#!/bin/bash

# Chonker6 launcher - Terminal PDF viewer with text extraction
CHONKER6_HOME="/Users/jack/chonker6/projects/chonker6"

# Check if installation exists
if [ ! -d "$CHONKER6_HOME" ]; then
    echo "Error: Chonker6 not found at $CHONKER6_HOME"
    exit 1
fi

# Check if binary exists, build if needed
if [ ! -f "$CHONKER6_HOME/target/release/chonker6" ]; then
    echo "Building chonker6 for first time use..."
    cd "$CHONKER6_HOME"
    cargo build --release || {
        echo "Failed to build chonker6"
        exit 1
    }
fi

# Set library path and run
export DYLD_LIBRARY_PATH="$CHONKER6_HOME/lib:$DYLD_LIBRARY_PATH"
cd "$CHONKER6_HOME"
exec "$CHONKER6_HOME/target/release/chonker6" "$@"
EOF

# Make it executable
chmod +x /tmp/chonker6

# Try to install to /usr/local/bin (may need sudo)
if [ -w /usr/local/bin ]; then
    mv /tmp/chonker6 /usr/local/bin/chonker6
    echo "✓ Installed to /usr/local/bin/chonker6"
else
    echo "Need sudo permission to install to /usr/local/bin"
    sudo mv /tmp/chonker6 /usr/local/bin/chonker6
    echo "✓ Installed to /usr/local/bin/chonker6"
fi

echo ""
echo "Installation complete!"
echo ""
echo "You can now run 'chonker6' from anywhere."
echo "The alias in .zshrc has been updated as backup."
echo ""
echo "To test, open a new terminal and type: chonker6"