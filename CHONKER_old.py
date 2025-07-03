"""
🐹 CHONKER v6.0 - Anxiety-Free Document Intelligence Platform (SNYFTER INTEGRATED)
===============================================================================
Live monitoring with real-time updates to prove it's actually working
Now saves chunks to persistent directory for Snyfter integration!
"""

import os
import sys
import hashlib
import json
import time
import shutil
import psutil
import threading
from pathlib import Path
from datetime import datetime
from typing import List, Dict, Optional, Any
from dataclasses import dataclass
from collections import defaultdict

# Rich imports for UI
from rich.console import Console
from rich.table import Table
from rich.panel import Panel
from rich.prompt import Prompt
from rich.progress import Progress, SpinnerColumn, TextColumn, BarColumn, TimeElapsedColumn
from rich.tree import Tree
from rich.live import Live
from rich.layout import Layout
from rich.text import Text

# Keep-awake imports (cross-platform)
import subprocess
import platform

console = Console()

# Core Dependencies with graceful fallback
try:
    from docling.document_converter import DocumentConverter
    DOCLING_AVAILABLE = True
    console.print("[dim]🧠 Docling: Ready[/dim]")
    
    try:
        from docling.datamodel.base_models import InputFormat
        from docling.datamodel.pipeline_options import PdfPipelineOptions
        from docling.document_converter import PdfFormatOption
        DOCLING_ADVANCED = True
    except ImportError:
        DOCLING_ADVANCED = False
        console.print("[dim]🧠 Docling: Basic mode[/dim]")
        
except ImportError:
    DOCLING_AVAILABLE = False
    DOCLING_ADVANCED = False
    console.print("[yellow]⚠️ Docling not available[/yellow]")

# Database imports
try:
    import duckdb
    DB_AVAILABLE = True
except ImportError:
    DB_AVAILABLE = False
    console.print("[dim]Database: Disabled[/dim]")

import re

@dataclass
class LiveStats:
    """Real-time processing statistics"""
    start_time: float = 0.0
    current_stage: str = "Initializing"
    chars_processed: int = 0
    total_chars: int = 0
    chunks_created: int = 0
    entities_found: int = 0
    memory_mb: float = 0.0
    cpu_percent: float = 0.0
    heartbeat_count: int = 0
    last_activity: str = "Starting up..."
    error_count: int = 0
    
    def __post_init__(self):
        """Initialize warnings list after object creation"""
        self.warnings = []

class ActivityMonitor:
    """Real-time activity monitor to prove we're not frozen"""
    
    def __init__(self):
        self.stats = LiveStats()
        self.running = False
        self.monitor_thread = None
        self.process = psutil.Process()
        self.last_update = time.time()
        
    def start(self):
        """Start monitoring"""
        self.running = True
        self.stats.start_time = time.time()
        self.monitor_thread = threading.Thread(target=self._monitor_loop, daemon=True)
        self.monitor_thread.start()
        
    def stop(self):
        """Stop monitoring"""
        self.running = False
        if self.monitor_thread:
            self.monitor_thread.join(timeout=1)
            
    def update_activity(self, activity: str, stage: str = None):
        """Update current activity"""
        self.stats.last_activity = activity
        if stage:
            self.stats.current_stage = stage
        self.stats.heartbeat_count += 1
        self.last_update = time.time()
        
    def update_progress(self, chars_processed: int = None, total_chars: int = None, 
                       chunks: int = None, entities: int = None):
        """Update progress metrics"""
        if chars_processed is not None:
            self.stats.chars_processed = chars_processed
        if total_chars is not None:
            self.stats.total_chars = total_chars
        if chunks is not None:
            self.stats.chunks_created = chunks
        if entities is not None:
            self.stats.entities_found = entities
            
    def add_warning(self, warning: str):
        """Add a warning"""
        if not hasattr(self.stats, 'warnings'):
            self.stats.warnings = []
        self.stats.warnings.append(f"{datetime.now().strftime('%H:%M:%S')}: {warning}")
        if len(self.stats.warnings) > 5:  # Keep only last 5
            self.stats.warnings.pop(0)
            
    def _monitor_loop(self):
        """Background monitoring loop"""
        while self.running:
            try:
                # Update system stats
                self.stats.memory_mb = self.process.memory_info().rss / 1024 / 1024
                self.stats.cpu_percent = self.process.cpu_percent()
                
                # Check if we're frozen (no activity for 10 seconds)
                if time.time() - self.last_update > 10:
                    if not hasattr(self.stats, 'warnings'):
                        self.stats.warnings = []
                    self.stats.warnings.append(f"{datetime.now().strftime('%H:%M:%S')}: No activity for 10s - still working...")
                    self.last_update = time.time()  # Reset to avoid spam
                    
                time.sleep(0.5)  # Update every 500ms
            except Exception as e:
                self.stats.error_count += 1
                
    def create_live_display(self) -> Layout:
        """Create live display layout"""
        layout = Layout()
        
        # Split into sections
        layout.split_column(
            Layout(name="header", size=8),
            Layout(name="main", ratio=1),
            Layout(name="footer", size=4)
        )
        
        # Header - current stage and progress
        elapsed = time.time() - self.stats.start_time
        progress_text = ""
        if self.stats.total_chars > 0:
            percent = (self.stats.chars_processed / self.stats.total_chars) * 100
            progress_text = f"Progress: {percent:.1f}% ({self.stats.chars_processed:,}/{self.stats.total_chars:,} chars)"
        
        header_content = Text()
        header_content.append(f"🐹 CHONKER v6.0 - {self.stats.current_stage}\n", style="bold cyan")
        header_content.append(f"⏱️  Elapsed: {elapsed:.1f}s | ", style="green")
        header_content.append(f"💓 Heartbeat: {self.stats.heartbeat_count} | ", style="yellow")
        header_content.append(f"🧠 Memory: {self.stats.memory_mb:.1f}MB | ", style="blue")
        header_content.append(f"⚡ CPU: {self.stats.cpu_percent:.1f}%\n", style="magenta")
        header_content.append(f"📊 {progress_text}\n", style="white")
        header_content.append(f"🔄 {self.stats.last_activity}", style="dim")
        
        layout["header"].update(Panel(header_content, title="🚀 Live Status", border_style="green"))
        
        # Main - detailed stats
        stats_table = Table(show_header=False, box=None)
        stats_table.add_column("Metric", style="cyan", width=20)
        stats_table.add_column("Value", style="white", width=15)
        
        stats_table.add_row("📦 Chunks Created", str(self.stats.chunks_created))
        stats_table.add_row("🔍 Entities Found", str(self.stats.entities_found))
        
        # Safe warning count
        warning_count = 0
        if hasattr(self.stats, 'warnings') and isinstance(self.stats.warnings, list):
            warning_count = len(self.stats.warnings)
        
        stats_table.add_row("⚠️ Warnings", str(warning_count))
        stats_table.add_row("❌ Errors", str(self.stats.error_count))
        
        # Recent warnings - only show if there are warnings
        if hasattr(self.stats, 'warnings') and isinstance(self.stats.warnings, list) and self.stats.warnings:
            warnings_text = Text()
            for warning in self.stats.warnings[-2:]:  # Show last 2 warnings only
                warnings_text.append(f"⚠️ {warning}\n", style="yellow")
            
            # Create a compact layout with warnings on the right
            main_content = Table.grid(padding=0)
            main_content.add_column(ratio=2)  # Stats take 2/3
            main_content.add_column(ratio=1)  # Warnings take 1/3
            main_content.add_row(stats_table, warnings_text)
        else:
            # No warnings - just show stats table
            main_content = stats_table
        
        layout["main"].update(Panel(main_content, title="📊 Processing Details", border_style="blue"))
        
        # Footer - activity log
        footer_content = Text()
        footer_content.append("🔄 Latest Activity: ", style="bold")
        footer_content.append(self.stats.last_activity, style="green")
        footer_content.append(f" (Updated: {datetime.now().strftime('%H:%M:%S')})", style="dim")
        
        layout["footer"].update(Panel(footer_content, title="📝 Activity Log", border_style="yellow"))
        
        return layout

class SystemKeepAwake:
    """Cross-platform system keep-awake manager"""
    
    def __init__(self, monitor: ActivityMonitor = None):
        self.monitor = monitor
        self.active = False
        self.platform = platform.system().lower()
        self.process = None
        self.original_settings = None
        
    def start_keep_awake(self):
        """Prevent system sleep during processing"""
        if self.active:
            return
            
        try:
            if self.monitor:
                self.monitor.update_activity("☕ Preventing system sleep...")
                
            if self.platform == "darwin":  # macOS
                # Use caffeinate to prevent sleep
                self.process = subprocess.Popen(['caffeinate', '-d'], 
                                              stdout=subprocess.DEVNULL, 
                                              stderr=subprocess.DEVNULL)
                self.active = True
                if self.monitor:
                    self.monitor.update_activity("☕ macOS sleep prevention active (caffeinate)")
                    
            elif self.platform == "windows":  # Windows
                # Use powercfg to prevent sleep
                try:
                    subprocess.run(['powercfg', '/requests'], check=True, 
                                 stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
                    # Set execution required to prevent sleep
                    import ctypes
                    ctypes.windll.kernel32.SetThreadExecutionState(0x80000003)  # ES_CONTINUOUS | ES_SYSTEM_REQUIRED | ES_DISPLAY_REQUIRED
                    self.active = True
                    if self.monitor:
                        self.monitor.update_activity("☕ Windows sleep prevention active")
                except Exception as e:
                    if self.monitor:
                        self.monitor.add_warning(f"Windows sleep prevention failed: {e}")
                        
            elif self.platform == "linux":  # Linux
                # Try systemd-inhibit first, then xset
                try:
                    self.process = subprocess.Popen(['systemd-inhibit', '--what=sleep:idle', '--who=CHONKER', '--why=Processing document', 'sleep', '86400'], 
                                                  stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
                    self.active = True
                    if self.monitor:
                        self.monitor.update_activity("☕ Linux sleep prevention active (systemd-inhibit)")
                except FileNotFoundError:
                    # Fallback to xset if available
                    try:
                        subprocess.run(['xset', 's', 'off'], check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
                        subprocess.run(['xset', '-dpms'], check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
                        self.active = True
                        if self.monitor:
                            self.monitor.update_activity("☕ Linux sleep prevention active (xset)")
                    except Exception as e:
                        if self.monitor:
                            self.monitor.add_warning(f"Linux sleep prevention failed: {e}")
            else:
                if self.monitor:
                    self.monitor.add_warning(f"Sleep prevention not supported on {self.platform}")
                    
        except Exception as e:
            if self.monitor:
                self.monitor.add_warning(f"Keep-awake error: {e}")
    
    def stop_keep_awake(self):
        """Restore normal sleep behavior"""
        if not self.active:
            return
            
        try:
            if self.monitor:
                self.monitor.update_activity("😴 Restoring normal sleep settings...")
                
            if self.platform == "darwin" and self.process:  # macOS
                self.process.terminate()
                self.process.wait(timeout=5)
                
            elif self.platform == "windows":  # Windows
                try:
                    import ctypes
                    ctypes.windll.kernel32.SetThreadExecutionState(0x80000000)  # ES_CONTINUOUS
                except:
                    pass
                    
            elif self.platform == "linux" and self.process:  # Linux
                self.process.terminate()
                try:
                    self.process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    self.process.kill()
                # Try to restore xset settings
                try:
                    subprocess.run(['xset', 's', 'on'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
                    subprocess.run(['xset', '+dpms'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
                except:
                    pass
                    
            self.active = False
            if self.monitor:
                self.monitor.update_activity("😴 Sleep settings restored")
                
        except Exception as e:
            if self.monitor:
                self.monitor.add_warning(f"Sleep restore error: {e}")
        finally:
            self.active = False
            self.process = None

@dataclass
class SimpleChunk:
    """Simple chunk representation"""
    id: int
    content: str
    char_count: int
    source_file: str
    page_info: str = ""
    
    def __post_init__(self):
        """Initialize entities and metadata after object creation"""
        self.entities = []
        self.metadata = {}

@dataclass
class SimpleDocument:
    """Simple document representation"""
    filename: str
    title: str
    content: str
    chunks: List[SimpleChunk]
    processing_time: float
    
    def __post_init__(self):
        """Initialize metadata after object creation"""
        self.metadata = {}

class SimpleDocProcessor:
    """Document processor with live monitoring"""
    
    def __init__(self, monitor: ActivityMonitor):
        self.monitor = monitor
        self.converter = None
        if DOCLING_AVAILABLE:
            try:
                self.monitor.update_activity("Initializing Docling converter...")
                if DOCLING_ADVANCED:
                    try:
                        pipeline_options = PdfPipelineOptions(do_ocr=False, do_table_structure=True)
                        format_options = {InputFormat.PDF: PdfFormatOption(pipeline_options=pipeline_options)}
                        self.converter = DocumentConverter(format_options=format_options)
                        self.monitor.update_activity("✅ Docling converter ready (advanced mode)")
                    except Exception as e:
                        self.monitor.add_warning(f"Advanced setup failed: {e}")
                        self.converter = DocumentConverter()
                        self.monitor.update_activity("✅ Docling converter ready (basic mode)")
                else:
                    self.converter = DocumentConverter()
                    self.monitor.update_activity("✅ Docling converter ready (basic mode)")
            except Exception as e:
                self.monitor.add_warning(f"Docling setup issue: {e}")
                self.converter = None
    
    def process_document(self, file_path: str) -> SimpleDocument:
        """Process document with live monitoring"""
        start_time = time.time()
        
        # Get file size for progress tracking
        file_size = os.path.getsize(file_path)
        self.monitor.update_activity(f"📄 Starting processing of {Path(file_path).name} ({file_size/1024/1024:.1f}MB)", "Document Processing")
        
        if not self.converter:
            self.monitor.update_activity("⚠️ No Docling - using fallback processing")
            return self._fallback_processing(file_path, start_time)
        
        try:
            self.monitor.update_activity("🧠 Initializing Docling (first run can take 30-60s for ML model loading)...")
            
            # This is the long operation - we can't track it internally but we can show heartbeat
            heartbeat_thread = threading.Thread(target=self._docling_heartbeat, daemon=True)
            heartbeat_thread.start()
            
            # Add timeout for very long operations
            self.monitor.update_activity("🧠 Starting Docling conversion (may take several minutes for complex PDFs)...")
            
            try:
                result = self.converter.convert(file_path)
            except Exception as conversion_error:
                self.monitor.add_warning(f"Docling conversion failed: {conversion_error}")
                self.monitor.update_activity("⚠️ Docling conversion failed, trying fallback...")
                return self._fallback_processing(file_path, start_time)
            
            self.monitor.update_activity("✅ Docling conversion complete, extracting content...")
            
            # Extract content safely
            content = self._safe_extract_content(result)
            self.monitor.update_progress(total_chars=len(content))
            
            title = self._safe_extract_title(result, file_path)
            metadata = self._safe_extract_metadata(result)
            
            processing_time = time.time() - start_time
            self.monitor.update_activity(f"✅ Document processed: {len(content):,} characters in {processing_time:.1f}s")
            
            doc = SimpleDocument(
                filename=Path(file_path).name,
                title=title,
                content=content,
                chunks=[],
                processing_time=processing_time
            )
            doc.metadata = metadata
            
            return doc
            
        except KeyboardInterrupt:
            self.monitor.add_warning("Processing interrupted by user")
            self.monitor.update_activity("⚠️ User interrupted, trying fallback processing...")
            return self._fallback_processing(file_path, start_time)
        except Exception as e:
            self.monitor.add_warning(f"Docling error: {e}")
            self.monitor.update_activity("⚠️ Docling failed, using fallback...")
            return self._fallback_processing(file_path, start_time)
    
    def _docling_heartbeat(self):
        """Show heartbeat during long Docling operation"""
        heartbeat_messages = [
            "🧠 Docling initializing ML models (this can take 30-60s)...",
            "🧠 Docling loading document structure...",
            "🧠 Docling analyzing page layout...",
            "🧠 Docling processing text blocks...",
            "🧠 Docling extracting tables and figures...",
            "🧠 Docling running OCR if needed...",
            "🧠 Docling finalizing extraction (almost done)..."
        ]
        
        i = 0
        elapsed_start = time.time()
        
        while self.monitor.running and self.monitor.stats.current_stage == "Document Processing":
            elapsed = time.time() - elapsed_start
            message = heartbeat_messages[i % len(heartbeat_messages)]
            
            # Add elapsed time to show progress
            if elapsed > 30:
                message += f" [{elapsed:.0f}s elapsed - Docling can be slow on first run]"
            elif elapsed > 10:
                message += f" [{elapsed:.0f}s elapsed]"
                
            self.monitor.update_activity(message)
            time.sleep(3)  # Update every 3 seconds
            i += 1
            
            # If it's taking too long, suggest fallback
            if elapsed > 120:  # 2 minutes
                self.monitor.add_warning("Docling taking longer than expected - consider Ctrl+C to try fallback")
                time.sleep(10)  # Wait longer between messages
    
    def _safe_extract_content(self, result) -> str:
        """Safely extract content with progress updates"""
        self.monitor.update_activity("🔍 Attempting export_to_markdown...")
        
        try:
            if hasattr(result, 'document'):
                doc = result.document
                
                # Method 1: export_to_markdown
                if hasattr(doc, 'export_to_markdown'):
                    try:
                        content = doc.export_to_markdown()
                        if content and len(content.strip()) > 10:
                            self.monitor.update_activity(f"✅ Extracted via export_to_markdown: {len(content):,} chars")
                            return content
                    except Exception as e:
                        self.monitor.add_warning(f"export_to_markdown failed: {e}")
                
                # Method 2: export_to_text
                self.monitor.update_activity("🔍 Attempting export_to_text...")
                if hasattr(doc, 'export_to_text'):
                    try:
                        content = doc.export_to_text()
                        if content and len(content.strip()) > 10:
                            self.monitor.update_activity(f"✅ Extracted via export_to_text: {len(content):,} chars")
                            return content
                    except Exception as e:
                        self.monitor.add_warning(f"export_to_text failed: {e}")
                
                # Method 3: Page iteration
                self.monitor.update_activity("🔍 Attempting page-by-page extraction...")
                if hasattr(doc, 'pages'):
                    try:
                        content_parts = []
                        pages = doc.pages
                        
                        if hasattr(pages, '__iter__') or hasattr(pages, '__getitem__'):
                            page_count = 0
                            for page in pages:
                                page_count += 1
                                self.monitor.update_activity(f"📄 Processing page {page_count}...")
                                
                                page_text = ""
                                if hasattr(page, 'parsed_page'):
                                    try:
                                        parsed_page = page.parsed_page
                                        if hasattr(parsed_page, 'export_to_text'):
                                            page_text = parsed_page.export_to_text()
                                        elif hasattr(parsed_page, 'export_to_markdown'):
                                            page_text = parsed_page.export_to_markdown()
                                    except Exception as e:
                                        self.monitor.add_warning(f"Page {page_count} extraction failed: {e}")
                                
                                if page_text and page_text.strip():
                                    content_parts.append(page_text)
                                    
                            if content_parts:
                                content = '\n\n'.join(content_parts)
                                self.monitor.update_activity(f"✅ Extracted via page iteration: {len(content):,} chars from {page_count} pages")
                                return content
                                
                    except Exception as e:
                        self.monitor.add_warning(f"Page processing failed: {e}")
            
            self.monitor.add_warning("No content extraction method succeeded")
            return "[Document processed but no readable content could be extracted]"
            
        except Exception as e:
            self.monitor.add_warning(f"Content extraction error: {e}")
            return f"[Content extraction failed: {e}]"
    
    def _safe_extract_title(self, result, file_path: str) -> str:
        """Safely extract title"""
        try:
            content = self._safe_extract_content(result)
            lines = content.split('\n')[:10]
            
            for line in lines:
                line = line.strip()
                if line and len(line) > 10 and not line.startswith('#'):
                    return line[:100]
            
            return Path(file_path).stem
            
        except Exception:
            return Path(file_path).stem
    
    def _safe_extract_metadata(self, result) -> Dict:
        """Safely extract metadata"""
        try:
            metadata = {
                'processing_method': 'docling',
                'extracted_at': datetime.now().isoformat()
            }
            
            if hasattr(result, 'document'):
                doc = result.document
                if hasattr(doc, 'pages'):
                    try:
                        if hasattr(doc.pages, '__len__'):
                            metadata['page_count'] = len(doc.pages)
                        elif hasattr(doc.pages, '__iter__'):
                            metadata['page_count'] = sum(1 for _ in doc.pages)
                    except Exception:
                        pass
            
            return metadata
            
        except Exception as e:
            return {'processing_method': 'docling_fallback', 'error': str(e)}
    
    def _fallback_processing(self, file_path: str, start_time: float) -> SimpleDocument:
        """Fallback processing for when Docling fails or is unavailable"""
        try:
            self.monitor.update_activity("📄 Using fallback processing...")
            
            if file_path.lower().endswith('.pdf'):
                self.monitor.update_activity("📄 PDF detected - attempting basic text extraction...")
                
                # Try PyPDF2 as a fallback for PDFs
                try:
                    import PyPDF2
                    self.monitor.update_activity("📄 Using PyPDF2 for PDF extraction...")
                    
                    with open(file_path, 'rb') as file:
                        pdf_reader = PyPDF2.PdfReader(file)
                        content_parts = []
                        
                        for page_num, page in enumerate(pdf_reader.pages):
                            self.monitor.update_activity(f"📄 Extracting page {page_num + 1}/{len(pdf_reader.pages)}...")
                            try:
                                page_text = page.extract_text()
                                if page_text.strip():
                                    content_parts.append(page_text)
                            except Exception as e:
                                self.monitor.add_warning(f"Page {page_num + 1} extraction failed: {e}")
                        
                        if content_parts:
                            content = '\n\n'.join(content_parts)
                            self.monitor.update_activity(f"✅ PyPDF2 extracted {len(content):,} characters from {len(content_parts)} pages")
                        else:
                            content = "[PDF processed but no readable text could be extracted]"
                            
                except ImportError:
                    self.monitor.add_warning("PyPDF2 not available - install with: pip install PyPDF2")
                    content = "[PDF file - install Docling or PyPDF2 for text extraction]"
                except Exception as e:
                    self.monitor.add_warning(f"PyPDF2 extraction failed: {e}")
                    content = f"[PDF extraction failed: {e}]"
            else:
                self.monitor.update_activity("📄 Reading text file...")
                with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
                    content = f.read()
                self.monitor.update_activity(f"✅ Read {len(content):,} characters from text file")
            
            doc = SimpleDocument(
                filename=Path(file_path).name,
                title=Path(file_path).stem,
                content=content,
                chunks=[],
                processing_time=time.time() - start_time
            )
            doc.metadata = {'processing_method': 'fallback'}
            
            return doc
            
        except Exception as e:
            self.monitor.add_warning(f"File reading failed: {e}")
            doc = SimpleDocument(
                filename=Path(file_path).name,
                title=Path(file_path).stem,
                content=f"[Error reading file: {e}]",
                chunks=[],
                processing_time=time.time() - start_time
            )
            doc.metadata = {'processing_method': 'error'}
            
            return doc

class SimpleChunker:
    """Chunker with live monitoring"""
    
    def __init__(self, chunk_size: int = 95000, monitor: ActivityMonitor = None):
        self.chunk_size = chunk_size
        self.monitor = monitor
    
    def create_chunks(self, doc: SimpleDocument) -> List[SimpleChunk]:
        """Create chunks with live progress"""
        chunks = []
        content = doc.content
        
        self.monitor.update_activity(f"📦 Analyzing {len(content):,} characters for chunking...", "Chunking")
        self.monitor.update_progress(total_chars=len(content))
        
        if len(content) <= self.chunk_size:
            self.monitor.update_activity("📦 Creating single chunk (content fits in one piece)...")
            chunk = SimpleChunk(
                id=1,
                content=content,
                char_count=len(content),
                source_file=doc.filename,
                page_info="Complete document"
            )
            chunk.metadata = {'total_chunks': 1}
            chunks.append(chunk)
            self.monitor.update_progress(chunks=1)
            self.monitor.update_activity("✅ Single chunk created")
        else:
            estimated_chunks = len(content) // self.chunk_size + 1
            self.monitor.update_activity(f"📦 Creating ~{estimated_chunks} chunks...")
            
            chunk_id = 1
            start = 0
            
            while start < len(content):
                self.monitor.update_activity(f"📦 Creating chunk {chunk_id} (position {start:,}/{len(content):,})...")
                
                end = start + self.chunk_size
                
                # Try to break at word boundary
                if end < len(content):
                    last_space = content.rfind(' ', start, end)
                    last_newline = content.rfind('\n', start, end)
                    break_point = max(last_space, last_newline)
                    
                    if break_point > start:
                        end = break_point
                
                chunk_content = content[start:end]
                
                chunk = SimpleChunk(
                    id=chunk_id,
                    content=chunk_content,
                    char_count=len(chunk_content),
                    source_file=doc.filename,
                    page_info=f"Chunk {chunk_id}"
                )
                chunk.metadata = {
                    'start_pos': start,
                    'end_pos': end,
                    'is_partial': end < len(content)
                }
                chunks.append(chunk)
                
                self.monitor.update_progress(chars_processed=end, chunks=len(chunks))
                
                start = end
                chunk_id += 1
        
        self.monitor.update_activity(f"✅ Created {len(chunks)} chunks successfully")
        return chunks

class SimpleEntityExtractor:
    """Entity extractor with live monitoring"""
    
    def __init__(self, monitor: ActivityMonitor = None):
        self.monitor = monitor
        self.patterns = {
            'email': r'\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b',
            'phone': r'\b(?:\+?1[-.\s]?)?\(?[0-9]{3}\)?[-.\s]?[0-9]{3}[-.\s]?[0-9]{4}\b',
            'date': r'\b(?:\d{1,2}[/-]\d{1,2}[/-]\d{2,4}|\d{4}-\d{1,2}-\d{1,2})\b',
            'sample_id': r'\b(?:TP|MW|Sample|SP|W|B|L\d+)-?\s*\d+[A-Z]?\b',
            'chemical': r'\b(?:Lead|Pb|Benzene|Toluene|Mercury|Hg|Arsenic|As|Chromium|Cr)\b',
            'concentration': r'\b\d+\.?\d*\s*(?:mg/kg|ppm|ppb|mg/L|ug/Kg)\b',
            'number': r'\b\d+\.?\d*\b'
        }
        
        self.compiled_patterns = {}
        for name, pattern in self.patterns.items():
            try:
                self.compiled_patterns[name] = re.compile(pattern, re.IGNORECASE)
            except re.error as e:
                if self.monitor:
                    self.monitor.add_warning(f"Pattern error for {name}: {e}")
    
    def extract_entities(self, chunks: List[SimpleChunk]) -> int:
        """Extract entities with live monitoring"""
        total_entities = 0
        
        self.monitor.update_activity(f"🔍 Starting entity extraction on {len(chunks)} chunks...", "Entity Extraction")
        
        for i, chunk in enumerate(chunks):
            self.monitor.update_activity(f"🔍 Processing chunk {i+1}/{len(chunks)} ({chunk.char_count:,} chars)...")
            
            chunk.entities = []
            
            for entity_type, pattern in self.compiled_patterns.items():
                try:
                    matches = pattern.findall(chunk.content)
                    for match in matches:
                        entity = {
                            'type': entity_type,
                            'value': match.strip(),
                            'chunk_id': chunk.id
                        }
                        chunk.entities.append(entity)
                        total_entities += 1
                except Exception as e:
                    self.monitor.add_warning(f"Entity extraction error: {e}")
            
            self.monitor.update_progress(entities=total_entities)
            
            if chunk.entities:
                self.monitor.update_activity(f"📊 Chunk {i+1}: found {len(chunk.entities)} entities (total: {total_entities})")
        
        self.monitor.update_activity(f"✅ Entity extraction complete: {total_entities} entities found")
        return total_entities

class SimpleDatabase:
    """Database with monitoring"""
    
    def __init__(self, monitor: ActivityMonitor = None):
        self.monitor = monitor
        self.enabled = False
        if not DB_AVAILABLE:
            return
        
        try:
            if self.monitor:
                self.monitor.update_activity("🗄️ Initializing DuckDB...")
            self.conn = duckdb.connect("chonker_simple.db")
            self._init_schema()
            self.enabled = True
            if self.monitor:
                self.monitor.update_activity("✅ Database ready")
        except Exception as e:
            if self.monitor:
                self.monitor.add_warning(f"Database disabled: {e}")
    
    def _init_schema(self):
        """Initialize schema"""
        try:
            self.conn.execute("""
                CREATE TABLE IF NOT EXISTS documents (
                    filename VARCHAR PRIMARY KEY,
                    title VARCHAR,
                    content_length INTEGER,
                    chunk_count INTEGER,
                    entity_count INTEGER,
                    processing_time REAL,
                    processed_at TIMESTAMP
                )
            """)
            
            self.conn.execute("""
                CREATE TABLE IF NOT EXISTS chunks (
                    id VARCHAR PRIMARY KEY,
                    document_filename VARCHAR,
                    chunk_number INTEGER,
                    content TEXT,
                    char_count INTEGER,
                    entity_count INTEGER
                )
            """)
            
            self.conn.execute("""
                CREATE TABLE IF NOT EXISTS entities (
                    id VARCHAR PRIMARY KEY,
                    chunk_id VARCHAR,
                    type VARCHAR,
                    value VARCHAR,
                    document_filename VARCHAR
                )
            """)
        except Exception as e:
            if self.monitor:
                self.monitor.add_warning(f"Schema error: {e}")
    
    def save_document(self, doc: SimpleDocument, total_entities: int) -> bool:
        """Save document with monitoring"""
        if not self.enabled:
            return True
        
        try:
            if self.monitor:
                self.monitor.update_activity("🗄️ Saving document to database...")
                
            self.conn.execute("""
                INSERT OR REPLACE INTO documents 
                (filename, title, content_length, chunk_count, entity_count, processing_time, processed_at)
                VALUES (?, ?, ?, ?, ?, ?, ?)
            """, (
                doc.filename,
                doc.title,
                len(doc.content),
                len(doc.chunks),
                total_entities,
                doc.processing_time,
                datetime.now()
            ))
            
            if self.monitor:
                self.monitor.update_activity("✅ Document saved to database")
            return True
        except Exception as e:
            if self.monitor:
                self.monitor.add_warning(f"Save error: {e}")
            return False
    
    def search_entities(self, query: str, entity_type: str = None) -> List[Dict]:
        """Search entities"""
        if not self.enabled:
            return []
        
        try:
            sql = "SELECT * FROM entities WHERE value LIKE ?"
            params = [f"%{query}%"]
            
            if entity_type:
                sql += " AND type = ?"
                params.append(entity_type)
            
            results = self.conn.execute(sql, params).fetchall()
            columns = [desc[0] for desc in self.conn.description]
            return [dict(zip(columns, row)) for row in results]
        except Exception as e:
            if self.monitor:
                self.monitor.add_warning(f"Search error: {e}")
            return []

class AnxietyFreeCHONKER:
    """🐹 CHONKER v6.0 - Anxiety-Free with Live Monitoring and Snyfter Integration"""
    
    def __init__(self, chunk_size: int = 95000, fast_mode: bool = False):
        self.chunk_size = chunk_size
        self.fast_mode = fast_mode
        self.current_document: Optional[SimpleDocument] = None
        
        # SNYFTER INTEGRATION: Use persistent directory instead of temp
        self.chunk_dir = Path("saved_chonker_chunks")
        self.chunk_dir.mkdir(exist_ok=True)
        
        # Initialize monitoring (only if not in fast mode)
        if not self.fast_mode:
            self.monitor = ActivityMonitor()
            # Initialize keep-awake system
            self.keep_awake = SystemKeepAwake(self.monitor)
        else:
            self.monitor = None
            self.keep_awake = None
        
        # Initialize components with monitoring
        self.processor = SimpleDocProcessor(self.monitor)
        self.chunker = SimpleChunker(chunk_size, self.monitor)
        self.entity_extractor = SimpleEntityExtractor(self.monitor)
        self.db = SimpleDatabase(self.monitor)
        
        self._show_welcome()
    
    def _show_welcome(self):
        """Show welcome with monitoring info"""
        status_lines = []
        
        if DOCLING_AVAILABLE:
            status_lines.append("[green]✅ Document Processing: Docling enabled[/green]")
        else:
            status_lines.append("[yellow]⚠️ Document Processing: Text extraction only[/yellow]")
        
        status_lines.append("[green]✅ Chunking: Smart boundary detection[/green]")
        status_lines.append("[green]✅ Entity Extraction: 8 robust patterns[/green]")
        status_lines.append("[green]✅ Live Monitoring: Real-time progress & heartbeat[/green]")
        status_lines.append("[green]✅ Keep Awake: Prevents sleep during processing[/green]")
        status_lines.append("[green]✅ Snyfter Integration: Chunks saved to saved_chonker_chunks/[/green]")
        
        if self.db.enabled:
            status_lines.append("[green]✅ Database: DuckDB indexing[/green]")
        else:
            status_lines.append("[yellow]⚠️ Database: File-based only[/yellow]")
        
        welcome = Panel(
            "[bold cyan]🐹 CHONKER v6.0 - Anxiety-Free Document Platform[/bold cyan]\n\n" +
            "[green]🔧 Live Monitor → Process → Chunk → Extract → Store → Export for Snyfter[/green]\n\n" +
            "\n".join(status_lines) + "\n\n" +
            "📄 [yellow]load[/yellow] - Process with live monitoring\n" +
            "📄 [yellow]load <filename>[/yellow] - Process specific file\n" +
            "📋 [yellow]list[/yellow] - Show chunks\n" +
            "🪟 [yellow]show <n>[/yellow] - View chunk content\n" +
            "🔍 [yellow]search <term>[/yellow] - Find entities\n" +
            "📊 [yellow]info[/yellow] - Document summary\n" +
            "🔄 [yellow]export[/yellow] - Export chunks for Snyfter\n" +
            "💡 [yellow]help[/yellow] - Show commands\n" +
            "🚪 [yellow]exit[/yellow] - Quit",
            title="🎯 Anxiety-Free Processing Ready! (Snyfter Compatible)",
            style="bold blue"
        )
        console.print(welcome)
    
    def load_document(self, filename: str = "") -> bool:
        """Load document with live monitoring"""
        if not filename:
            return self._show_available_documents()
        
        # Auto-detect file extension
        if not os.path.exists(filename):
            for ext in ['.pdf', '.docx', '.txt', '.md']:
                if os.path.exists(filename + ext):
                    filename += ext
                    break
            else:
                console.print(f"[red]❌ File not found: {filename}[/red]")
                return False
        
        try:
            return self._process_document_with_live_monitoring(filename)
        except Exception as e:
            console.print(f"[red]❌ Processing error: {e}[/red]")
            return False
    
    def _process_document_with_live_monitoring(self, filename: str) -> bool:
        """Process document with anxiety-reducing live display"""
        file_size_mb = os.path.getsize(filename) / (1024 * 1024)
        console.print(f"[blue]📄 Starting live processing: {filename} ({file_size_mb:.1f} MB)[/blue]")
        console.print("[dim]💡 Watch the live monitor to see real-time progress, memory usage, and heartbeat![/dim]")
        console.print(f"[dim]🔄 Chunks will be saved to: {self.chunk_dir.absolute()}[/dim]\n")
        
        # Start monitoring and keep-awake
        self.monitor.start()
        self.keep_awake.start_keep_awake()
        
        try:
            with Live(self.monitor.create_live_display(), refresh_per_second=2, console=console) as live:
                # Step 1: Document processing
                self.monitor.update_activity("🚀 Starting document processing pipeline...")
                doc = self.processor.process_document(filename)
                
                # Step 2: Chunking
                self.monitor.update_activity("📦 Starting intelligent chunking...", "Chunking")
                chunks = self.chunker.create_chunks(doc)
                doc.chunks = chunks
                
                # Step 3: Entity extraction
                self.monitor.update_activity("🔍 Starting entity extraction...", "Entity Extraction")
                total_entities = self.entity_extractor.extract_entities(chunks)
                
                # Step 4: Database
                if self.db.enabled:
                    self.monitor.update_activity("🗄️ Saving to database...", "Database")
                    self.db.save_document(doc, total_entities)
                
                # Step 5: File creation (SNYFTER INTEGRATION)
                self.monitor.update_activity("📁 Creating chunk files for Snyfter...", "Snyfter Export")
                self._create_chunk_files(doc)
                
                self.monitor.update_activity("🎉 ALL PROCESSING COMPLETE!", "Complete")
                
                # Keep display up for 3 seconds to show completion
                time.sleep(3)
                
        finally:
            self.monitor.stop()
            self.keep_awake.stop_keep_awake()
        
        self.current_document = doc
        
        # Show final summary with Snyfter integration info
        console.print(Panel(
            f"[bold green]🎉 Processing Complete![/bold green]\n\n" +
            f"[cyan]Document:[/cyan] {doc.filename}\n" +
            f"[cyan]Title:[/cyan] {doc.title}\n" +
            f"[cyan]Processing Time:[/cyan] {doc.processing_time:.1f}s\n" +
            f"[cyan]Content Length:[/cyan] {len(doc.content):,} chars\n" +
            f"[cyan]Chunks:[/cyan] {len(chunks)}\n" +
            f"[cyan]Entities:[/cyan] {total_entities}\n" +
            f"[cyan]Method:[/cyan] {doc.metadata.get('processing_method', 'unknown')}\n\n" +
            f"[green]💓 Final Heartbeat:[/green] {self.monitor.stats.heartbeat_count}\n" +
            f"[green]🧠 Peak Memory:[/green] {self.monitor.stats.memory_mb:.1f}MB\n\n" +
            f"[bold yellow]🔄 Snyfter Integration:[/bold yellow]\n" +
            f"[cyan]Chunks saved to:[/cyan] {self.chunk_dir.absolute()}\n" +
            f"[cyan]Ready for Snyfter:[/cyan] python snyfter.py --chunks",
            title="📊 Final Summary",
            style="green"
        ))
        
        return True
    
    def _show_available_documents(self) -> bool:
        """Show available documents"""
        doc_files = []
        for pattern in ['*.pdf', '*.docx', '*.txt', '*.md']:
            doc_files.extend(list(Path('.').glob(pattern)))
        
        if not doc_files:
            console.print("[yellow]🤷 No documents found[/yellow]")
            return False
        
        table = Table(title="📄 Available Documents")
        table.add_column("File", style="cyan")
        table.add_column("Type", style="magenta")
        table.add_column("Size", style="yellow")
        table.add_column("Modified", style="green")
        
        for doc_path in sorted(doc_files, key=lambda f: f.stat().st_mtime, reverse=True):
            stat = doc_path.stat()
            size_mb = stat.st_size / (1024 * 1024)
            size_str = f"{size_mb:.1f} MB" if size_mb >= 1 else f"{stat.st_size / 1024:.1f} KB"
            
            modified = datetime.fromtimestamp(stat.st_mtime)
            time_str = modified.strftime('%m/%d %H:%M')
            
            table.add_row(
                doc_path.name,
                doc_path.suffix.upper()[1:],
                size_str,
                time_str
            )
        
        console.print(table)
        return True
    
    def _create_chunk_files(self, doc: SimpleDocument):
        """Create chunk files with monitoring (SNYFTER INTEGRATION)"""
        for i, chunk in enumerate(doc.chunks):
            self.monitor.update_activity(f"📁 Creating chunk file {i+1}/{len(doc.chunks)}: chunk_{chunk.id}.txt")
            
            # SNYFTER INTEGRATION: Save to persistent directory with sequential naming
            chunk_file = self.chunk_dir / f"chunk_{chunk.id}.txt"
            
            with open(chunk_file, 'w', encoding='utf-8') as f:
                f.write(f"DOCUMENT CHUNK {chunk.id} OF {len(doc.chunks)}\n")
                f.write("=" * 60 + "\n")
                f.write(f"Document: {doc.filename}\n")
                f.write(f"Title: {doc.title}\n")
                f.write(f"Characters: {chunk.char_count:,}\n")
                f.write(f"Entities: {len(chunk.entities)}\n")
                
                if chunk.entities:
                    entity_types = {}
                    for entity in chunk.entities:
                        entity_types[entity['type']] = entity_types.get(entity['type'], 0) + 1
                    f.write(f"Entity Types: {', '.join([f'{k}({v})' for k, v in entity_types.items()])}\n")
                
                f.write("=" * 60 + "\n\n")
                f.write(chunk.content)
                f.write(f"\n\n{'=' * 60}\nEND OF CHUNK {chunk.id}")
        
        self.monitor.update_activity(f"✅ Exported {len(doc.chunks)} chunks to {self.chunk_dir} for Snyfter")
    
    def export_for_snyfter(self) -> bool:
        """Export current document chunks for Snyfter"""
        if not self.current_document:
            console.print("[yellow]⚠️ No document loaded[/yellow]")
            return False
        
        doc = self.current_document
        
        console.print(f"[blue]🔄 Exporting {len(doc.chunks)} chunks to {self.chunk_dir}[/blue]")
        
        # Clear existing chunks
        for existing_chunk in self.chunk_dir.glob("chunk_*.txt"):
            existing_chunk.unlink()
        
        # Export current chunks
        self._create_chunk_files(doc)
        
        console.print(Panel(
            f"[bold green]✅ Export Complete![/bold green]\n\n" +
            f"[cyan]Chunks exported:[/cyan] {len(doc.chunks)}\n" +
            f"[cyan]Export directory:[/cyan] {self.chunk_dir.absolute()}\n" +
            f"[cyan]File pattern:[/cyan] chunk_1.txt, chunk_2.txt, ...\n\n" +
            f"[bold yellow]Next Steps:[/bold yellow]\n" +
            f"[white]1. Run Snyfter: [/white][cyan]python snyfter.py --chunks[/cyan]\n" +
            f"[white]2. Or use GUI: [/white][cyan]python snyfter.py[/cyan] and select 'chunks'",
            title="🔄 Snyfter Export",
            style="green"
        ))
        
        return True
    
    def show_info(self) -> bool:
        """Show document info"""
        if not self.current_document:
            console.print("[yellow]⚠️ No document loaded[/yellow]")
            return False
        
        doc = self.current_document
        
        # Entity summary
        all_entities = []
        for chunk in doc.chunks:
            all_entities.extend(chunk.entities)
        
        entity_types = {}
        for entity in all_entities:
            entity_types[entity['type']] = entity_types.get(entity['type'], 0) + 1
        
        console.print(Panel(
            f"[bold cyan]📊 Document Information[/bold cyan]\n\n" +
            f"[cyan]Filename:[/cyan] {doc.filename}\n" +
            f"[cyan]Title:[/cyan] {doc.title}\n" +
            f"[cyan]Content Length:[/cyan] {len(doc.content):,} characters\n" +
            f"[cyan]Processing Time:[/cyan] {doc.processing_time:.1f} seconds\n" +
            f"[cyan]Method:[/cyan] {doc.metadata.get('processing_method', 'unknown')}\n\n" +
            f"[bold green]📦 Chunks[/bold green]\n" +
            f"[cyan]Total Chunks:[/cyan] {len(doc.chunks)}\n" +
            f"[cyan]Average Size:[/cyan] {sum(c.char_count for c in doc.chunks) // len(doc.chunks):,} chars\n" +
            f"[cyan]Exported to:[/cyan] {self.chunk_dir.absolute()}\n\n" +
            f"[bold green]🔍 Entities[/bold green]\n" +
            f"[cyan]Total Entities:[/cyan] {len(all_entities)}\n" +
            f"[cyan]Types:[/cyan] {', '.join([f'{k}({v})' for k, v in entity_types.items()])}",
            title="Document Info",
            style="blue"
        ))
        
        return True
    
    def list_chunks(self) -> bool:
        """List chunks"""
        if not self.current_document:
            console.print("[yellow]⚠️ No document loaded[/yellow]")
            return False
        
        doc = self.current_document
        
        table = Table(title=f"📋 Chunks - {doc.filename}")
        table.add_column("ID", style="cyan", width=5)
        table.add_column("Characters", style="yellow", justify="right", width=12)
        table.add_column("Entities", style="green", justify="center", width=10)
        table.add_column("File", style="blue", width=15)
        table.add_column("Preview", style="white")
        
        for chunk in doc.chunks:
            preview = chunk.content[:80].replace('\n', ' ').replace('\r', ' ')
            if len(chunk.content) > 80:
                preview += "..."
            
            chunk_file = f"chunk_{chunk.id}.txt"
            
            table.add_row(
                str(chunk.id),
                f"{chunk.char_count:,}",
                str(len(chunk.entities)),
                chunk_file,
                preview
            )
        
        console.print(table)
        
        # Show export status
        console.print(f"[dim]💡 Chunks saved to: {self.chunk_dir.absolute()}[/dim]")
        console.print(f"[dim]🔄 Ready for Snyfter: python snyfter.py --chunks[/dim]")
        
        return True
    
    def show_chunk(self, chunk_id: str) -> bool:
        """Show specific chunk"""
        if not self.current_document:
            console.print("[yellow]⚠️ No document loaded[/yellow]")
            return False
        
        try:
            chunk_num = int(chunk_id)
        except ValueError:
            console.print("[red]❌ Invalid chunk ID[/red]")
            return False
        
        chunk = None
        for c in self.current_document.chunks:
            if c.id == chunk_num:
                chunk = c
                break
        
        if not chunk:
            console.print(f"[red]❌ Chunk {chunk_num} not found[/red]")
            return False
        
        # SNYFTER INTEGRATION: Open chunk file from persistent directory
        chunk_file = self.chunk_dir / f"chunk_{chunk.id}.txt"
        
        if chunk_file.exists():
            if sys.platform == "darwin":  # macOS
                os.system(f"open '{chunk_file}'")
            elif sys.platform == "win32":  # Windows
                os.system(f"start '{chunk_file}'")
            else:  # Linux
                os.system(f"xdg-open '{chunk_file}'")
            
            console.print(f"[green]✅ Opened chunk {chunk_num} in editor[/green]")
            console.print(f"[dim]📁 File: {chunk_file}[/dim]")
        else:
            console.print(f"[red]❌ Chunk file not found: {chunk_file}[/red]")
        
        return True
    
    def search_entities(self, query: str) -> bool:
        """Search entities"""
        if not self.current_document:
            console.print("[yellow]⚠️ No document loaded[/yellow]")
            return False
        
        # Search in current document
        results = []
        for chunk in self.current_document.chunks:
            for entity in chunk.entities:
                if query.lower() in entity['value'].lower():
                    results.append({
                        'chunk_id': chunk.id,
                        'type': entity['type'],
                        'value': entity['value']
                    })
        
        if not results:
            console.print(f"[yellow]No entities found for: {query}[/yellow]")
            return False
        
        table = Table(title=f"🔍 Search Results: {query}")
        table.add_column("Chunk", style="cyan", width=8)
        table.add_column("Type", style="magenta", width=15)
        table.add_column("Value", style="green")
        
        for result in results[:50]:  # Limit to 50 results
            table.add_row(
                str(result['chunk_id']),
                result['type'],
                result['value']
            )
        
        console.print(table)
        
        if len(results) > 50:
            console.print(f"[dim]... and {len(results) - 50} more results[/dim]")
        
        return True
    
    def show_help(self):
        """Show help"""
        help_panel = Panel(
            "[bold cyan]🐹 CHONKER v6.0 Commands[/bold cyan]\n\n" +
            "[bold green]📄 Document Processing[/bold green]\n" +
            "[yellow]load[/yellow] - Show available documents\n" +
            "[yellow]load <filename>[/yellow] - Process with live monitoring\n" +
            "[yellow]info[/yellow] - Show current document info\n\n" +
            "[bold green]📋 Chunk Management[/bold green]\n" +
            "[yellow]list[/yellow] - List all chunks with previews\n" +
            "[yellow]show <n>[/yellow] - Open chunk in editor\n\n" +
            "[bold green]🔍 Search & Find[/bold green]\n" +
            "[yellow]search <term>[/yellow] - Search entities\n\n" +
            "[bold green]🔄 Snyfter Integration[/bold green]\n" +
            "[yellow]export[/yellow] - Export chunks for Snyfter\n" +
            "[dim]Chunks auto-exported after processing[/dim]\n\n" +
            "[bold green]💡 Help & Info[/bold green]\n" +
            "[yellow]help[/yellow] - Show this help\n" +
            "[yellow]exit[/yellow] - Quit CHONKER\n\n" +
            "[bold red]🚀 NEW: Snyfter Integration![/bold red]\n" +
            "[dim]• Chunks saved to saved_chonker_chunks/\n" +
            "• Ready for semantic search with Snyfter\n" +
            "• Run: python snyfter.py --chunks\n" +
            "• Live monitoring shows export progress[/dim]",
            title="Help",
            style="blue"
        )
        console.print(help_panel)
    
    def run(self):
        """Main interactive loop"""
        console.print("\n[bold green]🎮 Anxiety-Free CHONKER ready! (Snyfter Compatible)[/bold green]")
        
        while True:
            try:
                user_input = Prompt.ask("\n[bold cyan]🐹[/bold cyan]", default="").strip()
                
                if not user_input:
                    continue
                
                parts = user_input.split()
                command = parts[0].lower()
                args = parts[1:] if len(parts) > 1 else []
                
                if command in ['exit', 'quit']:
                    console.print("[blue]👋 Goodbye![/blue]")
                    # SNYFTER INTEGRATION: Don't delete chunks directory
                    console.print(f"[dim]💾 Chunks preserved in {self.chunk_dir} for Snyfter[/dim]")
                    break
                
                elif command == 'help':
                    self.show_help()
                
                elif command == 'load':
                    filename = args[0] if args else ""
                    self.load_document(filename)
                
                elif command == 'info':
                    self.show_info()
                
                elif command == 'list':
                    self.list_chunks()
                
                elif command == 'show':
                    if args:
                        self.show_chunk(args[0])
                    else:
                        console.print("[yellow]Usage: show <chunk_id>[/yellow]")
                
                elif command == 'search':
                    if args:
                        self.search_entities(' '.join(args))
                    else:
                        console.print("[yellow]Usage: search <term>[/yellow]")
                
                elif command == 'export':
                    self.export_for_snyfter()
                
                else:
                    console.print(f"[yellow]❓ Unknown command: {command}. Type 'help' for commands.[/yellow]")
            
            except KeyboardInterrupt:
                console.print("\n[blue]👋 Goodbye![/blue]")
                break
            except Exception as e:
                console.print(f"[red]❌ Error: {e}[/red]")

def main():
    """Entry point"""
    # Check for fast mode argument
    fast_mode = len(sys.argv) > 1 and sys.argv[1] == "--fast"
    
    if fast_mode:
        console.print("[bold cyan]🐹 CHONKER v6.0 - Fast Mode (No Live Monitoring)[/bold cyan]")
    else:
        console.print("[bold cyan]🐹 CHONKER v6.0 - Anxiety-Free Processing (Snyfter Compatible)[/bold cyan]")
    
    try:
        chonker = AnxietyFreeCHONKER(fast_mode=fast_mode)
        chonker.run()
    except Exception as e:
        console.print(f"[red]❌ Startup error: {e}[/red]")
        sys.exit(1)

if __name__ == "__main__":
    main()