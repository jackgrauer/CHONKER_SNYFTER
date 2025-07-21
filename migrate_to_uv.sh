#!/bin/bash

echo "🚀 Migrating CHONKER SNYFTER to uv..."
echo "====================================="

# Check if uv is installed
if ! command -v uv &> /dev/null; then
    echo "📦 Installing uv..."
    curl -LsSf https://astral.sh/uv/install.sh | sh
    echo "✅ uv installed!"
fi

# Remove old venv if it exists
if [ -d "venv" ]; then
    echo "🗑️  Removing old venv directory..."
    rm -rf venv
fi

# Create new virtual environment with uv
echo "🔧 Creating new virtual environment with uv..."
uv venv

# Install dependencies
echo "📥 Installing dependencies..."
source .venv/bin/activate
uv pip install -r requirements.txt

echo ""
echo "✅ Migration complete!"
echo ""
echo "To activate the new environment, run:"
echo "  source .venv/bin/activate"
echo ""
echo "Then you can run CHONKER with:"
echo "  python chonker_snyfter_elegant_v2.py"