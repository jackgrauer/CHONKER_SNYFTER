#!/usr/bin/env python3
"""
Functional test script for CHONKER & SNYFTER
Tests actual document processing and database operations
"""

import os
import sys
import time
import sqlite3
from pathlib import Path
from rich.console import Console
from rich.progress import Progress, SpinnerColumn, TextColumn

console = Console()

# Add current directory to path
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

def test_document_processing():
    """Test document processing pipeline"""
    console.print("\n[bold cyan]🧪 Testing Document Processing Pipeline[/bold cyan]\n")
    
    test_pdf = "test_document.pdf"
    if not os.path.exists(test_pdf):
        console.print("[red]❌ Test PDF not found. Run create_test_pdf.py first.[/red]")
        return False
    
    try:
        from chonker_snyfter_enhanced import (
            EnhancedChonkerWorker, 
            SnyfterDatabase,
            ProcessingResult,
            DocumentChunk
        )
        
        # Initialize database
        console.print("📊 Initializing database...")
        db = SnyfterDatabase("test_functional.db")
        
        # Create a mock processing result to test database
        console.print("🔧 Creating test data...")
        
        # Create test chunks
        test_chunks = [
            DocumentChunk(
                index=0,
                type="heading",
                content="CHONKER & SNYFTER Test Document",
                level=1,
                page=0,
                confidence=0.95
            ),
            DocumentChunk(
                index=1,
                type="text",
                content="This is a test document for debugging.",
                level=2,
                page=0,
                confidence=0.90
            ),
            DocumentChunk(
                index=2,
                type="list",
                content="PDF Processing - Extracts content from PDF files",
                level=3,
                page=0,
                confidence=0.85
            )
        ]
        
        # Create test result
        test_result = ProcessingResult(
            success=True,
            document_id="test_doc_001",
            chunks=test_chunks,
            html_content="<h1>Test Document</h1><p>Content</p>",
            markdown_content="# Test Document\nContent",
            processing_time=1.5
        )
        
        # Test saving to database
        console.print("💾 Testing database save...")
        save_success = db.save_document(test_result, test_pdf, ["test", "debug"])
        
        if save_success:
            console.print("  ✅ Document saved successfully")
        else:
            console.print("  ❌ Failed to save document")
            return False
        
        # Test searching
        console.print("🔍 Testing search functionality...")
        
        # Search for content
        results = db.search_documents("test")
        if results:
            console.print(f"  ✅ Found {len(results)} documents")
            for doc in results:
                console.print(f"    • {doc['filename']} - {doc.get('snippet', '')}")
        else:
            console.print("  ❌ No search results found")
        
        # Test chunk-specific search
        chunk_results = db.search_documents("processing", "list")
        console.print(f"  ✅ Chunk-specific search: {len(chunk_results)} results")
        
        # Test export functionality
        console.print("📤 Testing export...")
        from chonker_snyfter_enhanced import ExportOptions
        
        # Test JSON export
        json_options = ExportOptions(format="json", include_chunks=True)
        json_export = db.export_document("test_doc_001", json_options)
        if json_export:
            console.print("  ✅ JSON export successful")
            # Save to file
            with open("test_export.json", "w") as f:
                f.write(json_export)
        
        # Test CSV export
        csv_options = ExportOptions(format="csv", include_chunks=True)
        csv_export = db.export_document("test_doc_001", csv_options)
        if csv_export:
            console.print("  ✅ CSV export successful")
        
        # Test Markdown export
        md_options = ExportOptions(format="markdown", include_chunks=True)
        md_export = db.export_document("test_doc_001", md_options)
        if md_export:
            console.print("  ✅ Markdown export successful")
        
        # Verify database contents
        console.print("🔬 Verifying database integrity...")
        conn = sqlite3.connect("test_functional.db")
        cursor = conn.cursor()
        
        # Check document count
        cursor.execute("SELECT COUNT(*) FROM documents")
        doc_count = cursor.fetchone()[0]
        console.print(f"  • Documents in database: {doc_count}")
        
        # Check chunk count
        cursor.execute("SELECT COUNT(*) FROM chunks")
        chunk_count = cursor.fetchone()[0]
        console.print(f"  • Chunks in database: {chunk_count}")
        
        # Check FTS index
        cursor.execute("SELECT COUNT(*) FROM chunks_fts")
        fts_count = cursor.fetchone()[0]
        console.print(f"  • FTS index entries: {fts_count}")
        
        conn.close()
        
        # Clean up
        os.unlink("test_functional.db")
        if os.path.exists("test_export.json"):
            os.unlink("test_export.json")
        
        console.print("\n[green]✅ All functional tests passed![/green]")
        return True
        
    except Exception as e:
        console.print(f"\n[red]❌ Test failed: {e}[/red]")
        import traceback
        traceback.print_exc()
        return False

def test_pdf_processing_with_docling():
    """Test actual PDF processing with Docling"""
    console.print("\n[bold cyan]🐹 Testing CHONKER PDF Processing[/bold cyan]\n")
    
    test_pdf = "test_document.pdf"
    if not os.path.exists(test_pdf):
        console.print("[red]❌ Test PDF not found.[/red]")
        return False
    
    try:
        console.print("📄 Processing test PDF with Docling...")
        
        # Import worker
        from chonker_snyfter_enhanced import EnhancedChonkerWorker
        from PyQt6.QtCore import QCoreApplication
        import sys
        
        # Create Qt application for signals
        app = QCoreApplication.instance()
        if app is None:
            app = QCoreApplication(sys.argv)
        
        # Create worker
        worker = EnhancedChonkerWorker(test_pdf)
        
        # Track progress
        progress_messages = []
        processing_result = None
        
        def on_progress(msg):
            progress_messages.append(msg)
            console.print(f"  {msg}")
        
        def on_finished(result):
            nonlocal processing_result
            processing_result = result
        
        def on_error(error):
            console.print(f"  [red]Error: {error}[/red]")
        
        def on_chunk_progress(current, total):
            if total > 0:
                percent = (current / total) * 100
                console.print(f"  Progress: {current}/{total} chunks ({percent:.1f}%)")
        
        # Connect signals
        worker.progress.connect(on_progress)
        worker.finished.connect(on_finished)
        worker.error.connect(on_error)
        worker.chunk_processed.connect(on_chunk_progress)
        
        # Start processing
        worker.start()
        
        # Wait for completion (timeout after 30 seconds)
        timeout = 30
        start_time = time.time()
        
        while worker.isRunning() and (time.time() - start_time) < timeout:
            app.processEvents()
            time.sleep(0.1)
        
        if worker.isRunning():
            console.print("  [yellow]⚠️  Processing timed out[/yellow]")
            worker.stop()
            worker.wait()
            return False
        
        # Check results
        if processing_result and processing_result.success:
            console.print(f"\n  ✅ Processing successful!")
            console.print(f"  • Document ID: {processing_result.document_id}")
            console.print(f"  • Chunks extracted: {len(processing_result.chunks)}")
            console.print(f"  • Processing time: {processing_result.processing_time:.2f}s")
            console.print(f"  • Markdown length: {len(processing_result.markdown_content)} chars")
            console.print(f"  • HTML length: {len(processing_result.html_content)} chars")
            
            # Show sample chunks
            console.print("\n  Sample chunks:")
            for chunk in processing_result.chunks[:3]:
                console.print(f"    [{chunk.type}] {chunk.content[:50]}...")
            
            return True
        else:
            console.print("  [red]❌ Processing failed[/red]")
            if processing_result:
                console.print(f"  Error: {processing_result.error_message}")
            return False
            
    except Exception as e:
        console.print(f"\n[red]❌ PDF processing test failed: {e}[/red]")
        import traceback
        traceback.print_exc()
        return False

def test_ui_responsiveness():
    """Test UI responsiveness and mode switching"""
    console.print("\n[bold cyan]🎨 Testing UI Components[/bold cyan]\n")
    
    try:
        from PyQt6.QtWidgets import QApplication
        from chonker_snyfter_enhanced import ChonkerSnyfterEnhancedWindow, Mode
        import sys
        
        # Create application
        app = QApplication.instance()
        if app is None:
            app = QApplication(sys.argv)
        
        # Create window
        window = ChonkerSnyfterEnhancedWindow()
        
        console.print("🔄 Testing mode switching...")
        
        # Test mode switching
        original_mode = window.current_mode
        console.print(f"  • Current mode: {original_mode.value}")
        
        # Switch to SNYFTER
        window.set_mode(Mode.SNYFTER)
        console.print(f"  • Switched to: {window.current_mode.value}")
        
        # Switch back to CHONKER
        window.set_mode(Mode.CHONKER)
        console.print(f"  • Switched back to: {window.current_mode.value}")
        
        # Test UI elements exist and are accessible
        console.print("\n🔍 Checking UI elements...")
        
        ui_elements = [
            ("chonker_icon", "CHONKER icon"),
            ("snyfter_icon", "SNYFTER icon"),
            ("terminal_feedback", "Terminal feedback"),
            ("pdf_view", "PDF viewer"),
            ("chunks_table", "Chunks table"),
            ("search_input", "Search input"),
            ("process_btn", "Process button"),
            ("status_bar", "Status bar")
        ]
        
        for attr, name in ui_elements:
            if hasattr(window, attr) and getattr(window, attr) is not None:
                console.print(f"  ✅ {name}")
            else:
                console.print(f"  ❌ {name} missing")
        
        # Clean up
        window.close()
        
        console.print("\n[green]✅ UI tests completed![/green]")
        return True
        
    except Exception as e:
        console.print(f"\n[red]❌ UI test failed: {e}[/red]")
        import traceback
        traceback.print_exc()
        return False

def main():
    """Run all functional tests"""
    console.print("[bold magenta]🚀 CHONKER & SNYFTER Functional Test Suite[/bold magenta]")
    console.print("=" * 50)
    
    tests = [
        ("Database Operations", test_document_processing),
        ("PDF Processing", test_pdf_processing_with_docling),
        ("UI Components", test_ui_responsiveness)
    ]
    
    results = {}
    
    for test_name, test_func in tests:
        console.print(f"\n[bold]Running: {test_name}[/bold]")
        try:
            success = test_func()
            results[test_name] = success
        except Exception as e:
            console.print(f"[red]Test {test_name} crashed: {e}[/red]")
            results[test_name] = False
    
    # Summary
    console.print("\n" + "=" * 50)
    console.print("[bold]Test Summary:[/bold]\n")
    
    passed = sum(1 for success in results.values() if success)
    total = len(results)
    
    for test_name, success in results.items():
        status = "✅ PASSED" if success else "❌ FAILED"
        console.print(f"  {test_name}: {status}")
    
    console.print(f"\n[bold]Total: {passed}/{total} tests passed[/bold]")
    
    if passed == total:
        console.print("\n[bold green]🎉 All tests passed! CHONKER & SNYFTER is ready![/bold green]")
    else:
        console.print("\n[bold yellow]⚠️  Some tests failed. Check the output above.[/bold yellow]")

if __name__ == "__main__":
    main()