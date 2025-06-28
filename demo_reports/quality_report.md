# CHONKER Quality Report
Generated: Fri Jun 27 21:06:46 EDT 2025

## Database Statistics
```
📊 CHONKER Database Status
========================
Documents: 28
Total chunks: 616
Database size: 10.21 MB
Last updated: 2025-06-28T01:06:23.388542+00:00

Recent Documents:
-----------------
• /Users/jack/Documents/1.pdf (2025-06-28T01:06:23.388542+00:00)
• /Users/jack/Documents/1.pdf (2025-06-28T01:05:46.426119+00:00)
• /Users/jack/Documents/1.pdf (2025-06-28T01:04:18.760501+00:00)
• /Users/jack/Documents/1.pdf (2025-06-28T00:58:24.205736+00:00)
• /Users/jack/Documents/1.pdf (2025-06-28T00:56:15.057751+00:00)
```

## Data Export
- Format: csv
- File: full_dataset.csv
- Location: demo_reports

## Quick Analysis
To analyze the data:

### Python
```python
import pandas as pd
df = pd.read_csv('demo_reports/full_dataset.csv')
print("Document count:", df['document_id'].nunique())
print("Average chunk length:", df['char_count'].mean())
print("Tools used:", df['filename'].value_counts())
```

### CLI Analysis
```bash
# Count documents
cut -d, -f1 demo_reports/full_dataset.csv | sort | uniq -c

# Average chunk size
awk -F, 'NR>1 {sum+=$7; count++} END {print "Average chars:", sum/count}' demo_reports/full_dataset.csv

# Tool distribution
cut -d, -f2 demo_reports/full_dataset.csv | sort | uniq -c
```
