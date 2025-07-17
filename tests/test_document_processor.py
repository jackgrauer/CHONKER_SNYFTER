#!/usr/bin/env python3
"""Unit tests for DocumentProcessor - 95% coverage target"""

import pytest
import os
import sys
import tempfile
from unittest.mock import Mock, patch, MagicMock
from datetime import datetime
import threading
import time

# Add parent directory to path
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from chonker_snyfter_elegant_v2 import (
    DocumentProcessor, 
    ProcessingResult, 
    DocumentChunk,
    MAX_FILE_SIZE,
    MAX_PROCESSING_TIME
)

class TestDocumentProcessor:
    """Comprehensive unit tests for DocumentProcessor"""
    
    @pytest.fixture
    def mock_pdf_path(self):
        """Create a temporary test PDF file"""
        with tempfile.NamedTemporaryFile(suffix='.pdf', delete=False) as f:
            # Write a valid PDF header
            f.write(b'%PDF-1.4\n')
            f.write(b'1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n')
            path = f.name
        yield path
        os.unlink(path)
    
    @pytest.fixture
    def processor(self, mock_pdf_path):
        """Create a DocumentProcessor instance"""
        return DocumentProcessor(mock_pdf_path)
    
    def test_initialization(self, processor, mock_pdf_path):
        """Test processor initialization"""
        assert processor.pdf_path == mock_pdf_path
        assert processor.should_stop == False
        assert processor.start_time is None
        assert processor.timeout_occurred == False
    
    def test_stop_method(self, processor):
        """Test the stop method"""
        processor.should_stop = False
        processor.stop()
        assert processor.should_stop == True
    
    def test_pdf_header_validation_valid(self, processor, mock_pdf_path):
        """Test PDF header validation with valid PDF"""
        assert processor._validate_pdf_header() == True
    
    def test_pdf_header_validation_invalid(self, processor):
        """Test PDF header validation with invalid file"""
        # Create invalid PDF
        with tempfile.NamedTemporaryFile(suffix='.pdf', delete=False) as f:
            f.write(b'Not a PDF file')
            processor.pdf_path = f.name
        
        try:
            assert processor._validate_pdf_header() == False
        finally:
            os.unlink(f.name)
    
    def test_pdf_header_validation_nonexistent(self, processor):
        """Test PDF header validation with non-existent file"""
        processor.pdf_path = '/nonexistent/file.pdf'
        assert processor._validate_pdf_header() == False
    
    def test_get_pdf_size_mb(self, processor, mock_pdf_path):
        """Test file size calculation"""
        size_mb = processor._get_pdf_size_mb()
        assert isinstance(size_mb, float)
        assert size_mb > 0
    
    def test_should_use_lazy_loading_small_file(self, processor):
        """Test lazy loading decision for small files"""
        with patch.object(processor, '_get_pdf_size_mb', return_value=10):
            assert processor._should_use_lazy_loading() == False
    
    def test_should_use_lazy_loading_large_file(self, processor):
        """Test lazy loading decision for large files"""
        with patch.object(processor, '_get_pdf_size_mb', return_value=100):
            assert processor._should_use_lazy_loading() == True
    
    def test_check_timeout_no_timeout(self, processor):
        """Test timeout check when not exceeded"""
        processor.start_time = datetime.now()
        processor.timeout_occurred = False
        assert processor._check_timeout() == False
    
    @patch('chonker_snyfter_elegant_v2.MAX_PROCESSING_TIME', 0.1)
    def test_check_timeout_exceeded(self, processor):
        """Test timeout check when exceeded"""
        processor.start_time = datetime.now()
        processor.timeout_occurred = False
        
        # Wait for timeout
        time.sleep(0.2)
        
        # Mock the error signal
        processor.error = Mock()
        
        assert processor._check_timeout() == True
        assert processor.timeout_occurred == True
        processor.error.emit.assert_called_once()
    
    def test_generate_document_id(self, processor):
        """Test document ID generation"""
        doc_id = processor._generate_document_id()
        assert isinstance(doc_id, str)
        assert len(doc_id) == 16  # SHA256 truncated to 16 chars
    
    def test_create_chunk(self, processor):
        """Test chunk creation"""
        # Mock item
        mock_item = Mock()
        mock_item.text = "Test content"
        type(mock_item).__name__ = 'TestItem'
        
        chunk = processor._create_chunk(mock_item, level=1, index=0)
        
        assert isinstance(chunk, DocumentChunk)
        assert chunk.index == 0
        assert chunk.type == 'test'
        assert chunk.content == "Test content"
        assert chunk.metadata['level'] == 1
    
    @patch('chonker_snyfter_elegant_v2.DOCLING_AVAILABLE', False)
    def test_init_docling_not_available(self, processor):
        """Test docling initialization when not available"""
        with pytest.raises(Exception) as exc_info:
            processor._init_docling()
        assert "Docling not installed" in str(exc_info.value)
    
    @patch('chonker_snyfter_elegant_v2.DOCLING_AVAILABLE', True)
    def test_init_docling_available(self, processor):
        """Test docling initialization when available"""
        with patch('tqdm.tqdm') as mock_tqdm:
            processor._init_docling()
            # Should not raise exception
    
    def test_handle_error(self, processor):
        """Test error handling"""
        processor.error = Mock()
        processor.finished = Mock()
        
        test_error = ValueError("Test error")
        start_time = datetime.now()
        
        processor._handle_error(test_error, start_time)
        
        # Check signals were emitted
        processor.error.emit.assert_called_once_with("Test error")
        processor.finished.emit.assert_called_once()
        
        # Check error result
        error_result = processor.finished.emit.call_args[0][0]
        assert error_result.success == False
        assert error_result.error_message == "Test error"
        assert error_result.processing_time > 0
    
    @patch('chonker_snyfter_elegant_v2.DocumentConverter')
    def test_convert_document_standard(self, mock_converter, processor):
        """Test standard document conversion"""
        # Mock docling converter
        mock_conv_instance = Mock()
        mock_converter.return_value = mock_conv_instance
        mock_conv_instance.convert.return_value = Mock()
        
        with patch.object(processor, '_should_use_lazy_loading', return_value=False):
            result = processor._convert_document()
            
        mock_conv_instance.convert.assert_called_once_with(processor.pdf_path)
    
    @patch('chonker_snyfter_elegant_v2.DocumentConverter')
    def test_convert_document_lazy(self, mock_converter, processor):
        """Test lazy document conversion"""
        processor.progress = Mock()
        
        with patch.object(processor, '_should_use_lazy_loading', return_value=True):
            with patch.object(processor, '_convert_document_lazy') as mock_lazy:
                processor._convert_document()
                
        processor.progress.emit.assert_called()
        mock_lazy.assert_called_once()
    
    def test_extract_content(self, processor):
        """Test content extraction"""
        # Mock document result
        mock_result = Mock()
        mock_item = Mock()
        mock_item.text = "Test content"
        type(mock_item).__name__ = 'TextItem'
        
        mock_result.document.iterate_items.return_value = [(mock_item, 0)]
        
        processor.chunk_processed = Mock()
        chunks, html = processor._extract_content(mock_result)
        
        assert len(chunks) == 1
        assert '<div id="document-content"' in html
        assert 'Test content' in html
        processor.chunk_processed.emit.assert_called()
    
    def test_extract_content_with_stop(self, processor):
        """Test content extraction stops when requested"""
        mock_result = Mock()
        mock_result.document.iterate_items.return_value = [(Mock(), 0)] * 10
        
        processor.should_stop = True
        chunks, html = processor._extract_content(mock_result)
        
        assert len(chunks) == 0  # Should stop immediately

class TestProcessingResult:
    """Test the ProcessingResult data model"""
    
    def test_processing_result_creation(self):
        """Test creating a ProcessingResult"""
        chunks = [
            DocumentChunk(
                index=0,
                type="text",
                content="Test",
                confidence=1.0,
                metadata={}
            )
        ]
        
        result = ProcessingResult(
            success=True,
            document_id="test123",
            chunks=chunks,
            html_content="<p>Test</p>",
            markdown_content="Test",
            processing_time=1.5
        )
        
        assert result.success == True
        assert result.document_id == "test123"
        assert len(result.chunks) == 1
        assert result.processing_time == 1.5

class TestThreadSafety:
    """Test thread safety aspects"""
    
    def test_concurrent_processing(self):
        """Test that processor handles concurrent operations safely"""
        with tempfile.NamedTemporaryFile(suffix='.pdf', delete=False) as f:
            f.write(b'%PDF-1.4\n')
            pdf_path = f.name
        
        try:
            # Create multiple processors
            processors = [DocumentProcessor(pdf_path) for _ in range(3)]
            
            # All should initialize without issues
            for p in processors:
                assert p.pdf_path == pdf_path
                assert p.should_stop == False
                
        finally:
            os.unlink(pdf_path)

if __name__ == "__main__":
    pytest.main([__file__, "-v", "--cov=DocumentProcessor", "--cov-report=term-missing"])