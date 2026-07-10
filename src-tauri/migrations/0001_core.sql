PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS app_metadata (
    id TEXT PRIMARY KEY NOT NULL CHECK (id = 'app'),
    selected_checklist_id TEXT NOT NULL,
    revision INTEGER NOT NULL DEFAULT 0,
    sort_mode TEXT NOT NULL CHECK (sort_mode IN ('PRIORITY', 'TIME')),
    hide_completed INTEGER NOT NULL DEFAULT 0 CHECK (hide_completed IN (0, 1)),
    quick_delete INTEGER NOT NULL DEFAULT 0 CHECK (quick_delete IN (0, 1))
);

CREATE TABLE IF NOT EXISTS checklists (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL CHECK (length(trim(name)) > 0),
    sort_index INTEGER NOT NULL,
    created_at_millis INTEGER NOT NULL,
    deleted_at_millis INTEGER
);

CREATE TABLE IF NOT EXISTS todo_items (
    id TEXT PRIMARY KEY NOT NULL,
    checklist_id TEXT NOT NULL REFERENCES checklists(id) ON DELETE CASCADE,
    sort_index INTEGER NOT NULL,
    title TEXT NOT NULL CHECK (length(trim(title)) > 0),
    priority TEXT NOT NULL CHECK (priority IN ('XHIGH', 'HIGH', 'MEDIUM', 'LOW')),
    due_at_millis INTEGER NOT NULL,
    completed INTEGER NOT NULL DEFAULT 0 CHECK (completed IN (0, 1)),
    created_at_millis INTEGER NOT NULL,
    reminder_repeat TEXT NOT NULL CHECK (reminder_repeat IN ('NONE', 'DAILY', 'WEEKLY')),
    image_file_name TEXT,
    trashed_from_checklist_id TEXT,
    trashed_from_checklist_name TEXT,
    trashed_at_millis INTEGER
);

CREATE INDEX IF NOT EXISTS idx_todo_checklist_sort ON todo_items(checklist_id, sort_index);

CREATE TABLE IF NOT EXISTS local_settings (
    id TEXT PRIMARY KEY NOT NULL CHECK (id = 'settings'),
    dark_theme INTEGER NOT NULL DEFAULT 0 CHECK (dark_theme IN (0, 1)),
    dock_plus_placement TEXT NOT NULL CHECK (dock_plus_placement IN ('CENTER', 'LEFT_EDGE', 'RIGHT_EDGE')),
    dock_actions_json TEXT NOT NULL,
    never_show_update_dialog INTEGER NOT NULL DEFAULT 0 CHECK (never_show_update_dialog IN (0, 1)),
    future_sync_enabled INTEGER NOT NULL DEFAULT 0 CHECK (future_sync_enabled IN (0, 1))
);
