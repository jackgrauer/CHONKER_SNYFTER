use arrow::array::{
    Array, ArrayRef, StringArray, Int64Array, Float64Array, TimestampMillisecondArray, BooleanArray
};
use arrow::datatypes::{DataType, Field, Schema, TimeUnit};
use arrow::record_batch::RecordBatch;
use parquet::arrow::{ArrowWriter, arrow_reader::ParquetRecordBatchReaderBuilder};
use parquet::file::properties::WriterProperties;
use parquet::basic::Compression;
use sqlx::{SqlitePool, Row};
use anyhow::{Result, anyhow};
use std::fs::File;
use std::path::Path;
use std::sync::Arc;
use tracing::{info, debug};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParquetExportOptions {
    pub compression: CompressionType,
    pub batch_size: usize,
    pub include_metadata: bool,
    pub include_spatial_data: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionType {
    None,
    Snappy,
    Gzip,
    Lz4,
    Zstd,
}

impl Default for ParquetExportOptions {
    fn default() -> Self {
        Self {
            compression: CompressionType::Snappy,
            batch_size: 1000,
            include_metadata: true,
            include_spatial_data: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DocumentExportData {
    pub document_id: String,
    pub filename: String,
    pub chunk_id: String,
    pub chunk_index: i64,
    pub content: String,
    pub page_range: String,
    pub element_types: String, // JSON array as string
    pub char_count: i64,
    pub spatial_bounds: Option<String>, // JSON as string
    pub complexity_score: Option<f64>,
    pub processing_time_ms: i64,
    pub processing_path: String,
    pub created_at: i64, // Timestamp in milliseconds
    pub has_tables: bool,
    pub has_images: bool,
    pub has_forms: bool,
}

pub struct ParquetExporter {
    pool: SqlitePool,
    schema: Arc<Schema>,
}

impl ParquetExporter {
    pub fn new(pool: SqlitePool) -> Self {
        let schema = Arc::new(Self::create_schema());
        Self { pool, schema }
    }
    
    fn create_schema() -> Schema {
        Schema::new(vec![
            // Document identifiers
            Field::new("document_id", DataType::Utf8, false),
            Field::new("filename", DataType::Utf8, false),
            
            // Chunk information
            Field::new("chunk_id", DataType::Utf8, false),
            Field::new("chunk_index", DataType::Int64, false),
            Field::new("content", DataType::Utf8, false),
            Field::new("page_range", DataType::Utf8, true),
            Field::new("element_types", DataType::Utf8, true),
            Field::new("char_count", DataType::Int64, false),
            Field::new("spatial_bounds", DataType::Utf8, true),
            
            // Processing metadata
            Field::new("complexity_score", DataType::Float64, true),
            Field::new("processing_time_ms", DataType::Int64, false),
            Field::new("processing_path", DataType::Utf8, true),
            Field::new("created_at", DataType::Timestamp(TimeUnit::Millisecond, None), false),
            
            // Document characteristics (boolean flags for analytics)
            Field::new("has_tables", DataType::Boolean, false),
            Field::new("has_images", DataType::Boolean, false),
            Field::new("has_forms", DataType::Boolean, false),
        ])
    }
    
    /// Export all documents and chunks to Parquet format
    pub async fn export_all<P: AsRef<Path>>(&self, output_path: P, options: ParquetExportOptions) -> Result<()> {
        info!("📊 Starting Parquet export to: {:?}", output_path.as_ref());
        
        let file = File::create(output_path.as_ref())?;
        
        // Configure compression
        let compression = match options.compression {
            CompressionType::None => Compression::UNCOMPRESSED,
            CompressionType::Snappy => Compression::SNAPPY,
            CompressionType::Gzip => Compression::GZIP(Default::default()),
            CompressionType::Lz4 => Compression::LZ4,
            CompressionType::Zstd => Compression::ZSTD(Default::default()),
        };
        
        let props = WriterProperties::builder()
            .set_compression(compression)
            .set_write_batch_size(options.batch_size)
            .build();
            
        let mut writer = ArrowWriter::try_new(file, self.schema.clone(), Some(props))?;
        
        // Process data in batches to manage memory usage
        let mut offset = 0;
        let batch_size = options.batch_size as i64;
        
        loop {
            let data = self.fetch_export_data(batch_size, offset).await?;
            
            if data.is_empty() {
                break;
            }
            
            let record_batch = self.create_record_batch(&data)?;
            writer.write(&record_batch)?;
            
            offset += batch_size;
            debug!("📊 Exported batch: {} records", data.len());
        }
        
        writer.close()?;
        
        info!("✅ Parquet export completed successfully");
        Ok(())
    }
    
    /// Export specific document to Parquet
    pub async fn export_document<P: AsRef<Path>>(&self, document_id: &str, output_path: P, options: ParquetExportOptions) -> Result<()> {
        info!("📊 Exporting document {} to Parquet", document_id);
        
        let data = self.fetch_document_data(document_id).await?;
        
        if data.is_empty() {
            return Err(anyhow!("No data found for document: {}", document_id));
        }
        
        let file = File::create(output_path.as_ref())?;
        
        let compression = match options.compression {
            CompressionType::None => Compression::UNCOMPRESSED,
            CompressionType::Snappy => Compression::SNAPPY,
            CompressionType::Gzip => Compression::GZIP(Default::default()),
            CompressionType::Lz4 => Compression::LZ4,
            CompressionType::Zstd => Compression::ZSTD(Default::default()),
        };
        
        let props = WriterProperties::builder()
            .set_compression(compression)
            .build();
            
        let mut writer = ArrowWriter::try_new(file, self.schema.clone(), Some(props))?;
        
        let record_batch = self.create_record_batch(&data)?;
        writer.write(&record_batch)?;
        writer.close()?;
        
        info!("✅ Document {} exported to Parquet", document_id);
        Ok(())
    }
    
    /// Fetch export data from database with joins
    async fn fetch_export_data(&self, limit: i64, offset: i64) -> Result<Vec<DocumentExportData>> {
        let rows = sqlx::query(
            r#"
            SELECT 
                d.id as document_id,
                d.filename,
                c.id as chunk_id,
                c.chunk_index,
                c.content,
                c.page_range,
                c.element_types,
                c.char_count,
                c.spatial_bounds,
                d.processing_time_ms,
                d.created_at,
                CASE 
                    WHEN c.element_types LIKE '%table%' THEN 1 
                    ELSE 0 
                END as has_tables,
                CASE 
                    WHEN c.element_types LIKE '%image%' THEN 1 
                    ELSE 0 
                END as has_images,
                CASE 
                    WHEN c.element_types LIKE '%form%' THEN 1 
                    ELSE 0 
                END as has_forms
            FROM documents d
            JOIN chunks c ON d.id = c.document_id
            ORDER BY d.created_at DESC, c.chunk_index ASC
            LIMIT ? OFFSET ?
            "#
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;
        
        let mut export_data = Vec::new();
        
        for row in rows {
            let created_at_str: String = row.get("created_at");
            let created_at_timestamp = chrono::DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|e| anyhow!("Failed to parse timestamp: {}", e))?
                .timestamp_millis();
                
            export_data.push(DocumentExportData {
                document_id: row.get("document_id"),
                filename: row.get("filename"),
                chunk_id: row.get("chunk_id"),
                chunk_index: row.get("chunk_index"),
                content: row.get("content"),
                page_range: row.get::<Option<String>, _>("page_range").unwrap_or_default(),
                element_types: row.get::<Option<String>, _>("element_types").unwrap_or_default(),
                char_count: row.get("char_count"),
                spatial_bounds: row.get("spatial_bounds"),
                complexity_score: None, // TODO: Add complexity score to documents table
                processing_time_ms: row.get("processing_time_ms"),
                processing_path: "unknown".to_string(), // TODO: Add processing path to documents table
                created_at: created_at_timestamp,
                has_tables: row.get::<i64, _>("has_tables") == 1,
                has_images: row.get::<i64, _>("has_images") == 1,
                has_forms: row.get::<i64, _>("has_forms") == 1,
            });
        }
        
        Ok(export_data)
    }
    
    /// Fetch data for a specific document
    async fn fetch_document_data(&self, document_id: &str) -> Result<Vec<DocumentExportData>> {
        let rows = sqlx::query(
            r#"
            SELECT 
                d.id as document_id,
                d.filename,
                c.id as chunk_id,
                c.chunk_index,
                c.content,
                c.page_range,
                c.element_types,
                c.char_count,
                c.spatial_bounds,
                d.processing_time_ms,
                d.created_at,
                CASE 
                    WHEN c.element_types LIKE '%table%' THEN 1 
                    ELSE 0 
                END as has_tables,
                CASE 
                    WHEN c.element_types LIKE '%image%' THEN 1 
                    ELSE 0 
                END as has_images,
                CASE 
                    WHEN c.element_types LIKE '%form%' THEN 1 
                    ELSE 0 
                END as has_forms
            FROM documents d
            JOIN chunks c ON d.id = c.document_id
            WHERE d.id = ?
            ORDER BY c.chunk_index ASC
            "#
        )
        .bind(document_id)
        .fetch_all(&self.pool)
        .await?;
        
        let mut export_data = Vec::new();
        
        for row in rows {
            let created_at_str: String = row.get("created_at");
            let created_at_timestamp = chrono::DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|e| anyhow!("Failed to parse timestamp: {}", e))?
                .timestamp_millis();
                
            export_data.push(DocumentExportData {
                document_id: row.get("document_id"),
                filename: row.get("filename"),
                chunk_id: row.get("chunk_id"),
                chunk_index: row.get("chunk_index"),
                content: row.get("content"),
                page_range: row.get::<Option<String>, _>("page_range").unwrap_or_default(),
                element_types: row.get::<Option<String>, _>("element_types").unwrap_or_default(),
                char_count: row.get("char_count"),
                spatial_bounds: row.get("spatial_bounds"),
                complexity_score: None,
                processing_time_ms: row.get("processing_time_ms"),
                processing_path: "unknown".to_string(),
                created_at: created_at_timestamp,
                has_tables: row.get::<i64, _>("has_tables") == 1,
                has_images: row.get::<i64, _>("has_images") == 1,
                has_forms: row.get::<i64, _>("has_forms") == 1,
            });
        }
        
        Ok(export_data)
    }
    
    /// Create Arrow RecordBatch from export data
    fn create_record_batch(&self, data: &[DocumentExportData]) -> Result<RecordBatch> {
        let len = data.len();
        
        // Create arrays for each column
        let document_ids: ArrayRef = Arc::new(StringArray::from(
            data.iter().map(|d| d.document_id.as_str()).collect::<Vec<_>>()
        ));
        
        let filenames: ArrayRef = Arc::new(StringArray::from(
            data.iter().map(|d| d.filename.as_str()).collect::<Vec<_>>()
        ));
        
        let chunk_ids: ArrayRef = Arc::new(StringArray::from(
            data.iter().map(|d| d.chunk_id.as_str()).collect::<Vec<_>>()
        ));
        
        let chunk_indices: ArrayRef = Arc::new(Int64Array::from(
            data.iter().map(|d| d.chunk_index).collect::<Vec<_>>()
        ));
        
        let contents: ArrayRef = Arc::new(StringArray::from(
            data.iter().map(|d| d.content.as_str()).collect::<Vec<_>>()
        ));
        
        let page_ranges: ArrayRef = Arc::new(StringArray::from(
            data.iter().map(|d| Some(d.page_range.as_str())).collect::<Vec<_>>()
        ));
        
        let element_types: ArrayRef = Arc::new(StringArray::from(
            data.iter().map(|d| Some(d.element_types.as_str())).collect::<Vec<_>>()
        ));
        
        let char_counts: ArrayRef = Arc::new(Int64Array::from(
            data.iter().map(|d| d.char_count).collect::<Vec<_>>()
        ));
        
        let spatial_bounds: ArrayRef = Arc::new(StringArray::from(
            data.iter().map(|d| d.spatial_bounds.as_deref()).collect::<Vec<_>>()
        ));
        
        let complexity_scores: ArrayRef = Arc::new(Float64Array::from(
            data.iter().map(|d| d.complexity_score).collect::<Vec<_>>()
        ));
        
        let processing_times: ArrayRef = Arc::new(Int64Array::from(
            data.iter().map(|d| d.processing_time_ms).collect::<Vec<_>>()
        ));
        
        let processing_paths: ArrayRef = Arc::new(StringArray::from(
            data.iter().map(|d| Some(d.processing_path.as_str())).collect::<Vec<_>>()
        ));
        
        let timestamps: ArrayRef = Arc::new(TimestampMillisecondArray::from(
            data.iter().map(|d| Some(d.created_at)).collect::<Vec<_>>()
        ));
        
        let has_tables: ArrayRef = Arc::new(BooleanArray::from(
            data.iter().map(|d| Some(d.has_tables)).collect::<Vec<_>>()
        ));
        
        let has_images: ArrayRef = Arc::new(BooleanArray::from(
            data.iter().map(|d| Some(d.has_images)).collect::<Vec<_>>()
        ));
        
        let has_forms: ArrayRef = Arc::new(BooleanArray::from(
            data.iter().map(|d| Some(d.has_forms)).collect::<Vec<_>>()
        ));
        
        let record_batch = RecordBatch::try_new(
            self.schema.clone(),
            vec![
                document_ids,
                filenames,
                chunk_ids,
                chunk_indices,
                contents,
                page_ranges,
                element_types,
                char_counts,
                spatial_bounds,
                complexity_scores,
                processing_times,
                processing_paths,
                timestamps,
                has_tables,
                has_images,
                has_forms,
            ]
        )?;
        
        Ok(record_batch)
    }
    
    /// Read Parquet file back to verify export
    pub fn read_parquet<P: AsRef<Path>>(path: P) -> Result<Vec<DocumentExportData>> {
        let file = File::open(path.as_ref())?;
        let reader = ParquetRecordBatchReaderBuilder::try_new(file)?.build()?;
        
        let mut all_data = Vec::new();
        
        for batch_result in reader {
            let batch = batch_result?;
            
            for row_idx in 0..batch.num_rows() {
                // Extract data from each column
                let document_id = batch.column(0).as_any()
                    .downcast_ref::<StringArray>()
                    .unwrap()
                    .value(row_idx)
                    .to_string();
                    
                let filename = batch.column(1).as_any()
                    .downcast_ref::<StringArray>()
                    .unwrap()
                    .value(row_idx)
                    .to_string();
                    
                // Continue for other fields as needed...
                // This is a simplified example
                
                all_data.push(DocumentExportData {
                    document_id,
                    filename,
                    chunk_id: "".to_string(),
                    chunk_index: 0,
                    content: "".to_string(),
                    page_range: "".to_string(),
                    element_types: "".to_string(),
                    char_count: 0,
                    spatial_bounds: None,
                    complexity_score: None,
                    processing_time_ms: 0,
                    processing_path: "".to_string(),
                    created_at: 0,
                    has_tables: false,
                    has_images: false,
                    has_forms: false,
                });
            }
        }
        
        Ok(all_data)
    }
    
    /// Generate Python loading script for the exported Parquet file
    pub fn generate_python_script<P: AsRef<Path>>(parquet_path: P, script_path: P) -> Result<()> {
        let python_code = format!(
            r#"#!/usr/bin/env python3
"""
CHONKER Parquet Data Loader
Generated automatically by CHONKER_SNYFTER
"""

import pandas as pd
import pyarrow.parquet as pq
import pyarrow as pa
from pathlib import Path

def load_chonker_data(parquet_file: str = "{}") -> pd.DataFrame:
    """Load CHONKER data from Parquet file."""
    
    # Read the Parquet file
    df = pd.read_parquet(parquet_file)
    
    # Parse timestamps
    df['created_at'] = pd.to_datetime(df['created_at'])
    
    # Parse JSON columns
    df['element_types_parsed'] = df['element_types'].apply(
        lambda x: eval(x) if x else []
    )
    
    print(f"Loaded {{len(df)}} chunks from {{df['document_id'].nunique()}} documents")
    print(f"Date range: {{df['created_at'].min()}} to {{df['created_at'].max()}}")
    
    return df

def analyze_content_types(df: pd.DataFrame):
    """Analyze document content types."""
    
    print("\\nContent Analysis:")
    print(f"Documents with tables: {{df['has_tables'].sum()}}")
    print(f"Documents with images: {{df['has_images'].sum()}}")
    print(f"Documents with forms: {{df['has_forms'].sum()}}")
    
    print("\\nTop file types:")
    file_extensions = df['filename'].str.extract(r'\.([^.]+)$')[0]
    print(file_extensions.value_counts().head())

def processing_performance_analysis(df: pd.DataFrame):
    """Analyze processing performance."""
    
    print("\\nProcessing Performance:")
    print(f"Average processing time: {{df['processing_time_ms'].mean():.1f}}ms")
    print(f"Average chunk size: {{df['char_count'].mean():.0f}} characters")
    
    # Performance by processing path
    if 'processing_path' in df.columns:
        perf_by_path = df.groupby('processing_path')['processing_time_ms'].agg(['mean', 'count'])
        print("\\nPerformance by processing path:")
        print(perf_by_path)

if __name__ == "__main__":
    # Load and analyze data
    df = load_chonker_data()
    analyze_content_types(df)
    processing_performance_analysis(df)
    
    # Display sample data
    print("\\nSample data:")
    print(df.head())
"#,
            parquet_path.as_ref().display()
        );
        
        std::fs::write(script_path.as_ref(), python_code)?;
        
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(script_path.as_ref())?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(script_path.as_ref(), perms)?;
        }
        
        info!("📝 Generated Python loading script: {:?}", script_path.as_ref());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_schema_creation() {
        let schema = ParquetExporter::create_schema();
        assert_eq!(schema.fields().len(), 16);
        assert_eq!(schema.field(0).name(), "document_id");
        assert_eq!(schema.field(1).name(), "filename");
    }
    
    #[test]
    fn test_compression_options() {
        let options = ParquetExportOptions {
            compression: CompressionType::Snappy,
            batch_size: 500,
            include_metadata: true,
            include_spatial_data: false,
        };
        
        assert_eq!(options.batch_size, 500);
        assert!(options.include_metadata);
    }
}
