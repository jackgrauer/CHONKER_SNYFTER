#!/usr/bin/env python3
"""Strip text layer from PDF using Ghostscript."""

import sys
import subprocess
from pathlib import Path

def strip_text_with_ghostscript(input_path, output_path):
    """Use Ghostscript to create a PDF with images only (no text layer)."""
    
    # Ghostscript command to render PDF as images and recreate PDF
    # This preserves the visual appearance but removes text
    gs_cmd = [
        "gs",
        "-sDEVICE=pdfwrite",
        "-dNOPAUSE",
        "-dBATCH",
        "-dSAFER",
        "-dFILTERTEXT",  # This removes text
        "-sOutputFile=" + str(output_path),
        str(input_path)
    ]
    
    try:
        print("Processing with Ghostscript...")
        result = subprocess.run(gs_cmd, capture_output=True, text=True)
        
        if result.returncode != 0:
            print(f"Ghostscript error: {result.stderr}")
            return False
            
        # Get file sizes
        input_size = Path(input_path).stat().st_size / 1024
        output_size = Path(output_path).stat().st_size / 1024
        
        print(f"\nCreated {output_path} without text layer")
        print(f"Input size: {input_size:.1f} KB")
        print(f"Output size: {output_size:.1f} KB")
        
        return True
        
    except FileNotFoundError:
        print("Error: Ghostscript (gs) not found. Please install it:")
        print("  brew install ghostscript")
        return False
    except Exception as e:
        print(f"Error: {e}")
        return False

if __name__ == "__main__":
    if len(sys.argv) != 3:
        print("Usage: python strip_text_gs.py <input.pdf> <output.pdf>")
        sys.exit(1)
    
    input_path = Path(sys.argv[1])
    output_path = Path(sys.argv[2])
    
    if not input_path.exists():
        print(f"Error: {input_path} does not exist")
        sys.exit(1)
    
    success = strip_text_with_ghostscript(input_path, output_path)
    sys.exit(0 if success else 1)