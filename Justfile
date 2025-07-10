# Justfile for CHONKER_SNYFTER project automations

# Setup Python virtual environment
setup-venv:
    @echo "Setting up Python virtual environment..."
    python3 -m venv .venv
    @echo "Virtual environment created at .venv"
    @echo "To activate: source .venv/bin/activate"

# Install Python dependencies in virtual environment
install-python:
    @echo "Installing Python dependencies..."
    .venv/bin/pip install --upgrade pip
    .venv/bin/pip install docling requests python-dotenv
    @echo "Python dependencies installed!"

# Install all dependencies
install: setup-venv install-python
    @echo "Installing dependencies..."
    # Install frontend dependencies
    cd frontend/chonker-modern && npm install --legacy-peer-deps
    # Install Tauri dependencies
    cd src-tauri && npm install --legacy-peer-deps
    @echo "All dependencies installed!"

# Check the status of the project
status:
    @echo "Checking project status..."
    # Check Rust backend status
    cd src-tauri && cargo check
    # Check frontend status
    cd frontend/chonker-modern && npm run check || echo "Frontend check not available"
    # Check Tauri status
    cd src-tauri && npm run tauri info

# Run development servers
dev:
    @echo "Starting Tauri development..."
    cd src-tauri && npm run dev

# Start backend only
backend:
    @echo "Starting backend server..."
    cd src-tauri && cargo run

# Start frontend only
frontend:
    @echo "Starting frontend server..."
    cd frontend/chonker-modern && npm run dev

# Build the project
build:
    @echo "Building project..."
    cd src-tauri && npm run build

# Clean build artifacts
clean:
    @echo "Cleaning build artifacts..."
    cd src-tauri && cargo clean
    cd frontend/chonker-modern && rm -rf node_modules dist

# Run tests
test:
    @echo "Running tests..."
    cd src-tauri && cargo test
    cd frontend/chonker-modern && npm test || echo "Frontend tests not configured"

# Format code
fmt:
    @echo "Formatting code..."
    cd src-tauri && cargo fmt
    cd frontend/chonker-modern && npm run format || echo "Frontend formatting not configured"

# Run linter
lint:
    @echo "Running linter..."
    cd src-tauri && cargo clippy
    cd frontend/chonker-modern && npm run lint || echo "Frontend linting not configured"

# Show project info
info:
    @echo "=== CHONKER_SNYFTER Project Info ==="
    @echo "Current directory: $(pwd)"
    @echo "Rust version: $(rustc --version)"
    @echo "Node version: $(node --version)"
    @echo "NPM version: $(npm --version)"
    @echo "Tauri CLI: $(npm list -g @tauri-apps/cli 2>/dev/null || echo 'Not installed globally')"

# Fix common issues
fix:
    @echo "Running common fixes..."
    cd src-tauri && cargo update
    cd frontend/chonker-modern && npm audit fix || echo "No npm audit fixes needed"

# Git workflow commands
git-status:
    @echo "=== Git Status ==="
    git status --porcelain
    @echo "\n=== Git Log (last 5 commits) ==="
    git log --oneline -5

# Add all changes to git
git-add:
    @echo "Adding all changes to git..."
    git add .
    git status --short

# Commit with message
git-commit message:
    @echo "Committing changes: {{message}}"
    git commit -m "{{message}}"

# Quick commit with auto-generated message
git-quick:
    @echo "Quick commit with auto-generated message..."
    git add .
    git commit -m "Auto-commit: $(date +'%Y-%m-%d %H:%M:%S') - Updated project files"

# Push to remote
git-push:
    @echo "Pushing to remote..."
    git push origin main

# Pull from remote
git-pull:
    @echo "Pulling from remote..."
    git pull origin main

# Full workflow: add, commit, push
git-save message:
    @echo "Saving changes: {{message}}"
    git add .
    git commit -m "{{message}}"
    git push origin main

# Create .gitignore entries for common files
git-ignore:
    @echo "Updating .gitignore..."
    echo "\n# Virtual environment" >> .gitignore
    echo ".venv/" >> .gitignore
    echo "\n# Node modules" >> .gitignore
    echo "node_modules/" >> .gitignore
    echo "\n# Build artifacts" >> .gitignore
    echo "target/" >> .gitignore
    echo "dist/" >> .gitignore
    echo "\n# IDE files" >> .gitignore
    echo ".vscode/" >> .gitignore
    echo ".idea/" >> .gitignore
    echo "\n# OS files" >> .gitignore
    echo ".DS_Store" >> .gitignore
    echo "Thumbs.db" >> .gitignore
    @echo "Added common ignore patterns to .gitignore"

# Activate virtual environment (reminder)
activate:
    @echo "To activate the Python virtual environment, run:"
    @echo "source .venv/bin/activate"

# Default recipe
default:
    @just --list
