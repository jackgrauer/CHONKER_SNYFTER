# CHONKER SNYFTER - Turborepo Monorepo
# Document processing pipeline with Tauri 2.3.0 + Turborepo

set shell := ["zsh", "-c"]

default:
    @just --list

# Show project info
info:
    @echo "🐹 CHONKER SNYFTER - Turborepo Monorepo"
    @echo "📄 Status: $(git status --porcelain | wc -l | tr -d ' ') uncommitted changes"
    @echo "🦀 Rust: $(rustc --version)"
    @echo "📦 Node: $(node --version)"
    @echo "🔧 Tauri: $(cargo tauri --version)"
    @echo "⚡ Turbo: $(turbo --version)"
    @echo "📚 Workspaces: $(find apps packages -name package.json | wc -l | tr -d ' ') packages"
    @echo "🏗️  Last build: $(ls -la apps/tauri-app/target/release 2>/dev/null | wc -l | tr -d ' ') files"
    @echo "📋 Available commands: $(just --list | grep -E '^    [a-z]' | wc -l | tr -d ' ')"

# Check the status of the project
status:
    @echo "🐹 CHONKER SNYFTER Status Check"
    @echo "────────────────────────────"
    @echo "📁 Working Directory: $(pwd)"
    @echo "🦀 Rust: $(rustc --version)"
    @echo "📦 Node: $(node --version)"
    @echo "🔧 Tauri: $(cargo tauri --version || echo 'Not installed')"
    @echo "📚 Frontend deps: $(ls frontend/chonker-modern/node_modules 2>/dev/null | wc -l) packages"
    @echo "🔍 Recent activity: $(git log --oneline -5 | wc -l) recent commits"
    @echo "🏗️  Build status: $(cargo check 2>/dev/null && echo 'OK' || echo 'Needs attention')"
    @echo "🌐 Network: $(ping -c 1 google.com >/dev/null 2>&1 && echo 'Connected' || echo 'Offline')"
    @echo "────────────────────────────"
    @echo "🚀 Ready for $(just --list | grep -E '^    [a-z]' | wc -l) available commands"

# Install all dependencies
install:
    @echo "🐹 Installing all dependencies..."
    npm install
    @echo "✅ All dependencies installed"

# Run development servers with Turborepo
dev:
    @echo "🐹 Starting development servers..."
    npm run dev

# Start frontend only
frontend:
    @echo "🐹 Starting frontend..."
    cd apps/frontend && npm run dev

# Run tests with Turborepo
test:
    @echo "🐹 Running tests..."
    npm run test
    @echo "✅ Tests completed"

# Format code with Turborepo
fmt:
    @echo "🐹 Formatting code..."
    npm run format
    @echo "✅ Code formatted"

# Run linter with Turborepo
lint:
    @echo "🐹 Running linter..."
    npm run lint
    @echo "✅ Linting completed"

# Build the project with Turborepo
build:
    @echo "🐹 Building CHONKER SNYFTER..."
    npm run build
    @echo "✅ Build completed"

# Clean build artifacts
clean:
    @echo "🐹 Cleaning build artifacts..."
    npm run clean
    cargo clean
    rm -rf apps/tauri-app/target/
    find apps packages -name "dist" -type d -exec rm -rf {} + 2>/dev/null || true
    find apps packages -name ".vite" -type d -exec rm -rf {} + 2>/dev/null || true
    @echo "✅ Clean completed"

# Fix common issues
fix:
    @echo "🐹 Fixing common issues..."
    @echo "🔧 Clearing frontend cache..."
    cd frontend/chonker-modern && rm -rf node_modules/.vite dist
    @echo "🔧 Clearing Rust cache..."
    cargo clean
    @echo "🔧 Reinstalling dependencies..."
    just install
    @echo "✅ Common issues fixed"

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
