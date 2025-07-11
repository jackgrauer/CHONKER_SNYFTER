from fastapi import FastAPI, UploadFile, File, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
from typing import List, Optional, Dict, Any
import tempfile
import os
import json
from datetime import datetime
import logging

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

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

@app.post("/process", response_model=ProcessingResult)
async def process_document(file: UploadFile = File(...)):
    """
    Process a document using Docling
    """
    if not file.filename:
        raise HTTPException(status_code=400, detail="No file provided")
    
    # Check file type
    allowed_types = ['.pdf', '.docx', '.doc', '.txt']
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
        
        logger.info(f"Processing file: {file.filename} ({len(content)} bytes)")
        
        # Process with Docling
        result = await process_with_docling(temp_file_path, file.filename)
        
        # Clean up temporary file
        os.unlink(temp_file_path)
        
        return result
        
    except Exception as e:
        # Clean up on error
        if 'temp_file_path' in locals():
            try:
                os.unlink(temp_file_path)
            except:
                pass
        
        logger.error(f"Error processing document: {str(e)}")
        raise HTTPException(status_code=500, detail=f"Processing failed: {str(e)}")

async def process_with_docling(file_path: str, original_filename: str) -> ProcessingResult:
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
        
        # Extract text
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
        
        return ProcessingResult(
            success=True,
            message=f"Successfully processed {original_filename}",
            file_path=original_filename,
            extracted_text=extracted_text,
            tables=tables,
            metadata=metadata
        )
        
    except ImportError:
        # Fallback if Docling is not available
        logger.warning("Docling not available, using mock processing")
        return ProcessingResult(
            success=True,
            message=f"Mock processing of {original_filename} (Docling not installed)",
            file_path=original_filename,
            extracted_text=f"Mock extracted text from {original_filename}",
            tables=[
                TableData(
                    id="mock_table_1",
                    headers=["Column 1", "Column 2", "Column 3"],
                    rows=[
                        ["Row 1 Col 1", "Row 1 Col 2", "Row 1 Col 3"],
                        ["Row 2 Col 1", "Row 2 Col 2", "Row 2 Col 3"]
                    ],
                    caption="Mock table from document"
                )
            ],
            metadata=DocumentMetadata(
                title=f"Document: {original_filename}",
                author="Unknown",
                page_count=1,
                extracted_at=datetime.now()
            )
        )
    except Exception as e:
        logger.error(f"Docling processing error: {str(e)}")
        raise Exception(f"Document processing failed: {str(e)}")

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8000)
