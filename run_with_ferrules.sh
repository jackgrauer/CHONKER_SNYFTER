#!/bin/bash
set -e

echo "🚀 Building chonker5 with Ferrules vision integration..."
echo "=================================================="

# Build with ferrules feature enabled
cargo build --release --features ferrules

if [ $? -eq 0 ]; then
    echo "✅ Build successful!"
    echo ""
    echo "🏃 Running chonker5 with Ferrules ML model..."
    ./target/release/chonker5
else
    echo "❌ Build failed. Checking if we need to build Ferrules first..."
    
    # Try building ferrules-core first
    cd ferrules/ferrules-core
    cargo build --release
    cd ../..
    
    # Try again
    cargo build --release --features ferrules
    
    if [ $? -eq 0 ]; then
        echo "✅ Build successful after building Ferrules!"
        ./target/release/chonker5
    else
        echo "❌ Still failing. Running without Ferrules for now..."
        cargo run --release
    fi
fi