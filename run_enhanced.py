#!/usr/bin/env python3
"""
Launch script for the enhanced CHONKER & SNYFTER application
Handles dependency checking and environment setup
"""

import sys
import os
import subprocess
from pathlib import Path

def check_dependencies():
    """Check if all required dependencies are installed"""
    missing = []
    
    # Check core dependencies
    try:
        import PyQt6
    except ImportError:
        missing.append("PyQt6")
    
    try:
        import docling
    except ImportError:
        missing.append("docling")
    
    try:
        import fitz
    except ImportError:
        missing.append("PyMuPDF")
    
    try:
        import instructor
    except ImportError:
        missing.append("instructor")
    
    try:
        import openai
    except ImportError:
        missing.append("openai")
    
    try:
        from rich import console
    except ImportError:
        missing.append("rich")
    
    try:
        import pydantic
    except ImportError:
        missing.append("pydantic")
    
    return missing

def install_dependencies(missing):
    """Attempt to install missing dependencies"""
    print(f"🔧 Installing missing dependencies: {', '.join(missing)}")
    
    # Map package names to pip install names
    package_map = {
        "PyQt6": "PyQt6 PyQt6-Qt6 PyQt6-sip PyQt6-WebEngine PyQt6-WebEngine-Qt6",
        "docling": "docling",
        "PyMuPDF": "PyMuPDF",
        "instructor": "instructor",
        "openai": "openai",
        "rich": "rich",
        "pydantic": "pydantic"
    }
    
    for dep in missing:
        if dep in package_map:
            cmd = f"pip install {package_map[dep]}"
            print(f"  Running: {cmd}")
            try:
                subprocess.check_call(cmd.split())
            except subprocess.CalledProcessError:
                print(f"❌ Failed to install {dep}")
                return False
    
    return True

def create_assets():
    """Create assets directory if it doesn't exist"""
    assets_dir = Path("assets/emojis")
    if not assets_dir.exists():
        print("📁 Creating assets directory...")
        assets_dir.mkdir(parents=True, exist_ok=True)
        
        # Create placeholder emoji files if they don't exist
        chonker_path = assets_dir / "chonker.png"
        snyfter_path = assets_dir / "snyfter.png"
        
        if not chonker_path.exists():
            print("  Creating placeholder for chonker.png")
            # Create a simple colored square as placeholder
            try:
                from PIL import Image
                img = Image.new('RGBA', (64, 64), (255, 228, 181, 255))  # Peach color
                img.save(chonker_path)
            except ImportError:
                # If PIL not available, just create empty file
                chonker_path.touch()
        
        if not snyfter_path.exists():
            print("  Creating placeholder for snyfter.png")
            try:
                from PIL import Image
                img = Image.new('RGBA', (64, 64), (211, 211, 211, 255))  # Light gray
                img.save(snyfter_path)
            except ImportError:
                snyfter_path.touch()

def main():
    """Main entry point"""
    print("🚀 CHONKER & SNYFTER Enhanced Launcher")
    print("=" * 50)
    
    # Check Python version
    if sys.version_info < (3, 8):
        print(f"❌ Python 3.8 or higher required. You have {sys.version}")
        sys.exit(1)
    
    # Check dependencies
    missing = check_dependencies()
    
    if missing:
        print(f"⚠️  Missing dependencies detected: {', '.join(missing)}")
        response = input("Would you like to install them now? (y/n): ")
        
        if response.lower() == 'y':
            if not install_dependencies(missing):
                print("❌ Failed to install all dependencies. Please install manually.")
                sys.exit(1)
        else:
            print("❌ Cannot run without required dependencies.")
            sys.exit(1)
    
    # Create assets if needed
    create_assets()
    
    # Check if enhanced version exists
    enhanced_path = Path("chonker_snyfter_enhanced.py")
    if not enhanced_path.exists():
        print("❌ Enhanced version not found. Please ensure chonker_snyfter_enhanced.py exists.")
        sys.exit(1)
    
    # Launch the application
    print("\n✅ All checks passed! Launching CHONKER & SNYFTER...")
    print("=" * 50)
    
    try:
        # Import and run the enhanced version
        import chonker_snyfter_enhanced
        chonker_snyfter_enhanced.main()
    except Exception as e:
        print(f"\n❌ Error launching application: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)

if __name__ == "__main__":
    main()