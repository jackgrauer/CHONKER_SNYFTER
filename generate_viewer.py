#!/usr/bin/env python3
"""
CHONKER Document Viewer Generator
Creates a professional HTML viewer for any document processed by Docling.
"""

import json
import os
import sys
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Optional, Any


def markdown_table_to_html(table_lines: List[str]) -> str:
    """
    Convert markdown table lines to proper HTML table.
    """
    if not table_lines:
        return ""
    
    # Remove empty lines
    table_lines = [line for line in table_lines if line.strip()]
    
    if len(table_lines) < 2:
        return ""
    
    html = '<table class="faithful-table">'
    
    # Process header row
    header_row = table_lines[0].strip()
    if header_row.startswith('|') and header_row.endswith('|'):
        header_cells = [cell.strip() for cell in header_row[1:-1].split('|')]
        html += '<thead><tr>'
        for cell in header_cells:
            html += f'<th>{cell}</th>'
        html += '</tr></thead>'
    
    # Skip separator row (usually second row with dashes)
    data_rows = table_lines[2:] if len(table_lines) > 2 else []
    
    if data_rows:
        html += '<tbody>'
        for row in data_rows:
            row = row.strip()
            if row.startswith('|') and row.endswith('|'):
                cells = [cell.strip() for cell in row[1:-1].split('|')]
                html += '<tr>'
                for cell in cells:
                    html += f'<td>{cell}</td>'
                html += '</tr>'
        html += '</tbody>'
    
    html += '</table>'
    return html


def generate_faithful_content_from_html(html_content: str) -> str:
    """
    Generate faithful reproduction from native HTML with editing capabilities.
    """
    import re
    from html import escape
    
    # Parse HTML into editable sections
    faithful_content = ""
    section_id = 0
    
    # Split HTML into meaningful chunks (headings, paragraphs, tables)
    # This is a simplified parser - could be enhanced with proper HTML parsing
    patterns = [
        (r'<h([1-6])[^>]*>(.*?)</h[1-6]>', 'heading'),
        (r'<p[^>]*>(.*?)</p>', 'paragraph'),
        (r'<table[^>]*>.*?</table>', 'table'),
    ]
    
    for pattern, content_type in patterns:
        matches = re.findall(pattern, html_content, re.DOTALL | re.IGNORECASE)
        for match in matches:
            if content_type == 'heading':
                level, text = match
                faithful_content += f'''
                <div class="editable-section" data-section="text-{section_id}">
                    <div class="edit-overlay" onclick="editSection('text-{section_id}')">Edit</div>
                    <div class="section-content" id="text-{section_id}"><h{level}>{text}</h{level}></div>
                </div>
                '''
            elif content_type == 'paragraph':
                faithful_content += f'''
                <div class="editable-section" data-section="text-{section_id}">
                    <div class="edit-overlay" onclick="editSection('text-{section_id}')">Edit</div>
                    <div class="section-content" id="text-{section_id}"><p>{match}</p></div>
                </div>
                '''
            elif content_type == 'table':
                faithful_content += f'''
                <div class="editable-section" data-section="table-{section_id}">
                    <div class="edit-overlay" onclick="editSection('table-{section_id}')">Edit</div>
                    <div class="section-content" id="table-{section_id}">
                        <div class="table-container">
                            {match}
                        </div>
                    </div>
                </div>
                '''
            section_id += 1
    
    return faithful_content


def generate_faithful_content(document_text: str, tables: List[Dict]) -> str:
    """
    Generate faithful reproduction of document content with editing capabilities.
    """
    lines = document_text.split('\n')
    faithful_content = ""
    
    # Group lines into table sections
    current_table_lines = []
    section_id = 0
    
    for line in lines:
        if line.strip().startswith('|'):
            current_table_lines.append(line)
        else:
            # If we have accumulated table lines, create a table section
            if current_table_lines:
                table_html = markdown_table_to_html(current_table_lines)
                faithful_content += f'''
                <div class="editable-section" data-section="table-{section_id}">
                    <div class="edit-overlay" onclick="editSection('table-{section_id}')">Edit</div>
                    <div class="section-content" id="table-{section_id}">
                        <div class="table-container">
                            {table_html}
                        </div>
                    </div>
                </div>
                '''
                current_table_lines = []
                section_id += 1
            
            # Add non-table content if it's not empty
            if line.strip():
                faithful_content += f'''
                <div class="editable-section" data-section="text-{section_id}">
                    <div class="edit-overlay" onclick="editSection('text-{section_id}')">Edit</div>
                    <div class="section-content" id="text-{section_id}">{line}</div>
                </div>
                '''
                section_id += 1
    
    # Handle remaining table lines
    if current_table_lines:
        table_html = markdown_table_to_html(current_table_lines)
        faithful_content += f'''
        <div class="editable-section" data-section="table-{section_id}">
            <div class="edit-overlay" onclick="editSection('table-{section_id}')">Edit</div>
            <div class="section-content" id="table-{section_id}">
                <div class="table-container">
                    {table_html}
                </div>
            </div>
        </div>
        '''
    
    return faithful_content


def generate_document_viewer(
    document_text: str,
    tables: List[Dict],
    metadata: Dict,
    output_path: str,
    document_name: str = "Processed Document",
    native_html: str = None
) -> str:
    """
    Generate a professional HTML viewer for any document type.
    
    Args:
        document_text: Markdown-formatted extracted text
        tables: List of table data structures
        metadata: Document metadata (title, author, etc.)
        output_path: Where to save the HTML file
        document_name: Display name for the document
    
    Returns:
        Path to the generated HTML file
    """
    
    # Extract document info from metadata
    doc_title = metadata.get('title', document_name)
    doc_author = metadata.get('author', 'Unknown')
    page_count = metadata.get('page_count', 'Unknown')
    extracted_at = metadata.get('extracted_at', datetime.now().isoformat())
    
    # Parse extracted timestamp
    try:
        if isinstance(extracted_at, str):
            extracted_dt = datetime.fromisoformat(extracted_at.replace('Z', '+00:00'))
        else:
            extracted_dt = extracted_at
        extracted_display = extracted_dt.strftime('%Y-%m-%d %H:%M:%S')
    except:
        extracted_display = str(extracted_at)
    
    # Generate HTML content with faithful reproduction and editing capabilities
    html_content = f"""<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{doc_title} - CHONKER Document Viewer</title>
    <style>
        body {{
            font-family: 'Times New Roman', serif;
            margin: 0;
            padding: 20px;
            background-color: #f8f8f8;
            line-height: 1.2;
            font-size: 12px;
        }}
        .container {{
            max-width: 1400px;
            margin: 0 auto;
            background: white;
            padding: 30px;
            border: 1px solid #ccc;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }}
        .header {{
            text-align: center;
            margin-bottom: 30px;
            padding-bottom: 20px;
            border-bottom: 2px solid #000;
        }}
        .document-title {{
            font-size: 18px;
            font-weight: bold;
            color: #000;
            margin-bottom: 10px;
        }}
        .document-meta {{
            font-size: 11px;
            color: #666;
            margin-bottom: 15px;
        }}
        .editing-controls {{
            position: sticky;
            top: 0;
            background: #f0f0f0;
            padding: 10px;
            border-bottom: 1px solid #ccc;
            margin-bottom: 20px;
            z-index: 100;
        }}
        .edit-btn {{
            background: #4CAF50;
            color: white;
            border: none;
            padding: 8px 16px;
            border-radius: 4px;
            cursor: pointer;
            font-size: 12px;
            margin-right: 10px;
        }}
        .edit-btn:hover {{
            background: #45a049;
        }}
        .edit-btn.active {{
            background: #ff9800;
        }}
        .save-btn {{
            background: #2196F3;
            color: white;
            border: none;
            padding: 8px 16px;
            border-radius: 4px;
            cursor: pointer;
            font-size: 12px;
        }}
        .save-btn:hover {{
            background: #1976D2;
        }}
        .faithful-document {{
            background: white;
            border: 1px solid #000;
            min-height: 800px;
            padding: 40px;
            font-family: 'Times New Roman', serif;
            font-size: 11px;
            line-height: 1.1;
        }}
        .document-content {{
            white-space: pre-wrap;
            font-family: 'Times New Roman', serif;
            font-size: 11px;
            line-height: 1.1;
        }}
        .editable-section {{
            position: relative;
            margin: 5px 0;
            padding: 2px;
            border: 1px solid transparent;
            min-height: 20px;
        }}
        .editable-section:hover {{
            border: 1px dashed #3498db;
            background-color: #f8f9fa;
        }}
        .editable-section.editing {{
            border: 2px solid #3498db;
            background-color: #fff;
        }}
        .edit-overlay {{
            display: none; /* Hide edit overlays since tables are always editable */
        }}
        .edit-textarea {{
            width: 100%;
            min-height: 100px;
            font-family: 'Times New Roman', serif;
            font-size: 11px;
            line-height: 1.1;
            border: 1px solid #ccc;
            padding: 5px;
            resize: vertical;
        }}
        .faithful-table {{
            border-collapse: collapse;
            width: 100%;
            margin: 10px 0;
            font-size: 10px;
            border: 1px solid #000;
        }}
        .faithful-table th, .faithful-table td {{
            border: 1px solid #000;
            padding: 4px;
            text-align: left;
            vertical-align: top;
            white-space: nowrap;
            overflow: hidden;
            text-overflow: ellipsis;
            position: relative;
        }}
        .faithful-table th {{
            background-color: #f0f0f0;
            font-weight: bold;
        }}
        .faithful-table td.editable-cell {{
            cursor: pointer;
        }}
        .faithful-table td.editable-cell:hover {{
            background-color: #e3f2fd;
            outline: 2px solid #2196F3;
        }}
        .faithful-table td.editing {{
            background-color: #fff;
            outline: 2px solid #4CAF50;
        }}
        .cell-input {{
            width: 100%;
            height: 100%;
            border: none;
            background: transparent;
            font-family: inherit;
            font-size: inherit;
            padding: 0;
            margin: 0;
            outline: none;
        }}
        .editable-table {{
            position: relative;
        }}
        .editable-table:hover {{
            background-color: #f8f9fa;
        }}
        .table-edit-btn {{
            position: absolute;
            top: 5px;
            right: 5px;
            background: #ff9800;
            color: white;
            border: none;
            padding: 3px 8px;
            font-size: 10px;
            border-radius: 3px;
            cursor: pointer;
            opacity: 0;
            transition: opacity 0.2s;
        }}
        .editable-table:hover .table-edit-btn {{
            opacity: 1;
        }}
        .changes-summary {{
            position: fixed;
            top: 20px;
            right: 20px;
            background: #fff;
            border: 1px solid #ccc;
            padding: 15px;
            border-radius: 5px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
            max-width: 300px;
            z-index: 1000;
            display: none;
        }}
        .changes-summary.visible {{
            display: block;
        }}
        .change-item {{
            margin: 5px 0;
            padding: 5px;
            background: #f8f9fa;
            border-radius: 3px;
            font-size: 11px;
        }}
        .toggle-section {{
            margin: 20px 0;
            padding: 10px;
            background: #f0f0f0;
            border-radius: 5px;
        }}
        .section-toggle {{
            background: #666;
            color: white;
            border: none;
            padding: 8px 16px;
            border-radius: 4px;
            cursor: pointer;
            font-size: 12px;
            margin-right: 10px;
        }}
        .section-toggle:hover {{
            background: #555;
        }}
        .section-toggle.active {{
            background: #2196F3;
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <div class="document-title">{doc_title}</div>
            <div class="document-meta">
                <div class="meta-item">
                    <span class="meta-label">Author:</span> {doc_author}
                </div>
                <div class="meta-item">
                    <span class="meta-label">Pages:</span> {page_count}
                </div>
                <div class="meta-item">
                    <span class="meta-label">Processed:</span> {extracted_display}
                </div>
                <div class="meta-item">
                    <span class="meta-label">Processed by:</span> CHONKER Document Processor
                </div>
            </div>
        </div>

        <div class="editing-controls">
            <button class="save-btn" onclick="saveChanges()" id="save-btn">💾 Save Changes</button>
            <button class="section-toggle" onclick="toggleSection('faithful-view')" id="faithful-toggle">📄 Faithful View</button>
            <button class="section-toggle" onclick="toggleSection('raw-view')" id="raw-toggle">📝 Raw Text</button>
            <button class="section-toggle" onclick="toggleSection('tables-view')" id="tables-toggle">📊 Tables ({len(tables)})</button>
        </div>

        <div class="changes-summary" id="changes-summary">
            <h4>📝 Changes Made</h4>
            <div id="changes-list"></div>
        </div>

        <div class="content-section" id="faithful-view">
            <div class="faithful-document">
                <div class="document-content" id="faithful-content">
                    {generate_faithful_content_from_html(native_html) if native_html else generate_faithful_content(document_text, tables)}
                </div>
            </div>
        </div>

        <div class="content-section" id="raw-view" style="display: none;">
            <div class="section-title">Raw Extracted Content</div>
            <div class="editable-section" data-section="raw-text">
                <div class="edit-overlay" onclick="editSection('raw-text')">Edit</div>
                <div class="document-content" id="raw-content">{document_text}</div>
            </div>
        </div>

        <div class="content-section" id="tables-view" style="display: none;">
"""
    
    # Add tables section
    if tables:
        for i, table in enumerate(tables):
            html_content += f"""
                <div class="table-title">Table {i+1}</div>
                <div class="table-container">
                    <table>
"""
            
            # Add table headers
            if table.get('headers'):
                html_content += "<thead><tr>"
                for header in table['headers']:
                    html_content += f"<th>{header}</th>"
                html_content += "</tr></thead>"
            
            # Add table rows
            if table.get('rows'):
                html_content += "<tbody>"
                for row in table['rows']:
                    html_content += "<tr>"
                    for cell in row:
                        html_content += f"<td>{cell}</td>"
                    html_content += "</tr>"
                html_content += "</tbody>"
            
            html_content += """
                    </table>
                </div>
"""
    else:
        html_content += """
                <div class="no-tables">
                    No tables found in this document.
                </div>
"""
    
    # Close tables section and add footer
    html_content += f"""
            </div>
        </div>

        <div class="footer">
            Generated by CHONKER Document Processor • {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}
        </div>
    </div>

"""    
    
    # Add JavaScript with proper escaping
    javascript_code = '''
    <script>
        let editMode = true; // Always in edit mode
        let changes = [];
        
        function toggleSection(sectionId) {
            const sections = ['faithful-view', 'raw-view', 'tables-view'];
            const buttons = ['faithful-toggle', 'raw-toggle', 'tables-toggle'];
            
            // Hide all sections
            sections.forEach(id => {
                const section = document.getElementById(id);
                if (section) section.style.display = 'none';
            });
            
            // Remove active class from all buttons
            buttons.forEach(id => {
                const button = document.getElementById(id);
                if (button) button.classList.remove('active');
            });
            
            // Show selected section
            const targetSection = document.getElementById(sectionId);
            if (targetSection) {
                targetSection.style.display = 'block';
                // Find corresponding button
                const buttonId = sectionId.replace('-view', '-toggle');
                const button = document.getElementById(buttonId);
                if (button) button.classList.add('active');
            }
        }
        
        function editSection(sectionId) {
            const section = document.getElementById(sectionId);
            if (!section) return;
            
            // Check if this section contains a table
            const tableElement = section.querySelector('table');
            
            if (tableElement) {
                // For tables, enable cell-level editing
                enableTableCellEditing(tableElement, sectionId);
            } else {
                // For non-table content, use text editing
                editTextContent(section, sectionId);
            }
        }
        
        function enableTableCellEditing(table, sectionId) {
            // Make all table cells editable
            const cells = table.querySelectorAll('td');
            cells.forEach(cell => {
                cell.classList.add('editable-cell');
                cell.onclick = function() {
                    editCell(cell, sectionId);
                };
            });
        }
        
        function disableTableCellEditing(table) {
            const cells = table.querySelectorAll('td');
            cells.forEach(cell => {
                cell.classList.remove('editable-cell');
                cell.onclick = null;
            });
        }
        
        function editCell(cell, sectionId) {
            if (cell.classList.contains('editing')) return;
            
            const originalText = cell.textContent;
            cell.classList.add('editing');
            
            const input = document.createElement('input');
            input.type = 'text';
            input.className = 'cell-input';
            input.value = originalText;
            
            cell.innerHTML = '';
            cell.appendChild(input);
            input.focus();
            input.select();
            
            function saveCell() {
                const newText = input.value;
                cell.textContent = newText;
                cell.classList.remove('editing');
                
                if (newText !== originalText) {
                    changes.push({
                        section: sectionId + '-cell',
                        original: originalText,
                        modified: newText,
                        timestamp: new Date().toLocaleString()
                    });
                    updateChangesDisplay();
                }
            }
            
            function cancelEdit() {
                cell.textContent = originalText;
                cell.classList.remove('editing');
            }
            
            input.onblur = saveCell;
            input.onkeydown = function(e) {
                if (e.key === 'Enter') {
                    saveCell();
                } else if (e.key === 'Escape') {
                    cancelEdit();
                }
            };
        }
        
        function editTextContent(section, sectionId) {
            const originalText = section.innerText;
            const textarea = document.createElement('textarea');
            textarea.className = 'edit-textarea';
            textarea.value = originalText;
            
            const saveBtn = document.createElement('button');
            saveBtn.textContent = 'Save';
            saveBtn.className = 'save-btn';
            saveBtn.style.marginTop = '10px';
            
            const cancelBtn = document.createElement('button');
            cancelBtn.textContent = 'Cancel';
            cancelBtn.className = 'edit-btn';
            cancelBtn.style.marginTop = '10px';
            cancelBtn.style.marginLeft = '10px';
            
            const container = section.parentElement;
            const originalContainerHTML = container.innerHTML;
            container.innerHTML = '';
            container.appendChild(textarea);
            container.appendChild(saveBtn);
            container.appendChild(cancelBtn);
            
            saveBtn.onclick = function() {
                const newText = textarea.value;
                if (newText !== originalText) {
                    changes.push({
                        section: sectionId,
                        original: originalText,
                        modified: newText,
                        timestamp: new Date().toLocaleString()
                    });
                    updateChangesDisplay();
                }
                
                container.innerHTML = `
                    \u003cdiv class=\"edit-overlay\" onclick=\"editSection('` + sectionId + `')\"\u003eEdit\u003c/div\u003e
                    \u003cdiv class=\"section-content\" id=\"` + sectionId + `\"\u003e` + escapeHtml(newText) + `\u003c/div\u003e
                `;
            };
            
            cancelBtn.onclick = function() {
                container.innerHTML = originalContainerHTML;
            };
            
            textarea.focus();
        }
        
        function escapeHtml(text) {
            const div = document.createElement('div');
            div.textContent = text;
            return div.innerHTML;
        }
        
        function updateChangesDisplay() {
            const changesList = document.getElementById('changes-list');
            changesList.innerHTML = '';
            
            changes.forEach((change, index) => {
                const changeItem = document.createElement('div');
                changeItem.className = 'change-item';
                changeItem.innerHTML = `
                    <strong>Section ` + change.section + `:</strong><br>
                    <small>` + change.timestamp + `</small><br>
                    <small>Changed ` + change.original.length + ` → ` + change.modified.length + ` chars</small>
                `;
                changesList.appendChild(changeItem);
            });
        }
        
        function saveChanges() {
            if (changes.length === 0) {
                alert('No changes to save!');
                return;
            }
            
            const data = {
                document_title: document.querySelector('.document-title').textContent,
                changes: changes,
                saved_at: new Date().toISOString()
            };
            
            // Create download link
            const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = document.querySelector('.document-title').textContent + '_changes.json';
            a.click();
            URL.revokeObjectURL(url);
            
            alert('Saved ' + changes.length + ' changes to file!');
        }
        
        // Initialize on page load
        window.onload = function() {
            // Show faithful view by default
            toggleSection('faithful-view');
            // Show changes summary since we're always in edit mode
            const changesSummary = document.getElementById('changes-summary');
            if (changesSummary) changesSummary.classList.add('visible');
            
            // Auto-enable table editing for all tables
            const tables = document.querySelectorAll('.faithful-table');
            tables.forEach((table, index) => {
                enableTableCellEditing(table, 'table-' + index);
            });
        }
    </script>
    '''
    
    html_content += javascript_code + """
</body>
</html>"""
    
    # Write the HTML file
    with open(output_path, 'w', encoding='utf-8') as f:
        f.write(html_content)
    
    return output_path


def main():
    """
    Command-line interface for the document viewer generator.
    Usage: python generate_viewer.py <document_base_name>
    """
    if len(sys.argv) != 2:
        print("Usage: python generate_viewer.py <document_base_name>")
        print("Example: python generate_viewer.py mydocument")
        sys.exit(1)
    
    base_name = sys.argv[1]
    processed_dir = Path("apps/doc-service/processed_documents")
    
    # Load the extracted data
    text_file = processed_dir / f"{base_name}_text.md"
    tables_file = processed_dir / f"{base_name}_tables.json"
    metadata_file = processed_dir / f"{base_name}_metadata.json"
    native_html_file = processed_dir / f"{base_name}_native.html"
    
    # Check if files exist
    if not text_file.exists():
        print(f"Error: {text_file} not found")
        sys.exit(1)
    
    # Load text content
    with open(text_file, 'r', encoding='utf-8') as f:
        document_text = f.read()
    
    # Load tables if available
    tables = []
    if tables_file.exists():
        with open(tables_file, 'r', encoding='utf-8') as f:
            tables = json.load(f)
    
    # Load metadata if available
    metadata = {}
    if metadata_file.exists():
        with open(metadata_file, 'r', encoding='utf-8') as f:
            metadata = json.load(f)
    
    # Check if native HTML is available for better fidelity
    native_html_content = None
    if native_html_file.exists():
        with open(native_html_file, 'r', encoding='utf-8') as f:
            native_html_content = f.read()
        print(f"✨ Using native HTML for better fidelity")
    
    # Generate the viewer
    output_path = f"{base_name}_viewer.html"
    generated_path = generate_document_viewer(
        document_text=document_text,
        tables=tables,
        metadata=metadata,
        output_path=output_path,
        document_name=base_name.replace('_', ' ').title(),
        native_html=native_html_content
    )
    
    print(f"✅ Generated document viewer: {generated_path}")
    print(f"📖 Open in browser: open {generated_path}")


if __name__ == "__main__":
    main()
