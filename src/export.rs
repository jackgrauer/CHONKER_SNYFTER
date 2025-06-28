use anyhow::Result;
use polars::prelude::*;
use std::path::Path;
use tracing::info;
use crate::database::ChonkerDatabase;

/// DataFrame exporter for various output formats
pub struct DataFrameExporter {
    database: ChonkerDatabase,
}

impl DataFrameExporter {
    pub fn new(database: ChonkerDatabase) -> Self {
        Self { database }
    }
    
    /// Export documents to CSV format
    pub async fn export_to_csv(&self, output_path: &Path, doc_id_filter: Option<&str>) -> Result<()> {
        info!("Exporting to CSV: {:?}", output_path);
        
        let df = self.create_dataframe(doc_id_filter).await?;
        
        let mut file = std::fs::File::create(output_path)?;
        CsvWriter::new(&mut file)
            .include_header(true)
            .finish(&mut df.clone())?;
        
        info!("CSV export completed: {} rows", df.height());
        Ok(())
    }
    
    /// Export documents to JSON format
    pub async fn export_to_json(&self, output_path: &Path, doc_id_filter: Option<&str>) -> Result<()> {
        info!("Exporting to JSON: {:?}", output_path);
        
        let df = self.create_dataframe(doc_id_filter).await?;
        
        let mut file = std::fs::File::create(output_path)?;
        JsonWriter::new(&mut file).finish(&mut df.clone())?;
        
        info!("JSON export completed: {} rows", df.height());
        Ok(())
    }
    
    /// Export documents to Parquet format
    pub async fn export_to_parquet(&self, output_path: &Path, doc_id_filter: Option<&str>) -> Result<()> {
        info!("Exporting to Parquet: {:?}", output_path);
        
        let df = self.create_dataframe(doc_id_filter).await?;
        
        let file = std::fs::File::create(output_path)?;
        ParquetWriter::new(file).finish(&mut df.clone())?;
        
        info!("Parquet export completed: {} rows", df.height());
        Ok(())
    }
    
    /// Create DataFrame from database documents
    async fn create_dataframe(&self, doc_id_filter: Option<&str>) -> Result<DataFrame> {
        // Extract data for DataFrame columns
        let mut doc_ids: Vec<String> = Vec::new();
        let mut filenames: Vec<String> = Vec::new();
        let mut chunk_ids: Vec<i64> = Vec::new();
        let mut contents: Vec<String> = Vec::new();
        let mut page_ranges: Vec<String> = Vec::new();
        let mut element_types: Vec<String> = Vec::new();
        let mut char_counts: Vec<i64> = Vec::new();
        let mut created_ats: Vec<String> = Vec::new();
        
        if let Some(doc_id) = doc_id_filter {
            // Get chunks for specific document
            let chunks = self.database.get_document_chunks(doc_id).await?;
            let documents = self.database.get_all_documents().await?;
            let doc = documents.iter().find(|d| d.id == doc_id);
            
            if let Some(doc) = doc {
                for chunk in chunks {
                    doc_ids.push(doc.id.clone());
                    filenames.push(doc.filename.clone());
                    chunk_ids.push(chunk.id);
                    contents.push(chunk.content);
                    page_ranges.push(chunk.page_range);
                    element_types.push(chunk.element_types.join(", "));
                    char_counts.push(chunk.char_count);
                    created_ats.push(doc.created_at.clone());
                }
            }
        } else {
            // Get all documents and their chunks
            let documents = self.database.get_all_documents().await?;
            
            for doc in documents {
                let chunks = self.database.get_document_chunks(&doc.id).await?;
                
                for chunk in chunks {
                    doc_ids.push(doc.id.clone());
                    filenames.push(doc.filename.clone());
                    chunk_ids.push(chunk.id);
                    contents.push(chunk.content);
                    page_ranges.push(chunk.page_range);
                    element_types.push(chunk.element_types.join(", "));
                    char_counts.push(chunk.char_count);
                    created_ats.push(doc.created_at.clone());
                }
            }
        }
        
        if doc_ids.is_empty() {
            return Ok(DataFrame::empty());
        }
        
        // Create DataFrame
        let df = df! {
            "document_id" => doc_ids,
            "filename" => filenames,
            "chunk_id" => chunk_ids,
            "content" => contents,
            "page_range" => page_ranges,
            "element_types" => element_types,
            "char_count" => char_counts,
            "created_at" => created_ats,
        }?;
        
        Ok(df)
    }
    
    /// Get export statistics
    pub async fn get_export_stats(&self, doc_id_filter: Option<&str>) -> Result<ExportStats> {
        let df = self.create_dataframe(doc_id_filter).await?;
        
        let row_count = df.height();
        let total_chars: i64 = df
            .column("char_count")?
            .sum::<i64>()
            .unwrap_or(0);
        
        let unique_docs = df
            .column("document_id")?
            .n_unique()?;
        
        Ok(ExportStats {
            row_count,
            unique_documents: unique_docs,
            total_characters: total_chars,
        })
    }
    
    /// Export with custom query/filtering
    pub async fn export_filtered_csv(
        &self,
        output_path: &Path,
        filter_query: &str,
    ) -> Result<()> {
        info!("Exporting filtered data to CSV: {:?}", output_path);
        
        let df = self.create_dataframe(None).await?;
        
        // Basic filtering - for now just log the query and return all data
        info!("Filter query: {}", filter_query);
        let filtered_df = df;
        
        let mut file = std::fs::File::create(output_path)?;
        CsvWriter::new(&mut file)
            .include_header(true)
            .finish(&mut filtered_df.clone())?;
        
        info!("Filtered CSV export completed: {} rows", filtered_df.height());
        Ok(())
    }
    
    /// Create summary DataFrame with aggregated statistics
    pub async fn export_summary_csv(&self, output_path: &Path) -> Result<()> {
        info!("Exporting summary data to CSV: {:?}", output_path);
        
        let df = self.create_dataframe(None).await?;
        
        let summary_df = df
            .lazy()
            .group_by([col("document_id"), col("filename")])
            .agg([
                col("chunk_id").count().alias("chunk_count"),
                col("char_count").sum().alias("total_chars"),
                col("page_range").first().alias("first_page"),
                col("created_at").first().alias("created_at"),
            ])
            .collect()?;
        
        let mut file = std::fs::File::create(output_path)?;
        CsvWriter::new(&mut file)
            .include_header(true)
            .finish(&mut summary_df.clone())?;
        
        info!("Summary CSV export completed: {} rows", summary_df.height());
        Ok(())
    }
}

#[derive(Debug)]
pub struct ExportStats {
    pub row_count: usize,
    pub unique_documents: usize,
    pub total_characters: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_export_stats() {
        // This would require a test database setup
        // Placeholder for now
    }
}
