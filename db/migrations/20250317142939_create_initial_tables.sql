-- Add migration script here

CREATE TABLE models_group (
    group_id INTEGER PRIMARY KEY,
    group_name TEXT NOT NULL,
    group_created TEXT NOT NULL
);

CREATE TABLE models (
    model_id INTEGER PRIMARY KEY,
    model_name TEXT NOT NULL,
    model_sha256 TEXT NOT NULL,
    model_filetype TEXT NOT NULL,
    model_url TEXT NULL,
    model_desc TEXT NULL,
    model_added TEXT NOT NULL,
    model_size INTEGER NOT NULL,
    model_group_id INTEGER NULL,
    FOREIGN KEY(model_group_id) REFERENCES models_group(group_id) ON DELETE SET NULL
);

CREATE TABLE labels (
    label_id INTEGER PRIMARY KEY,
    label_name TEXT NOT NULL,
    label_color INTEGER NOT NULL
);

CREATE TABLE models_labels (
    id INTEGER PRIMARY KEY,
    label_id INTEGER NOT NULL,
    model_id INTEGER NOT NULL,
    FOREIGN KEY (label_id) REFERENCES labels(label_id) ON DELETE CASCADE,
    FOREIGN KEY (model_id) REFERENCES models(model_id) ON DELETE CASCADE
);