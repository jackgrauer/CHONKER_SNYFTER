CREATE TABLE IF NOT EXISTS annotations (
    id INTEGER PRIMARY KEY,
    document_id TEXT NOT NULL,
    start_pos INTEGER,
    end_pos INTEGER,
    highlighted_text TEXT,
    code_id INTEGER,
    memo TEXT,
    created_date TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (document_id) REFERENCES documents(id),
    FOREIGN KEY (code_id) REFERENCES codes(id)
);

CREATE INDEX idx_annotations_doc ON annotations(document_id);
