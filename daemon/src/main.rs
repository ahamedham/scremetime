mod collectors;
mod db;

use collectors::system::SystemCollector;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// Short intervals while we are building and watching it work. We will move
// these into a proper config file once more collectors exist and revisit
// what interval is actually efficient for each one.
const BATTERY_POLL_INTERVAL: Duration = Duration::from_secs(5);
const SYSTEM_POLL_INTERVAL: Duration = Duration::from_secs(3);

fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is set before 1970")
        .as_secs() as i64
}

#[tokio::main]
async fn main() {
    let conn = db::open().expect("failed to open database");
    println!(
        "scremetime daemon started. database at {:?}",
        db::data_dir().join("data.db")
    );

    let mut battery_interval = tokio::time::interval(BATTERY_POLL_INTERVAL);
    let mut system_interval = tokio::time::interval(SYSTEM_POLL_INTERVAL);
    let mut system_collector = SystemCollector::new();

    loop {
        tokio::select! {
            _ = battery_interval.tick() => {
                match collectors::battery::read_battery() {
                    Some(reading) => {
                        let sample = db::BatterySample {
                            timestamp: unix_now(),
                            percentage: reading.percentage,
                            state: reading.state.clone(),
                            power_draw_watts: reading.power_draw_watts,
                            time_to_empty_seconds: reading.time_to_empty_seconds,
                            time_to_full_seconds: reading.time_to_full_seconds,
                        };
                        match db::insert_battery_sample(&conn, &sample) {
                            Ok(()) => println!(
                                "battery: {}% ({}), {}",
                                reading.percentage,
                                reading.state,
                                reading
                                    .power_draw_watts
                                    .map(|w| format!("{:.2}W", w))
                                    .unwrap_or_else(|| "power draw unknown".to_string())
                            ),
                            Err(e) => eprintln!("failed to write battery sample: {e}"),
                        }
                    }
                    None => eprintln!("could not read battery info from this system"),
                }
            }
            _ = system_interval.tick() => {
                let reading = system_collector.read();
                let sample = db::SystemSample {
                    timestamp: unix_now(),
                    cpu_percent: reading.cpu_percent as f64,
                    mem_used_bytes: reading.mem_used_bytes as i64,
                    mem_total_bytes: reading.mem_total_bytes as i64,
                };
                match db::insert_system_sample(&conn, &sample) {
                    Ok(()) => println!(
                        "system: cpu {:.1}%, mem {:.1}/{:.1} GB",
                        reading.cpu_percent,
                        reading.mem_used_bytes as f64 / 1_073_741_824.0,
                        reading.mem_total_bytes as f64 / 1_073_741_824.0
                    ),
                    Err(e) => eprintln!("failed to write system sample: {e}"),
                }
            }
            _ = tokio::signal::ctrl_c() => {
                println!("shutting down");
                break;
            }
        }
    }
}
