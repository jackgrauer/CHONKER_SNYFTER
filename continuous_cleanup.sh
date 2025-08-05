#!/bin/bash

# Continuous Cleanup System for chonker5.rs
# This script monitors changes and maintains code quality automatically

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
WATCH_FILE="chonker5.rs"
CLEANUP_LOG="cleanup.log"
LAST_HASH_FILE=".last_cleanup_hash"
CLEANUP_INTERVAL=1800  # seconds between checks (30 minutes)

# Initialize
echo -e "${BLUE}🧹 Continuous Cleanup System Started${NC}"
echo "Monitoring: $WATCH_FILE"
echo "Check interval: $(($CLEANUP_INTERVAL / 60)) minutes"
echo "Press Ctrl+C to stop"
echo ""

# Create initial hash if file exists
if [ -f "$WATCH_FILE" ]; then
    md5sum "$WATCH_FILE" | cut -d' ' -f1 > "$LAST_HASH_FILE"
fi

# Function to get file hash
get_file_hash() {
    if [ -f "$WATCH_FILE" ]; then
        md5sum "$WATCH_FILE" | cut -d' ' -f1
    else
        echo "none"
    fi
}

# Function to run cleanup
run_cleanup() {
    echo -e "${YELLOW}🔄 Running cleanup checks...${NC}"
    
    # Create timestamp
    TIMESTAMP=$(date "+%Y-%m-%d %H:%M:%S")
    echo "[$TIMESTAMP] Starting cleanup" >> "$CLEANUP_LOG"
    
    # Track issues found
    ISSUES_FOUND=0
    
    # 1. Check compilation
    echo -n "  • Checking compilation... "
    if cargo check --quiet 2>/dev/null; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${RED}✗${NC}"
        echo "    Fix: Review compilation errors"
        ((ISSUES_FOUND++))
    fi
    
    # 2. Run clippy
    echo -n "  • Running clippy... "
    CLIPPY_OUTPUT=$(cargo clippy --quiet -- -W clippy::all 2>&1 || true)
    if [ -z "$CLIPPY_OUTPUT" ]; then
        echo -e "${GREEN}✓${NC}"
    else
        CLIPPY_COUNT=$(echo "$CLIPPY_OUTPUT" | grep -c "warning:" || true)
        echo -e "${YELLOW}${CLIPPY_COUNT} warnings${NC}"
        ((ISSUES_FOUND+=$CLIPPY_COUNT))
    fi
    
    # 3. Check formatting
    echo -n "  • Checking formatting... "
    if cargo fmt -- --check >/dev/null 2>&1; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${YELLOW}needs formatting${NC}"
        cargo fmt --quiet
        echo "    Applied: cargo fmt"
        ((ISSUES_FOUND++))
    fi
    
    # 4. Check for common issues
    echo -n "  • Checking for unwrap() calls... "
    UNWRAP_COUNT=$(grep -c "\.unwrap()" "$WATCH_FILE" || true)
    if [ "$UNWRAP_COUNT" -eq 0 ]; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${YELLOW}${UNWRAP_COUNT} found${NC}"
        echo "    Consider: Replace with proper error handling"
    fi
    
    # 5. Check for TODO comments
    echo -n "  • Checking for TODOs... "
    TODO_COUNT=$(grep -c "TODO\|FIXME\|XXX" "$WATCH_FILE" || true)
    if [ "$TODO_COUNT" -eq 0 ]; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${BLUE}${TODO_COUNT} found${NC}"
    fi
    
    # 6. Run tests if they exist
    if [ -d "tests" ] || grep -q "#\[test\]" "$WATCH_FILE"; then
        echo -n "  • Running tests... "
        if cargo test --quiet 2>/dev/null; then
            echo -e "${GREEN}✓${NC}"
        else
            echo -e "${RED}✗${NC}"
            echo "    Fix: Review failing tests"
            ((ISSUES_FOUND++))
        fi
    fi
    
    # Summary
    echo ""
    if [ "$ISSUES_FOUND" -eq 0 ]; then
        echo -e "${GREEN}✨ Code is clean!${NC}"
    else
        echo -e "${YELLOW}📋 Found $ISSUES_FOUND issue(s) to address${NC}"
    fi
    
    echo "[$TIMESTAMP] Completed - Issues: $ISSUES_FOUND" >> "$CLEANUP_LOG"
    echo ""
}

# Function to run quick cleanup
run_quick_cleanup() {
    # Auto-fix what we can
    cargo fmt --quiet 2>/dev/null || true
    
    # Log the action
    echo "[$(date "+%Y-%m-%d %H:%M:%S")] Quick cleanup applied" >> "$CLEANUP_LOG"
}

# Main monitoring loop
while true; do
    CURRENT_HASH=$(get_file_hash)
    
    if [ -f "$LAST_HASH_FILE" ]; then
        LAST_HASH=$(cat "$LAST_HASH_FILE")
        
        if [ "$CURRENT_HASH" != "$LAST_HASH" ] && [ "$CURRENT_HASH" != "none" ]; then
            echo -e "${BLUE}📝 Change detected!${NC}"
            
            # Quick cleanup first
            run_quick_cleanup
            
            # Full cleanup check
            run_cleanup
            
            # Update hash
            echo "$CURRENT_HASH" > "$LAST_HASH_FILE"
        fi
    else
        echo "$CURRENT_HASH" > "$LAST_HASH_FILE"
    fi
    
    sleep "$CLEANUP_INTERVAL"
done