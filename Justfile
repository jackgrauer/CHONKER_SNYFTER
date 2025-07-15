# 🐹 CHONKER & 🐁 SNYFTER Development Commands
# Character-driven development with just

# Default recipe shows available commands
default:
    @just --list --unsorted

# 🐹 CHONKER Commands
chonker-feed pdf_path:
    #!/usr/bin/env bash
    echo "🐹 *sniff sniff* Feeding {{pdf_path}} to CHONKER..."
    python chonker_snyfter.py --mode chonker --input "{{pdf_path}}"

chonker-test:
    #!/usr/bin/env bash
    echo "🐹 Testing CHONKER's digestion system..."
    pytest tests/test_chonker.py -v

# 🐁 SNYFTER Commands  
snyfter-search query:
    #!/usr/bin/env bash
    echo "🐁 *adjusts glasses* Searching archives for: {{query}}"
    python chonker_snyfter.py --mode snyfter --search "{{query}}"

snyfter-catalog:
    #!/usr/bin/env bash
    echo "🐁 Cataloging recent additions..."
    python chonker_snyfter.py --mode snyfter --catalog

# Development Commands
dev:
    #!/usr/bin/env bash
    echo "🐹🐁 Starting development mode..."
    source ../chonksnyft-env/bin/activate
    python chonker_snyfter.py

install:
    #!/usr/bin/env bash
    echo "📦 Installing dependencies for our furry friends..."
    source ../chonksnyft-env/bin/activate
    pip install -r requirements.txt

# Git Commands with Character Flair
commit message:
    #!/usr/bin/env bash
    echo "💾 Committing with message: {{message}}"
    git add -A
    git commit -m "{{message}} 🐹🐁"

push:
    #!/usr/bin/env bash
    echo "📤 Pushing to hamster burrow (remote repo)..."
    git push

status:
    #!/usr/bin/env bash
    echo "📊 Checking what CHONKER and SNYFTER have been up to..."
    git status

# Testing & Quality
test-all:
    #!/usr/bin/env bash
    echo "🧪 Running all tests..."
    just chonker-test
    just snyfter-test
    just test-integration

lint:
    #!/usr/bin/env bash
    echo "🧹 Tidying up the code..."
    ruff check .
    ruff format .

# Character Consistency Checks
check-characters:
    #!/usr/bin/env bash
    echo "🎭 Checking character consistency..."
    python scripts/check_character_consistency.py

# Database Management
db-init:
    #!/usr/bin/env bash
    echo "🗄️ Initializing SNYFTER's card catalog..."
    python -c "from chonker_snyfter import SnyfterDatabase; SnyfterDatabase().init_archives()"

# Quick Commands
run: dev
test: test-all
clean:
    rm -rf __pycache__ .pytest_cache *.pyc