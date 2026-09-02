use chrono::{DateTime, Local, TimeZone};
use clap::{Parser, Subcommand};
use scremetime_daemon::db;
use scremetime_daemon::time_util::Period;

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
        /// Reporting period: today, week, month, or all
        #[arg(long, default_value = "all")]
        period: String,
    },
    /// Recent battery samples
    Battery {
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

fn main() {
    let cli = Cli::parse();
    let conn = db::open().expect("failed to open database");

    match cli.command {
        Command::Apps { period } => {
            let period = Period::parse(&period)
                .unwrap_or_else(|| panic!("unknown period '{period}', expected today, week, month, or all"));
            let totals = db::app_usage_totals(&conn, period.since()).expect("query failed");
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
        Command::Idle { limit } => {
            let rows = db::recent_idle_events(&conn, limit).expect("query failed");
            for row in rows {
                println!("{}  {}", format_timestamp(row.timestamp), row.event_type);
            }
        }
    }
}
