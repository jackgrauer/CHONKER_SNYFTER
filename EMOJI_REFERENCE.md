# 🐹🐁 Official CHONKER & SNYFTER Emoji Reference

## Critical Emojis (MUST BE EXACT)

### CHONKER - The Chubby Hamster
- **Emoji**: 🐹
- **Unicode**: U+1F439
- **Name**: HAMSTER FACE
- **Description**: 2016 Android Noto emoji - chubby hamster face
- **Character**: Enthusiastic PDF muncher who makes happy hamster noises

### SNYFTER - The Skinny Mouse  
- **Emoji**: 🐁
- **Unicode**: U+1F401
- **Name**: MOUSE
- **Description**: Side-view mouse, appears skinny/hypothyroidal
- **Character**: Meticulous librarian with tiny glasses

## DO NOT USE THESE (Wrong Emojis)
- ❌ 🐭 (U+1F42D) - MOUSE FACE - This is the wrong mouse!
- ❌ 🐀 (U+1F400) - RAT - Definitely not SNYFTER!
- ❌ 🦫 (U+1F9AB) - BEAVER - Not even close!

## Supporting Emojis (OK to use)
- 📚 - For SNYFTER's library/archive
- 💾 - For saving/database operations
- 📂 - For file operations
- 🔍 - For searching
- ✅ - For success
- ❌ - For errors
- 📝 - For notes/documentation

## Character Voice Examples

### CHONKER Says:
- "🐹 *nom nom nom* Processing..."
- "🐹 *burp* All done!"
- "🐹 This PDF gave me a tummy ache!"

### SNYFTER Says:
- "🐁 *adjusts tiny glasses* Welcome to the archive."
- "🐁 Filed under reference 2024-001."
- "🐁 *shuffles index cards* Searching..."

## Implementation Checklist
- [ ] All CHONKER references use 🐹 (U+1F439)
- [ ] All SNYFTER references use 🐁 (U+1F401)
- [ ] No mouse face emoji 🐭 anywhere in code
- [ ] Error messages include character emoji
- [ ] UI labels include character emoji
- [ ] Git commits can include both 🐹🐁

## Testing the Emojis
```python
# Correct emojis test
assert "🐹" == "\U0001F439"  # CHONKER
assert "🐁" == "\U0001F401"  # SNYFTER

# Wrong emoji test (should fail)
assert "🐭" != "🐁"  # Mouse face is NOT SNYFTER!
```