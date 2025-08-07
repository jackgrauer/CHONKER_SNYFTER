# Future Features - Chonker5 Vision-Text Pipeline

This document outlines the planned integration of the vision-text mapping pipeline that is currently in development. The code infrastructure is in place but awaiting full activation.

## Vision-Text Mapping Pipeline

### Overview
The vision-text mapping pipeline combines Ferrules ML-based layout detection with PDFium's precise text extraction to create an intelligent character matrix representation of PDF documents.

### Key Components

#### 1. **AI Sensor Stack** (Partially Implemented)
- **VisionSensor**: Integrates with Ferrules for layout detection
- **ExtractionSensor**: Enhanced PDFium text extraction with spatial awareness
- **FusionSensor**: Combines vision and text data for optimal results

#### 2. **Spatial Indexing** (Implemented)
- R-tree spatial index for O(log n) text object queries
- `SpatialTextObject` wrapper for efficient lookups
- Methods: `build_spatial_index()`, `find_overlapping_text()`

#### 3. **Vision Context Structures** (Ready for Integration)
- `VisionContext`: Complete document analysis metadata
- `DocumentLayout`: Page dimensions and structure information
- `ReadingPath`: Navigation order for natural reading flow
- `SemanticHint`: Content understanding and prioritization

#### 4. **Smart Character Grid** (Infrastructure Ready)
- `SmartCharacterGrid`: Enhanced matrix with semantic understanding
- `SemanticRegion`: Content-aware text regions
- Confidence mapping for quality visualization

### Pending Integration Tasks

#### Phase 1: Ferrules Integration (Simplified)
```rust
// CURRENT STATE: Simplified to detect occupied grid squares
fn detect_occupied_grid_squares() -> Vec<VisionTextRegion>
// Uses flood-fill algorithm to find contiguous text regions
// 8-connected neighbor search for better region detection

// FUTURE: Full ferrules integration
fn run_ferrules_vision_analysis() -> VisionContext
fn parse_ferrules_to_vision_context() -> Result<VisionContext>
```

#### Phase 2: Spatial Fusion
```rust
// Text-to-region mapping using R-tree
fn map_text_objects_to_regions() // Uses spatial index
fn resolve_text_conflict() // Handles overlapping text
```

#### Phase 3: UI Integration
- Reading order navigation controls
- Confidence visualization overlay
- Semantic region highlighting
- Interactive region selection

### Reserved Methods and Fields

#### CharacterMatrixEngine Methods
- `find_overlapping_text()` - Spatial queries for vision regions
- `generate_optimal_character_matrix()` - Advanced matrix sizing
- `character_matrix_to_image_high_quality()` - For Ferrules processing
- `convert_ferrules_to_char_regions()` - JSON parsing implementation
- `merge_adjacent_regions()` - Region optimization
- `flood_fill_region()` - Fallback detection method

#### Data Structure Fields
- `VisionContext.reading_order` - Navigation sequence
- `VisionContext.semantic_hints` - Content classification
- `DocumentLayout.page_width/height` - Layout calculations
- `DocumentLayout.column_count` - Multi-column support
- `FontAnalyzer.char_width_cache` - Variable-width fonts
- `SpatialMatcher.text_similarity_threshold` - Fuzzy matching
- `BoundingBox.color` - Visual indicators

### Feature Flags (Proposed)

```toml
[features]
vision-pipeline = ["ferrules", "spatial-index", "semantic-analysis"]
reading-navigation = ["vision-pipeline"]
confidence-visualization = ["vision-pipeline"]
```

### Activation Timeline

1. **DONE**: Simplified ferrules to detect occupied grid squares (flood-fill based)
2. **Q1 2024**: Complete Ferrules binary integration with character matrix images
3. **Q2 2024**: Enable spatial fusion and conflict resolution
4. **Q3 2024**: Add reading order navigation UI
5. **Q4 2024**: Full semantic analysis features

### Testing Strategy

- Unit tests for each component in isolation
- Integration tests with sample PDFs
- Performance benchmarks for spatial queries
- Visual regression tests for UI features

### Dependencies

- `ferrules` - Vision model (binary integration)
- `rstar` - R-tree spatial indexing (already added)
- `rayon` - Parallel processing (already added)

## Contributing

When working on vision-text pipeline features:
1. Remove relevant `#[allow(dead_code)]` attributes
2. Update this document with implementation progress
3. Add comprehensive tests for new functionality
4. Update UI to expose new capabilities

## References

- [Ferrules Documentation](https://github.com/ferrules/ferrules)
- [PDFium API Reference](https://pdfium.googlesource.com/pdfium/)
- [R-tree Spatial Indexing](https://docs.rs/rstar/)