#!/bin/bash

# 🐹 CHONKER Phoenix Launcher
# The lean, focused PDF-to-SQL hamster

echo "🐹 Launching CHONKER Phoenix..."
echo "The Focused PDF-to-SQL Hamster"
echo ""

# Check if virtual environment exists
if [ ! -d "venv" ]; then
    echo "❌ Virtual environment not found!"
    echo "Please run: python3 -m venv venv && source venv/bin/activate && pip install -r requirements.txt"
    exit 1
fi

# Check if CHONKER Phoenix exists
if [ ! -f "chonker_phoenix.py" ]; then
    echo "❌ CHONKER Phoenix not found!"
    echo "Missing: chonker_phoenix.py"
    exit 1
fi

# Check for sacred hamster emoji
if [ ! -f "icons/hamster_android7.png" ]; then
    echo "⚠️  CRITICAL WARNING: Sacred Android 7.1 hamster emoji missing!"
    echo "   Expected location: icons/hamster_android7.png"
    echo "   This is the HIGHEST DIRECTIVE - please restore immediately!"
    echo ""
    echo "   Continuing with fallback hamster..."
    echo ""
fi

# Activate virtual environment and launch
echo "🔧 Activating virtual environment..."
source venv/bin/activate

echo "🚀 Starting CHONKER Phoenix..."
echo ""
echo "Features:"
echo "  🐹 PDF chomping with Docling"
echo "  🔍 Editable HTML tables" 
echo "  💾 SQL export with type inference"
echo "  🛡️ Full defensive armor (500MB limit, timeouts)"
echo "  🎨 Dark theme"
echo ""
echo "Workflow: PDF → Chomp → Edit → SQL"
echo "Lines of code: 1,020 (vs 2,589 in bloated CHONKER)"
echo ""

# Launch CHONKER Phoenix
python chonker_phoenix.py

echo ""
echo "🐹 CHONKER Phoenix session ended"