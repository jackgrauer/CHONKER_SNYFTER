#!/usr/bin/env python3
"""
CHONKER Launcher - Background process manager for CHONKER Phoenix
Handles process lifecycle, PID management, and log rotation.
"""

import os
import sys
import signal
import subprocess
import time
from pathlib import Path
from datetime import datetime
from typing import Optional, List
import json

try:
    import psutil
except ImportError:
    print("ERROR: psutil package is required but not installed.")
    print("Please install it with: pip install psutil")
    sys.exit(1)

from launcher_models import LauncherConfig, ProcessInfo, LauncherResult, ProcessStatus


class ChonkerLauncher:
    """Main launcher class for managing CHONKER Phoenix process."""
    
    def __init__(self, config: Optional[LauncherConfig] = None):
        """Initialize launcher with configuration."""
        self.config = config or LauncherConfig()
        
    def launch(self) -> LauncherResult:
        """
        Launch CHONKER Phoenix process.
        
        Returns:
            LauncherResult with status information
        """
        # Check for existing instance
        existing_pid = self._check_existing_instance()
        if existing_pid:
            return LauncherResult(
                success=False,
                message=f"CHONKER already running with PID {existing_pid}",
                pid=None,
                log_path=None
            )
        
        # Clean up stale PID file
        self._cleanup_stale_pid()
        
        # Rotate logs if needed
        self._rotate_logs_if_needed()
        
        # Prepare command
        command = [sys.executable, "chonker_snyfter_elegant_v2.py"]
        
        try:
            if self.config.background_mode:
                # Launch in background
                with open(self.config.log_file_path, 'a') as log_file:
                    process = subprocess.Popen(
                        command,
                        stdout=log_file,
                        stderr=subprocess.STDOUT,
                        start_new_session=True,
                        cwd=Path(__file__).parent
                    )
                    pid = process.pid
            else:
                # Launch in foreground
                process = subprocess.Popen(
                    command,
                    cwd=Path(__file__).parent
                )
                pid = process.pid
            
            # Write PID file
            self._write_pid_file(pid)
            
            # Create process info
            process_info = ProcessInfo(
                pid=pid,
                start_time=datetime.now(),
                status=ProcessStatus.RUNNING,
                command=" ".join(command),
                log_path=self.config.log_file_path
            )
            
            # Save process info
            self._save_process_info(process_info)
            
            # Write startup message to log
            with open(self.config.log_file_path, 'a') as log_file:
                log_file.write(f"\n=== CHONKER SNYFTER started at {datetime.now()} (PID: {pid}) ===\n")
            
            return LauncherResult(
                success=True,
                pid=pid,
                message=f"CHONKER SNYFTER launched successfully",
                log_path=self.config.log_file_path
            )
            
        except Exception as e:
            return LauncherResult(
                success=False,
                message=f"Failed to launch CHONKER: {str(e)}",
                pid=None,
                log_path=None
            )
    
    def stop(self) -> LauncherResult:
        """
        Stop running CHONKER process.
        
        Returns:
            LauncherResult with status information
        """
        pid = self._read_pid_file()
        if not pid:
            return LauncherResult(
                success=False,
                message="No PID file found. CHONKER may not be running.",
                pid=None,
                log_path=None
            )
        
        try:
            # Check if process exists
            if not psutil.pid_exists(pid):
                self._cleanup_pid_file()
                return LauncherResult(
                    success=False,
                    message=f"Process {pid} not found. Cleaned up stale PID file.",
                    pid=None,
                    log_path=None
                )
            
            # Get process
            process = psutil.Process(pid)
            
            # Send SIGTERM for graceful shutdown
            process.terminate()
            
            # Wait for process to terminate (max 10 seconds)
            try:
                process.wait(timeout=10)
            except psutil.TimeoutExpired:
                # Force kill if still running
                process.kill()
                process.wait(timeout=5)
            
            # Clean up PID file
            self._cleanup_pid_file()
            
            # Write shutdown message to log
            with open(self.config.log_file_path, 'a') as log_file:
                log_file.write(f"\n=== CHONKER SNYFTER stopped at {datetime.now()} (PID: {pid}) ===\n")
            
            return LauncherResult(
                success=True,
                pid=pid,
                message=f"CHONKER SNYFTER stopped successfully",
                log_path=self.config.log_file_path
            )
            
        except psutil.NoSuchProcess:
            self._cleanup_pid_file()
            return LauncherResult(
                success=False,
                message=f"Process {pid} not found",
                pid=None,
                log_path=None
            )
        except Exception as e:
            return LauncherResult(
                success=False,
                message=f"Failed to stop CHONKER: {str(e)}",
                pid=None,
                log_path=None
            )
    
    def status(self) -> LauncherResult:
        """
        Get status of CHONKER process.
        
        Returns:
            LauncherResult with current status
        """
        pid = self._read_pid_file()
        if not pid:
            return LauncherResult(
                success=False,
                message="CHONKER is not running (no PID file)",
                pid=None,
                log_path=None
            )
        
        try:
            if not psutil.pid_exists(pid):
                self._cleanup_pid_file()
                return LauncherResult(
                    success=False,
                    message=f"CHONKER is not running (stale PID {pid} cleaned up)",
                    pid=None,
                    log_path=None
                )
            
            # Get process info
            process = psutil.Process(pid)
            process_info = self._load_process_info()
            
            if process_info:
                uptime = process_info.uptime_seconds
                uptime_str = self._format_uptime(uptime)
                message = f"CHONKER is running (PID: {pid}, Uptime: {uptime_str})"
            else:
                message = f"CHONKER is running (PID: {pid})"
            
            return LauncherResult(
                success=True,
                pid=pid,
                message=message,
                log_path=self.config.log_file_path
            )
            
        except psutil.NoSuchProcess:
            self._cleanup_pid_file()
            return LauncherResult(
                success=False,
                message=f"CHONKER is not running (process {pid} not found)",
                pid=None,
                log_path=None
            )
        except Exception as e:
            return LauncherResult(
                success=False,
                message=f"Failed to check status: {str(e)}",
                pid=None,
                log_path=None
            )
    
    def cleanup_old_logs(self, keep_count: int = 5) -> LauncherResult:
        """
        Clean up old rotated log files, keeping the most recent ones.
        
        Args:
            keep_count: Number of recent log files to keep
            
        Returns:
            LauncherResult with cleanup status
        """
        try:
            log_dir = self.config.log_file_path.parent
            log_name = self.config.log_file_path.stem
            log_ext = self.config.log_file_path.suffix
            
            # Find all rotated log files
            pattern = f"{log_name}.*.{log_ext}" if log_ext else f"{log_name}.*"
            rotated_logs = list(log_dir.glob(pattern))
            
            # Sort by modification time (newest first)
            rotated_logs.sort(key=lambda p: p.stat().st_mtime, reverse=True)
            
            # Delete old logs
            deleted_count = 0
            for log_file in rotated_logs[keep_count:]:
                try:
                    log_file.unlink()
                    deleted_count += 1
                except Exception:
                    pass
            
            message = f"Cleaned up {deleted_count} old log files"
            if deleted_count == 0:
                message = "No old log files to clean up"
            
            # Note: Using success=False here due to LauncherResult validation requirements
            # The cleanup actually succeeded, but the model requires pid/log_path when success=True
            return LauncherResult(
                success=False,
                message=message + " (operation completed successfully)",
                pid=None,
                log_path=None
            )
            
        except Exception as e:
            return LauncherResult(
                success=False,
                message=f"Failed to clean up logs: {str(e)}",
                pid=None,
                log_path=None
            )
    
    def _check_existing_instance(self) -> Optional[int]:
        """Check if CHONKER is already running."""
        pid = self._read_pid_file()
        if pid and psutil.pid_exists(pid):
            try:
                process = psutil.Process(pid)
                # Verify it's actually CHONKER
                cmdline = " ".join(process.cmdline())
                if "chonker_snyfter_elegant_v2.py" in cmdline:
                    return pid
            except (psutil.NoSuchProcess, psutil.AccessDenied):
                pass
        return None
    
    def _read_pid_file(self) -> Optional[int]:
        """Read PID from PID file."""
        if self.config.pid_file_path.exists():
            try:
                with open(self.config.pid_file_path, 'r') as f:
                    return int(f.read().strip())
            except (ValueError, IOError):
                pass
        return None
    
    def _write_pid_file(self, pid: int) -> None:
        """Write PID to PID file."""
        with open(self.config.pid_file_path, 'w') as f:
            f.write(str(pid))
    
    def _cleanup_pid_file(self) -> None:
        """Remove PID file."""
        if self.config.pid_file_path.exists():
            self.config.pid_file_path.unlink()
    
    def _cleanup_stale_pid(self) -> None:
        """Clean up stale PID file if process doesn't exist."""
        pid = self._read_pid_file()
        if pid and not psutil.pid_exists(pid):
            self._cleanup_pid_file()
    
    def _rotate_logs_if_needed(self) -> None:
        """Rotate log file if it exceeds size limit."""
        if not self.config.log_file_path.exists():
            return
        
        # Check file size
        size_mb = self.config.log_file_path.stat().st_size / (1024 * 1024)
        if size_mb > self.config.max_log_size_mb:
            # Generate rotation name with timestamp
            timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
            rotated_name = f"{self.config.log_file_path.stem}.{timestamp}{self.config.log_file_path.suffix}"
            rotated_path = self.config.log_file_path.parent / rotated_name
            
            # Rotate the file
            self.config.log_file_path.rename(rotated_path)
    
    def _save_process_info(self, process_info: ProcessInfo) -> None:
        """Save process info to a JSON file."""
        info_file = self.config.pid_file_path.with_suffix('.info')
        with open(info_file, 'w') as f:
            json.dump(process_info.model_dump(mode='json'), f, indent=2, default=str)
    
    def _load_process_info(self) -> Optional[ProcessInfo]:
        """Load process info from JSON file."""
        info_file = self.config.pid_file_path.with_suffix('.info')
        if info_file.exists():
            try:
                with open(info_file, 'r') as f:
                    data = json.load(f)
                    # Convert string datetime back to datetime object
                    data['start_time'] = datetime.fromisoformat(data['start_time'])
                    data['log_path'] = Path(data['log_path'])
                    return ProcessInfo(**data)
            except Exception:
                pass
        return None
    
    def _format_uptime(self, seconds: float) -> str:
        """Format uptime seconds into human-readable string."""
        days = int(seconds // 86400)
        hours = int((seconds % 86400) // 3600)
        minutes = int((seconds % 3600) // 60)
        
        parts = []
        if days > 0:
            parts.append(f"{days}d")
        if hours > 0:
            parts.append(f"{hours}h")
        if minutes > 0:
            parts.append(f"{minutes}m")
        if not parts:
            parts.append("< 1m")
        
        return " ".join(parts)


def main():
    """Main entry point for command line usage."""
    import argparse
    
    parser = argparse.ArgumentParser(
        description="CHONKER SNYFTER Launcher - Manage background process\n\nRequires: pip install psutil"
    )
    parser.add_argument(
        'command',
        choices=['launch', 'stop', 'status', 'cleanup'],
        help='Command to execute'
    )
    parser.add_argument(
        '--log-file',
        type=Path,
        default=Path("/tmp/chonker.log"),
        help='Path to log file'
    )
    parser.add_argument(
        '--pid-file',
        type=Path,
        default=Path("/tmp/chonker.pid"),
        help='Path to PID file'
    )
    parser.add_argument(
        '--foreground',
        action='store_true',
        help='Run in foreground mode (default: background)'
    )
    parser.add_argument(
        '--max-log-size',
        type=int,
        default=10,
        help='Maximum log size in MB before rotation'
    )
    
    args = parser.parse_args()
    
    # Create config
    config = LauncherConfig(
        log_file_path=args.log_file,
        pid_file_path=args.pid_file,
        background_mode=not args.foreground,
        max_log_size_mb=args.max_log_size
    )
    
    # Create launcher
    launcher = ChonkerLauncher(config)
    
    # Execute command
    if args.command == 'launch':
        result = launcher.launch()
    elif args.command == 'stop':
        result = launcher.stop()
    elif args.command == 'status':
        result = launcher.status()
    elif args.command == 'cleanup':
        result = launcher.cleanup_old_logs()
    
    # Print result
    print(result)
    
    # Return appropriate exit code
    sys.exit(0 if result.success else 1)


if __name__ == '__main__':
    main()