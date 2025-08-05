#!/bin/bash

# Cleanup Daemon - Background service for continuous code quality
# Can be run as a background process or systemd service

set -e

# Configuration
DAEMON_NAME="chonker5-cleanup"
PID_FILE="/tmp/${DAEMON_NAME}.pid"
LOG_FILE="cleanup-daemon.log"
WATCH_FILE="chonker5.rs"

# Colors (for interactive mode)
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

# Function to check if daemon is running
is_running() {
    if [ -f "$PID_FILE" ]; then
        PID=$(cat "$PID_FILE")
        if ps -p "$PID" > /dev/null 2>&1; then
            return 0
        fi
    fi
    return 1
}

# Start daemon
start_daemon() {
    if is_running; then
        echo -e "${YELLOW}Daemon is already running (PID: $(cat $PID_FILE))${NC}"
        return 1
    fi
    
    echo -e "${GREEN}Starting cleanup daemon...${NC}"
    
    # Run in background
    (
        echo $$ > "$PID_FILE"
        exec > "$LOG_FILE" 2>&1
        
        echo "[$(date)] Cleanup daemon started (PID: $$)"
        
        # Set error handling to continue on errors
        set +e
        
        # Main daemon loop
        while true; do
            # Check if main file exists
            if [ ! -f "$WATCH_FILE" ]; then
                echo "[$(date)] Warning: $WATCH_FILE not found"
                sleep 30
                continue
            fi
            
            # Run quality checks silently
            ISSUES=""
            
            # Compilation check
            if ! cargo check --quiet 2>/dev/null; then
                ISSUES="${ISSUES}compilation "
            fi
            
            # Clippy check
            if cargo clippy --quiet -- -W clippy::all 2>&1 | grep -q "warning:"; then
                ISSUES="${ISSUES}clippy "
            fi
            
            # Format check
            if ! cargo fmt -- --check >/dev/null 2>&1; then
                cargo fmt --quiet 2>/dev/null || true
                ISSUES="${ISSUES}formatting(fixed) "
            fi
            
            # Log issues if any
            if [ -n "$ISSUES" ]; then
                echo "[$(date)] Issues detected: $ISSUES"
            fi
            
            # Sleep before next check
            sleep 300  # Check every 5 minutes
        done
    ) &
    
    sleep 1
    
    if is_running; then
        echo -e "${GREEN}✓ Daemon started successfully (PID: $(cat $PID_FILE))${NC}"
        echo "Log file: $LOG_FILE"
    else
        echo -e "${RED}✗ Failed to start daemon${NC}"
        return 1
    fi
}

# Stop daemon
stop_daemon() {
    if ! is_running; then
        echo -e "${YELLOW}Daemon is not running${NC}"
        return 1
    fi
    
    PID=$(cat "$PID_FILE")
    echo -e "${YELLOW}Stopping cleanup daemon (PID: $PID)...${NC}"
    
    kill "$PID" 2>/dev/null || true
    rm -f "$PID_FILE"
    
    echo -e "${GREEN}✓ Daemon stopped${NC}"
}

# Check daemon status
status_daemon() {
    if is_running; then
        PID=$(cat "$PID_FILE")
        echo -e "${GREEN}● Cleanup daemon is running (PID: $PID)${NC}"
        
        # Show recent log entries
        if [ -f "$LOG_FILE" ]; then
            echo -e "\nRecent activity:"
            tail -5 "$LOG_FILE" | sed 's/^/  /'
        fi
    else
        echo -e "${RED}● Cleanup daemon is not running${NC}"
    fi
}

# Main command handler
case "${1:-}" in
    start)
        start_daemon
        ;;
    stop)
        stop_daemon
        ;;
    restart)
        stop_daemon
        sleep 1
        start_daemon
        ;;
    status)
        status_daemon
        ;;
    logs)
        if [ -f "$LOG_FILE" ]; then
            tail -f "$LOG_FILE"
        else
            echo "No log file found"
        fi
        ;;
    *)
        echo "Usage: $0 {start|stop|restart|status|logs}"
        echo ""
        echo "Commands:"
        echo "  start    - Start the cleanup daemon"
        echo "  stop     - Stop the cleanup daemon"
        echo "  restart  - Restart the cleanup daemon"
        echo "  status   - Check daemon status"
        echo "  logs     - Follow daemon logs"
        exit 1
        ;;
esac