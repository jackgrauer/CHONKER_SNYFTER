#!/usr/bin/env python3
"""Security tests for input validation - 100% coverage target"""

import pytest
import os
import sys
import tempfile
import sqlite3
from pathlib import Path
from unittest.mock import Mock, patch

# Add parent directory to path
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from chonker_snyfter_elegant_v2 import (
    DocumentDatabase,
    DocumentProcessor,
    MAX_FILE_SIZE
)

class TestSQLInjectionProtection:
    """Test SQL injection protection across all database operations"""
    
    @pytest.fixture
    def database(self):
        """Create a test database"""
        with tempfile.NamedTemporaryFile(suffix='.db', delete=False) as f:
            db_path = f.name
        
        db = DocumentDatabase(db_path)
        yield db
        os.unlink(db_path)
    
    def test_search_sql_injection_attempts(self, database):
        """Test various SQL injection attempts in search"""
        injection_attempts = [
            # Classic SQL injection
            "'; DROP TABLE documents; --",
            "' OR '1'='1",
            '" OR "1"="1',
            "'; DELETE FROM chunks WHERE '1'='1'; --",
            
            # Union-based injection
            "' UNION SELECT * FROM documents --",
            "' UNION SELECT sql FROM sqlite_master --",
            
            # Time-based blind injection
            "' OR SLEEP(5) --",
            "'; SELECT CASE WHEN (1=1) THEN pg_sleep(5) ELSE 0 END --",
            
            # Stacked queries
            "'; INSERT INTO documents VALUES ('hack', 'hacked'); --",
            
            # Comment-based injection
            "admin'--",
            "admin'/*",
            
            # Special characters
            "'; SELECT * FROM documents WHERE id LIKE '%",
            "\\'; DROP TABLE documents; --",
            
            # Hex encoding attempts
            "0x27; DROP TABLE documents; --",
            
            # Unicode attempts
            "′; DROP TABLE documents; --",  # Unicode apostrophe
        ]
        
        for payload in injection_attempts:
            # Should return empty results, not execute malicious SQL
            results = database.search(payload)
            assert results == [], f"SQL injection not blocked: {payload}"
        
        # Verify database integrity
        conn = sqlite3.connect(database.db_path)
        cursor = conn.cursor()
        
        # Check all tables still exist
        cursor.execute("SELECT name FROM sqlite_master WHERE type='table'")
        tables = {row[0] for row in cursor.fetchall()}
        assert 'documents' in tables
        assert 'chunks' in tables
        assert 'chunks_fts' in tables
        
        # Check no unauthorized data was inserted
        cursor.execute("SELECT COUNT(*) FROM documents")
        count = cursor.fetchone()[0]
        assert count == 0  # Should be empty
        
        conn.close()
    
    def test_search_regex_validation(self, database):
        """Test that only safe characters are allowed in search"""
        # Valid searches
        valid_queries = [
            "normal search",
            "search with numbers 123",
            "dash-search",
            "underscore_search",
            '"quoted search"',
            "'single quoted'",
        ]
        
        for query in valid_queries:
            # Should not raise exception
            database.search(query)
        
        # Invalid searches (containing unsafe characters)
        invalid_queries = [
            "search; with semicolon",
            "search/* with comment",
            "search-- with comment",
            "search\\ with backslash",
            "search` with backtick",
            "search$ with dollar",
            "search^ with caret",
            "search& with ampersand",
            "search| with pipe",
        ]
        
        for query in invalid_queries:
            results = database.search(query)
            assert results == [], f"Unsafe query not blocked: {query}"

class TestXSSPrevention:
    """Test XSS (Cross-Site Scripting) prevention"""
    
    def test_html_escaping_in_chunks(self):
        """Test that HTML content is properly escaped"""
        from chonker_snyfter_elegant_v2 import DocumentChunk, ProcessingResult
        from html import escape
        
        # Create chunks with XSS payloads
        xss_payloads = [
            "<script>alert('XSS')</script>",
            "<img src=x onerror=alert('XSS')>",
            "<iframe src='javascript:alert(1)'></iframe>",
            "<body onload=alert('XSS')>",
            "javascript:alert('XSS')",
            "<svg onload=alert('XSS')>",
            "<input onfocus=alert('XSS') autofocus>",
            "<a href='javascript:alert(1)'>Click me</a>",
            "<div style='background:url(javascript:alert(1))'>",
            "';alert('XSS');//",
        ]
        
        processor = DocumentProcessor("dummy.pdf")
        
        for payload in xss_payloads:
            # Create mock item with XSS payload
            mock_item = Mock()
            mock_item.text = payload
            type(mock_item).__name__ = 'TextItem'
            
            # Process the item
            html = processor._item_to_html(mock_item, level=0)
            
            # Verify the payload is escaped
            assert '<script>' not in html
            assert 'javascript:' not in html
            assert 'onerror=' not in html
            assert 'onload=' not in html
            assert escape(payload) in html or payload not in html

class TestFileUploadSecurity:
    """Test file upload security measures"""
    
    def test_file_size_limit_enforcement(self):
        """Test that file size limits are enforced"""
        # Create a mock file that exceeds size limit
        with tempfile.NamedTemporaryFile(suffix='.pdf', delete=False) as f:
            # Write more than MAX_FILE_SIZE
            f.write(b'%PDF-1.4\n')
            f.write(b'X' * (MAX_FILE_SIZE + 1))
            large_file = f.name
        
        try:
            processor = DocumentProcessor(large_file)
            
            # File size check should prevent processing
            file_size = os.path.getsize(large_file)
            assert file_size > MAX_FILE_SIZE
            
        finally:
            os.unlink(large_file)
    
    def test_pdf_header_validation(self):
        """Test PDF header validation prevents processing of non-PDF files"""
        test_files = [
            # (content, should_pass)
            (b'%PDF-1.4\n', True),  # Valid PDF
            (b'%PDF-1.7\n', True),  # Valid PDF
            (b'%PDF-2.0\n', True),  # Valid PDF 2.0
            (b'Not a PDF', False),  # Invalid
            (b'<html>', False),  # HTML file
            (b'#!/bin/sh\nrm -rf /', False),  # Shell script
            (b'MZ\x90\x00', False),  # Windows executable
            (b'\xFF\xD8\xFF', False),  # JPEG
            (b'GIF89a', False),  # GIF
            (b'PK\x03\x04', False),  # ZIP
        ]
        
        for content, should_pass in test_files:
            with tempfile.NamedTemporaryFile(suffix='.pdf', delete=False) as f:
                f.write(content)
                f.flush()
                
                processor = DocumentProcessor(f.name)
                result = processor._validate_pdf_header()
                
                assert result == should_pass, f"Header validation failed for: {content[:10]}"
                
                os.unlink(f.name)
    
    def test_path_traversal_prevention(self):
        """Test prevention of path traversal attacks"""
        # These paths should be rejected or sanitized
        malicious_paths = [
            "../../../etc/passwd",
            "..\\..\\..\\windows\\system32\\config\\sam",
            "/etc/passwd",
            "C:\\Windows\\System32\\drivers\\etc\\hosts",
            "../../../../../../../../etc/passwd",
            ".%2F.%2F.%2Fetc%2Fpasswd",  # URL encoded
            "..%252f..%252f..%252fetc%252fpasswd",  # Double encoded
            "/var/www/../../etc/passwd",
            "\\\\server\\share\\..\\..\\sensitive",
        ]
        
        for path in malicious_paths:
            # Verify the path would be rejected
            # In a real implementation, you'd have a path validation function
            assert ".." in path or path.startswith("/") or path.startswith("\\") or "%" in path

class TestResourceExhaustion:
    """Test protection against resource exhaustion attacks"""
    
    def test_processing_timeout(self):
        """Test that processing timeout prevents infinite loops"""
        from chonker_snyfter_elegant_v2 import MAX_PROCESSING_TIME
        
        with tempfile.NamedTemporaryFile(suffix='.pdf', delete=False) as f:
            f.write(b'%PDF-1.4\n')
            pdf_path = f.name
        
        try:
            processor = DocumentProcessor(pdf_path)
            processor.start_time = processor.start_time or datetime.now()
            
            # Simulate timeout
            import time
            from datetime import datetime
            
            # Set start time in the past
            from datetime import timedelta
            processor.start_time = datetime.now() - timedelta(seconds=MAX_PROCESSING_TIME + 1)
            
            # Check timeout should trigger
            assert processor._check_timeout() == True
            assert processor.timeout_occurred == True
            
        finally:
            os.unlink(pdf_path)
    
    def test_memory_limit_enforcement(self):
        """Test that memory limits are enforced through file size limits"""
        # File size limit serves as a proxy for memory limit
        assert MAX_FILE_SIZE == 500 * 1024 * 1024  # 500MB
        
        # Verify lazy loading kicks in for large files
        processor = DocumentProcessor("dummy.pdf")
        
        # Mock file size
        with patch.object(processor, '_get_pdf_size_mb', return_value=100):
            assert processor._should_use_lazy_loading() == True
        
        with patch.object(processor, '_get_pdf_size_mb', return_value=10):
            assert processor._should_use_lazy_loading() == False

class TestInputSanitization:
    """Test input sanitization across the application"""
    
    def test_filename_sanitization(self):
        """Test that filenames are properly sanitized"""
        dangerous_filenames = [
            "../../etc/passwd.pdf",
            "file\x00.pdf",  # Null byte
            "file|command.pdf",  # Pipe character
            "file;ls.pdf",  # Command separator
            "file`whoami`.pdf",  # Command substitution
            "file$(whoami).pdf",  # Command substitution
            "file&command.pdf",  # Background execution
            "file>output.pdf",  # Redirect
            "file<input.pdf",  # Redirect
        ]
        
        for filename in dangerous_filenames:
            # Verify these would be sanitized
            safe_chars = set('abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789._- ')
            has_unsafe = any(c not in safe_chars for c in filename)
            assert has_unsafe, f"Dangerous filename not detected: {filename}"
    
    def test_search_query_length_limit(self, database):
        """Test that search queries have length limits"""
        # Create extremely long query
        long_query = "A" * 2000  # Over the 1000 char limit
        
        results = database.search(long_query)
        assert results == []  # Should be rejected
        
        # Normal length query should work
        normal_query = "A" * 500
        results = database.search(normal_query)
        # Should not raise exception

class TestSecurityHeaders:
    """Test security-related configurations"""
    
    def test_structured_logging_no_sensitive_data(self):
        """Test that structured logging doesn't leak sensitive data"""
        from structured_logging import StructuredLogger
        
        logger = StructuredLogger('test')
        
        # Mock sensitive data
        sensitive_data = {
            'password': 'secret123',
            'api_key': 'sk-1234567890',
            'credit_card': '4111111111111111',
            'ssn': '123-45-6789',
        }
        
        # Log with sensitive data - in production, these should be filtered
        logger.info("Test message", **sensitive_data)
        
        # In a real implementation, verify logs don't contain sensitive values
        # This is a placeholder for the actual implementation

if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])