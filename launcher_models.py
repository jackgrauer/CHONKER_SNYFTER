"""
Pydantic models for CHONKER launcher system.

This module provides data models for managing background process launching
and monitoring, designed to avoid bash timeout issues.
"""

import os
from datetime import datetime
from enum import Enum
from pathlib import Path
from typing import Optional

from pydantic import BaseModel, Field, field_validator, ConfigDict


class ProcessStatus(str, Enum):
    """Enum representing the possible states of a launched process."""
    RUNNING = "RUNNING"
    STOPPED = "STOPPED"
    CRASHED = "CRASHED"


class LauncherConfig(BaseModel):
    """
    Configuration model for the CHONKER launcher.
    
    This model defines the configuration parameters for launching
    and managing background processes, including logging, process
    management, and restart behavior.
    """
    model_config = ConfigDict(
        str_strip_whitespace=True,
        validate_assignment=True,
        extra="forbid"
    )
    
    log_file_path: Path = Field(
        default=Path("/tmp/chonker.log"),
        description="Path to the log file where process output will be written"
    )
    
    pid_file_path: Path = Field(
        default=Path("/tmp/chonker.pid"),
        description="Path to the PID file for tracking the process ID"
    )
    
    background_mode: bool = Field(
        default=True,
        description="Whether to run the process in background mode (detached)"
    )
    
    auto_restart: bool = Field(
        default=False,
        description="Whether to automatically restart the process if it crashes"
    )
    
    max_log_size_mb: int = Field(
        default=10,
        gt=0,
        le=1000,
        description="Maximum size of the log file in megabytes before rotation"
    )
    
    @field_validator('log_file_path', 'pid_file_path')
    @classmethod
    def validate_path_parent_exists(cls, v: Path) -> Path:
        """Ensure the parent directory of the path exists or can be created."""
        parent = v.parent
        if not parent.exists():
            try:
                parent.mkdir(parents=True, exist_ok=True)
            except PermissionError:
                raise ValueError(
                    f"Cannot create parent directory for path: {v}. "
                    "Please ensure you have write permissions."
                )
        return v
    
    @field_validator('log_file_path', 'pid_file_path')
    @classmethod
    def validate_path_writable(cls, v: Path) -> Path:
        """Ensure the path is writable."""
        # If file exists, check if it's writable
        if v.exists() and not v.is_file():
            raise ValueError(f"Path exists but is not a file: {v}")
        
        # Check if parent directory is writable
        parent = v.parent
        if not os.access(parent, os.W_OK):
            raise ValueError(
                f"No write permission for directory: {parent}. "
                "Please choose a writable location."
            )
        return v


class ProcessInfo(BaseModel):
    """
    Information about a launched process.
    
    This model tracks the state and metadata of a process launched
    by the CHONKER launcher, including its PID, status, and associated
    log files.
    """
    model_config = ConfigDict(
        str_strip_whitespace=True,
        validate_assignment=True,
        extra="forbid"
    )
    
    pid: int = Field(
        ...,
        gt=0,
        description="Process ID of the launched process"
    )
    
    start_time: datetime = Field(
        ...,
        description="Timestamp when the process was started"
    )
    
    status: ProcessStatus = Field(
        ...,
        description="Current status of the process"
    )
    
    command: str = Field(
        ...,
        min_length=1,
        description="The command that was executed to start the process"
    )
    
    log_path: Path = Field(
        ...,
        description="Path to the log file containing process output"
    )
    
    @field_validator('log_path')
    @classmethod
    def validate_log_path_exists(cls, v: Path) -> Path:
        """Ensure the log path exists and is a file."""
        if not v.exists():
            raise ValueError(f"Log file does not exist: {v}")
        if not v.is_file():
            raise ValueError(f"Log path is not a file: {v}")
        return v
    
    @property
    def uptime_seconds(self) -> float:
        """Calculate the uptime of the process in seconds."""
        if self.status == ProcessStatus.RUNNING:
            return (datetime.now() - self.start_time).total_seconds()
        return 0.0
    
    @property
    def is_running(self) -> bool:
        """Check if the process is currently running."""
        return self.status == ProcessStatus.RUNNING


class LauncherResult(BaseModel):
    """
    Result model for launcher operations.
    
    This model encapsulates the result of a launcher operation,
    including success status, process information, and any relevant
    messages or errors.
    """
    model_config = ConfigDict(
        str_strip_whitespace=True,
        validate_assignment=True,
        extra="forbid"
    )
    
    success: bool = Field(
        ...,
        description="Whether the launcher operation was successful"
    )
    
    pid: Optional[int] = Field(
        None,
        gt=0,
        description="Process ID if the launch was successful"
    )
    
    message: str = Field(
        ...,
        min_length=1,
        description="Descriptive message about the operation result"
    )
    
    log_path: Optional[Path] = Field(
        None,
        description="Path to the log file if process was launched"
    )
    
    @field_validator('pid')
    @classmethod
    def validate_pid_with_success(cls, v: Optional[int], info) -> Optional[int]:
        """Ensure PID is provided when success is True."""
        if info.data.get('success') and v is None:
            raise ValueError(
                "PID must be provided when success is True"
            )
        if not info.data.get('success') and v is not None:
            raise ValueError(
                "PID should not be provided when success is False"
            )
        return v
    
    @field_validator('log_path')
    @classmethod
    def validate_log_path_with_success(cls, v: Optional[Path], info) -> Optional[Path]:
        """Ensure log_path is provided when success is True."""
        if info.data.get('success') and v is None:
            raise ValueError(
                "log_path must be provided when success is True"
            )
        if v is not None and not v.exists():
            raise ValueError(f"Log file does not exist: {v}")
        return v
    
    @property
    def is_successful(self) -> bool:
        """Convenience property to check if the operation was successful."""
        return self.success
    
    def __str__(self) -> str:
        """String representation of the launcher result."""
        if self.success:
            return f"Success: {self.message} (PID: {self.pid}, Log: {self.log_path})"
        return f"Failed: {self.message}"


