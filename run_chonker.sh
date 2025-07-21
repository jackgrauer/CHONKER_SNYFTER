#!/bin/bash

# Simple launcher for CHONKER SNYFTER

# Check if virtual environment exists
if [ ! -d ".venv" ]; then
    echo "Virtual environment not found. Running setup..."
    ./migrate_to_uv.sh
fi

# Activate virtual environment and run
source .venv/bin/activate
python chonker_snyfter_elegant_v2.py "$@"