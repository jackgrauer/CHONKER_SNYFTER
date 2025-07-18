#!/bin/bash
#
# CHONKER Launcher - Bash wrapper for Python launcher
# This script activates the virtual environment and calls the Python launcher
#

# Get the directory where this script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    local status=$1
    local message=$2
    
    case $status in
        "error")
            echo -e "${RED}[ERROR]${NC} $message"
            ;;
        "success")
            echo -e "${GREEN}[SUCCESS]${NC} $message"
            ;;
        "info")
            echo -e "${YELLOW}[INFO]${NC} $message"
            ;;
        *)
            echo "$message"
            ;;
    esac
}

# Check if virtual environment exists
if [ ! -d "$SCRIPT_DIR/venv" ]; then
    print_status "error" "Virtual environment not found at $SCRIPT_DIR/venv"
    print_status "info" "Please create it with: python3 -m venv venv"
    exit 1
fi

# Activate virtual environment
print_status "info" "Activating virtual environment..."
source "$SCRIPT_DIR/venv/bin/activate"

# Check if activation was successful
if [ -z "$VIRTUAL_ENV" ]; then
    print_status "error" "Failed to activate virtual environment"
    exit 1
fi

# Check if psutil is installed (required dependency)
if ! python -c "import psutil" 2>/dev/null; then
    print_status "error" "Required package 'psutil' not found"
    print_status "info" "Please install it with: pip install psutil"
    exit 1
fi

# Run the Python launcher with all arguments
print_status "info" "Running CHONKER launcher..."
if [ $# -eq 0 ]; then
    # Default to launch command if no arguments provided
    python "$SCRIPT_DIR/chonker_launcher.py" launch
else
    python "$SCRIPT_DIR/chonker_launcher.py" "$@"
fi

# Capture exit code
EXIT_CODE=$?

# Deactivate virtual environment
deactivate

# Exit with the same code as the Python script
exit $EXIT_CODE