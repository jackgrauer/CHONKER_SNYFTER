#!/bin/bash
# CHONKER Quality Check Script
# Analyze extraction quality and generate reports

set -e

CHONKER_BIN="./target/debug/chonker"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1"
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

usage() {
    echo "Usage: $0 [options]"
    echo ""
    echo "Options:"
    echo "  -o, --output DIR    Output directory for reports"
    echo "  -f, --format FORMAT Export format (csv, json, parquet)"
    echo "  -h, --help          Show this help"
}

OUTPUT_DIR="./reports"
EXPORT_FORMAT="csv"

while [[ $# -gt 0 ]]; do
    case $1 in
        -o|--output)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        -f|--format)
            EXPORT_FORMAT="$2"
            shift 2
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

mkdir -p "$OUTPUT_DIR"

log "🔍 CHONKER Quality Check Started"

# Generate database report
log "📊 Generating database statistics..."
$CHONKER_BIN status > "$OUTPUT_DIR/database_stats.txt"

# Export all data for analysis
log "📄 Exporting full dataset..."
$CHONKER_BIN export --format "$EXPORT_FORMAT" --output "$OUTPUT_DIR/full_dataset.$EXPORT_FORMAT"

# Create summary report
log "📝 Creating summary report..."
cat > "$OUTPUT_DIR/quality_report.md" << EOF
# CHONKER Quality Report
Generated: $(date)

## Database Statistics
\`\`\`
$(cat "$OUTPUT_DIR/database_stats.txt")
\`\`\`

## Data Export
- Format: $EXPORT_FORMAT
- File: full_dataset.$EXPORT_FORMAT
- Location: $OUTPUT_DIR

## Quick Analysis
To analyze the data:

### Python
\`\`\`python
import pandas as pd
df = pd.read_csv('$OUTPUT_DIR/full_dataset.csv')
print("Document count:", df['document_id'].nunique())
print("Average chunk length:", df['char_count'].mean())
print("Tools used:", df['filename'].value_counts())
\`\`\`

### CLI Analysis
\`\`\`bash
# Count documents
cut -d, -f1 $OUTPUT_DIR/full_dataset.csv | sort | uniq -c

# Average chunk size
awk -F, 'NR>1 {sum+=\$7; count++} END {print "Average chars:", sum/count}' $OUTPUT_DIR/full_dataset.csv

# Tool distribution
cut -d, -f2 $OUTPUT_DIR/full_dataset.csv | sort | uniq -c
\`\`\`
EOF

success "✅ Quality check completed"
log "Reports saved to: $OUTPUT_DIR"
log "📖 View summary: cat $OUTPUT_DIR/quality_report.md"
