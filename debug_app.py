#!/usr/bin/env python3
"""
Debug script for CHONKER & SNYFTER application
Tests all major functionality and reports issues
"""

import sys
import os
import time
from pathlib import Path
from PyQt6.QtWidgets import QApplication
from PyQt6.QtCore import QTimer, Qt
from rich.console import Console
from rich.table import Table
from rich.panel import Panel
from rich import box

console = Console()

class DebugTester:
    """Automated tester for the application"""
    
    def __init__(self):
        self.app = None
        self.window = None
        self.test_results = {}
        self.test_pdf = "test_document.pdf"
        
    def run_tests(self):
        """Run all tests"""
        console.print(Panel.fit(
            "[bold cyan]🔍 CHONKER & SNYFTER Debug Suite[/bold cyan]\n"
            "Running comprehensive tests...",
            title="Debug Mode",
            box=box.ROUNDED
        ))
        
        # Test 1: Check dependencies
        self.test_dependencies()
        
        # Test 2: Check file system
        self.test_filesystem()
        
        # Test 3: Test database
        self.test_database()
        
        # Test 4: Test UI components
        self.test_ui_components()
        
        # Test 5: Test PDF processing
        self.test_pdf_processing()
        
        # Display results
        self.display_results()
    
    def test_dependencies(self):
        """Test all required dependencies"""
        console.print("\n[bold]📦 Testing Dependencies...[/bold]")
        
        deps = {
            "PyQt6": "import PyQt6",
            "docling": "import docling",
            "PyMuPDF": "import fitz",
            "instructor": "import instructor",
            "openai": "import openai",
            "rich": "import rich",
            "pydantic": "import pydantic",
            "reportlab": "import reportlab"
        }
        
        for name, import_stmt in deps.items():
            try:
                exec(import_stmt)
                self.test_results[f"dep_{name}"] = ("✅", f"{name} installed")
                console.print(f"  ✅ {name}")
            except ImportError as e:
                self.test_results[f"dep_{name}"] = ("❌", f"{name} missing: {e}")
                console.print(f"  ❌ {name} - {e}")
    
    def test_filesystem(self):
        """Test file system requirements"""
        console.print("\n[bold]📁 Testing File System...[/bold]")
        
        # Check directories
        dirs_to_check = [
            "assets",
            "assets/emojis",
            "android_7_1_emojis",
            "__pycache__"
        ]
        
        for dir_name in dirs_to_check:
            path = Path(dir_name)
            if path.exists():
                self.test_results[f"dir_{dir_name}"] = ("✅", f"{dir_name} exists")
                console.print(f"  ✅ {dir_name}/")
            else:
                self.test_results[f"dir_{dir_name}"] = ("⚠️", f"{dir_name} missing")
                console.print(f"  ⚠️  {dir_name}/ missing")
        
        # Check key files
        files_to_check = [
            ("chonker_snyfter.py", "Original app"),
            ("chonker_snyfter_enhanced.py", "Enhanced app"),
            ("snyfter_archives.db", "Database"),
            ("test_document.pdf", "Test PDF"),
            ("assets/emojis/chonker.png", "Chonker emoji"),
            ("assets/emojis/snyfter.png", "Snyfter emoji")
        ]
        
        for file_name, desc in files_to_check:
            path = Path(file_name)
            if path.exists():
                size = path.stat().st_size
                self.test_results[f"file_{file_name}"] = ("✅", f"{desc} ({size} bytes)")
                console.print(f"  ✅ {file_name} - {desc}")
            else:
                self.test_results[f"file_{file_name}"] = ("❌", f"{desc} missing")
                console.print(f"  ❌ {file_name} - {desc} missing")
    
    def test_database(self):
        """Test database functionality"""
        console.print("\n[bold]🗄️ Testing Database...[/bold]")
        
        try:
            from chonker_snyfter_enhanced import SnyfterDatabase
            
            # Test database initialization
            db = SnyfterDatabase("test_snyfter.db")
            self.test_results["db_init"] = ("✅", "Database initialized")
            console.print("  ✅ Database initialization")
            
            # Test table creation
            import sqlite3
            conn = sqlite3.connect("test_snyfter.db")
            cursor = conn.cursor()
            
            # Check tables
            cursor.execute("SELECT name FROM sqlite_master WHERE type='table'")
            tables = cursor.fetchall()
            table_names = [t[0] for t in tables]
            
            expected_tables = ["documents", "chunks", "chunks_fts"]
            for table in expected_tables:
                if table in table_names:
                    self.test_results[f"db_table_{table}"] = ("✅", f"Table {table} exists")
                    console.print(f"  ✅ Table '{table}' exists")
                else:
                    self.test_results[f"db_table_{table}"] = ("❌", f"Table {table} missing")
                    console.print(f"  ❌ Table '{table}' missing")
            
            conn.close()
            
            # Clean up test database
            os.unlink("test_snyfter.db")
            
        except Exception as e:
            self.test_results["db_test"] = ("❌", f"Database test failed: {e}")
            console.print(f"  ❌ Database test failed: {e}")
    
    def test_ui_components(self):
        """Test UI components can be created"""
        console.print("\n[bold]🎨 Testing UI Components...[/bold]")
        
        try:
            # Create test app
            app = QApplication.instance()
            if app is None:
                app = QApplication(sys.argv)
            
            # Test creating main window
            from chonker_snyfter_enhanced import ChonkerSnyfterEnhancedWindow
            
            # Just test instantiation, don't show
            window = ChonkerSnyfterEnhancedWindow()
            self.test_results["ui_window"] = ("✅", "Main window created")
            console.print("  ✅ Main window creation")
            
            # Test key widgets exist
            widgets_to_check = [
                ("pdf_view", "PDF viewer"),
                ("chunks_table", "Chunks table"),
                ("markdown_view", "Markdown view"),
                ("search_input", "Search input"),
                ("terminal_feedback", "Terminal feedback")
            ]
            
            for attr, desc in widgets_to_check:
                if hasattr(window, attr):
                    self.test_results[f"ui_{attr}"] = ("✅", f"{desc} exists")
                    console.print(f"  ✅ {desc}")
                else:
                    self.test_results[f"ui_{attr}"] = ("❌", f"{desc} missing")
                    console.print(f"  ❌ {desc} missing")
            
            # Clean up
            window.close()
            
        except Exception as e:
            self.test_results["ui_test"] = ("❌", f"UI test failed: {e}")
            console.print(f"  ❌ UI test failed: {e}")
    
    def test_pdf_processing(self):
        """Test PDF processing functionality"""
        console.print("\n[bold]📄 Testing PDF Processing...[/bold]")
        
        try:
            # Check if we have PyMuPDF
            import fitz
            self.test_results["pdf_fitz"] = ("✅", "PyMuPDF available")
            console.print("  ✅ PyMuPDF available")
            
            # Test opening our test PDF
            if os.path.exists(self.test_pdf):
                doc = fitz.open(self.test_pdf)
                page_count = doc.page_count
                self.test_results["pdf_open"] = ("✅", f"Test PDF opened ({page_count} pages)")
                console.print(f"  ✅ Test PDF opened - {page_count} pages")
                
                # Test text extraction
                first_page = doc[0]
                text = first_page.get_text()
                if text:
                    self.test_results["pdf_text"] = ("✅", f"Text extracted ({len(text)} chars)")
                    console.print(f"  ✅ Text extraction works")
                else:
                    self.test_results["pdf_text"] = ("⚠️", "No text extracted")
                    console.print(f"  ⚠️  No text extracted")
                
                doc.close()
            else:
                self.test_results["pdf_test"] = ("❌", "Test PDF not found")
                console.print("  ❌ Test PDF not found")
            
            # Test Docling if available
            try:
                from docling.document_converter import DocumentConverter
                self.test_results["pdf_docling"] = ("✅", "Docling available")
                console.print("  ✅ Docling available")
            except ImportError:
                self.test_results["pdf_docling"] = ("⚠️", "Docling not available")
                console.print("  ⚠️  Docling not available")
                
        except Exception as e:
            self.test_results["pdf_processing"] = ("❌", f"PDF test failed: {e}")
            console.print(f"  ❌ PDF processing test failed: {e}")
    
    def display_results(self):
        """Display test results summary"""
        console.print("\n[bold]📊 Test Results Summary[/bold]\n")
        
        # Create results table
        table = Table(show_header=True, header_style="bold magenta")
        table.add_column("Category", style="cyan", width=20)
        table.add_column("Test", style="white", width=30)
        table.add_column("Status", justify="center", width=10)
        table.add_column("Details", style="dim", width=40)
        
        # Group results by category
        categories = {
            "Dependencies": "dep_",
            "File System": ["dir_", "file_"],
            "Database": "db_",
            "UI Components": "ui_",
            "PDF Processing": "pdf_"
        }
        
        for category, prefix in categories.items():
            if isinstance(prefix, list):
                tests = [(k, v) for k, v in self.test_results.items() 
                        if any(k.startswith(p) for p in prefix)]
            else:
                tests = [(k, v) for k, v in self.test_results.items() 
                        if k.startswith(prefix)]
            
            for test_name, (status, details) in tests:
                clean_name = test_name.split('_', 1)[1] if '_' in test_name else test_name
                table.add_row(category, clean_name, status, details)
        
        console.print(table)
        
        # Summary statistics
        total_tests = len(self.test_results)
        passed = sum(1 for _, (status, _) in self.test_results.items() if status == "✅")
        warnings = sum(1 for _, (status, _) in self.test_results.items() if status == "⚠️")
        failed = sum(1 for _, (status, _) in self.test_results.items() if status == "❌")
        
        console.print(f"\n[bold]Total Tests:[/bold] {total_tests}")
        console.print(f"[green]Passed:[/green] {passed}")
        console.print(f"[yellow]Warnings:[/yellow] {warnings}")
        console.print(f"[red]Failed:[/red] {failed}")
        
        # Recommendations
        if failed > 0 or warnings > 0:
            console.print("\n[bold yellow]💡 Recommendations:[/bold yellow]")
            
            if "dep_docling" in self.test_results and self.test_results["dep_docling"][0] == "❌":
                console.print("  • Install docling: pip install docling")
            
            if "file_assets/emojis/chonker.png" in self.test_results and \
               self.test_results["file_assets/emojis/chonker.png"][0] == "❌":
                console.print("  • Run the app once to create fallback emojis")
            
            if any(k.startswith("db_table_") and v[0] == "❌" 
                   for k, v in self.test_results.items()):
                console.print("  • Delete snyfter_archives.db to recreate database")

def main():
    """Run debug tests"""
    tester = DebugTester()
    tester.run_tests()
    
    # Ask if user wants to launch the app
    console.print("\n[bold cyan]Would you like to launch the application now? (y/n)[/bold cyan]")
    response = input("> ")
    
    if response.lower() == 'y':
        console.print("\n[bold green]Launching CHONKER & SNYFTER...[/bold green]")
        os.system("python chonker_snyfter_enhanced.py")

if __name__ == "__main__":
    main()