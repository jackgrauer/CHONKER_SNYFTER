# CHONKER SNYFTER - Document Processing Pipeline
# Python + Docling + HTML viewer generation

set shell := ["zsh", "-c"]

default:
    @just --list

# 🐹 CHONKER - Main command with splash screen
chonker:
    @echo "\n"
    @echo "██████╗██╗  ██╗ ██████╗ ███╗   ██╗██╗  ██╗███████╗██████╗ "
    @echo "██╔════╝██║  ██║██╔═══██╗████╗  ██║██║ ██╔╝██╔════╝██╔══██╗"
    @echo "██║     ███████║██║   ██║██╔██╗ ██║█████╔╝ █████╗  ██████╔╝"
    @echo "██║     ██╔══██║██║   ██║██║╚██╗██║██╔═██╗ ██╔══╝  ██╔══██╗"
    @echo "╚██████╗██║  ██║╚██████╔╝██║ ╚████║██║  ██╗███████╗██║  ██║"
    @echo " ╚═════╝╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═══╝╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝"
    @echo "\n🚀 SNYFTER - Document Processing Pipeline"
    @echo "════════════════════════════════════════════════════════════"
    @echo "🐍 Python Environment: $(cd apps/doc-service && source venv/bin/activate && python --version 2>/dev/null || echo 'Virtual env not found')"
    @echo "📦 Docling Status: $(cd apps/doc-service && source venv/bin/activate && python -c 'import docling; print("✅ Ready")' 2>/dev/null || echo '❌ Not installed')"
    @echo "📁 Processed Docs: $(ls -1 apps/doc-service/processed_documents/ 2>/dev/null | wc -l | tr -d ' ') files"
    @echo "🌐 Service Status: $(curl -s http://localhost:8000/health >/dev/null 2>&1 && echo '✅ Running' || echo '❌ Stopped')"
    @echo "📋 Git Status: $(git status --porcelain | wc -l | tr -d ' ') uncommitted changes"
    @echo "────────────────────────────────────────────────────────────"
    @echo "\n🎯 Quick Commands:"
    @echo "  just tui        - Launch Terminal UI 💻"
    @echo "  just service    - Start service (background)"
    @echo "  just service-fg - Start service (foreground)"
    @echo "  just stop       - Stop service"
    @echo "  just restart    - Restart service"
    @echo "  just status     - Check system status"
    @echo "  just --list     - Show all available commands"
    @echo "\n💡 Activating virtual environment..."
    @cd apps/doc-service && source venv/bin/activate && exec zsh

# Kill any existing service on port 8000
kill-service:
    @echo "🐹 Checking for existing service on port 8000..."
    @lsof -ti:8000 | xargs kill -9 2>/dev/null || echo "No existing service found"
    @echo "✅ Port 8000 is now free"

# Start development environment
dev:
    @echo "🐹 Starting CHONKER development environment..."
    @echo "🛑 Stopping any existing service..."
    @lsof -ti:8000 | xargs kill -9 2>/dev/null || echo "No existing service found"
    @echo "🐍 Activating virtual environment..."
    cd apps/doc-service && source venv/bin/activate && python main.py

# Start document processing service (background mode)
service:
    @echo "🐹 Starting CHONKER document processing service..."
    @echo "🛑 Stopping any existing service..."
    @lsof -ti:8000 | xargs kill -9 2>/dev/null || echo "No existing service found"
    @sleep 1
    @echo "🚀 Starting service in background..."
    @cd apps/doc-service && source venv/bin/activate && nohup python main.py > service.log 2>&1 &
    @sleep 2
    @echo "✅ Service started in background (PID: $(lsof -ti:8000))"
    @echo "🌐 Service URL: http://localhost:8000"
    @echo "📚 API docs: http://localhost:8000/docs"
    @echo "📋 View logs: tail -f apps/doc-service/service.log"
    @echo "🛑 Stop with: just stop"

# Start service in foreground (for debugging)
service-fg:
    @echo "🐹 Starting CHONKER service in foreground..."
    @echo "🛑 Stopping any existing service..."
    @lsof -ti:8000 | xargs kill -9 2>/dev/null || echo "No existing service found"
    @sleep 1
    @echo "🚀 Service will be available at http://localhost:8000"
    @echo "📚 API docs at http://localhost:8000/docs"
    @echo "💾 Press Ctrl+C to stop"
    cd apps/doc-service && source venv/bin/activate && python main.py

# Stop background service
stop:
    @echo "🐹 Stopping CHONKER service..."
    @lsof -ti:8000 | xargs kill -9 2>/dev/null || echo "No service running"
    @echo "✅ Service stopped"

# Restart service
restart:
    @echo "🐹 Restarting CHONKER service..."
    @just stop
    @sleep 1
    @just service

# Launch CHONKER Terminal UI
tui:
    @echo "🐹 Launching CHONKER Terminal UI..."
    @echo "💻 Starting terminal interface for document processing"
    cd apps/doc-service && source venv/bin/activate && python chonker_terminal_ui.py

# Show system status
status:
    @echo "🐹 CHONKER System Status"
    @echo "════════════════════════════════════════════════════════════"
    @echo "🐍 Python: $(cd apps/doc-service && source venv/bin/activate && python --version 2>/dev/null || echo 'Virtual env not found')"
    @echo "📦 Docling: $(cd apps/doc-service && source venv/bin/activate && python -c 'import docling; print("✅ Installed")' 2>/dev/null || echo '❌ Not installed')"
    @echo "🌐 Service: $(curl -s http://localhost:8000/health >/dev/null 2>&1 && echo '✅ Running on port 8000' || echo '❌ Not running')"
    @echo "📁 Output dir: $(ls -la apps/doc-service/processed_documents/ 2>/dev/null | wc -l | tr -d ' ') files"
    @echo "🔄 Git: $(git status --porcelain | wc -l | tr -d ' ') uncommitted changes"
    @echo "🌐 Network: $(ping -c 1 google.com > /dev/null 2>&1 && echo '✅ Connected' || echo '❌ Offline')"
    @echo "────────────────────────────────────────────────────────────"

# Show project info
info:
    @echo "🐹 CHONKER SNYFTER - Document Processing Pipeline"
    @echo "📄 Status: $(git status --porcelain | wc -l | tr -d ' ') uncommitted changes"
    @echo "🐍 Python: $(python --version 2>/dev/null || echo 'Not found')"
    @echo "📦 Docling: $(python -c 'import docling; print("Installed")' 2>/dev/null || echo 'Not installed')"
    @echo "📁 Processed docs: $(ls -1 apps/doc-service/processed_documents/ 2>/dev/null | wc -l | tr -d ' ') files"
    @echo "🌐 HTML viewers: $(ls -1 *.html 2>/dev/null | wc -l | tr -d ' ') files"
    @echo "📋 Available commands: $(just --list | grep -E '^    [a-z]' | wc -l | tr -d ' ')"

# Check Python environment
check:
    @echo "🐹 CHONKER Environment Check"
    @echo "────────────────────────────"
    @echo "📁 Working Directory: $(pwd)"
    @echo "🐍 Python: $(python --version 2>/dev/null || echo 'Not found')"
    @echo "📦 Docling: $(python -c 'import docling; print("✅ Installed")' 2>/dev/null || echo '❌ Not installed')"
    @echo "📁 Output directory: $(ls -la apps/doc-service/processed_documents/ 2>/dev/null | wc -l | tr -d ' ') files"
    @echo "🌐 Network: $(ping -c 1 google.com > /dev/null 2>&1 && echo 'Connected' || echo 'Offline')"
    @echo "────────────────────────────"

# Install Python dependencies
install:
    @echo "🐹 Installing Python dependencies..."
    cd apps/doc-service && pip install -r requirements.txt
    @echo "✅ Dependencies installed"

# Process a document (usage: just process document.pdf)
process FILE:
    @echo "🐹 Processing document: {{FILE}}"
    python process_document.py "{{FILE}}"

# Generate viewer for already processed document
viewer BASENAME:
    @echo "🐹 Generating viewer for: {{BASENAME}}"
    python generate_viewer.py "{{BASENAME}}"

# List processed documents
list:
    @echo "🐹 Processed Documents:"
    @ls -la apps/doc-service/processed_documents/ 2>/dev/null || echo "No processed documents found"
    @echo ""
    @echo "🌐 HTML Viewers:"
    @ls -la *.html 2>/dev/null || echo "No HTML viewers found"

# Clean processed files
clean:
    @echo "🐹 Cleaning processed files..."
    rm -rf apps/doc-service/processed_documents/*
    rm -f *.html
    @echo "✅ Clean completed"

# Start Python backend service
backend:
    @echo "🐹 Starting Python backend service..."
    cd apps/doc-service && python main.py

# Git workflow commands
git-status:
    @echo "🐹 Git Status"
    @echo "────────────────────────────"
    git status
    @echo "────────────────────────────"
    @echo "📈 Branch: $(git branch --show-current)"
    @echo "📊 Changes: $(git status --porcelain | wc -l) files"
    @echo "🔄 Commits ahead: $(git rev-list --count HEAD ^origin/$(git branch --show-current) 2>/dev/null || echo '0')"
    @echo "📥 Commits behind: $(git rev-list --count origin/$(git branch --show-current) ^HEAD 2>/dev/null || echo '0')"

# Add all changes to git
git-add:
    @echo "🐹 Adding all changes to git..."
    git add .
    @echo "✅ Changes added"

# Commit with message
git-commit message:
    @echo "🐹 Committing changes..."
    git commit -m "{{message}}"
    @echo "✅ Changes committed"

# Pull from remote
git-pull:
    @echo "🐹 Pulling from remote..."
    git pull
    @echo "✅ Pull completed"

# Push to remote
git-push:
    @echo "🐹 Pushing to remote..."
    git push
    @echo "✅ Push completed"

# Quick commit with auto-generated message
git-quick:
    @echo "🐹 Quick commit..."
    git add .
    git commit -m "Quick update: $(date '+%Y-%m-%d %H:%M:%S')"
    @echo "✅ Quick commit completed"

# Full workflow: add, commit, push
git-save message:
    @echo "🐹 Full git workflow..."
    git add .
    git commit -m "{{message}}"
    git push
    @echo "✅ Full workflow completed"

# Create .gitignore entries for common files
git-ignore:
    @echo "🐹 Creating .gitignore entries..."
    echo "# CHONKER SNYFTER specific" >> .gitignore
    echo "*.db" >> .gitignore
    echo "*.db-journal" >> .gitignore
    echo "*.log" >> .gitignore
    echo "# IDE and OS" >> .gitignore
    echo ".DS_Store" >> .gitignore
    echo "Thumbs.db" >> .gitignore
    echo "# Rust" >> .gitignore
    echo "src-tauri/target/" >> .gitignore
    echo "**/*.rs.bk" >> .gitignore
    echo "# Node" >> .gitignore
    echo "node_modules/" >> .gitignore
    echo "dist/" >> .gitignore
    echo ".vite/" >> .gitignore
    @echo "✅ .gitignore updated"
