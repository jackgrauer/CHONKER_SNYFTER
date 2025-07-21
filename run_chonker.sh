#!/bin/bash

# Simple launcher for CHONKER

# Check if virtual environment exists
if [ ! -d ".venv" ]; then
    echo "Virtual environment not found. Running setup..."
    ./migrate_to_uv.sh
fi

# Activate virtual environment and run
source .venv/bin/activate
python chonker.py "$@"