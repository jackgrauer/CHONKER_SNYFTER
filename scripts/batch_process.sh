#!/bin/bash
# CHONKER Batch Processing Script
# Process multiple PDFs through the complete pipeline

set -e

CHONKER_BIN="./target/debug/chonker"
OUTPUT_DIR="./processed"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

usage() {
    echo "Usage: $0 <pdf_directory> [options]"
    echo ""
    echo "Options:"
    echo "  -t, --tool TOOL     Extraction tool (auto, magic-pdf, docling)"
    echo "  -c, --correct       Apply markdown corrections"
    echo "  -s, --store         Store results in database"
    echo "  -e, --export FORMAT Export to format (csv, json, parquet)"
    echo "  -h, --help          Show this help"
    echo ""
    echo "Example:"
    echo "  $0 /path/to/pdfs -t docling -c -s -e csv"
}

log() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Parse arguments
PDF_DIR=""
TOOL="auto"
CORRECT=false
STORE=false
EXPORT_FORMAT=""

while [[ $# -gt 0 ]]; do
    case $1 in
        -t|--tool)
            TOOL="$2"
            shift 2
            ;;
        -c|--correct)
            CORRECT=true
            shift
            ;;
        -s|--store)
            STORE=true
            shift
            ;;
        -e|--export)
            EXPORT_FORMAT="$2"
            shift 2
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        -*)
            error "Unknown option $1"
            usage
            exit 1
            ;;
        *)
            if [[ -z "$PDF_DIR" ]]; then
                PDF_DIR="$1"
            else
                error "Multiple directories specified"
                usage
                exit 1
            fi
            shift
            ;;
    esac
done

if [[ -z "$PDF_DIR" ]]; then
    error "PDF directory required"
    usage
    exit 1
fi

if [[ ! -d "$PDF_DIR" ]]; then
    error "Directory $PDF_DIR does not exist"
    exit 1
fi

# Create output directory
mkdir -p "$OUTPUT_DIR"

log "🐹 CHONKER Batch Processing Started"
log "Directory: $PDF_DIR"
log "Tool: $TOOL"
log "Apply corrections: $CORRECT"
log "Store in database: $STORE"
[[ -n "$EXPORT_FORMAT" ]] && log "Export format: $EXPORT_FORMAT"

# Find all PDFs
PDF_FILES=($(find "$PDF_DIR" -name "*.pdf" -type f))
TOTAL_FILES=${#PDF_FILES[@]}

if [[ $TOTAL_FILES -eq 0 ]]; then
    warn "No PDF files found in $PDF_DIR"
    exit 0
fi

log "Found $TOTAL_FILES PDF files"

# Process each PDF
PROCESSED=0
FAILED=0

for pdf_file in "${PDF_FILES[@]}"; do
    filename=$(basename "$pdf_file")
    base_name="${filename%.*}"
    
    log "Processing: $filename"
    
    # Extract PDF
    extract_args=(extract "$pdf_file" --tool "$TOOL" --output "$OUTPUT_DIR/${base_name}.md")
    [[ "$STORE" == true ]] && extract_args+=(--store)
    
    if $CHONKER_BIN "${extract_args[@]}"; then
        success "✅ Extracted: $filename"
        
        # Apply corrections if requested
        if [[ "$CORRECT" == true ]]; then
            if $CHONKER_BIN process "$OUTPUT_DIR/${base_name}.md" --correct --output "$OUTPUT_DIR/${base_name}_corrected.md"; then
                success "✅ Corrected: $filename"
            else
                warn "⚠️  Failed to correct: $filename"
            fi
        fi
        
        ((PROCESSED++))
    else
        error "❌ Failed to extract: $filename"
        ((FAILED++))
    fi
done

# Export if requested
if [[ -n "$EXPORT_FORMAT" ]] && [[ $PROCESSED -gt 0 ]]; then
    export_file="$OUTPUT_DIR/batch_export.${EXPORT_FORMAT}"
    log "Exporting to $export_file"
    
    if $CHONKER_BIN export --format "$EXPORT_FORMAT" --output "$export_file"; then
        success "✅ Export completed: $export_file"
    else
        error "❌ Export failed"
    fi
fi

# Summary
log "🎉 Batch processing completed"
log "Total files: $TOTAL_FILES"
success "Processed: $PROCESSED"
[[ $FAILED -gt 0 ]] && error "Failed: $FAILED"

# Show database status
log "Database status:"
$CHONKER_BIN status
