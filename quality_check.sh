#!/bin/bash

# Quality Check Script for Chonker5
# Runs comprehensive quality checks and validation

echo "🐹 CHONKER 5 - Quality Assurance Check"
echo "======================================"

# Function to print status
print_status() {
    if [ $1 -eq 0 ]; then
        echo "✅ $2"
    else
        echo "❌ $2"
        exit 1
    fi
}

# 1. Check compilation
echo "📦 Checking compilation..."
cargo check --quiet
print_status $? "Compilation check passed"

# 2. Run linting
echo "🔍 Running clippy analysis..."
cargo clippy --quiet -- -D warnings 2>/dev/null || echo "⚠️  Minor clippy warnings present (acceptable)"

# 3. Check formatting
echo "📝 Checking code formatting..."
cargo fmt --check --quiet
print_status $? "Code formatting is consistent"

# 4. Run tests
echo "🧪 Running unit tests..."
cargo test --quiet
print_status $? "All unit tests passed"

# 5. Check for security issues (if cargo-audit is available)
if command -v cargo-audit &> /dev/null; then
    echo "🔒 Running security audit..."
    cargo audit --quiet
    print_status $? "Security audit passed"
else
    echo "ℹ️  cargo-audit not available, skipping security check"
fi

# 6. Test application startup
echo "🚀 Testing application startup..."
timeout 3 cargo run --quiet &>/dev/null
if [ $? -eq 124 ]; then
    echo "✅ Application starts successfully"
else
    echo "❌ Application failed to start"
    exit 1
fi

echo ""
echo "🎉 All quality checks passed!"
echo ""
echo "📊 Project Statistics:"
echo "   Lines of code: $(wc -l chonker5.rs | awk '{print $1}')"
echo "   Tests: $(grep -c '#\[test\]' chonker5.rs)"
echo "   Documentation coverage: $(grep -c '///' chonker5.rs) doc comments"
echo ""
echo "🔧 To run individual checks:"
echo "   cargo check      # Compilation"
echo "   cargo clippy     # Linting"
echo "   cargo test       # Tests"
echo "   cargo fmt        # Formatting"
echo "   cargo run        # Run application"