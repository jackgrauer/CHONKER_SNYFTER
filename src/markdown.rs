use anyhow::Result;
use pulldown_cmark::{Parser, Options, html};
use std::collections::HashMap;
use tracing::info;

/// Markdown processor for corrections and transformations
pub struct MarkdownProcessor {
    correction_rules: HashMap<String, String>,
}

impl MarkdownProcessor {
    pub fn new() -> Self {
        let mut correction_rules = HashMap::new();
        
        // Common OCR corrections
        correction_rules.insert("rn".to_string(), "m".to_string());
        correction_rules.insert("l".to_string(), "I".to_string()); // Context-dependent
        correction_rules.insert("0".to_string(), "O".to_string()); // Context-dependent
        correction_rules.insert("5".to_string(), "S".to_string()); // Context-dependent
        
        // Common formatting fixes
        correction_rules.insert(" ,".to_string(), ",".to_string());
        correction_rules.insert(" .".to_string(), ".".to_string());
        correction_rules.insert("( ".to_string(), "(".to_string());
        correction_rules.insert(" )".to_string(), ")".to_string());
        
        Self {
            correction_rules,
        }
    }
    
    /// Apply corrections to markdown content
    pub fn apply_corrections(&self, content: &str) -> Result<String> {
        info!("Applying markdown corrections");
        
        let mut corrected = content.to_string();
        
        // Apply basic corrections
        for (pattern, replacement) in &self.correction_rules {
            corrected = corrected.replace(pattern, replacement);
        }
        
        // Fix multiple spaces
        corrected = regex::Regex::new(r" +")?.replace_all(&corrected, " ").to_string();
        
        // Fix line breaks (remove single line breaks, keep double line breaks)
        // Split on double newlines, then process each section
        let sections: Vec<&str> = corrected.split("\n\n").collect();
        let processed_sections: Vec<String> = sections.iter()
            .map(|section| section.replace("\n", " "))
            .collect();
        corrected = processed_sections.join("\n\n");
        
        // Fix bullet points
        corrected = regex::Regex::new(r"^[\u2022\u2023\u25E6\u2043\u2219]\s*")?.replace_all(&corrected, "• ").to_string();
        
        // Normalize headers
        corrected = self.normalize_headers(&corrected)?;
        
        // Fix table formatting
        corrected = self.fix_table_formatting(&corrected)?;
        
        Ok(corrected)
    }
    
    /// Normalize markdown content without aggressive corrections
    pub fn normalize(&self, content: &str) -> Result<String> {
        info!("Normalizing markdown content");
        
        let mut normalized = content.to_string();
        
        // Fix multiple spaces
        normalized = regex::Regex::new(r" +")?.replace_all(&normalized, " ").to_string();
        
        // Normalize line endings
        normalized = normalized.replace("\r\n", "\n").replace("\r", "\n");
        
        // Fix excessive blank lines
        normalized = regex::Regex::new(r"\n{3,}")?.replace_all(&normalized, "\n\n").to_string();
        
        // Trim whitespace from lines
        normalized = normalized
            .lines()
            .map(|line| line.trim_end())
            .collect::<Vec<_>>()
            .join("\n");
        
        Ok(normalized)
    }
    
    /// Normalize header formatting
    fn normalize_headers(&self, content: &str) -> Result<String> {
        let mut result = content.to_string();
        
        // Fix headers with inconsistent spacing
        result = regex::Regex::new(r"^(#+)\s*(.+)$")?.replace_all(&result, "$1 $2").to_string();
        
        // Ensure blank line after headers
        result = regex::Regex::new(r"(^#+.*$)(\n)([^#\n])")?.replace_all(&result, "$1\n\n$3").to_string();
        
        Ok(result)
    }
    
    /// Fix table formatting issues
    fn fix_table_formatting(&self, content: &str) -> Result<String> {
        let mut result = content.to_string();
        
        // Fix pipe spacing in tables
        result = regex::Regex::new(r"\s*\|\s*")?.replace_all(&result, " | ").to_string();
        
        // Simple table header detection and separation
        let lines: Vec<String> = result.lines().map(|s| s.to_string()).collect();
        let mut fixed_lines = Vec::new();
        
        for (i, line) in lines.iter().enumerate() {
            fixed_lines.push(line.clone());
            
            // If this looks like a table header and next line doesn't look like separator
            if line.contains('|') && line.split('|').count() > 2 {
                if i + 1 < lines.len() {
                    let next_line = &lines[i + 1];
                    if !next_line.contains("---") && !next_line.contains("===") {
                        // Add separator row
                        let sep_count = line.split('|').count() - 1;
                        let separator = format!("{}{}---|", "|", "---|".repeat(sep_count - 1));
                        fixed_lines.push(separator);
                    }
                }
            }
        }
        
        Ok(fixed_lines.join("\n"))
    }
    
    /// Validate markdown syntax
    pub fn validate(&self, content: &str) -> Result<Vec<String>> {
        let mut issues = Vec::new();
        
        // Check for common markdown issues
        let options = Options::empty();
        let parser = Parser::new_ext(content, options);
        
        for event in parser {
            match event {
                pulldown_cmark::Event::Start(pulldown_cmark::Tag::CodeBlock(_)) => {
                    // Could check for unclosed code blocks
                },
                pulldown_cmark::Event::Start(pulldown_cmark::Tag::Link { dest_url, .. }) => {
                    // Could validate URLs
                    if dest_url.is_empty() {
                        issues.push("Empty link URL found".to_string());
                    }
                },
                _ => {}
            }
        }
        
        Ok(issues)
    }
    
    /// Convert markdown to HTML for preview
    pub fn to_html(&self, content: &str) -> Result<String> {
        let options = Options::all();
        let parser = Parser::new_ext(content, options);
        
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);
        
        Ok(html_output)
    }
    
    /// Extract text content from markdown (strip formatting)
    pub fn to_plain_text(&self, content: &str) -> Result<String> {
        let options = Options::all();
        let parser = Parser::new_ext(content, options);
        
        let mut text = String::new();
        for event in parser {
            match event {
                pulldown_cmark::Event::Text(t) => text.push_str(&t),
                pulldown_cmark::Event::Code(c) => text.push_str(&c),
                pulldown_cmark::Event::SoftBreak | pulldown_cmark::Event::HardBreak => text.push('\n'),
                _ => {}
            }
        }
        
        Ok(text)
    }
    
    /// Get word count and other statistics
    pub fn get_stats(&self, content: &str) -> Result<MarkdownStats> {
        let plain_text = self.to_plain_text(content)?;
        
        let word_count = plain_text.split_whitespace().count();
        let char_count = plain_text.chars().count();
        let line_count = content.lines().count();
        
        // Count markdown elements
        let header_count = content.lines().filter(|line| line.trim_start().starts_with('#')).count();
        let code_block_count = content.matches("```").count() / 2;
        let table_count = content.lines().filter(|line| line.contains('|')).count();
        
        Ok(MarkdownStats {
            word_count,
            char_count,
            line_count,
            header_count,
            code_block_count,
            table_count,
        })
    }
}

#[derive(Debug)]
pub struct MarkdownStats {
    pub word_count: usize,
    pub char_count: usize,
    pub line_count: usize,
    pub header_count: usize,
    pub code_block_count: usize,
    pub table_count: usize,
}

impl Default for MarkdownProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize() {
        let processor = MarkdownProcessor::new();
        let input = "# Header\n\n\nToo many    spaces\n\n\n\nAnother line";
        let result = processor.normalize(input).unwrap();
        assert!(!result.contains("   "));
        assert!(!result.contains("\n\n\n"));
    }

    #[test]
    fn test_apply_corrections() {
        let processor = MarkdownProcessor::new();
        let input = "This is a test , with spacing issues .";
        let result = processor.apply_corrections(input).unwrap();
        assert_eq!(result, "This is a test, with spacing issues.");
    }

    #[test]
    fn test_stats() {
        let processor = MarkdownProcessor::new();
        let input = "# Header\n\nThis is a test with **bold** text.\n\n```code```";
        let stats = processor.get_stats(input).unwrap();
        assert_eq!(stats.header_count, 1);
        assert_eq!(stats.code_block_count, 1);
        assert!(stats.word_count > 0);
    }
}
