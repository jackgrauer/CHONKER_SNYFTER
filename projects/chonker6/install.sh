#!/bin/bash
# Install script for Chonker6 - makes it available globally

echo "📦 Installing Chonker6 globally..."
echo ""

# Build in release mode if not already built
if [ ! -f "/Users/jack/chonker6/target/release/chonker6" ]; then
    echo "Building Chonker6 in release mode..."
    cd /Users/jack/chonker6
    cargo build --release
fi

# Option 1: Symlink to /usr/local/bin (recommended)
echo "Creating symlink in /usr/local/bin..."
sudo ln -sf /Users/jack/chonker6/target/release/chonker6 /usr/local/bin/chonker6

# Option 2: Copy the binary (alternative)
# echo "Copying binary to /usr/local/bin..."
# sudo cp /Users/jack/chonker6/target/release/chonker6 /usr/local/bin/chonker6

echo ""
echo "✅ Installation complete!"
echo ""
echo "You can now run Chonker6 from anywhere by typing:"
echo "  chonker6"
echo ""
echo "To uninstall later, run:"
echo "  sudo rm /usr/local/bin/chonker6"