#!/usr/bin/env python3
"""
Get the EXACT 2016 Android Noto Emojis from historical commits
"""

import urllib.request
import os

# These URLs point to the EXACT 2016 commit in the Noto emoji repo
# From the Android 7.0 Nougat release (2016)
EXACT_2016_URLS = {
    # From commit before the 2017 redesign
    "CHONKER_hamster_2016.png": "https://github.com/googlefonts/noto-emoji/raw/914c9ecb9b16b13903e030bbd64a3604e7c7dd90/png/128/emoji_u1f439.png",
    "SNYFTER_mouse_2016.png": "https://github.com/googlefonts/noto-emoji/raw/914c9ecb9b16b13903e030bbd64a3604e7c7dd90/png/128/emoji_u1f401.png",
    
    # Also try the Android 7.0 specific branch
    "CHONKER_android7.png": "https://github.com/googlefonts/noto-emoji/raw/v2017-05-18-cook/png/128/emoji_u1f439.png",
    "SNYFTER_android7.png": "https://github.com/googlefonts/noto-emoji/raw/v2017-05-18-cook/png/128/emoji_u1f401.png",
}

def download_exact_2016():
    """Download the EXACT 2016 versions"""
    
    os.makedirs("exact_2016_emojis", exist_ok=True)
    
    print("🔍 Fetching EXACT 2016 Android Noto emojis...")
    print("   (From before the 2017 redesign)\n")
    
    for filename, url in EXACT_2016_URLS.items():
        filepath = os.path.join("exact_2016_emojis", filename)
        
        print(f"Downloading {filename}...")
        try:
            urllib.request.urlretrieve(url, filepath)
            print(f"✅ Saved: {filepath}")
        except Exception as e:
            print(f"❌ Failed: {e}")
            # Try alternate sources
            if "hamster" in filename:
                # Android KitKat era (2013-2014)
                alt_url = "https://github.com/googlefonts/noto-emoji/raw/v1.04/png/128/emoji_u1f439.png"
                try:
                    urllib.request.urlretrieve(alt_url, filepath.replace(".png", "_kitkat.png"))
                    print(f"  → Got KitKat era version instead")
                except:
                    pass

if __name__ == "__main__":
    download_exact_2016()
    print("\n📁 Check 'exact_2016_emojis' folder!")
    print("🕰️  These should be the chunky, original designs!")