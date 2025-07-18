#!/usr/bin/env python3
"""
Demo script to showcase the CHONKER development enhancements
"""

from rich.console import Console
from rich.panel import Panel
from rich.syntax import Syntax
import time

from config.settings import settings
from utils.display import (
    success, error, warning, info,
    create_status_table, display_chunk_info,
    create_stats_panel, format_size
)

console = Console()

def main():
    # Header
    console.print(Panel.fit(
        "[bold cyan]CHONKER Development Enhancements Demo[/bold cyan]\n"
        "[dim]Showcasing local development tools[/dim]",
        border_style="cyan"
    ))
    
    # 1. Configuration Management
    console.print("\n[bold yellow]1. Configuration Management with Pydantic Settings[/bold yellow]")
    info("Settings are loaded from environment variables and .env files")
    
    config_demo = f"""
Current Configuration:
- App Name: {settings.app_name}
- Processing Mode: {settings.processing_mode}
- Chunk Size: {settings.chunk_size} pages
- Output Format: {settings.output_format}
- Theme Color: {settings.theme_color}
"""
    console.print(Panel(config_demo, title="Settings", border_style="green"))
    
    # 2. Rich Display Utilities
    console.print("\n[bold yellow]2. Rich Display Utilities[/bold yellow]")
    success("✅ This is a success message")
    warning("⚠️  This is a warning message")
    error("❌ This is an error message")
    info("ℹ️  This is an info message")
    
    # 3. Status Tables
    console.print("\n[bold yellow]3. Beautiful Tables[/bold yellow]")
    
    results = [
        {"file": "report2024.pdf", "pages": 125, "chunks": 13, "tables": 8, "status": "✅"},
        {"file": "invoice_batch.pdf", "pages": 45, "chunks": 5, "tables": 15, "status": "✅"},
        {"file": "manual_scan.pdf", "pages": 89, "chunks": 9, "tables": 3, "status": "⚠️"},
    ]
    
    table = create_status_table("Processing Results", results)
    console.print(table)
    
    # 4. Chunk Information Display
    console.print("\n[bold yellow]4. Chunk Information Display[/bold yellow]")
    
    chunk_data = {
        "filename": "quarterly_report.pdf",
        "chunk_index": 3,
        "total_chunks": 8,
        "start_page": 21,
        "end_page": 30,
        "size": 2457600,
        "table_count": 4
    }
    display_chunk_info(chunk_data)
    
    # 5. Statistics Panel
    console.print("\n[bold yellow]5. Statistics Panel[/bold yellow]")
    
    stats = {
        "total_documents": 156,
        "processed_today": 23,
        "tables_extracted": 342,
        "total_size": format_size(1024**3 * 5.7),
        "processing_time": "02:34:15",
        "success_rate": "97.4%"
    }
    console.print(create_stats_panel(stats))
    
    # 6. CLI Commands Preview
    console.print("\n[bold yellow]6. CLI Commands (via Typer)[/bold yellow]")
    
    cli_examples = """
# Initialize a new project
python cli.py init --name my-project

# Chunk PDFs with custom settings
python cli.py chunk ./documents --chunk-size 20 --format json

# Validate files
python cli.py validate ./output --fix

# Interactive configuration
python cli.py config --edit

# Show current configuration
python cli.py config --show
"""
    
    syntax = Syntax(cli_examples, "bash", theme="monokai", line_numbers=False)
    console.print(Panel(syntax, title="CLI Examples", border_style="blue"))
    
    # 7. Interactive Features
    console.print("\n[bold yellow]7. Interactive Features (via Questionary)[/bold yellow]")
    info("The CLI includes interactive prompts for:")
    console.print("  • Project initialization wizard")
    console.print("  • Configuration editor")
    console.print("  • Processing mode selection")
    console.print("  • Confirmation dialogs")
    
    # Footer
    console.print("\n")
    console.print(Panel.fit(
        "[bold green]✨ All enhancements use local packages only![/bold green]\n"
        "[dim]No API keys or external services required[/dim]",
        border_style="green"
    ))

if __name__ == "__main__":
    main()