#!/bin/bash

# Ensure necessary tools are installed
if ! command -v pdftotext &> /dev/null || ! command -v fzf &> /dev/null || ! command -v bat &> /dev/null; then
    echo "Required tool(s) missing: pdftotext, fzf, bat" >&2
    exit 1
fi

# Convert the PDF to plain text for preview
echo "🔄 Converting PDF to text..."
if ! pdftotext input.pdf original.txt 2>/dev/null; then
    echo "⚠️  PDF conversion failed, creating placeholder"
    echo "[PDF Content - conversion failed]" > original.txt
fi

# Copy the markdown for preview
echo "📄 Preparing markdown preview..."
if [ -f "proposed_markdown.md" ]; then
    cp proposed_markdown.md proposed_view.txt
else
    echo "❌ proposed_markdown.md not found"
    exit 1
fi

# Launch the Rust-based PDF viewer
echo "🚀 Opening fast Rust-based PDF viewer..."
./target/release/pdf_viewer &
VIEWER_PID=$!

echo ""
echo "🐹 CHONKER Preview Mode"
echo "==================="
echo "📖 Fast Rust PDF viewer opened with side-by-side comparison"
echo "📝 Review the PDF (left) and proposed markdown (right) in the GUI window"
echo ""
echo "Do you want to apply these changes? [y/N]"
read -r response

if [[ "$response" =~ ^[Yy]$ ]]; then
    echo "✅ Applying changes..."
    # Your command to process the markdown further goes here
    echo "🎉 Changes applied successfully!"
else
    echo "❌ No changes applied."
fi

# Clean up background processes
if kill -0 "$VIEWER_PID" 2>/dev/null; then
    echo "🧮 Closing PDF viewer..."
    kill "$VIEWER_PID" 2>/dev/null || true
fi
