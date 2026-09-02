use scremetime_daemon::db;
use scremetime_daemon::time_util::Period;
use serde::Serialize;
use std::sync::Mutex;
use tauri::{Manager, State};

/// Holds the one database connection this app uses. rusqlite's Connection
/// is not safe to share across threads without a lock, and Tauri may run
/// commands from multiple invocations concurrently, so every command
/// below goes through this mutex.
struct AppState {
    conn: Mutex<rusqlite::Connection>,
}

#[derive(Serialize)]
struct AppUsage {
    app_name: String,
    total_seconds: i64,
}

#[derive(Serialize)]
struct BatterySample {
    timestamp: i64,
    percentage: i64,
    state: String,
    power_draw_watts: Option<f64>,
}

#[derive(Serialize)]
struct IdleEvent {
    timestamp: i64,
    event_type: String,
}

#[tauri::command]
fn get_app_usage(state: State<AppState>, period: String) -> Result<Vec<AppUsage>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let period = Period::parse(&period).ok_or_else(|| format!("unknown period '{period}'"))?;
    db::app_usage_totals(&conn, period.since())
        .map(|rows| {
            rows.into_iter()
                .map(|(app_name, total_seconds)| AppUsage {
                    app_name,
                    total_seconds,
                })
                .collect()
        })
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_battery_samples(state: State<AppState>, limit: u32) -> Result<Vec<BatterySample>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::recent_battery_samples(&conn, limit)
        .map(|rows| {
            rows.into_iter()
                .map(|r| BatterySample {
                    timestamp: r.timestamp,
                    percentage: r.percentage,
                    state: r.state,
                    power_draw_watts: r.power_draw_watts,
                })
                .collect()
        })
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_idle_events(state: State<AppState>, limit: u32) -> Result<Vec<IdleEvent>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::recent_idle_events(&conn, limit)
        .map(|rows| {
            rows.into_iter()
                .map(|r| IdleEvent {
                    timestamp: r.timestamp,
                    event_type: r.event_type,
                })
                .collect()
        })
        .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let conn = db::open().expect("failed to open the scremetime database");
            app.manage(AppState {
                conn: Mutex::new(conn),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_app_usage,
            get_battery_samples,
            get_idle_events,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
