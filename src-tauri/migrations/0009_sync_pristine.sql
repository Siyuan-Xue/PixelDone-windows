CREATE TABLE IF NOT EXISTS sync_pristine_records (
    owner_user_id TEXT NOT NULL,
    record_type TEXT NOT NULL CHECK (record_type IN ('checklist', 'item', 'settings')),
    local_id TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    remote_version INTEGER,
    updated_at_millis INTEGER NOT NULL,
    PRIMARY KEY (owner_user_id, record_type, local_id)
);

CREATE TABLE IF NOT EXISTS sync_pristine_state (
    owner_user_id TEXT PRIMARY KEY NOT NULL,
    initialized_at_millis INTEGER NOT NULL
);
