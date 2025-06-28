#!/bin/bash

# CHONKER TUI Debug Development Script
# This script sets up an enhanced debugging environment for the CHONKER TUI application

echo "🐹 CHONKER TUI Debug Environment"
echo "================================="

# Set up debug environment variables - filter out noisy database logs
export RUST_LOG=chonker_tui=debug,sqlx=warn,info
export RUST_BACKTRACE=1
export CHONKER_DEBUG=1
export CHONKER_LOG_LEVEL=debug

# Create logs directory if it doesn't exist
mkdir -p logs

# Clean up old logs
rm -f logs/chonker_debug.log
rm -f logs/chonker_performance.log

echo "🔧 Environment configured:"
echo "   RUST_LOG: $RUST_LOG"
echo "   RUST_BACKTRACE: $RUST_BACKTRACE"
echo "   CHONKER_DEBUG: $CHONKER_DEBUG"
echo "   CHONKER_LOG_LEVEL: $CHONKER_LOG_LEVEL"
echo ""

# Build in debug mode
echo "🏗️  Building CHONKER in debug mode..."
cargo build --features debug

if [ $? -eq 0 ]; then
    echo "✅ Build successful!"
    echo ""
    echo "🚀 Starting CHONKER TUI with debug logging..."
    echo "   Log files will be created in: logs/"
    echo "   Press Ctrl+C to stop and view logs"
    echo ""
    
    # Run the application
    cargo run --features debug
    
    echo ""
    echo "📋 Debug session complete. Log files:"
    echo "   - logs/chonker_debug.log (application logs)"
    echo "   - logs/chonker_performance.log (performance metrics)"
    
    # Optionally show recent logs
    if [ -f logs/chonker_debug.log ]; then
        echo ""
        echo "🔍 Recent debug logs:"
        tail -20 logs/chonker_debug.log
    fi
else
    echo "❌ Build failed. Please check the compilation errors above."
    exit 1
fi
