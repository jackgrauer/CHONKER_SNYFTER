#!/usr/bin/env python3
"""
🐹 CHONKER - Consolidated WYSIWYG Document Editor
A complete document processing and editing pipeline in one file.
Fixed version with working trackpad scrolling support.
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
    """Generate a minimal WYSIWYG editor with PDF.js viewer and fixed trackpad support."""
    
    doc_title = metadata.get('title', 'Document')
    pdf_data_url = encode_pdf_as_base64(pdf_file) if pdf_file and os.path.exists(pdf_file) else ""
    
    if pdf_data_url:
        print("Encoding PDF as base64...")
    
    # Process HTML content
    if native_html:
        from bs4 import BeautifulSoup
        soup = BeautifulSoup(native_html, 'html.parser')
        for tag in soup.find_all({'html', 'head', 'body', 'meta', 'title', 'style', 'script'}):
            tag.decompose() if tag.name in {'style', 'script'} else tag.unwrap()
        content_html = str(soup).strip() or f"<div>{document_text}</div>"
    else:
        content_html = f"<div>{document_text}</div>"
    
    # Create the minimal WYSIWYG editor HTML with fixed trackpad support
    html_content = f"""<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{doc_title}</title>
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="dns-prefetch" href="https://fonts.googleapis.com">
    <style>
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: white;
            min-height: 100vh;
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
            flex-shrink: 0;
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
            display: flex;
            align-items: center;
            justify-content: center;
            gap: 5px;
            font-weight: 500;
        }}
        
        .toolbar-container button:hover {{
            background: #4a5f7a;
            transform: translateY(-1px);
            box-shadow: 0 2px 4px rgba(0,0,0,0.2);
        }}
        
        .toolbar-container button:active {{
            transform: translateY(0);
            box-shadow: none;
        }}
        
        .toolbar-container button:disabled {{
            opacity: 0.5;
            cursor: not-allowed;
            transform: none;
        }}
        
        .toolbar-container button.primary {{ background: #3498db; }}
        .toolbar-container button.primary:hover {{ background: #2980b9; }}
        .toolbar-container button.danger {{ background: #e74c3c; }}
        .toolbar-container button.danger:hover {{ background: #c0392b; }}
        .toolbar-container button.accent {{ background: #9b59b6; }}
        .toolbar-container button.accent:hover {{ background: #8e44ad; }}
        
        .page-info {{
            font-size: 14px;
            padding: 0 15px;
            color: #ecf0f1;
            font-weight: 500;
        }}
        
        .zoom-controls {{
            display: flex;
            align-items: center;
            gap: 10px;
            margin-left: auto;
        }}
        
        .zoom-level {{
            font-size: 14px;
            min-width: 60px;
            text-align: center;
            color: #ecf0f1;
            font-weight: 500;
        }}
        
        .main-container {{
            flex: 1;
            display: flex;
            overflow: hidden;
            position: relative;
        }}
        
        .pane-left {{
            flex: 1;
            min-width: 300px;
            border-right: 1px solid #2c3e50;
            position: relative;
            overflow: auto;
            background: #1a1a1a;
            height: 100%;
            display: block;
        }}
        
        .resizer {{
            width: 5px;
            background: #ddd;
            cursor: col-resize;
            position: relative;
            transition: background 0.2s;
        }}
        
        .resizer:hover {{
            background: #bbb;
        }}
        
        .resizer:active {{
            background: #999;
        }}
        
        .pdf-container {{
            display: block;
            text-align: center;
            padding: 40px;
            background: #2d2d2d;
            position: relative;
            width: auto;
            height: auto;
            min-width: 100%;
        }}
        
        #pdfCanvas {{
            box-shadow: 0 4px 20px rgba(0, 0, 0, 0.8);
            background: white;
            display: inline-block;
            margin: 0 auto;
            pointer-events: auto;
            touch-action: auto;
        }}
        
        .pane-right {{
            flex: 1;
            min-width: 300px;
            overflow-y: auto;
            position: relative;
        }}
        
        .editor {{
            padding: 40px;
            min-height: 100vh;
            outline: none;
            font-family: 'Menlo', 'Monaco', 'Consolas', 'Liberation Mono', 'Courier New', monospace;
            font-size: 14px;
            line-height: 1.8;
            color: #333;
            max-width: 100%;
            margin: 0;
            border: none;
            box-shadow: none;
            word-wrap: break-word;
            word-break: break-word;
            overflow-wrap: break-word;
            white-space: pre-wrap;
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
            max-width: 100%;
            margin: 20px 0;
            border: 1px solid #ccc;
            table-layout: fixed;
            overflow-x: auto;
            display: block;
        }}
        
        .editor th, .editor td {{
            padding: 12px;
            text-align: left;
            border: 1px solid #ccc;
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
            max-width: 100%;
            overflow-wrap: break-word;
        }}
        
        /* Responsive images */
        .editor img {{
            max-width: 100%;
            height: auto;
            display: block;
            margin: 10px 0;
        }}
        
        /* Responsive code blocks */
        .editor pre {{
            max-width: 100%;
            overflow-x: auto;
            white-space: pre-wrap;
            word-wrap: break-word;
            background: #f5f5f5;
            padding: 10px;
            border-radius: 4px;
            margin: 10px 0;
        }}
        
        /* Long URLs and text */
        .editor a {{
            word-break: break-all;
            overflow-wrap: break-word;
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
        
        /* Context menu */
        .context-menu {{
            position: fixed;
            background: white;
            border: 1px solid #ccc;
            border-radius: 4px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.2);
            display: none;
            z-index: 1000;
            padding: 5px 0;
            min-width: 150px;
        }}
        
        .context-menu-item {{
            padding: 8px 16px;
            cursor: pointer;
            font-size: 14px;
            color: #333;
            transition: background 0.2s;
        }}
        
        .context-menu-item:hover {{
            background: #f0f0f0;
        }}
        
        .context-menu-separator {{
            height: 1px;
            background: #e0e0e0;
            margin: 5px 0;
        }}
        
        .loading {{
            position: absolute;
            top: 50%;
            left: 50%;
            transform: translate(-50%, -50%);
            font-size: 18px;
            color: #ccc;
            background: rgba(0, 0, 0, 0.8);
            padding: 20px 30px;
            border-radius: 8px;
        }}
    </style>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/pdf.js/3.11.174/pdf.min.js"></script>
</head>
<body>
    <div class="toolbar-container">
        <button class="primary" onclick="openNewDocument()">📄 Open New</button>
        <button class="danger" onclick="showOptimizeMenu()">⚡ Optimize</button>
        <button id="prevPage" onclick="changePage(-1)">← Previous</button>
        <span class="page-info">
            Page <span id="pageNum">1</span> of <span id="pageCount">1</span>
        </span>
        <button id="nextPage" onclick="changePage(1)">Next →</button>
        
        <div class="zoom-controls">
            <button onclick="changeZoom(-0.2)">➖ Zoom Out</button>
            <span class="zoom-level" id="zoomLevel">100%</span>
            <button onclick="changeZoom(0.2)">➕ Zoom In</button>
            <button onclick="fitToWidth()">📐 Fit Width</button>
            <button class="accent" onclick="toggleFullscreen()">⛶ Fullscreen</button>
        </div>
    </div>
    
    <div class="main-container">
        <div class="pane-left" id="pdfPane">
            <div class="pdf-container" id="pdfContainer">
                <canvas id="pdfCanvas"></canvas>
                <div class="loading" id="loading">Loading PDF...</div>
            </div>
        </div>
        <div class="resizer" id="resizer"></div>
        <div class="pane-right" id="editorPane">
            <div class="editor" contenteditable="true" id="editor">
                {content_html}
            </div>
        </div>
    </div>
    
    <div class="status" id="status">Saved!</div>
    
    <!-- Context Menu -->
    <div class="context-menu" id="contextMenu">
        <div class="context-menu-item" onclick="addRowAbove()">↑ Add Row Above</div>
        <div class="context-menu-item" onclick="addRowBelow()">↓ Add Row Below</div>
        <div class="context-menu-separator"></div>
        <div class="context-menu-item" onclick="deleteRow()">🗑️ Delete Row</div>
        <div class="context-menu-separator"></div>
        <div class="context-menu-item" onclick="addColumnLeft()">← Add Column Left</div>
        <div class="context-menu-item" onclick="addColumnRight()">→ Add Column Right</div>
        <div class="context-menu-separator"></div>
        <div class="context-menu-item" onclick="deleteColumn()">🗑️ Delete Column</div>
    </div>
    
    <script>
        // Function to show PDF optimization menu
        function showOptimizeMenu() {{
            const options = [
                'OCR Text Recognition',
                'Enhance Scanned Pages',
                'Fix Text Extraction',
                'Reduce File Size',
                'Convert Image PDFs'
            ];
            
            let menu = '🐹 CHONKER PDF Optimization\\n\\n';
            menu += 'Select an optimization option:\\n\\n';
            options.forEach((opt, i) => {{
                menu += `${{i+1}}. ${{opt}}\\n`;
            }});
            menu += '\\nNote: These optimizations require server-side processing.';
            
            alert(menu);
        }}
        
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
        
        if (pdfData && pdfData !== 'data:application/pdf;base64,') {{
            console.log('Loading PDF...');
            pdfjsLib.getDocument(pdfData).promise.then(function(pdf) {{
                console.log('PDF loaded successfully, pages:', pdf.numPages);
                pdfDoc = pdf;
                document.getElementById('pageCount').textContent = pdf.numPages;
                document.getElementById('loading').style.display = 'none';
                
                // Initial page render with fit to width
                fitToWidth();
            }}).catch(function(error) {{
                console.error('Error loading PDF:', error);
                document.getElementById('loading').innerHTML = 'Error loading PDF<br><small>' + error.message + '</small>';
            }});
        }} else {{
            console.log('No PDF data available');
            document.getElementById('loading').innerHTML = 'No PDF loaded<br><small>Select a PDF file to begin</small>';
        }}
        
        // Render the page
        function renderPage(num) {{
            if (!pdfDoc) {{
                console.log('No PDF document loaded');
                return;
            }}
            
            console.log('Rendering page', num);
            pageRendering = true;
            
            pdfDoc.getPage(num).then(function(page) {{
                const viewport = page.getViewport({{ scale: scale }});
                
                // Use the scale directly without auto-fitting
                const scaledViewport = page.getViewport({{ scale: scale }});
                
                canvas.height = scaledViewport.height;
                canvas.width = scaledViewport.width;
                
                console.log('Canvas dimensions:', canvas.width, 'x', canvas.height);
                
                // Force the canvas to display at its actual pixel dimensions
                canvas.style.width = canvas.width + 'px';
                canvas.style.height = canvas.height + 'px';
                
                const renderContext = {{
                    canvasContext: ctx,
                    viewport: scaledViewport
                }};
                
                const renderTask = page.render(renderContext);
                
                renderTask.promise.then(function() {{
                    console.log('Page rendered successfully');
                    pageRendering = false;
                    
                    if (pageNumPending !== null) {{
                        renderPage(pageNumPending);
                        pageNumPending = null;
                    }}
                }}).catch(function(error) {{
                    console.error('Error rendering page:', error);
                    pageRendering = false;
                }});
            }}).catch(function(error) {{
                console.error('Error getting page:', error);
                pageRendering = false;
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
        const changePage = (delta) => {{
            if (!pdfDoc) return;
            const newPage = pageNum + delta;
            if (newPage >= 1 && newPage <= pdfDoc.numPages) {{
                pageNum = newPage;
                queueRenderPage(pageNum);
            }}
        }};
        
        // Change zoom
        const changeZoom = (delta) => {{
            scale = Math.max(0.5, Math.min(3.0, scale + delta));
            document.getElementById('zoomLevel').textContent = Math.round(scale * 100) + '%';
            queueRenderPage(pageNum);
        }};
        
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
        
        // FIXED: Enhanced trackpad and mouse wheel support
        const pdfContainer = document.getElementById('pdfContainer');
        
        // Variables for touch tracking
        let touchStartX = 0;
        let touchStartY = 0;
        let lastTouchX = 0;
        let lastTouchY = 0;
        let isTwoFingerTouch = false;
        let touchStartTime = 0;
        let velocityX = 0;
        let velocityY = 0;
        let lastMoveTime = 0;
        
        // Variables for pinch zoom
        let startScale = 1;
        
        // Enhanced wheel handler for trackpad support
        const pdfPane = document.getElementById('pdfPane');
        
        // Debug logging
        console.log('Setting up wheel handlers...');
        
        // Attach wheel handler to the scrollable pane
        pdfPane.addEventListener('wheel', function(e) {{
            const beforeScroll = {{
                scrollTop: pdfPane.scrollTop,
                scrollLeft: pdfPane.scrollLeft
            }};
            
            console.log('Wheel event on pdfPane:', {{
                deltaX: e.deltaX,
                deltaY: e.deltaY,
                ctrlKey: e.ctrlKey,
                metaKey: e.metaKey,
                target: e.target.id || e.target.className,
                scrollTopBefore: beforeScroll.scrollTop,
                scrollLeftBefore: beforeScroll.scrollLeft
            }});
            
            // For pinch-to-zoom gestures
            if (e.ctrlKey || e.metaKey) {{
                e.preventDefault();
                const delta = e.deltaY > 0 ? -0.1 : 0.1;
                changeZoom(delta);
                return;
            }}
            
            // For normal scrolling - manually apply the scroll
            if (!e.ctrlKey && !e.metaKey) {{
                e.preventDefault(); // Prevent default to control scrolling manually
                
                pdfPane.scrollLeft += e.deltaX;
                pdfPane.scrollTop += e.deltaY;
                
                // Log if scroll actually happened
                setTimeout(() => {{
                    console.log('After scroll:', {{
                        scrollTop: pdfPane.scrollTop,
                        scrollLeft: pdfPane.scrollLeft,
                        didScrollVertically: pdfPane.scrollTop !== beforeScroll.scrollTop,
                        didScrollHorizontally: pdfPane.scrollLeft !== beforeScroll.scrollLeft
                    }});
                }}, 0);
            }}
        }}, {{ passive: false }}); // Need passive: false to preventDefault
        
        // Also add a wheel handler to the canvas to ensure events bubble up
        canvas.addEventListener('wheel', function(e) {{
            console.log('Wheel event on canvas - letting it bubble');
            // Don't prevent default or stop propagation - let it bubble to pdfPane
        }}, {{ passive: true }});
        
        // Touch events for iOS devices and touch screens
        pdfContainer.addEventListener('touchstart', function(e) {{
            if (e.touches.length === 2) {{
                e.preventDefault();
                isTwoFingerTouch = true;
                touchStartTime = Date.now();
                
                touchStartX = (e.touches[0].clientX + e.touches[1].clientX) / 2;
                touchStartY = (e.touches[0].clientY + e.touches[1].clientY) / 2;
                lastTouchX = touchStartX;
                lastTouchY = touchStartY;
                
                velocityX = 0;
                velocityY = 0;
                lastMoveTime = touchStartTime;
            }}
        }}, {{ passive: false }});
        
        pdfContainer.addEventListener('touchmove', function(e) {{
            if (e.touches.length === 2 && isTwoFingerTouch) {{
                e.preventDefault();
                
                const currentTime = Date.now();
                const currentX = (e.touches[0].clientX + e.touches[1].clientX) / 2;
                const currentY = (e.touches[0].clientY + e.touches[1].clientY) / 2;
                
                const deltaX = currentX - lastTouchX;
                const deltaY = currentY - lastTouchY;
                const deltaTime = currentTime - lastMoveTime;
                
                // Calculate velocity for momentum scrolling
                if (deltaTime > 0) {{
                    velocityX = deltaX / deltaTime * 16; // Convert to pixels per frame
                    velocityY = deltaY / deltaTime * 16;
                }}
                
                // Natural scrolling (inverted)
                pdfContainer.scrollLeft -= deltaX;
                pdfContainer.scrollTop -= deltaY;
                
                lastTouchX = currentX;
                lastTouchY = currentY;
                lastMoveTime = currentTime;
            }}
        }}, {{ passive: false }});
        
        pdfContainer.addEventListener('touchend', function(e) {{
            if (isTwoFingerTouch) {{
                const touchEndTime = Date.now();
                const touchDuration = touchEndTime - touchStartTime;
                const totalDeltaX = lastTouchX - touchStartX;
                const totalDeltaY = lastTouchY - touchStartY;
                
                // Check for quick swipe gesture for page navigation
                if (touchDuration < 300 && Math.abs(totalDeltaX) > Math.abs(totalDeltaY) && Math.abs(totalDeltaX) > 50) {{
                    // Quick horizontal swipe for page change
                    if (totalDeltaX > 0) {{
                        changePage(-1); // Swipe right = previous page
                    }} else {{
                        changePage(1); // Swipe left = next page
                    }}
                }} else {{
                    // Apply momentum scrolling
                    applyMomentum();
                }}
                
                isTwoFingerTouch = false;
            }}
        }});
        
        // Momentum scrolling function
        function applyMomentum() {{
            const friction = 0.95;
            const minVelocity = 0.5;
            
            function momentumStep() {{
                if (Math.abs(velocityX) > minVelocity || Math.abs(velocityY) > minVelocity) {{
                    pdfContainer.scrollLeft -= velocityX;
                    pdfContainer.scrollTop -= velocityY;
                    
                    velocityX *= friction;
                    velocityY *= friction;
                    
                    requestAnimationFrame(momentumStep);
                }}
            }}
            
            requestAnimationFrame(momentumStep);
        }}
        
        // Pinch to zoom with gesturestart/gesturechange (Safari)
        pdfContainer.addEventListener('gesturestart', function(e) {{
            e.preventDefault();
            startScale = scale;
        }}, {{ passive: false }});
        
        pdfContainer.addEventListener('gesturechange', function(e) {{
            e.preventDefault();
            const newScale = Math.max(0.5, Math.min(3.0, startScale * e.scale));
            if (newScale !== scale) {{
                scale = newScale;
                document.getElementById('zoomLevel').textContent = Math.round(scale * 100) + '%';
                queueRenderPage(pageNum);
            }}
        }}, {{ passive: false }});
        
        // Resizable panes
        const resizer = document.getElementById('resizer');
        const leftPane = document.getElementById('pdfPane');
        const rightPane = document.getElementById('editorPane');
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
        
        // Table context menu functionality
        let currentCell = null;
        const contextMenu = document.getElementById('contextMenu');
        
        // Hide context menu on click outside
        document.addEventListener('click', function() {{
            contextMenu.style.display = 'none';
        }});
        
        // Unified table manipulation
        function tableAction(action) {{
            if (!currentCell) return;
            contextMenu.style.display = 'none';
            
            const row = currentCell.parentElement;
            const table = currentCell.closest('table');
            const cellIndex = Array.from(row.children).indexOf(currentCell);
            
            switch(action) {{
                case 'addRowAbove':
                case 'addRowBelow': {{
                    const newRow = row.cloneNode(true);
                    newRow.querySelectorAll('td, th').forEach(c => c.textContent = '');
                    row.parentElement.insertBefore(newRow, action === 'addRowAbove' ? row : row.nextSibling);
                    break;
                }}
                case 'deleteRow': {{
                    if (row.parentElement.children.length > 1) row.remove();
                    else alert('Cannot delete the last row');
                    break;
                }}
                case 'addColumnLeft':
                case 'addColumnRight': {{
                    table.querySelectorAll('tr').forEach(r => {{
                        const newCell = document.createElement(r.children[cellIndex].tagName);
                        Object.assign(newCell.style, {{border: '1px solid #ccc', padding: '12px'}});
                        r.insertBefore(newCell, r.children[cellIndex + (action === 'addColumnLeft' ? 0 : 1)]);
                    }});
                    break;
                }}
                case 'deleteColumn': {{
                    if (row.children.length > 1) {{
                        table.querySelectorAll('tr').forEach(r => r.children[cellIndex].remove());
                    }} else alert('Cannot delete the last column');
                    break;
                }}
            }}
        }}
        
        // Make functions global
        ['addRowAbove', 'addRowBelow', 'deleteRow', 'addColumnLeft', 'addColumnRight', 'deleteColumn'].forEach(fn => {{
            window[fn] = () => tableAction(fn);
        }});
        
        // Cache localStorage key
        const STORAGE_KEY = 'chonker_document_{doc_title}';
        const DISABLE_AUTOSAVE = false; // Set to true to disable auto-save
        
        // Auto-save functionality with optimized debouncing
        let autoSaveTimer;
        let hasChanges = false;
        let lastSavedContent = '';
        
        // Setup on load
        window.addEventListener('load', function() {{
            const editor = document.getElementById('editor');
            const status = document.getElementById('status');
            
            // Enable editing immediately
            editor.focus();
            
            // Load from localStorage if available
            const saved = !DISABLE_AUTOSAVE ? localStorage.getItem(STORAGE_KEY) : null;
            
            if (saved) {{
                if (confirm('Found auto-saved version. Load it?')) {{
                    editor.innerHTML = saved;
                    lastSavedContent = saved;
                }} else {{
                    // User chose not to load - clear it
                    localStorage.removeItem(STORAGE_KEY);
                    lastSavedContent = editor.innerHTML;
                    console.log('Cleared auto-saved content');
                }}
            }} else {{
                lastSavedContent = editor.innerHTML;
            }}
            
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
            
            // Add context menu to table cells
            editor.addEventListener('contextmenu', function(e) {{
                const target = e.target;
                if (target.tagName === 'TD' || target.tagName === 'TH') {{
                    e.preventDefault();
                    currentCell = target;
                    contextMenu.style.left = e.pageX + 'px';
                    contextMenu.style.top = e.pageY + 'px';
                    contextMenu.style.display = 'block';
                }}
            }});
            
            // Initial render at 200% zoom to ensure both vertical and horizontal scrolling
            setTimeout(() => {{
                if (pdfDoc) {{
                    scale = 2.0; // Start at 200% zoom for both directions
                    document.getElementById('zoomLevel').textContent = '200%';
                    queueRenderPage(pageNum);
                    
                    // Check if scrollbars are present
                    setTimeout(() => {{
                        const pdfPane = document.getElementById('pdfPane');
                        const scrollInfo = {{
                            scrollHeight: pdfPane.scrollHeight,
                            clientHeight: pdfPane.clientHeight,
                            scrollWidth: pdfPane.scrollWidth,
                            clientWidth: pdfPane.clientWidth,
                            hasVerticalScroll: pdfPane.scrollHeight > pdfPane.clientHeight,
                            hasHorizontalScroll: pdfPane.scrollWidth > pdfPane.clientWidth,
                            scrollTop: pdfPane.scrollTop,
                            scrollLeft: pdfPane.scrollLeft
                        }};
                        console.log('PDF Pane scroll info:', scrollInfo);
                        
                        // Force a layout recalculation
                        pdfPane.style.display = 'none';
                        pdfPane.offsetHeight; // Force reflow
                        pdfPane.style.display = '';
                        
                        // Re-check after reflow
                        const updatedInfo = {{
                            scrollHeight: pdfPane.scrollHeight,
                            clientHeight: pdfPane.clientHeight,
                            hasVerticalScroll: pdfPane.scrollHeight > pdfPane.clientHeight,
                            pdfContainerHeight: pdfContainer.offsetHeight,
                            canvasHeight: canvas.height
                        }};
                        console.log('Updated scroll info after reflow:', updatedInfo);
                        
                        // Center the view initially
                        if (scrollInfo.hasHorizontalScroll) {{
                            pdfPane.scrollLeft = (pdfPane.scrollWidth - pdfPane.clientWidth) / 2;
                        }}
                        if (scrollInfo.hasVerticalScroll) {{
                            pdfPane.scrollTop = (pdfPane.scrollHeight - pdfPane.clientHeight) / 2;
                        }}
                    }}, 500);
                }}
            }}, 100);
        }});
        
        // Save to localStorage with redundancy check
        function saveToLocalStorage() {{
            if (DISABLE_AUTOSAVE) return;
            
            const editor = document.getElementById('editor');
            const content = editor.innerHTML;
            if (content !== lastSavedContent) {{
                localStorage.setItem(STORAGE_KEY, content);
                lastSavedContent = content;
                showStatus('Auto-saved');
            }}
        }}
        
        // Save/Export document
        function saveDocument(exportOnly = false) {{
            const editor = document.getElementById('editor');
            const content = exportOnly ? editor.innerHTML : document.documentElement.outerHTML;
            const blob = new Blob([content], {{ type: 'text/html' }});
            const url = URL.createObjectURL(blob);
            
            const a = document.createElement('a');
            a.href = url;
            a.download = `{doc_title}_${{exportOnly ? 'content' : 'edited'}}.html`;
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
            URL.revokeObjectURL(url);
            
            showStatus(exportOnly ? 'Exported!' : 'Saved!');
        }}
        
        const exportDocument = () => saveDocument(true);
        
        // Print document
        function printDocument() {{
            window.print();
        }}
        
        // Show status message with CSS transitions
        function showStatus(message) {{
            const status = document.getElementById('status');
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
        
        // Log for debugging
        console.log('CHONKER: Enhanced trackpad scrolling initialized');
        
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
            print("- PDF viewer with custom controls on the left")
            print("- Editable content on the right")
            print("- Use arrow keys or buttons to navigate PDF")
            print("- Two-finger trackpad scrolling (vertical and horizontal)")
            print("- Pinch to zoom on trackpad")
            print("- Natural scrolling with momentum")
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
