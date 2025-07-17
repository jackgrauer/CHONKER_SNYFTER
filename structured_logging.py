#!/usr/bin/env python3
"""Structured logging system for CHONKER & SNYFTER"""

import logging
import json
from logging.handlers import RotatingFileHandler
from datetime import datetime
from pathlib import Path
from typing import Dict, Any, Optional

class JSONFormatter(logging.Formatter):
    """JSON formatter for structured logging"""
    
    def format(self, record: logging.LogRecord) -> str:
        log_obj = {
            'timestamp': datetime.now().isoformat(),
            'level': record.levelname,
            'component': record.name,
            'message': record.getMessage(),
            'module': record.module,
            'function': record.funcName,
            'line': record.lineno,
        }
        
        # Add extra fields if present
        if hasattr(record, 'extra'):
            log_obj['extra'] = record.extra
            
        # Add exception info if present
        if record.exc_info:
            log_obj['exception'] = self.formatException(record.exc_info)
            
        return json.dumps(log_obj)

class StructuredLogger:
    """Structured logging system with JSON output and rotation"""
    
    def __init__(self, name: str, log_dir: Optional[Path] = None):
        self.logger = logging.getLogger(name)
        self.logger.setLevel(logging.INFO)
        
        # Prevent duplicate handlers
        if self.logger.handlers:
            return
            
        # Set up log directory
        log_dir = log_dir or Path.home() / '.chonker_logs'
        log_dir.mkdir(exist_ok=True)
        
        # File handler with rotation
        file_handler = RotatingFileHandler(
            log_dir / 'chonker_snyfter.log',
            maxBytes=10*1024*1024,  # 10MB
            backupCount=5
        )
        file_handler.setFormatter(JSONFormatter())
        self.logger.addHandler(file_handler)
        
        # Console handler for errors
        console_handler = logging.StreamHandler()
        console_handler.setLevel(logging.ERROR)
        console_handler.setFormatter(logging.Formatter(
            '%(asctime)s - %(name)s - %(levelname)s - %(message)s'
        ))
        self.logger.addHandler(console_handler)
    
    def info(self, message: str, **kwargs):
        """Log info with optional extra data"""
        self.logger.info(message, extra={'extra': kwargs} if kwargs else {})
    
    def warning(self, message: str, **kwargs):
        """Log warning with optional extra data"""
        self.logger.warning(message, extra={'extra': kwargs} if kwargs else {})
    
    def error(self, message: str, **kwargs):
        """Log error with optional extra data"""
        self.logger.error(message, extra={'extra': kwargs} if kwargs else {})
    
    def debug(self, message: str, **kwargs):
        """Log debug with optional extra data"""
        self.logger.debug(message, extra={'extra': kwargs} if kwargs else {})
    
    def log_event(self, event_type: str, data: Dict[str, Any]):
        """Log a structured event"""
        self.info(f"Event: {event_type}", event_type=event_type, data=data)
    
    def log_performance(self, operation: str, duration: float, **metrics):
        """Log performance metrics"""
        self.info(
            f"Performance: {operation} took {duration:.2f}s",
            operation=operation,
            duration=duration,
            metrics=metrics
        )
    
    def log_error_with_context(self, error: Exception, context: Dict[str, Any]):
        """Log error with full context"""
        self.logger.error(
            f"Error: {type(error).__name__}: {str(error)}",
            exc_info=True,
            extra={'extra': {
                'error_type': type(error).__name__,
                'error_message': str(error),
                'context': context
            }}
        )

# Global logger instances
app_logger = StructuredLogger('chonker.app')
db_logger = StructuredLogger('chonker.database')
processing_logger = StructuredLogger('chonker.processing')
ui_logger = StructuredLogger('chonker.ui')

if __name__ == "__main__":
    # Test the logging system
    app_logger.info("Application started", version="2.0", mode="test")
    
    app_logger.log_event("pdf_opened", {
        'file': 'test.pdf',
        'size_mb': 25.3
    })
    
    app_logger.log_performance("pdf_processing", 45.2, 
        chunks_processed=150,
        memory_mb=256
    )
    
    try:
        raise ValueError("Test error")
    except Exception as e:
        app_logger.log_error_with_context(e, {
            'operation': 'test',
            'file': 'test.pdf'
        })
    
    print("Structured logging test complete - check ~/.chonker_logs/")