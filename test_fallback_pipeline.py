#!/usr/bin/env python3
"""
Test script for the staged fallback PDF extraction pipeline
"""

import sys
import os
import subprocess

def test_garbage_detection():
    """Test the garbage detection function"""
    sys.path.append('/Users/jack/CHONKER_SNYFTER/python')
    from smoldocling_bridge import is_garbage_result
    
    # Test cases
    test_cases = [
        ("remote sensing\nIn this image, there is a document with a date and a signature.", True),
        ("This is a proper document with actual content and meaningful text that is longer than 50 characters.", False),
        ("short", True),
        ("In this image there is a document with some text", True),
        ("Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.", False),
    ]
    
    print("🧪 Testing garbage detection...")
    for text, expected in test_cases:
        result = is_garbage_result(text)
        status = "✅" if result == expected else "❌"
        print(f"{status} Text: '{text[:50]}...' -> {result} (expected: {expected})")
    
    return True

def test_swift_ocr():
    """Test Swift OCR binary"""
    print("\n🧪 Testing Swift OCR binary...")
    swift_binary = '/Users/jack/CHONKER_SNYFTER/swift/.build/release/apple_vision_ocr'
    
    if not os.path.exists(swift_binary):
        print(f"❌ Swift binary not found at {swift_binary}")
        return False
    
    try:
        result = subprocess.run([swift_binary], capture_output=True, text=True, timeout=5)
        if "Usage:" in result.stdout:
            print("✅ Swift OCR binary is working")
            return True
        else:
            print(f"❌ Swift OCR binary error: {result.stderr}")
            return False
    except Exception as e:
        print(f"❌ Swift OCR binary test failed: {e}")
        return False

def test_rust_enhancer():
    """Test Rust image enhancer binary"""
    print("\n🧪 Testing Rust image enhancer binary...")
    rust_binary = '/Users/jack/CHONKER_SNYFTER/image_enhancer/target/release/image_enhancer'
    
    if not os.path.exists(rust_binary):
        print(f"❌ Rust binary not found at {rust_binary}")
        return False
    
    try:
        result = subprocess.run([rust_binary, '--help'], capture_output=True, text=True, timeout=5)
        if "Usage:" in result.stdout:
            print("✅ Rust image enhancer binary is working")
            return True
        else:
            print(f"❌ Rust image enhancer binary error: {result.stderr}")
            return False
    except Exception as e:
        print(f"❌ Rust image enhancer binary test failed: {e}")
        return False

def test_dependencies():
    """Test Python dependencies"""
    print("\n🧪 Testing Python dependencies...")
    
    dependencies = [
        ('platform', 'Platform detection'),
        ('subprocess', 'Process execution'),
        ('json', 'JSON handling'),
        ('pathlib', 'Path handling'),
    ]
    
    for dep, desc in dependencies:
        try:
            __import__(dep)
            print(f"✅ {desc} ({dep}) - OK")
        except ImportError:
            print(f"❌ {desc} ({dep}) - MISSING")
            return False
    
    # Test optional dependencies
    try:
        from pdf2image import convert_from_path
        print("✅ pdf2image - OK")
    except ImportError:
        print("⚠️  pdf2image - MISSING (will fallback to ImageMagick)")
    
    return True

def main():
    print("🚀 Testing CHONKER Staged Fallback Pipeline")
    print("=" * 50)
    
    tests = [
        test_dependencies,
        test_garbage_detection,
        test_swift_ocr,
        test_rust_enhancer,
    ]
    
    results = []
    for test in tests:
        try:
            results.append(test())
        except Exception as e:
            print(f"❌ Test failed with exception: {e}")
            results.append(False)
    
    print("\n" + "=" * 50)
    print("📊 Test Results:")
    print(f"✅ Passed: {sum(results)}")
    print(f"❌ Failed: {len(results) - sum(results)}")
    
    if all(results):
        print("\n🎉 All tests passed! Your staged fallback pipeline is ready!")
        print("\nNext steps:")
        print("1. Install pdf2image: pip install pdf2image")
        print("2. Test with a real PDF: python python/smoldocling_bridge.py your_pdf.pdf")
        print("3. Watch the magic happen with garbage detection and OCR fallback!")
    else:
        print("\n⚠️  Some tests failed. Please check the errors above.")
    
    return all(results)

if __name__ == "__main__":
    success = main()
    sys.exit(0 if success else 1)
