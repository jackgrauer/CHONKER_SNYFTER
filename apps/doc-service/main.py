from fastapi import FastAPI, UploadFile, File, HTTPException, Request, WebSocket, WebSocketDisconnect
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
from typing import List, Optional, Dict, Any, AsyncGenerator
import tempfile
import os
import json
from datetime import datetime
import logging
import uuid
import asyncio

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# WebSocket Connection Manager
class ConnectionManager:
    def __init__(self):
        self.active_connections: Dict[str, WebSocket] = {}
    
    async def connect(self, session_id: str, websocket: WebSocket):
        await websocket.accept()
        self.active_connections[session_id] = websocket
        logger.info(f"[{session_id}] WebSocket connected")
    
    def disconnect(self, session_id: str):
        self.active_connections.pop(session_id, None)
        logger.info(f"[{session_id}] WebSocket disconnected")
    
    async def send_progress(self, session_id: str, data: dict):
        if websocket := self.active_connections.get(session_id):
            try:
                await websocket.send_json(data)
            except Exception as e:
                logger.error(f"[{session_id}] Failed to send progress: {e}")
                self.disconnect(session_id)

manager = ConnectionManager()

app = FastAPI(
    title="CHONKER Document Processing Service",
    description="Document processing service using Docling",
    version="1.0.0"
)

# Enable CORS for Tauri
app.add_middleware(
    CORSMiddleware,
    allow_origins=[
        "tauri://localhost",
        "http://localhost:*",
        "https://localhost:*",
        "http://127.0.0.1:*",
        "https://127.0.0.1:*"
    ],
    allow_methods=["*"],
    allow_headers=["*"],
    allow_credentials=True,
)

# Response models
class TableData(BaseModel):
    id: str
    headers: List[str]
    rows: List[List[str]]
    caption: Optional[str] = None

class DocumentMetadata(BaseModel):
    title: Optional[str] = None
    author: Optional[str] = None
    page_count: Optional[int] = None
    created_at: Optional[datetime] = None
    extracted_at: datetime

class ProcessingResult(BaseModel):
    success: bool
    message: str
    file_path: str
    request_id: str
    extracted_text: Optional[str] = None
    tables: Optional[List[TableData]] = None
    metadata: Optional[DocumentMetadata] = None

@app.get("/")
async def root():
    return {
        "service": "CHONKER Document Processing",
        "status": "running",
        "version": "1.0.0"
    }

@app.get("/health")
async def health_check():
    return {
        "status": "healthy",
        "timestamp": datetime.now().isoformat()
    }

# Configuration
MAX_FILE_SIZE = 50 * 1024 * 1024  # 50MB limit
OUTPUT_DIR = os.path.join(os.getcwd(), "processed_documents")

# Create output directory if it doesn't exist
os.makedirs(OUTPUT_DIR, exist_ok=True)

@app.post("/process", response_model=ProcessingResult)
async def process_document(file: UploadFile = File(...)):
    """
    Process a document using Docling
    """
    request_id = str(uuid.uuid4())
    logger.info(f"[{request_id}] Starting document processing request")
    
    if not file.filename:
        raise HTTPException(status_code=400, detail="No file provided")
    
    # Check file size
    if file.size and file.size > MAX_FILE_SIZE:
        raise HTTPException(
            status_code=413, 
            detail=f"File too large: {file.size} bytes. Maximum allowed: {MAX_FILE_SIZE} bytes"
        )
    
    # Check file type - Docling supported formats
    allowed_types = ['.pdf', '.docx', '.pptx', '.html', '.md', '.csv', '.xlsx', '.asciidoc']
    file_extension = os.path.splitext(file.filename)[1].lower()
    
    if file_extension not in allowed_types:
        raise HTTPException(
            status_code=400, 
            detail=f"Unsupported file type: {file_extension}. Supported types: {allowed_types}"
        )
    
    try:
        # Create temporary file
        with tempfile.NamedTemporaryFile(delete=False, suffix=file_extension) as temp_file:
            content = await file.read()
            temp_file.write(content)
            temp_file_path = temp_file.name
        
        logger.info(f"[{request_id}] Processing file: {file.filename} ({len(content)} bytes)")
        
        # Process with Docling
        result = await process_with_docling(temp_file_path, file.filename, request_id)
        
        # Clean up temporary file
        os.unlink(temp_file_path)
        
        logger.info(f"[{request_id}] Processing completed successfully")
        return result
        
    except Exception as e:
        # Clean up on error
        if 'temp_file_path' in locals():
            try:
                os.unlink(temp_file_path)
            except:
                pass
        
        logger.error(f"[{request_id}] Error processing document: {str(e)}")
        raise HTTPException(status_code=500, detail=f"Processing failed: {str(e)}")

@app.websocket("/ws/process/{session_id}")
async def websocket_process(websocket: WebSocket, session_id: str):
    await manager.connect(session_id, websocket)
    
    try:
        # Receive file metadata first
        metadata = await websocket.receive_json()
        filename = metadata.get('filename', 'unknown')
        
        # Receive file data
        file_data = await websocket.receive_bytes()
        
        logger.info(f"[{session_id}] Processing {filename} via WebSocket ({len(file_data)} bytes)")
        
        # Process with progress updates
        async for progress in process_document_with_progress(file_data, filename, session_id):
            await manager.send_progress(session_id, progress)
            
    except WebSocketDisconnect:
        logger.info(f"[{session_id}] WebSocket disconnected by client")
        manager.disconnect(session_id)
    except Exception as e:
        logger.error(f"[{session_id}] WebSocket error: {e}")
        await manager.send_progress(session_id, {
            "type": "error",
            "message": str(e)
        })
        manager.disconnect(session_id)

def generate_editable_html(extracted_html: str, base_name: str) -> str:
    """Add editing features to Docling's HTML output"""
    
    # Extract the body content from Docling's HTML
    import re
    body_match = re.search(r'<body[^>]*>(.*?)</body>', extracted_html, re.DOTALL)
    if body_match:
        body_content = body_match.group(1)
    else:
        # Fallback to using the whole HTML if no body tag found
        body_content = extracted_html
    
    return f'''<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{base_name} - Editable Document</title>
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700&display=swap" rel="stylesheet">
    <style>
        :root {{
            --primary-color: #2563eb;
            --secondary-color: #64748b;
            --background-color: #ffffff;
            --surface-color: #f8fafc;
            --border-color: #e2e8f0;
            --text-primary: #1e293b;
            --text-secondary: #64748b;
            --shadow-sm: 0 1px 2px 0 rgb(0 0 0 / 0.05);
            --shadow-md: 0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1);
            --shadow-lg: 0 10px 15px -3px rgb(0 0 0 / 0.1), 0 4px 6px -4px rgb(0 0 0 / 0.1);
        }}
        
        * {{
            box-sizing: border-box;
        }}
        
        body {{
            font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            font-size: 16px;
            line-height: 1.7;
            color: var(--text-primary);
            background-color: var(--background-color);
            margin: 0;
            padding: 0;
            min-height: 100vh;
            -webkit-font-smoothing: antialiased;
            -moz-osx-font-smoothing: grayscale;
        }}
        
        .document-container {{
            max-width: 900px;
            margin: 0 auto;
            padding: 2rem;
            background: white;
            min-height: 100vh;
            box-shadow: var(--shadow-sm);
        }}
        
        /* Typography */
        h1, h2, h3, h4, h5, h6 {{
            color: var(--text-primary);
            font-weight: 600;
            margin-top: 2rem;
            margin-bottom: 1rem;
            line-height: 1.3;
        }}
        
        h1 {{
            font-size: 2.5rem;
            font-weight: 700;
            margin-top: 0;
            padding-bottom: 0.5rem;
            border-bottom: 3px solid var(--primary-color);
        }}
        
        h2 {{
            font-size: 2rem;
            color: var(--primary-color);
            margin-top: 3rem;
        }}
        
        h3 {{
            font-size: 1.5rem;
            color: var(--secondary-color);
        }}
        
        p {{
            margin-bottom: 1.5rem;
            color: var(--text-primary);
        }}
        
        /* Lists */
        ul, ol {{
            margin: 1.5rem 0;
            padding-left: 1.5rem;
        }}
        
        li {{
            margin-bottom: 0.5rem;
            color: var(--text-primary);
        }}
        
        /* Tables */
        table {{
            width: 100%;
            border-collapse: collapse;
            margin: 2rem 0;
            background: white;
            border-radius: 8px;
            overflow: hidden;
            box-shadow: var(--shadow-md);
        }}
        
        th, td {{
            padding: 1rem;
            text-align: left;
            border-bottom: 1px solid var(--border-color);
        }}
        
        th {{
            background: linear-gradient(135deg, var(--primary-color), #3b82f6);
            color: white;
            font-weight: 600;
            text-transform: uppercase;
            font-size: 0.875rem;
            letter-spacing: 0.05em;
        }}
        
        td {{
            background: white;
            transition: background-color 0.2s ease;
        }}
        
        tr:hover td {{
            background-color: var(--surface-color);
        }}
        
        tr:nth-child(even) td {{
            background-color: #f9fafb;
        }}
        
        tr:nth-child(even):hover td {{
            background-color: var(--surface-color);
        }}
        
        /* Strong and emphasis */
        strong {{
            font-weight: 600;
            color: var(--text-primary);
        }}
        
        em {{
            font-style: italic;
            color: var(--text-secondary);
        }}
        
        /* Editing styles */
        [contenteditable] {{
            outline: none;
            transition: all 0.2s ease;
        }}
        
        [contenteditable]:focus {{
            outline: none;
            background-color: rgba(37, 99, 235, 0.03);
            border-radius: 4px;
        }}
        
        [contenteditable]:hover {{
            cursor: text;
        }}
        
        /* Status indicator */
        .status-indicator {{
            position: fixed;
            top: 20px;
            right: 20px;
            background: var(--primary-color);
            color: white;
            padding: 8px 16px;
            border-radius: 20px;
            font-size: 0.875rem;
            font-weight: 500;
            box-shadow: var(--shadow-lg);
            z-index: 1000;
            display: flex;
            align-items: center;
            gap: 8px;
        }}
        
        .status-indicator::before {{
            content: "✏️";
            font-size: 1rem;
        }}
        
        /* Responsive design */
        @media (max-width: 768px) {{
            .document-container {{
                padding: 1rem;
            }}
            
            h1 {{
                font-size: 2rem;
            }}
            
            h2 {{
                font-size: 1.5rem;
            }}
            
            table {{
                font-size: 0.875rem;
            }}
            
            th, td {{
                padding: 0.75rem;
            }}
        }}
        
        /* Print styles */
        @media print {{
            .status-indicator {{
                display: none;
            }}
            
            .document-container {{
                max-width: none;
                margin: 0;
                padding: 0;
                box-shadow: none;
            }}
            
            body {{
                font-size: 12pt;
                line-height: 1.4;
            }}
        }}
        
        /* Dark mode support */
        @media (prefers-color-scheme: dark) {{
            :root {{
                --background-color: #0f172a;
                --surface-color: #1e293b;
                --text-primary: #f1f5f9;
                --text-secondary: #94a3b8;
                --border-color: #334155;
            }}
            
            .document-container {{
                background: var(--surface-color);
                color: var(--text-primary);
            }}
            
            table {{
                background: var(--surface-color);
            }}
            
            td {{
                background: var(--surface-color);
            }}
            
            tr:nth-child(even) td {{
                background: #1a2332;
            }}
        }}
    </style>
</head>
<body>
    <div class="status-indicator">
        Editing Mode Active
    </div>
    
    <div class="document-container" contenteditable="true">
        {body_content}
    </div>
    
    <script>
        // Auto-save functionality
        let saveTimeout;
        let isModified = false;
        
        document.addEventListener('input', function() {{
            if (!isModified) {{
                isModified = true;
                document.title = '● ' + document.title;
            }}
            
            clearTimeout(saveTimeout);
            saveTimeout = setTimeout(autoSave, 2000);
        }});
        
        function autoSave() {{
            // Save to localStorage as backup
            localStorage.setItem('{base_name}_backup', document.querySelector('.document-container').innerHTML);
            
            // Update title to show saved status
            if (isModified) {{
                document.title = document.title.replace('● ', '');
                isModified = false;
            }}
        }}
        
        // Restore from backup if available
        document.addEventListener('DOMContentLoaded', function() {{
            const backup = localStorage.getItem('{base_name}_backup');
            if (backup && confirm('Found a backup of this document. Restore it?')) {{
                document.querySelector('.document-container').innerHTML = backup;
            }}
        }});
        
        // Keyboard shortcuts
        document.addEventListener('keydown', function(e) {{
            if (e.ctrlKey || e.metaKey) {{
                switch(e.key) {{
                    case 's':
                        e.preventDefault();
                        saveDocument();
                        break;
                    case 'z':
                        if (e.shiftKey) {{
                            document.execCommand('redo');
                        }} else {{
                            document.execCommand('undo');
                        }}
                        break;
                    case 'b':
                        e.preventDefault();
                        document.execCommand('bold');
                        break;
                    case 'i':
                        e.preventDefault();
                        document.execCommand('italic');
                        break;
                }}
            }}
        }});
        
        function saveDocument() {{
            const content = document.documentElement.outerHTML;
            const blob = new Blob([content], {{type: 'text/html'}});
            const url = URL.createObjectURL(blob);
            
            const a = document.createElement('a');
            a.href = url;
            a.download = '{base_name}_edited.html';
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
            URL.revokeObjectURL(url);
            
            // Clear backup after manual save
            localStorage.removeItem('{base_name}_backup');
            isModified = false;
            document.title = document.title.replace('● ', '');
        }}
    </script>
</body>
</html>'''

def save_processing_results(base_name: str, extracted_text: str, tables: list, metadata: DocumentMetadata, extracted_html: str = None) -> list:
    """Save processing results to output files"""
    output_files = []
    
    # Save extracted text as markdown
    text_file_path = os.path.join(OUTPUT_DIR, f"{base_name}_text.md")
    with open(text_file_path, 'w', encoding='utf-8') as text_file:
        text_file.write(extracted_text)
    output_files.append(text_file_path)
    
    # Save tables as JSON
    if tables:
        tables_file_path = os.path.join(OUTPUT_DIR, f"{base_name}_tables.json")
        with open(tables_file_path, 'w', encoding='utf-8') as tables_file:
            json.dump(tables, tables_file, indent=2, ensure_ascii=False)
        output_files.append(tables_file_path)
    
    # Save metadata as JSON
    metadata_file_path = os.path.join(OUTPUT_DIR, f"{base_name}_metadata.json")
    with open(metadata_file_path, 'w', encoding='utf-8') as metadata_file:
        json.dump({
            "title": metadata.title,
            "author": metadata.author,
            "page_count": metadata.page_count,
            "extracted_at": metadata.extracted_at.isoformat()
        }, metadata_file, indent=2, ensure_ascii=False)
    output_files.append(metadata_file_path)

    # Save markdown file for faithful document recreation view
    markdown_file_path = os.path.join(OUTPUT_DIR, f"{base_name}_faithful.md")
    with open(markdown_file_path, 'w', encoding='utf-8') as markdown_file:
        markdown_file.write(extracted_text)
    output_files.append(markdown_file_path)
    
    # Save editable HTML file with editing features
    if extracted_html:
        editable_html = generate_editable_html(extracted_html, base_name)
        html_file_path = os.path.join(OUTPUT_DIR, f"{base_name}_editable.html")
        with open(html_file_path, 'w', encoding='utf-8') as html_file:
            html_file.write(editable_html)
        output_files.append(html_file_path)
    
    # Also save the original native HTML if available
    if extracted_html:
        native_html_file_path = os.path.join(OUTPUT_DIR, f"{base_name}_native.html")
        with open(native_html_file_path, 'w', encoding='utf-8') as html_file:
            html_file.write(extracted_html)
        output_files.append(native_html_file_path)

    logger.info(f"Saved processing results to: {output_files}")
    return output_files

async def process_document_with_progress(file_data: bytes, filename: str, session_id: str) -> AsyncGenerator[Dict[str, Any], None]:
    """Process document with real-time progress updates"""
    
    # Stage 1: Initialization
    yield {
        "type": "progress",
        "stage": "initializing",
        "percent": 0,
        "message": "Starting document processing..."
    }
    
    try:
        # Stage 2: File preparation (10%)
        yield {
            "type": "progress",
            "stage": "preparing",
            "percent": 10,
            "message": "Preparing file for processing..."
        }
        
        # Create temporary file (let Docling determine if it can handle the format)
        file_extension = os.path.splitext(filename)[1].lower() or '.tmp'
        
        with tempfile.NamedTemporaryFile(delete=False, suffix=file_extension) as temp_file:
            temp_file.write(file_data)
            temp_file_path = temp_file.name
        
        # Stage 3: Document analysis (30%)
        yield {
            "type": "progress",
            "stage": "analyzing",
            "percent": 30,
            "message": "Analyzing document structure..."
        }
        
        # Stage 4: Docling processing (40-80%)
        yield {
            "type": "progress",
            "stage": "processing",
            "percent": 50,
            "message": "Processing with Docling..."
        }
        
        # Import and process with Docling
        from docling.document_converter import DocumentConverter
        converter = DocumentConverter()
        result = converter.convert(temp_file_path)
        
        # Stage 5: Text extraction (80%)
        yield {
            "type": "progress",
            "stage": "extracting",
            "percent": 80,
            "message": "Extracting text content..."
        }
        
        extracted_text = result.document.export_to_markdown()
        extracted_html = result.document.export_to_html()
        
        # Stage 6: Table extraction (90%)
        yield {
            "type": "progress",
            "stage": "tables",
            "percent": 90,
            "message": "Extracting tables and metadata..."
        }
        
        # Extract tables
        tables = []
        if hasattr(result.document, 'tables'):
            for i, table in enumerate(result.document.tables):
                table_data = TableData(
                    id=f"table_{i}",
                    headers=table.get_headers() if hasattr(table, 'get_headers') else [],
                    rows=table.get_rows() if hasattr(table, 'get_rows') else [],
                    caption=table.caption if hasattr(table, 'caption') else None
                )
                tables.append(table_data.model_dump())
        
        # Extract metadata
        metadata = DocumentMetadata(
            title=getattr(result.document, 'title', None),
            author=getattr(result.document, 'author', None),
            page_count=getattr(result.document, 'page_count', None),
            extracted_at=datetime.now()
        )
        
        # Clean up temporary file
        os.unlink(temp_file_path)
        
        # Stage 7: Save output files (95%)
        yield {
            "type": "progress",
            "stage": "saving",
            "percent": 95,
            "message": "Saving output files..."
        }
        
        # Save output files
        base_name = os.path.splitext(filename)[0]
        output_files = save_processing_results(base_name, extracted_text, tables, metadata, extracted_html)
        
        # Stage 8: Complete
        yield {
            "type": "complete",
            "percent": 100,
            "message": f"Processing complete! Saved {len(output_files)} files.",
            "result": {
                "text": extracted_text,
                "tables": tables,
                "metadata": {
                    "page_count": metadata.page_count if hasattr(metadata, 'page_count') else None,
                    "created_at": str(metadata.created_at) if hasattr(metadata, 'created_at') else None
                },
                "output_files": output_files
            }
        }
        
    except ImportError:
        logger.error(f"[{session_id}] Docling not available")
        yield {
            "type": "error",
            "message": "Document processing service misconfigured: Docling not installed"
        }
    except Exception as e:
        logger.error(f"[{session_id}] Processing error: {str(e)}")
        yield {
            "type": "error",
            "message": f"Document processing failed: {str(e)}"
        }

async def process_with_docling(file_path: str, original_filename: str, request_id: str) -> ProcessingResult:
    """
    Process document with Docling
    """
    try:
        # Import Docling (only when needed)
        from docling.document_converter import DocumentConverter
        
        # Create converter
        converter = DocumentConverter()
        
        # Convert document
        result = converter.convert(file_path)
        
        # Extract text as HTML for better fidelity
        extracted_html = result.document.export_to_html()
        # Also keep markdown for backwards compatibility
        extracted_text = result.document.export_to_markdown()
        
        # Extract tables (simplified - you can enhance this)
        tables = []
        if hasattr(result.document, 'tables'):
            for i, table in enumerate(result.document.tables):
                table_data = TableData(
                    id=f"table_{i}",
                    headers=table.get_headers() if hasattr(table, 'get_headers') else [],
                    rows=table.get_rows() if hasattr(table, 'get_rows') else [],
                    caption=table.caption if hasattr(table, 'caption') else None
                )
                tables.append(table_data)
        
        # Extract metadata
        metadata = DocumentMetadata(
            title=getattr(result.document, 'title', None),
            author=getattr(result.document, 'author', None),
            page_count=getattr(result.document, 'page_count', None),
            extracted_at=datetime.now()
        )
        
        # Save output files
        base_name = os.path.splitext(original_filename)[0]
        output_files = save_processing_results(base_name, extracted_text, [table.model_dump() for table in tables], metadata, extracted_html)
        
        return ProcessingResult(
            success=True,
            message=f"Successfully processed {original_filename}",
            file_path=original_filename,
            request_id=request_id,
            extracted_text=extracted_text,
            tables=tables,
            metadata=metadata
        )
        
    except ImportError:
        logger.error(f"[{request_id}] Docling not available - service misconfigured")
        raise Exception("Document processing service misconfigured: Docling not installed")
    except Exception as e:
        logger.error(f"[{request_id}] Docling processing error: {str(e)}")
        raise Exception(f"Document processing failed: {str(e)}")

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8000)
