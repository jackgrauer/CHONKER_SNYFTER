# CHONKER Quality Report
Generated: Fri Jun 27 20:58:30 EDT 2025

## Database Statistics
```
📊 CHONKER Database Status
========================
Documents: 25
Total chunks: 616
Database size: 10.20 MB
Last updated: 2025-06-28T00:58:24.205736+00:00

Recent Documents:
-----------------
• /Users/jack/Documents/1.pdf (2025-06-28T00:58:24.205736+00:00)
• /Users/jack/Documents/1.pdf (2025-06-28T00:56:15.057751+00:00)
• 1.pdf (2025-06-27T19:46:48.277933+00:00)
• 1.pdf (2025-06-27T19:38:30.838607+00:00)
• 1.pdf (2025-06-27T19:32:47.196964+00:00)
```

## Data Export
- Format: csv
- File: full_dataset.csv
- Location: ./reports

## Quick Analysis
To analyze the data:

### Python
```python
import pandas as pd
df = pd.read_csv('./reports/full_dataset.csv')
print("Document count:", df['document_id'].nunique())
print("Average chunk length:", df['char_count'].mean())
print("Tools used:", df['filename'].value_counts())
```

### CLI Analysis
```bash
# Count documents
cut -d, -f1 ./reports/full_dataset.csv | sort | uniq -c

# Average chunk size
awk -F, 'NR>1 {sum+=$7; count++} END {print "Average chars:", sum/count}' ./reports/full_dataset.csv

# Tool distribution
cut -d, -f2 ./reports/full_dataset.csv | sort | uniq -c
```
