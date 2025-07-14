#!/usr/bin/env python3
"""
CHONKER - Consolidated WYSIWYG Document Editor
A complete document processing and editing pipeline in one file.
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


def generate_wysiwyg_editor(document_text: str, tables: List[Dict], metadata: Dict, output_path: str, native_html: str = None, pdf_file: str = None) -> str:
    """Generate a minimal WYSIWYG editor with PDF.js viewer."""
    
    # Get document title
    doc_title = metadata.get('title', 'Document')
    
    # Encode PDF as base64 data URL to avoid CORS issues
    pdf_data_url = ""
    if pdf_file and os.path.exists(pdf_file):
        print("Encoding PDF as base64...")
        pdf_data_url = encode_pdf_as_base64(pdf_file)
    
    # Use native HTML if available, otherwise use a simple fallback
    if native_html:
        # Clean up the HTML but preserve all formatting
        from bs4 import BeautifulSoup
        
        soup = BeautifulSoup(native_html, 'html.parser')
        
        # Single pass with set lookup for better performance
        tags_to_remove = {'html', 'head', 'body', 'meta', 'title', 'style', 'script'}
        for tag in soup.find_all(tags_to_remove):
            if tag.name in {'style', 'script'}:
                tag.decompose()
            else:
                tag.unwrap()
        
        content_html = str(soup).strip()
        
        # If we still have nothing, fallback to text
        if not content_html:
            content_html = f"<div>{document_text}</div>"
    else:
        # Simple fallback
        content_html = f"<div>{document_text}</div>"
    
    # Create the minimal WYSIWYG editor HTML with PDF.js viewer
    html_content = f"""<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{doc_title}</title>
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="dns-prefetch" href="https://fonts.googleapis.com">
    <style>
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}
        
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: white;
            min-height: 100vh;
            overflow-x: hidden;
            display: grid;
            grid-template-columns: 1fr 1fr;
            gap: 0;
        }}
        
        .pane-left {{
            height: 100vh;
            border-right: 1px solid #ddd;
            position: relative;
            overflow: hidden;
            background: #f5f5f5;
            display: flex;
            flex-direction: column;
        }}
        
        .pdf-toolbar {{
            background: #2c3e50;
            color: white;
            padding: 10px;
            display: flex;
            align-items: center;
            gap: 15px;
            flex-shrink: 0;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }}
        
        .pdf-toolbar button {{
            background: #34495e;
            color: white;
            border: none;
            padding: 6px 12px;
            cursor: pointer;
            border-radius: 4px;
            font-size: 14px;
            transition: background 0.2s;
        }}
        
        .pdf-toolbar button:hover {{
            background: #4a5f7a;
        }}
        
        .pdf-toolbar button:active {{
            transform: translateY(1px);
        }}
        
        .pdf-toolbar button:disabled {{
            opacity: 0.5;
            cursor: not-allowed;
        }}
        
        .page-info {{
            font-size: 14px;
            padding: 0 10px;
        }}
        
        .zoom-controls {{
            display: flex;
            align-items: center;
            gap: 10px;
            margin-left: auto;
        }}
        
        .zoom-level {{
            font-size: 14px;
            min-width: 50px;
            text-align: center;
        }}
        
        .pdf-container {{
            flex: 1;
            overflow: auto;
            display: flex;
            justify-content: center;
            padding: 20px;
            background: #e0e0e0;
        }}
        
        #pdfCanvas {{
            box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
            background: white;
            max-width: 100%;
            height: auto;
        }}
        
        .pane-right {{
            height: 100vh;
            overflow-y: auto;
        }}
        
        .editor {{
            padding: 40px;
            min-height: 100vh;
            outline: none;
            font-size: 16px;
            line-height: 1.6;
            color: #333;
            max-width: none;
            margin: 0;
            border: none;
            box-shadow: none;
        }}
        
        .editor:focus {{
            outline: none;
        }}
        
        /* Preserve all formatting from Docling */
        .editor h1, .editor h2, .editor h3, .editor h4, .editor h5, .editor h6 {{
            margin: 20px 0 10px 0;
            color: #2c3e50;
        }}
        
        .editor p {{
            margin: 10px 0;
        }}
        
        .editor table {{
            border-collapse: collapse;
            width: 100%;
            margin: 20px 0;
        }}
        
        .editor th, .editor td {{
            padding: 12px;
            text-align: left;
        }}
        
        .editor th {{
            font-weight: bold;
        }}
        
        .editor strong {{
            font-weight: bold;
        }}
        
        .editor em {{
            font-style: italic;
        }}
        
        .editor ul, .editor ol {{
            margin: 10px 0;
            padding-left: 30px;
        }}
        
        .editor li {{
            margin: 5px 0;
        }}
        
        .editor blockquote {{
            border-left: 4px solid #007bff;
            padding-left: 20px;
            margin: 20px 0;
            color: #666;
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
        
        /* Loading spinner */
        .loading {{
            position: absolute;
            top: 50%;
            left: 50%;
            transform: translate(-50%, -50%);
            font-size: 18px;
            color: #666;
        }}
        
        /* New document button */
        .new-doc-button {{
            position: fixed;
            top: 20px;
            left: 20px;
            background: #3498db;
            color: white;
            border: none;
            padding: 10px 20px;
            border-radius: 4px;
            cursor: pointer;
            font-size: 14px;
            z-index: 1000;
            box-shadow: 0 2px 4px rgba(0,0,0,0.2);
        }}
        
        .new-doc-button:hover {{
            background: #2980b9;
        }}
        
        /* Fullscreen button */
        .fullscreen-button {{
            position: fixed;
            bottom: 20px;
            right: 20px;
            background: #9b59b6;
            color: white;
            border: none;
            padding: 10px 15px;
            border-radius: 4px;
            cursor: pointer;
            font-size: 14px;
            z-index: 1000;
            box-shadow: 0 2px 4px rgba(0,0,0,0.2);
        }}
        
        .fullscreen-button:hover {{
            background: #8e44ad;
        }}
    </style>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/pdf.js/3.11.174/pdf.min.js"></script>
</head>
<body>
    <button class="new-doc-button" onclick="openNewDocument()">📄 Open New Document</button>
    <button class="fullscreen-button" onclick="toggleFullscreen()">⛶ Fullscreen</button>
    
    <div class="pane-left">
        <div class="pdf-toolbar">
            <button id="prevPage" onclick="changePage(-1)">← Previous</button>
            <span class="page-info">
                Page <span id="pageNum">1</span> of <span id="pageCount">1</span>
            </span>
            <button id="nextPage" onclick="changePage(1)">Next →</button>
            
            <div class="zoom-controls">
                <button onclick="changeZoom(-0.2)">−</button>
                <span class="zoom-level" id="zoomLevel">100%</span>
                <button onclick="changeZoom(0.2)">+</button>
                <button onclick="fitToWidth()">Fit Width</button>
            </div>
        </div>
        <div class="pdf-container" id="pdfContainer">
            <canvas id="pdfCanvas"></canvas>
            <div class="loading" id="loading">Loading PDF...</div>
        </div>
    </div>
    <div class="pane-right">
        <div class="editor" contenteditable="true" id="editor">
            {content_html}
        </div>
    </div>
    
    <div class="status" id="status">Saved!</div>
    
    <script>
        // Function to toggle fullscreen
        function toggleFullscreen() {{
            if (!document.fullscreenElement) {{
                document.documentElement.requestFullscreen().catch(err => {{
                    console.log('Error attempting to enable fullscreen:', err);
                }});
            }} else {{
                document.exitFullscreen();
            }}
        }}
        
        // Function to open new document
        function openNewDocument() {{
            if (confirm('Open a new document? Any unsaved changes will be lost.')) {{
                // Create a file input and trigger it
                const input = document.createElement('input');
                input.type = 'file';
                input.accept = '.pdf';
                input.onchange = function(e) {{
                    const file = e.target.files[0];
                    if (file) {{
                        alert('To open a new file, please restart CHONKER and select the new PDF.');
                    }}
                }};
                input.click();
            }}
        }}
        
        // PDF.js setup
        pdfjsLib.GlobalWorkerOptions.workerSrc = 'https://cdnjs.cloudflare.com/ajax/libs/pdf.js/3.11.174/pdf.worker.min.js';
        
        let pdfDoc = null;
        let pageNum = 1;
        let pageRendering = false;
        let pageNumPending = null;
        let scale = 1.0;
        const canvas = document.getElementById('pdfCanvas');
        const ctx = canvas.getContext('2d');
        
        // Load the PDF from base64 data
        const pdfData = '{pdf_data_url}';
        
        if (pdfData) {{
            pdfjsLib.getDocument(pdfData).promise.then(function(pdf) {{
                pdfDoc = pdf;
                document.getElementById('pageCount').textContent = pdf.numPages;
                document.getElementById('loading').style.display = 'none';
                
                // Initial page render
                renderPage(pageNum);
            }}).catch(function(error) {{
                console.error('Error loading PDF:', error);
                document.getElementById('loading').textContent = 'Error loading PDF';
            }});
        }} else {{
            document.getElementById('loading').textContent = 'No PDF data available';
        }}
        
        // Render the page
        function renderPage(num) {{
            if (!pdfDoc) return;
            
            pageRendering = true;
            
            pdfDoc.getPage(num).then(function(page) {{
                const viewport = page.getViewport({{ scale: scale }});
                canvas.height = viewport.height;
                canvas.width = viewport.width;
                
                const renderContext = {{
                    canvasContext: ctx,
                    viewport: viewport
                }};
                
                const renderTask = page.render(renderContext);
                
                renderTask.promise.then(function() {{
                    pageRendering = false;
                    
                    if (pageNumPending !== null) {{
                        renderPage(pageNumPending);
                        pageNumPending = null;
                    }}
                }});
            }});
            
            // Update page counter
            document.getElementById('pageNum').textContent = num;
            
            // Update button states
            document.getElementById('prevPage').disabled = (num <= 1);
            document.getElementById('nextPage').disabled = (num >= pdfDoc.numPages);
        }}
        
        // Queue render page
        function queueRenderPage(num) {{
            if (pageRendering) {{
                pageNumPending = num;
            }} else {{
                renderPage(num);
            }}
        }}
        
        // Change page
        function changePage(delta) {{
            if (!pdfDoc) return;
            
            const newPage = pageNum + delta;
            if (newPage >= 1 && newPage <= pdfDoc.numPages) {{
                pageNum = newPage;
                queueRenderPage(pageNum);
            }}
        }}
        
        // Change zoom
        function changeZoom(delta) {{
            scale = Math.max(0.5, Math.min(3.0, scale + delta));
            document.getElementById('zoomLevel').textContent = Math.round(scale * 100) + '%';
            queueRenderPage(pageNum);
        }}
        
        // Fit to width
        function fitToWidth() {{
            if (!pdfDoc) return;
            
            const container = document.getElementById('pdfContainer');
            const containerWidth = container.clientWidth - 40; // Subtract padding
            
            pdfDoc.getPage(pageNum).then(function(page) {{
                const viewport = page.getViewport({{ scale: 1.0 }});
                scale = containerWidth / viewport.width;
                document.getElementById('zoomLevel').textContent = Math.round(scale * 100) + '%';
                queueRenderPage(pageNum);
            }});
        }}
        
        // Keyboard navigation for PDF
        document.addEventListener('keydown', function(e) {{
            if (e.target === document.body || e.target.classList.contains('pane-left')) {{
                switch(e.key) {{
                    case 'ArrowLeft':
                    case 'PageUp':
                        changePage(-1);
                        break;
                    case 'ArrowRight':
                    case 'PageDown':
                        changePage(1);
                        break;
                }}
            }}
        }});
        
        // Cache localStorage key
        const STORAGE_KEY = 'chonker_document_{doc_title}';
        
        // Auto-save functionality with optimized debouncing
        let autoSaveTimer;
        let hasChanges = false;
        let lastSavedContent = '';
        
        const editor = document.getElementById('editor');
        const status = document.getElementById('status');
        
        // Setup on load
        window.addEventListener('load', function() {{
            // Enable editing immediately
            editor.focus();
            
            // Load from localStorage if available
            const saved = localStorage.getItem(STORAGE_KEY);
            if (saved && confirm('Found auto-saved version. Load it?')) {{
                editor.innerHTML = saved;
                lastSavedContent = saved;
            }} else {{
                lastSavedContent = editor.innerHTML;
            }}
            
            // Initial fit to width after a short delay
            if (pdfDoc) {{
                setTimeout(fitToWidth, 500);
            }}
        }});
        
        // Optimized auto-save with debouncing
        editor.addEventListener('input', function() {{
            hasChanges = true;
            if (!autoSaveTimer) {{
                autoSaveTimer = setTimeout(function() {{
                    if (hasChanges) {{
                        saveToLocalStorage();
                        hasChanges = false;
                    }}
                    autoSaveTimer = null;
                }}, 5000);
            }}
        }});
        
        // Save to localStorage with redundancy check
        function saveToLocalStorage() {{
            const content = editor.innerHTML;
            if (content !== lastSavedContent) {{
                localStorage.setItem(STORAGE_KEY, content);
                lastSavedContent = content;
                showStatus('Auto-saved');
            }}
        }}
        
        // Save document
        function saveDocument() {{
            const content = editor.innerHTML;
            const blob = new Blob([document.documentElement.outerHTML], {{
                type: 'text/html'
            }});
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
        
        // Export as HTML
        function exportDocument() {{
            const content = editor.innerHTML;
            const blob = new Blob([content], {{
                type: 'text/html'
            }});
            const url = URL.createObjectURL(blob);
            
            const a = document.createElement('a');
            a.href = url;
            a.download = '{doc_title}_content.html';
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
            URL.revokeObjectURL(url);
            
            showStatus('Exported!');
        }}
        
        // Print document
        function printDocument() {{
            window.print();
        }}
        
        // Show status message with CSS transitions
        function showStatus(message) {{
            status.textContent = message;
            status.classList.add('show');
            setTimeout(() => {{
                status.classList.remove('show');
            }}, 1500);
        }}
        
        // Handle keyboard shortcuts
        document.addEventListener('keydown', function(e) {{
            if (e.ctrlKey || e.metaKey) {{
                switch(e.key) {{
                    case 's':
                        e.preventDefault();
                        saveDocument();
                        break;
                    case 'p':
                        e.preventDefault();
                        printDocument();
                        break;
                    case 'b':
                        e.preventDefault();
                        document.execCommand('bold', false, null);
                        break;
                    case 'i':
                        e.preventDefault();
                        document.execCommand('italic', false, null);
                        break;
                    case 'u':
                        e.preventDefault();
                        document.execCommand('underline', false, null);
                        break;
                }}
            }}
            
            // ESC to exit fullscreen
            if (e.key === 'Escape' && document.fullscreenElement) {{
                document.exitFullscreen();
            }}
        }});
        
    </script>
</body>
</html>"""
    
    # Write the HTML file WITHOUT minification
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
                "--disable-web-security",
                "--allow-file-access-from-files",
                "--disable-features=VizDisplayCompositor",
                "--no-first-run",
                "--no-default-browser-check",
                "--disable-default-apps",
                "--disable-popup-blocking",
                "--disable-extensions",
                "--disable-plugins",
                "--disable-background-timer-throttling",
                "--disable-backgrounding-occluded-windows",
                "--disable-renderer-backgrounding",
                "--disable-features=TranslateUI",
                "--disable-ipc-flooding-protection",
                "--disable-session-crashed-bubble",
                "--disable-infobars",
                "--hide-scrollbars"
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
        print("Launching CHONKER editor...")
        success = launch_browser(html_file)
        
        if success:
            print("\nCHONKER editor launched successfully!")
            print("\nFeatures:")
            print("- PDF viewer with custom controls on the left")
            print("- Editable content on the right")
            print("- Use arrow keys or buttons to navigate PDF")
            print("- Click 'Fullscreen' button to enter fullscreen mode")
            print("- Ctrl+S to save, Ctrl+B for bold, etc.")
            print("- Click 'Open New Document' to load another PDF")
            print("\nNote: Keep this terminal open while using the editor.")
        else:
            print("Failed to launch browser")
    else:
        print("Failed to process PDF document")


if __name__ == "__main__":
    main()
