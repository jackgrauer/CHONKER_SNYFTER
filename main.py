#!/usr/bin/env python3
"""CHONKER - Elegant PDF processing with hamster wisdom"""

import sys
import logging
from pathlib import Path

from PyQt6.QtWidgets import QApplication
from PyQt6.QtGui import QIcon
from PyQt6.QtCore import Qt

from chonker.ui.main_window import MainWindow


def setup_logging():
    """Configure logging"""
    logging.basicConfig(
        level=logging.INFO,
        format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
        handlers=[
            logging.StreamHandler(sys.stdout),
            logging.FileHandler('chonker.log')
        ]
    )


def load_icon():
    """Load application icon"""
    icon_path = Path(__file__).parent / "assets" / "emojis" / "chonker.png"
    if icon_path.exists():
        return QIcon(str(icon_path))
    return None


def main():
    """Main entry point"""
    # Setup logging
    setup_logging()
    
    print("🐹 CHONKER ready!")
    
    # Create application
    app = QApplication(sys.argv)
    app.setApplicationName("CHONKER")
    app.setOrganizationName("ChonkerDev")
    
    # Set application icon
    icon = load_icon()
    if icon:
        app.setWindowIcon(icon)
    
    # Enable high DPI support (not needed in Qt6)
    
    # Create and show main window
    window = MainWindow()
    
    # Load PDF from command line if provided
    if len(sys.argv) > 1:
        pdf_path = sys.argv[1]
        if Path(pdf_path).exists() and pdf_path.lower().endswith('.pdf'):
            window.load_pdf(pdf_path)
    
    window.show()
    
    # Run application
    sys.exit(app.exec())


if __name__ == '__main__':
    main()