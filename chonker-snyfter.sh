#!/bin/bash
# Launch script for CHONKER & SNYFTER

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}🚀 CHONKER & SNYFTER Launcher${NC}"
echo "=================================="

# Check if virtual environment exists
VENV_PATH="/Users/jack/chonksnyft-env"
if [ ! -d "$VENV_PATH" ]; then
    echo -e "${RED}❌ Virtual environment not found at $VENV_PATH${NC}"
    exit 1
fi

# Navigate to app directory
APP_DIR="/Users/jack/CHONKER_SNYFTER"
cd "$APP_DIR" || exit 1

# Activate virtual environment and run
echo -e "${YELLOW}📦 Activating virtual environment...${NC}"
source "$VENV_PATH/bin/activate"

echo -e "${GREEN}🐹 Starting CHONKER & SNYFTER...${NC}"
echo "The application window should appear shortly."
echo "Keep this terminal open while using the app."
echo ""
echo "Press Ctrl+C to quit when done."
echo "=================================="

# Run the enhanced version
python chonker_snyfter_enhanced.py

# Deactivate when done
deactivate
echo -e "${GREEN}✅ CHONKER & SNYFTER closed.${NC}"