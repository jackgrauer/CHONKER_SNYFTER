CREATE TABLE IF NOT EXISTS codes (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    color TEXT DEFAULT '#1ABC9C',
    parent_id INTEGER,
    created_date TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (parent_id) REFERENCES codes(id)
);

CREATE INDEX idx_codes_parent ON codes(parent_id);
