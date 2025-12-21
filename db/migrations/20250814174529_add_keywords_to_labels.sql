-- Add migration script here

CREATE TABLE label_keywords (
    keyword_id INTEGER PRIMARY KEY,
    keyword_name TEXT NOT NULL UNIQUE,
    keyword_label_id INTEGER NOT NULL,
    FOREIGN KEY (keyword_label_id) REFERENCES labels(label_id) ON DELETE CASCADE
);