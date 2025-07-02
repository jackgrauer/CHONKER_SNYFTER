use crate::error::{ChonkerError, ChonkerResult};
use sqlx::{Sqlite, SqlitePool, Row};
use std::path::Path;
use uuid::Uuid;
use chrono::Utc;
use tracing::{info, debug};
use serde::{Serialize, Deserialize};
use anyhow::Result;

// Import search functionality
mod search;
pub use search::{FTSManager, SearchQuery, SearchResult, SearchOptions};

/// MLX-optimized database configuration
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DatabaseConfig {
    pub connection_pool_size: u32,
    pub batch_size: usize,
    pub enable_wal_mode: bool,
    pub cache_size_mb: u32,
    pub enable_memory_mapping: bool,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            connection_pool_size: 2, // Keep small for MLX memory
            batch_size: 100,         // Align with MLX processing batches
            enable_wal_mode: true,   // Better for concurrent reads
            cache_size_mb: 64,       // Reserve memory for MLX
            enable_memory_mapping: true, // Zero-copy operations
        }
    }
}

/// Main database interface optimized for MLX acceleration
pub struct ChonkerDatabase {
    pool: SqlitePool,
    #[allow(dead_code)]
    config: DatabaseConfig,
}

impl ChonkerDatabase {
    /// Initialize database with MLX-optimized settings
    pub async fn new(db_path: &str, config: DatabaseConfig) -> ChonkerResult<Self> {
        info!("Initializing CHONKER database at: {}", db_path);
        
        // Build connection string with MLX optimizations
        let _cache_size = -(config.cache_size_mb as i32);
        let _mmap_size = if config.enable_memory_mapping { 
            config.cache_size_mb * 1024 * 1024 
        } else { 
            0 
        };
        
        let pool = SqlitePool::connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(db_path)
                .create_if_missing(true)
                .journal_mode(if config.enable_wal_mode { 
                    sqlx::sqlite::SqliteJournalMode::Wal 
                } else { 
                    sqlx::sqlite::SqliteJournalMode::Delete 
                })
                .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
                .pragma("cache_size", format!("-{}", config.cache_size_mb * 1024))
                .pragma("temp_store", "memory")
                .pragma("mmap_size", format!("{}", config.cache_size_mb * 1024 * 1024))
        )
        .await
        .map_err(|e| ChonkerError::database("pool_creation", e))?;
        
        let db = Self { pool, config };
        
        // Run migrations
        db.run_migrations().await?;
        
        info!("🐹 CHONKER database initialized successfully");
        Ok(db)
    }
    
    /// Run database migrations
    async fn run_migrations(&self) -> ChonkerResult<()> {
        info!("Running database migrations");
        
        // Create documents table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS documents (
                id TEXT PRIMARY KEY,
                filename TEXT NOT NULL,
                file_path TEXT NOT NULL,
                file_size INTEGER NOT NULL,
                file_hash TEXT NOT NULL,
                processing_options TEXT NOT NULL, -- JSON
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'processing', -- processing, completed, failed
                total_chunks INTEGER DEFAULT 0,
                processing_time_ms INTEGER DEFAULT 0
            )
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| ChonkerError::database("create_documents_table", e))?;
        
        // Create chunks table optimized for MLX batch operations
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS chunks (
                id TEXT PRIMARY KEY,
                document_id TEXT NOT NULL,
                chunk_index INTEGER NOT NULL,
                content TEXT NOT NULL,
                content_hash TEXT NOT NULL,
                char_count INTEGER NOT NULL,
                page_range TEXT,
                element_types TEXT NOT NULL, -- JSON array
                spatial_bounds TEXT,         -- JSON for MLX processing
                embedding_vector BLOB,       -- For future MLX embeddings
                adversarial_score REAL DEFAULT 0.0,
                created_at TEXT NOT NULL,
                FOREIGN KEY (document_id) REFERENCES documents (id) ON DELETE CASCADE
            )
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| ChonkerError::database("create_chunks_table", e))?;
        
        // Create indexes for efficient querying
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_chunks_document_id ON chunks (document_id)")
            .execute(&self.pool)
            .await
            .map_err(|e| ChonkerError::database("create_index_document_id", e))?;
            
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_chunks_chunk_index ON chunks (chunk_index)")
            .execute(&self.pool)
            .await
            .map_err(|e| ChonkerError::database("create_index_chunk_index", e))?;
            
        // Full-text search index for content
        sqlx::query(
            r#"
            CREATE VIRTUAL TABLE IF NOT EXISTS chunks_fts USING fts5(
                content, 
                document_id UNINDEXED, 
                chunk_id UNINDEXED,
                content='chunks', 
                content_rowid='rowid'
            )
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| ChonkerError::database("create_fts_table", e))?;
        
        info!("Database migrations completed");
        Ok(())
    }
    
    /// Save document with batch optimization for MLX
    pub async fn save_document(
        &self,
        filename: &str,
        file_path: &Path,
        chunks: &[DocumentChunk],
        options: &ProcessingOptions,
        processing_time_ms: u64,
    ) -> ChonkerResult<String> {
        let document_id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        
        info!("Saving document: {} with {} chunks", filename, chunks.len());
        
        // Start transaction for atomic operations
        let mut tx = self.pool.begin().await
            .map_err(|e| ChonkerError::database("begin_transaction", e))?;
        
        // Get file metadata
        let file_metadata = std::fs::metadata(file_path)
            .map_err(|e| ChonkerError::file_io(file_path.to_string_lossy().to_string(), e))?;
        let file_size = file_metadata.len() as i64;
        
        // Create file hash for deduplication
        let file_hash = format!("{:x}", md5::compute(
            std::fs::read(file_path)
                .map_err(|e| ChonkerError::file_io(file_path.to_string_lossy().to_string(), e))?
        ));
        
        // Insert document record
        let options_json = serde_json::to_string(options)
            .map_err(|e| ChonkerError::SystemResource {
                resource: "serialize_options".to_string(),
                source: Box::new(e),
            })?;
            
        sqlx::query(
            r#"
            INSERT INTO documents (
                id, filename, file_path, file_size, file_hash, 
                processing_options, created_at, updated_at, status, 
                total_chunks, processing_time_ms
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'completed', ?, ?)
            "#
        )
        .bind(&document_id)
        .bind(filename)
        .bind(file_path.to_string_lossy().as_ref())
        .bind(file_size)
        .bind(&file_hash)
        .bind(&options_json)
        .bind(&now)
        .bind(&now)
        .bind(chunks.len() as i64)
        .bind(processing_time_ms as i64)
        .execute(&mut *tx)
        .await
        .map_err(|e| ChonkerError::database("insert_document", e))?;
        
        // Batch insert chunks for MLX efficiency
        for batch in chunks.chunks(self.config.batch_size) {
            self.insert_chunk_batch(&mut tx, &document_id, batch).await?;
        }
        
        // Commit transaction
        tx.commit().await
            .map_err(|e| ChonkerError::database("commit_transaction", e))?;
        
        info!("✅ Document saved successfully: {}", document_id);
        Ok(document_id)
    }
    
    /// Insert batch of chunks efficiently
    async fn insert_chunk_batch(
        &self,
        tx: &mut sqlx::Transaction<'_, Sqlite>,
        document_id: &str,
        chunks: &[DocumentChunk],
    ) -> ChonkerResult<()> {
        for chunk in chunks {
            let chunk_id = Uuid::new_v4().to_string();
            let now = Utc::now().to_rfc3339();
            let content_hash = format!("{:x}", md5::compute(&chunk.content));
            let element_types_json = serde_json::to_string(&chunk.element_types)
                .map_err(|e| ChonkerError::SystemResource {
                    resource: "serialize_element_types".to_string(),
                    source: Box::new(e),
                })?;
            
            sqlx::query(
                r#"
                INSERT INTO chunks (
                    id, document_id, chunk_index, content, content_hash,
                    char_count, page_range, element_types, spatial_bounds, created_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#
            )
            .bind(&chunk_id)
            .bind(document_id)
            .bind(chunk.id as i64)
            .bind(&chunk.content)
            .bind(&content_hash)
            .bind(chunk.char_count as i64)
            .bind(&chunk.page_range)
            .bind(&element_types_json)
            .bind(&chunk.spatial_bounds)
            .bind(&now)
            .execute(&mut **tx)
            .await
            .map_err(|e| ChonkerError::database("insert_chunk", e))?;
            
            // Insert into FTS for search
            sqlx::query("INSERT INTO chunks_fts (content, document_id, chunk_id) VALUES (?, ?, ?)")
                .bind(&chunk.content)
                .bind(document_id)
                .bind(&chunk_id)
                .execute(&mut **tx)
                .await
                .map_err(|e| ChonkerError::database("insert_chunk_fts", e))?;
        }
        
        debug!("Inserted batch of {} chunks", chunks.len());
        Ok(())
    }
    
    /// Load documents with pagination for MLX batch processing
    pub async fn load_documents(&self, limit: Option<i64>, offset: Option<i64>) -> ChonkerResult<Vec<DocumentRecord>> {
        let limit = limit.unwrap_or(50);
        let offset = offset.unwrap_or(0);
        
        let rows = sqlx::query(
            r#"
            SELECT id, filename, file_size, total_chunks, processing_time_ms, created_at, status
            FROM documents 
            ORDER BY created_at DESC 
            LIMIT ? OFFSET ?
            "#
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ChonkerError::database("load_documents", e))?;
        
        let mut documents = Vec::new();
        for row in rows {
            documents.push(DocumentRecord {
                id: row.try_get("id").unwrap_or_default(),
                filename: row.try_get("filename").unwrap_or_default(),
                file_size: row.try_get::<i64, _>("file_size").unwrap_or(0) as u64,
                total_chunks: row.try_get::<Option<i64>, _>("total_chunks").unwrap_or(Some(0)).unwrap_or(0) as usize,
                processing_time_ms: row.try_get::<Option<i64>, _>("processing_time_ms").unwrap_or(Some(0)).unwrap_or(0) as u64,
                created_at: row.try_get("created_at").unwrap_or_default(),
                status: row.try_get("status").unwrap_or_default(),
            });
        }
        
        Ok(documents)
    }
    
    /// Load chunks for a document with MLX batch optimization
    pub async fn load_chunks(&self, document_id: &str) -> ChonkerResult<Vec<DocumentChunk>> {
        let rows = sqlx::query(
            r#"
            SELECT chunk_index, content, char_count, page_range, element_types, spatial_bounds
            FROM chunks 
            WHERE document_id = ?
            ORDER BY chunk_index
            "#
        )
        .bind(document_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ChonkerError::database("load_chunks", e))?;
        
        let mut chunks = Vec::with_capacity(rows.len());
        for row in rows {
            let element_types_str: String = row.try_get("element_types").unwrap_or_default();
            let element_types: Vec<String> = serde_json::from_str(&element_types_str)
                .map_err(|e| ChonkerError::SystemResource {
                    resource: "deserialize_element_types".to_string(),
                    source: Box::new(e),
                })?;
                
            chunks.push(DocumentChunk {
                id: row.try_get::<i64, _>("chunk_index").unwrap_or(0),
                content: row.try_get("content").unwrap_or_default(),
                page_range: row.try_get::<Option<String>, _>("page_range").unwrap_or(None).unwrap_or_default(),
                element_types,
                spatial_bounds: row.try_get::<Option<String>, _>("spatial_bounds").unwrap_or(None),
                char_count: row.try_get::<i64, _>("char_count").unwrap_or(0),
            });
        }
        
        Ok(chunks)
    }
    
    /// Full-text search with relevance scoring
    pub async fn search_chunks(
        &self, 
        query: &str, 
        limit: Option<i64>
    ) -> ChonkerResult<Vec<ChunkSearchResult>> {
        let limit = limit.unwrap_or(20);
        
        let rows = sqlx::query(
            r#"
            SELECT 
                c.id, c.content, c.char_count, c.page_range, c.element_types,
                d.filename, d.id as document_id
            FROM chunks_fts fts
            JOIN chunks c ON c.rowid = fts.rowid
            JOIN documents d ON d.id = c.document_id
            WHERE chunks_fts MATCH ?
            ORDER BY fts.rank
            LIMIT ?
            "#
        )
        .bind(query)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ChonkerError::database("search_chunks", e))?;
        
        let mut results = Vec::new();
        for row in rows {
            let element_types_str: String = row.try_get("element_types").unwrap_or_default();
            let element_types: Vec<String> = serde_json::from_str(&element_types_str).unwrap_or_default();
            
            results.push(ChunkSearchResult {
                chunk_id: row.try_get("id").unwrap_or_default(),
                document_id: row.try_get("document_id").unwrap_or_default(),
                document_filename: row.try_get("filename").unwrap_or_default(),
                content: row.try_get("content").unwrap_or_default(),
                char_count: row.try_get::<i64, _>("char_count").unwrap_or(0) as usize,
                page_range: row.try_get::<Option<String>, _>("page_range").unwrap_or(None).unwrap_or_default(),
                element_types,
                relevance_score: 1.0, // Simplified for now
            });
        }
        
        Ok(results)
    }
    
    /// Get database statistics for monitoring
    pub async fn get_stats(&self) -> ChonkerResult<DatabaseStats> {
        let doc_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM documents")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| ChonkerError::database("get_doc_count", e))?;
            
        let chunk_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM chunks")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| ChonkerError::database("get_chunk_count", e))?;
            
        let total_size: Option<i64> = sqlx::query_scalar("SELECT SUM(file_size) FROM documents")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| ChonkerError::database("get_total_size", e))?;
            
        let last_updated: Option<String> = sqlx::query_scalar("SELECT updated_at FROM documents ORDER BY updated_at DESC LIMIT 1")
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| ChonkerError::database("get_last_updated", e))?;
            
        Ok(DatabaseStats {
            document_count: doc_count as usize,
            chunk_count: chunk_count as usize,
            database_size_mb: (total_size.unwrap_or(0) as f64) / (1024.0 * 1024.0),
            last_updated: last_updated.unwrap_or("Never".to_string()),
        })
    }
    
    /// CLI-specific methods
    pub async fn store_document(&mut self, filename: String, content: String, metadata: serde_json::Value) -> Result<String> {
        let document_id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        
        sqlx::query(
            r#"
            INSERT INTO documents (
                id, filename, file_path, file_size, file_hash, 
                processing_options, created_at, updated_at, status
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'completed')
            "#
        )
        .bind(&document_id)
        .bind(&filename)
        .bind(&filename) // Use filename as path for now
        .bind(content.len() as i64)
        .bind(format!("{:x}", md5::compute(&content)))
        .bind(metadata.to_string())
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;
        
        Ok(document_id)
    }
    
    pub async fn get_all_documents(&self) -> Result<Vec<Document>> {
        let rows = sqlx::query(
            "SELECT id, filename, created_at FROM documents ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;
        
        Ok(rows.into_iter().map(|row| Document {
            id: row.get("id"),
            filename: row.get("filename"),
            created_at: row.get("created_at"),
        }).collect())
    }
    
    pub async fn get_document_chunks(&self, doc_id: &str) -> Result<Vec<DocumentChunk>> {
        let rows = sqlx::query(
            "SELECT chunk_index, content, page_range, element_types, spatial_bounds, char_count FROM chunks WHERE document_id = ? ORDER BY chunk_index"
        )
        .bind(doc_id)
        .fetch_all(&self.pool)
        .await?;
        
        let mut chunks = Vec::new();
        for row in rows {
            let element_types_str: String = row.get("element_types");
            let element_types: Vec<String> = serde_json::from_str(&element_types_str).unwrap_or_default();
            
            chunks.push(DocumentChunk {
                id: row.get::<i64, _>("chunk_index"),
                content: row.get("content"),
                page_range: row.get::<Option<String>, _>("page_range").unwrap_or_default(),
                element_types,
                spatial_bounds: row.get::<Option<String>, _>("spatial_bounds"),
                char_count: row.get("char_count"),
            });
        }
        
        Ok(chunks)
    }
    
    pub async fn get_recent_documents(&self, limit: usize) -> Result<Vec<Document>> {
        let rows = sqlx::query(
            "SELECT id, filename, created_at FROM documents ORDER BY created_at DESC LIMIT ?"
        )
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;
        
        Ok(rows.into_iter().map(|row| Document {
            id: row.get("id"),
            filename: row.get("filename"),
            created_at: row.get("created_at"),
        }).collect())
    }
    
    pub async fn delete_document(&self, doc_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM documents WHERE id = ?")
            .bind(doc_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
    
    /// Get FTS manager for advanced search capabilities
    pub fn get_fts_manager(&self) -> FTSManager {
        FTSManager::new(self.pool.clone())
    }
    
    /// Initialize FTS5 search indexes
    pub async fn init_search(&self) -> ChonkerResult<()> {
        let fts = self.get_fts_manager();
        fts.init_schema().await.map_err(|e| ChonkerError::SystemResource {
            resource: "fts_init".to_string(),
            source: Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())),
        })?;
        
        fts.rebuild_indexes().await.map_err(|e| ChonkerError::SystemResource {
            resource: "fts_rebuild".to_string(),
            source: Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())),
        })?;
        
        info!("🔍 Search indexes initialized");
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct DocumentRecord {
    pub id: String,
    pub filename: String,
    pub file_size: u64,
    pub total_chunks: usize,
    pub processing_time_ms: u64,
    pub created_at: String,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct ChunkSearchResult {
    pub chunk_id: String,
    pub document_id: String,
    pub document_filename: String,
    pub content: String,
    pub char_count: usize,
    pub page_range: String,
    pub element_types: Vec<String>,
    pub relevance_score: f64,
}

#[derive(Debug, Default)]
pub struct DatabaseStats {
    pub document_count: usize,
    pub chunk_count: usize,
    pub database_size_mb: f64,
    pub last_updated: String,
}

// CLI-specific document types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub filename: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentChunk {
    pub id: i64,
    pub content: String,
    pub page_range: String,
    pub element_types: Vec<String>,
    pub spatial_bounds: Option<String>,
    pub char_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingOptions {
    pub tool: String,
    pub extract_tables: bool,
    pub extract_formulas: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[tokio::test]
    async fn test_database_creation() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let config = DatabaseConfig::default();
        
        let _db = ChonkerDatabase::new(db_path.to_str().unwrap(), config).await.unwrap();
    }
}
