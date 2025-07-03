use serde::{Deserialize, Serialize};

/// Unique identifier for document elements
pub type ElementId = String;

/// Core document structure containing all elements with full fidelity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub elements: Vec<DocumentElement>,
    pub metadata: DocumentMetadata,
    pub page_count: usize,
}

/// Document metadata preserving all DocTags information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub creation_date: Option<String>,
    pub modification_date: Option<String>,
    pub page_dimensions: Vec<PageDimensions>,
    pub docling_version: Option<String>,
    pub processing_time: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageDimensions {
    pub page_number: usize,
    pub width: f32,
    pub height: f32,
}

/// Rich document elements with full semantic information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocumentElement {
    Table {
        id: ElementId,
        data: TableData,
        bounds: BoundingBox,
        page_number: usize,
        caption: Option<String>,
        table_type: TableType,
    },
    Paragraph {
        id: ElementId,
        text: String,
        style: TextStyle,
        bounds: BoundingBox,
        page_number: usize,
    },
    Heading {
        id: ElementId,
        text: String,
        level: u8, // 1-6 for H1-H6
        style: TextStyle,
        bounds: BoundingBox,
        page_number: usize,
    },
    List {
        id: ElementId,
        items: Vec<ListItem>,
        list_type: ListType,
        bounds: BoundingBox,
        page_number: usize,
    },
    Image {
        id: ElementId,
        data: ImageRef,
        caption: Option<String>,
        bounds: BoundingBox,
        page_number: usize,
    },
    Formula {
        id: ElementId,
        latex: String,
        rendered: Option<ImageRef>,
        bounds: BoundingBox,
        page_number: usize,
    },
    Section {
        id: ElementId,
        title: String,
        elements: Vec<DocumentElement>,
        bounds: BoundingBox,
        page_number: usize,
    },
}

/// Rich table data with full structure preservation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableData {
    pub cells: Vec<Vec<Cell>>,
    pub headers: Vec<TableHeader>,
    pub col_widths: Vec<f32>,
    pub row_heights: Vec<f32>,
    pub total_rows: usize,
    pub total_cols: usize,
    pub merged_regions: Vec<MergedRegion>,
}

/// Individual table cell with rich content and styling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cell {
    pub content: CellContent,
    pub span: CellSpan,
    pub style: CellStyle,
    pub is_header: bool,
    pub is_empty: bool,
    pub confidence: Option<f32>,
}

/// Cell content types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CellContent {
    Text(String),
    Number(f64),
    Formula(String),
    Empty,
    Mixed(Vec<ContentFragment>),
}

/// Content fragments for mixed cell content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentFragment {
    Text(String),
    Number(f64),
    Superscript(String),
    Subscript(String),
    Bold(String),
    Italic(String),
}

/// Cell spanning information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellSpan {
    pub row_span: usize,
    pub col_span: usize,
}

/// Cell styling information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellStyle {
    pub background_color: Option<Color>,
    pub text_color: Option<Color>,
    pub font_weight: FontWeight,
    pub font_size: Option<f32>,
    pub alignment: TextAlignment,
    pub border: BorderStyle,
}

/// Table header information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableHeader {
    pub text: String,
    pub column_index: usize,
    pub span: usize,
    pub is_multi_level: bool,
    pub parent_header: Option<String>,
}

/// Merged cell regions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergedRegion {
    pub top_row: usize,
    pub left_col: usize,
    pub bottom_row: usize,
    pub right_col: usize,
}

/// Table classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TableType {
    Financial,
    Environmental,
    Scientific,
    DataMatrix,
    Summary,
    Complex,
    General,
}

/// Text styling information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextStyle {
    pub font_family: Option<String>,
    pub font_size: Option<f32>,
    pub font_weight: FontWeight,
    pub color: Option<Color>,
    pub alignment: TextAlignment,
    pub line_height: Option<f32>,
}

/// List types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ListType {
    Ordered,
    Unordered,
    Nested,
}

/// List item structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListItem {
    pub text: String,
    pub level: usize,
    pub sub_items: Vec<ListItem>,
}

/// Image reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageRef {
    pub path: Option<String>,
    pub data: Option<Vec<u8>>,
    pub format: ImageFormat,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

/// Image formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImageFormat {
    PNG,
    JPEG,
    SVG,
    PDF,
    Unknown,
}

/// Bounding box for spatial information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Color representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

/// Font weight
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FontWeight {
    Normal,
    Bold,
    Light,
    Medium,
    SemiBold,
    ExtraBold,
}

/// Text alignment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextAlignment {
    Left,
    Center,
    Right,
    Justify,
}

/// Border styling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BorderStyle {
    pub width: f32,
    pub color: Option<Color>,
    pub style: BorderType,
}

/// Border types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BorderType {
    None,
    Solid,
    Dashed,
    Dotted,
}

impl Default for CellSpan {
    fn default() -> Self {
        Self { row_span: 1, col_span: 1 }
    }
}

impl Default for CellStyle {
    fn default() -> Self {
        Self {
            background_color: None,
            text_color: None,
            font_weight: FontWeight::Normal,
            font_size: None,
            alignment: TextAlignment::Left,
            border: BorderStyle::default(),
        }
    }
}

impl Default for BorderStyle {
    fn default() -> Self {
        Self {
            width: 1.0,
            color: None,
            style: BorderType::Solid,
        }
    }
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font_family: None,
            font_size: None,
            font_weight: FontWeight::Normal,
            color: None,
            alignment: TextAlignment::Left,
            line_height: None,
        }
    }
}

impl Document {
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
            metadata: DocumentMetadata::default(),
            page_count: 0,
        }
    }

    pub fn add_element(&mut self, element: DocumentElement) {
        self.elements.push(element);
    }

    pub fn get_tables(&self) -> Vec<&DocumentElement> {
        self.elements.iter()
            .filter(|e| matches!(e, DocumentElement::Table { .. }))
            .collect()
    }

    pub fn get_element_by_id(&self, id: &str) -> Option<&DocumentElement> {
        self.elements.iter().find(|e| e.id() == id)
    }

    pub fn get_element_by_id_mut(&mut self, id: &str) -> Option<&mut DocumentElement> {
        self.elements.iter_mut().find(|e| e.id() == id)
    }
}

impl Default for DocumentMetadata {
    fn default() -> Self {
        Self {
            title: None,
            author: None,
            creation_date: None,
            modification_date: None,
            page_dimensions: Vec::new(),
            docling_version: None,
            processing_time: None,
        }
    }
}

impl DocumentElement {
    pub fn id(&self) -> &str {
        match self {
            DocumentElement::Table { id, .. } => id,
            DocumentElement::Paragraph { id, .. } => id,
            DocumentElement::Heading { id, .. } => id,
            DocumentElement::List { id, .. } => id,
            DocumentElement::Image { id, .. } => id,
            DocumentElement::Formula { id, .. } => id,
            DocumentElement::Section { id, .. } => id,
        }
    }

    pub fn bounds(&self) -> &BoundingBox {
        match self {
            DocumentElement::Table { bounds, .. } => bounds,
            DocumentElement::Paragraph { bounds, .. } => bounds,
            DocumentElement::Heading { bounds, .. } => bounds,
            DocumentElement::List { bounds, .. } => bounds,
            DocumentElement::Image { bounds, .. } => bounds,
            DocumentElement::Formula { bounds, .. } => bounds,
            DocumentElement::Section { bounds, .. } => bounds,
        }
    }

    pub fn page_number(&self) -> usize {
        match self {
            DocumentElement::Table { page_number, .. } => *page_number,
            DocumentElement::Paragraph { page_number, .. } => *page_number,
            DocumentElement::Heading { page_number, .. } => *page_number,
            DocumentElement::List { page_number, .. } => *page_number,
            DocumentElement::Image { page_number, .. } => *page_number,
            DocumentElement::Formula { page_number, .. } => *page_number,
            DocumentElement::Section { page_number, .. } => *page_number,
        }
    }
}

impl TableData {
    pub fn new(rows: usize, cols: usize) -> Self {
        let mut cells = Vec::new();
        for _ in 0..rows {
            let mut row = Vec::new();
            for _ in 0..cols {
                row.push(Cell::empty());
            }
            cells.push(row);
        }

        Self {
            cells,
            headers: Vec::new(),
            col_widths: vec![100.0; cols],
            row_heights: vec![30.0; rows],
            total_rows: rows,
            total_cols: cols,
            merged_regions: Vec::new(),
        }
    }

    pub fn get_cell(&self, row: usize, col: usize) -> Option<&Cell> {
        self.cells.get(row)?.get(col)
    }

    pub fn get_cell_mut(&mut self, row: usize, col: usize) -> Option<&mut Cell> {
        self.cells.get_mut(row)?.get_mut(col)
    }

    pub fn set_cell(&mut self, row: usize, col: usize, cell: Cell) {
        if let Some(target_cell) = self.get_cell_mut(row, col) {
            *target_cell = cell;
        }
    }
}

impl Cell {
    pub fn empty() -> Self {
        Self {
            content: CellContent::Empty,
            span: CellSpan::default(),
            style: CellStyle::default(),
            is_header: false,
            is_empty: true,
            confidence: None,
        }
    }

    pub fn text(text: String) -> Self {
        Self {
            content: CellContent::Text(text),
            span: CellSpan::default(),
            style: CellStyle::default(),
            is_header: false,
            is_empty: false,
            confidence: None,
        }
    }

    pub fn number(value: f64) -> Self {
        Self {
            content: CellContent::Number(value),
            span: CellSpan::default(),
            style: CellStyle::default(),
            is_header: false,
            is_empty: false,
            confidence: None,
        }
    }

    pub fn header(text: String) -> Self {
        Self {
            content: CellContent::Text(text),
            span: CellSpan::default(),
            style: CellStyle::default(),
            is_header: true,
            is_empty: false,
            confidence: None,
        }
    }

    pub fn as_text(&self) -> String {
        match &self.content {
            CellContent::Text(t) => t.clone(),
            CellContent::Number(n) => n.to_string(),
            CellContent::Formula(f) => f.clone(),
            CellContent::Empty => String::new(),
            CellContent::Mixed(fragments) => {
                fragments.iter().map(|f| match f {
                    ContentFragment::Text(t) => t.clone(),
                    ContentFragment::Number(n) => n.to_string(),
                    ContentFragment::Superscript(s) => format!("^{}", s),
                    ContentFragment::Subscript(s) => format!("_{}", s),
                    ContentFragment::Bold(b) => format!("**{}**", b),
                    ContentFragment::Italic(i) => format!("*{}*", i),
                }).collect::<Vec<_>>().join("")
            }
        }
    }
}
