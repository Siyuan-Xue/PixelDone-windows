ALTER TABLE local_settings ADD COLUMN sidebar_width_px INTEGER NOT NULL DEFAULT 256 CHECK (sidebar_width_px BETWEEN 220 AND 420);
