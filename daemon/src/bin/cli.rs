use chrono::{DateTime, Local, TimeZone};
use clap::{Parser, Subcommand};
use scremetime_daemon::db;

#[derive(Parser)]
#[command(name = "scremetime", about = "Inspect data collected by the scremetime daemon")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// App usage totals, most used first
    Apps {
        /// Only include sessions that started today
        #[arg(long)]
        today: bool,
    },
    /// Recent battery samples
    Battery {
        #[arg(long, default_value_t = 20)]
        limit: u32,
    },
    /// Recent CPU and memory samples
    System {
        #[arg(long, default_value_t = 20)]
        limit: u32,
    },
    /// Recent disk I/O activity
    Disk {
        #[arg(long, default_value_t = 20)]
        limit: u32,
    },
    /// Idle, lock, and suspend/resume events
    Idle {
        #[arg(long, default_value_t = 20)]
        limit: u32,
    },
}

fn format_timestamp(ts: i64) -> String {
    Local
        .timestamp_opt(ts, 0)
        .single()
        .map(|dt: DateTime<Local>| dt.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|| format!("timestamp {ts}"))
}

fn format_duration(seconds: i64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;
    if hours > 0 {
        format!("{hours}h {minutes}m")
    } else if minutes > 0 {
        format!("{minutes}m {secs}s")
    } else {
        format!("{secs}s")
    }
}

fn format_bytes(bytes: i64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    format!("{value:.1} {}", UNITS[unit])
}

fn today_start_timestamp() -> i64 {
    Local::now()
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .expect("midnight is a valid time")
        .and_local_timezone(Local)
        .single()
        .expect("today's midnight is unambiguous in the local timezone")
        .timestamp()
}

fn main() {
    let cli = Cli::parse();
    let conn = db::open().expect("failed to open database");

    match cli.command {
        Command::Apps { today } => {
            let since = today.then(today_start_timestamp);
            let totals = db::app_usage_totals(&conn, since).expect("query failed");
            if totals.is_empty() {
                println!("no completed app sessions recorded yet");
            }
            for (app_name, total_seconds) in totals {
                println!("{:<10}  {}", format_duration(total_seconds), app_name);
            }
        }
        Command::Battery { limit } => {
            let rows = db::recent_battery_samples(&conn, limit).expect("query failed");
            for row in rows {
                let watts = row
                    .power_draw_watts
                    .map(|w| format!("{w:.2}W"))
                    .unwrap_or_else(|| "unknown draw".to_string());
                println!(
                    "{}  {:>3}%  {:<12}  {}",
                    format_timestamp(row.timestamp),
                    row.percentage,
                    row.state,
                    watts
                );
            }
        }
        Command::System { limit } => {
            let rows = db::recent_system_samples(&conn, limit).expect("query failed");
            for row in rows {
                println!(
                    "{}  cpu {:>5.1}%  mem {} / {}",
                    format_timestamp(row.timestamp),
                    row.cpu_percent,
                    format_bytes(row.mem_used_bytes),
                    format_bytes(row.mem_total_bytes)
                );
            }
        }
        Command::Disk { limit } => {
            let rows = db::recent_disk_io_samples(&conn, limit).expect("query failed");
            for row in rows {
                println!(
                    "{}  {:<20}  read {}  write {}",
                    format_timestamp(row.timestamp),
                    row.process_name,
                    format_bytes(row.read_bytes),
                    format_bytes(row.write_bytes)
                );
            }
        }
        Command::Idle { limit } => {
            let rows = db::recent_idle_events(&conn, limit).expect("query failed");
            for row in rows {
                println!("{}  {}", format_timestamp(row.timestamp), row.event_type);
            }
        }
    }
}
