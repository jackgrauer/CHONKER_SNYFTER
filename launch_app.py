#!/usr/bin/env python3
"""
Launcher for CHONKER & SNYFTER that keeps the app running
"""

import sys
import os
import subprocess
from pathlib import Path

def launch_app():
    """Launch the application properly"""
    # Get the script directory
    script_dir = Path(__file__).parent
    os.chdir(script_dir)
    
    # Check if we're in virtual environment
    if not hasattr(sys, 'real_prefix') and not (hasattr(sys, 'base_prefix') and sys.base_prefix != sys.prefix):
        print("⚠️  Virtual environment not activated!")
        print("Please run: source /Users/jack/chonksnyft-env/bin/activate")
        return
    
    print("🚀 Launching CHONKER & SNYFTER...")
    print("=" * 50)
    print("The app will stay open until you close it.")
    print("Check your dock/taskbar for the application window.")
    print("=" * 50)
    
    try:
        # Import and run directly (keeps app running)
        import chonker_snyfter_enhanced
        chonker_snyfter_enhanced.main()
    except KeyboardInterrupt:
        print("\n👋 CHONKER & SNYFTER closed by user")
    except Exception as e:
        print(f"\n❌ Error: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    launch_app()