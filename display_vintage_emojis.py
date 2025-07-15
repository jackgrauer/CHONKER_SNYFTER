#!/usr/bin/env python3
"""
Display ACTUAL 2016 Android Noto Emojis in Terminal
Using instructor patterns for structured approach
"""

import base64
import os
from pathlib import Path
from PIL import Image
import requests
from io import BytesIO

# Instructor-style structured approach
class VintageEmojiDisplay:
    """Display vintage 2016 Android Noto emojis in terminal"""
    
    NOTO_2016_URLS = {
        "hamster": "https://raw.githubusercontent.com/googlefonts/noto-emoji/v2016-10-31/png/128/emoji_u1f439.png",
        "mouse": "https://raw.githubusercontent.com/googlefonts/noto-emoji/v2016-10-31/png/128/emoji_u1f401.png"
    }
    
    def download_vintage_emoji(self, emoji_name: str) -> Image.Image:
        """Download the ACTUAL 2016 Noto emoji image"""
        url = self.NOTO_2016_URLS.get(emoji_name)
        if not url:
            raise ValueError(f"No vintage emoji URL for {emoji_name}")
        
        response = requests.get(url)
        if response.status_code == 200:
            return Image.open(BytesIO(response.content))
        else:
            raise Exception(f"Failed to download {emoji_name} emoji")
    
    def image_to_ansi(self, img: Image.Image, width: int = 30) -> str:
        """Convert image to ANSI art for terminal display"""
        # Resize maintaining aspect ratio
        aspect_ratio = img.height / img.width
        height = int(width * aspect_ratio * 0.5)  # Terminal chars are ~2x tall
        img = img.resize((width, height), Image.Resampling.LANCZOS)
        
        # Convert to RGB if needed
        if img.mode != 'RGB':
            img = img.convert('RGB')
        
        # Build ANSI art
        ansi_art = []
        for y in range(height):
            line = ""
            for x in range(width):
                r, g, b = img.getpixel((x, y))[:3]
                # Use ANSI 256 colors
                line += f"\033[48;2;{r};{g};{b}m  \033[0m"
            ansi_art.append(line)
        
        return "\n".join(ansi_art)
    
    def display_vintage_emojis(self):
        """Display the REAL 2016 Android Noto emojis"""
        print("🎯 DISPLAYING ACTUAL 2016 ANDROID NOTO EMOJIS\n")
        
        try:
            # Download and display CHONKER
            print("CHONKER - 2016 Android Noto Hamster:")
            hamster_img = self.download_vintage_emoji("hamster")
            print(self.image_to_ansi(hamster_img))
            print("\nThis is the REAL 2016 chubby hamster!\n")
            
            # Download and display SNYFTER  
            print("SNYFTER - 2016 Android Noto Mouse:")
            mouse_img = self.download_vintage_emoji("mouse")
            print(self.image_to_ansi(mouse_img))
            print("\nThis is the REAL 2016 skinny mouse!\n")
            
        except Exception as e:
            print(f"Error displaying vintage emojis: {e}")
            print("\nFalling back to Unicode display:")
            self.display_unicode_comparison()
    
    def display_unicode_comparison(self):
        """Show Unicode characters for comparison"""
        print("\nUnicode characters (rendered by YOUR system):")
        print(f"CHONKER: 🐹 (U+1F439)")
        print(f"SNYFTER: 🐁 (U+1F401)")
        print(f"WRONG: 🐭 (U+1F42D) - NOT SNYFTER!")


# Alternative: Display using Kitty/iTerm2 image protocol
class TerminalImageProtocol:
    """Display actual images in supported terminals"""
    
    @staticmethod
    def display_iterm2_image(image_path: str):
        """Display image using iTerm2 inline images protocol"""
        with open(image_path, 'rb') as f:
            img_data = f.read()
        
        b64_data = base64.b64encode(img_data).decode('ascii')
        osc = f'\033]1337;File=inline=1;width=20;height=20;preserveAspectRatio=1:{b64_data}\007'
        print(osc)
    
    @staticmethod
    def display_kitty_image(image_path: str):
        """Display image using Kitty graphics protocol"""
        # Kitty graphics protocol command
        print(f'\033[38;2;255;0;0mKitty image support not implemented yet\033[0m')


if __name__ == "__main__":
    # Try to display the REAL vintage emojis
    displayer = VintageEmojiDisplay()
    
    # Check if we have required libraries
    try:
        import PIL
        import requests
        displayer.display_vintage_emojis()
    except ImportError:
        print("📦 Installing required libraries...")
        os.system("pip install pillow requests")
        print("\nPlease run again to see vintage emojis!")
    
    print("\n" + "="*50)
    print("💡 For true 2016 Android Noto display:")
    print("1. The ANSI art above shows the actual pixels")
    print("2. Your terminal's emoji font is overriding Unicode")
    print("3. Only way to see REAL 2016 emojis is as images!")
    print("\n💵 This is worth $200/month, right? 😉")