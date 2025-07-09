#!/usr/bin/env python3

import sys
import os
sys.path.append('python')

from smoldocling_bridge import is_garbage_result, apply_apple_vision_ocr, enhance_image_with_rust
import platform

# Test text that should trigger fallback
test_text = """remote sensing
In this image, there is a document with a date and a signature.
<!-- image -->"""

print("🔍 Debugging fallback pipeline...")
print(f"Platform: {platform.system()}")
print(f"Test text: {repr(test_text)}")
print(f"Text length: {len(test_text.strip())}")

# Test garbage detection
print("\n🧪 Testing garbage detection...")
result = is_garbage_result(test_text)
print(f"is_garbage_result: {result}")

if result:
    print("✅ Garbage detection working - text identified as garbage")
    
    if platform.system() == 'Darwin':
        print("\n🍎 macOS detected - testing Apple Vision OCR...")
        # Test with a fake PDF path for now
        fake_pdf_path = "/Users/jack/CHONKER_SNYFTER/1.pdf"
        
        if os.path.exists(fake_pdf_path):
            print(f"📄 Testing with PDF: {fake_pdf_path}")
            try:
                ocr_result = apply_apple_vision_ocr(fake_pdf_path)
                print(f"🔍 OCR result: {repr(ocr_result)}")
                
                if is_garbage_result(ocr_result):
                    print("🔧 OCR still garbage, testing image enhancement...")
                    enhance_image_with_rust(fake_pdf_path)
                    enhanced_ocr = apply_apple_vision_ocr(fake_pdf_path)
                    print(f"🔍 Enhanced OCR result: {repr(enhanced_ocr)}")
                
            except Exception as e:
                print(f"❌ Error during OCR testing: {e}")
        else:
            print(f"❌ Test PDF not found at {fake_pdf_path}")
    else:
        print("⚠️  Not on macOS - fallback not available")
else:
    print("❌ Garbage detection failed - text not identified as garbage")

print("\n🏁 Debug complete")
