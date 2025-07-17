#!/usr/bin/env python3
"""Script to identify and report unused code sections"""

import re
import ast

def find_unused_methods(filepath):
    """Find methods that are likely unused"""
    with open(filepath, 'r') as f:
        content = f.read()
    
    # Find all method definitions
    method_pattern = r'def\s+(\w+)\s*\('
    methods = re.findall(method_pattern, content)
    
    # Find method calls
    call_pattern = r'(?:self\.)(\w+)\s*\('
    calls = re.findall(call_pattern, content)
    
    # Methods that are never called
    unused = []
    for method in methods:
        if method not in ['__init__', '__str__', '__repr__']:  # Skip special methods
            # Check if method is called
            if method not in calls and f'.{method}' not in content:
                # Check if it's connected to signals
                if f'.connect(self.{method})' not in content and f'.triggered.connect(self.{method})' not in content:
                    unused.append(method)
    
    return unused

if __name__ == "__main__":
    unused = find_unused_methods('chonker_snyfter_enhanced.py')
    print(f"Found {len(unused)} potentially unused methods:")
    for method in unused:
        print(f"  - {method}")