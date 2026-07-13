ALTER TABLE todo_attachments RENAME TO todo_attachments_legacy;

CREATE TABLE todo_attachments (
    todo_id TEXT PRIMARY KEY NOT NULL,
    local_file_name TEXT,
    attachment_id TEXT,
    object_path TEXT,
    content_sha256 TEXT,
    content_type TEXT,
    byte_size INTEGER,
    updated_at_millis INTEGER NOT NULL,
    deleted_at_millis INTEGER,
    remote_version INTEGER,
    sync_state TEXT NOT NULL DEFAULT 'SYNCED',
    last_error TEXT
);

INSERT INTO todo_attachments (
    todo_id,
    local_file_name,
    content_sha256,
    updated_at_millis,
    sync_state
)
SELECT
    todo_id,
    local_file_name,
    content_sha256,
    updated_at_millis,
    CASE WHEN local_file_name IS NULL THEN 'SYNCED' ELSE 'PENDING_UPLOAD' END
FROM todo_attachments_legacy;

INSERT OR IGNORE INTO todo_attachments (
    todo_id,
    local_file_name,
    updated_at_millis,
    sync_state
)
SELECT
    id,
    image_file_name,
    updated_at_millis,
    'PENDING_UPLOAD'
FROM todo_items
WHERE image_file_name IS NOT NULL;

DROP TABLE todo_attachments_legacy;

CREATE INDEX todo_attachments_sync_state_idx
    ON todo_attachments(sync_state, updated_at_millis);

CREATE TABLE todo_image_local_cleanup_queue (
    todo_id TEXT NOT NULL,
    object_path TEXT NOT NULL,
    queued_at_millis INTEGER NOT NULL,
    PRIMARY KEY (todo_id, object_path)
);
