#!/bin/bash
set -e

echo "🧪 Testing Ferrules Integration (Simple Test)"
echo "==========================================="
echo ""

# Check if we can build the test binary with ferrules
echo "📦 Building test with Ferrules feature..."
if cargo build --bin test_ferrules_integration --features ferrules 2>/dev/null; then
    echo "✅ Build successful with Ferrules!"
    echo ""
    echo "🏃 Running test..."
    ./target/debug/test_ferrules_integration
    
    # Check if it's using ferrules
    if ./target/debug/test_ferrules_integration 2>&1 | grep -q "Using Ferrules ML model"; then
        echo ""
        echo "🎉 SUCCESS: Ferrules ML model is active!"
    else
        echo ""
        echo "⚠️  WARNING: Still using flood-fill fallback"
    fi
else
    echo "❌ Build failed with Ferrules"
    echo ""
    echo "🔧 Building without Ferrules (flood-fill only)..."
    cargo build --bin test_ferrules_integration
    ./target/debug/test_ferrules_integration
fi