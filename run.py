#!/usr/bin/env python3
"""
Simple runner for CHONKER & SNYFTER
"""

import subprocess
import sys
import os

# Set environment
os.environ['QT_MAC_WANTS_LAYER'] = '1'  # For macOS compatibility

# Run the app
if __name__ == "__main__":
    print("🐹 CHONKER & 🐁 SNYFTER Starting...")
    print("Check your dock for the application window.")
    print("\nThe app is running in the GUI.")
    print("You can close this terminal or press Ctrl+C when done.")
    
    try:
        # Direct import and run
        import chonker_snyfter_enhanced
        chonker_snyfter_enhanced.main()
    except KeyboardInterrupt:
        print("\n👋 Goodbye!")
    except Exception as e:
        print(f"\n❌ Error: {e}")