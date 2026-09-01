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

-- CPU and memory: polled at an interval
CREATE TABLE IF NOT EXISTS system_samples (
    id INTEGER PRIMARY KEY,
    timestamp INTEGER NOT NULL,
    cpu_percent REAL NOT NULL,
    mem_used_bytes INTEGER NOT NULL,
    mem_total_bytes INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_system_time ON system_samples(timestamp);

-- Idle time and power state transitions: one row per event, not polled
CREATE TABLE IF NOT EXISTS idle_events (
    id INTEGER PRIMARY KEY,
    timestamp INTEGER NOT NULL,
    event_type TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_idle_time ON idle_events(timestamp);

-- Disk I/O per process: polled at an interval
CREATE TABLE IF NOT EXISTS disk_io_samples (
    id INTEGER PRIMARY KEY,
    timestamp INTEGER NOT NULL,
    pid INTEGER NOT NULL,
    process_name TEXT NOT NULL,
    read_bytes INTEGER NOT NULL,
    write_bytes INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_disk_time ON disk_io_samples(timestamp);
