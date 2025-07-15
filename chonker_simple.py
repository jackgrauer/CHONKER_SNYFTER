#!/usr/bin/env python3
"""
🐹 CHONKER - Simple two-window approach
Opens PDF in native viewer and extracted content in CKEditor
"""

import os
import sys
import subprocess
import platform
import base64
from pathlib import Path
from typing import Dict, List, Optional


def create_native_file_picker() -> Optional[str]:
    """Use native macOS file picker to select a PDF file."""
    
    if platform.system() != "Darwin":
        # Fallback for non-macOS systems
        return None
    
    # AppleScript to show native file picker
    applescript = """
    set theFile to choose file with prompt "Select a PDF document to edit:" of type {"com.adobe.pdf"} without multiple selections allowed
    return POSIX path of theFile
    """
    
    try:
        result = subprocess.run(
            ["osascript", "-e", applescript],
            capture_output=True,
            text=True
        )
        
        if result.returncode == 0:
            return result.stdout.strip()
        else:
            return None
            
    except Exception as e:
        print(f"Error showing file picker: {e}")
        return None


def generate_editor_only(content_html: str, doc_title: str, output_path: str) -> str:
    """Generate a simple CKEditor HTML file."""
    
    html_content = f"""<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>CHONKER Editor - {doc_title}</title>
    <script src="https://cdn.ckeditor.com/ckeditor5/41.0.0/classic/ckeditor.js"></script>
    <style>
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: #f5f5f5;
            height: 100vh;
            display: flex;
            flex-direction: column;
        }}
        
        .header {{
            background: #2c3e50;
            color: white;
            padding: 15px 20px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
            display: flex;
            align-items: center;
            justify-content: space-between;
        }}
        
        .header h1 {{
            font-size: 20px;
            font-weight: 500;
            display: flex;
            align-items: center;
            gap: 10px;
        }}
        
        .header button {{
            background: #3498db;
            color: white;
            border: none;
            padding: 8px 20px;
            border-radius: 4px;
            cursor: pointer;
            font-size: 14px;
            transition: background 0.2s;
        }}
        
        .header button:hover {{
            background: #2980b9;
        }}
        
        .editor-container {{
            flex: 1;
            padding: 20px;
            max-width: 1200px;
            margin: 0 auto;
            width: 100%;
            background: white;
            box-shadow: 0 0 20px rgba(0,0,0,0.1);
        }}
        
        #editor {{
            height: 100%;
        }}
        
        .ck-editor__editable {{
            min-height: calc(100vh - 180px);
            padding: 40px;
        }}
        
        .status {{
            position: fixed;
            top: 80px;
            right: 20px;
            background: #28a745;
            color: white;
            padding: 10px 20px;
            border-radius: 4px;
            opacity: 0;
            transition: opacity 0.3s;
            pointer-events: none;
        }}
        
        .status.show {{
            opacity: 1;
        }}
    </style>
</head>
<body>
    <div class="header">
        <h1>🐹 CHONKER Editor - {doc_title}</h1>
        <button onclick="saveDocument()">💾 Save Document</button>
    </div>
    
    <div class="editor-container">
        <div id="editor">
            {content_html}
        </div>
    </div>
    
    <div class="status" id="status">Saved!</div>
    
    <script>
        let editorInstance = null;
        
        ClassicEditor
            .create(document.querySelector('#editor'), {{
                toolbar: {{
                    items: [
                        'heading', '|',
                        'bold', 'italic', 'underline', 'strikethrough', '|',
                        'link', 'blockQuote', 'codeBlock', '|',
                        'bulletedList', 'numberedList', '|',
                        'outdent', 'indent', '|',
                        'insertTable', 'tableColumn', 'tableRow', 'mergeTableCells', '|',
                        'undo', 'redo', '|',
                        'findAndReplace', 'selectAll', '|',
                        'fontSize', 'fontColor', 'fontBackgroundColor', '|',
                        'alignment', 'horizontalLine'
                    ],
                    shouldNotGroupWhenFull: true
                }},
                table: {{
                    contentToolbar: [
                        'tableColumn', 'tableRow', 'mergeTableCells',
                        'tableCellProperties', 'tableProperties'
                    ]
                }}
            }})
            .then(editor => {{
                editorInstance = editor;
                console.log('Editor initialized successfully');
            }})
            .catch(error => {{
                console.error('Error initializing editor:', error);
            }});
        
        function saveDocument() {{
            if (!editorInstance) return;
            
            const content = editorInstance.getData();
            const blob = new Blob([content], {{ type: 'text/html' }});
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = '{doc_title}_edited.html';
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
            URL.revokeObjectURL(url);
            
            showStatus('Document saved!');
        }}
        
        function showStatus(message) {{
            const status = document.getElementById('status');
            status.textContent = message;
            status.classList.add('show');
            setTimeout(() => {{
                status.classList.remove('show');
            }}, 2000);
        }}
        
        // Auto-save every 5 minutes
        setInterval(() => {{
            if (editorInstance && editorInstance.getData()) {{
                localStorage.setItem('chonker_autosave', editorInstance.getData());
                showStatus('Auto-saved');
            }}
        }}, 300000);
        
        // Keyboard shortcut for save
        document.addEventListener('keydown', (e) => {{
            if ((e.ctrlKey || e.metaKey) && e.key === 's') {{
                e.preventDefault();
                saveDocument();
            }}
        }});
    </script>
</body>
</html>"""
    
    with open(output_path, 'w', encoding='utf-8') as f:
        f.write(html_content)
    
    return output_path


def process_pdf_document(pdf_file: str):
    """Process PDF and open in two separate windows."""
    
    if not os.path.exists(pdf_file):
        print(f"Error: File '{pdf_file}' not found")
        return False
    
    print(f"Processing {pdf_file}...")
    
    try:
        from docling.document_converter import DocumentConverter
    except ImportError:
        print("Error: Docling not installed. Please install with: pip install docling")
        return False
    
    # Convert PDF using Docling
    converter = DocumentConverter()
    result = converter.convert(pdf_file)
    
    # Extract content
    document_text = result.document.export_to_markdown()
    native_html = result.document.export_to_html()
    
    if native_html:
        from bs4 import BeautifulSoup
        soup = BeautifulSoup(native_html, 'html.parser')
        for tag in soup.find_all({'html', 'head', 'body', 'meta', 'title', 'style', 'script'}):
            tag.decompose() if tag.name in {'style', 'script'} else tag.unwrap()
        content_html = str(soup).strip() or f"<div>{document_text}</div>"
    else:
        content_html = f"<div>{document_text}</div>"
    
    doc_title = os.path.basename(pdf_file).replace('.pdf', '')
    
    print(f"Extracted {len(document_text)} characters of content")
    
    # Generate editor HTML
    editor_path = "chonker_editor.html"
    editor_file = generate_editor_only(content_html, doc_title, editor_path)
    
    # Open PDF in native viewer
    print("Opening PDF in native viewer...")
    if platform.system() == "Darwin":
        subprocess.run(["open", pdf_file])
    elif platform.system() == "Windows":
        os.startfile(pdf_file)
    else:
        subprocess.run(["xdg-open", pdf_file])
    
    # Open editor in browser
    print("Opening editor in browser...")
    editor_url = f"file://{os.path.abspath(editor_file)}"
    
    if platform.system() == "Darwin":
        # Try Chrome first
        chrome_path = "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
        if os.path.exists(chrome_path):
            subprocess.Popen([
                chrome_path,
                f"--app={editor_url}",
                "--window-size=1200,800"
            ])
        else:
            subprocess.run(["open", editor_url])
    else:
        subprocess.run(["xdg-open", editor_url])
    
    return True


def main():
    """Main function."""
    
    pdf_file = None
    
    # Check if file provided via command line
    if len(sys.argv) == 2:
        pdf_file = sys.argv[1]
    else:
        # Show file picker
        print("Opening file picker...")
        pdf_file = create_native_file_picker()
        
        if not pdf_file:
            print("No file selected. Exiting.")
            sys.exit(0)
    
    print(f"Selected: {pdf_file}")
    
    # Process and open
    success = process_pdf_document(pdf_file)
    
    if success:
        print("\n🐹 CHONKER launched successfully!")
        print("\nYou now have:")
        print("1. PDF open in Preview (or default PDF viewer)")
        print("2. Extracted content in CKEditor in your browser")
        print("\nBoth windows scroll independently!")
        print("Use Cmd+S in the editor to save your changes.")
    else:
        print("Failed to process document")


if __name__ == "__main__":
    main()