#!/usr/bin/env python3
"""
🎭 Character Consistency Checker
Ensures CHONKER and SNYFTER's personalities shine through the code
"""

import re
import sys
from pathlib import Path
from typing import List, Tuple


class CharacterChecker:
    def __init__(self):
        self.issues = []
        self.chonker_patterns = {
            "messages": [
                r"nom", r"munch", r"chomp", r"burp", 
                r"sniff", r"digest", r"tasty", r"yum"
            ],
            "errors": [
                r"tummy ache", r"indigestion", r"too scuzzy",
                r"can't digest", r"cough"
            ]
        }
        self.snyfter_patterns = {
            "messages": [
                r"catalog", r"archive", r"file", r"reference",
                r"adjusts glasses", r"precisely", r"meticulous"
            ],
            "errors": [
                r"misfiled", r"catalog error", r"reference not found",
                r"improper classification"
            ]
        }
    
    def check_file(self, filepath: Path) -> List[Tuple[int, str, str]]:
        """Check a single file for character consistency"""
        issues = []
        
        with open(filepath, 'r', encoding='utf-8') as f:
            lines = f.readlines()
        
        for i, line in enumerate(lines, 1):
            # Check for generic errors
            if re.search(r'Error:|Exception:|Failed:', line, re.IGNORECASE):
                if not ('🐹' in line or '🐁' in line):
                    issues.append((i, line.strip(), "Generic error without character"))
            
            # Check for print statements
            if 'print(' in line and not line.strip().startswith('#'):
                if not any(emoji in line for emoji in ['🐹', '🐁']):
                    issues.append((i, line.strip(), "Print without character context"))
            
            # Check CHONKER-specific code
            if 'chonker' in line.lower():
                if not any(re.search(pattern, line, re.IGNORECASE) 
                          for patterns in self.chonker_patterns.values() 
                          for pattern in patterns):
                    if 'class' not in line and 'def' not in line:
                        issues.append((i, line.strip(), "CHONKER code lacks personality"))
            
            # Check SNYFTER-specific code
            if 'snyfter' in line.lower():
                if not any(re.search(pattern, line, re.IGNORECASE) 
                          for patterns in self.snyfter_patterns.values() 
                          for pattern in patterns):
                    if 'class' not in line and 'def' not in line:
                        issues.append((i, line.strip(), "SNYFTER code lacks personality"))
        
        return issues
    
    def check_messages(self, filepath: Path) -> List[Tuple[int, str, str]]:
        """Check that messages match character personality"""
        issues = []
        
        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()
        
        # Find all string literals that look like messages
        message_pattern = r'["\']([^"\']*(?:🐹|🐁)[^"\']*)["\']'
        messages = re.findall(message_pattern, content)
        
        for msg in messages:
            if '🐹' in msg:
                # Check CHONKER messages
                if not any(re.search(pattern, msg, re.IGNORECASE) 
                          for pattern in self.chonker_patterns['messages']):
                    issues.append((0, msg, "CHONKER message lacks hamster personality"))
            
            if '🐁' in msg:
                # Check SNYFTER messages
                if not any(re.search(pattern, msg, re.IGNORECASE) 
                          for pattern in self.snyfter_patterns['messages']):
                    issues.append((0, msg, "SNYFTER message lacks librarian personality"))
        
        return issues
    
    def run_checks(self):
        """Run all character consistency checks"""
        print("🎭 Character Consistency Check\n")
        
        py_files = list(Path('.').glob('**/*.py'))
        py_files = [f for f in py_files if '__pycache__' not in str(f)]
        
        total_issues = 0
        
        for filepath in py_files:
            file_issues = self.check_file(filepath)
            msg_issues = self.check_messages(filepath)
            
            all_issues = file_issues + msg_issues
            
            if all_issues:
                print(f"\n❌ {filepath}")
                for line_num, line, issue in all_issues:
                    if line_num > 0:
                        print(f"  Line {line_num}: {issue}")
                        print(f"    > {line[:60]}...")
                    else:
                        print(f"  {issue}: {line[:60]}...")
                total_issues += len(all_issues)
        
        if total_issues == 0:
            print("✅ All files maintain character consistency!")
            print("🐹 CHONKER approves! *happy hamster noises*")
            print("🐁 SNYFTER approves! *adjusts glasses with satisfaction*")
        else:
            print(f"\n❌ Found {total_issues} character consistency issues")
            print("💡 Tips:")
            print("  - All errors should reference character-appropriate terms")
            print("  - CHONKER uses food/digestion metaphors")
            print("  - SNYFTER uses library/cataloging metaphors")
            return 1
        
        return 0


if __name__ == "__main__":
    checker = CharacterChecker()
    sys.exit(checker.run_checks())