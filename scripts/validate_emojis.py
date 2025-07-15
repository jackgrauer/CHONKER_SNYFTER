#!/usr/bin/env python3
"""
🐹🐁 Emoji Validator - Ensures we use the EXACT right emojis
"""

import re
import sys
from pathlib import Path
from typing import List, Tuple


# THE ONLY CORRECT EMOJIS
CHONKER_EMOJI = "🐹"  # U+1F439 - HAMSTER FACE
SNYFTER_EMOJI = "🐁"  # U+1F401 - MOUSE (side view)

# WRONG EMOJIS TO CATCH
WRONG_MOUSE_EMOJI = "🐭"  # U+1F42D - MOUSE FACE (front view) - NOT SNYFTER!
WRONG_EMOJIS = {
    "🐭": "Mouse face - NOT SNYFTER! Use 🐁 instead",
    "🐀": "Rat - NOT SNYFTER! Use 🐁 instead",
    "🦫": "Beaver - What are you thinking?! Use 🐹 for CHONKER"
}


class EmojiValidator:
    def __init__(self):
        self.errors = []
        self.files_checked = 0
        
    def check_file(self, filepath: Path) -> List[Tuple[int, str, str]]:
        """Check a file for emoji correctness"""
        errors = []
        
        try:
            with open(filepath, 'r', encoding='utf-8') as f:
                lines = f.readlines()
            
            for i, line in enumerate(lines, 1):
                # Check for wrong emojis
                for wrong_emoji, message in WRONG_EMOJIS.items():
                    if wrong_emoji in line:
                        errors.append((i, line.strip(), message))
                
                # Check character references have correct emojis
                if 'chonker' in line.lower() and CHONKER_EMOJI not in line:
                    # Allow if it's just a variable/function name
                    if any(pattern in line for pattern in ['def ', 'class ', '=', '(', '.']):
                        continue
                    if '"' in line or "'" in line:  # It's a string
                        errors.append((i, line.strip(), f"CHONKER reference missing {CHONKER_EMOJI}"))
                
                if 'snyfter' in line.lower() and SNYFTER_EMOJI not in line:
                    # Allow if it's just a variable/function name
                    if any(pattern in line for pattern in ['def ', 'class ', '=', '(', '.']):
                        continue
                    if '"' in line or "'" in line:  # It's a string
                        errors.append((i, line.strip(), f"SNYFTER reference missing {SNYFTER_EMOJI}"))
            
        except Exception as e:
            errors.append((0, str(filepath), f"Error reading file: {e}"))
        
        return errors
    
    def validate_project(self) -> bool:
        """Validate all Python files in the project"""
        print("🐹🐁 Emoji Validation Check\n")
        print(f"Correct emojis:")
        print(f"  CHONKER: {CHONKER_EMOJI} (U+1F439)")
        print(f"  SNYFTER: {SNYFTER_EMOJI} (U+1F401)")
        print(f"\nChecking files...\n")
        
        # Find all Python files
        py_files = list(Path('.').glob('**/*.py'))
        py_files = [f for f in py_files if '__pycache__' not in str(f)]
        
        # Also check markdown files
        md_files = list(Path('.').glob('**/*.md'))
        all_files = py_files + md_files
        
        total_errors = 0
        
        for filepath in all_files:
            self.files_checked += 1
            errors = self.check_file(filepath)
            
            if errors:
                print(f"❌ {filepath}")
                for line_num, line, error in errors:
                    if line_num > 0:
                        print(f"  Line {line_num}: {error}")
                        print(f"    > {line[:60]}...")
                    else:
                        print(f"  {error}")
                total_errors += len(errors)
        
        print(f"\n{'='*50}")
        print(f"Files checked: {self.files_checked}")
        
        if total_errors == 0:
            print("\n✅ All emojis are correct!")
            print(f"{CHONKER_EMOJI} CHONKER is happy! *cheerful hamster noises*")
            print(f"{SNYFTER_EMOJI} SNYFTER approves! *adjusts glasses with precision*")
            return True
        else:
            print(f"\n❌ Found {total_errors} emoji errors!")
            print("\n⚠️  CRITICAL: Wrong emojis detected!")
            print(f"Remember: SNYFTER is {SNYFTER_EMOJI} NOT {WRONG_MOUSE_EMOJI}")
            return False


def test_emoji_rendering():
    """Test that emojis render correctly"""
    print("\n📋 Emoji Rendering Test:")
    print(f"CHONKER should be a hamster face: {CHONKER_EMOJI}")
    print(f"SNYFTER should be a side-view mouse: {SNYFTER_EMOJI}")
    print(f"This is the WRONG mouse (face view): {WRONG_MOUSE_EMOJI}")
    print("\nIf SNYFTER looks like a mouse FACE rather than SIDE VIEW, something's wrong!")


if __name__ == "__main__":
    validator = EmojiValidator()
    
    # Run validation
    success = validator.validate_project()
    
    # Show rendering test
    test_emoji_rendering()
    
    # Exit with error code if validation failed
    sys.exit(0 if success else 1)