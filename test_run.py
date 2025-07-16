#!/usr/bin/env python3
"""Test script to debug the app"""

import sys
import traceback

try:
    from PyQt6.QtWidgets import QApplication
    from PyQt6.QtGui import QPixmap
    
    print("Creating QApplication...")
    app = QApplication(sys.argv)
    
    print("Testing emoji loading...")
    chonker = QPixmap("assets/emojis/chonker.png")
    print(f"Chonker loaded: {not chonker.isNull()}, size: {chonker.size()}")
    
    snyfter = QPixmap("assets/emojis/snyfter.png") 
    print(f"Snyfter loaded: {not snyfter.isNull()}, size: {snyfter.size()}")
    
    print("\nImporting main app...")
    from chonker_snyfter import ChonkerSnyfterMainWindow
    
    print("Creating main window...")
    window = ChonkerSnyfterMainWindow()
    
    print("Showing window...")
    window.show()
    
    print("Starting event loop...")
    sys.exit(app.exec())
    
except Exception as e:
    print(f"\nERROR: {e}")
    traceback.print_exc()