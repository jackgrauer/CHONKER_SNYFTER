#!/usr/bin/env python3
"""
CLI wrapper for domain-agnostic PDF extraction pipeline
Usage:
  python extract.py --debug test.pdf
  python extract.py --docling-only test.pdf 
  python extract.py --compare run1/ run2/
"""

import sys
from pathlib import Path

# Add the python directory to the path
sys.path.insert(0, str(Path(__file__).parent / "python"))

from extraction_pipeline import main

if __name__ == "__main__":
    main()
