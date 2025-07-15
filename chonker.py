#!/usr/bin/env python3
"""
🐹 CHONKER - Consolidated WYSIWYG Document Editor
A complete document processing and editing pipeline using iframes for independent scrolling.
"""

import os
import sys
import subprocess
import platform
import re
import json
import tempfile
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
            # Remove trailing newline and return the file path
            return result.stdout.strip()
        else:
            # User cancelled
            return None
            
    except Exception as e:
        print(f"Error showing file picker: {e}")
        return None


def encode_pdf_as_base64(pdf_path: str) -> str:
    """Encode PDF file as base64 data URL."""
    with open(pdf_path, 'rb') as f:
        pdf_data = f.read()
        base64_data = base64.b64encode(pdf_data).decode('utf-8')
        return f"data:application/pdf;base64,{base64_data}"


def generate_pdf_viewer(pdf_file: str, output_path: str) -> str:
    """Generate standalone PDF viewer HTML."""
    pdf_data_url = encode_pdf_as_base64(pdf_file) if pdf_file and os.path.exists(pdf_file) else ""
    
    html_content = f"""<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>PDF Viewer</title>
    <style>
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{
            background: #2d2d2d;
            overflow: auto;
            height: 100vh;
            display: flex;
            align-items: center;
            justify-content: center;
        }}
        #pdfCanvas {{
            box-shadow: 0 4px 20px rgba(0, 0, 0, 0.8);
            background: white;
            display: block;
            margin: 40px;
        }}
    </style>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/pdf.js/3.11.174/pdf.min.js"></script>
</head>
<body>
    <canvas id="pdfCanvas"></canvas>
    
    <script>
        pdfjsLib.GlobalWorkerOptions.workerSrc = 'https://cdnjs.cloudflare.com/ajax/libs/pdf.js/3.11.174/pdf.worker.min.js';
        let pdfDoc = null;
        let pageNum = 1;
        let scale = 1.5;
        const canvas = document.getElementById('pdfCanvas');
        const ctx = canvas.getContext('2d');
        
        const pdfData = '{pdf_data_url}';
        
        if (pdfData && pdfData !== 'data:application/pdf;base64,') {{
            pdfjsLib.getDocument(pdfData).promise.then(function(pdf) {{
                pdfDoc = pdf;
                renderPage(pageNum);
            }});
        }}
        
        function renderPage(num) {{
            pdfDoc.getPage(num).then(function(page) {{
                const viewport = page.getViewport({{ scale: scale }});
                canvas.height = viewport.height;
                canvas.width = viewport.width;
                
                const renderContext = {{
                    canvasContext: ctx,
                    viewport: viewport
                }};
                
                page.render(renderContext);
            }});
        }}
        
        // Listen for messages from parent
        window.addEventListener('message', function(e) {{
            if (e.data.action === 'nextPage' && pageNum < pdfDoc.numPages) {{
                pageNum++;
                renderPage(pageNum);
            }} else if (e.data.action === 'prevPage' && pageNum > 1) {{
                pageNum--;
                renderPage(pageNum);
            }} else if (e.data.action === 'zoom') {{
                scale = Math.max(0.5, Math.min(3.0, scale + e.data.delta));
                renderPage(pageNum);
            }}
        }});
    </script>
</body>
</html>"""
    
    with open(output_path, 'w', encoding='utf-8') as f:
        f.write(html_content)
    return output_path


def generate_editor(content_html: str, output_path: str) -> str:
    """Generate standalone CKEditor HTML."""
    html_content = f"""<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Editor</title>
    <script src="https://cdn.ckeditor.com/ckeditor5/41.0.0/classic/ckeditor.js"></script>
    <style>
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{ background: white; height: 100vh; overflow: hidden; }}
        #editor {{ height: 100%; }}
        .ck-editor__editable {{
            height: calc(100vh - 100px);
            overflow-y: auto !important;
        }}
    </style>
</head>
<body>
    <div id="editor">
        {content_html}
    </div>
    
    <script>
        ClassicEditor
            .create(document.querySelector('#editor'), {{
                toolbar: {{
                    items: [
                        'heading', '|',
                        'bold', 'italic', 'underline', 'strikethrough', '|',
                        'link', 'blockQuote', 'codeBlock', '|',
                        'bulletedList', 'numberedList', 'outdent', 'indent', '|',
                        'insertTable', 'tableColumn', 'tableRow', 'mergeTableCells', '|',
                        'undo', 'redo', '|',
                        'findAndReplace'
                    ]
                }},
                table: {{
                    contentToolbar: [
                        'tableColumn', 'tableRow', 'mergeTableCells',
                        'tableCellProperties', 'tableProperties'
                    ]
                }}
            }})
            .then(editor => {{
                window.editor = editor;
                
                // Listen for save commands from parent
                window.addEventListener('message', function(e) {{
                    if (e.data.action === 'getData') {{
                        parent.postMessage({{
                            action: 'editorData',
                            data: editor.getData()
                        }}, '*');
                    }}
                }});
            }})
            .catch(error => {{
                console.error('Error initializing CKEditor:', error);
            }});
    </script>
</body>
</html>"""
    
    with open(output_path, 'w', encoding='utf-8') as f:
        f.write(html_content)
    return output_path


def generate_wysiwyg_editor(document_text: str, tables: List[Dict], metadata: Dict, output_path: str, native_html: str = None, pdf_file: str = None) -> str:
    """Generate main container with iframes."""
    doc_title = metadata.get('title', 'Document')
    
    # Process HTML content
    if native_html:
        from bs4 import BeautifulSoup
        soup = BeautifulSoup(native_html, 'html.parser')
        for tag in soup.find_all({'html', 'head', 'body', 'meta', 'title', 'style', 'script'}):
            tag.decompose() if tag.name in {'style', 'script'} else tag.unwrap()
        content_html = str(soup).strip() or f"<div>{document_text}</div>"
    else:
        content_html = f"<div>{document_text}</div>"
    
    # Generate separate HTML files
    pdf_viewer_path = generate_pdf_viewer(pdf_file, "chonker_pdf.html")
    editor_path = generate_editor(content_html, "chonker_editor.html")
    
    # Create the main container HTML with iframes
    html_content = f"""<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{doc_title}</title>
    <style>
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: #1a1a1a;
            height: 100vh;
            overflow: hidden;
            display: flex;
            flex-direction: column;
        }}
        
        .toolbar-container {{
            background: #2c3e50;
            color: white;
            padding: 10px;
            display: flex;
            align-items: center;
            gap: 10px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.2);
            border-bottom: 2px solid #34495e;
        }}
        
        .toolbar-container button {{
            background: #34495e;
            color: white;
            border: none;
            padding: 8px 16px;
            cursor: pointer;
            border-radius: 4px;
            font-size: 13px;
            transition: all 0.2s;
            min-width: 100px;
            height: 32px;
        }}
        
        .toolbar-container button:hover {{
            background: #4a5f7a;
        }}
        
        .toolbar-container button:disabled {{
            opacity: 0.5;
            cursor: not-allowed;
        }}
        
        .page-info {{
            font-size: 14px;
            padding: 0 15px;
            color: #ecf0f1;
        }}
        
        .zoom-controls {{
            display: flex;
            align-items: center;
            gap: 10px;
            margin-left: auto;
        }}
        
        .main-container {{
            flex: 1;
            display: flex;
            overflow: hidden;
            position: relative;
        }}
        
        .iframe-pane {{
            flex: 1;
            min-width: 300px;
            height: 100%;
            position: relative;
        }}
        
        .iframe-pane iframe {{
            width: 100%;
            height: 100%;
            border: none;
        }}
        
        .iframe-pane.left {{
            border-right: 2px solid #2c3e50;
        }}
        
        .resizer {{
            width: 5px;
            background: #ddd;
            cursor: col-resize;
            position: relative;
        }}
        
        .resizer:hover {{
            background: #bbb;
        }}
        
        .status {{
            position: fixed;
            top: 20px;
            right: 20px;
            background: #28a745;
            color: white;
            padding: 8px 16px;
            border-radius: 4px;
            font-size: 14px;
            z-index: 1000;
            transition: opacity 0.3s ease;
            opacity: 0;
            pointer-events: none;
        }}
        
        .status.show {{
            opacity: 1;
        }}
    </style>
</head>
<body>
    <div class="toolbar-container">
        <button id="prevPage" onclick="sendToPDF('prevPage')">← Previous</button>
        <span class="page-info">
            Page <span id="pageNum">1</span> of <span id="pageCount">-</span>
        </span>
        <button id="nextPage" onclick="sendToPDF('nextPage')">Next →</button>
        
        <div class="zoom-controls">
            <button onclick="sendToPDF('zoom', -0.2)">➖ Zoom Out</button>
            <span class="zoom-level" id="zoomLevel">150%</span>
            <button onclick="sendToPDF('zoom', 0.2)">➕ Zoom In</button>
            <button onclick="saveDocument()">💾 Save</button>
        </div>
    </div>
    
    <div class="main-container">
        <div class="iframe-pane left">
            <iframe id="pdfFrame" src="chonker_pdf.html"></iframe>
        </div>
        <div class="resizer" id="resizer"></div>
        <div class="iframe-pane right">
            <iframe id="editorFrame" src="chonker_editor.html"></iframe>
        </div>
    </div>
    
    <div class="status" id="status">Saved!</div>
    
    <script>
        // Send messages to PDF iframe
        function sendToPDF(action, value) {{
            const pdfFrame = document.getElementById('pdfFrame');
            if (action === 'zoom') {{
                pdfFrame.contentWindow.postMessage({{ action: 'zoom', delta: value }}, '*');
            }} else {{
                pdfFrame.contentWindow.postMessage({{ action: action }}, '*');
            }}
        }}
        
        // Save document
        function saveDocument() {{
            const editorFrame = document.getElementById('editorFrame');
            editorFrame.contentWindow.postMessage({{ action: 'getData' }}, '*');
        }}
        
        // Listen for messages from iframes
        window.addEventListener('message', function(e) {{
            if (e.data.action === 'editorData') {{
                const blob = new Blob([e.data.data], {{ type: 'text/html' }});
                const url = URL.createObjectURL(blob);
                const a = document.createElement('a');
                a.href = url;
                a.download = '{doc_title}_edited.html';
                document.body.appendChild(a);
                a.click();
                document.body.removeChild(a);
                URL.revokeObjectURL(url);
                showStatus('Saved!');
            }}
        }});
        
        // Show status
        function showStatus(message) {{
            const status = document.getElementById('status');
            status.textContent = message;
            status.classList.add('show');
            setTimeout(() => {{
                status.classList.remove('show');
            }}, 1500);
        }}
        
        // Resizable panes
        const resizer = document.getElementById('resizer');
        const leftPane = document.querySelector('.iframe-pane.left');
        const rightPane = document.querySelector('.iframe-pane.right');
        let isResizing = false;
        
        resizer.addEventListener('mousedown', function(e) {{
            isResizing = true;
            document.body.style.cursor = 'col-resize';
            e.preventDefault();
        }});
        
        document.addEventListener('mousemove', function(e) {{
            if (!isResizing) return;
            
            const containerWidth = document.querySelector('.main-container').offsetWidth;
            const newLeftWidth = e.clientX;
            const leftPercent = (newLeftWidth / containerWidth) * 100;
            const rightPercent = 100 - leftPercent;
            
            if (leftPercent > 20 && leftPercent < 80) {{
                leftPane.style.flex = `0 0 ${{leftPercent}}%`;
                rightPane.style.flex = `0 0 ${{rightPercent}}%`;
            }}
        }});
        
        document.addEventListener('mouseup', function() {{
            isResizing = false;
            document.body.style.cursor = 'default';
        }});
    </script>
</body>
</html>"""
    
    # Write the HTML file
    with open(output_path, 'w', encoding='utf-8') as f:
        f.write(html_content)
    
    return output_path


def launch_browser(html_file_path: str):
    """Launch the HTML file in the browser."""
    
    if not os.path.exists(html_file_path):
        print(f"Error: HTML file not found at {html_file_path}")
        return False
    
    # Convert to absolute path
    html_file_path = os.path.abspath(html_file_path)
    file_url = f"file://{html_file_path}"
    
    system = platform.system()
    
    if system == "Darwin":  # macOS
        # Try Chrome first (app mode - no browser chrome)
        chrome_path = "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
        if os.path.exists(chrome_path):
            print("Launching editor in Chrome app mode...")
            subprocess.Popen([
                chrome_path,
                f"--app={file_url}",
                "--window-size=1470,956",
                "--disable-web-security",
                "--allow-file-access-from-files",
                "--no-first-run",
                "--disable-default-apps",
                "--disable-popup-blocking",
                "--disable-infobars"
            ])
            return True
        
        # Try Safari if Chrome isn't available
        try:
            print("Launching editor in Safari...")
            subprocess.run(["open", "-a", "Safari", file_url])
            return True
        except:
            pass
    
    # Fallback to default browser
    print("Opening editor in default browser...")
    if system == "Darwin":
        subprocess.run(["open", file_url])
    elif system == "Linux":
        subprocess.run(["xdg-open", file_url])
    elif system == "Windows":
        subprocess.run(["start", file_url], shell=True)
    
    return True


def process_pdf_document(pdf_file: str) -> str:
    """Process PDF document using Docling and generate WYSIWYG editor."""
    
    if not os.path.exists(pdf_file):
        print(f"Error: File '{pdf_file}' not found")
        return None
    
    print(f"Processing {pdf_file}...")
    
    try:
        # Import Docling
        from docling.document_converter import DocumentConverter
    except ImportError:
        print("Error: Docling not installed. Please install with: pip install docling")
        return None
    
    # Convert PDF using Docling
    converter = DocumentConverter()
    result = converter.convert(pdf_file)
    
    # Extract content with enhanced options to capture all elements
    document_text = result.document.export_to_markdown()
    native_html = result.document.export_to_html()
    
    # Extract tables (if any)
    tables = []
    for table in result.document.tables:
        tables.append({
            'data': table.export_to_dataframe().to_dict('records') if hasattr(table, 'export_to_dataframe') else [],
            'caption': getattr(table, 'caption', '')
        })
    
    print(f"Extracted {len(tables)} tables from document")
    print(f"Document has {len(document_text)} characters of content")
    
    # Extract metadata
    metadata = {
        'title': getattr(result.document, 'title', os.path.basename(pdf_file)),
        'source': pdf_file
    }
    
    # Generate the WYSIWYG editor
    output_path = "chonker_editor.html"
    result_path = generate_wysiwyg_editor(
        document_text=document_text,
        tables=tables,
        metadata=metadata,
        output_path=output_path,
        native_html=native_html,
        pdf_file=os.path.abspath(pdf_file)
    )
    
    print(f"WYSIWYG editor generated: {result_path}")
    return result_path


def main():
    """Main function to process PDF and launch WYSIWYG editor."""
    
    pdf_file = None
    
    # Check if a file was provided via command line
    if len(sys.argv) == 2:
        pdf_file = sys.argv[1]
        print(f"Processing file from command line: {pdf_file}")
    else:
        # Show native file picker directly
        print("Opening native file picker...")
        pdf_file = create_native_file_picker()
        
        if not pdf_file:
            print("No file selected. Exiting.")
            sys.exit(0)
        
        print(f"Selected file: {pdf_file}")
    
    # Process the PDF document
    html_file = process_pdf_document(pdf_file)
    
    if html_file:
        # Launch the browser
        print("Launching 🐹 CHONKER editor...")
        success = launch_browser(html_file)
        
        if success:
            print("\n🐹 CHONKER editor launched successfully!")
            print("\nFeatures:")
            print("- PDF viewer in left iframe - scrolls independently!")
            print("- CKEditor in right iframe - scrolls independently!")
            print("- Use navigation buttons to control PDF")
            print("- Full table editing support in CKEditor")
            print("- Save button exports edited content")
            print("\nNote: Keep this terminal open while using the editor.")
        else:
            print("Failed to launch browser")
    else:
        print("Failed to process PDF document")


if __name__ == "__main__":
    main()