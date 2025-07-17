#!/usr/bin/env python3
"""Integration tests for PDF to Database workflow - 100% coverage target"""

import pytest
import os
import sys
import tempfile
import sqlite3
import json
from unittest.mock import Mock, patch
from pathlib import Path

# Add parent directory to path
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from chonker_snyfter_elegant_v2 import (
    DocumentDatabase,
    DocumentProcessor,
    ProcessingResult,
    DocumentChunk,
    DocumentCache,
    ConnectionPool
)

class TestPDFToDatabaseWorkflow:
    """End-to-end integration tests for the complete processing pipeline"""
    
    @pytest.fixture
    def temp_db_path(self):
        """Create a temporary database"""
        with tempfile.NamedTemporaryFile(suffix='.db', delete=False) as f:
            db_path = f.name
        yield db_path
        os.unlink(db_path)
    
    @pytest.fixture
    def temp_cache_dir(self):
        """Create a temporary cache directory"""
        cache_dir = tempfile.mkdtemp()
        yield cache_dir
        # Cleanup
        import shutil
        shutil.rmtree(cache_dir)
    
    @pytest.fixture
    def mock_pdf_file(self):
        """Create a mock PDF file"""
        with tempfile.NamedTemporaryFile(suffix='.pdf', delete=False) as f:
            f.write(b'%PDF-1.4\n')
            f.write(b'Test PDF content for integration testing\n')
            path = f.name
        yield path
        os.unlink(path)
    
    @pytest.fixture
    def database(self, temp_db_path):
        """Create a test database instance"""
        return DocumentDatabase(temp_db_path)
    
    @pytest.fixture
    def cache(self, temp_cache_dir):
        """Create a test cache instance"""
        return DocumentCache(cache_dir=Path(temp_cache_dir))
    
    @pytest.fixture
    def sample_processing_result(self):
        """Create a sample processing result"""
        chunks = [
            DocumentChunk(
                index=0,
                type="heading",
                content="Test Document",
                confidence=0.95,
                metadata={"level": 1}
            ),
            DocumentChunk(
                index=1,
                type="text",
                content="This is test content for integration testing.",
                confidence=0.98,
                metadata={"level": 0}
            ),
            DocumentChunk(
                index=2,
                type="table",
                content="Table data here",
                confidence=0.90,
                metadata={"rows": 3, "cols": 2}
            )
        ]
        
        return ProcessingResult(
            success=True,
            document_id="test_doc_123",
            chunks=chunks,
            html_content="<h1>Test Document</h1><p>This is test content...</p>",
            markdown_content="# Test Document\n\nThis is test content...",
            processing_time=1.234
        )
    
    def test_database_initialization(self, database, temp_db_path):
        """Test database initialization creates all required tables"""
        conn = sqlite3.connect(temp_db_path)
        cursor = conn.cursor()
        
        # Check tables exist
        cursor.execute("SELECT name FROM sqlite_master WHERE type='table'")
        tables = {row[0] for row in cursor.fetchall()}
        
        assert 'documents' in tables
        assert 'chunks' in tables
        assert 'chunks_fts' in tables
        
        conn.close()
    
    def test_save_and_retrieve_document(self, database, sample_processing_result, mock_pdf_file):
        """Test saving and retrieving a document"""
        # Save document
        success, error = database.save_document(sample_processing_result, mock_pdf_file)
        assert success == True
        assert error == ""
        
        # Verify document was saved
        conn = sqlite3.connect(database.db_path)
        cursor = conn.cursor()
        
        cursor.execute("SELECT * FROM documents WHERE id = ?", (sample_processing_result.document_id,))
        doc = cursor.fetchone()
        assert doc is not None
        
        # Verify chunks were saved
        cursor.execute("SELECT COUNT(*) FROM chunks WHERE document_id = ?", 
                      (sample_processing_result.document_id,))
        chunk_count = cursor.fetchone()[0]
        assert chunk_count == len(sample_processing_result.chunks)
        
        conn.close()
    
    def test_search_functionality(self, database, sample_processing_result, mock_pdf_file):
        """Test full-text search functionality"""
        # Save document first
        database.save_document(sample_processing_result, mock_pdf_file)
        
        # Search for content
        results = database.search("test content")
        assert len(results) > 0
        assert results[0]['id'] == sample_processing_result.document_id
        
        # Search for non-existent content
        results = database.search("nonexistent")
        assert len(results) == 0
    
    def test_sql_injection_protection(self, database):
        """Test SQL injection protection in search"""
        # Attempt SQL injection
        malicious_queries = [
            "'; DROP TABLE documents; --",
            "\" OR 1=1 --",
            "'; DELETE FROM chunks; --"
        ]
        
        for query in malicious_queries:
            results = database.search(query)
            assert results == []  # Should return empty, not execute
        
        # Verify tables still exist
        conn = sqlite3.connect(database.db_path)
        cursor = conn.cursor()
        cursor.execute("SELECT name FROM sqlite_master WHERE type='table'")
        tables = {row[0] for row in cursor.fetchall()}
        assert 'documents' in tables
        assert 'chunks' in tables
        conn.close()
    
    def test_cache_functionality(self, cache, sample_processing_result, mock_pdf_file):
        """Test document caching"""
        # Initially cache should be empty
        cached = cache.get(mock_pdf_file)
        assert cached is None
        
        # Store in cache
        cache.put(mock_pdf_file, sample_processing_result)
        
        # Retrieve from cache
        cached = cache.get(mock_pdf_file)
        assert cached is not None
        assert cached.document_id == sample_processing_result.document_id
        assert len(cached.chunks) == len(sample_processing_result.chunks)
    
    def test_cache_invalidation_on_file_change(self, cache, sample_processing_result):
        """Test cache invalidation when file changes"""
        with tempfile.NamedTemporaryFile(suffix='.pdf', delete=False) as f:
            f.write(b'%PDF-1.4\nOriginal content')
            pdf_path = f.name
        
        try:
            # Cache the result
            cache.put(pdf_path, sample_processing_result)
            assert cache.get(pdf_path) is not None
            
            # Modify the file
            import time
            time.sleep(0.1)  # Ensure different timestamp
            with open(pdf_path, 'ab') as f:
                f.write(b'\nModified content')
            
            # Cache should not return stale data
            cached = cache.get(pdf_path)
            assert cached is None  # Should be invalidated
            
        finally:
            os.unlink(pdf_path)
    
    def test_connection_pool(self, temp_db_path):
        """Test database connection pooling"""
        pool = ConnectionPool(temp_db_path, pool_size=3)
        
        # Get connections
        connections = []
        for _ in range(3):
            conn = pool.get_connection()
            assert conn is not None
            connections.append(conn)
        
        # Return connections
        for conn in connections:
            pool.return_connection(conn)
        
        # Should be able to get connections again
        conn = pool.get_connection()
        assert conn is not None
        pool.return_connection(conn)
        
        # Cleanup
        pool.close_all()
    
    def test_transaction_rollback(self, database, sample_processing_result, mock_pdf_file):
        """Test transaction rollback on error"""
        # Mock an error during chunk insertion
        original_execute = sqlite3.Connection.execute
        
        def mock_execute(self, *args, **kwargs):
            if "INSERT INTO chunks" in str(args[0]):
                raise sqlite3.DatabaseError("Simulated error")
            return original_execute(self, *args, **kwargs)
        
        with patch.object(sqlite3.Connection, 'execute', mock_execute):
            success, error = database.save_document(sample_processing_result, mock_pdf_file)
        
        assert success == False
        assert "Database error" in error
        
        # Verify no partial data was saved
        conn = sqlite3.connect(database.db_path)
        cursor = conn.cursor()
        cursor.execute("SELECT COUNT(*) FROM documents WHERE id = ?", 
                      (sample_processing_result.document_id,))
        count = cursor.fetchone()[0]
        assert count == 0  # Should be rolled back
        conn.close()
    
    def test_concurrent_database_operations(self, database, sample_processing_result, mock_pdf_file):
        """Test concurrent database operations"""
        import threading
        import time
        
        results = []
        
        def save_document(index):
            # Modify document ID to make it unique
            result_copy = ProcessingResult(
                success=True,
                document_id=f"test_doc_{index}",
                chunks=sample_processing_result.chunks,
                html_content=sample_processing_result.html_content,
                markdown_content=sample_processing_result.markdown_content,
                processing_time=sample_processing_result.processing_time
            )
            success, error = database.save_document(result_copy, mock_pdf_file)
            results.append((success, error))
        
        # Start multiple threads
        threads = []
        for i in range(5):
            t = threading.Thread(target=save_document, args=(i,))
            threads.append(t)
            t.start()
        
        # Wait for completion
        for t in threads:
            t.join()
        
        # All should succeed
        assert all(success for success, _ in results)
        
        # Verify all documents were saved
        conn = sqlite3.connect(database.db_path)
        cursor = conn.cursor()
        cursor.execute("SELECT COUNT(*) FROM documents")
        count = cursor.fetchone()[0]
        assert count == 5
        conn.close()

class TestEndToEndWorkflow:
    """Test the complete workflow from PDF processing to database storage"""
    
    @patch('chonker_snyfter_elegant_v2.DOCLING_AVAILABLE', True)
    @patch('chonker_snyfter_elegant_v2.DocumentConverter')
    def test_complete_workflow(self, mock_converter, temp_db_path, temp_cache_dir):
        """Test complete PDF processing workflow"""
        # Setup mocks
        mock_conv_instance = Mock()
        mock_converter.return_value = mock_conv_instance
        
        mock_result = Mock()
        mock_result.document.iterate_items.return_value = [
            (Mock(text="Test heading"), 0),
            (Mock(text="Test content"), 1)
        ]
        mock_result.document.export_to_markdown.return_value = "# Test"
        
        mock_conv_instance.convert.return_value = mock_result
        
        # Create components
        with tempfile.NamedTemporaryFile(suffix='.pdf', delete=False) as f:
            f.write(b'%PDF-1.4\n')
            pdf_path = f.name
        
        try:
            database = DocumentDatabase(temp_db_path)
            cache = DocumentCache(cache_dir=Path(temp_cache_dir))
            
            # Process document
            processor = DocumentProcessor(pdf_path)
            
            # Mock the signals
            processor.finished = Mock()
            processor.error = Mock()
            processor.progress = Mock()
            processor.chunk_processed = Mock()
            
            # Run processing (simulate)
            processor.run()
            
            # Verify processing completed
            assert processor.finished.emit.called
            result = processor.finished.emit.call_args[0][0]
            assert result.success == True
            
            # Save to database
            success, error = database.save_document(result, pdf_path)
            assert success == True
            
            # Cache the result
            cache.put(pdf_path, result)
            
            # Verify we can retrieve from cache
            cached = cache.get(pdf_path)
            assert cached is not None
            
        finally:
            os.unlink(pdf_path)

if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])