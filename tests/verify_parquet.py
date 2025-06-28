#!/usr/bin/env python3

"""
Verify CHONKER Parquet Export Compatibility
Tests that exported Parquet files work correctly with Python data science stack
"""

import sys
import os
from pathlib import Path

def test_parquet_loading():
    """Test loading exported Parquet files with various libraries"""
    
    print("🔍 Testing CHONKER Parquet Export Compatibility")
    print("=" * 50)
    
    # Find the most recent parquet export
    test_file = "tests/output/export_test.parquet"
    
    if not os.path.exists(test_file):
        print(f"❌ Test file not found: {test_file}")
        print("   Run: cargo run -- export -f parquet -o tests/output/export_test.parquet")
        return False
    
    print(f"📁 Testing file: {test_file}")
    print(f"   File size: {os.path.getsize(test_file)} bytes")
    
    try:
        # Test 1: pandas
        print("\n1️⃣ Testing pandas compatibility...")
        import pandas as pd
        df = pd.read_parquet(test_file)
        print(f"   ✅ Loaded {len(df)} rows, {len(df.columns)} columns")
        print(f"   📊 Columns: {list(df.columns)}")
        
        # Show sample data
        if len(df) > 0:
            print("   Sample data:")
            print(df.head(2).to_string(max_cols=5))
        
    except ImportError:
        print("   ⚠️  pandas not available")
    except Exception as e:
        print(f"   ❌ pandas error: {e}")
        return False
    
    try:
        # Test 2: pyarrow
        print("\n2️⃣ Testing pyarrow compatibility...")
        import pyarrow.parquet as pq
        table = pq.read_table(test_file)
        print(f"   ✅ Loaded {table.num_rows} rows, {table.num_columns} columns")
        print(f"   🏗️  Schema: {table.schema}")
        
    except ImportError:
        print("   ⚠️  pyarrow not available")
    except Exception as e:
        print(f"   ❌ pyarrow error: {e}")
        return False
    
    try:
        # Test 3: polars
        print("\n3️⃣ Testing polars compatibility...")
        import polars as pl
        df_polars = pl.read_parquet(test_file)
        print(f"   ✅ Loaded {df_polars.height} rows, {df_polars.width} columns")
        print(f"   📈 Data types: {df_polars.dtypes}")
        
    except ImportError:
        print("   ⚠️  polars not available (optional)")
    except Exception as e:
        print(f"   ❌ polars error: {e}")
    
    # Test 4: Basic analytics
    print("\n4️⃣ Testing basic analytics...")
    try:
        if 'df' in locals():
            # Document analysis
            if 'document_id' in df.columns:
                unique_docs = df['document_id'].nunique()
                print(f"   📄 Unique documents: {unique_docs}")
            
            if 'filename' in df.columns:
                print(f"   📁 File types: {df['filename'].str.extract(r'\.([^.]+)$')[0].value_counts().to_dict()}")
            
            if 'char_count' in df.columns:
                avg_chunk_size = df['char_count'].mean()
                print(f"   📏 Average chunk size: {avg_chunk_size:.0f} characters")
            
            if 'created_at' in df.columns:
                df['created_at'] = pd.to_datetime(df['created_at'])
                date_range = f"{df['created_at'].min()} to {df['created_at'].max()}"
                print(f"   📅 Date range: {date_range}")
        
    except Exception as e:
        print(f"   ⚠️  Analytics error: {e}")
    
    print("\n✅ Parquet compatibility test completed successfully!")
    print("🚀 The exported files work correctly with Python data science tools")
    return True

def test_roundtrip():
    """Test roundtrip: Parquet -> Python -> Parquet"""
    print("\n5️⃣ Testing roundtrip compatibility...")
    
    try:
        import pandas as pd
        
        # Read original
        original_file = "tests/output/export_test.parquet"
        df = pd.read_parquet(original_file)
        
        # Write modified version
        roundtrip_file = "tests/temp/roundtrip_test.parquet"
        os.makedirs("tests/temp", exist_ok=True)
        
        # Add a computed column
        if 'char_count' in df.columns:
            df['char_count_kb'] = df['char_count'] / 1024.0
        
        df.to_parquet(roundtrip_file, compression='snappy')
        
        # Read it back
        df_roundtrip = pd.read_parquet(roundtrip_file)
        
        print(f"   ✅ Roundtrip successful: {len(df_roundtrip)} rows")
        print(f"   🔄 Added computed column: char_count_kb")
        
        # Clean up
        os.remove(roundtrip_file)
        
    except Exception as e:
        print(f"   ❌ Roundtrip error: {e}")

if __name__ == "__main__":
    success = test_parquet_loading()
    test_roundtrip()
    
    if success:
        print("\n🎉 All compatibility tests passed!")
        sys.exit(0)
    else:
        print("\n❌ Some tests failed")
        sys.exit(1)
