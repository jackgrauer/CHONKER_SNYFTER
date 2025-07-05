#!/usr/bin/env python3
"""
Test script to generate and verify structured table data for CHONKER GUI
"""
import json
import sys
from pathlib import Path

def create_test_table_data():
    """Create test structured table data similar to Docling output"""
    return {
        "metadata": {
            "source_file": "test_table.pdf",
            "extraction_tool": "Docling Structured Bridge",
            "extraction_time": "2025-01-01T12:00:00Z",
            "page_count": 1
        },
        "elements": [
            {
                "id": "text_0",
                "type": "text",
                "element_index": 0,
                "content": "Sample Analysis Report"
            },
            {
                "id": "table_0", 
                "type": "table",
                "element_index": 1,
                "table_structure": {
                    "num_rows": 4,
                    "num_cols": 4,
                    "cells": []
                },
                "grid_data": {
                    "num_rows": 4,
                    "num_cols": 4,
                    "grid": [
                        [
                            {"text": "Sample ID", "row_span": 1, "col_span": 1},
                            {"text": "Lab ID", "row_span": 1, "col_span": 1},
                            {"text": "Date", "row_span": 1, "col_span": 1},
                            {"text": "Result", "row_span": 1, "col_span": 1}
                        ],
                        [
                            {"text": "SB-001", "row_span": 1, "col_span": 1},
                            {"text": "L123456", "row_span": 1, "col_span": 1},
                            {"text": "2024-05-30", "row_span": 1, "col_span": 1},
                            {"text": "25.4", "row_span": 1, "col_span": 1}
                        ],
                        [
                            {"text": "SB-002", "row_span": 1, "col_span": 1},
                            {"text": "L123457", "row_span": 1, "col_span": 1},
                            {"text": "2024-05-30", "row_span": 1, "col_span": 1},
                            {"text": "31.7", "row_span": 1, "col_span": 1}
                        ],
                        [
                            {"text": "SB-003", "row_span": 1, "col_span": 1},
                            {"text": "L123458", "row_span": 1, "col_span": 1},
                            {"text": "2024-05-30", "row_span": 1, "col_span": 1},
                            {"text": "18.9", "row_span": 1, "col_span": 1}
                        ]
                    ]
                }
            },
            {
                "id": "text_1",
                "type": "text", 
                "element_index": 2,
                "content": "All samples passed quality control requirements."
            }
        ]
    }

def main():
    if len(sys.argv) > 1 and sys.argv[1] == "--test":
        test_data = create_test_table_data()
        print(json.dumps(test_data, indent=2))
    else:
        print("Test table data generator for CHONKER GUI")
        print("Usage: python test_table_rendering.py --test")

if __name__ == "__main__":
    main()
