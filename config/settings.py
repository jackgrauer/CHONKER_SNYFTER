#!/usr/bin/env python3
"""
Enhanced CHONKER settings using pydantic-settings for environment management.
Provides a clean interface for all configuration options.
"""

from pathlib import Path
from typing import List, Optional, Dict, Any
from pydantic import Field, field_validator, DirectoryPath, FilePath
from pydantic_settings import BaseSettings, SettingsConfigDict
from pydantic_extra_types.color import Color
from enum import Enum

class ProcessingMode(str, Enum):
    """Document processing modes"""
    FAST = "fast"
    ACCURATE = "accurate"
    BALANCED = "balanced"

class OutputFormat(str, Enum):
    """Supported output formats"""
    JSON = "json"
    CSV = "csv"
    SQLITE = "sqlite"
    MARKDOWN = "markdown"

class CHONKERSettings(BaseSettings):
    """Main CHONKER configuration settings"""
    
    model_config = SettingsConfigDict(
        env_file=".env",
        env_file_encoding="utf-8",
        case_sensitive=False,
        env_prefix="CHONKER_",
        extra="ignore",
        json_schema_extra={
            "example": {
                "app_name": "CHONKER",
                "chunk_size": 10,
                "max_file_size_mb": 500,
                "processing_mode": "balanced"
            }
        }
    )
    
    # Application metadata
    app_name: str = Field(default="CHONKER", description="Application name")
    version: str = Field(default="2.0.0", description="Application version")
    description: str = Field(
        default="Document chunking and table extraction system",
        description="Application description"
    )
    
    # Processing settings
    chunk_size: int = Field(
        default=10,
        ge=1,
        le=100,
        description="Number of pages per chunk"
    )
    max_file_size_mb: int = Field(
        default=500,
        ge=1,
        le=2000,
        description="Maximum file size in MB"
    )
    processing_mode: ProcessingMode = Field(
        default=ProcessingMode.BALANCED,
        description="Processing accuracy vs speed trade-off"
    )
    enable_ocr: bool = Field(
        default=True,
        description="Enable OCR for scanned documents"
    )
    ocr_languages: List[str] = Field(
        default=["eng"],
        description="OCR language codes (e.g., 'eng', 'fra', 'deu')"
    )
    
    # Output settings
    output_format: OutputFormat = Field(
        default=OutputFormat.JSON,
        description="Default output format"
    )
    output_dir: Path = Field(
        default=Path("output"),
        description="Output directory for results"
    )
    keep_intermediate_files: bool = Field(
        default=False,
        description="Keep intermediate processing files"
    )
    
    # Performance settings
    num_workers: int = Field(
        default=4,
        ge=1,
        le=16,
        description="Number of parallel workers"
    )
    gpu_enabled: bool = Field(
        default=True,
        description="Use GPU acceleration if available"
    )
    memory_limit_gb: float = Field(
        default=8.0,
        ge=1.0,
        le=64.0,
        description="Memory limit in GB"
    )
    
    # UI settings
    theme_color: Color = Field(
        default=Color("#00CED1"),
        description="Primary theme color"
    )
    show_progress: bool = Field(
        default=True,
        description="Show progress bars"
    )
    verbose: bool = Field(
        default=False,
        description="Enable verbose output"
    )
    
    # Advanced settings
    cache_enabled: bool = Field(
        default=True,
        description="Enable result caching"
    )
    cache_dir: Path = Field(
        default=Path.home() / ".chonker_cache",
        description="Cache directory"
    )
    log_level: str = Field(
        default="INFO",
        pattern="^(DEBUG|INFO|WARNING|ERROR|CRITICAL)$",
        description="Logging level"
    )
    
    @field_validator('output_dir', 'cache_dir')
    @classmethod
    def create_directories(cls, v: Path) -> Path:
        """Ensure directories exist"""
        v.mkdir(parents=True, exist_ok=True)
        return v
    
    @field_validator('ocr_languages')
    @classmethod
    def validate_languages(cls, v: List[str]) -> List[str]:
        """Validate OCR language codes"""
        valid_languages = {
            'eng', 'fra', 'deu', 'spa', 'ita', 'por', 'rus', 
            'jpn', 'chi_sim', 'chi_tra', 'ara', 'hin', 'kor'
        }
        for lang in v:
            if lang not in valid_languages:
                raise ValueError(f"Invalid language code: {lang}")
        return v
    
    def get_processing_params(self) -> Dict[str, Any]:
        """Get processing parameters based on mode"""
        params = {
            ProcessingMode.FAST: {
                "dpi": 150,
                "quality": 70,
                "accuracy": 0.7
            },
            ProcessingMode.ACCURATE: {
                "dpi": 300,
                "quality": 95,
                "accuracy": 0.95
            },
            ProcessingMode.BALANCED: {
                "dpi": 200,
                "quality": 85,
                "accuracy": 0.85
            }
        }
        return params[self.processing_mode]
    
    def to_display_dict(self) -> Dict[str, Any]:
        """Get settings for display (hiding sensitive info)"""
        data = self.model_dump()
        # Hide any sensitive fields if needed
        return data

# Singleton instance
settings = CHONKERSettings()

if __name__ == "__main__":
    # Test settings loading
    from rich import print as rprint
    from rich.panel import Panel
    from rich.table import Table
    
    # Display current settings
    table = Table(title="CHONKER Settings", show_header=True)
    table.add_column("Setting", style="cyan")
    table.add_column("Value", style="green")
    table.add_column("Type", style="yellow")
    
    for field_name, field_value in settings.model_dump().items():
        table.add_row(
            field_name,
            str(field_value),
            type(field_value).__name__
        )
    
    rprint(Panel(table, title="Current Configuration", border_style="blue"))
    
    # Show processing params
    params = settings.get_processing_params()
    rprint(f"\nProcessing parameters for {settings.processing_mode} mode:")
    rprint(params)