CREATE TABLE local_settings_wide_sidebar (
    id TEXT PRIMARY KEY NOT NULL CHECK (id = 'settings'),
    dark_theme INTEGER NOT NULL DEFAULT 0 CHECK (dark_theme IN (0, 1)),
    dock_plus_placement TEXT NOT NULL CHECK (dock_plus_placement IN ('CENTER', 'LEFT_EDGE', 'RIGHT_EDGE')),
    dock_actions_json TEXT NOT NULL,
    never_show_update_dialog INTEGER NOT NULL DEFAULT 0 CHECK (never_show_update_dialog IN (0, 1)),
    future_sync_enabled INTEGER NOT NULL DEFAULT 0 CHECK (future_sync_enabled IN (0, 1)),
    language_mode TEXT NOT NULL DEFAULT 'system' CHECK (language_mode IN ('system', 'en', 'zh-Hans', 'ar', 'fr', 'ru', 'es')),
    language_remote_version INTEGER,
    autostart_enabled INTEGER NOT NULL DEFAULT 1 CHECK (autostart_enabled IN (0, 1)),
    automatic_update_check_enabled INTEGER NOT NULL DEFAULT 1 CHECK (automatic_update_check_enabled IN (0, 1)),
    autostart_initialized INTEGER NOT NULL DEFAULT 0 CHECK (autostart_initialized IN (0, 1)),
    last_update_check_at_millis INTEGER,
    next_update_check_at_millis INTEGER,
    enhanced_xhigh_alarm_enabled INTEGER NOT NULL DEFAULT 0 CHECK (enhanced_xhigh_alarm_enabled IN (0, 1)),
    sidebar_width_px INTEGER NOT NULL DEFAULT 320 CHECK (sidebar_width_px BETWEEN 200 AND 720)
);

INSERT INTO local_settings_wide_sidebar (
    id,
    dark_theme,
    dock_plus_placement,
    dock_actions_json,
    never_show_update_dialog,
    future_sync_enabled,
    language_mode,
    language_remote_version,
    autostart_enabled,
    automatic_update_check_enabled,
    autostart_initialized,
    last_update_check_at_millis,
    next_update_check_at_millis,
    enhanced_xhigh_alarm_enabled,
    sidebar_width_px
)
SELECT
    id,
    dark_theme,
    dock_plus_placement,
    dock_actions_json,
    never_show_update_dialog,
    future_sync_enabled,
    language_mode,
    language_remote_version,
    autostart_enabled,
    automatic_update_check_enabled,
    autostart_initialized,
    last_update_check_at_millis,
    next_update_check_at_millis,
    enhanced_xhigh_alarm_enabled,
    sidebar_width_px
FROM local_settings;

DROP TABLE local_settings;
ALTER TABLE local_settings_wide_sidebar RENAME TO local_settings;
