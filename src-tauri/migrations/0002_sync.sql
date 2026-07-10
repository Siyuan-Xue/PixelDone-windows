CREATE TABLE IF NOT EXISTS sync_cursors (
    owner_user_id TEXT PRIMARY KEY NOT NULL,
    remote_version INTEGER NOT NULL,
    schema_version INTEGER NOT NULL,
    updated_at_millis INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS sync_mutations (
    owner_user_id TEXT NOT NULL,
    mutation_uuid TEXT NOT NULL,
    base_version INTEGER,
    payload_json TEXT NOT NULL,
    created_at_millis INTEGER NOT NULL,
    attempts INTEGER NOT NULL DEFAULT 0,
    last_error TEXT,
    PRIMARY KEY (owner_user_id, mutation_uuid)
);

CREATE TABLE IF NOT EXISTS sync_conflicts (
    owner_user_id TEXT NOT NULL,
    record_type TEXT NOT NULL CHECK (record_type IN ('checklist', 'item', 'attachment')),
    local_id TEXT NOT NULL,
    local_payload_json TEXT NOT NULL,
    remote_payload_json TEXT NOT NULL,
    fields_json TEXT NOT NULL,
    remote_version INTEGER,
    created_at_millis INTEGER NOT NULL,
    PRIMARY KEY (owner_user_id, record_type, local_id)
);

CREATE TABLE IF NOT EXISTS checklist_tombstones (
    checklist_id TEXT PRIMARY KEY NOT NULL,
    owner_user_id TEXT,
    deleted_at_millis INTEGER NOT NULL,
    remote_version INTEGER
);
