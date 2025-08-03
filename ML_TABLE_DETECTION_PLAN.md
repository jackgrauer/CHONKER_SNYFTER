# ML-Based Table Detection Plan for CHONKER 5

## Current Status

### layoutparser-ort Issues
- The crate has API compatibility issues with ort 2.0
- It was written for an older version of the ort crate
- The imports `Session`, `SessionBuilder`, `SessionOutputs` have moved to `ort::session::*`
- Would require forking and updating the crate to work with current ort version

### Options for ML-Based Table Detection

#### Option 1: Fix layoutparser-ort (Medium effort)
1. Fork the layoutparser-ort repository
2. Update imports to use `ort::session::*`
3. Fix the input macro issues
4. Test with pretrained models
5. Integrate into CHONKER 5

#### Option 2: Use Python Bridge (Low effort)
1. Use existing Python libraries (detectron2, layoutparser, etc.)
2. Convert PDF pages to images using pdftoppm
3. Call Python script from Rust
4. Parse JSON results back in Rust

#### Option 3: Direct ONNX Integration (High effort)
1. Download pretrained ONNX models directly
2. Use ort crate directly without layoutparser-ort wrapper
3. Implement the image preprocessing and postprocessing
4. More control but more work

#### Option 4: Cloud API (Low effort, requires internet)
1. Use services like AWS Textract, Google Document AI
2. Send PDF pages to API
3. Get back table detection results
4. Requires API keys and internet connection

## Recommended Approach

For CHONKER 5, I recommend starting with **Option 2 (Python Bridge)** because:
- It's the quickest to implement
- Python has mature libraries for document analysis
- We can always migrate to a pure Rust solution later
- It allows testing different models easily

## Implementation Steps

1. **PDF to Image Conversion**
   ```bash
   pdftoppm -png -r 300 input.pdf output_dir/page
   ```

2. **Python Table Detection Script**
   - Already created: `pdf_table_detector.py`
   - Supports multiple backends (camelot, pdfplumber, tabula)

3. **Rust Integration**
   - Call Python script via subprocess
   - Parse JSON results
   - Overlay table boundaries on PDF view

4. **Future Enhancement**
   - Once working, consider fixing layoutparser-ort
   - Or implement direct ONNX inference in Rust

## Benefits of ML-Based Detection

- Better accuracy than heuristic-based detection
- Can detect complex table layouts
- Handles borderless tables
- Can distinguish between tables and other structured content
- Pre-trained models available for document layouts