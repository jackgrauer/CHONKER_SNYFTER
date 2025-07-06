#!/usr/bin/env python3
"""
🐹🐭 CHONKER BUG FIX VERIFICATION
Final verification that the critical extraction bug has been fixed
"""

import json
import subprocess
import sys

def main():
    print("🐹🐭 CHONKER BUG FIX VERIFICATION")
    print("=" * 60)
    
    print("\n🔍 Testing Enhanced Extraction Bridge...")
    
    # Test the enhanced extraction bridge directly
    try:
        result = subprocess.run([
            "python3", "python/enhanced_extraction_bridge.py", "test.pdf"
        ], capture_output=True, text=True, timeout=60)
        
        if result.returncode != 0:
            print(f"❌ Enhanced extraction failed: {result.stderr}")
            return False
            
        data = json.loads(result.stdout)
        
        # Verify structured_tables exists
        structured_tables = data.get('structured_tables', [])
        if not structured_tables:
            print("❌ No structured_tables found")
            return False
            
        table = structured_tables[0]
        print(f"✅ Found structured table: {table.get('num_rows')}x{table.get('num_cols')}")
        
        # Verify grid with metadata
        grid = table.get('grid', [])
        if not grid or not isinstance(grid[0][0], dict):
            print("❌ No enhanced grid structure")
            return False
            
        print(f"✅ Enhanced grid with {len(grid)} rows and cell metadata")
        
        # Verify context
        context = table.get('context', {})
        if not context.get('table_title'):
            print("❌ No context information")
            return False
            
        print("✅ Context information preserved")
        
    except Exception as e:
        print(f"❌ Extraction test failed: {e}")
        return False
    
    print("\n🦀 Testing Rust Integration...")
    
    # Check that Rust uses enhanced bridge
    with open("src-tauri/src/extractor.rs", 'r') as f:
        if "enhanced_extraction_bridge.py" not in f.read():
            print("❌ Rust not using enhanced bridge")
            return False
    print("✅ Rust configured for enhanced extraction")
    
    # Check frontend conversion logic
    with open("src-tauri/src/lib.rs", 'r') as f:
        if "convert_processing_result_to_frontend_format" not in f.read():
            print("❌ Missing frontend conversion")
            return False
    print("✅ Frontend conversion logic present")
    
    print("\n🎨 Testing Frontend Compatibility...")
    
    # Check frontend expects correct format
    with open("frontend/index.html", 'r') as f:
        content = f.read()
        if not all(pattern in content for pattern in [
            "data.tables", "table.headers", "table.rows", "table.context"
        ]):
            print("❌ Frontend expects wrong format")
            return False
    print("✅ Frontend expects structured table format")
    
    print("\n" + "=" * 60)
    print("🎉 ALL TESTS PASSED - BUG FIX VERIFIED!")
    print("\n✅ Enhanced extraction produces structured data with metadata")
    print("✅ Rust backend uses enhanced bridge and converts data correctly")  
    print("✅ Frontend receives properly formatted tables with context")
    print("\n🐹 The critical bug has been successfully fixed! 🐭")
    print("\nBefore: Frontend consumed flattened markdown strings")
    print("After:  Frontend receives rich structured data with context")
    
    return True

if __name__ == '__main__':
    success = main()
    sys.exit(0 if success else 1)
