use sqlx::{SqlitePool, Row};
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use tracing::{info, debug};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub document_id: i64,
    pub filename: String,
    pub snippet: String,
    pub rank: f64,
    pub chunk_id: Option<i64>,
    pub page_range: Option<String>,
    pub relevance_score: f64,
}

#[derive(Debug, Clone)]
pub enum SearchQuery {
    Simple(String),
    Phrase(String),           // "exact phrase"
    Prefix(String),          // term*
    Boolean(BooleanQuery),   // term1 AND term2
    Near(String, String, u32), // NEAR(term1, term2, 5)
}

#[derive(Debug, Clone)]
pub struct BooleanQuery {
    pub left: String,
    pub operator: BooleanOperator,
    pub right: String,
}

#[derive(Debug, Clone)]
pub enum BooleanOperator {
    And,
    Or,
    Not,
}

#[derive(Debug, Clone)]
pub struct SearchOptions {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub highlight: bool,
    pub snippet_length: u32,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            limit: Some(50),
            offset: None,
            highlight: true,
            snippet_length: 30,
        }
    }
}

pub struct FTSManager {
    pool: SqlitePool,
}

impl FTSManager {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
    
    /// Initialize FTS5 virtual tables and triggers
    pub async fn init_schema(&self) -> Result<()> {
        info!("🔍 Initializing FTS5 search schema");
        
        // Create FTS5 virtual table for documents
        sqlx::query(
            r#"
            CREATE VIRTUAL TABLE IF NOT EXISTS documents_fts USING fts5(
                content,
                filename,
                content='',
                tokenize='unicode61 remove_diacritics 2'
            )
            "#
        )
        .execute(&self.pool)
        .await?;
        
        // Create FTS5 virtual table for chunks with more granular search
        sqlx::query(
            r#"
            CREATE VIRTUAL TABLE IF NOT EXISTS chunks_fts USING fts5(
                chunk_text,
                element_types,
                page_range,
                content='',
                tokenize='porter'
            )
            "#
        )
        .execute(&self.pool)
        .await?;
        
        // Create FTS5 for entities (when we add entity extraction)
        sqlx::query(
            r#"
            CREATE VIRTUAL TABLE IF NOT EXISTS entities_fts USING fts5(
                entity_type,
                entity_value,
                context,
                content='',
                tokenize='unicode61'
            )
            "#
        )
        .execute(&self.pool)
        .await?;
        
        info!("✅ FTS5 schema initialized");
        Ok(())
    }
    
    /// Rebuild FTS5 indexes from existing data
    pub async fn rebuild_indexes(&self) -> Result<()> {
        info!("🔄 Rebuilding FTS5 indexes");
        
        // Clear existing FTS data
        sqlx::query("DELETE FROM documents_fts").execute(&self.pool).await?;
        sqlx::query("DELETE FROM chunks_fts").execute(&self.pool).await?;
        
        // Populate documents FTS from existing documents and chunks
        sqlx::query(
            r#"
            INSERT INTO documents_fts(rowid, content, filename)
            SELECT 
                d.id,
                GROUP_CONCAT(dc.content, ' ') as content,
                d.filename
            FROM documents d
            LEFT JOIN document_chunks dc ON d.id = dc.document_id
            GROUP BY d.id, d.filename
            "#
        )
        .execute(&self.pool)
        .await?;
        
        // Populate chunks FTS
        sqlx::query(
            r#"
            INSERT INTO chunks_fts(rowid, chunk_text, element_types, page_range)
            SELECT 
                id,
                content,
                COALESCE(element_types, ''),
                page_range
            FROM document_chunks
            "#
        )
        .execute(&self.pool)
        .await?;
        
        // Optimize FTS indexes
        sqlx::query("INSERT INTO documents_fts(documents_fts) VALUES('optimize')").execute(&self.pool).await?;
        sqlx::query("INSERT INTO chunks_fts(chunks_fts) VALUES('optimize')").execute(&self.pool).await?;
        
        info!("✅ FTS5 indexes rebuilt and optimized");
        Ok(())
    }
    
    /// Search documents with FTS5
    pub async fn search_documents(&self, query: SearchQuery, options: SearchOptions) -> Result<Vec<SearchResult>> {
        let fts_query = query.to_fts5_query();
        debug!("🔍 Executing FTS5 query: {}", fts_query);
        
        let limit_clause = options.limit.map(|l| format!("LIMIT {}", l)).unwrap_or_default();
        let offset_clause = options.offset.map(|o| format!("OFFSET {}", o)).unwrap_or_default();
        
        let sql = if options.highlight {
            format!(
                r#"
                SELECT 
                    d.id as document_id,
                    d.filename,
                    snippet(documents_fts, 0, '<mark>', '</mark>', '...', {}) as snippet,
                    bm25(documents_fts) as rank,
                    NULL as chunk_id,
                    NULL as page_range,
                    (1.0 / (1.0 + abs(bm25(documents_fts)))) as relevance_score
                FROM documents_fts 
                JOIN documents d ON documents_fts.rowid = d.id
                WHERE documents_fts MATCH ?
                ORDER BY rank
                {} {}
                "#, 
                options.snippet_length, limit_clause, offset_clause
            )
        } else {
            format!(
                r#"
                SELECT 
                    d.id as document_id,
                    d.filename,
                    substr(d.filename, 1, 100) as snippet,
                    bm25(documents_fts) as rank,
                    NULL as chunk_id,
                    NULL as page_range,
                    (1.0 / (1.0 + abs(bm25(documents_fts)))) as relevance_score
                FROM documents_fts 
                JOIN documents d ON documents_fts.rowid = d.id
                WHERE documents_fts MATCH ?
                ORDER BY rank
                {} {}
                "#,
                limit_clause, offset_clause
            )
        };
        
        let rows = sqlx::query(&sql)
            .bind(&fts_query)
            .fetch_all(&self.pool)
            .await?;
        
        let mut results = Vec::new();
        for row in rows {
            results.push(SearchResult {
                document_id: row.get("document_id"),
                filename: row.get("filename"),
                snippet: row.get("snippet"),
                rank: row.get("rank"),
                chunk_id: row.get("chunk_id"),
                page_range: row.get("page_range"),
                relevance_score: row.get("relevance_score"),
            });
        }
        
        info!("🔍 Found {} search results", results.len());
        Ok(results)
    }
    
    /// Search chunks with more granular results
    pub async fn search_chunks(&self, query: SearchQuery, options: SearchOptions) -> Result<Vec<SearchResult>> {
        let fts_query = query.to_fts5_query();
        debug!("🔍 Executing chunk FTS5 query: {}", fts_query);
        
        let limit_clause = options.limit.map(|l| format!("LIMIT {}", l)).unwrap_or_default();
        let offset_clause = options.offset.map(|o| format!("OFFSET {}", o)).unwrap_or_default();
        
        let sql = format!(
            r#"
            SELECT 
                d.id as document_id,
                d.filename,
                snippet(chunks_fts, 0, '<mark>', '</mark>', '...', {}) as snippet,
                bm25(chunks_fts) as rank,
                dc.id as chunk_id,
                dc.page_range,
                (1.0 / (1.0 + abs(bm25(chunks_fts)))) as relevance_score
            FROM chunks_fts 
            JOIN document_chunks dc ON chunks_fts.rowid = dc.id
            JOIN documents d ON dc.document_id = d.id
            WHERE chunks_fts MATCH ?
            ORDER BY rank
            {} {}
            "#,
            options.snippet_length, limit_clause, offset_clause
        );
        
        let rows = sqlx::query(&sql)
            .bind(&fts_query)
            .fetch_all(&self.pool)
            .await?;
        
        let mut results = Vec::new();
        for row in rows {
            results.push(SearchResult {
                document_id: row.get("document_id"),
                filename: row.get("filename"),
                snippet: row.get("snippet"),
                rank: row.get("rank"),
                chunk_id: row.get("chunk_id"),
                page_range: row.get("page_range"),
                relevance_score: row.get("relevance_score"),
            });
        }
        
        info!("🔍 Found {} chunk search results", results.len());
        Ok(results)
    }
    
    /// Combined search across documents and chunks
    pub async fn search_all(&self, query: SearchQuery, options: SearchOptions) -> Result<Vec<SearchResult>> {
        let mut all_results = Vec::new();
        
        // Search documents
        let doc_results = self.search_documents(query.clone(), options.clone()).await?;
        all_results.extend(doc_results);
        
        // Search chunks
        let chunk_results = self.search_chunks(query, options).await?;
        all_results.extend(chunk_results);
        
        // Sort by relevance score
        all_results.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());
        
        Ok(all_results)
    }
    
    /// Update FTS index when a document is added
    pub async fn index_document(&self, document_id: i64, filename: &str, content: &str) -> Result<()> {
        sqlx::query(
            "INSERT OR REPLACE INTO documents_fts(rowid, content, filename) VALUES (?, ?, ?)"
        )
        .bind(document_id)
        .bind(content)
        .bind(filename)
        .execute(&self.pool)
        .await?;
        
        debug!("📝 Indexed document {} in FTS", document_id);
        Ok(())
    }
    
    /// Update FTS index when a chunk is added
    pub async fn index_chunk(&self, chunk_id: i64, chunk_text: &str, element_types: &str, page_range: &str) -> Result<()> {
        sqlx::query(
            "INSERT OR REPLACE INTO chunks_fts(rowid, chunk_text, element_types, page_range) VALUES (?, ?, ?, ?)"
        )
        .bind(chunk_id)
        .bind(chunk_text)
        .bind(element_types)
        .bind(page_range)
        .execute(&self.pool)
        .await?;
        
        debug!("📝 Indexed chunk {} in FTS", chunk_id);
        Ok(())
    }
    
    /// Get search suggestions based on partial input
    pub async fn get_suggestions(&self, partial: &str, limit: u32) -> Result<Vec<String>> {
        let query = format!("{}*", partial);
        
        let rows = sqlx::query(
            r#"
            SELECT DISTINCT highlight(documents_fts, 0, '', '') as term
            FROM documents_fts 
            WHERE documents_fts MATCH ?
            LIMIT ?
            "#
        )
        .bind(&query)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;
        
        let suggestions: Vec<String> = rows.into_iter()
            .map(|row| row.get::<String, _>("term"))
            .collect();
        
        Ok(suggestions)
    }
}

impl SearchQuery {
    pub fn to_fts5_query(&self) -> String {
        match self {
            SearchQuery::Simple(term) => term.clone(),
            SearchQuery::Phrase(phrase) => format!("\"{}\"", phrase),
            SearchQuery::Prefix(prefix) => format!("{}*", prefix),
            SearchQuery::Boolean(boolean) => boolean.to_string(),
            SearchQuery::Near(term1, term2, distance) => {
                format!("NEAR(\"{}\", \"{}\", {})", term1, term2, distance)
            }
        }
    }
    
    pub fn parse(input: &str) -> Self {
        // Simple parser - could be expanded
        if input.starts_with('"') && input.ends_with('"') {
            SearchQuery::Phrase(input[1..input.len()-1].to_string())
        } else if input.ends_with('*') {
            SearchQuery::Prefix(input[..input.len()-1].to_string())
        } else if input.contains(" AND ") {
            let parts: Vec<&str> = input.splitn(2, " AND ").collect();
            SearchQuery::Boolean(BooleanQuery {
                left: parts[0].trim().to_string(),
                operator: BooleanOperator::And,
                right: parts[1].trim().to_string(),
            })
        } else if input.contains(" OR ") {
            let parts: Vec<&str> = input.splitn(2, " OR ").collect();
            SearchQuery::Boolean(BooleanQuery {
                left: parts[0].trim().to_string(),
                operator: BooleanOperator::Or,
                right: parts[1].trim().to_string(),
            })
        } else {
            SearchQuery::Simple(input.to_string())
        }
    }
}

impl std::fmt::Display for BooleanQuery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.left, self.operator, self.right)
    }
}

impl std::fmt::Display for BooleanOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BooleanOperator::And => write!(f, "AND"),
            BooleanOperator::Or => write!(f, "OR"),
            BooleanOperator::Not => write!(f, "NOT"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_query_parsing() {
        assert!(matches!(SearchQuery::parse("hello"), SearchQuery::Simple(_)));
        assert!(matches!(SearchQuery::parse("\"hello world\""), SearchQuery::Phrase(_)));
        assert!(matches!(SearchQuery::parse("hello*"), SearchQuery::Prefix(_)));
        assert!(matches!(SearchQuery::parse("hello AND world"), SearchQuery::Boolean(_)));
    }
    
    #[test]
    fn test_fts5_query_generation() {
        let phrase = SearchQuery::Phrase("hello world".to_string());
        assert_eq!(phrase.to_fts5_query(), "\"hello world\"");
        
        let prefix = SearchQuery::Prefix("test".to_string());
        assert_eq!(prefix.to_fts5_query(), "test*");
    }
}
