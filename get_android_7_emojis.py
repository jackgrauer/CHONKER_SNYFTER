#!/usr/bin/env python3
"""
Get the EXACT Android 7.0 (2016) emojis from Emojipedia
These are the ones you saw in egui!
"""

import urllib.request
import os

# The EXACT Android 7.0 emojis from Emojipedia
ANDROID_7_EMOJIS = {
    # These are served by Emojipedia's CDN
    "CHONKER_android_7.png": "https://em-content.zobj.net/source/google/110/hamster_1f439.png",
    "SNYFTER_android_7.png": "https://em-content.zobj.net/source/google/110/mouse_1f401.png",
    
    # Alternative URLs from Emojipedia's Android 7.0 archive
    "CHONKER_nougat.png": "https://em-content.zobj.net/thumbs/120/google/110/hamster-face_1f439.png", 
    "SNYFTER_nougat.png": "https://em-content.zobj.net/thumbs/120/google/110/mouse_1f401.png",
}

def download_android_7_emojis():
    """Download the Android 7.0 Nougat emojis - the EXACT ones from egui"""
    
    os.makedirs("android_7_emojis", exist_ok=True)
    
    print("🎯 Getting Android 7.0 Nougat (2016) emojis...")
    print("   The EXACT ones from egui!\n")
    
    for filename, url in ANDROID_7_EMOJIS.items():
        filepath = os.path.join("android_7_emojis", filename)
        
        print(f"Downloading {filename}...")
        try:
            # Set user agent to avoid blocking
            opener = urllib.request.build_opener()
            opener.addheaders = [('User-Agent', 'Mozilla/5.0')]
            urllib.request.install_opener(opener)
            
            urllib.request.urlretrieve(url, filepath)
            print(f"✅ Saved: {filepath}")
            
        except Exception as e:
            print(f"❌ Failed: {e}")

if __name__ == "__main__":
    download_android_7_emojis()
    print("\n🐹 These are the CHUNKY Android 7.0 emojis!")
    print("🐁 The ones with the distinctive Google style from 2016!")
    print("📁 Check 'android_7_emojis' folder!")