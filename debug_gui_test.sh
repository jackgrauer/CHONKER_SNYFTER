#!/bin/bash

# CHONKER GUI Debug Test Script
# Automatically loads a PDF, processes it, and logs the output

echo "🐹 CHONKER GUI Debug Test"
echo "========================="

# Set up debug environment - more targeted logging
export RUST_LOG=chonker_tui::smart_chunker=debug,chonker_tui::extraction=debug,chonker_tui::app=info,sqlx=warn
export RUST_BACKTRACE=1
export CHONKER_DEBUG=1
export CHONKER_LOG_LEVEL=info

# Create logs directory
mkdir -p logs

# Clean up old logs
rm -f logs/gui_debug.log

# Check if we have a test PDF
TEST_PDF="/Users/jack/Documents/1.pdf"
if [ ! -f "$TEST_PDF" ]; then
    echo "❌ Test PDF not found at $TEST_PDF"
    echo "Please ensure you have a PDF file at this location for testing"
    exit 1
fi

echo "📄 Using test PDF: $TEST_PDF"
echo "🔧 Debug environment configured"
echo "🚀 Starting GUI with debug logging..."
echo ""

# Run the GUI application with debug output
# The GUI will log to both console and file
./target/release/chonker_gui 2>&1 | tee logs/gui_debug.log &

# Get the PID of the GUI process
GUI_PID=$!

echo "🎯 GUI started with PID: $GUI_PID"
echo "📝 Debug output is being logged to: logs/gui_debug.log"
echo ""
echo "🔍 To debug the text display issue:"
echo "   1. The GUI should now be running"
echo "   2. It should auto-load $TEST_PDF"
echo "   3. Press SPACE to process the document"
echo "   4. Check Panel B for the text output"
echo "   5. Press Ctrl+C here to stop and view logs"
echo ""
echo "💡 Watching for processing completion..."

# Monitor the log file for processing completion
tail -f logs/gui_debug.log &
TAIL_PID=$!

# Wait for user interrupt
trap 'echo ""; echo "🛑 Stopping debug session..."; kill $GUI_PID 2>/dev/null; kill $TAIL_PID 2>/dev/null; exit 0' INT

wait $GUI_PID

echo ""
echo "📋 GUI session ended. Final log contents:"
echo "=========================================="
cat logs/gui_debug.log

echo ""
echo "🔍 Analysis complete. Check the logs above for text processing details."
