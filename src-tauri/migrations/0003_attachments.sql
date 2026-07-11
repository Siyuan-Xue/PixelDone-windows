CREATE TABLE IF NOT EXISTS todo_attachments (
    todo_id TEXT PRIMARY KEY NOT NULL REFERENCES todo_items(id) ON DELETE CASCADE,
    local_file_name TEXT,
    content_sha256 TEXT,
    updated_at_millis INTEGER NOT NULL
);
