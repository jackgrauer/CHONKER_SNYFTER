#!/usr/bin/env python3
"""
Quick test script for the domain-agnostic extraction pipeline
"""

import sys
from pathlib import Path

# Add the python directory to the path
sys.path.insert(0, str(Path(__file__).parent / "python"))

from extraction_pipeline import ExtractionPipeline

def test_pipeline():
    """Test the pipeline with test.pdf"""
    
    # Check if test.pdf exists
    test_pdf = Path("test.pdf")
    if not test_pdf.exists():
        print("❌ test.pdf not found - please ensure it exists in the current directory")
        return False
    
    print("🧪 Testing domain-agnostic extraction pipeline...")
    
    # Create pipeline
    pipeline = ExtractionPipeline(output_dir="test_pipeline_outputs")
    
    try:
        # Run in debug mode for full traceability
        results = pipeline.debug_mode(str(test_pdf))
        
        # Print summary
        tables_count = len(results.get('structured_tables', []))
        print(f"\n✅ Pipeline test successful!")
        print(f"📊 Extracted {tables_count} tables")
        print(f"💾 Outputs saved to: test_pipeline_outputs/")
        print(f"📁 Check raw.json, processed.json, final.json for detailed analysis")
        
        return True
        
    except Exception as e:
        print(f"❌ Pipeline test failed: {e}")
        import traceback
        traceback.print_exc()
        return False

if __name__ == "__main__":
    success = test_pipeline()
    sys.exit(0 if success else 1)
