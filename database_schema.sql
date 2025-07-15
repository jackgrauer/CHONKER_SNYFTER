-- CHONKER Database Schema
-- Extensible document extraction and storage system

CREATE TABLE documents (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    filename TEXT NOT NULL,
    file_path TEXT NOT NULL,
    file_hash TEXT UNIQUE NOT NULL,
    file_size INTEGER,
    processed_date TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    processing_version TEXT DEFAULT '1.0',
    metadata JSON, -- Store PDF metadata as JSON
    status TEXT DEFAULT 'pending' CHECK (status IN ('pending', 'processing', 'completed', 'failed', 'reviewed'))
);

CREATE TABLE extracted_content (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    document_id INTEGER NOT NULL,
    content_type TEXT NOT NULL CHECK (content_type IN ('html', 'markdown', 'plain_text')),
    content TEXT NOT NULL,
    extraction_method TEXT DEFAULT 'docling',
    quality_score REAL, -- 0.0 to 1.0 confidence score
    created_date TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (document_id) REFERENCES documents(id) ON DELETE CASCADE
);

CREATE TABLE document_tables (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    document_id INTEGER NOT NULL,
    table_index INTEGER NOT NULL, -- Order within document
    table_data JSON NOT NULL, -- Store table as JSON array
    column_headers JSON, -- Store headers separately for querying
    extraction_confidence REAL,
    page_number INTEGER,
    bbox JSON, -- Bounding box coordinates [x, y, width, height]
    created_date TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (document_id) REFERENCES documents(id) ON DELETE CASCADE
);

CREATE TABLE processing_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    document_id INTEGER NOT NULL,
    step TEXT NOT NULL, -- 'preprocessing', 'extraction', 'postprocessing', 'qc'
    status TEXT NOT NULL CHECK (status IN ('started', 'completed', 'failed', 'skipped')),
    message TEXT,
    details JSON, -- Store any additional processing details
    processing_time_ms INTEGER,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (document_id) REFERENCES documents(id) ON DELETE CASCADE
);

CREATE TABLE qc_reviews (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    document_id INTEGER NOT NULL,
    reviewer TEXT, -- User/system identifier
    review_date TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    overall_quality TEXT CHECK (overall_quality IN ('excellent', 'good', 'fair', 'poor', 'failed')),
    notes TEXT,
    corrections JSON, -- Store any manual corrections made
    approved BOOLEAN DEFAULT FALSE,
    FOREIGN KEY (document_id) REFERENCES documents(id) ON DELETE CASCADE
);

CREATE TABLE document_chunks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    document_id INTEGER NOT NULL,
    chunk_index INTEGER NOT NULL, -- Order within document
    chunk_type TEXT NOT NULL, -- 'paragraph', 'heading', 'table', 'list', 'image'
    content TEXT NOT NULL,
    page_number INTEGER,
    bbox JSON, -- Bounding box for bidirectional selection
    parent_chunk_id INTEGER, -- For hierarchical content
    extraction_confidence REAL,
    created_date TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (document_id) REFERENCES documents(id) ON DELETE CASCADE,
    FOREIGN KEY (parent_chunk_id) REFERENCES document_chunks(id) ON DELETE SET NULL
);

-- Indexes for performance
CREATE INDEX idx_documents_hash ON documents(file_hash);
CREATE INDEX idx_documents_status ON documents(status);
CREATE INDEX idx_documents_date ON documents(processed_date);
CREATE INDEX idx_content_document ON extracted_content(document_id);
CREATE INDEX idx_tables_document ON document_tables(document_id);
CREATE INDEX idx_chunks_document ON document_chunks(document_id);
CREATE INDEX idx_chunks_page ON document_chunks(page_number);
CREATE INDEX idx_logs_document ON processing_logs(document_id);
CREATE INDEX idx_qc_document ON qc_reviews(document_id);

-- Views for common queries
CREATE VIEW document_summary AS
SELECT 
    d.id,
    d.filename,
    d.processed_date,
    d.status,
    COUNT(DISTINCT c.id) as chunk_count,
    COUNT(DISTINCT t.id) as table_count,
    AVG(c.extraction_confidence) as avg_confidence,
    qc.overall_quality,
    qc.approved
FROM documents d
LEFT JOIN document_chunks c ON d.id = c.document_id
LEFT JOIN document_tables t ON d.id = t.document_id
LEFT JOIN qc_reviews qc ON d.id = qc.document_id
GROUP BY d.id, qc.overall_quality, qc.approved;

CREATE VIEW recent_documents AS
SELECT *
FROM document_summary
ORDER BY processed_date DESC
LIMIT 50;