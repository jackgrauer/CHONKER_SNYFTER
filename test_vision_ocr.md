# Ferrules Table Extraction Investigation

## The Problem
Ferrules is extracting table captions and descriptions but not the actual table data. For example:
- ✅ Extracts: "Table 1 shows information related to..."
- ✅ Extracts: "Table 1 City of Philadelphia..."  
- ❌ Missing: The actual table rows with data

## Apple Vision Integration
Ferrules uses Apple Vision for OCR on macOS, but it seems to be:
1. Not detecting tables as structured data
2. Only using Vision for text that can't be extracted directly from PDF

## Possible Solutions

### 1. Force OCR Mode
Some PDF parsers have options to force OCR even on text-based PDFs. This might help if tables are rendered differently.

### 2. Use External Table Detection
Since our ultra-aggressive detection finds where tables SHOULD be, we could:
- Use our detection to identify table regions
- Extract those regions as images
- Run dedicated table extraction on those regions

### 3. Alternative PDF Parsers
- `pdf-extract` - Another Rust PDF library
- Python bridges to `camelot` or `tabula`
- Direct Apple Vision API for table detection

### 4. Hybrid Approach
1. Use ferrules for general text extraction
2. Use our detection to find table locations
3. Use specialized tools for just those regions

## Next Steps
The issue appears to be that ferrules doesn't have specific table extraction logic - it treats tables as regular text blocks or skips them entirely.