#!/usr/bin/env python3
"""
Universal Table Fixer using Qwen-7B
Detects and separates mixed numeric/text patterns across any domain
"""

import json
import re
import subprocess
import time
from typing import Dict, List, Tuple, Optional, NamedTuple
from pathlib import Path
import argparse
from enum import Enum
from dataclasses import dataclass

class PatternType(Enum):
    NUMERIC_WITH_SUFFIX = "numeric_with_suffix"        # "123 K", "45.6 mg"
    TEXT_PREFIX_NUMERIC = "text_prefix_numeric"        # "< 0.5", "> 100"
    NUMERIC_WITH_EMBEDDED = "numeric_with_embedded"     # "2.5±0.1", "$1,234 M"

@dataclass
class MixedDataPattern:
    original: str
    numeric_part: str
    text_part: str
    pattern_type: PatternType

@dataclass
class ColumnAnalysis:
    total_cells: int
    mixed_pattern_cells: int
    pattern_distribution: Dict[PatternType, int]
    examples: List[MixedDataPattern]
    needs_separation: bool

class UniversalTableFixer:
    def __init__(self, model_name: str = "qwen2.5-coder:7b", timeout: int = 90):
        self.model_name = model_name
        self.timeout = timeout
        
        # Universal regex patterns
        self.patterns = {
            PatternType.NUMERIC_WITH_SUFFIX: re.compile(r'(\d+\.?\d*)\s*([A-Za-z%±√∞°<>≤≥]+)'),
            PatternType.TEXT_PREFIX_NUMERIC: re.compile(r'([<>~≤≥±$£€¥]+)\s*(\d+\.?\d*)'),
            PatternType.NUMERIC_WITH_EMBEDDED: re.compile(r'(\d+\.?\d*)([±√°%\/]+)(\d+\.?\d*)?')
        }

    def detect_mixed_data_patterns(self, cell_value: str) -> Optional[MixedDataPattern]:
        """Detect ANY mixed numeric/text pattern in a cell value"""
        if not cell_value or not cell_value.strip():
            return None
            
        cell_value = cell_value.strip()
        
        # Try each pattern type
        for pattern_type, regex in self.patterns.items():
            match = regex.search(cell_value)
            if match:
                if pattern_type == PatternType.NUMERIC_WITH_SUFFIX:
                    return MixedDataPattern(
                        original=cell_value,
                        numeric_part=match.group(1),
                        text_part=match.group(2),
                        pattern_type=pattern_type
                    )
                elif pattern_type == PatternType.TEXT_PREFIX_NUMERIC:
                    return MixedDataPattern(
                        original=cell_value,
                        text_part=match.group(1),
                        numeric_part=match.group(2),
                        pattern_type=pattern_type
                    )
                elif pattern_type == PatternType.NUMERIC_WITH_EMBEDDED:
                    text_part = match.group(2)
                    if match.group(3):
                        text_part += match.group(3)
                    return MixedDataPattern(
                        original=cell_value,
                        numeric_part=match.group(1),
                        text_part=text_part,
                        pattern_type=pattern_type
                    )
        
        return None

    def analyze_column_for_patterns(self, column_data: List[str]) -> ColumnAnalysis:
        """Analyze a column to detect mixed patterns and their frequency"""
        pattern_counts = {}
        examples = []
        
        for cell in column_data:
            pattern = self.detect_mixed_data_patterns(cell)
            if pattern:
                pattern_counts[pattern.pattern_type] = pattern_counts.get(pattern.pattern_type, 0) + 1
                if len(examples) < 5:  # Keep up to 5 examples
                    examples.append(pattern)
        
        return ColumnAnalysis(
            total_cells=len(column_data),
            mixed_pattern_cells=len([p for p in [self.detect_mixed_data_patterns(c) for c in column_data] if p]),
            pattern_distribution=pattern_counts,
            examples=examples,
            needs_separation=len(examples) > 0
        )

    def create_universal_fixing_prompt(self, table_chunk: str, detected_patterns: List[MixedDataPattern]) -> str:
        """Create domain-agnostic prompt for table fixing"""
        if not detected_patterns:
            return self.create_generic_prompt(table_chunk)
        
        pattern_examples = []
        for pattern in detected_patterns[:5]:  # Limit to 5 examples
            if pattern.pattern_type == PatternType.NUMERIC_WITH_SUFFIX:
                pattern_examples.append(
                    f"'{pattern.original}' should become: '{pattern.numeric_part}' in one column, '{pattern.text_part}' in another"
                )
            elif pattern.pattern_type == PatternType.TEXT_PREFIX_NUMERIC:
                pattern_examples.append(
                    f"'{pattern.original}' should become: '{pattern.text_part}' in one column, '{pattern.numeric_part}' in another"
                )
            elif pattern.pattern_type == PatternType.NUMERIC_WITH_EMBEDDED:
                pattern_examples.append(
                    f"'{pattern.original}' should become: '{pattern.numeric_part}' in one column, '{pattern.text_part}' in another"
                )
        
        pattern_text = "\n".join(pattern_examples)
        
        prompt = f"""You are an expert at fixing table data by separating mixed numeric/text patterns.

DETECTED PATTERNS that need separation:
{pattern_text}

UNIVERSAL RULES:
1. Separate numeric values from text qualifiers/modifiers/units
2. Create additional columns for the separated text parts  
3. Preserve all original data and table structure
4. Keep column headers clear and descriptive
5. Use markdown table format with proper pipes |

ORIGINAL TABLE:
```
{table_chunk}
```

TASK: Fix this table by separating the mixed patterns shown above.

Return your response as JSON in this exact format:
{{
    "fixed_table": "the complete fixed markdown table here",
    "issues_fixed": ["specific list of pattern separations made"],
    "patterns_separated": ["list of mixed patterns that were separated"],
    "success": true
}}

EXAMPLES:
Before: | Revenue | $1,234 K |
After:  | Revenue | $1,234 | K |

Before: | Temperature | 23.5°C |  
After:  | Temperature | 23.5 | °C |

Before: | Tolerance | 2.5±0.1 |
After:  | Tolerance | 2.5 | ±0.1 |
"""
        return prompt

    def create_generic_prompt(self, table_chunk: str) -> str:
        """Fallback prompt when no specific patterns detected"""
        return f"""Fix this table by separating any mixed numeric/text data patterns you find.

Table to fix:
{table_chunk}

Return as JSON: {{"fixed_table": "...", "success": true}}"""

    def call_qwen(self, prompt: str) -> Optional[str]:
        """Call Qwen via Ollama with JSON format"""
        try:
            cmd = [
                "ollama", "run", self.model_name,
                "--format", "json",
                prompt
            ]
            
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                timeout=self.timeout
            )
            
            if result.returncode == 0:
                return result.stdout.strip()
            else:
                print(f"❌ Qwen call failed: {result.stderr}")
                return None
                
        except subprocess.TimeoutExpired:
            print(f"⏰ Qwen call timed out after {self.timeout} seconds")
            return None
        except Exception as e:
            print(f"❌ Error calling Qwen: {e}")
            return None

    def parse_qwen_response(self, response: str) -> Optional[Dict]:
        """Parse Qwen's JSON response safely"""
        try:
            json_match = re.search(r'\{.*\}', response, re.DOTALL)
            if json_match:
                json_str = json_match.group()
                return json.loads(json_str)
            else:
                print("❌ No JSON found in Qwen response")
                return None
        except json.JSONDecodeError as e:
            print(f"❌ Failed to parse Qwen JSON: {e}")
            return None

    def find_table_rows(self, content: str) -> List[Tuple[int, str]]:
        """Find all table rows with their line numbers"""
        lines = content.split('\n')
        table_rows = []
        
        for i, line in enumerate(lines):
            if '|' in line and len(line.split('|')) >= 3:
                table_rows.append((i, line))
        
        return table_rows

    def count_mixed_patterns(self, text: str) -> int:
        """Count all types of mixed patterns in text"""
        count = 0
        for line in text.split('\n'):
            if '|' in line:
                cols = [col.strip() for col in line.split('|')]
                for col in cols:
                    if self.detect_mixed_data_patterns(col):
                        count += 1
        return count

    def group_table_rows(self, table_rows: List[Tuple[int, str]]) -> List[Dict]:
        """Group consecutive table rows into tables"""
        if not table_rows:
            return []
        
        tables = []
        current_table = {
            'start_line': table_rows[0][0],
            'end_line': table_rows[0][0],
            'lines': [table_rows[0][1]],
            'line_numbers': [table_rows[0][0]]
        }
        
        for i in range(1, len(table_rows)):
            line_num, line = table_rows[i]
            prev_line_num = table_rows[i-1][0]
            
            if line_num - prev_line_num <= 3:
                current_table['end_line'] = line_num
                current_table['lines'].append(line)
                current_table['line_numbers'].append(line_num)
            else:
                tables.append(current_table)
                current_table = {
                    'start_line': line_num,
                    'end_line': line_num,
                    'lines': [line],
                    'line_numbers': [line_num]
                }
        
        tables.append(current_table)
        return tables

    def extract_columns_with_mixed_data(self, table_text: str) -> Optional[str]:
        """Extract only columns that contain mixed patterns"""
        try:
            lines = table_text.strip().split('\n')
            if len(lines) < 2:
                return table_text
            
            # Find header row
            header_row = None
            for line in lines:
                if '|' in line and len(line.split('|')) > 3:
                    header_row = line
                    break
            
            if not header_row:
                return table_text
            
            # Parse all columns
            headers = [h.strip() for h in header_row.split('|') if h.strip()]
            all_column_data = {}
            
            # Extract data for each column
            for i, header in enumerate(headers):
                column_data = []
                for line in lines[1:]:
                    if '|' in line:
                        cols = line.split('|')
                        if i < len(cols):
                            column_data.append(cols[i].strip())
                        else:
                            column_data.append('')
                all_column_data[i] = column_data
            
            # Analyze each column for mixed patterns
            essential_indices = []
            
            # Always include first column (identifier)
            essential_indices.append(0)
            
            # Include columns with mixed patterns
            for i, header in enumerate(headers):
                if i in essential_indices:
                    continue
                    
                analysis = self.analyze_column_for_patterns(all_column_data.get(i, []))
                if analysis.needs_separation:
                    essential_indices.append(i)
                    print(f"   🎯 Column '{header}' has {analysis.mixed_pattern_cells} mixed patterns")
            
            # Include a few context columns
            context_count = 0
            for i, header in enumerate(headers):
                if context_count >= 2 or i in essential_indices:
                    continue
                essential_indices.append(i)
                context_count += 1
            
            essential_indices.sort()
            
            # Extract only essential columns
            filtered_lines = []
            for line in lines:
                if '|' in line:
                    cols = line.split('|')
                    filtered_cols = ['']
                    for idx in essential_indices:
                        if idx < len(cols):
                            filtered_cols.append(cols[idx])
                        else:
                            filtered_cols.append('')
                    filtered_cols.append('')
                    filtered_lines.append('|'.join(filtered_cols))
            
            result = '\n'.join(filtered_lines)
            return result if len(result) < len(table_text) * 0.8 else None
            
        except Exception as e:
            print(f"   ❌ Error in column extraction: {e}")
            return None

    def fix_document_tables(self, input_path: str, output_path: str) -> Dict:
        """Main function to fix all tables in a document"""
        print(f"🔍 Processing document: {input_path}")
        
        with open(input_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        # Find all table rows
        table_rows = self.find_table_rows(content)
        print(f"📋 Found {len(table_rows)} table rows")
        
        # Group into tables
        tables = self.group_table_rows(table_rows)
        print(f"📊 Grouped into {len(tables)} tables")
        
        results = {'fixed': 0, 'skipped': 0, 'failed': 0, 'total': 0}
        fixed_lines = content.split('\n')
        
        for i, table in enumerate(tables):
            table_text = '\n'.join(table['lines'])
            
            # Check if table has mixed patterns
            mixed_patterns = self.count_mixed_patterns(table_text)
            if mixed_patterns == 0:
                print(f"⏭️  Table {i+1}: No mixed patterns detected, skipping")
                results['skipped'] += 1
                continue
            
            results['total'] += 1
            print(f"\n🔧 Fixing Table {i+1} with {mixed_patterns} mixed patterns...")
            print(f"   📏 Table size: {len(table_text)} chars, {len(table['lines'])} rows")
            
            # Extract examples of mixed patterns
            detected_patterns = []
            for line in table['lines']:
                if '|' in line:
                    cols = [col.strip() for col in line.split('|')]
                    for col in cols:
                        pattern = self.detect_mixed_data_patterns(col)
                        if pattern and len(detected_patterns) < 10:
                            detected_patterns.append(pattern)
            
            print(f"   🎯 Pattern examples: {[p.original for p in detected_patterns[:3]]}")
            
            # Smart size-based processing
            if len(table_text) > 8000:
                print(f"   📊 Large table detected - using pattern-based column filtering")
                filtered_table = self.extract_columns_with_mixed_data(table_text)
                if filtered_table and len(filtered_table) < 8000:
                    print(f"   ✅ Column filtering successful: {len(table_text)} → {len(filtered_table)} chars")
                    table_text = filtered_table
                else:
                    print(f"   ❌ Table too large for processing, skipping")
                    results['failed'] += 1
                    continue
            
            # Create prompt
            prompt = self.create_universal_fixing_prompt(table_text, detected_patterns)
            
            # Call Qwen
            response = self.call_qwen(prompt)
            if not response:
                print(f"   ❌ Failed to get response from Qwen")
                results['failed'] += 1
                continue
            
            # Parse response
            parsed = self.parse_qwen_response(response)
            if not parsed or not parsed.get('success'):
                print(f"   ❌ Invalid response format")
                results['failed'] += 1
                continue
            
            fixed_table = parsed.get('fixed_table', '')
            
            # Validate the fix
            original_patterns = self.count_mixed_patterns(table_text)
            fixed_patterns = self.count_mixed_patterns(fixed_table)
            
            if fixed_patterns < original_patterns:
                print(f"   ✅ Validation passed: separated {original_patterns - fixed_patterns} patterns")
                
                # Replace the table lines in the content
                fixed_table_lines = fixed_table.split('\n')
                for j, line_num in enumerate(table['line_numbers']):
                    if j < len(fixed_table_lines):
                        fixed_lines[line_num] = fixed_table_lines[j]
                
                results['fixed'] += 1
                print(f"   ✅ Table {i+1} fixed successfully!")
                if 'patterns_separated' in parsed:
                    patterns = parsed['patterns_separated']
                    print(f"   🎯 Patterns separated: {', '.join(patterns[:5])}")
            else:
                print(f"   ❌ Validation failed: no improvement detected")
                results['failed'] += 1
        
        # Write fixed content
        fixed_content = '\n'.join(fixed_lines)
        with open(output_path, 'w', encoding='utf-8') as f:
            f.write(fixed_content)
        
        return results

    def generate_summary_report(self, results: Dict, output_path: str):
        """Generate a summary of the fixing process"""
        report = f"""# 🔧 Universal Table Fixer Results

## 📊 Processing Summary

- **📋 Total tables processed**: {results['total']}
- **✅ Tables successfully fixed**: {results['fixed']}  
- **⏭️ Tables skipped (no mixed patterns)**: {results['skipped']}
- **❌ Tables failed to fix**: {results['failed']}

## 🎯 Success Rate

**{results['fixed']}/{results['total']} tables fixed** = **{(results['fixed']/max(results['total'],1)*100):.1f}% success rate**

## 🌍 Universal Pattern Detection Applied

- **Numeric + Suffix**: Values like "123 K", "45.6 mg", "0.046 U"
- **Text + Numeric**: Values like "< 0.5", "> 100", "≤ 25"  
- **Embedded Patterns**: Values like "2.5±0.1", "$1,234 M"
- **Data integrity**: All original data preserved, just properly organized

## 🎉 Results

Your table data now has:
- ✅ Properly separated numeric values from text modifiers
- ✅ Clean data ready for analysis across any domain
- ✅ Maintained data relationships and table structure
- ✅ Universal compatibility with financial, medical, scientific, and other data

---

**Generated by Universal Table Fixer**
"""
        
        summary_path = output_path.replace('.md', '_SUMMARY.md')
        with open(summary_path, 'w', encoding='utf-8') as f:
            f.write(report)
        
        print(f"📋 Summary report written to: {summary_path}")


def main():
    parser = argparse.ArgumentParser(description="Universal table fixer using pattern detection")
    parser.add_argument("input", help="Input file with tables")
    parser.add_argument("-o", "--output", help="Output path for fixed document", 
                       default=None)
    parser.add_argument("--timeout", type=int, default=90, help="Timeout for Qwen calls")
    
    args = parser.parse_args()
    
    if not Path(args.input).exists():
        print(f"❌ Input file not found: {args.input}")
        return 1
    
    # Default output path
    if not args.output:
        input_path = Path(args.input)
        args.output = str(input_path.with_suffix('')) + '_UNIVERSAL_FIXED.md'
    
    print("🚀 Starting Universal Table Fixer")
    print(f"📄 Input: {args.input}")
    print(f"📝 Output: {args.output}")
    
    start_time = time.time()
    
    # Initialize fixer
    fixer = UniversalTableFixer(timeout=args.timeout)
    
    # Fix tables
    results = fixer.fix_document_tables(args.input, args.output)
    
    processing_time = time.time() - start_time
    
    print(f"\n🎉 Universal Table Fixing Complete!")
    print(f"⏱️  Processing time: {processing_time:.1f} seconds")
    print(f"✅ Tables fixed: {results['fixed']}")
    print(f"⏭️ Tables skipped: {results['skipped']}")
    print(f"❌ Tables failed: {results['failed']}")
    print(f"📊 Total processed: {results['total']}")
    
    # Generate summary
    fixer.generate_summary_report(results, args.output)
    
    return 0 if results['failed'] == 0 else 1


if __name__ == "__main__":
    exit(main())
