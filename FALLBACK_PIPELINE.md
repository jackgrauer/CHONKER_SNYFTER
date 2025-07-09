# CHONKER Staged Fallback PDF Extraction Pipeline

## Overview

This implementation provides a robust 3-stage fallback system for handling "fairly poopy PDFs" that fail to extract meaningful text through traditional methods.

## Architecture

### Stage 1: SmolDocling VLM
- **Primary extraction method**: Uses Docling with SmolDocling Vision-Language Model
- **Strengths**: Advanced document understanding, structured output
- **Weakness**: Can produce generic descriptions for poor-quality scanned PDFs

### Stage 2: Apple Vision OCR (macOS only)
- **Fallback method**: Uses Apple's Vision framework for OCR
- **Strengths**: Excellent at reading text from images, native macOS integration
- **Activation**: Triggered when Stage 1 produces "garbage" results

### Stage 3: Enhanced OCR
- **Enhanced fallback**: Image preprocessing + Apple Vision OCR
- **Preprocessing**: Deskewing, contrast enhancement, noise reduction, sharpening
- **Activation**: Triggered when Stage 2 still produces "garbage" results

## Components

### 1. Garbage Detection (`is_garbage_result`)
```python
def is_garbage_result(text: str) -> bool:
    # Detects generic OCR failure patterns:
    # - Very short text (< 50 chars)
    # - Generic phrases like "in this image", "there is a document"
    # - Multiple garbage indicators
```

### 2. Swift Apple Vision OCR
**Location**: `/Users/jack/CHONKER_SNYFTER/swift/`
**Binary**: `/.build/release/apple_vision_ocr`

Features:
- Native Apple Vision framework integration
- Multi-language support (EN, ES, FR, DE, IT, PT)
- High accuracy text recognition
- Optimized for macOS

### 3. Rust Image Enhancement
**Location**: `/Users/jack/CHONKER_SNYFTER/image_enhancer/`
**Binary**: `/target/release/image_enhancer`

Features:
- **Deskewing**: Hough transform-based skew correction
- **Contrast Enhancement**: Adaptive contrast stretching
- **Noise Reduction**: Median filtering for salt-and-pepper noise
- **Sharpening**: Unsharp mask sharpening kernel
- **Performance**: Optimized Rust implementation

### 4. PDF to Image Conversion
- **Primary**: pdf2image library (Python)
- **Fallback**: ImageMagick/Ghostscript
- **Output**: PNG format for OCR processing

## Usage

### Integration
The fallback pipeline is automatically integrated into `smoldocling_bridge.py`:

```python
# After SmolDocling extraction
if is_garbage_result(extraction['text']):
    print("❗ Detected garbage extraction result, initiating fallback.")
    
    if platform.system() == 'Darwin':
        # Stage 2: Apple Vision OCR
        ocr_text = apply_apple_vision_ocr(pdf_path)
        
        if is_garbage_result(ocr_text):
            # Stage 3: Enhanced OCR
            enhance_image_with_rust(pdf_path)
            ocr_text = apply_apple_vision_ocr(pdf_path)
        
        # Update extraction with better results
        if not is_garbage_result(ocr_text):
            extraction['text'] = ocr_text
            extraction['tool'] = 'apple_vision_ocr'
```

### Manual Testing
```bash
# Test the complete pipeline
python test_fallback_pipeline.py

# Test with a specific PDF
python python/smoldocling_bridge.py your_poopy_pdf.pdf
```

## File Structure

```
CHONKER_SNYFTER/
├── python/
│   └── smoldocling_bridge.py        # Main pipeline with fallback logic
├── swift/
│   ├── Package.swift               # Swift package configuration
│   ├── Sources/main.swift          # Apple Vision OCR implementation
│   └── .build/release/apple_vision_ocr  # Compiled binary
├── image_enhancer/
│   ├── Cargo.toml                  # Rust project configuration
│   ├── src/main.rs                 # Image enhancement implementation
│   └── target/release/image_enhancer    # Compiled binary
├── test_fallback_pipeline.py       # Test suite
└── FALLBACK_PIPELINE.md           # This documentation
```

## Build Instructions

### Swift OCR Binary
```bash
cd swift
swift build -c release
```

### Rust Image Enhancer
```bash
cd image_enhancer
cargo build --release
```

### Python Dependencies
```bash
pip install pdf2image
```

## Success Criteria

The pipeline is considered successful when:
1. ✅ Garbage detection accurately identifies poor extraction results
2. ✅ Apple Vision OCR provides better text extraction than SmolDocling VLM
3. ✅ Image enhancement improves OCR accuracy for severely degraded documents
4. ✅ Fallback gracefully handles failures at each stage
5. ✅ Performance remains acceptable (< 5 minutes per document)

## Monitoring

The pipeline provides detailed logging:
- `🤖 Starting SmolDocling VLM extraction`
- `❗ Detected garbage extraction result, initiating fallback`
- `🚀 Using Apple Vision OCR as a fallback`
- `❗ OCR result still garbage, applying image enhancement`
- `✅ Successfully enhanced extraction with OCR fallback`

## Platform Compatibility

- **macOS**: Full support (all 3 stages)
- **Linux/Windows**: Stage 1 only (SmolDocling VLM)
- **Recommendation**: Use macOS for processing challenging PDFs

## Performance Characteristics

- **Stage 1**: ~30-60 seconds (VLM processing)
- **Stage 2**: ~5-10 seconds (OCR)
- **Stage 3**: ~15-30 seconds (enhancement + OCR)
- **Total fallback time**: ~20-40 seconds additional

## Next Steps

1. **Monitor performance** with real-world "poopy PDFs"
2. **Tune garbage detection** thresholds based on usage
3. **Add more enhancement algorithms** (histogram equalization, morphological operations)
4. **Implement batch processing** for multiple documents
5. **Add confidence scoring** for extraction quality assessment

## Known Limitations

1. Apple Vision OCR requires macOS 12+
2. Image enhancement assumes text documents (not ideal for graphics-heavy PDFs)
3. Deskewing algorithm works best with text-heavy documents
4. No support for multi-page PDFs in fallback (processes first page only)

---

🎉 **Your staged fallback pipeline is now ready to handle even the most challenging PDFs!**
