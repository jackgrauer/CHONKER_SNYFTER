#!/bin/bash
# Run the single-file version of CHONKER

# Activate virtual environment if it exists
if [ -f ".venv/bin/activate" ]; then
    source .venv/bin/activate
fi

# Run the single-file chonker.py
python chonker.py "$@"