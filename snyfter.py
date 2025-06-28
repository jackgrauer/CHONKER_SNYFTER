#!/usr/bin/env python3
"""
🐭 SNYFTER v9.1 - Enhanced Data Extraction from CHONKER Chunks
===============================================================================
Enhanced implementation with real chunk loading and improved extraction capabilities.
API key required - no fallback classification.
"""

import os
import sys
import pandas as pd
import requests
import json
from pathlib import Path
from typing import Dict, List, Optional, Any, Tuple
import re
from datetime import datetime

# Rich imports for UI
from rich.console import Console
from rich.panel import Panel
from rich.prompt import Prompt, Confirm
from rich.table import Table
from rich.progress import Progress, SpinnerColumn, TextColumn
from rich.syntax import Syntax

console = Console()

class SnyfterData:
    """Holds the current state of extraction pipeline"""
    
    def __init__(self):
        self.chunks: List[Tuple[str, str]] = []  # (chunk_id, content)
        self.classifications: Dict[str, str] = {}
        self.datasets: Dict[str, pd.DataFrame] = {}
        self.config: Dict[str, Any] = {}
        self.chunk_metadata: Dict[str, Any] = {}

class ChunkLoader:
    """Step 1: Load chunks from CHONKER output"""
    
    def __init__(self):
        # Get user home directory
        home = Path.home()
        
        self.chunk_dirs = [
            # Absolute path in user's home
            home / "saved_chonker_chunks",
            # Current directory paths
            Path("./saved_chonker_chunks"),
            Path("./chunks"),
            Path("../saved_chonker_chunks"),
            Path("./chonker_output"),
            # Common absolute paths
            Path("/Users/jack/saved_chonker_chunks"),  # Your specific path
        ]
        
        # Add custom path from environment variable if set
        custom_path = os.getenv("CHONKER_OUTPUT_DIR")
        if custom_path:
            self.chunk_dirs.insert(0, Path(custom_path))
    
    def load_chunks(self, chunk_numbers: Optional[List[int]] = None) -> List[Tuple[str, str]]:
        """Load all chunk content from CHONKER output"""
        console.print("[cyan]📄 Loading CHONKER chunks...[/cyan]")
        chunks = []
        
        # Find chunk directory
        chunk_dir = None
        for dir_path in self.chunk_dirs:
            if dir_path.exists() and dir_path.is_dir():
                chunk_dir = dir_path
                console.print(f"[green]✓ Found chunk directory: {chunk_dir}[/green]")
                break
        
        if not chunk_dir:
            console.print("[red]❌ No chunk directory found![/red]")
            console.print("[yellow]Expected locations:[/yellow]")
            for dir_path in self.chunk_dirs:
                console.print(f"  - {dir_path}")
            console.print("[dim]Set CHONKER_OUTPUT_DIR environment variable to specify custom location[/dim]")
            return []
        
        # Load chunks from files
        chunk_files = sorted(chunk_dir.glob("chunk_*.txt"))
        
        if not chunk_files:
            console.print(f"[red]❌ No chunk files found in {chunk_dir}[/red]")
            return []
        
        # Filter chunks if specific numbers requested
        if chunk_numbers:
            filtered_files = []
            for num in chunk_numbers:
                # Try to find chunk file with this number
                for cf in chunk_files:
                    if f"chunk_{num}" in cf.stem or f"chunk_{num:03d}" in cf.stem:
                        filtered_files.append(cf)
                        break
            chunk_files = filtered_files
            
            if not chunk_files:
                console.print(f"[red]❌ No chunks found for numbers: {chunk_numbers}[/red]")
                return []
        
        with Progress(
            SpinnerColumn(),
            TextColumn("[progress.description]{task.description}"),
            console=console
        ) as progress:
            task = progress.add_task(f"Loading {len(chunk_files)} chunks...", total=len(chunk_files))
            
            for chunk_file in chunk_files:
                try:
                    with open(chunk_file, 'r', encoding='utf-8') as f:
                        content = f.read()
                        chunk_id = chunk_file.stem
                        chunks.append((chunk_id, content))
                        progress.advance(task)
                except Exception as e:
                    console.print(f"[red]❌ Error loading {chunk_file}: {e}[/red]")
        
        console.print(f"[green]✅ Loaded {len(chunks)} chunks successfully[/green]")
        return chunks

class ChunkClassifier:
    """Step 2: Classify chunks using Claude API with adaptive schema discovery"""
    
    def __init__(self):
        # Claude API configuration
        self.api_key = os.getenv("ANTHROPIC_API_KEY", "")
        self.api_url = "https://api.anthropic.com/v1/messages"
        self.model = "claude-3-5-sonnet-20241022"
        
        # Seed schema - minimal starting types that expand during discovery
        self.discovered_schema = [
            "narrative_text",
            "data_table", 
            "metadata_block"
        ]
        
        # Track classification confidence and reasoning
        self.classification_log = []
        
        # Check API key availability
        if not self.api_key:
            self._show_api_key_help()
            raise ValueError("ANTHROPIC_API_KEY is required for classification")
    
    def _show_api_key_help(self):
        """Show detailed help for setting up API key"""
        console.print("[red]❌ ANTHROPIC_API_KEY not found![/red]")
        console.print("\n[yellow]How to set your API key:[/yellow]")
        console.print("1. Get your API key from: https://console.anthropic.com/")
        console.print("2. Set it as an environment variable:")
        console.print("   [cyan]export ANTHROPIC_API_KEY=sk-ant-...[/cyan]")
        console.print("   Or add to your ~/.bashrc or ~/.zshrc")
        console.print("\n[yellow]Current environment check:[/yellow]")
        
        # Check various possible locations
        env_vars_to_check = [
            "ANTHROPIC_API_KEY",
            "CLAUDE_API_KEY", 
            "ANTHROPIC_KEY",
            "API_KEY"
        ]
        
        found_any = False
        for var in env_vars_to_check:
            value = os.getenv(var)
            if value:
                console.print(f"  ✓ {var}: {value[:10]}...")
                found_any = True
            else:
                console.print(f"  ✗ {var}: not set")
        
        if not found_any:
            console.print("\n[red]No API key environment variables found.[/red]")
        
        console.print(f"\n[dim]Current working directory: {os.getcwd()}[/dim]")
        console.print(f"[dim]Python executable: {sys.executable}[/dim]")
    
    def classify_chunks(self, chunks: List[Tuple[str, str]]) -> Dict[str, str]:
        """
        Pass 1: Adaptive schema discovery through sequential content analysis.
        
        The LLM processes chunks sequentially, discovering new data patterns and 
        expanding its classification vocabulary as it encounters different content types.
        Early chunks help identify basic patterns, later chunks benefit from the 
        accumulated knowledge of document structure.
        
        Result: A custom taxonomy of data types specific to this document.
        """
        console.print("[cyan]🔍 Pass 1: Discovering document data patterns...[/cyan]")
        
        classifications = {}
        
        # Process chunks sequentially to build up schema knowledge
        with Progress(
            SpinnerColumn(),
            TextColumn("[progress.description]{task.description}"),
            console=console
        ) as progress:
            task = progress.add_task("Analyzing chunks...", total=len(chunks))
            
            for i, (chunk_id, chunk_content) in enumerate(chunks):
                # Create classification prompt with current schema
                classification_result = self._classify_single_chunk(chunk_content, i)
                
                # Store result
                classifications[chunk_id] = classification_result['type']
                self.classification_log.append(classification_result)
                
                # Expand schema if new pattern discovered
                if classification_result['is_new_type']:
                    self.discovered_schema.append(classification_result['type'])
                    console.print(f"[green]📋 Discovered new data type: {classification_result['type']}[/green]")
                
                progress.advance(task)
        
        # Show final discovered schema
        self._display_schema_summary()
        
        return classifications
    
    def _classify_single_chunk(self, chunk: str, chunk_index: int) -> Dict[str, Any]:
        """Classify a single chunk using Claude API"""
        
        try:
            # Build classification prompt
            prompt = self._build_classification_prompt(chunk, chunk_index)
            
            # Call Claude API
            headers = {
                "Content-Type": "application/json",
                "x-api-key": self.api_key,
                "anthropic-version": "2023-06-01"
            }
            
            payload = {
                "model": self.model,
                "max_tokens": 500,
                "messages": [
                    {
                        "role": "user",
                        "content": prompt
                    }
                ]
            }
            
            response = requests.post(self.api_url, headers=headers, json=payload)
            
            if response.status_code == 200:
                result = response.json()
                classification_text = result['content'][0]['text']
                
                # Parse JSON response
                try:
                    classification_data = json.loads(classification_text)
                    
                    # Check if this is a new type
                    is_new = classification_data['type'] not in self.discovered_schema
                    
                    return {
                        'type': classification_data['type'],
                        'confidence': classification_data.get('confidence', 0.0),
                        'reasoning': classification_data.get('reasoning', ''),
                        'key_entities': classification_data.get('key_entities', []),
                        'is_new_type': is_new
                    }
                    
                except json.JSONDecodeError:
                    console.print(f"[red]❌ Invalid JSON from Claude for chunk {chunk_index}[/red]")
                    console.print(f"[dim]Response: {classification_text[:200]}...[/dim]")
                    raise ValueError(f"Invalid JSON response from Claude API")
                    
            else:
                console.print(f"[red]❌ API Error {response.status_code} for chunk {chunk_index}[/red]")
                try:
                    error_details = response.json()
                    console.print(f"[red]Error details: {error_details}[/red]")
                except:
                    console.print(f"[red]Response: {response.text[:200]}...[/red]")
                raise ValueError(f"Claude API returned status {response.status_code}")
                
        except Exception as e:
            console.print(f"[red]❌ Classification error for chunk {chunk_index}: {e}[/red]")
            raise
    
    def _build_classification_prompt(self, chunk: str, chunk_index: int) -> str:
        """Build Claude prompt for classification with schema discovery"""
        
        # Context from previous chunks
        context_types = list(set([log['type'] for log in self.classification_log[-3:]]))
        
        # Limit chunk size for prompt
        chunk_preview = chunk[:2000] + "..." if len(chunk) > 2000 else chunk
        
        prompt = f"""TASK: Classify this document chunk for data extraction.

DISCOVERED DATA TYPES SO FAR: {', '.join(self.discovered_schema)}
RECENT CONTEXT: {', '.join(context_types) if context_types else 'Beginning of document'}

CHUNK {chunk_index + 1} CONTENT:
{chunk_preview}

INSTRUCTIONS:
1. If this chunk matches a KNOWN TYPE from the discovered list, use that exact name
2. If this is a NEW PATTERN not yet discovered, create a descriptive type name like:
   - "environmental_lab_results" 
   - "monitoring_well_coordinates"
   - "regulatory_standards_table"
   - "cost_breakdown_data"
   - "soil_contamination_measurements"

3. Focus on DATA CONTENT, not formatting. Look for:
   - What kind of measurements/values are present?
   - What domain does this data serve (environmental, financial, etc.)?
   - What would someone want to extract from this?

4. Be specific but consistent. If you see similar patterns, use the same type name.

RESPOND WITH JSON ONLY:
{{
  "type": "exact_type_name",
  "confidence": 0.0-1.0,
  "reasoning": "why this classification fits",
  "key_entities": ["list", "of", "important", "data", "elements"]
}}"""
        return prompt
    
    def _display_schema_summary(self):
        """Display the discovered document schema"""
        console.print(f"\n[bold green]📋 Document Schema Discovery Complete[/bold green]")
        
        schema_table = Table(title="Discovered Data Types")
        schema_table.add_column("Type", style="cyan")
        schema_table.add_column("Count", style="yellow")
        schema_table.add_column("Avg Confidence", style="green")
        schema_table.add_column("Key Entities", style="dim")
        
        # Count occurrences of each type
        type_counts = {}
        type_confidences = {}
        type_entities = {}
        
        for log_entry in self.classification_log:
            data_type = log_entry['type']
            type_counts[data_type] = type_counts.get(data_type, 0) + 1
            
            if data_type not in type_confidences:
                type_confidences[data_type] = []
            type_confidences[data_type].append(log_entry['confidence'])
            
            # Collect unique entities for this type
            if data_type not in type_entities:
                type_entities[data_type] = set()
            type_entities[data_type].update(log_entry.get('key_entities', []))
        
        # Display schema summary
        for data_type in self.discovered_schema:
            count = type_counts.get(data_type, 0)
            if count > 0:  # Only show types that actually appeared
                avg_confidence = sum(type_confidences.get(data_type, [0])) / max(len(type_confidences.get(data_type, [1])), 1)
                entities = list(type_entities.get(data_type, []))[:3]  # Show first 3 entities
                entities_str = ', '.join(entities) + ('...' if len(type_entities.get(data_type, [])) > 3 else '')
                
                schema_table.add_row(
                    data_type,
                    str(count),
                    f"{avg_confidence:.2f}",
                    entities_str
                )
        
        console.print(schema_table)
        
        # Show schema evolution
        new_types = [log['type'] for log in self.classification_log if log['is_new_type']]
        if new_types:
            console.print(f"\n[green]🔍 New types discovered: {', '.join(set(new_types))}[/green]")
        
        console.print(f"[dim]💡 This document contains {len([t for t in type_counts if type_counts[t] > 0])} distinct data types[/dim]")

class DataExtractor:
    """Step 3: Extract structured data from classified chunks"""
    
    def __init__(self):
        self.api_key = os.getenv("ANTHROPIC_API_KEY", "")
        self.api_url = "https://api.anthropic.com/v1/messages"
        self.model = "claude-3-5-sonnet-20241022"
    
    def extract_data(self, chunks: List[Tuple[str, str]], classifications: Dict[str, str], 
                    config: Dict[str, Any]) -> Dict[str, pd.DataFrame]:
        """Pass 2: Extract structured data based on classifications"""
        console.print("[cyan]🔬 Pass 2: Extracting structured data...[/cyan]")
        
        # Group chunks by classification type
        chunks_by_type = {}
        for chunk_id, chunk_content in chunks:
            chunk_type = classifications.get(chunk_id, 'unknown')
            if chunk_type not in chunks_by_type:
                chunks_by_type[chunk_type] = []
            chunks_by_type[chunk_type].append((chunk_id, chunk_content))
        
        datasets = {}
        
        # Extract data from each type
        for data_type, typed_chunks in chunks_by_type.items():
            if data_type in ['narrative_text', 'metadata_block']:
                continue  # Skip non-data chunks
            
            console.print(f"[dim]Extracting from {len(typed_chunks)} {data_type} chunks...[/dim]")
            
            # Use appropriate extraction method
            if 'environmental' in data_type:
                df = self._extract_environmental_data(typed_chunks)
                if not df.empty:
                    datasets['environmental_data'] = df
            elif 'financial' in data_type:
                df = self._extract_financial_data(typed_chunks)
                if not df.empty:
                    datasets['financial_data'] = df
            elif 'table' in data_type:
                df = self._extract_table_data(typed_chunks)
                if not df.empty:
                    datasets[f'{data_type}_extracted'] = df
        
        if not datasets:
            console.print("[yellow]⚠️ No structured data could be extracted[/yellow]")
            console.print("[dim]This might happen if chunks don't contain recognizable data patterns[/dim]")
        
        return datasets
    
    def _extract_environmental_data(self, chunks: List[Tuple[str, str]]) -> pd.DataFrame:
        """Extract environmental measurement data"""
        data_rows = []
        
        # Pattern matching for environmental data
        measurement_pattern = r'([A-Za-z\s]+):\s*([\d.]+)\s*(mg/L|mg/kg|ppm|ppb|μg/L|µg/L)'
        location_pattern = r'(Location|Site|Well|Sample):\s*([A-Za-z0-9\s\-]+)'
        date_pattern = r'(Date|Sampled):\s*(\d{4}-\d{2}-\d{2}|\d{1,2}/\d{1,2}/\d{4})'
        
        for chunk_id, content in chunks:
            # Extract measurements
            measurements = re.findall(measurement_pattern, content)
            
            # Extract location
            location_match = re.search(location_pattern, content)
            location = location_match.group(2).strip() if location_match else chunk_id
            
            # Extract date
            date_match = re.search(date_pattern, content)
            date = date_match.group(2) if date_match else None
            
            # Create rows for each measurement
            for analyte, value, unit in measurements:
                data_rows.append({
                    'location': location,
                    'analyte': analyte.strip(),
                    'value': float(value),
                    'unit': unit,
                    'date': date,
                    'source_chunk': chunk_id
                })
        
        if data_rows:
            return pd.DataFrame(data_rows)
        return pd.DataFrame()
    
    def _extract_financial_data(self, chunks: List[Tuple[str, str]]) -> pd.DataFrame:
        """Extract financial data"""
        data_rows = []
        
        # Pattern matching for financial data
        money_pattern = r'([A-Za-z\s]+):\s*\$?([\d,]+(?:\.\d{2})?)'
        percentage_pattern = r'([A-Za-z\s]+):\s*([\d.]+)%'
        
        for chunk_id, content in chunks:
            # Extract monetary values
            money_matches = re.findall(money_pattern, content)
            for item, value in money_matches:
                if any(keyword in item.lower() for keyword in ['cost', 'budget', 'expense', 'revenue', 'allocated']):
                    data_rows.append({
                        'item': item.strip(),
                        'amount': float(value.replace(',', '')),
                        'type': 'monetary',
                        'source_chunk': chunk_id
                    })
            
            # Extract percentages
            percent_matches = re.findall(percentage_pattern, content)
            for item, value in percent_matches:
                data_rows.append({
                    'item': item.strip(),
                    'amount': float(value),
                    'type': 'percentage',
                    'source_chunk': chunk_id
                })
        
        if data_rows:
            return pd.DataFrame(data_rows)
        return pd.DataFrame()
    
    def _extract_table_data(self, chunks: List[Tuple[str, str]]) -> pd.DataFrame:
        """Extract generic table data"""
        all_dfs = []
        
        for chunk_id, content in chunks:
            # Try to parse as CSV-like data
            lines = content.strip().split('\n')
            if len(lines) > 1:
                # Detect delimiter
                delimiters = ['\t', '|', ',']
                delimiter = None
                for d in delimiters:
                    if d in lines[0]:
                        delimiter = d
                        break
                
                if delimiter:
                    try:
                        # Parse table
                        rows = []
                        headers = [h.strip() for h in lines[0].split(delimiter)]
                        
                        for line in lines[1:]:
                            if line.strip():
                                values = [v.strip() for v in line.split(delimiter)]
                                if len(values) == len(headers):
                                    rows.append(values)
                        
                        if rows:
                            df = pd.DataFrame(rows, columns=headers)
                            df['source_chunk'] = chunk_id
                            all_dfs.append(df)
                    except:
                        pass
        
        if all_dfs:
            return pd.concat(all_dfs, ignore_index=True)
        return pd.DataFrame()

class ConfigManager:
    """Step 4: Handle custom extraction instructions"""
    
    def __init__(self):
        self.config_file = Path("snyfter_config.json")
    
    def setup_config(self) -> Dict[str, Any]:
        """Interactive config setup"""
        console.print("[cyan]🔧 Setting up extraction configuration...[/cyan]")
        
        config = {
            "extraction_focus": "environmental data",
            "priority_entities": ["sample_ids", "concentrations"],
            "custom_instructions": "",
            "output_format": "csv"
        }
        
        # Check if config file exists
        if self.config_file.exists():
            if Confirm.ask("[yellow]Found existing config. Load it?[/yellow]"):
                try:
                    with open(self.config_file, 'r') as f:
                        config = json.load(f)
                    console.print("[green]✅ Loaded existing configuration[/green]")
                    return config
                except:
                    console.print("[red]❌ Error loading config, using defaults[/red]")
        
        # Interactive configuration
        console.print("\n[bold]Configure extraction settings:[/bold]")
        
        # Extraction focus
        focus_options = ["environmental data", "financial data", "all data types", "custom"]
        console.print("\nExtraction focus options:")
        for i, option in enumerate(focus_options, 1):
            console.print(f"  {i}. {option}")
        
        choice = Prompt.ask("Select extraction focus", choices=["1", "2", "3", "4"], default="1")
        config["extraction_focus"] = focus_options[int(choice) - 1]
        
        if config["extraction_focus"] == "custom":
            config["extraction_focus"] = Prompt.ask("Enter custom extraction focus")
        
        # Priority entities
        entities = Prompt.ask(
            "Priority entities (comma-separated)",
            default=", ".join(config["priority_entities"])
        )
        config["priority_entities"] = [e.strip() for e in entities.split(",")]
        
        # Custom instructions
        if Confirm.ask("Add custom extraction instructions?", default=False):
            console.print("[dim]Enter instructions (press Enter twice to finish):[/dim]")
            lines = []
            while True:
                line = input()
                if line:
                    lines.append(line)
                else:
                    break
            config["custom_instructions"] = "\n".join(lines)
        
        # Output format
        format_choice = Prompt.ask(
            "Output format",
            choices=["csv", "excel", "json"],
            default="csv"
        )
        config["output_format"] = format_choice
        
        # Save configuration
        if Confirm.ask("Save configuration for future use?", default=True):
            try:
                with open(self.config_file, 'w') as f:
                    json.dump(config, f, indent=2)
                console.print(f"[green]✅ Configuration saved to {self.config_file}[/green]")
            except Exception as e:
                console.print(f"[red]❌ Error saving config: {e}[/red]")
        
        return config

class DataExporter:
    """Step 5: Export datasets to files"""
    
    def __init__(self):
        self.output_dir = Path("snyfter_output")
        self.output_dir.mkdir(exist_ok=True)
    
    def export_datasets(self, datasets: Dict[str, pd.DataFrame], format_type: str = "csv"):
        """Export datasets to Python-ready files"""
        console.print(f"[cyan]📊 Exporting {len(datasets)} datasets as {format_type}...[/cyan]")
        
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        export_dir = self.output_dir / f"export_{timestamp}"
        export_dir.mkdir(exist_ok=True)
        
        exported_files = []
        
        for name, df in datasets.items():
            try:
                if format_type == "csv":
                    filename = export_dir / f"{name}.csv"
                    df.to_csv(filename, index=False)
                elif format_type == "excel":
                    filename = export_dir / f"{name}.xlsx"
                    df.to_excel(filename, index=False)
                elif format_type == "json":
                    filename = export_dir / f"{name}.json"
                    df.to_json(filename, orient='records', indent=2)
                else:
                    console.print(f"[red]❌ Unknown format: {format_type}[/red]")
                    continue
                
                exported_files.append(filename)
                console.print(f"[green]  ✅ {filename.name} ({len(df)} rows)[/green]")
                
            except Exception as e:
                console.print(f"[red]❌ Error exporting {name}: {e}[/red]")
        
        # Create summary file
        summary_file = export_dir / "extraction_summary.txt"
        with open(summary_file, 'w') as f:
            f.write(f"SNYFTER Extraction Summary\n")
            f.write(f"Generated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}\n")
            f.write(f"{'='*50}\n\n")
            
            for name, df in datasets.items():
                f.write(f"Dataset: {name}\n")
                f.write(f"Rows: {len(df)}\n")
                f.write(f"Columns: {', '.join(df.columns)}\n")
                f.write(f"\n")
        
        console.print(f"\n[bold green]🎉 Export complete![/bold green]")
        console.print(f"[dim]Files saved to: {export_dir}[/dim]")
        
        # Create Python loading script
        self._create_loading_script(export_dir, datasets.keys(), format_type)
    
    def _create_loading_script(self, export_dir: Path, dataset_names: List[str], format_type: str):
        """Create a Python script to load the exported datasets"""
        script_content = f'''#!/usr/bin/env python3
"""
Auto-generated script to load SNYFTER extracted datasets
Generated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}
"""

import pandas as pd
from pathlib import Path

# Define the directory containing the exported files
data_dir = Path(__file__).parent

# Load all datasets
datasets = {{}}

'''
        
        for name in dataset_names:
            if format_type == "csv":
                script_content += f'datasets["{name}"] = pd.read_csv(data_dir / "{name}.csv")\n'
            elif format_type == "excel":
                script_content += f'datasets["{name}"] = pd.read_excel(data_dir / "{name}.xlsx")\n'
            elif format_type == "json":
                script_content += f'datasets["{name}"] = pd.read_json(data_dir / "{name}.json")\n'
        
        script_content += '''
# Display summary
print("Loaded datasets:")
for name, df in datasets.items():
    print(f"  {name}: {len(df)} rows, {len(df.columns)} columns")

# Example usage:
# df = datasets["environmental_data"]
# print(df.head())
'''
        
        script_file = export_dir / "load_datasets.py"
        with open(script_file, 'w') as f:
            f.write(script_content)
        
        console.print(f"[green]  ✅ Created loading script: {script_file.name}[/green]")

class SnyfterUI:
    """Step-by-step UI for building out the pipeline"""
    
    def __init__(self):
        self.data = SnyfterData()
        
        # Components for each step
        self.chunk_loader = ChunkLoader()
        self.classifier = ChunkClassifier()
        self.extractor = DataExtractor()
        self.config_manager = ConfigManager()
        self.exporter = DataExporter()
        
        self.commands = {
            # Step-by-step pipeline
            'load': (self.cmd_load_chunks, "Step 1: Load chunks from CHONKER"),
            'classify': (self.cmd_classify_chunks, "Step 2: Classify chunks with LLM"),
            'extract': (self.cmd_extract_data, "Step 3: Extract structured data"),
            'config': (self.cmd_setup_config, "Step 4: Set custom extraction rules"),
            'export': (self.cmd_export_data, "Step 5: Export datasets to files"),
            
            # Utilities
            'status': (self.cmd_show_status, "Show current pipeline status"),
            'analyze': (self.cmd_analyze_data, "View extracted datasets"),
            'preview': (self.cmd_preview_chunks, "Preview loaded chunks"),
            'apikey': (self.cmd_check_api_key, "Check API key configuration"),
            'reset': (self.cmd_reset, "Reset and reload script"),
            'help': (self.cmd_help, "Show available commands"),
            'exit': (self.cmd_exit, "Exit SNYFTER")
        }
    
    def show_welcome(self):
        """Display welcome message"""
        welcome = Panel(
            "[bold magenta]🐭 SNYFTER v9.1 - Step-by-Step Data Extraction[/bold magenta]\n\n" +
            "[bright_magenta]Build the extraction pipeline one step at a time[/bright_magenta]\n\n" +
            "[yellow]Pipeline Steps:[/yellow]\n" +
            "[dim]1.[/dim] [cyan]load[/cyan]     - Load CHONKER chunks\n" +
            "[dim]2.[/dim] [cyan]classify[/cyan] - Classify content types (LLM Pass 1)\n" +
            "[dim]3.[/dim] [cyan]extract[/cyan]  - Extract structured data (LLM Pass 2)\n" +
            "[dim]4.[/dim] [cyan]config[/cyan]   - Set custom extraction rules\n" +
            "[dim]5.[/dim] [cyan]export[/cyan]   - Save Python-ready datasets\n\n" +
            "[dim]Type 'help' for all commands or 'apikey' to check API setup[/dim]",
            title="Welcome",
            style="bold magenta"
        )
        console.print(welcome)
    
    def cmd_load_chunks(self, *args):
        """Step 1: Load chunks from CHONKER"""
        # Check if specific chunks requested
        if args:
            chunk_spec = args[0]
            if chunk_spec.isdigit():
                # Load single chunk by number
                self.data.chunks = self.chunk_loader.load_chunks(chunk_numbers=[int(chunk_spec)])
            elif '-' in chunk_spec:
                # Load range of chunks (e.g., "1-5")
                start, end = chunk_spec.split('-')
                chunk_numbers = list(range(int(start), int(end) + 1))
                self.data.chunks = self.chunk_loader.load_chunks(chunk_numbers=chunk_numbers)
            else:
                console.print("[yellow]Invalid chunk specification. Use: load [chunk_num] or load [start-end][/yellow]")
                return
        else:
            # Load all chunks
            self.data.chunks = self.chunk_loader.load_chunks()
        
        if self.data.chunks:
            console.print(f"[green]✅ Loaded {len(self.data.chunks)} chunks[/green]")
            console.print("[dim]💡 Next: run 'classify' to classify chunk content types[/dim]")
            console.print("[dim]    or 'preview' to see chunk contents[/dim]")
        else:
            console.print("[red]❌ No chunks found! Run CHONKER first or check paths.[/red]")
    
    def cmd_preview_chunks(self, *args):
        """Preview loaded chunks"""
        if not self.data.chunks:
            console.print("[yellow]⚠️ No chunks loaded. Run 'load' first.[/yellow]")
            return
        
        # Show how many to preview
        num_preview = min(3, len(self.data.chunks))
        console.print(f"\n[cyan]Previewing first {num_preview} chunks:[/cyan]")
        
        for i, (chunk_id, content) in enumerate(self.data.chunks[:num_preview]):
            console.print(f"\n[bold yellow]━━━ {chunk_id} ━━━[/bold yellow]")
            
            # Show first 10 lines or 500 chars
            preview = content[:500] + "..." if len(content) > 500 else content
            lines = preview.split('\n')[:10]
            preview = '\n'.join(lines)
            
            syntax = Syntax(preview, "text", theme="monokai", line_numbers=True)
            console.print(syntax)
        
        if len(self.data.chunks) > num_preview:
            console.print(f"\n[dim]... and {len(self.data.chunks) - num_preview} more chunks[/dim]")
    
    def cmd_classify_chunks(self, *args):
        """Step 2: Classify chunks with LLM"""
        if not self.data.chunks:
            console.print("[yellow]⚠️ No chunks loaded. Run 'load' first.[/yellow]")
            return
        
        try:
            self.data.classifications = self.classifier.classify_chunks(self.data.chunks)
            
            # Show classification results
            console.print(f"[green]✅ Classified {len(self.data.classifications)} chunks[/green]")
            
            # Show summary
            classification_counts = {}
            for classification in self.data.classifications.values():
                classification_counts[classification] = classification_counts.get(classification, 0) + 1
            
            for content_type, count in classification_counts.items():
                console.print(f"[cyan]  📊 {content_type}:[/cyan] {count} chunks")
            
            console.print("[dim]💡 Next: run 'extract' to extract structured data[/dim]")
            
        except ValueError as e:
            console.print(f"[red]❌ {e}[/red]")
            console.print("[dim]💡 Run 'apikey' to check your API key setup[/dim]")
    
    def cmd_extract_data(self, *args):
        """Step 3: Extract structured data"""
        if not self.data.classifications:
            console.print("[yellow]⚠️ No classifications found. Run 'classify' first.[/yellow]")
            return
        
        self.data.datasets = self.extractor.extract_data(
            self.data.chunks, 
            self.data.classifications,
            self.data.config
        )
        
        console.print(f"[green]✅ Extracted {len(self.data.datasets)} datasets[/green]")
        
        # Show dataset summary
        for name, df in self.data.datasets.items():
            console.print(f"[cyan]  📊 {name}:[/cyan] {len(df)} rows, {len(df.columns)} columns")
        
        console.print("[dim]💡 Next: run 'export' to save datasets or 'analyze' to view them[/dim]")
    
    def cmd_setup_config(self, *args):
        """Step 4: Set custom extraction rules"""
        self.data.config = self.config_manager.setup_config()
        console.print("[green]✅ Configuration updated[/green]")
        console.print("[dim]💡 Config will be used in next 'extract' run[/dim]")
    
    def cmd_export_data(self, *args):
        """Step 5: Export datasets to files"""
        if not self.data.datasets:
            console.print("[yellow]⚠️ No datasets found. Run 'extract' first.[/yellow]")
            return
        
        format_type = args[0] if args else self.data.config.get("output_format", "csv")
        self.exporter.export_datasets(self.data.datasets, format_type)
    
    def cmd_check_api_key(self, *args):
        """Check API key configuration"""
        console.print("[cyan]🔑 API Key Configuration Check[/cyan]")
        
        api_key = os.getenv("ANTHROPIC_API_KEY", "")
        if api_key:
            console.print(f"[green]✅ ANTHROPIC_API_KEY found: {api_key[:10]}...[/green]")
            console.print(f"[dim]Length: {len(api_key)} characters[/dim]")
            
            # Check format
            if not api_key.startswith("sk-ant-"):
                console.print("[red]❌ Invalid format! API key should start with 'sk-ant-'[/red]")
                console.print(f"[yellow]Your key starts with: {api_key[:10]}[/yellow]")
                return
            
            # Test API connection
            console.print("\n[cyan]Testing API connection...[/cyan]")
            try:
                headers = {
                    "Content-Type": "application/json",
                    "x-api-key": api_key,
                    "anthropic-version": "2023-06-01"
                }
                
                payload = {
                    "model": "claude-3-5-sonnet-20241022",
                    "max_tokens": 10,
                    "messages": [{"role": "user", "content": "Hello"}]
                }
                
                response = requests.post("https://api.anthropic.com/v1/messages", 
                                       headers=headers, json=payload, timeout=10)
                
                if response.status_code == 200:
                    console.print("[green]✅ API connection successful![/green]")
                elif response.status_code == 401:
                    console.print("[red]❌ 401 Unauthorized - Invalid API key[/red]")
                    console.print("[yellow]Your API key is incorrectly formatted or expired[/yellow]")
                    try:
                        error_details = response.json()
                        console.print(f"[red]Details: {error_details}[/red]")
                    except:
                        console.print(f"[red]Response: {response.text[:200]}[/red]")
                else:
                    console.print(f"[red]❌ API Error {response.status_code}[/red]")
                    try:
                        error_details = response.json()
                        console.print(f"[red]Details: {error_details}[/red]")
                    except:
                        console.print(f"[red]Response: {response.text[:200]}[/red]")
                        
            except Exception as e:
                console.print(f"[red]❌ Connection failed: {e}[/red]")
        else:
            console.print("[red]❌ No ANTHROPIC_API_KEY found[/red]")
            console.print("\n[yellow]Setup instructions:[/yellow]")
            console.print("1. Get API key from: https://console.anthropic.com/")
            console.print("2. Set environment variable:")
            console.print("   [cyan]export ANTHROPIC_API_KEY=sk-ant-...[/cyan]")
            console.print("3. Or add to ~/.bashrc or ~/.zshrc")
            console.print("\n[red]🚨 Common issues:[/red]")
            console.print("• API key must start with 'sk-ant-'")
            console.print("• No spaces or quotes around the key")
            console.print("• Key must be active (check Anthropic console)")
    
    def cmd_show_status(self, *args):
        """Show current pipeline status"""
        status_table = Table(title="🐭 Pipeline Status")
        status_table.add_column("Step", style="cyan")
        status_table.add_column("Status", style="green")
        status_table.add_column("Details", style="dim")
        
        # Check each step
        status_table.add_row(
            "1. Load Chunks",
            "✅ Complete" if self.data.chunks else "⏳ Pending",
            f"{len(self.data.chunks)} chunks" if self.data.chunks else "Run 'load'"
        )
        
        status_table.add_row(
            "2. Classify",
            "✅ Complete" if self.data.classifications else "⏳ Pending", 
            f"{len(self.data.classifications)} classified" if self.data.classifications else "Run 'classify'"
        )
        
        status_table.add_row(
            "3. Extract Data",
            "✅ Complete" if self.data.datasets else "⏳ Pending",
            f"{len(self.data.datasets)} datasets" if self.data.datasets else "Run 'extract'"
        )
        
        status_table.add_row(
            "4. Config",
            "✅ Set" if self.data.config else "⚪ Optional",
            "Custom rules" if self.data.config else "Using defaults"
        )
        
        console.print(status_table)
    
    def cmd_analyze_data(self, *args):
        """View extracted datasets"""
        if not self.data.datasets:
            console.print("[yellow]No datasets available. Run 'extract' first.[/yellow]")
            return
        
        for name, df in self.data.datasets.items():
            console.print(f"\n[bold cyan]📊 Dataset: {name}[/bold cyan]")
            console.print(df.head())
            console.print(f"[dim]Shape: {df.shape[0]} rows × {df.shape[1]} columns[/dim]")
            console.print(f"[dim]Columns: {', '.join(df.columns)}[/dim]")
    
    def cmd_help(self, *args):
        """Show help"""
        table = Table(title="🐭 SNYFTER Commands")
        table.add_column("Command", style="yellow", width=12)
        table.add_column("Description", style="bright_magenta")
        
        for cmd, (func, desc) in self.commands.items():
            table.add_row(cmd, desc)
        
        console.print(table)
        
        console.print("\n[bold magenta]💡 Typical Flow:[/bold magenta]")
        console.print("[dim]apikey → load → classify → extract → export[/dim]")
        console.print("[dim]Use 'status' to see where you are in the pipeline[/dim]")
    
    def cmd_reset(self, *args):
        """Reset by restarting the entire script"""
        console.print("[yellow]🔄 Restarting SNYFTER to load code changes...[/yellow]")
        
        # Get the current script path
        script_path = os.path.abspath(__file__)
        
        # Get the Python executable
        python = sys.executable
        
        # Restart the script
        os.execv(python, [python] + [script_path])
    
    def cmd_exit(self, *args):
        """Exit SNYFTER"""
        return 'exit'
    
    def run(self):
        """Main interactive loop"""
        self.show_welcome()
        
        # Check API key on startup
        api_key = os.getenv("ANTHROPIC_API_KEY", "")
        if not api_key:
            console.print("[yellow]⚠️ No API key detected. Run 'apikey' for setup help.[/yellow]")
        
        console.print("[dim]💡 Start with 'load' to load CHONKER chunks[/dim]\n")
        
        while True:
            try:
                user_input = Prompt.ask("[bold magenta]🐭 snyfter[/bold magenta]", default="").strip()
                
                if not user_input:
                    continue
                
                parts = user_input.split()
                cmd = parts[0].lower()
                args = parts[1:]
                
                if cmd in self.commands:
                    result = self.commands[cmd][0](*args)
                    if result == 'exit':
                        console.print("[magenta]👋 Goodbye![/magenta]")
                        break
                else:
                    console.print(f"[yellow]❓ Unknown command: {cmd}. Type 'help' for commands.[/yellow]")
                    
            except KeyboardInterrupt:
                console.print("\n[yellow]💡 Use 'exit' to quit[/yellow]")
            except Exception as e:
                console.print(f"[red]❌ Error: {e}[/red]")

def main():
    """Entry point"""
    console.print("[bold cyan]🐭 SNYFTER v9.1 - Step-by-Step Data Extraction[/bold cyan]")
    
    try:
        ui = SnyfterUI()
        ui.run()
    except Exception as e:
        console.print(f"[red]❌ Startup error: {e}[/red]")

if __name__ == "__main__":
    main()