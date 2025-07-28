#!/usr/bin/env python3
"""Migration script to transition from old chonker.py to new modular structure"""

import shutil
from pathlib import Path


def migrate():
    """Perform migration"""
    print("🐹 CHONKER Migration Tool")
    print("========================")
    
    # Check if old chonker.py exists
    old_chonker = Path("chonker.py")
    if old_chonker.exists():
        # Backup old file
        backup_path = Path("chonker_old.py")
        if not backup_path.exists():
            shutil.copy(old_chonker, backup_path)
            print(f"✓ Backed up old chonker.py to {backup_path}")
        else:
            print(f"⚠️  Backup already exists at {backup_path}")
    
    # Check if new structure exists
    new_main = Path("main.py")
    chonker_dir = Path("chonker")
    
    if new_main.exists() and chonker_dir.exists():
        print("✓ New modular structure is ready")
        
        # Create launcher script
        launcher = Path("run_chonker_new.sh")
        launcher.write_text("""#!/bin/bash
# New CHONKER launcher using modular structure

# Activate virtual environment
source .venv/bin/activate

# Run new modular CHONKER
python main.py "$@"
""")
        launcher.chmod(0o755)
        print(f"✓ Created new launcher: {launcher}")
        
        print("\n📋 Migration Summary:")
        print("- Old monolithic file backed up to chonker_old.py")
        print("- New modular structure in chonker/ directory")
        print("- Main entry point: main.py")
        print("- Run with: ./run_chonker_new.sh")
        
        print("\n🎯 Key Improvements:")
        print("- Proper coordinate system handling (no more overlaps!)")
        print("- Clean separation of concerns")
        print("- Testable components")
        print("- Better error handling")
        print("- Consistent spatial layout")
        
    else:
        print("❌ New structure not found. Make sure all files are in place.")
        

if __name__ == "__main__":
    migrate()