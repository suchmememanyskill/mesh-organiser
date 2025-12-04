-- Add migration script here

CREATE TABLE users (
    user_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    user_name TEXT NOT NULL,
    user_email TEXT NOT NULL UNIQUE,
    user_password_hash TEXT NOT NULL,
    user_created_at TEXT NOT NULL,
    user_sync_url TEXT,
    user_sync_token TEXT,
    user_last_sync TEXT,
    user_permissions INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE blobs (
    blob_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    blob_sha256 TEXT NOT NULL UNIQUE,
    blob_filetype TEXT NOT NULL,
    blob_size INTEGER NOT NULL,
    blob_added TEXT NOT NULL
);

UPDATE models SET model_added = strftime('%Y-%m-%dT%H:%M:%SZ', model_added);
UPDATE models_group SET group_created = strftime('%Y-%m-%dT%H:%M:%SZ', group_created);
UPDATE resources SET resource_created = strftime('%Y-%m-%dT%H:%M:%SZ', resource_created);

INSERT INTO users (user_name, user_email, user_password_hash, user_created_at, user_permissions) VALUES ('local', 'local@noemail.com', 'hashed_password_here', strftime('%Y-%m-%dT%H:%M:%SZ', datetime('now')), 3);

ALTER TABLE models add model_unique_global_id TEXT NOT NULL DEFAULT '';
UPDATE models SET model_unique_global_id = LOWER(HEX(RANDOMBLOB(16)));
ALTER TABLE labels add label_unique_global_id TEXT NOT NULL DEFAULT '';
UPDATE labels SET label_unique_global_id = LOWER(HEX(RANDOMBLOB(16)));
ALTER TABLE models_group add group_unique_global_id TEXT NOT NULL DEFAULT '';
UPDATE models_group SET group_unique_global_id = LOWER(HEX(RANDOMBLOB(16)));
ALTER TABLE resources add resource_unique_global_id TEXT NOT NULL DEFAULT '';
UPDATE resources SET resource_unique_global_id = LOWER(HEX(RANDOMBLOB(16)));

ALTER TABLE models ADD model_user_id INTEGER REFERENCES users(user_id) ON DELETE CASCADE;
UPDATE models SET model_user_id = 1 WHERE model_user_id IS NULL;
ALTER TABLE labels ADD label_user_id INTEGER REFERENCES users(user_id) ON DELETE CASCADE;
UPDATE labels SET label_user_id = 1 WHERE label_user_id IS NULL;
ALTER TABLE models_group ADD group_user_id INTEGER REFERENCES users(user_id) ON DELETE CASCADE;
UPDATE models_group SET group_user_id = 1 WHERE group_user_id IS NULL;
ALTER TABLE resources ADD resource_user_id INTEGER REFERENCES users(user_id) ON DELETE CASCADE;
UPDATE resources SET resource_user_id = 1 WHERE resource_user_id IS NULL;

ALTER TABLE models ADD model_last_modified TEXT NOT NULL DEFAULT '2000-01-01T00:00:00Z';
UPDATE models SET model_last_modified = model_added;
ALTER TABLE labels ADD label_last_modified TEXT NOT NULL DEFAULT '2000-01-01T00:00:00Z';
ALTER TABLE models_group ADD group_last_modified TEXT NOT NULL DEFAULT '2000-01-01T00:00:00Z';
UPDATE models_group SET group_last_modified = group_created;
ALTER TABLE resources ADD resource_last_modified TEXT NOT NULL DEFAULT '2000-01-01T00:00:00Z';
UPDATE resources SET resource_last_modified = resource_created;

ALTER TABLE models ADD model_blob_id INTEGER REFERENCES blobs(blob_id) ON DELETE CASCADE;

INSERT INTO blobs (blob_sha256, blob_filetype, blob_size, blob_added)
SELECT model_sha256, model_filetype, model_size, model_added FROM models;

UPDATE models SET model_blob_id = (
    SELECT blob_id FROM blobs WHERE blobs.blob_sha256 = models.model_sha256 AND blobs.blob_filetype = models.model_filetype AND blobs.blob_size = models.model_size LIMIT 1
);

ALTER TABLE models DROP model_sha256;
ALTER TABLE models DROP model_filetype;
ALTER TABLE models DROP model_size;

CREATE INDEX idx_model_user_id ON models(model_user_id);
CREATE INDEX idx_labels_user_id ON labels(label_user_id);
CREATE INDEX idx_models_group_user_id ON models_group(group_user_id);
CREATE INDEX idx_resources_user_id ON resources(resource_user_id);
CREATE INDEX idx_models_labels_model_id ON models_labels(model_id);
CREATE INDEX idx_models_labels_model_id_label_id ON models_labels(model_id, label_id);

-- TODO: This transaction is quite messy. labels/models_group/resources have a new unconstrained (nullable) unique_global_id and user_id column.
-- Consider cleaning this up in a future migration.

CREATE TABLE shares (
    share_id TEXT NOT NULL PRIMARY KEY,
    share_user_id INTEGER NOT NULL REFERENCES users(user_id) ON DELETE CASCADE,
    share_created_at TEXT NOT NULL,
    share_name TEXT NOT NULL
);

CREATE INDEX idx_shares_user_id ON shares(share_user_id);

CREATE TABLE shares_models (
    share_model_id INTEGER NOT NULL PRIMARY KEY,
    share_id INTEGER NOT NULL REFERENCES shares(share_id) ON DELETE CASCADE,
    model_id INTEGER NOT NULL REFERENCES models(model_id) ON DELETE CASCADE
);