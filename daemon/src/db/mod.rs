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
