-- Add migration script here

CREATE TABLE labels_labels (
    child_label_id INTEGER NOT NULL,
    parent_label_id INTEGER NOT NULL,
    PRIMARY KEY (child_label_id, parent_label_id),
    FOREIGN KEY (child_label_id) REFERENCES labels(label_id) ON DELETE CASCADE,
    FOREIGN KEY (parent_label_id) REFERENCES labels(label_id) ON DELETE CASCADE
);