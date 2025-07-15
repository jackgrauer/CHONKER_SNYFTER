#!/usr/bin/env python3
"""
Download and save ACTUAL 2016 Android Noto Emoji PNGs
"""

import urllib.request
import os

# Direct URLs to the ACTUAL 2016 Android Noto emoji PNGs
NOTO_2016_EMOJIS = {
    "hamster_1f439.png": "https://github.com/googlefonts/noto-emoji/raw/v2.038/png/128/emoji_u1f439.png",
    "mouse_1f401.png": "https://github.com/googlefonts/noto-emoji/raw/v2.038/png/128/emoji_u1f401.png",
}

def download_real_emojis():
    """Download the ACTUAL 2016 Noto emoji PNG files"""
    
    # Create emoji directory
    os.makedirs("noto_emojis", exist_ok=True)
    
    for filename, url in NOTO_2016_EMOJIS.items():
        filepath = os.path.join("noto_emojis", filename)
        
        print(f"Downloading {filename}...")
        try:
            urllib.request.urlretrieve(url, filepath)
            print(f"✅ Saved to {filepath}")
            
            # Also save with descriptive names
            if "1f439" in filename:
                urllib.request.urlretrieve(url, "noto_emojis/CHONKER_2016.png")
            elif "1f401" in filename:
                urllib.request.urlretrieve(url, "noto_emojis/SNYFTER_2016.png")
                
        except Exception as e:
            print(f"❌ Failed to download {filename}: {e}")
    
    print("\n✅ Real 2016 Android Noto emojis downloaded!")
    print("📁 Check the 'noto_emojis' folder to see them!")

if __name__ == "__main__":
    download_real_emojis()