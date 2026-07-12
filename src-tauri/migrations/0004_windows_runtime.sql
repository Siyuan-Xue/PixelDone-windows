ALTER TABLE local_settings ADD COLUMN autostart_enabled INTEGER NOT NULL DEFAULT 1 CHECK (autostart_enabled IN (0, 1));
ALTER TABLE local_settings ADD COLUMN automatic_update_check_enabled INTEGER NOT NULL DEFAULT 1 CHECK (automatic_update_check_enabled IN (0, 1));
ALTER TABLE local_settings ADD COLUMN autostart_initialized INTEGER NOT NULL DEFAULT 0 CHECK (autostart_initialized IN (0, 1));
ALTER TABLE local_settings ADD COLUMN last_update_check_at_millis INTEGER;
ALTER TABLE local_settings ADD COLUMN next_update_check_at_millis INTEGER;

CREATE TABLE IF NOT EXISTS reminder_delivery_state (
    todo_id TEXT PRIMARY KEY NOT NULL,
    scheduled_at_millis INTEGER,
    snoozed_until_millis INTEGER,
    last_fired_at_millis INTEGER,
    updated_at_millis INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_reminder_delivery_scheduled
    ON reminder_delivery_state(scheduled_at_millis);
