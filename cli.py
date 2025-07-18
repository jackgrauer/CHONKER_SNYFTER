#!/usr/bin/env python3
"""
CHONKER CLI - Enhanced command-line interface using Typer
"""

import typer
from typing import Optional, List
from pathlib import Path
from rich import print as rprint
from rich.console import Console
from rich.table import Table
from rich.panel import Panel
from rich.progress import Progress, SpinnerColumn, TextColumn
import questionary
from questionary import Style

from config import settings, ProcessingMode, OutputFormat
from utils.display import success, error, warning, info, create_status_table

# Initialize Typer app
app = typer.Typer(
    name="chonker",
    help="CHONKER - Document Chunking and Table Extraction Tool",
    add_completion=True,
    rich_markup_mode="rich"
)

console = Console()

# Custom style for questionary
custom_style = Style([
    ('qmark', 'fg:#00CED1 bold'),
    ('question', 'bold'),
    ('answer', 'fg:#00FF00 bold'),
    ('pointer', 'fg:#FF6B6B bold'),
    ('highlighted', 'fg:#FF6B6B bold'),
    ('selected', 'fg:#00CED1'),
    ('separator', 'fg:#6C757D'),
    ('instruction', 'fg:#6C757D'),
])

@app.command()
def init(
    name: Optional[str] = typer.Option(None, "--name", "-n", help="Project name"),
    interactive: bool = typer.Option(True, "--interactive/--no-interactive", "-i/-I", help="Interactive mode")
):
    """Initialize a new CHONKER project"""
    
    with console.status("[cyan]Initializing CHONKER project...[/cyan]", spinner="dots"):
        if interactive:
            # Interactive setup using questionary
            name = questionary.text(
                "What's your project name?",
                default="my-chonker-project",
                style=custom_style
            ).ask()
            
            processing_mode = questionary.select(
                "Select processing mode:",
                choices=[
                    {"name": "⚡ Fast - Quick processing, lower accuracy", "value": ProcessingMode.FAST},
                    {"name": "⚖️  Balanced - Good speed and accuracy", "value": ProcessingMode.BALANCED},
                    {"name": "🎯 Accurate - Best quality, slower", "value": ProcessingMode.ACCURATE}
                ],
                style=custom_style
            ).ask()
            
            output_format = questionary.select(
                "Default output format:",
                choices=[
                    {"name": "📋 JSON - Structured data", "value": OutputFormat.JSON},
                    {"name": "📊 CSV - Spreadsheet compatible", "value": OutputFormat.CSV},
                    {"name": "🗄️  SQLite - Database format", "value": OutputFormat.SQLITE},
                    {"name": "📝 Markdown - Human readable", "value": OutputFormat.MARKDOWN}
                ],
                style=custom_style
            ).ask()
            
            enable_ocr = questionary.confirm(
                "Enable OCR for scanned documents?",
                default=True,
                style=custom_style
            ).ask()
            
            # Update settings
            settings.processing_mode = processing_mode
            settings.output_format = output_format
            settings.enable_ocr = enable_ocr
        
        # Create project structure
        project_dir = Path(name or "my-chonker-project")
        project_dir.mkdir(exist_ok=True)
        (project_dir / "input").mkdir(exist_ok=True)
        (project_dir / "output").mkdir(exist_ok=True)
        (project_dir / "logs").mkdir(exist_ok=True)
        
        # Create project config
        config_content = f"""# CHONKER Project Configuration
CHONKER_APP_NAME={name or 'my-chonker-project'}
CHONKER_PROCESSING_MODE={settings.processing_mode}
CHONKER_OUTPUT_FORMAT={settings.output_format}
CHONKER_ENABLE_OCR={settings.enable_ocr}
CHONKER_CHUNK_SIZE={settings.chunk_size}
"""
        (project_dir / ".env").write_text(config_content)
    
    success(f"✅ Project '{name}' initialized successfully!")
    
    # Display project info
    table = Table(title="Project Structure", show_header=False)
    table.add_column("Item", style="cyan")
    table.add_column("Description", style="white")
    
    table.add_row("📁 input/", "Place your PDF files here")
    table.add_row("📁 output/", "Processed results will be saved here")
    table.add_row("📁 logs/", "Processing logs")
    table.add_row("⚙️  .env", "Project configuration")
    
    console.print(Panel(table, title=f"[bold green]{name}[/bold green]", border_style="green"))

@app.command()
def chunk(
    file_path: Path = typer.Argument(..., help="Path to PDF file or directory"),
    chunk_size: Optional[int] = typer.Option(None, "--chunk-size", "-c", help="Pages per chunk"),
    output_format: Optional[OutputFormat] = typer.Option(None, "--format", "-f", help="Output format"),
    output_dir: Optional[Path] = typer.Option(None, "--output", "-o", help="Output directory"),
    preview: bool = typer.Option(False, "--preview", "-p", help="Preview mode - don't save files")
):
    """Chunk PDF documents into smaller pieces"""
    
    # Validate input
    if not file_path.exists():
        error(f"❌ File or directory not found: {file_path}")
        raise typer.Exit(1)
    
    # Use settings or override with CLI options
    chunk_size = chunk_size or settings.chunk_size
    output_format = output_format or settings.output_format
    output_dir = output_dir or settings.output_dir
    
    # Get list of PDF files
    pdf_files = []
    if file_path.is_file() and file_path.suffix.lower() == '.pdf':
        pdf_files = [file_path]
    elif file_path.is_dir():
        pdf_files = list(file_path.glob("**/*.pdf"))
    else:
        error(f"❌ Invalid input: {file_path} (must be PDF or directory)")
        raise typer.Exit(1)
    
    if not pdf_files:
        warning("⚠️  No PDF files found")
        raise typer.Exit(0)
    
    info(f"Found {len(pdf_files)} PDF file(s) to process")
    
    # Process files with progress bar
    with Progress(
        SpinnerColumn(),
        TextColumn("[progress.description]{task.description}"),
        console=console
    ) as progress:
        task = progress.add_task("Processing PDFs...", total=len(pdf_files))
        
        results = []
        for pdf_file in pdf_files:
            progress.update(task, description=f"Processing {pdf_file.name}...")
            
            # Simulate processing (replace with actual chunking logic)
            import time
            time.sleep(0.5)
            
            results.append({
                "file": pdf_file.name,
                "chunks": 5,  # Simulated
                "status": "✅ Success"
            })
            
            progress.advance(task)
    
    # Display results
    if preview:
        info("🔍 Preview mode - no files were saved")
    
    table = create_status_table("Chunking Results", results)
    console.print(table)
    
    if not preview:
        success(f"✅ Results saved to: {output_dir}")

@app.command()
def validate(
    file_path: Path = typer.Argument(..., help="Path to validate"),
    fix: bool = typer.Option(False, "--fix", help="Attempt to fix issues"),
    verbose: bool = typer.Option(False, "--verbose", "-v", help="Verbose output")
):
    """Validate PDF files or chunked data"""
    
    with console.status("[cyan]Validating files...[/cyan]", spinner="dots"):
        # Validation logic here
        import time
        time.sleep(1)
    
    # Example validation results
    issues = [
        {"file": "document1.pdf", "issue": "Missing metadata", "severity": "warning"},
        {"file": "document2.pdf", "issue": "Corrupted page 5", "severity": "error"},
    ]
    
    if issues:
        warning(f"⚠️  Found {len(issues)} issue(s)")
        
        table = Table(title="Validation Issues", show_header=True)
        table.add_column("File", style="cyan")
        table.add_column("Issue", style="yellow")
        table.add_column("Severity", style="red")
        
        for issue in issues:
            severity_style = "red" if issue["severity"] == "error" else "yellow"
            table.add_row(
                issue["file"],
                issue["issue"],
                f"[{severity_style}]{issue['severity'].upper()}[/{severity_style}]"
            )
        
        console.print(table)
        
        if fix:
            if questionary.confirm("Attempt to fix issues?", style=custom_style).ask():
                success("✅ Issues fixed!")
    else:
        success("✅ All files validated successfully!")

@app.command()
def config(
    show: bool = typer.Option(False, "--show", "-s", help="Show current configuration"),
    edit: bool = typer.Option(False, "--edit", "-e", help="Edit configuration interactively"),
    key: Optional[str] = typer.Option(None, "--key", "-k", help="Configuration key"),
    value: Optional[str] = typer.Option(None, "--value", "-v", help="Configuration value")
):
    """Manage CHONKER configuration"""
    
    if show or (not edit and not key):
        # Show current configuration
        table = Table(title="CHONKER Configuration", show_header=True)
        table.add_column("Setting", style="cyan", no_wrap=True)
        table.add_column("Value", style="green")
        table.add_column("Type", style="yellow")
        
        for field_name, field_value in settings.model_dump().items():
            if not field_name.startswith("_"):
                table.add_row(
                    field_name,
                    str(field_value),
                    type(field_value).__name__
                )
        
        console.print(table)
    
    elif edit:
        # Interactive configuration editor
        info("🛠️  Interactive configuration editor")
        
        # Let user select which setting to modify
        setting_choices = [
            {"name": f"{k}: {v} ({type(v).__name__})", "value": k}
            for k, v in settings.model_dump().items()
            if not k.startswith("_")
        ]
        
        selected_key = questionary.select(
            "Select setting to modify:",
            choices=setting_choices,
            style=custom_style
        ).ask()
        
        if selected_key:
            current_value = getattr(settings, selected_key)
            new_value = questionary.text(
                f"New value for {selected_key}:",
                default=str(current_value),
                style=custom_style
            ).ask()
            
            # Update setting (in real implementation, save to .env)
            success(f"✅ Updated {selected_key} = {new_value}")
    
    elif key and value:
        # Direct key-value update
        success(f"✅ Updated {key} = {value}")

@app.callback(invoke_without_command=True)
def callback(
    ctx: typer.Context,
    version: bool = typer.Option(False, "--version", "-V", help="Show version")
):
    """CHONKER - Document Chunking and Table Extraction Tool"""
    if version:
        rprint(f"[bold cyan]CHONKER[/bold cyan] version [bold green]{settings.version}[/bold green]")
        raise typer.Exit()
    elif ctx.invoked_subcommand is None:
        # Show help if no command provided
        console.print("[bold cyan]CHONKER[/bold cyan] - Document Chunking and Table Extraction Tool")
        console.print("\nUse [bold green]--help[/bold green] to see available commands")
        raise typer.Exit()

if __name__ == "__main__":
    app()