#!/usr/bin/env python3
"""
Display utilities using Rich for beautiful terminal output
"""

from typing import List, Dict, Any, Optional
from rich.console import Console
from rich.table import Table
from rich.panel import Panel
from rich.syntax import Syntax
from rich.progress import Progress, BarColumn, TextColumn, TimeRemainingColumn
from rich.tree import Tree
from rich.layout import Layout
from rich.live import Live
from rich.text import Text
from datetime import datetime

# Initialize console
console = Console()

# Color scheme
COLORS = {
    "success": "green",
    "error": "red",
    "warning": "yellow",
    "info": "cyan",
    "primary": "#00CED1",  # Dark Turquoise
    "secondary": "#FF6B6B",  # Light Red
    "accent": "#4ECDC4",    # Medium Turquoise
}

def success(message: str) -> None:
    """Display a success message"""
    console.print(f"[bold {COLORS['success']}]{message}[/bold {COLORS['success']}]")

def error(message: str) -> None:
    """Display an error message"""
    console.print(f"[bold {COLORS['error']}]{message}[/bold {COLORS['error']}]")

def warning(message: str) -> None:
    """Display a warning message"""
    console.print(f"[bold {COLORS['warning']}]{message}[/bold {COLORS['warning']}]")

def info(message: str) -> None:
    """Display an info message"""
    console.print(f"[bold {COLORS['info']}]{message}[/bold {COLORS['info']}]")

def create_status_table(title: str, data: List[Dict[str, Any]]) -> Table:
    """Create a formatted status table"""
    table = Table(title=title, show_header=True, header_style="bold magenta")
    
    if data:
        # Add columns based on first item keys
        for key in data[0].keys():
            table.add_column(key.replace("_", " ").title(), style="cyan")
        
        # Add rows
        for item in data:
            table.add_row(*[str(v) for v in item.values()])
    
    return table

def create_progress_bar() -> Progress:
    """Create a customized progress bar"""
    return Progress(
        TextColumn("[bold blue]{task.description}", justify="right"),
        BarColumn(bar_width=None),
        "[progress.percentage]{task.percentage:>3.0f}%",
        "•",
        TimeRemainingColumn(),
        console=console
    )

def display_chunk_info(chunk_data: Dict[str, Any]) -> None:
    """Display information about a document chunk"""
    panel_content = f"""
[bold]File:[/bold] {chunk_data.get('filename', 'Unknown')}
[bold]Chunk:[/bold] {chunk_data.get('chunk_index', 0)} of {chunk_data.get('total_chunks', 0)}
[bold]Pages:[/bold] {chunk_data.get('start_page', 0)}-{chunk_data.get('end_page', 0)}
[bold]Size:[/bold] {chunk_data.get('size', 0):,} bytes
[bold]Tables:[/bold] {chunk_data.get('table_count', 0)}
    """
    
    console.print(Panel(
        panel_content.strip(),
        title="[bold cyan]Chunk Information[/bold cyan]",
        border_style="cyan"
    ))

def display_table_data(headers: List[str], rows: List[List[Any]], title: str = "Extracted Table") -> None:
    """Display extracted table data"""
    table = Table(title=title, show_header=True, header_style="bold magenta")
    
    # Add columns
    for header in headers:
        table.add_column(header, style="cyan", no_wrap=False)
    
    # Add rows (limit to first 10 for display)
    for i, row in enumerate(rows[:10]):
        if i < 10:
            table.add_row(*[str(cell) for cell in row])
    
    if len(rows) > 10:
        table.add_row(*["..." for _ in headers])
        table.add_row(*[f"({len(rows)} total rows)" for _ in headers])
    
    console.print(table)

def display_file_tree(directory: str, pattern: str = "*.pdf") -> None:
    """Display a tree view of files"""
    tree = Tree(f"[bold cyan]{directory}[/bold cyan]")
    
    # In real implementation, walk the directory
    # For demo, showing example structure
    docs = tree.add("[yellow]📁 documents[/yellow]")
    docs.add("[green]📄 report1.pdf[/green]")
    docs.add("[green]📄 report2.pdf[/green]")
    
    tables = tree.add("[yellow]📁 tables[/yellow]")
    tables.add("[blue]📊 extracted_tables.csv[/blue]")
    
    console.print(tree)

def create_dashboard() -> Layout:
    """Create a dashboard layout for monitoring"""
    layout = Layout()
    
    layout.split_column(
        Layout(name="header", size=3),
        Layout(name="body"),
        Layout(name="footer", size=3)
    )
    
    layout["body"].split_row(
        Layout(name="left"),
        Layout(name="right")
    )
    
    # Header
    layout["header"].update(Panel(
        "[bold cyan]CHONKER Processing Dashboard[/bold cyan]",
        style="cyan"
    ))
    
    # Footer
    layout["footer"].update(Panel(
        f"[dim]Last updated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}[/dim]",
        style="dim"
    ))
    
    return layout

def display_code(code: str, language: str = "python", title: Optional[str] = None) -> None:
    """Display syntax-highlighted code"""
    syntax = Syntax(
        code,
        language,
        theme="monokai",
        line_numbers=True,
        word_wrap=True
    )
    
    if title:
        console.print(Panel(syntax, title=title, border_style="cyan"))
    else:
        console.print(syntax)

def create_comparison_table(before: Dict[str, Any], after: Dict[str, Any], title: str = "Comparison") -> Table:
    """Create a before/after comparison table"""
    table = Table(title=title, show_header=True)
    table.add_column("Metric", style="cyan", no_wrap=True)
    table.add_column("Before", style="red")
    table.add_column("After", style="green")
    table.add_column("Change", style="yellow")
    
    for key in before.keys():
        if key in after:
            before_val = before[key]
            after_val = after[key]
            
            # Calculate change for numeric values
            if isinstance(before_val, (int, float)) and isinstance(after_val, (int, float)):
                change = after_val - before_val
                change_str = f"+{change}" if change > 0 else str(change)
            else:
                change_str = "→"
            
            table.add_row(
                key.replace("_", " ").title(),
                str(before_val),
                str(after_val),
                change_str
            )
    
    return table

def live_progress_example():
    """Example of live updating display"""
    table = Table(title="Processing Status")
    table.add_column("File", style="cyan")
    table.add_column("Status", style="yellow")
    table.add_column("Progress", style="green")
    
    with Live(table, console=console, refresh_per_second=4) as live:
        for i in range(10):
            table.add_row(f"file_{i}.pdf", "Processing...", f"{i*10}%")
            live.update(table)
            import time
            time.sleep(0.5)

def format_size(bytes: int) -> str:
    """Format bytes to human readable size"""
    for unit in ['B', 'KB', 'MB', 'GB', 'TB']:
        if bytes < 1024.0:
            return f"{bytes:.2f} {unit}"
        bytes /= 1024.0
    return f"{bytes:.2f} PB"

def create_stats_panel(stats: Dict[str, Any]) -> Panel:
    """Create a statistics panel"""
    stats_text = "\n".join([
        f"[bold]{k.replace('_', ' ').title()}:[/bold] {v}"
        for k, v in stats.items()
    ])
    
    return Panel(
        stats_text,
        title="[bold cyan]Statistics[/bold cyan]",
        border_style="cyan",
        padding=(1, 2)
    )

# Utility function for styled prompts
def styled_input(prompt: str, default: Optional[str] = None) -> str:
    """Get styled user input"""
    if default:
        console.print(f"[bold cyan]{prompt}[/bold cyan] [dim](default: {default})[/dim]: ", end="")
    else:
        console.print(f"[bold cyan]{prompt}[/bold cyan]: ", end="")
    
    value = input()
    return value or default

if __name__ == "__main__":
    # Demo the display utilities
    console.print("[bold cyan]CHONKER Display Utilities Demo[/bold cyan]\n")
    
    # Messages
    success("✅ Success message example")
    error("❌ Error message example")
    warning("⚠️  Warning message example")
    info("ℹ️  Info message example")
    
    # Status table
    print("\n")
    status_data = [
        {"file": "doc1.pdf", "status": "✅ Complete", "chunks": 5},
        {"file": "doc2.pdf", "status": "⏳ Processing", "chunks": 3},
        {"file": "doc3.pdf", "status": "❌ Failed", "chunks": 0},
    ]
    table = create_status_table("Processing Status", status_data)
    console.print(table)
    
    # Chunk info
    print("\n")
    chunk_info = {
        "filename": "example.pdf",
        "chunk_index": 2,
        "total_chunks": 5,
        "start_page": 11,
        "end_page": 20,
        "size": 1024000,
        "table_count": 3
    }
    display_chunk_info(chunk_info)
    
    # Code display
    print("\n")
    code_example = '''def process_document(pdf_path: Path) -> List[Chunk]:
    """Process a PDF document into chunks"""
    chunks = []
    with open(pdf_path, 'rb') as f:
        # Processing logic here
        pass
    return chunks'''
    
    display_code(code_example, title="Example Code")
    
    # Stats panel
    print("\n")
    stats = {
        "total_files": 42,
        "processed": 38,
        "failed": 4,
        "total_size": format_size(1024**3 * 2.5),
        "processing_time": "00:15:32"
    }
    console.print(create_stats_panel(stats))