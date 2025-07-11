# CHONKER Document Processing Service

FastAPI service that provides document processing capabilities using Docling.

## Features

- **PDF Processing**: Extract text, tables, and metadata from PDF documents
- **FastAPI**: Modern, fast Python web framework
- **CORS Support**: Configured for Tauri frontend integration
- **Mock Processing**: Fallback when Docling is not available
- **Type Safety**: Full Pydantic models for request/response validation

## API Endpoints

### `GET /`
Service status and information

### `GET /health`
Health check endpoint

### `POST /process`
Process a document file
- **Input**: Multipart form with file upload
- **Output**: JSON with extracted text, tables, and metadata

## Setup

```bash
# Create virtual environment
python3 -m venv venv
source venv/bin/activate

# Install dependencies
pip install -r requirements.txt

# Run development server
python -m uvicorn main:app --reload --host 0.0.0.0 --port 8000
```

## Integration with Turborepo

This service is integrated with the CHONKER SNYFTER Turborepo:

```bash
# From root directory
npm run dev          # Starts all services including doc-service
```

## Usage with Tauri

The Tauri app automatically calls this service at `http://localhost:8000/process` when processing documents.

## Development

- **Mock Mode**: If Docling is not installed, the service runs in mock mode
- **CORS**: Configured for Tauri origins
- **Logging**: INFO level logging for debugging
- **Error Handling**: Graceful fallbacks and proper HTTP status codes
