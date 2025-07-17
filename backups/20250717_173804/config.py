#!/usr/bin/env python3
"""Configuration management system for CHONKER & SNYFTER"""

from pydantic import BaseModel, Field, validator
from typing import Optional, Dict, Any
from pathlib import Path
import os
import json
from enum import Enum

class LogLevel(str, Enum):
    DEBUG = "DEBUG"
    INFO = "INFO"
    WARNING = "WARNING"
    ERROR = "ERROR"
    CRITICAL = "CRITICAL"

class DatabaseConfig(BaseModel):
    """Database configuration settings"""
    path: Path = Field(default=Path("snyfter_archive.db"), description="Database file path")
    connection_pool_size: int = Field(default=5, ge=1, le=20, description="Connection pool size")
    enable_wal: bool = Field(default=True, description="Enable Write-Ahead Logging")
    timeout: float = Field(default=30.0, gt=0, description="Query timeout in seconds")
    
    @validator('path')
    def validate_path(cls, v):
        """Ensure database directory exists"""
        v = Path(v)
        v.parent.mkdir(parents=True, exist_ok=True)
        return v

class ProcessingConfig(BaseModel):
    """Document processing configuration"""
    max_file_size_mb: int = Field(default=500, ge=1, le=2000, description="Maximum file size in MB")
    max_processing_time_seconds: int = Field(default=300, ge=30, le=3600, description="Maximum processing time")
    lazy_loading_threshold_mb: int = Field(default=50, ge=10, le=500, description="File size threshold for lazy loading")
    chunk_size: int = Field(default=10, ge=1, le=100, description="Pages per chunk for lazy loading")
    enable_gpu: bool = Field(default=True, description="Enable GPU acceleration if available")
    
    @validator('max_file_size_mb')
    def validate_file_size(cls, v):
        """Convert MB to bytes"""
        return v * 1024 * 1024

class CacheConfig(BaseModel):
    """Caching configuration"""
    enabled: bool = Field(default=True, description="Enable document caching")
    cache_dir: Path = Field(default=Path.home() / '.chonker_cache', description="Cache directory")
    max_cache_size_mb: int = Field(default=500, ge=50, le=5000, description="Maximum cache size in MB")
    ttl_hours: int = Field(default=24, ge=1, le=168, description="Cache time-to-live in hours")
    
    @validator('cache_dir')
    def create_cache_dir(cls, v):
        """Ensure cache directory exists"""
        v = Path(v)
        v.mkdir(parents=True, exist_ok=True)
        return v

class LoggingConfig(BaseModel):
    """Logging configuration"""
    enabled: bool = Field(default=True, description="Enable structured logging")
    log_dir: Path = Field(default=Path.home() / '.chonker_logs', description="Log directory")
    log_level: LogLevel = Field(default=LogLevel.INFO, description="Logging level")
    max_log_size_mb: int = Field(default=10, ge=1, le=100, description="Maximum log file size")
    backup_count: int = Field(default=5, ge=1, le=20, description="Number of log backups to keep")
    log_to_console: bool = Field(default=False, description="Also log to console")
    
    @validator('log_dir')
    def create_log_dir(cls, v):
        """Ensure log directory exists"""
        v = Path(v)
        v.mkdir(parents=True, exist_ok=True)
        return v

class SecurityConfig(BaseModel):
    """Security configuration"""
    enable_input_validation: bool = Field(default=True, description="Enable input validation")
    enable_xss_protection: bool = Field(default=True, description="Enable XSS protection")
    allowed_file_extensions: list[str] = Field(default=[".pdf"], description="Allowed file extensions")
    enable_audit_logging: bool = Field(default=True, description="Enable audit trail")
    session_timeout_minutes: int = Field(default=30, ge=5, le=480, description="Session timeout")

class UIConfig(BaseModel):
    """User interface configuration"""
    theme: str = Field(default="dark", pattern="^(dark|light|auto)$", description="UI theme")
    window_width: int = Field(default=1400, ge=800, le=3840, description="Default window width")
    window_height: int = Field(default=900, ge=600, le=2160, description="Default window height")
    font_size: int = Field(default=12, ge=8, le=24, description="Base font size")
    show_emoji: bool = Field(default=True, description="Show emoji in UI")
    enable_animations: bool = Field(default=True, description="Enable UI animations")

class AppConfig(BaseModel):
    """Main application configuration"""
    app_name: str = Field(default="CHONKER & SNYFTER", description="Application name")
    version: str = Field(default="2.0.0", description="Application version")
    environment: str = Field(default="production", pattern="^(development|staging|production)$")
    debug_mode: bool = Field(default=False, description="Enable debug mode")
    
    # Sub-configurations
    database: DatabaseConfig = Field(default_factory=DatabaseConfig)
    processing: ProcessingConfig = Field(default_factory=ProcessingConfig)
    cache: CacheConfig = Field(default_factory=CacheConfig)
    logging: LoggingConfig = Field(default_factory=LoggingConfig)
    security: SecurityConfig = Field(default_factory=SecurityConfig)
    ui: UIConfig = Field(default_factory=UIConfig)
    
    class Config:
        env_file = '.env'
        env_file_encoding = 'utf-8'
        env_nested_delimiter = '__'  # Allows DATABASE__PATH in .env

class ConfigManager:
    """Configuration manager with file loading and validation"""
    
    def __init__(self, config_file: Optional[Path] = None, env_file: Optional[Path] = None):
        self.config_file = config_file or Path("config.json")
        self.env_file = env_file or Path(".env")
        self._config: Optional[AppConfig] = None
    
    def load(self) -> AppConfig:
        """Load configuration from files and environment"""
        config_data = {}
        
        # Load from JSON file if exists
        if self.config_file.exists():
            with open(self.config_file, 'r') as f:
                config_data = json.load(f)
        
        # Load from environment variables (overrides JSON)
        config_data = self._merge_env_vars(config_data)
        
        # Create and validate configuration
        self._config = AppConfig(**config_data)
        return self._config
    
    def save(self, config: Optional[AppConfig] = None):
        """Save configuration to file"""
        config = config or self._config
        if not config:
            raise ValueError("No configuration to save")
        
        with open(self.config_file, 'w') as f:
            json.dump(config.model_dump(), f, indent=2, default=str)
    
    def _merge_env_vars(self, config_data: Dict[str, Any]) -> Dict[str, Any]:
        """Merge environment variables into config data"""
        # Check for standard env vars
        env_mappings = {
            'CHONKER_DEBUG': ('debug_mode', bool),
            'CHONKER_ENV': ('environment', str),
            'CHONKER_DB_PATH': ('database.path', str),
            'CHONKER_LOG_LEVEL': ('logging.log_level', str),
            'CHONKER_MAX_FILE_SIZE': ('processing.max_file_size_mb', int),
        }
        
        for env_var, (config_path, type_func) in env_mappings.items():
            value = os.getenv(env_var)
            if value is not None:
                # Convert value to appropriate type
                if type_func == bool:
                    value = value.lower() in ('true', '1', 'yes', 'on')
                else:
                    value = type_func(value)
                
                # Set nested value
                self._set_nested(config_data, config_path, value)
        
        return config_data
    
    def _set_nested(self, data: Dict[str, Any], path: str, value: Any):
        """Set a nested dictionary value using dot notation"""
        keys = path.split('.')
        current = data
        
        for key in keys[:-1]:
            if key not in current:
                current[key] = {}
            current = current[key]
        
        current[keys[-1]] = value
    
    def generate_example_files(self):
        """Generate example configuration files"""
        # Generate example config.json
        example_config = AppConfig()
        with open("config.example.json", 'w') as f:
            json.dump(example_config.model_dump(), f, indent=2, default=str)
        
        # Generate example .env file
        env_content = """# CHONKER & SNYFTER Configuration
# Copy this file to .env and modify as needed

# Application settings
CHONKER_ENV=production
CHONKER_DEBUG=false

# Database settings
CHONKER_DB_PATH=./snyfter_archive.db

# Processing settings
CHONKER_MAX_FILE_SIZE=500

# Logging settings
CHONKER_LOG_LEVEL=INFO

# Advanced settings (use with caution)
# DATABASE__CONNECTION_POOL_SIZE=10
# PROCESSING__ENABLE_GPU=true
# CACHE__ENABLED=true
# SECURITY__ENABLE_AUDIT_LOGGING=true
"""
        with open(".env.example", 'w') as f:
            f.write(env_content)
        
        # Generate JSON schema
        with open("config_schema.json", 'w') as f:
            json.dump(AppConfig.model_json_schema(), f, indent=2)

# Global configuration instance
_config_manager = ConfigManager()
config = _config_manager.load()

if __name__ == "__main__":
    # Generate example files
    manager = ConfigManager()
    manager.generate_example_files()
    
    # Load and display configuration
    config = manager.load()
    print("Configuration loaded successfully!")
    print(f"Environment: {config.environment}")
    print(f"Debug mode: {config.debug_mode}")
    print(f"Database path: {config.database.path}")
    print(f"Max file size: {config.processing.max_file_size_mb / (1024*1024)}MB")
    
    # Save configuration
    manager.save(config)
    print("\nConfiguration saved to config.json")


def extend_database_for_snyfter(self):
    """Extend DocumentDatabase with SNYFTER tables"""
    
    # Add new tables for qualitative coding
    snyfter_schema = """
    CREATE TABLE IF NOT EXISTS codes (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL,
        description TEXT,
        color TEXT DEFAULT '#1ABC9C',
        parent_id INTEGER,
        created_date TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        FOREIGN KEY (parent_id) REFERENCES codes(id)
    );
    
    CREATE TABLE IF NOT EXISTS annotations (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        document_id TEXT NOT NULL,
        start_pos INTEGER NOT NULL,
        end_pos INTEGER NOT NULL,
        highlighted_text TEXT,
        code_id INTEGER,
        memo TEXT,
        created_date TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        FOREIGN KEY (document_id) REFERENCES documents(id),
        FOREIGN KEY (code_id) REFERENCES codes(id)
    );
    
    CREATE INDEX idx_codes_parent ON codes(parent_id);
    CREATE INDEX idx_annotations_doc ON annotations(document_id);
    CREATE INDEX idx_annotations_code ON annotations(code_id);
    """
    
    with self.get_connection() as conn:
        conn.executescript(snyfter_schema)
        conn.commit()
