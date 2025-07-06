#!/usr/bin/env python3
"""
Simple extraction bridge - just calls what works and outputs JSON
No complexity, no transformation, just pass through the working extractor
"""

import json
import sys
import os
from pathlib import Path
from extraction_pipeline import ExtractionPipeline

# Redirect stderr to /dev/null to avoid interfering with JSON output
import contextlib

@contextlib.contextmanager
def suppress_stderr():
    with open(os.devnull, "w") as devnull:
        old_stderr = sys.stderr
        sys.stderr = devnull
        try:
            yield
        finally:
            sys.stderr = old_stderr

def main():
    if len(sys.argv) != 2:
        print(json.dumps({
            "success": False,
            "error": "Usage: simple_extraction_bridge.py <pdf_path>"
        }))
        sys.exit(1)
    
    pdf_path = sys.argv[1]
    
    # Validate file exists
    if not Path(pdf_path).exists():
        print(json.dumps({
            "success": False,
            "error": f"PDF file not found: {pdf_path}"
        }))
        sys.exit(1)
    
    try:
        # Use the extraction pipeline that actually works - suppress stderr
        with suppress_stderr():
            pipeline = ExtractionPipeline()
            results = pipeline.run(pdf_path, save_intermediates=False)
        
        # Just pass through what works - the structured_tables from the pipeline
        structured_tables = results.get('structured_tables', [])
        
        # Create simple output in the format Rust expects
        output = {
            "success": True,
            "tool": "simple_extraction_bridge", 
            "extractions": [{
                "page_number": 1,
                "text": results.get('document', {}).get('markdown', ''),
                "tables": [],  # Legacy field
                "figures": [],
                "formulas": [],
                "confidence": 0.95,
                "layout_boxes": [],  # Required field
                "tool": "simple_extraction_bridge"
            }],
            "metadata": {
                "total_pages": 1,
                "processing_time": 0
            },
            "structured_tables": structured_tables  # The actual good data
        }
        
        print(json.dumps(output, ensure_ascii=False, default=str))
        
    except Exception as e:
        print(json.dumps({
            "success": False,
            "error": f"Extraction failed: {str(e)}"
        }))
        sys.exit(1)

if __name__ == '__main__':
    main()
