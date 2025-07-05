use scraper::{Html, Selector, ElementRef};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::error::{ChonkerError, ChonkerResult};

/// Document structure extracted from HTML using CSS selectors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HtmlDocument {
    pub title: Option<String>,
    pub elements: Vec<HtmlElement>,
    pub metadata: DocumentMetadata,
}

/// Document-agnostic HTML element with rich formatting information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HtmlElement {
    pub id: String,
    pub element_type: ElementType,
    pub content: String,
    pub formatting: ElementFormatting,
    pub position: ElementPosition,
    pub attributes: HashMap<String, String>,
    pub children: Vec<HtmlElement>,
}

/// Comprehensive element types that preserve document semantics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ElementType {
    // Text elements
    Heading { level: u8 },
    Paragraph,
    Text,
    
    // Structural elements
    Table { 
        headers: Vec<String>,
        rows: Vec<Vec<TableCell>>,
        caption: Option<String>
    },
    List {
        list_type: ListType,
        items: Vec<String>
    },
    
    // Content elements
    Image {
        src: Option<String>,
        alt: Option<String>,
        caption: Option<String>
    },
    Link {
        href: Option<String>,
        text: String
    },
    
    // Document structure
    Section,
    Article,
    Header,
    Footer,
    
    // Formatting elements
    Strong,
    Emphasis,
    Code,
    Quote,
    
    // Generic container
    Container,
}

/// Rich formatting information extracted from CSS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementFormatting {
    pub font_family: Option<String>,
    pub font_size: Option<String>,
    pub font_weight: Option<String>,
    pub font_style: Option<String>,
    pub color: Option<String>,
    pub background_color: Option<String>,
    pub text_align: Option<String>,
    pub line_height: Option<String>,
    pub margin: Option<String>,
    pub padding: Option<String>,
    pub border: Option<String>,
    pub width: Option<String>,
    pub height: Option<String>,
    pub display: Option<String>,
    pub position: Option<String>,
    pub css_classes: Vec<String>,
}

/// Element positioning information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementPosition {
    pub page_number: Option<u32>,
    pub top: Option<f32>,
    pub left: Option<f32>,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub z_index: Option<i32>,
}

/// Table cell with rich content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableCell {
    pub content: String,
    pub cell_type: CellType,
    pub formatting: ElementFormatting,
    pub span: CellSpan,
    pub attributes: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CellType {
    Header,
    Data,
    Footer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellSpan {
    pub colspan: u32,
    pub rowspan: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ListType {
    Ordered,
    Unordered,
    Definition,
}

/// Document metadata extracted from HTML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub source_file: String,
    pub extraction_tool: String,
    pub extraction_time: String,
    pub page_count: Option<u32>,
    pub language: Option<String>,
    pub document_title: Option<String>,
    pub author: Option<String>,
    pub creation_date: Option<String>,
    pub modification_date: Option<String>,
    pub css_stylesheets: Vec<String>,
    pub scripts: Vec<String>,
}

/// Main HTML document extractor using CSS selectors
pub struct HtmlExtractor {
    /// CSS selectors for different document elements
    selectors: DocumentSelectors,
}

/// Comprehensive CSS selectors for document-agnostic processing
pub struct DocumentSelectors {
    // Text elements
    pub headings: Selector,
    pub paragraphs: Selector,
    pub text_content: Selector,
    
    // Structural elements
    pub tables: Selector,
    pub table_headers: Selector,
    pub table_rows: Selector,
    pub table_cells: Selector,
    pub lists: Selector,
    pub list_items: Selector,
    
    // Content elements
    pub images: Selector,
    pub links: Selector,
    
    // Document structure
    pub sections: Selector,
    pub articles: Selector,
    pub headers: Selector,
    pub footers: Selector,
    
    // Formatting elements
    pub strong: Selector,
    pub emphasis: Selector,
    pub code: Selector,
    pub quotes: Selector,
    
    // Special patterns for document types
    pub financial_tables: Selector,
    pub data_tables: Selector,
    pub form_elements: Selector,
    pub highlighted_content: Selector,
}

impl HtmlExtractor {
    pub fn new() -> ChonkerResult<Self> {
        let selectors = DocumentSelectors::new()?;
        Ok(Self { selectors })
    }
    
    /// Extract structured document from HTML content
    pub fn extract_document(&self, html_content: &str, source_file: &str) -> ChonkerResult<HtmlDocument> {
        let document = Html::parse_document(html_content);
        
        // Extract metadata
        let metadata = self.extract_metadata(&document, source_file)?;
        
        // Extract document title
        let title = self.extract_title(&document);
        
        // Extract all elements
        let elements = self.extract_elements(&document)?;
        
        Ok(HtmlDocument {
            title,
            elements,
            metadata,
        })
    }
    
    /// Extract document metadata from HTML head and document structure
    fn extract_metadata(&self, document: &Html, source_file: &str) -> ChonkerResult<DocumentMetadata> {
        let title_selector = Selector::parse("title").map_err(|e| ChonkerError::PdfProcessing {
            message: format!("CSS selector error: {:?}", e),
            source: Some(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "HTML parsing error"))),
        })?;
        
        let meta_selector = Selector::parse("meta").map_err(|e| ChonkerError::PdfProcessing {
            message: format!("CSS selector error: {:?}", e),
            source: Some(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "HTML parsing error"))),
        })?;
        
        let document_title = document.select(&title_selector)
            .next()
            .map(|element| element.text().collect::<String>());
        
        let mut author = None;
        let mut creation_date = None;
        let mut language = None;
        
        // Extract metadata from meta tags
        for meta in document.select(&meta_selector) {
            if let Some(name) = meta.value().attr("name") {
                if let Some(content) = meta.value().attr("content") {
                    match name.to_lowercase().as_str() {
                        "author" => author = Some(content.to_string()),
                        "date" | "creation-date" => creation_date = Some(content.to_string()),
                        "language" | "lang" => language = Some(content.to_string()),
                        _ => {}
                    }
                }
            }
        }
        
        Ok(DocumentMetadata {
            source_file: source_file.to_string(),
            extraction_tool: "CHONKER HTML Extractor".to_string(),
            extraction_time: chrono::Utc::now().to_rfc3339(),
            page_count: None, // Will be filled by caller if known
            language,
            document_title,
            author,
            creation_date,
            modification_date: None,
            css_stylesheets: Vec::new(),
            scripts: Vec::new(),
        })
    }
    
    /// Extract document title
    fn extract_title(&self, document: &Html) -> Option<String> {
        // Try multiple strategies for finding the document title
        let strategies = vec![
            "h1",
            "title",
            ".document-title",
            ".title",
            "[data-title]",
        ];
        
        for strategy in strategies {
            if let Ok(selector) = Selector::parse(strategy) {
                if let Some(element) = document.select(&selector).next() {
                    let title = element.text().collect::<String>().trim().to_string();
                    if !title.is_empty() {
                        return Some(title);
                    }
                }
            }
        }
        
        None
    }
    
    /// Extract all document elements using CSS selectors
    fn extract_elements(&self, document: &Html) -> ChonkerResult<Vec<HtmlElement>> {
        let mut elements = Vec::new();
        let mut element_id = 0;
        
        // Extract elements in document order
        let body_selector = Selector::parse("body, main, article, .content, .document").unwrap();
        let root = document.select(&body_selector).next()
            .unwrap_or_else(|| document.root_element());
        
        self.extract_children_recursive(root, &mut elements, &mut element_id)?;
        
        Ok(elements)
    }
    
    /// Recursively extract child elements while preserving document structure
    fn extract_children_recursive(
        &self, 
        element: ElementRef, 
        elements: &mut Vec<HtmlElement>, 
        element_id: &mut usize
    ) -> ChonkerResult<()> {
        for child in element.children() {
            if let Some(child_element) = ElementRef::wrap(child) {
                let tag_name = child_element.value().name();
                
                // Skip script and style elements
                if matches!(tag_name, "script" | "style" | "noscript") {
                    continue;
                }
                
                let extracted = self.extract_single_element(child_element, element_id)?;
                if let Some(extracted) = extracted {
                    elements.push(extracted);
                }
            }
        }
        
        Ok(())
    }
    
    /// Extract a single HTML element with all its formatting and content
    fn extract_single_element(&self, element: ElementRef, element_id: &mut usize) -> ChonkerResult<Option<HtmlElement>> {
        let tag_name = element.value().name();
        let text_content = element.text().collect::<String>().trim().to_string();
        
        // Skip empty text nodes
        if text_content.is_empty() && !matches!(tag_name, "img" | "br" | "hr" | "input") {
            return Ok(None);
        }
        
        *element_id += 1;
        
        let element_type = self.determine_element_type(element, tag_name)?;
        let formatting = self.extract_formatting(element);
        let position = self.extract_position(element);
        let attributes = self.extract_attributes(element);
        
        // Extract children for container elements
        let mut children = Vec::new();
        if matches!(tag_name, "div" | "section" | "article" | "main" | "aside" | "nav") {
            self.extract_children_recursive(element, &mut children, element_id)?;
        }
        
        Ok(Some(HtmlElement {
            id: format!("element_{}", element_id),
            element_type,
            content: text_content,
            formatting,
            position,
            attributes,
            children,
        }))
    }
    
    /// Determine the semantic type of an HTML element
    fn determine_element_type(&self, element: ElementRef, tag_name: &str) -> ChonkerResult<ElementType> {
        match tag_name {
            "h1" => Ok(ElementType::Heading { level: 1 }),
            "h2" => Ok(ElementType::Heading { level: 2 }),
            "h3" => Ok(ElementType::Heading { level: 3 }),
            "h4" => Ok(ElementType::Heading { level: 4 }),
            "h5" => Ok(ElementType::Heading { level: 5 }),
            "h6" => Ok(ElementType::Heading { level: 6 }),
            "p" => Ok(ElementType::Paragraph),
            "table" => {
                let (headers, rows) = self.extract_table_data(element)?;
                let caption = self.extract_table_caption(element);
                Ok(ElementType::Table { headers, rows, caption })
            },
            "ul" | "ol" | "dl" => {
                let list_type = match tag_name {
                    "ol" => ListType::Ordered,
                    "ul" => ListType::Unordered,
                    "dl" => ListType::Definition,
                    _ => ListType::Unordered,
                };
                let items = self.extract_list_items(element);
                Ok(ElementType::List { list_type, items })
            },
            "img" => {
                let src = element.value().attr("src").map(|s| s.to_string());
                let alt = element.value().attr("alt").map(|s| s.to_string());
                Ok(ElementType::Image { src, alt, caption: None })
            },
            "a" => {
                let href = element.value().attr("href").map(|s| s.to_string());
                let text = element.text().collect::<String>();
                Ok(ElementType::Link { href, text })
            },
            "section" => Ok(ElementType::Section),
            "article" => Ok(ElementType::Article),
            "header" => Ok(ElementType::Header),
            "footer" => Ok(ElementType::Footer),
            "strong" | "b" => Ok(ElementType::Strong),
            "em" | "i" => Ok(ElementType::Emphasis),
            "code" | "pre" => Ok(ElementType::Code),
            "blockquote" | "q" => Ok(ElementType::Quote),
            _ => Ok(ElementType::Text),
        }
    }
    
    /// Extract comprehensive formatting information from element styles and classes
    fn extract_formatting(&self, element: ElementRef) -> ElementFormatting {
        let style_attr = element.value().attr("style").unwrap_or("");
        let class_attr = element.value().attr("class").unwrap_or("");
        
        let css_classes: Vec<String> = class_attr
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();
        
        // Parse inline styles
        let mut formatting = ElementFormatting {
            font_family: None,
            font_size: None,
            font_weight: None,
            font_style: None,
            color: None,
            background_color: None,
            text_align: None,
            line_height: None,
            margin: None,
            padding: None,
            border: None,
            width: None,
            height: None,
            display: None,
            position: None,
            css_classes,
        };
        
        // Parse style attribute
        for style_rule in style_attr.split(';') {
            if let Some((property, value)) = style_rule.split_once(':') {
                let property = property.trim();
                let value = value.trim().to_string();
                
                match property {
                    "font-family" => formatting.font_family = Some(value),
                    "font-size" => formatting.font_size = Some(value),
                    "font-weight" => formatting.font_weight = Some(value),
                    "font-style" => formatting.font_style = Some(value),
                    "color" => formatting.color = Some(value),
                    "background-color" => formatting.background_color = Some(value),
                    "text-align" => formatting.text_align = Some(value),
                    "line-height" => formatting.line_height = Some(value),
                    "margin" => formatting.margin = Some(value),
                    "padding" => formatting.padding = Some(value),
                    "border" => formatting.border = Some(value),
                    "width" => formatting.width = Some(value),
                    "height" => formatting.height = Some(value),
                    "display" => formatting.display = Some(value),
                    "position" => formatting.position = Some(value),
                    _ => {}
                }
            }
        }
        
        formatting
    }
    
    /// Extract positioning information (for documents with layout data)
    fn extract_position(&self, element: ElementRef) -> ElementPosition {
        // Try to extract position from data attributes or computed styles
        let top = element.value().attr("data-top")
            .and_then(|s| s.parse().ok());
        let left = element.value().attr("data-left")
            .and_then(|s| s.parse().ok());
        let width = element.value().attr("data-width")
            .and_then(|s| s.parse().ok());
        let height = element.value().attr("data-height")
            .and_then(|s| s.parse().ok());
        let page_number = element.value().attr("data-page")
            .and_then(|s| s.parse().ok());
        
        ElementPosition {
            page_number,
            top,
            left,
            width,
            height,
            z_index: None,
        }
    }
    
    /// Extract all HTML attributes as key-value pairs
    fn extract_attributes(&self, element: ElementRef) -> HashMap<String, String> {
        element.value().attrs()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect()
    }
    
    /// Extract table headers and data
    fn extract_table_data(&self, table_element: ElementRef) -> ChonkerResult<(Vec<String>, Vec<Vec<TableCell>>)> {
        let header_selector = Selector::parse("th").unwrap();
        let row_selector = Selector::parse("tr").unwrap();
        let cell_selector = Selector::parse("td, th").unwrap();
        
        // Extract headers
        let headers: Vec<String> = table_element.select(&header_selector)
            .map(|th| th.text().collect::<String>().trim().to_string())
            .collect();
        
        // Extract data rows
        let mut rows = Vec::new();
        for row in table_element.select(&row_selector) {
            let mut row_cells = Vec::new();
            
            for cell in row.select(&cell_selector) {
                let content = cell.text().collect::<String>().trim().to_string();
                let cell_type = if cell.value().name() == "th" {
                    CellType::Header
                } else {
                    CellType::Data
                };
                
                let colspan = cell.value().attr("colspan")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(1);
                let rowspan = cell.value().attr("rowspan")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(1);
                
                let table_cell = TableCell {
                    content,
                    cell_type,
                    formatting: self.extract_formatting(cell),
                    span: CellSpan { colspan, rowspan },
                    attributes: self.extract_attributes(cell),
                };
                
                row_cells.push(table_cell);
            }
            
            if !row_cells.is_empty() {
                rows.push(row_cells);
            }
        }
        
        Ok((headers, rows))
    }
    
    /// Extract table caption if present
    fn extract_table_caption(&self, table_element: ElementRef) -> Option<String> {
        let caption_selector = Selector::parse("caption").unwrap();
        table_element.select(&caption_selector)
            .next()
            .map(|caption| caption.text().collect::<String>().trim().to_string())
            .filter(|s| !s.is_empty())
    }
    
    /// Extract list items
    fn extract_list_items(&self, list_element: ElementRef) -> Vec<String> {
        let item_selector = Selector::parse("li, dt, dd").unwrap();
        list_element.select(&item_selector)
            .map(|item| item.text().collect::<String>().trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }
}

impl DocumentSelectors {
    pub fn new() -> ChonkerResult<Self> {
        Ok(Self {
            // Text elements
            headings: Selector::parse("h1, h2, h3, h4, h5, h6").map_err(Self::selector_error)?,
            paragraphs: Selector::parse("p").map_err(Self::selector_error)?,
            text_content: Selector::parse("p, span, div, text").map_err(Self::selector_error)?,
            
            // Structural elements
            tables: Selector::parse("table").map_err(Self::selector_error)?,
            table_headers: Selector::parse("th").map_err(Self::selector_error)?,
            table_rows: Selector::parse("tr").map_err(Self::selector_error)?,
            table_cells: Selector::parse("td, th").map_err(Self::selector_error)?,
            lists: Selector::parse("ul, ol, dl").map_err(Self::selector_error)?,
            list_items: Selector::parse("li, dt, dd").map_err(Self::selector_error)?,
            
            // Content elements
            images: Selector::parse("img").map_err(Self::selector_error)?,
            links: Selector::parse("a[href]").map_err(Self::selector_error)?,
            
            // Document structure
            sections: Selector::parse("section, .section").map_err(Self::selector_error)?,
            articles: Selector::parse("article, .article").map_err(Self::selector_error)?,
            headers: Selector::parse("header, .header").map_err(Self::selector_error)?,
            footers: Selector::parse("footer, .footer").map_err(Self::selector_error)?,
            
            // Formatting elements
            strong: Selector::parse("strong, b, .bold").map_err(Self::selector_error)?,
            emphasis: Selector::parse("em, i, .italic").map_err(Self::selector_error)?,
            code: Selector::parse("code, pre, .code").map_err(Self::selector_error)?,
            quotes: Selector::parse("blockquote, q, .quote").map_err(Self::selector_error)?,
            
            // Special patterns for document types
            financial_tables: Selector::parse(".financial-table").map_err(Self::selector_error)?,
            data_tables: Selector::parse("table, .data-table").map_err(Self::selector_error)?,
            form_elements: Selector::parse("form, input, select, textarea, .form").map_err(Self::selector_error)?,
            highlighted_content: Selector::parse(".highlight, .important, .note, .warning, .error, mark").map_err(Self::selector_error)?,
        })
    }
    
    fn selector_error(e: scraper::error::SelectorErrorKind) -> ChonkerError {
        ChonkerError::PdfProcessing {
            message: format!("Invalid CSS selector: {:?}", e),
            source: Some(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData, 
                "CSS selector compilation failed"
            ))),
        }
    }
}

impl Default for HtmlExtractor {
    fn default() -> Self {
        Self::new().expect("Failed to create HTML extractor")
    }
}

/// Convert HtmlDocument to the existing DocumentChunk format for compatibility
impl HtmlDocument {
    pub fn to_document_chunks(&self) -> Vec<crate::app::DocumentChunk> {
        let mut chunks = Vec::new();
        let mut chunk_id = 1;
        
        for element in &self.elements {
            self.element_to_chunks(element, &mut chunks, &mut chunk_id);
        }
        
        chunks
    }
    
    fn element_to_chunks(&self, element: &HtmlElement, chunks: &mut Vec<crate::app::DocumentChunk>, chunk_id: &mut usize) {
        let (content, element_types) = match &element.element_type {
            ElementType::Heading { level } => {
                (element.content.clone(), vec![format!("heading_level_{}", level), "heading".to_string()])
            },
            ElementType::Paragraph => {
                (element.content.clone(), vec!["paragraph".to_string(), "text".to_string()])
            },
            ElementType::Table { headers, rows, caption } => {
                let mut table_content = String::new();
                
                // Add table caption if present
                if let Some(cap) = caption {
                    if !cap.is_empty() {
                        table_content.push_str(&format!("**{}**\n\n", cap));
                    }
                }
                
                // Determine column count
                let max_cols = headers.len().max(
                    rows.iter().map(|row| row.len()).max().unwrap_or(0)
                );
                
                if max_cols > 0 {
                    // Create proper markdown table with alignment
                    if !headers.is_empty() {
                        // Add headers with proper spacing
                        table_content.push_str("| ");
                        for (i, header) in headers.iter().enumerate() {
                            let clean_header = header.trim().replace("\n", " ").replace("|", "\\|");
                            table_content.push_str(&clean_header);
                            if i < max_cols - 1 {
                                table_content.push_str(" | ");
                            }
                        }
                        // Fill remaining columns if headers < max_cols
                        for i in headers.len()..max_cols {
                            if i < max_cols - 1 {
                                table_content.push_str(" | ");
                            }
                        }
                        table_content.push_str(" |\n");
                        
                        // Add separator row with proper alignment
                        table_content.push_str("|");
                        for i in 0..max_cols {
                            table_content.push_str("---|")
                        }
                        table_content.push_str("\n");
                    }
                    
                    // Add data rows with proper formatting
                    for row in rows {
                        table_content.push_str("| ");
                        for i in 0..max_cols {
                            let cell_content = if i < row.len() {
                                let cell = &row[i];
                                let clean_content = cell.content.trim()
                                    .replace("\n", " ")
                                    .replace("|", "\\|")
                                    .replace("  ", " "); // Remove double spaces
                                
                                // Preserve formatting hints from cell styling
                                if cell.formatting.font_weight.as_ref().map_or(false, |w| w.contains("bold")) ||
                                   cell.formatting.css_classes.contains(&"bold".to_string()) {
                                    if !clean_content.is_empty() {
                                        format!("**{}**", clean_content)
                                    } else {
                                        clean_content
                                    }
                                } else if cell.formatting.font_style.as_ref().map_or(false, |s| s.contains("italic")) ||
                                         cell.formatting.css_classes.contains(&"italic".to_string()) {
                                    if !clean_content.is_empty() {
                                        format!("*{}*", clean_content)
                                    } else {
                                        clean_content
                                    }
                                } else {
                                    clean_content
                                }
                            } else {
                                "".to_string()
                            };
                            
                            table_content.push_str(&cell_content);
                            if i < max_cols - 1 {
                                table_content.push_str(" | ");
                            }
                        }
                        table_content.push_str(" |\n");
                    }
                } else {
                    // Fallback for empty table
                    table_content.push_str("*[Empty Table]*\n");
                }
                
                table_content.push_str("\n"); // Add spacing after table
                (table_content, vec!["table".to_string(), "structured".to_string()])
            },
            ElementType::List { list_type: _, items } => {
                let list_content = items.iter()
                    .map(|item| format!("- {}", item))
                    .collect::<Vec<_>>()
                    .join("\n");
                (list_content, vec!["list".to_string(), "structured".to_string()])
            },
            ElementType::Image { src: _, alt, caption: _ } => {
                let content = alt.as_ref().unwrap_or(&"[Image]".to_string()).clone();
                (content, vec!["image".to_string(), "media".to_string()])
            },
            _ => {
                (element.content.clone(), vec!["text".to_string()])
            }
        };
        
        if !content.trim().is_empty() {
            let page_range = element.position.page_number
                .map(|p| format!("page_{}", p))
                .unwrap_or_else(|| format!("element_{}", chunk_id));
            
            chunks.push(crate::app::DocumentChunk {
                id: *chunk_id,
                content,
                page_range,
                element_types,
                spatial_bounds: Some(format!(
                    "top:{:?},left:{:?},width:{:?},height:{:?}",
                    element.position.top,
                    element.position.left,
                    element.position.width,
                    element.position.height
                )),
                char_count: element.content.len(),
                table_data: None,
            });
            
            *chunk_id += 1;
        }
        
        // Process children
        for child in &element.children {
            self.element_to_chunks(child, chunks, chunk_id);
        }
    }
}
