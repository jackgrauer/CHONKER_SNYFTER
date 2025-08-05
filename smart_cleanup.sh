#!/bin/bash

# Smart Cleanup System with Sub-Agent Integration
# Monitors changes and triggers intelligent cleanup when needed

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
NC='\033[0m'

# Configuration
WATCH_FILE="chonker5.rs"
CHANGES_THRESHOLD=10  # Trigger deep cleanup after N changes
COMPLEXITY_THRESHOLD=100  # Lines changed before triggering sub-agent
CHANGE_COUNTER_FILE=".change_counter"
LAST_DEEP_CLEANUP=".last_deep_cleanup"

# Initialize counter
if [ ! -f "$CHANGE_COUNTER_FILE" ]; then
    echo "0" > "$CHANGE_COUNTER_FILE"
fi

echo -e "${MAGENTA}🤖 Smart Cleanup System Initialized${NC}"
echo "Monitoring: $WATCH_FILE"
echo "Quick cleanup: Every change"
echo "Deep cleanup: Every $CHANGES_THRESHOLD changes"
echo "Sub-agent cleanup: Large changes (>$COMPLEXITY_THRESHOLD lines)"
echo ""

# Function to count changed lines
count_changed_lines() {
    if [ -f ".last_file_backup" ]; then
        diff -u ".last_file_backup" "$WATCH_FILE" 2>/dev/null | grep -E "^[+-]" | grep -v "^[+-]{3}" | wc -l || echo "0"
    else
        echo "0"
    fi
}

# Function to trigger sub-agent cleanup
trigger_subagent_cleanup() {
    echo -e "${MAGENTA}🤖 Triggering AI-powered deep cleanup...${NC}"
    
    # Create a marker file for the sub-agent
    cat > .cleanup_request.md << EOF
# Cleanup Request for chonker5.rs

## Context
The file has undergone significant changes and needs comprehensive cleanup.

## Tasks
1. Remove any dead code or commented sections
2. Improve error handling patterns
3. Consolidate duplicate logic
4. Ensure consistent naming conventions
5. Add missing documentation
6. Optimize performance bottlenecks
7. Run all quality checks

## Quality Standards
- All clippy warnings should be addressed
- Tests should pass
- Documentation should be complete
- Error handling should be robust
EOF
    
    echo "Sub-agent cleanup requested. The AI assistant will handle deep cleanup."
    echo "[$(date "+%Y-%m-%d %H:%M:%S")] Sub-agent cleanup triggered" >> cleanup.log
}

# Function for quick cleanup
quick_cleanup() {
    echo -e "${BLUE}⚡ Running quick cleanup...${NC}"
    
    # Auto-format
    cargo fmt --quiet 2>/dev/null || true
    
    # Remove trailing whitespace
    sed -i '' 's/[[:space:]]*$//' "$WATCH_FILE" 2>/dev/null || true
    
    # Basic clippy fixes that can be auto-applied
    cargo clippy --fix --allow-dirty --allow-staged --quiet 2>/dev/null || true
    
    echo -e "${GREEN}✓ Quick cleanup complete${NC}"
}

# Function for periodic deep cleanup
deep_cleanup() {
    echo -e "${YELLOW}🧹 Running deep cleanup...${NC}"
    
    # Run comprehensive checks
    ./quality_check.sh 2>/dev/null || echo "Quality check not available"
    
    # Reset counter
    echo "0" > "$CHANGE_COUNTER_FILE"
    date > "$LAST_DEEP_CLEANUP"
    
    echo -e "${GREEN}✓ Deep cleanup complete${NC}"
}

# Main monitoring function
monitor_changes() {
    # Save current state
    cp "$WATCH_FILE" ".last_file_backup" 2>/dev/null || true
    
    # Get current counter
    COUNTER=$(cat "$CHANGE_COUNTER_FILE")
    
    # Count lines changed
    LINES_CHANGED=$(count_changed_lines)
    
    echo -e "${BLUE}📝 Change detected (${LINES_CHANGED} lines modified)${NC}"
    
    # Always run quick cleanup
    quick_cleanup
    
    # Increment counter
    COUNTER=$((COUNTER + 1))
    echo "$COUNTER" > "$CHANGE_COUNTER_FILE"
    
    # Check if we need deep cleanup
    if [ "$COUNTER" -ge "$CHANGES_THRESHOLD" ]; then
        deep_cleanup
    elif [ "$LINES_CHANGED" -gt "$COMPLEXITY_THRESHOLD" ]; then
        trigger_subagent_cleanup
    fi
    
    # Update backup
    cp "$WATCH_FILE" ".last_file_backup"
    
    echo ""
}

# Cleanup on exit
cleanup_on_exit() {
    echo -e "\n${YELLOW}Cleanup system stopped${NC}"
    rm -f .last_file_backup
    exit 0
}

trap cleanup_on_exit EXIT INT TERM

# Initial setup
cp "$WATCH_FILE" ".last_file_backup" 2>/dev/null || true
LAST_HASH=$(md5sum "$WATCH_FILE" 2>/dev/null | cut -d' ' -f1 || echo "none")

# Main loop
echo "Monitoring for changes..."
while true; do
    if [ -f "$WATCH_FILE" ]; then
        CURRENT_HASH=$(md5sum "$WATCH_FILE" | cut -d' ' -f1)
        
        if [ "$CURRENT_HASH" != "$LAST_HASH" ]; then
            monitor_changes
            LAST_HASH="$CURRENT_HASH"
        fi
    fi
    
    sleep 60  # Check every minute
done