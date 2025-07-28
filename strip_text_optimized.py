#!/usr/bin/env python3
"""Strip text layer with optimized file size."""

import sys
import subprocess
from pathlib import Path
import tempfile

def strip_text_optimized(input_path, output_path, dpi=100):
    """Convert PDF to image then back to PDF with size optimization."""
    
    try:
        with tempfile.TemporaryDirectory() as tmpdir:
            temp_image = Path(tmpdir) / "page.jpg"  # Use JPEG for smaller size
            
            # Convert PDF to JPEG with lower DPI for smaller size
            gs_cmd = [
                "gs",
                "-sDEVICE=jpeg",
                "-dNOPAUSE",
                "-dBATCH", 
                "-dSAFER",
                f"-r{dpi}",  # Lower DPI for smaller file
                "-dJPEGQ=85",  # JPEG quality
                "-dFirstPage=1",
                "-dLastPage=1",
                f"-sOutputFile={temp_image}",
                str(input_path)
            ]
            
            print(f"Converting PDF to image at {dpi} DPI...")
            result = subprocess.run(gs_cmd, capture_output=True, text=True)
            if result.returncode != 0:
                print(f"Error converting to image: {result.stderr}")
                return False
            
            # Convert JPEG back to PDF
            convert_cmd = [
                "convert",
                str(temp_image),
                "-compress", "jpeg",
                "-quality", "75",  # Lower quality for smaller size
                str(output_path)
            ]
            
            print("Converting image back to PDF...")
            result = subprocess.run(convert_cmd, capture_output=True, text=True)
            
            if result.returncode != 0:
                # Fallback to Ghostscript
                gs_back = [
                    "gs",
                    "-sDEVICE=pdfwrite",
                    "-dNOPAUSE",
                    "-dBATCH",
                    "-dSAFER",
                    "-dDEVICEWIDTHPOINTS=612",
                    "-dDEVICEHEIGHTPOINTS=792",
                    "-dPDFFitPage",
                    "-dCompatibilityLevel=1.4",
                    "-dPDFSETTINGS=/ebook",  # Optimize for size
                    f"-sOutputFile={output_path}",
                    str(temp_image)
                ]
                result = subprocess.run(gs_back, capture_output=True, text=True)
                if result.returncode != 0:
                    print(f"Error converting to PDF: {result.stderr}")
                    return False
        
        # Get file sizes
        input_size = Path(input_path).stat().st_size / 1024
        output_size = Path(output_path).stat().st_size / 1024
        
        print(f"\nCreated {output_path} without text layer")
        print(f"Input size: {input_size:.1f} KB")
        print(f"Output size: {output_size:.1f} KB")
        print(f"DPI used: {dpi}")
        print("This PDF contains only an image - no selectable text!")
        
        return True
        
    except Exception as e:
        print(f"Error: {e}")
        return False

if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Usage: python strip_text_optimized.py <input.pdf> <output.pdf> [dpi]")
        print("  dpi: Optional, default 100. Lower = smaller file, lower quality")
        sys.exit(1)
    
    input_path = Path(sys.argv[1])
    output_path = Path(sys.argv[2])
    dpi = int(sys.argv[3]) if len(sys.argv) > 3 else 100
    
    if not input_path.exists():
        print(f"Error: {input_path} does not exist")
        sys.exit(1)
    
    success = strip_text_optimized(input_path, output_path, dpi)
    sys.exit(0 if success else 1)