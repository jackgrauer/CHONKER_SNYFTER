# Critical Render Logic for CHONKER5

## The Core Problem
Ferrules assigns blocks from page 2 to page 1, causing text overlap. Our custom widget must handle this correctly.

## Key Data Structure
```rust
// Ferrules outputs this JSON structure:
{
  "pages": [
    {"id": 0, "width": 612.0, "height": 792.0},
    {"id": 1, "width": 612.0, "height": 792.0}
  ],
  "blocks": [
    {
      "id": 0,
      "pages_id": [0],  // <- THIS IS THE KEY: which page the block belongs to
      "bbox": {"x0": 72.0, "y0": 100.0, "x1": 540.0, "y1": 120.0},
      "kind": {
        "block_type": "Header",  // <- We parse this
        "text": "Chapter 1"       // <- And this
      }
    }
  ]
}
```

## The Custom Widget Solution

### 1. Parse JSON Correctly
```rust
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
enum FerrulesKind {
    Structured { block_type: String, text: String },  // Main case
    Text { text: String },                            // Fallback
    Other(serde_json::Value),                        // Safety net
}
```

### 2. Render with Page Separation
```rust
// In the draw callback:
let mut current_y = 20.0;
let page_gap = 20.0;

for (page_idx, page) in doc.pages.iter().enumerate() {
    // Draw page background and border
    
    // CRITICAL: Only draw blocks that belong to THIS page
    for (block_idx, block) in doc.blocks.iter().enumerate() {
        if block.pages_id.contains(&page.id) {  // <- This check prevents overlap!
            // Draw the block at correct position
        }
    }
    
    current_y += page_height + page_gap;  // Move to next page position
}
```

### 3. Debug Visibility
We added maximum debug info to see exactly what's happening:
- Coordinate validation (red warning if out of bounds)
- Block labels showing "#5 Header" or "#12 Text"  
- "NO TEXT DATA" for blocks without text
- Text preview (first 100 chars)
- Status indicators

## Testing Steps
1. Run: `./chonker5.rs`
2. Open a multi-page PDF
3. Click "Structured Data"
4. Look for:
   - Real text content (not JSON/HTML)
   - Blocks on correct pages
   - No overlap between pages
   - Debug info showing block assignments

## Success Criteria
- Text renders as readable content (not JSON structure)
- Each page's blocks stay within that page's boundaries
- No text from page 2 appearing on page 1
- Can see block types (Header, Text, etc.) in debug labels

## Future Features (After Basic Rendering Works)
1. Click-to-edit functionality
2. Drag-and-drop block repositioning  
3. Export corrected layout
4. Zoom/pan improvements