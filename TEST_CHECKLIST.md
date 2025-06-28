# CHONKER Manual Test Checklist

Run these commands to verify everything works:

## ✅ Basic Tests (2 minutes)

```bash
# 1. Check CLI help
./target/debug/chonker --help

# 2. Check database status  
./target/debug/chonker status

# 3. Test Python bridge directly
python3 python/extraction_bridge.py "/Users/jack/Documents/1.pdf" --page 1

# 4. Extract a PDF
./target/debug/chonker extract "/Users/jack/Documents/1.pdf" --output test.md --store

# 5. Process markdown with corrections
./target/debug/chonker process test.md --correct --output test_corrected.md

# 6. Export to CSV
./target/debug/chonker export -f csv -o test_export.csv

# 7. Export to JSON  
./target/debug/chonker export -f json -o test_export.json
```

## ✅ Quality Check (1 minute)

```bash
# Generate quality report
./scripts/quality_check.sh

# View the report
cat reports/quality_report.md
```

## ✅ Performance Test (30 seconds)

```bash
# Benchmark extraction
./scripts/dev_workflow.sh benchmark "/Users/jack/Documents/1.pdf"
```

## ✅ TUI Test (Optional)

```bash
# Launch interactive TUI
./target/debug/chonker tui
# Press 'q' to quit
```

## ✅ Expected Results

After running these tests, you should have:

- ✅ **test.md** - Extracted markdown
- ✅ **test_corrected.md** - Processed markdown  
- ✅ **test_export.csv** - CSV export with your data
- ✅ **test_export.json** - JSON export
- ✅ **reports/** directory with quality analysis

## 🎯 What This Tests

1. **PDF Extraction Pipeline** - Python bridge → Rust CLI
2. **Markdown Processing** - OCR corrections, formatting
3. **Database Storage** - SQLite with proper schema
4. **Data Export** - Polars DataFrame → CSV/JSON  
5. **Performance** - Sub-second extraction times
6. **Quality Analysis** - Reports and statistics

## 🚨 Troubleshooting

If something fails:

1. **Python issues**: Install dependencies with `./scripts/dev_workflow.sh install-python`
2. **Path issues**: Run commands from the project root `/Users/jack/chonker-tui`
3. **PDF issues**: Make sure `/Users/jack/Documents/1.pdf` exists
4. **Permission issues**: Check file permissions with `ls -la`

## ✅ Success Criteria

- [ ] All commands run without errors
- [ ] Output files are generated and not empty
- [ ] CSV has proper headers and data
- [ ] JSON is valid (test with `python3 -m json.tool test_export.json`)
- [ ] Database shows increasing document count
- [ ] Extraction time is under 1 second
