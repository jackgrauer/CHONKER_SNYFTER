use crate::chonker_types::*;
use anyhow::Result;
use sqlx::{Pool, Row, Sqlite, SqlitePool};
use uuid::Uuid;

pub struct Database {
    pool: Pool<Sqlite>,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = SqlitePool::connect(database_url).await?;
        Ok(Database { pool })
    }

    pub async fn get_documents(&self) -> Result<Vec<Document>> {
        let rows = sqlx::query(
            "SELECT id, filename, file_path, file_hash, content_type, file_size, created_at, updated_at FROM documents ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        let mut documents = Vec::new();
        for row in rows {
            documents.push(Document {
                id: uuid::Uuid::parse_str(&row.get::<String, _>("id")).unwrap(),
                filename: row.get("filename"),
                file_path: row.get("file_path"),
                file_hash: row.get("file_hash"),
                content_type: row.get("content_type"),
                file_size: row.get("file_size"),
                created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at")).unwrap().with_timezone(&chrono::Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("updated_at")).unwrap().with_timezone(&chrono::Utc),
            });
        }

        Ok(documents)
    }

    pub async fn get_chunks_by_document(&self, document_id: Uuid) -> Result<Vec<DocumentChunk>> {
        let rows = sqlx::query(
            "SELECT id, document_id, chunk_index, content, element_types, created_at FROM chunks WHERE document_id = ? ORDER BY chunk_index"
        )
        .bind(document_id.to_string())
        .fetch_all(&self.pool)
        .await?;

        let mut result = Vec::new();
        for row in rows {
            let element_types: String = row.get("element_types");
            let content_type = if element_types.contains("table") {
                "table".to_string()
            } else {
                "text".to_string()
            };

            result.push(DocumentChunk {
                id: uuid::Uuid::parse_str(&row.get::<String, _>("id")).unwrap(),
                document_id: uuid::Uuid::parse_str(&row.get::<String, _>("document_id")).unwrap(),
                chunk_index: row.get("chunk_index"),
                content: row.get("content"),
                content_type,
                metadata: Some(element_types),
                table_data: None, // We'll need to parse this from content for now
                created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at")).unwrap().with_timezone(&chrono::Utc),
            });
        }

        Ok(result)
    }

    pub async fn get_table_chunks(&self) -> Result<Vec<DocumentChunk>> {
        let rows = sqlx::query(
            "SELECT id, document_id, chunk_index, content, element_types, created_at FROM chunks WHERE element_types LIKE '%table%' ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        let mut result = Vec::new();
        for row in rows {
            let element_types: String = row.get("element_types");
            
            result.push(DocumentChunk {
                id: uuid::Uuid::parse_str(&row.get::<String, _>("id")).unwrap(),
                document_id: uuid::Uuid::parse_str(&row.get::<String, _>("document_id")).unwrap(),
                chunk_index: row.get("chunk_index"),
                content: row.get("content"),
                content_type: "table".to_string(),
                metadata: Some(element_types),
                table_data: None, // Will parse from content
                created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at")).unwrap().with_timezone(&chrono::Utc),
            });
        }

        Ok(result)
    }
    
    pub async fn save_document(&self, document: &Document, chunks: &[DocumentChunk]) -> Result<String> {
        let mut transaction = self.pool.begin().await?;
        
        // Insert document
        sqlx::query(
            "INSERT INTO documents (id, filename, file_path, file_hash, content_type, file_size, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(document.id.to_string())
        .bind(&document.filename)
        .bind(&document.file_path)
        .bind(&document.file_hash)
        .bind(&document.content_type)
        .bind(document.file_size)
        .bind(document.created_at.to_rfc3339())
        .bind(document.updated_at.to_rfc3339())
        .execute(&mut *transaction)
        .await?;
        
        // Insert chunks
        for chunk in chunks {
            let table_data_json = chunk.table_data.as_ref()
                .map(|td| serde_json::to_string(td).unwrap_or_default());
                
            sqlx::query(
                "INSERT INTO chunks (id, document_id, chunk_index, content, content_type, metadata, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)"
            )
            .bind(chunk.id.to_string())
            .bind(chunk.document_id.to_string())
            .bind(chunk.chunk_index)
            .bind(&chunk.content)
            .bind(&chunk.content_type)
            .bind(table_data_json)
            .bind(chunk.created_at.to_rfc3339())
            .execute(&mut *transaction)
            .await?;
        }
        
        transaction.commit().await?;
        Ok(document.id.to_string())
    }
}
