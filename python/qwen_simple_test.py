#!/usr/bin/env python3
"""
Simple test of Qwen-7B table fixing with a small table
"""

import subprocess
import json

# Test table with misplaced qualifiers
test_table = """| Analyte | Concentration | Reporting Limit |
|---------|---------------|----------------|
| Benzene | 0.046 U       | 0.58           |
| Toluene | 0.17 J        | 0.43           |
| Ethylbenzene | 0.000851 J | 0.0079        |
"""

def call_qwen(prompt):
    """Call Qwen via Ollama"""
    try:
        cmd = ["ollama", "run", "qwen2.5-coder:7b", "--format", "json", prompt]
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=60)
        if result.returncode == 0:
            return result.stdout.strip()
        else:
            print(f"❌ Error: {result.stderr}")
            return None
    except Exception as e:
        print(f"❌ Exception: {e}")
        return None

# Prompt for Qwen
prompt = f"""You are an expert at fixing environmental laboratory data tables. 

ENVIRONMENTAL LAB CONVENTIONS:
- U = Undetected (below detection limit) - goes in separate QUALIFIER column  
- J = Estimated value (detected but below reporting limit) - goes in separate QUALIFIER column
- Values like "0.046 U" should be split into "0.046" (concentration) and "U" (qualifier)

ORIGINAL TABLE:
```markdown
{test_table}
```

TASK: Fix this table by:
1. Split combined values like "0.046 U" into separate columns
2. Add a Qualifier column between Concentration and Reporting Limit
3. Keep the same number of rows
4. Maintain markdown table format

Return JSON in this format:
{{
    "fixed_table": "the complete fixed markdown table",
    "changes_made": ["list of changes"],
    "validation": "explanation of fixes"
}}

EXAMPLE:
Before: | Benzene | 0.046 U | 0.58 |
After:  | Benzene | 0.046 | U | 0.58 |
"""

print("🚀 Testing Qwen-7B table fixing...")
print("📋 Original table:")
print(test_table)

response = call_qwen(prompt)
if response:
    try:
        # Parse response
        result = json.loads(response)
        
        print("\n✅ Fixed table:")
        print(result.get('fixed_table', 'No fixed table'))
        
        print(f"\n🔧 Changes made:")
        for change in result.get('changes_made', []):
            print(f"  - {change}")
            
        print(f"\n✔️ Validation: {result.get('validation', 'No validation')}")
        
    except json.JSONDecodeError as e:
        print(f"❌ Failed to parse JSON: {e}")
        print(f"Raw response: {response}")
else:
    print("❌ No response from Qwen")
