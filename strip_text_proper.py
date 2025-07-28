#!/usr/bin/env python3
"""Strip text layer by converting to image and back to PDF."""

import sys
import subprocess
from pathlib import Path
import tempfile

def strip_text_via_image(input_path, output_path):
    """Convert PDF to image then back to PDF to remove text layer."""
    
    try:
        with tempfile.TemporaryDirectory() as tmpdir:
            temp_image = Path(tmpdir) / "page.png"
            
            # Convert PDF page to PNG with ImageMagick
            convert_cmd = [
                "convert",
                "-density", "150",  # 150 DPI
                f"{input_path}[0]",  # First page only
                "-quality", "90",
                str(temp_image)
            ]
            
            print("Converting PDF to image...")
            result = subprocess.run(convert_cmd, capture_output=True, text=True)
            
            if result.returncode != 0:
                # Try with Ghostscript instead
                gs_cmd = [
                    "gs",
                    "-sDEVICE=png16m",
                    "-dNOPAUSE",
                    "-dBATCH", 
                    "-dSAFER",
                    "-r150",
                    "-dFirstPage=1",
                    "-dLastPage=1",
                    f"-sOutputFile={temp_image}",
                    str(input_path)
                ]
                result = subprocess.run(gs_cmd, capture_output=True, text=True)
                if result.returncode != 0:
                    print(f"Error converting to image: {result.stderr}")
                    return False
            
            # Convert image back to PDF with ImageMagick
            convert_back = [
                "convert",
                str(temp_image),
                "-compress", "jpeg",
                "-quality", "85",
                str(output_path)
            ]
            
            print("Converting image back to PDF...")
            result = subprocess.run(convert_back, capture_output=True, text=True)
            
            if result.returncode != 0:
                print(f"Error converting to PDF: {result.stderr}")
                print("Trying alternative method...")
                
                # Alternative: use img2pdf if available
                try:
                    import img2pdf
                    with open(output_path, "wb") as f:
                        f.write(img2pdf.convert(str(temp_image)))
                except ImportError:
                    print("Installing img2pdf...")
                    subprocess.check_call([sys.executable, "-m", "pip", "install", "--user", "img2pdf"])
                    import img2pdf
                    with open(output_path, "wb") as f:
                        f.write(img2pdf.convert(str(temp_image)))
        
        # Get file sizes
        input_size = Path(input_path).stat().st_size / 1024
        output_size = Path(output_path).stat().st_size / 1024
        
        print(f"\nCreated {output_path} without text layer")
        print(f"Input size: {input_size:.1f} KB")
        print(f"Output size: {output_size:.1f} KB")
        print("This PDF now contains only an image - no selectable text!")
        
        return True
        
    except FileNotFoundError as e:
        print(f"Error: Required tool not found: {e}")
        print("Please install ImageMagick: brew install imagemagick")
        return False
    except Exception as e:
        print(f"Error: {e}")
        return False

if __name__ == "__main__":
    if len(sys.argv) != 3:
        print("Usage: python strip_text_proper.py <input.pdf> <output.pdf>")
        sys.exit(1)
    
    input_path = Path(sys.argv[1])
    output_path = Path(sys.argv[2])
    
    if not input_path.exists():
        print(f"Error: {input_path} does not exist")
        sys.exit(1)
    
    success = strip_text_via_image(input_path, output_path)
    sys.exit(0 if success else 1)