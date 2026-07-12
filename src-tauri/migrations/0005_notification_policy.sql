ALTER TABLE local_settings ADD COLUMN enhanced_xhigh_alarm_enabled INTEGER NOT NULL DEFAULT 0 CHECK (enhanced_xhigh_alarm_enabled IN (0, 1));
