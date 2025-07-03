#!/Users/jack/CHONKER_SNYFTER/venv/bin/python3
"""
CHONKER Web Server - Environmental Lab Data Visualization
Flask-based web interface for displaying extracted environmental data
"""

import json
import os
import sys
from pathlib import Path
from flask import Flask, render_template, request, jsonify, send_from_directory
from werkzeug.utils import secure_filename
import subprocess
import tempfile
from datetime import datetime

# Add python directory to path for imports
sys.path.append(str(Path(__file__).parent / "python"))

app = Flask(__name__, 
           template_folder='web/templates',
           static_folder='web/static')

app.config['UPLOAD_FOLDER'] = 'uploads'
app.config['MAX_CONTENT_LENGTH'] = 50 * 1024 * 1024  # 50MB max file size
app.config['SECRET_KEY'] = 'chonker-env-lab-key'

# Ensure upload directory exists
os.makedirs(app.config['UPLOAD_FOLDER'], exist_ok=True)

ALLOWED_EXTENSIONS = {'pdf'}

def allowed_file(filename):
    return '.' in filename and filename.rsplit('.', 1)[1].lower() in ALLOWED_EXTENSIONS

@app.route('/')
def index():
    """Main dashboard page"""
    return render_template('dashboard.html')

@app.route('/upload', methods=['POST'])
def upload_file():
    """Handle PDF upload and processing"""
    if 'file' not in request.files:
        return jsonify({'error': 'No file selected'}), 400
    
    file = request.files['file']
    if file.filename == '':
        return jsonify({'error': 'No file selected'}), 400
    
    if file and allowed_file(file.filename):
        filename = secure_filename(file.filename)
        timestamp = datetime.now().strftime('%Y%m%d_%H%M%S')
        filename = f"{timestamp}_{filename}"
        filepath = os.path.join(app.config['UPLOAD_FOLDER'], filename)
        file.save(filepath)
        
        # Process the file using extraction_bridge.py
        try:
            result = process_pdf(filepath)
            return jsonify({
                'success': True,
                'filename': filename,
                'data': result
            })
        except Exception as e:
            return jsonify({'error': f'Processing failed: {str(e)}'}), 500
    
    return jsonify({'error': 'Invalid file type'}), 400

def process_pdf(filepath):
    """Process PDF using the extraction bridge"""
    venv_python = str(Path(__file__).parent / "venv" / "bin" / "python")
    extraction_script = str(Path(__file__).parent / "python" / "extraction_bridge.py")
    
    # Run the extraction
    cmd = [venv_python, extraction_script, filepath, "--tool", "docling_enhanced"]
    result = subprocess.run(cmd, capture_output=True, text=True, cwd=str(Path(__file__).parent))
    
    if result.returncode != 0:
        raise Exception(f"Extraction failed: {result.stderr}")
    
    # Parse the JSON output
    try:
        return json.loads(result.stdout)
    except json.JSONDecodeError as e:
        raise Exception(f"Failed to parse extraction output: {e}")

@app.route('/api/tables/<filename>')
def get_tables(filename):
    """Get structured table data for a processed file"""
    # This would retrieve cached results or reprocess if needed
    return jsonify({'tables': []})  # Placeholder

@app.route('/api/analysis/<filename>')
def get_analysis(filename):
    """Get analysis results for environmental data"""
    # This would perform analysis on the extracted data
    return jsonify({'analysis': {}})  # Placeholder

@app.route('/view/<filename>')
def view_results(filename):
    """View detailed results for a processed file"""
    return render_template('results.html', filename=filename)

if __name__ == '__main__':
    print("🌐 Starting CHONKER Environmental Lab Data Server...")
    print("📊 Dashboard will be available at: http://localhost:5000")
    app.run(debug=True, host='0.0.0.0', port=5000)
