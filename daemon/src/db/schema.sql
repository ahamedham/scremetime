-- App focus tracking: one row per app session, closed off when focus changes
CREATE TABLE IF NOT EXISTS app_sessions (
    id INTEGER PRIMARY KEY,
    app_name TEXT NOT NULL,
    start_time INTEGER NOT NULL,
    end_time INTEGER,
    duration_seconds INTEGER
);
CREATE INDEX IF NOT EXISTS idx_app_sessions_start ON app_sessions(start_time);
CREATE INDEX IF NOT EXISTS idx_app_sessions_app ON app_sessions(app_name);

-- Battery: polled at an interval
CREATE TABLE IF NOT EXISTS battery_samples (
    id INTEGER PRIMARY KEY,
    timestamp INTEGER NOT NULL,
    percentage INTEGER NOT NULL,
    state TEXT NOT NULL,
    power_draw_watts REAL,
    time_to_empty_seconds INTEGER,
    time_to_full_seconds INTEGER
);
CREATE INDEX IF NOT EXISTS idx_battery_time ON battery_samples(timestamp);

-- Idle time and power state transitions: one row per event, not polled
CREATE TABLE IF NOT EXISTS idle_events (
    id INTEGER PRIMARY KEY,
    timestamp INTEGER NOT NULL,
    event_type TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_idle_time ON idle_events(timestamp);

-- Explicitly drop tables from a prior version of the schema. This runs
-- every startup alongside the CREATE TABLE statements above; DROP TABLE
-- IF EXISTS is a no-op once the tables are gone, so this is safe to leave
-- in place rather than requiring anyone who already has the old schema to
-- run a separate migration step.
DROP TABLE IF EXISTS system_samples;
DROP TABLE IF EXISTS disk_io_samples;
