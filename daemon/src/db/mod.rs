use rusqlite::{params, Connection};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

const SCHEMA: &str = include_str!("schema.sql");

/// Where the database file lives: ~/.local/share/scremetime/data.db on
/// almost every Linux setup (follows the XDG base directory convention
/// that most Linux apps use for user data).
pub fn data_dir() -> PathBuf {
    dirs::data_local_dir()
        .expect("could not determine the local data directory")
        .join("scremetime")
}

pub fn open() -> rusqlite::Result<Connection> {
    let dir = data_dir();
    fs::create_dir_all(&dir).expect("failed to create data directory");
    restrict_to_owner(&dir);

    let db_path = dir.join("data.db");
    let conn = Connection::open(&db_path)?;

    // WAL mode lets something reading the database (the CLI now, a future
    // GUI or web dashboard) do so at the same time the daemon is writing,
    // without either one blocking the other.
    conn.pragma_update(None, "journal_mode", "WAL")?;

    conn.execute_batch(SCHEMA)?;
    restrict_to_owner(&db_path);

    Ok(conn)
}

/// Restrict a path to the owning user only: 0700 for directories (need the
/// execute bit to list/enter them), 0600 for files. This is the baseline
/// protection for locally stored data discussed with the user: other
/// accounts on the same machine cannot read this database.
fn restrict_to_owner(path: &Path) {
    let mode = if path.is_dir() { 0o700 } else { 0o600 };
    if let Ok(metadata) = fs::metadata(path) {
        let mut perms = metadata.permissions();
        perms.set_mode(mode);
        let _ = fs::set_permissions(path, perms);
    }
}

pub struct BatterySample {
    pub timestamp: i64,
    pub percentage: i64,
    pub state: String,
    pub power_draw_watts: Option<f64>,
    pub time_to_empty_seconds: Option<i64>,
    pub time_to_full_seconds: Option<i64>,
}

pub fn insert_battery_sample(conn: &Connection, sample: &BatterySample) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO battery_samples
            (timestamp, percentage, state, power_draw_watts, time_to_empty_seconds, time_to_full_seconds)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            sample.timestamp,
            sample.percentage,
            sample.state,
            sample.power_draw_watts,
            sample.time_to_empty_seconds,
            sample.time_to_full_seconds,
        ],
    )?;
    Ok(())
}

pub struct SystemSample {
    pub timestamp: i64,
    pub cpu_percent: f64,
    pub mem_used_bytes: i64,
    pub mem_total_bytes: i64,
}

pub fn insert_system_sample(conn: &Connection, sample: &SystemSample) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO system_samples (timestamp, cpu_percent, mem_used_bytes, mem_total_bytes)
         VALUES (?1, ?2, ?3, ?4)",
        params![
            sample.timestamp,
            sample.cpu_percent,
            sample.mem_used_bytes,
            sample.mem_total_bytes,
        ],
    )?;
    Ok(())
}

/// Opens a new app session and returns its row id, so the caller can
/// close it off later with end_app_session once the app loses focus.
pub fn start_app_session(conn: &Connection, app_name: &str, start_time: i64) -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT INTO app_sessions (app_name, start_time) VALUES (?1, ?2)",
        params![app_name, start_time],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn end_app_session(conn: &Connection, session_id: i64, end_time: i64) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE app_sessions SET end_time = ?1, duration_seconds = ?1 - start_time WHERE id = ?2",
        params![end_time, session_id],
    )?;
    Ok(())
}

pub fn insert_idle_event(conn: &Connection, timestamp: i64, event_type: &str) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO idle_events (timestamp, event_type) VALUES (?1, ?2)",
        params![timestamp, event_type],
    )?;
    Ok(())
}

pub struct DiskIoSample {
    pub timestamp: i64,
    pub pid: i32,
    pub process_name: String,
    pub read_bytes: i64,
    pub write_bytes: i64,
}

pub fn insert_disk_io_sample(conn: &Connection, sample: &DiskIoSample) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO disk_io_samples (timestamp, pid, process_name, read_bytes, write_bytes)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            sample.timestamp,
            sample.pid,
            sample.process_name,
            sample.read_bytes,
            sample.write_bytes,
        ],
    )?;
    Ok(())
}
