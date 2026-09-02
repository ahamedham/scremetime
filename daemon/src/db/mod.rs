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

// Read side, used by the CLI to inspect what the daemon has collected.

/// Total seconds spent in each app, most used first. Only counts sessions
/// that have actually closed (duration_seconds is set); a session still
/// open when this runs is not yet included in its app's total.
pub fn app_usage_totals(conn: &Connection, since: Option<i64>) -> rusqlite::Result<Vec<(String, i64)>> {
    let mut stmt = conn.prepare(
        "SELECT app_name, SUM(duration_seconds) AS total
         FROM app_sessions
         WHERE duration_seconds IS NOT NULL AND start_time >= ?1
         GROUP BY app_name
         ORDER BY total DESC",
    )?;
    let rows = stmt
        .query_map(params![since.unwrap_or(0)], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

/// Total seconds spent using apps, one row per calendar day (local
/// timezone), for the last `days` days including today. Used for the day
/// by day bar chart in the desktop app. Days with no usage are not
/// included; the caller fills gaps if it needs a fixed number of bars.
pub fn daily_usage_totals(conn: &Connection, days: u32) -> rusqlite::Result<Vec<(String, i64)>> {
    let mut stmt = conn.prepare(
        "SELECT strftime('%Y-%m-%d', start_time, 'unixepoch', 'localtime') AS day,
                SUM(duration_seconds) AS total
         FROM app_sessions
         WHERE duration_seconds IS NOT NULL
           AND start_time >= strftime('%s', 'now', 'localtime', '-' || ?1 || ' days', 'start of day', 'utc')
         GROUP BY day
         ORDER BY day ASC",
    )?;
    let rows = stmt
        .query_map(params![days], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

pub struct BatteryRow {
    pub timestamp: i64,
    pub percentage: i64,
    pub state: String,
    pub power_draw_watts: Option<f64>,
}

pub fn recent_battery_samples(conn: &Connection, limit: u32) -> rusqlite::Result<Vec<BatteryRow>> {
    let mut stmt = conn.prepare(
        "SELECT timestamp, percentage, state, power_draw_watts
         FROM battery_samples ORDER BY timestamp DESC LIMIT ?1",
    )?;
    let rows = stmt
        .query_map(params![limit], |row| {
            Ok(BatteryRow {
                timestamp: row.get(0)?,
                percentage: row.get(1)?,
                state: row.get(2)?,
                power_draw_watts: row.get(3)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

pub struct IdleEventRow {
    pub timestamp: i64,
    pub event_type: String,
}

pub fn recent_idle_events(conn: &Connection, limit: u32) -> rusqlite::Result<Vec<IdleEventRow>> {
    let mut stmt = conn.prepare(
        "SELECT timestamp, event_type FROM idle_events ORDER BY timestamp DESC LIMIT ?1",
    )?;
    let rows = stmt
        .query_map(params![limit], |row| {
            Ok(IdleEventRow {
                timestamp: row.get(0)?,
                event_type: row.get(1)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}
