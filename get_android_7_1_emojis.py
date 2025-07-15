#!/usr/bin/env python3
"""
Get the EXACT Android 7.1 emojis - the flat design ones!
"""

import urllib.request
import os

# Android 7.1 used a different emoji set - the flat design before blobs
ANDROID_7_1_URLS = {
    # From Emojipedia's Android 7.1 collection (design version 3)
    "CHONKER_android_7_1.png": "https://em-content.zobj.net/source/google/3/hamster-face_1f439.png",
    "SNYFTER_android_7_1.png": "https://em-content.zobj.net/source/google/3/mouse_1f401.png",
    
    # Alternative CDN paths
    "CHONKER_flat.png": "https://emojipedia-us.s3.dualstack.us-west-1.amazonaws.com/thumbs/120/google/3/hamster-face_1f439.png",
    "SNYFTER_flat.png": "https://emojipedia-us.s3.dualstack.us-west-1.amazonaws.com/thumbs/120/google/3/mouse_1f401.png",
}

def download_android_7_1():
    """Get the Android 7.1 flat design emojis"""
    
    os.makedirs("android_7_1_emojis", exist_ok=True)
    
    print("🎯 Getting Android 7.1 emojis - the FLAT DESIGN ones!")
    print("   These are the non-blob 2016 versions!\n")
    
    for filename, url in ANDROID_7_1_URLS.items():
        filepath = os.path.join("android_7_1_emojis", filename)
        
        print(f"Downloading {filename}...")
        try:
            headers = {
                'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36'
            }
            request = urllib.request.Request(url, headers=headers)
            response = urllib.request.urlopen(request)
            
            with open(filepath, 'wb') as f:
                f.write(response.read())
            
            print(f"✅ Saved: {filepath}")
            
        except Exception as e:
            print(f"❌ Failed: {e}")

if __name__ == "__main__":
    download_android_7_1()
    print("\n🐹 CHONKER - The flat design hamster from Android 7.1!")
    print("🐁 SNYFTER - The flat design mouse from Android 7.1!")
    print("📱 These are the 2016 Google designs before the blob era!")