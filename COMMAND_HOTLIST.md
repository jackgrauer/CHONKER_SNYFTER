# 🐹 CHONKER_SNYFTER - Command Hotlist

## 🚀 DEVELOPMENT COMMANDS
```bash
just dev           # Start full development (Tauri + frontend + backend)
just status        # Check project status (Rust, frontend, Tauri)
just install       # Install all dependencies (Python, frontend, Tauri)
just build         # Build the project for production
just clean         # Clean all build artifacts
```

## 🔧 COMPONENT-SPECIFIC
```bash
just backend       # Start backend server only
just frontend      # Start frontend server only
just test          # Run all tests
just fmt           # Format all code
just lint          # Run linters
just fix           # Fix common issues
```

## 🐍 PYTHON ENVIRONMENT
```bash
just setup-venv    # Create Python virtual environment
just install-python # Install Python dependencies
just activate      # Show how to activate venv
source .venv/bin/activate  # Actually activate venv
```

## 📊 GIT WORKFLOW
```bash
just git-status    # Show git status + recent commits
just git-add       # Add all changes to staging
just git-commit "message"  # Commit with custom message
just git-quick     # Quick commit with auto timestamp
just git-save "message"    # Add + commit + push in one go
just git-push      # Push to remote
just git-pull      # Pull from remote
```

## 📋 UTILITIES
```bash
just info          # Show project info (versions, paths)
just --list        # Show all available commands
just git-ignore    # Add common .gitignore patterns
```

## 🌐 URLS & PATHS
```
Frontend:    http://localhost:5173
Backend:     src-tauri/src/
Frontend:    frontend/chonker-modern/src/
Config:      src-tauri/tauri.conf.json
Python:      .venv/bin/activate
```

## ⚡ WARP SHORTCUTS (if configured)
```bash
gs    # git status
ga    # git add
gc    # git commit
gq    # git quick commit
gp    # git push
gl    # git pull
save  # git save workflow
```

## 🔥 EMERGENCY COMMANDS
```bash
just clean && just install  # Nuclear reset
rm -rf node_modules && just install  # Fix npm issues
cargo clean && just dev     # Fix Rust issues
```

---
**Project:** CHONKER_SNYFTER  
**Location:** /Users/jack/CHONKER_SNYFTER  
**Last Updated:** July 10, 2025
