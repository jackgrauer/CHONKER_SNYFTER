# CHONKER & SNYFTER Justfile
# Commands for common development tasks

# Default command - show available commands
default:
    @just --list

# Run the main application
run:
    source venv/bin/activate && python chonker_snyfter_elegant_v2.py

# Run tests
test:
    source venv/bin/activate && python -m pytest tests/

# Run performance benchmarks
bench:
    source venv/bin/activate && python feature_optimization.py

# Clean up Python cache files
clean:
    find . -type d -name "__pycache__" -exec rm -rf {} + 2>/dev/null || true
    find . -type f -name "*.pyc" -delete
    find . -type f -name "*.pyo" -delete
    find . -type f -name ".DS_Store" -delete

# Install dependencies
install:
    python3 -m venv venv
    source venv/bin/activate && pip install -r requirements.txt

# Update dependencies
update:
    source venv/bin/activate && pip install --upgrade -r requirements.txt

# Format code with black
format:
    source venv/bin/activate && black *.py

# Lint code
lint:
    source venv/bin/activate && ruff check *.py

# Type check with mypy
typecheck:
    source venv/bin/activate && mypy *.py

# Run security audit
security:
    source venv/bin/activate && python -m pytest tests/test_security.py -v

# Git status
status:
    git status

# Git commit with message
commit message:
    git add -A
    git commit -m "{{message}}"

# Git push
push:
    git push origin main

# Full cleanup and reinstall
reset:
    rm -rf venv
    just clean
    just install

# Development mode - run with auto-reload
dev:
    source venv/bin/activate && python chonker_snyfter_elegant_v2.py --debug

# Check what's actually working
check-features:
    @echo "=== CHONKER & SNYFTER Feature Status ==="
    @echo "✅ Keyboard shortcuts: WORKING"
    @echo "✅ Gesture detection: WORKING (terminal output)"
    @echo "✅ PDF zoom: WORKING"
    @echo "❌ HTML zoom: NOT WORKING"
    @echo "❓ VLM fallback: UNCLEAR"
    @echo "✅ Core processing: WORKING"