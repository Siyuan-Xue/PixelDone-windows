CREATE TABLE IF NOT EXISTS todo_attachments (
    todo_id TEXT PRIMARY KEY NOT NULL REFERENCES todo_items(id) ON DELETE CASCADE,
    local_file_name TEXT,
    remote_object_path TEXT,
    content_sha256 TEXT,
    sync_state TEXT NOT NULL DEFAULT 'LOCAL_ONLY' CHECK (sync_state IN ('LOCAL_ONLY', 'PENDING', 'SYNCED', 'ERROR')),
    updated_at_millis INTEGER NOT NULL
);
