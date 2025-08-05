#!/bin/bash

# Test script for chonker5 character matrix engine

echo "🐹 Testing CHONKER 5 Character Matrix Engine"

# Check if we can compile
echo "📦 Checking compilation..."
if cargo check --quiet; then
    echo "✅ Compilation check passed"
else
    echo "❌ Compilation failed"
    exit 1
fi

# Try to build
echo "🔨 Building..."
if cargo build --quiet; then
    echo "✅ Build successful"
else
    echo "❌ Build failed"
    exit 1
fi

echo ""
echo "🎯 Ready to run!"
echo "   Run with: cargo run"
echo ""
echo "📋 Key features:"
echo "   • PDF → Smallest character matrix conversion"
echo "   • Vision model identifies text regions in matrix" 
echo "   • Pdfium extracts all text precisely"
echo "   • Maps text into matrix using vision bounding boxes"
echo ""
echo "🎮 Controls:"
echo "   [O] - Open PDF file"
echo "   [M] - Create character matrix representation" 
echo "   [G] - Show debug/analysis info"
echo "   [B] - Toggle character region overlay"
echo ""
echo "Why this approach works:"
echo "• Character matrices preserve spatial layout naturally"
echo "• Vision models are good at identifying text regions"
echo "• Pdfium provides precise text content"
echo "• Combining them creates faithful character representation"
echo "• Smallest viable matrix + vision boxes + exact text = success"
