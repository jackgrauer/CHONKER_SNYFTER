#!/bin/bash

# SNYFTER Autonomous Development Launcher
# Fire-and-forget script that runs in background with caffeinate

echo "🚀 SNYFTER Autonomous Development Launcher"
echo "========================================"

# Check if virtual environment exists
if [ ! -d "venv" ]; then
    echo "❌ Virtual environment not found!"
    echo "Please run: python3 -m venv venv && source venv/bin/activate && pip install -r requirements.txt"
    exit 1
fi

# Create logs directory
mkdir -p logs

# Get current timestamp
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
LOG_FILE="logs/snyfter_dev_${TIMESTAMP}.log"

echo "📝 Logging to: $LOG_FILE"
echo "☕ Starting caffeinate to prevent sleep..."
echo "🔧 Running three-agent development..."
echo ""
echo "⚠️  IMPORTANT: This will run autonomously in the background!"
echo "   - AI features will be SKIPPED to avoid prompts"
echo "   - All files will be backed up before modification"
echo "   - Check $LOG_FILE for progress"
echo "   - Run 'tail -f $LOG_FILE' to monitor"
echo ""

# Confirm before starting
read -p "Start autonomous SNYFTER development? (y/n) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "❌ Cancelled"
    exit 1
fi

# Start the development in background with nohup
nohup bash -c "
    # Activate virtual environment
    source venv/bin/activate
    
    # Start caffeinate in background
    caffeinate -disu &
    CAFFEINATE_PID=\$!
    
    # Run the three-agent execution
    python snyfter_three_agent_execution.py 2>&1
    
    # Stop caffeinate
    kill \$CAFFEINATE_PID 2>/dev/null
    
    echo '✅ SNYFTER development completed!'
" > "$LOG_FILE" 2>&1 &

PROCESS_PID=$!

echo ""
echo "✅ SNYFTER development started!"
echo "   Process ID: $PROCESS_PID"
echo "   Log file: $LOG_FILE"
echo ""
echo "📊 Monitor progress with:"
echo "   tail -f $LOG_FILE"
echo ""
echo "🛑 Stop development with:"
echo "   kill $PROCESS_PID"
echo ""
echo "The system will:"
echo "1. Create Pydantic models for SNYFTER features"
echo "2. Generate database schemas"
echo "3. Build UI components"
echo "4. Integrate with existing CHONKER mode"
echo "5. Skip AI features to avoid user prompts"
echo ""
echo "Check these files after completion:"
echo "- snyfter_development_summary.md (final report)"
echo "- snyfter_models.py (data models)"
echo "- backups/ (original file backups)"
echo ""