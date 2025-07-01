//! Smart Column Extraction for Environmental Lab Tables
//! 
//! This module implements intelligent column filtering to reduce table size
//! for Qwen processing while preserving only the essential data needed
//! for qualifier separation.

use anyhow::{anyhow, Result};
use std::collections::HashMap;

/// Essential column types for environmental lab data
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ColumnType {
    Analyte,        // Substance name (Ethylbenzene, Benzene, etc.)
    Concentration,  // Value with embedded qualifier ("0.046 U")
    Qualifier,      // Separate qualifier column (if exists)
    ReportingLimit, // RL column
    DetectionLimit, // MDL column
    Other,          // All other columns
}

/// Represents a table column with its type and index
#[derive(Debug, Clone)]
pub struct TableColumn {
    pub index: usize,
    pub header: String,
    pub column_type: ColumnType,
}

/// Smart column extractor for environmental lab tables
pub struct SmartColumnExtractor {
    essential_keywords: HashMap<ColumnType, Vec<String>>,
}

impl SmartColumnExtractor {
    pub fn new() -> Self {
        let mut keywords = HashMap::new();
        
        keywords.insert(ColumnType::Analyte, vec![
            "analyte".to_string(),
            "parameter".to_string(),
            "compound".to_string(),
            "substance".to_string(),
        ]);
        
        keywords.insert(ColumnType::Concentration, vec![
            "result".to_string(),
            "concentration".to_string(),
            "value".to_string(),
            "conc".to_string(),
        ]);
        
        keywords.insert(ColumnType::Qualifier, vec![
            "qualifier".to_string(),
            "qual".to_string(),
            "q".to_string(),
        ]);
        
        keywords.insert(ColumnType::ReportingLimit, vec![
            "rl".to_string(),
            "reporting limit".to_string(),
            "report limit".to_string(),
        ]);
        
        keywords.insert(ColumnType::DetectionLimit, vec![
            "mdl".to_string(),
            "method detection limit".to_string(),
            "detection limit".to_string(),
        ]);
        
        Self {
            essential_keywords: keywords,
        }
    }
    
    /// Extract only essential columns from a large table
    pub fn extract_essential_columns(&self, table_chunk: &str) -> Result<String> {
        let lines: Vec<&str> = table_chunk.lines().collect();
        if lines.len() < 2 {
            return Ok(table_chunk.to_string());
        }
        
        // Find header row (usually first row with |)
        let header_row = lines.iter()
            .find(|line| line.contains('|') && line.split('|').count() > 3)
            .ok_or_else(|| anyhow!("No valid table header found"))?;
        
        // Parse column headers
        let columns = self.classify_columns(header_row)?;
        let essential_indices = self.get_essential_column_indices(&columns);
        
        if essential_indices.is_empty() {
            return Err(anyhow!("No essential columns found"));
        }
        
        // Extract only essential columns from all rows
        let filtered_lines: Vec<String> = lines.iter()
            .filter(|line| line.contains('|'))
            .map(|line| self.extract_columns_by_indices(line, &essential_indices))
            .collect::<Result<Vec<_>>>()?;
        
        let result = filtered_lines.join("\n");
        
        // Log the size reduction
        let original_size = table_chunk.len();
        let filtered_size = result.len();
        let reduction_pct = (1.0 - (filtered_size as f64 / original_size as f64)) * 100.0;
        
        eprintln!("📊 Column filtering: {} chars → {} chars ({:.1}% reduction)", 
                 original_size, filtered_size, reduction_pct);
        
        Ok(result)
    }
    
    /// Classify table columns by their content
    fn classify_columns(&self, header_row: &str) -> Result<Vec<TableColumn>> {
        let headers: Vec<String> = header_row
            .split('|')
            .enumerate()
            .filter_map(|(i, h)| {
                let trimmed = h.trim();
                if !trimmed.is_empty() {
                    Some(trimmed.to_lowercase())
                } else {
                    None
                }
            })
            .collect();
        
        let mut columns = Vec::new();
        
        for (index, header) in headers.iter().enumerate() {
            let column_type = self.classify_single_column(header);
            columns.push(TableColumn {
                index,
                header: header.clone(),
                column_type,
            });
        }
        
        Ok(columns)
    }
    
    /// Classify a single column header
    fn classify_single_column(&self, header: &str) -> ColumnType {
        let header_lower = header.to_lowercase();
        
        for (col_type, keywords) in &self.essential_keywords {
            for keyword in keywords {
                if header_lower.contains(keyword) {
                    return col_type.clone();
                }
            }
        }
        
        ColumnType::Other
    }
    
    /// Get indices of essential columns only
    fn get_essential_column_indices(&self, columns: &[TableColumn]) -> Vec<usize> {
        let mut indices = Vec::new();
        
        // Always include analyte column (for row identification)
        if let Some(analyte_col) = columns.iter().find(|c| matches!(c.column_type, ColumnType::Analyte)) {
            indices.push(analyte_col.index);
        }
        
        // Include all concentration columns (where qualifiers are embedded)
        for col in columns {
            match col.column_type {
                ColumnType::Concentration | ColumnType::Qualifier => {
                    if !indices.contains(&col.index) {
                        indices.push(col.index);
                    }
                }
                _ => {}
            }
        }
        
        // Include 1-2 limit columns for context
        let mut limit_count = 0;
        for col in columns {
            if limit_count >= 2 { break; }
            match col.column_type {
                ColumnType::ReportingLimit | ColumnType::DetectionLimit => {
                    if !indices.contains(&col.index) {
                        indices.push(col.index);
                        limit_count += 1;
                    }
                }
                _ => {}
            }
        }
        
        indices.sort();
        indices
    }
    
    /// Extract specific columns from a table row
    fn extract_columns_by_indices(&self, row: &str, indices: &[usize]) -> Result<String> {
        let parts: Vec<&str> = row.split('|').collect();
        
        let mut result_parts = vec![""]; // Start with empty for first |
        
        for &index in indices {
            if index < parts.len() {
                result_parts.push(parts[index]);
            } else {
                result_parts.push(""); // Missing column
            }
        }
        result_parts.push(""); // End with empty for last |
        
        Ok(result_parts.join("|"))
    }
    
    /// Merge fixed essential columns back into original table
    pub fn merge_fixed_columns_back(&self, original_table: &str, fixed_essential: &str) -> Result<String> {
        // Parse both tables into row structures
        let original_rows = self.parse_table_rows(original_table)?;
        let fixed_rows = self.parse_table_rows(fixed_essential)?;
        
        // Create mapping from analyte to fixed concentration values
        let fix_mapping = self.create_fix_mapping(&fixed_rows)?;
        
        // Apply fixes to original table
        let mut result_lines = Vec::new();
        
        for original_row in &original_rows {
            if let Some(analyte) = self.extract_analyte_from_row(original_row) {
                if let Some(fixed_concentrations) = fix_mapping.get(&analyte) {
                    // Replace concentration columns with fixed versions
                    let updated_row = self.apply_fixes_to_original_row(original_row, fixed_concentrations)?;
                    result_lines.push(updated_row);
                } else {
                    // No fixes for this row, keep original
                    result_lines.push(original_row.join("|"));
                }
            } else {
                // No analyte found (header row, etc.), keep original
                result_lines.push(original_row.join("|"));
            }
        }
        
        Ok(result_lines.join("\n"))
    }
    
    /// Parse table into rows of columns
    fn parse_table_rows(&self, table: &str) -> Result<Vec<Vec<String>>> {
        let rows: Vec<Vec<String>> = table.lines()
            .filter(|line| line.contains('|'))
            .map(|line| {
                line.split('|')
                    .map(|col| col.trim().to_string())
                    .collect()
            })
            .collect();
        
        Ok(rows)
    }
    
    /// Extract analyte name from a table row
    fn extract_analyte_from_row(&self, row: &[String]) -> Option<String> {
        // Usually the first non-empty column is the analyte
        for col in row {
            let trimmed = col.trim();
            if !trimmed.is_empty() && trimmed != "|" {
                return Some(trimmed.to_string());
            }
        }
        None
    }
    
    /// Create mapping from analyte names to fixed concentration values
    fn create_fix_mapping(&self, fixed_rows: &[Vec<String>]) -> Result<HashMap<String, Vec<String>>> {
        let mut mapping = HashMap::new();
        
        for row in fixed_rows {
            if let Some(analyte) = self.extract_analyte_from_row(row) {
                // Skip header rows
                if analyte.to_lowercase().contains("analyte") || 
                   analyte.to_lowercase().contains("parameter") {
                    continue;
                }
                
                // Extract concentration values (skip analyte column)
                let concentrations: Vec<String> = row.iter()
                    .skip(1)
                    .map(|s| s.clone())
                    .collect();
                
                mapping.insert(analyte, concentrations);
            }
        }
        
        Ok(mapping)
    }
    
    /// Apply fixed concentration values to original row
    fn apply_fixes_to_original_row(&self, original_row: &[String], fixed_concentrations: &[String]) -> Result<String> {
        // For now, implement a simple strategy:
        // Find concentration-like columns in original and replace them
        let mut result_row = original_row.to_vec();
        
        // This is a simplified version - in practice, you'd want more sophisticated
        // column matching based on the header analysis
        for (i, original_col) in original_row.iter().enumerate() {
            // If this column looks like a concentration with qualifier
            if self.looks_like_concentration_with_qualifier(original_col) {
                // Try to find a corresponding fixed value
                if let Some(fixed_val) = fixed_concentrations.get(0) {
                    result_row[i] = fixed_val.clone();
                }
            }
        }
        
        Ok(result_row.join("|"))
    }
    
    /// Check if a column value looks like a concentration with embedded qualifier
    fn looks_like_concentration_with_qualifier(&self, value: &str) -> bool {
        let trimmed = value.trim();
        // Look for patterns like "0.046 U" or "0.000851 J"
        let qualifier_pattern = regex::Regex::new(r"\b\d+\.?\d*\s+[UJB]\b").unwrap();
        qualifier_pattern.is_match(trimmed)
    }
}

/// Process a large table using smart column extraction
pub fn process_large_table_with_column_extraction(
    table_chunk: &str,
    qwen_processor: impl Fn(&str) -> Result<String>
) -> Result<String> {
    let extractor = SmartColumnExtractor::new();
    
    // Check if table is too large for direct processing
    if table_chunk.len() < 8000 {
        // Small enough - process normally
        return qwen_processor(table_chunk);
    }
    
    eprintln!("🔍 Large table detected ({} chars), using smart column extraction", table_chunk.len());
    
    // Extract only essential columns
    let essential_data = extractor.extract_essential_columns(table_chunk)?;
    
    if essential_data.len() < 8000 {
        // Column filtering worked - process the filtered data
        eprintln!("✅ Column filtering successful, processing {} chars", essential_data.len());
        let fixed_essential = qwen_processor(&essential_data)?;
        
        // Merge fixed columns back into original table
        return extractor.merge_fixed_columns_back(table_chunk, &fixed_essential);
    } else {
        // Still too big even after column filtering - chunk by rows
        eprintln!("⚠️  Still too large after column filtering, using row chunking");
        return process_essential_data_in_row_chunks(&essential_data, table_chunk, qwen_processor);
    }
}

/// Process filtered essential data in row chunks if still too large
fn process_essential_data_in_row_chunks(
    essential_data: &str,
    original_table: &str,
    qwen_processor: impl Fn(&str) -> Result<String>
) -> Result<String> {
    let lines: Vec<&str> = essential_data.lines().collect();
    if lines.is_empty() {
        return Ok(original_table.to_string());
    }
    
    // Keep header row
    let header = lines[0];
    let data_rows = &lines[1..];
    
    // Process in chunks of 20 rows
    let chunk_size = 20;
    let mut all_fixed_rows = vec![header.to_string()];
    
    for chunk in data_rows.chunks(chunk_size) {
        let chunk_data = [&[header], chunk].concat().join("\n");
        
        if chunk_data.len() < 8000 {
            match qwen_processor(&chunk_data) {
                Ok(fixed_chunk) => {
                    // Extract just the data rows (skip header) and convert to owned strings
                    let fixed_lines: Vec<String> = fixed_chunk.lines().map(|s| s.to_string()).collect();
                    if fixed_lines.len() > 1 {
                        all_fixed_rows.extend_from_slice(&fixed_lines[1..]);
                    }
                }
                Err(e) => {
                    eprintln!("⚠️  Chunk processing failed: {}, keeping original", e);
                    all_fixed_rows.extend(chunk.iter().map(|s| s.to_string()));
                }
            }
        } else {
            eprintln!("⚠️  Chunk still too large, keeping original");
            all_fixed_rows.extend(chunk.iter().map(|s| s.to_string()));
        }
    }
    
    let fixed_essential = all_fixed_rows.join("\n");
    
    // Merge back into original table
    let extractor = SmartColumnExtractor::new();
    extractor.merge_fixed_columns_back(original_table, &fixed_essential)
}
