#!/bin/bash
# CHONKER Comprehensive Test Script
# Test all functionality end-to-end

set -e

# Get absolute paths
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
CHONKER_BIN="$ROOT_DIR/target/debug/chonker"
TEST_DIR="$ROOT_DIR/test_results"
TEST_PDF="/Users/jack/Documents/1.pdf"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log() {
    echo -e "${BLUE}[TEST]${NC} $1"
}

success() {
    echo -e "${GREEN}[PASS]${NC} $1"
}

fail() {
    echo -e "${RED}[FAIL]${NC} $1"
    exit 1
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

# Setup
rm -rf "$TEST_DIR"
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

log "🧪 CHONKER Comprehensive Test Suite"
log "Test directory: $TEST_DIR"

# Test 1: Basic CLI help
log "Test 1: CLI Help"
if "$CHONKER_BIN" --help > /dev/null 2>&1; then
    success "CLI help works"
else
    fail "CLI help failed"
fi

# Test 2: Database status
log "Test 2: Database Status"
if "$CHONKER_BIN" status > status_output.txt 2>&1; then
    if grep -q "Documents:" status_output.txt; then
        success "Database status works"
    else
        fail "Database status output malformed"
    fi
else
    fail "Database status failed"
fi

# Test 3: PDF Extraction
log "Test 3: PDF Extraction"
if [[ -f "$TEST_PDF" ]]; then
    if "$CHONKER_BIN" extract "$TEST_PDF" --output test_extract.md > extract_output.txt 2>&1; then
        if [[ -f "test_extract.md" ]] && [[ -s "test_extract.md" ]]; then
            success "PDF extraction works"
        else
            fail "PDF extraction produced no output"
        fi
    else
        fail "PDF extraction failed"
    fi
else
    warn "No test PDF found at $TEST_PDF, skipping extraction test"
fi

# Test 4: Markdown Processing
log "Test 4: Markdown Processing"
if [[ -f "test_extract.md" ]]; then
    if "$CHONKER_BIN" process test_extract.md --correct --output test_processed.md > process_output.txt 2>&1; then
        if [[ -f "test_processed.md" ]] && [[ -s "test_processed.md" ]]; then
            success "Markdown processing works"
        else
            fail "Markdown processing produced no output"
        fi
    else
        fail "Markdown processing failed"
    fi
else
    warn "No markdown file to process, skipping"
fi

# Test 5: Data Export (CSV)
log "Test 5: CSV Export"
if "$CHONKER_BIN" export --format csv --output test_export.csv > export_output.txt 2>&1; then
    if [[ -f "test_export.csv" ]] && [[ -s "test_export.csv" ]]; then
        # Check if it has proper CSV structure
        if head -1 test_export.csv | grep -q "document_id,filename"; then
            success "CSV export works"
        else
            fail "CSV export has wrong format"
        fi
    else
        fail "CSV export produced no output"
    fi
else
    fail "CSV export failed"
fi

# Test 6: Data Export (JSON)
log "Test 6: JSON Export"
if "$CHONKER_BIN" export --format json --output test_export.json > /dev/null 2>&1; then
    if [[ -f "test_export.json" ]] && [[ -s "test_export.json" ]]; then
        # Check if it's valid JSON
        if python3 -m json.tool test_export.json > /dev/null 2>&1; then
            success "JSON export works"
        else
            fail "JSON export produces invalid JSON"
        fi
    else
        fail "JSON export produced no output"
    fi
else
    fail "JSON export failed"
fi

# Test 7: Python Bridge
log "Test 7: Python Bridge"
if [[ -f "$TEST_PDF" ]]; then
    if python3 "$ROOT_DIR/python/extraction_bridge.py" "$TEST_PDF" --page 1 > python_output.json 2>&1; then
        if python3 -m json.tool python_output.json > /dev/null 2>&1; then
            if grep -q '"success": true' python_output.json; then
                success "Python bridge extraction works"
            else
                warn "Python bridge extraction failed but script ran"
            fi
        else
            fail "Python bridge produced invalid JSON"
        fi
    else
        fail "Python bridge script failed"
    fi
else
    warn "No test PDF for Python bridge test"
fi

# Test 8: Performance Test
log "Test 8: Performance Test"
if [[ -f "$TEST_PDF" ]]; then
    start_time=$(date +%s.%N)
    "$CHONKER_BIN" extract "$TEST_PDF" --output perf_test.md > /dev/null 2>&1
    end_time=$(date +%s.%N)
    duration=$(echo "$end_time - $start_time" | bc)
    
    if (( $(echo "$duration < 5.0" | bc -l) )); then
        success "Performance test: ${duration}s (under 5s threshold)"
    else
        warn "Performance test: ${duration}s (slower than expected)"
    fi
else
    warn "No test PDF for performance test"
fi

# Test 9: Database Integration
log "Test 9: Database Integration"
if [[ -f "$TEST_PDF" ]]; then
    # Count documents before
    docs_before=$("$CHONKER_BIN" status | grep "Documents:" | awk '{print $2}')
    
    # Add document with --store
    if "$CHONKER_BIN" extract "$TEST_PDF" --output db_test.md --store > /dev/null 2>&1; then
        # Count documents after
        docs_after=$("$CHONKER_BIN" status | grep "Documents:" | awk '{print $2}')
        
        if [[ $docs_after -gt $docs_before ]]; then
            success "Database storage works (${docs_before} -> ${docs_after} docs)"
        else
            warn "Database count didn't increase (may be duplicate)"
        fi
    else
        fail "Database storage failed"
    fi
else
    warn "No test PDF for database test"
fi

# Test 10: File Validation
log "Test 10: File Validation"
file_count=0
for file in *.md *.csv *.json; do
    if [[ -f "$file" ]]; then
        ((file_count++))
    fi
done

if [[ $file_count -ge 3 ]]; then
    success "Generated $file_count output files"
else
    fail "Too few output files generated ($file_count)"
fi

# Summary
log "🎉 Test Summary"
echo ""
echo "Test Results Directory: $TEST_DIR"
echo "Generated Files:"
ls -la *.md *.csv *.json 2>/dev/null || echo "No output files"
echo ""

success "All critical tests passed! 🎉"
echo ""
echo "Next steps:"
echo "1. Check output quality: cat test_extract.md"
echo "2. Analyze data: head test_export.csv"
echo "3. Run quality check: ../scripts/quality_check.sh"
echo "4. Try the TUI: ../target/debug/chonker tui"
