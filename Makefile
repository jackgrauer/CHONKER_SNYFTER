# 🐹🐁 CHONKER & SNYFTER Makefile
# For those who prefer make over just

.PHONY: help dev test lint clean

help: ## Show this help
	@echo "🐹 CHONKER & 🐁 SNYFTER Development Commands"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

dev: ## Run in development mode
	@echo "🐹🐁 Starting CHONKER & SNYFTER..."
	@source ../chonksnyft-env/bin/activate && python chonker_snyfter.py

test: ## Run all tests
	@echo "🧪 Testing our furry friends..."
	@pytest tests/ -v

lint: ## Lint and format code
	@echo "🧹 CHONKER is tidying up..."
	@ruff check . --fix
	@ruff format .

clean: ## Clean up temporary files
	@echo "🧹 Cleaning the hamster cage..."
	@find . -type f -name "*.pyc" -delete
	@find . -type d -name "__pycache__" -delete
	@rm -rf .pytest_cache .ruff_cache

install: ## Install dependencies
	@echo "📦 Installing hamster food (dependencies)..."
	@pip install -r requirements.txt

git-status: ## Check git status with character flair
	@echo "🐹 CHONKER & 🐁 SNYFTER's changes:"
	@git status --short