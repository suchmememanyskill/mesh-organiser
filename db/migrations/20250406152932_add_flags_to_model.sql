-- Add migration script here

ALTER TABLE models
ADD model_flags INTEGER NOT NULL DEFAULT 0;