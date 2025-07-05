use crate::database::DocumentChunk;
use crate::error::{ChonkerError, ChonkerResult};
use tracing::{debug, info, warn};
use serde::{Deserialize, Serialize};

/// Byte range within the original document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ByteRange {
    pub start: usize,
    pub end: usize,
}

/// Types of structured elements in a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StructuredElement {
    Table {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
        bounds: ByteRange,
        table_id: Option<String>,
        caption: Option<String>,
        otsl_content: Option<String>, // Store OTSL representation
    },
    Text {
        content: String,
        element_type: TextType,
        bounds: ByteRange,
    },
    List {
        items: Vec<String>,
        ordered: bool,
        bounds: ByteRange,
    },
    Code {
        content: String,
        language: Option<String>,
        bounds: ByteRange,
    },
    Image {
        caption: Option<String>,
        description: Option<String>,
        bounds: ByteRange,
    },
    Formula {
        content: String,
        latex: Option<String>,
        bounds: ByteRange,
    },
    Heading {
        content: String,
        level: u8,
        bounds: ByteRange,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextType {
    Paragraph,
    Caption,
    Quote,
    Footnote,
    Reference,
    Abstract,
    Unknown,
}

/// Complete document structure parsed from Docling output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentStructure {
    pub elements: Vec<StructuredElement>,
    pub metadata: DocumentMetadata,
    pub original_content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub total_pages: usize,
    pub total_elements: usize,
    pub table_count: usize,
    pub figure_count: usize,
    pub formula_count: usize,
    pub processing_tool: String,
}

/// Smart chunker that respects document structure
pub struct SmartChunker {
    max_chunk_size: usize,
    min_chunk_size: usize,
    preserve_context: bool,
    overlap_size: usize,
}

impl Default for SmartChunker {
    fn default() -> Self {
        Self {
            max_chunk_size: 8000, // Characters, not bytes
            min_chunk_size: 1000,
            preserve_context: true,
            overlap_size: 200,
        }
    }
}

impl SmartChunker {
    pub fn new(max_size: usize, min_size: usize) -> Self {
        Self {
            max_chunk_size: max_size,
            min_chunk_size: min_size,
            preserve_context: true,
            overlap_size: max_size / 40, // 2.5% overlap
        }
    }

    /// Parse Docling output into structured document representation
    pub fn parse_docling_output(&self, docling_result: &serde_json::Value) -> ChonkerResult<DocumentStructure> {
        debug!("🔍 Parsing Docling output into structured document");

        let extractions = docling_result["extractions"]
            .as_array()
            .ok_or_else(|| ChonkerError::PdfProcessing {
                message: "No extractions found in Docling output".to_string(),
                source: None,
            })?;

        let mut elements = Vec::new();
        let mut current_position = 0;

        for extraction in extractions {
            let text = extraction["text"].as_str().unwrap_or("");
            let empty_vec = vec![];
            let structured_tables = extraction["tables"].as_array().unwrap_or(&empty_vec);
            
            // Parse the text content for structure
            let text_elements = self.parse_text_structure(text, &mut current_position)?;
            elements.extend(text_elements);

            // Process structured tables
            for (table_idx, table_data) in structured_tables.iter().enumerate() {
                if let Some(table_element) = self.parse_table_structure(table_data, &mut current_position)? {
                    elements.push(table_element);
                }
            }
        }

        let metadata = self.extract_metadata(docling_result)?;

        Ok(DocumentStructure {
            elements,
            metadata,
            original_content: serde_json::to_string_pretty(docling_result)
                .map_err(|e| ChonkerError::PdfProcessing {
                    message: format!("Failed to serialize docling result: {}", e),
                    source: None,
                })?,
        })
    }

    /// Parse text content for structural elements (headings, paragraphs, lists, etc.)
    fn parse_text_structure(&self, text: &str, position: &mut usize) -> ChonkerResult<Vec<StructuredElement>> {
        let mut elements = Vec::new();
        let lines: Vec<&str> = text.lines().collect();
        let mut current_line = 0;

        while current_line < lines.len() {
            let line = lines[current_line].trim();

            if line.is_empty() {
                current_line += 1;
                continue;
            }

            // Detect OTSL tables
            if line.starts_with("<otsl>") {
                if let Some((table_element, lines_consumed)) = self.parse_otsl_table(&lines[current_line..], position)? {
                    elements.push(table_element);
                    current_line += lines_consumed;
                    continue;
                }
            }

            // Detect headings
            if line.starts_with('#') {
                let level = line.chars().take_while(|&c| c == '#').count() as u8;
                let content = line.trim_start_matches('#').trim().to_string();
                let start_pos = *position;
                *position += line.len() + 1; // +1 for newline

                elements.push(StructuredElement::Heading {
                    content,
                    level,
                    bounds: ByteRange { start: start_pos, end: *position },
                });
                current_line += 1;
                continue;
            }

            // Detect lists
            if line.starts_with('-') || line.starts_with('*') || line.starts_with('+') {
                if let Some((list_element, lines_consumed)) = self.parse_list(&lines[current_line..], position)? {
                    elements.push(list_element);
                    current_line += lines_consumed;
                    continue;
                }
            }

            // Detect numbered lists
            if let Some(_caps) = regex::Regex::new(r"^\d+\.\s").unwrap().captures(line) {
                if let Some((list_element, lines_consumed)) = self.parse_numbered_list(&lines[current_line..], position)? {
                    elements.push(list_element);
                    current_line += lines_consumed;
                    continue;
                }
            }

            // Detect code blocks
            if line.starts_with("```") {
                if let Some((code_element, lines_consumed)) = self.parse_code_block(&lines[current_line..], position)? {
                    elements.push(code_element);
                    current_line += lines_consumed;
                    continue;
                }
            }

            // Default: treat as paragraph text
            let (paragraph, lines_consumed) = self.parse_paragraph(&lines[current_line..], position)?;
            elements.push(paragraph);
            current_line += lines_consumed;
        }

        Ok(elements)
    }

    /// Parse OTSL table markup into structured table element
    fn parse_otsl_table(&self, lines: &[&str], position: &mut usize) -> ChonkerResult<Option<(StructuredElement, usize)>> {
        let mut line_idx = 0;
        let mut headers = Vec::new();
        let mut rows = Vec::new();
        let mut in_thead = false;
        let mut in_tbody = false;
        let mut current_row = Vec::new();
        let start_pos = *position;

        // Find the end of the table
        let mut table_end = 0;
        for (i, &line) in lines.iter().enumerate() {
            if line.trim() == "</otsl>" {
                table_end = i + 1;
                break;
            }
        }

        if table_end == 0 {
            warn!("OTSL table not properly closed");
            return Ok(None);
        }

        // Parse table content
        for &line in &lines[1..table_end-1] { // Skip <otsl> and </otsl>
            let trimmed = line.trim();
            
            if trimmed == "<thead>" {
                in_thead = true;
            } else if trimmed == "</thead>" {
                in_thead = false;
                if !current_row.is_empty() {
                    headers = current_row.clone();
                    current_row.clear();
                }
            } else if trimmed == "<tbody>" {
                in_tbody = true;
            } else if trimmed == "</tbody>" {
                in_tbody = false;
                if !current_row.is_empty() {
                    rows.push(current_row.clone());
                    current_row.clear();
                }
            } else if trimmed == "<tr>" {
                current_row.clear();
            } else if trimmed == "</tr>" {
                if in_thead && !current_row.is_empty() {
                    headers = current_row.clone();
                } else if in_tbody && !current_row.is_empty() {
                    rows.push(current_row.clone());
                }
                current_row.clear();
            } else if trimmed.starts_with("<rhed>") && trimmed.ends_with("</rhed>") {
                let content = trimmed.trim_start_matches("<rhed>").trim_end_matches("</rhed>");
                current_row.push(content.to_string());
            } else if trimmed.starts_with("<fcel>") && trimmed.ends_with("</fcel>") {
                let content = trimmed.trim_start_matches("<fcel>").trim_end_matches("</fcel>");
                current_row.push(content.to_string());
            }
        }

        // Calculate total size for position tracking
        let total_size: usize = lines[..table_end].iter().map(|line| line.len() + 1).sum();
        *position += total_size;

        // Create OTSL content string
        let otsl_content = lines[..table_end].join("\n");

        Ok(Some((
            StructuredElement::Table {
                headers,
                rows,
                bounds: ByteRange { start: start_pos, end: *position },
                table_id: None,
                caption: None,
                otsl_content: Some(otsl_content),
            },
            table_end,
        )))
    }

    /// Parse a list structure
    fn parse_list(&self, lines: &[&str], position: &mut usize) -> ChonkerResult<Option<(StructuredElement, usize)>> {
        let mut items = Vec::new();
        let mut line_idx = 0;
        let start_pos = *position;

        while line_idx < lines.len() {
            let line = lines[line_idx].trim();
            
            if line.starts_with('-') || line.starts_with('*') || line.starts_with('+') {
                let item = line[1..].trim().to_string();
                if !item.is_empty() {
                    items.push(item);
                }
                *position += lines[line_idx].len() + 1;
                line_idx += 1;
            } else if line.is_empty() {
                *position += 1;
                line_idx += 1;
            } else {
                break; // End of list
            }
        }

        if items.is_empty() {
            return Ok(None);
        }

        Ok(Some((
            StructuredElement::List {
                items,
                ordered: false,
                bounds: ByteRange { start: start_pos, end: *position },
            },
            line_idx,
        )))
    }

    /// Parse a numbered list structure
    fn parse_numbered_list(&self, lines: &[&str], position: &mut usize) -> ChonkerResult<Option<(StructuredElement, usize)>> {
        let mut items = Vec::new();
        let mut line_idx = 0;
        let start_pos = *position;

        while line_idx < lines.len() {
            let line = lines[line_idx].trim();
            
            if let Some(caps) = regex::Regex::new(r"^(\d+)\.\s(.+)").unwrap().captures(line) {
                let item = caps.get(2).unwrap().as_str().to_string();
                items.push(item);
                *position += lines[line_idx].len() + 1;
                line_idx += 1;
            } else if line.is_empty() {
                *position += 1;
                line_idx += 1;
            } else {
                break; // End of list
            }
        }

        if items.is_empty() {
            return Ok(None);
        }

        Ok(Some((
            StructuredElement::List {
                items,
                ordered: true,
                bounds: ByteRange { start: start_pos, end: *position },
            },
            line_idx,
        )))
    }

    /// Parse a code block
    fn parse_code_block(&self, lines: &[&str], position: &mut usize) -> ChonkerResult<Option<(StructuredElement, usize)>> {
        let first_line = lines[0];
        let language = if first_line.len() > 3 {
            Some(first_line[3..].trim().to_string())
        } else {
            None
        };

        let mut content_lines = Vec::new();
        let mut line_idx = 1;
        let start_pos = *position;

        *position += first_line.len() + 1; // Opening ```

        while line_idx < lines.len() {
            let line = lines[line_idx];
            
            if line.trim() == "```" {
                *position += line.len() + 1; // Closing ```
                line_idx += 1;
                break;
            }
            
            content_lines.push(line);
            *position += line.len() + 1;
            line_idx += 1;
        }

        let content = content_lines.join("\n");

        Ok(Some((
            StructuredElement::Code {
                content,
                language,
                bounds: ByteRange { start: start_pos, end: *position },
            },
            line_idx,
        )))
    }

    /// Parse a paragraph of text
    fn parse_paragraph(&self, lines: &[&str], position: &mut usize) -> ChonkerResult<(StructuredElement, usize)> {
        let mut content_lines = Vec::new();
        let mut line_idx = 0;
        let start_pos = *position;

        while line_idx < lines.len() {
            let line = lines[line_idx];
            
            // Stop at structural boundaries
            if line.trim().is_empty() ||
               line.trim().starts_with('#') ||
               line.trim().starts_with("```") ||
               line.trim().starts_with('-') ||
               line.trim().starts_with('*') ||
               line.trim().starts_with('+') ||
               regex::Regex::new(r"^\d+\.\s").unwrap().is_match(line.trim()) ||
               line.trim().starts_with("<otsl>") {
                break;
            }
            
            content_lines.push(line);
            *position += line.len() + 1;
            line_idx += 1;
        }

        // Handle trailing empty line
        if line_idx < lines.len() && lines[line_idx].trim().is_empty() {
            *position += 1;
            line_idx += 1;
        }

        let content = content_lines.join("\n");

        Ok((
            StructuredElement::Text {
                content,
                element_type: TextType::Paragraph,
                bounds: ByteRange { start: start_pos, end: *position },
            },
            line_idx,
        ))
    }

    /// Parse structured table data from Docling JSON
    fn parse_table_structure(&self, table_data: &serde_json::Value, position: &mut usize) -> ChonkerResult<Option<StructuredElement>> {
        if let Some(processed_data) = table_data["processed_data"].as_array() {
            let mut headers = Vec::new();
            let mut rows = Vec::new();

            // Extract headers (first row typically)
            if let Some(first_row) = processed_data.first() {
                if let Some(header_array) = first_row.as_array() {
                    headers = header_array.iter()
                        .map(|v| v.as_str().unwrap_or("").to_string())
                        .collect();
                }
            }

            // Extract data rows (skip first row if it's headers)
            for row_value in processed_data.iter().skip(if headers.is_empty() { 0 } else { 1 }) {
                if let Some(row_array) = row_value.as_array() {
                    let row: Vec<String> = row_array.iter()
                        .map(|v| v.as_str().unwrap_or("").to_string())
                        .collect();
                    rows.push(row);
                }
            }

            if !headers.is_empty() || !rows.is_empty() {
                let start_pos = *position;
                let estimated_size = headers.iter().map(|h| h.len()).sum::<usize>() +
                                   rows.iter().map(|r| r.iter().map(|c| c.len()).sum::<usize>()).sum::<usize>();
                *position += estimated_size;

                return Ok(Some(StructuredElement::Table {
                    headers,
                    rows,
                    bounds: ByteRange { start: start_pos, end: *position },
                    table_id: table_data["table_index"].as_u64().map(|i| format!("table_{}", i)),
                    caption: None,
                    otsl_content: None, // Will be generated during chunking
                }));
            }
        }

        Ok(None)
    }

    /// Extract metadata from Docling result
    fn extract_metadata(&self, docling_result: &serde_json::Value) -> ChonkerResult<DocumentMetadata> {
        let metadata = &docling_result["metadata"];
        
        Ok(DocumentMetadata {
            total_pages: metadata["total_pages"].as_u64().unwrap_or(1) as usize,
            total_elements: 0, // Will be filled after parsing
            table_count: metadata["tables_found"].as_u64().unwrap_or(0) as usize,
            figure_count: metadata["figures_found"].as_u64().unwrap_or(0) as usize,
            formula_count: 0, // Will be calculated
            processing_tool: metadata.get("tool").and_then(|t| t.as_str()).unwrap_or("unknown").to_string(),
        })
    }

    /// Main chunking function that respects document structure
    pub fn chunk_document(&self, structure: DocumentStructure) -> ChonkerResult<Vec<DocumentChunk>> {
        info!("🧩 Starting smart chunking with {} elements", structure.elements.len());
        
        let mut chunks = Vec::new();
        let mut current_chunk = ChunkBuilder::new(self.max_chunk_size);
        let mut chunk_id = 1;

        for element in structure.elements {
            // Check if element fits in current chunk
            if !self.can_fit(&current_chunk, &element) {
                // Finalize current chunk at natural boundary
                if !current_chunk.is_empty() {
                    chunks.push(current_chunk.build(chunk_id, &structure.metadata)?);
                    chunk_id += 1;
                }
                current_chunk = ChunkBuilder::new(self.max_chunk_size);
            }

            match &element {
                StructuredElement::Table { .. } if self.element_size(&element) > self.max_chunk_size => {
                    // Split large tables intelligently
                    let table_chunks = self.split_table_with_context(element, chunk_id)?;
                    let table_chunks_len = table_chunks.len();
                    chunks.extend(table_chunks);
                    chunk_id += table_chunks_len as i64;
                }
                _ => {
                    current_chunk.add_element(element);
                }
            }
        }

        // Add final chunk if it has content
        if !current_chunk.is_empty() {
            chunks.push(current_chunk.build(chunk_id, &structure.metadata)?);
        }

        info!("✅ Smart chunking complete: {} chunks created", chunks.len());
        Ok(chunks)
    }

    /// Check if an element can fit in the current chunk
    fn can_fit(&self, chunk_builder: &ChunkBuilder, element: &StructuredElement) -> bool {
        let element_size = self.element_size(element);
        chunk_builder.current_size() + element_size <= self.max_chunk_size
    }

    /// Calculate the size of an element
    fn element_size(&self, element: &StructuredElement) -> usize {
        match element {
            StructuredElement::Table { headers, rows, otsl_content, .. } => {
                if let Some(otsl) = otsl_content {
                    otsl.len()
                } else {
                    headers.iter().map(|h| h.len()).sum::<usize>() +
                    rows.iter().map(|r| r.iter().map(|c| c.len()).sum::<usize>()).sum::<usize>()
                }
            }
            StructuredElement::Text { content, .. } => content.len(),
            StructuredElement::List { items, .. } => items.iter().map(|i| i.len()).sum(),
            StructuredElement::Code { content, .. } => content.len(),
            StructuredElement::Heading { content, .. } => content.len(),
            StructuredElement::Image { caption, description, .. } => {
                caption.as_ref().map(|c| c.len()).unwrap_or(0) +
                description.as_ref().map(|d| d.len()).unwrap_or(0)
            }
            StructuredElement::Formula { content, .. } => content.len(),
        }
    }

    /// Split large tables while preserving context
    fn split_table_with_context(&self, table_element: StructuredElement, mut chunk_id: i64) -> ChonkerResult<Vec<DocumentChunk>> {
        let mut chunks = Vec::new();

        if let StructuredElement::Table { headers, rows, table_id, caption, .. } = table_element {
            let header_size = headers.iter().map(|h| h.len()).sum::<usize>();
            let avg_row_size = if rows.is_empty() { 
                0 
            } else { 
                rows.iter().map(|r| r.iter().map(|c| c.len()).sum::<usize>()).sum::<usize>() / rows.len()
            };

            // Calculate how many rows can fit per chunk (keeping headers)
            let available_space = self.max_chunk_size - header_size - 200; // Buffer for markup
            let rows_per_chunk = if avg_row_size > 0 {
                std::cmp::max(1, available_space / avg_row_size)
            } else {
                rows.len()
            };

            info!("📊 Splitting large table: {} rows into chunks of {} rows each", rows.len(), rows_per_chunk);

            // Split table into chunks, always including headers
            for (chunk_idx, row_chunk) in rows.chunks(rows_per_chunk).enumerate() {
                let mut chunk_builder = ChunkBuilder::new(self.max_chunk_size);
                
                // Add table continuation metadata
                let table_caption = if chunk_idx == 0 {
                    caption.clone()
                } else {
                    Some(format!("{} (continued - part {})", 
                        caption.as_deref().unwrap_or("Table"), chunk_idx + 1))
                };

                let table_chunk = StructuredElement::Table {
                    headers: headers.clone(),
                    rows: row_chunk.to_vec(),
                    bounds: ByteRange { start: 0, end: 0 }, // Reset for chunks
                    table_id: table_id.as_ref().map(|id| format!("{}_part_{}", id, chunk_idx + 1)),
                    caption: table_caption,
                    otsl_content: None, // Will be generated
                };

                chunk_builder.add_element(table_chunk);
                
                let metadata = DocumentMetadata {
                    total_pages: 1,
                    total_elements: 1,
                    table_count: 1,
                    figure_count: 0,
                    formula_count: 0,
                    processing_tool: "smart_chunker".to_string(),
                };

                chunks.push(chunk_builder.build(chunk_id, &metadata)?);
                chunk_id += 1;
            }
        }

        Ok(chunks)
    }
}

/// Builder for creating document chunks
struct ChunkBuilder {
    elements: Vec<StructuredElement>,
    max_size: usize,
}

impl ChunkBuilder {
    fn new(max_size: usize) -> Self {
        Self {
            elements: Vec::new(),
            max_size,
        }
    }

    fn add_element(&mut self, element: StructuredElement) {
        self.elements.push(element);
    }

    fn current_size(&self) -> usize {
        self.elements.iter().map(|e| self.element_size(e)).sum()
    }

    fn element_size(&self, element: &StructuredElement) -> usize {
        match element {
            StructuredElement::Table { headers, rows, .. } => {
                headers.iter().map(|h| h.len()).sum::<usize>() +
                rows.iter().map(|r| r.iter().map(|c| c.len()).sum::<usize>()).sum::<usize>()
            }
            StructuredElement::Text { content, .. } => content.len(),
            StructuredElement::List { items, .. } => items.iter().map(|i| i.len()).sum(),
            StructuredElement::Code { content, .. } => content.len(),
            StructuredElement::Heading { content, .. } => content.len(),
            StructuredElement::Image { caption, description, .. } => {
                caption.as_ref().map(|c| c.len()).unwrap_or(0) +
                description.as_ref().map(|d| d.len()).unwrap_or(0)
            }
            StructuredElement::Formula { content, .. } => content.len(),
        }
    }

    fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    fn build(self, chunk_id: i64, metadata: &DocumentMetadata) -> ChonkerResult<DocumentChunk> {
        let mut content_parts = Vec::new();
        let mut element_types = Vec::new();
        let total_chars: usize;

        for element in self.elements {
            match element {
                StructuredElement::Table { headers, rows, otsl_content, caption, table_id, .. } => {
                    element_types.push("table".to_string());
                    
                    // Generate OTSL content if not provided
                    let table_content = if let Some(otsl) = otsl_content {
                        otsl
                    } else {
                        ChunkBuilder::generate_otsl_content_static(&headers, &rows, caption.as_deref(), table_id.as_deref())
                    };
                    
                    content_parts.push(table_content);
                }
                StructuredElement::Text { content, element_type, .. } => {
                    element_types.push(format!("text_{:?}", element_type).to_lowercase());
                    content_parts.push(content);
                }
                StructuredElement::List { items, ordered, .. } => {
                    element_types.push("list".to_string());
                    let list_content = if ordered {
                        items.iter().enumerate()
                            .map(|(i, item)| format!("{}. {}", i + 1, item))
                            .collect::<Vec<_>>()
                            .join("\n")
                    } else {
                        items.iter()
                            .map(|item| format!("- {}", item))
                            .collect::<Vec<_>>()
                            .join("\n")
                    };
                    content_parts.push(list_content);
                }
                StructuredElement::Code { content, language, .. } => {
                    element_types.push("code".to_string());
                    let code_content = if let Some(lang) = language {
                        format!("```{}\n{}\n```", lang, content)
                    } else {
                        format!("```\n{}\n```", content)
                    };
                    content_parts.push(code_content);
                }
                StructuredElement::Heading { content, level, .. } => {
                    element_types.push("heading".to_string());
                    let heading_content = format!("{} {}", "#".repeat(level as usize), content);
                    content_parts.push(heading_content);
                }
                StructuredElement::Image { caption, description, .. } => {
                    element_types.push("image".to_string());
                    let mut image_content = String::new();
                    if let Some(cap) = caption {
                        image_content.push_str(&format!("![{}]", cap));
                    }
                    if let Some(desc) = description {
                        image_content.push_str(&format!("\n{}", desc));
                    }
                    content_parts.push(image_content);
                }
                StructuredElement::Formula { content, latex, .. } => {
                    element_types.push("formula".to_string());
                    let formula_content = if let Some(latex_code) = latex {
                        format!("${}$ ({})", latex_code, content)
                    } else {
                        content
                    };
                    content_parts.push(formula_content);
                }
            }
        }

        let full_content = content_parts.join("\n\n");
        total_chars = full_content.chars().count();

        // Deduplicate element types
        element_types.sort();
        element_types.dedup();

        Ok(DocumentChunk {
            id: chunk_id,
            content: full_content,
            page_range: format!("1-{}", metadata.total_pages), // TODO: Track actual page ranges
            element_types,
            spatial_bounds: None, // TODO: Implement spatial bounds tracking
            char_count: total_chars as i64,
            table_data: None, // TODO: Add structured table data if needed
        })
    }

    fn generate_otsl_content_static(headers: &[String], rows: &[Vec<String>], caption: Option<&str>, table_id: Option<&str>) -> String {
        let mut otsl_lines = Vec::new();
        
        otsl_lines.push("<otsl>".to_string());
        
        // Add caption as comment if provided
        if let Some(cap) = caption {
            otsl_lines.push(format!("<!-- {} -->", cap));
        }
        
        // Add table ID as comment if provided
        if let Some(id) = table_id {
            otsl_lines.push(format!("<!-- Table ID: {} -->", id));
        }

        // Add headers
        if !headers.is_empty() {
            otsl_lines.push("<thead>".to_string());
            otsl_lines.push("<tr>".to_string());
            for header in headers {
                otsl_lines.push(format!("<rhed>{}</rhed>", header));
            }
            otsl_lines.push("</tr>".to_string());
            otsl_lines.push("</thead>".to_string());
        }

        // Add rows
        if !rows.is_empty() {
            otsl_lines.push("<tbody>".to_string());
            for row in rows {
                otsl_lines.push("<tr>".to_string());
                for (i, cell) in row.iter().enumerate() {
                    if i == 0 && !headers.is_empty() {
                        // First column could be row header
                        otsl_lines.push(format!("<rhed>{}</rhed>", cell));
                    } else {
                        otsl_lines.push(format!("<fcel>{}</fcel>", cell));
                    }
                }
                otsl_lines.push("</tr>".to_string());
            }
            otsl_lines.push("</tbody>".to_string());
        }

        otsl_lines.push("</otsl>".to_string());
        
        otsl_lines.join("\n")
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smart_chunker_creation() {
        let chunker = SmartChunker::default();
        assert_eq!(chunker.max_chunk_size, 8000);
        assert_eq!(chunker.min_chunk_size, 1000);
    }

    #[test]
    fn test_element_size_calculation() {
        let chunker = SmartChunker::default();
        
        let text_element = StructuredElement::Text {
            content: "Hello, world!".to_string(),
            element_type: TextType::Paragraph,
            bounds: ByteRange { start: 0, end: 13 },
        };
        
        assert_eq!(chunker.element_size(&text_element), 13);
    }
}
