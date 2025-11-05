-- Add migration script here

CREATE TABLE resources (
    resource_id INTEGER PRIMARY KEY,
    resource_name TEXT NOT NULL,
    resource_created TEXT NOT NULL,
    resource_flags INTEGER NOT NULL DEFAULT 0
);

ALTER TABLE models_group
ADD group_resource_id INTEGER NULL REFERENCES resources(resource_id) ON DELETE SET NULL;