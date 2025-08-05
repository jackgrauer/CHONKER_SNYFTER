#!/bin/bash

# Simple launcher for the cleanup daemon that keeps it running

echo "Starting cleanup daemon in background..."

# Kill any existing daemon
pkill -f "cleanup_daemon_worker.sh" 2>/dev/null || true

# Start the actual daemon worker
nohup bash -c '
while true; do
    if [ -f "chonker5.rs" ]; then
        # Run checks silently
        cargo check --quiet 2>/dev/null || echo "[$(date)] Compilation issues detected" >> cleanup-daemon.log
        cargo fmt --quiet -- --dangerously-skip-permissions 2>/dev/null || true
        
        # Only log if there were actual issues fixed
        if git diff --quiet chonker5.rs 2>/dev/null; then
            :  # No changes, do nothing
        else
            echo "[$(date)] Applied formatting fixes" >> cleanup-daemon.log
        fi
    fi
    
    sleep 300  # 5 minutes
done
' > /dev/null 2>&1 &

echo "Daemon started (PID: $!)"
echo $! > /tmp/chonker5-cleanup.pid
echo "Check cleanup-daemon.log for activity"