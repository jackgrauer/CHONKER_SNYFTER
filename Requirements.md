# CHONKER v6.0 & SNYFTER v9.1 Requirements
# Python 3.8+ required
# Install with: pip install -r requirements.txt

# Core UI and System Dependencies
rich>=13.0.0,<14.0.0
psutil>=5.9.0,<6.0.0

# Data Processing (SNYFTER)
pandas>=2.0.0,<3.0.0
requests>=2.31.0,<3.0.0

# Document Processing (CHONKER)
docling>=1.0.0,<2.0.0
PyPDF2>=3.0.0,<4.0.0

# Database Support (Optional)
duckdb>=0.9.0,<1.0.0

# Excel Export Support (Optional)
openpyxl>=3.1.0,<4.0.0
xlsxwriter>=3.1.0,<4.0.0

# Development/Testing (Optional)
# pytest>=7.0.0
# black>=23.0.0
# flake8>=6.0.0

# Alternative minimal install (core functionality only):
# pip install rich pandas requests psutil
