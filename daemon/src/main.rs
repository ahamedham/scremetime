use futures_util::StreamExt;
use scremetime_daemon::collectors::app_focus::app_focus_proxy;
use scremetime_daemon::collectors::disk_io::DiskIoCollector;
use scremetime_daemon::collectors::idle::{
    login_manager_proxy, screen_saver_proxy, IdleEvent, IdleWatcher,
};
use scremetime_daemon::collectors::system::SystemCollector;
use scremetime_daemon::collectors::battery;
use scremetime_daemon::db;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use zbus::Connection;

// Short intervals while we are building and watching it work. We will move
// these into a proper config file once more collectors exist and revisit
// what interval is actually efficient for each one.
const BATTERY_POLL_INTERVAL: Duration = Duration::from_secs(5);
const SYSTEM_POLL_INTERVAL: Duration = Duration::from_secs(3);
const DISK_IO_POLL_INTERVAL: Duration = Duration::from_secs(5);
const IDLE_POLL_INTERVAL: Duration = Duration::from_secs(5);

fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is set before 1970")
        .as_secs() as i64
}

fn record_idle_event(conn: &rusqlite::Connection, event: IdleEvent) {
    let event_str = event.as_db_str();
    match db::insert_idle_event(conn, unix_now(), event_str) {
        Ok(()) => println!("idle event: {event_str}"),
        Err(e) => eprintln!("failed to write idle event: {e}"),
    }
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
    let mut disk_io_interval = tokio::time::interval(DISK_IO_POLL_INTERVAL);
    let mut disk_io_collector = DiskIoCollector::new();

    // Idle time and screen lock come from GNOME's session bus services.
    // Suspend/resume comes from systemd-logind on the system bus, which is
    // deliberately not GNOME specific, unlike the rest of this section.
    let session_conn = Connection::session()
        .await
        .expect("failed to connect to the D-Bus session bus");
    let system_conn = Connection::system()
        .await
        .expect("failed to connect to the D-Bus system bus");

    let mut idle_watcher = IdleWatcher::new(&session_conn)
        .await
        .expect("failed to connect to the idle monitor");
    let mut idle_interval = tokio::time::interval(IDLE_POLL_INTERVAL);

    let screen_saver = screen_saver_proxy(&session_conn)
        .await
        .expect("failed to connect to the screen saver");
    let mut lock_events = screen_saver
        .receive_active_changed()
        .await
        .expect("failed to subscribe to lock events");

    let login_manager = login_manager_proxy(&system_conn)
        .await
        .expect("failed to connect to logind");
    let mut sleep_events = login_manager
        .receive_prepare_for_sleep()
        .await
        .expect("failed to subscribe to suspend events");

    // App focus tracking depends on the scremetime GNOME Shell extension
    // being installed and enabled. Creating the proxy and subscribing to
    // its signal both succeed regardless, since D-Bus lets you register
    // interest in a name before anything owns it; the extension's actual
    // absence only shows up once we try to call it, below. This means the
    // subscription stays valid even if the extension gets enabled later
    // while this same daemon process keeps running.
    let app_focus = app_focus_proxy(&session_conn).await.ok();
    let mut app_focus_events = match &app_focus {
        Some(proxy) => proxy.receive_focused_app_changed().await.ok(),
        None => None,
    };

    // Tracks the currently open app_sessions row, if any, so it can be
    // closed off the moment focus moves to a different app or the daemon
    // shuts down.
    let mut current_app_session: Option<i64> = None;
    match &app_focus {
        Some(proxy) => match proxy.get_focused_app().await {
            Ok(app_id) if !app_id.is_empty() => {
                current_app_session = db::start_app_session(&conn, &app_id, unix_now()).ok();
                println!("app focus: {app_id}");
            }
            Ok(_) => {}
            Err(_) => eprintln!(
                "app focus tracking unavailable: the scremetime GNOME Shell extension does not \
                 appear to be enabled yet. Other collectors will keep running; app tracking \
                 will pick up automatically once the extension is enabled."
            ),
        },
        None => eprintln!("app focus tracking unavailable: could not create the D-Bus proxy"),
    }

    loop {
        tokio::select! {
            _ = battery_interval.tick() => {
                match battery::read_battery() {
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
            _ = disk_io_interval.tick() => {
                let readings = disk_io_collector.read();
                let mut written = 0;
                for reading in &readings {
                    let sample = db::DiskIoSample {
                        timestamp: unix_now(),
                        pid: reading.pid,
                        process_name: reading.process_name.clone(),
                        read_bytes: reading.read_bytes as i64,
                        write_bytes: reading.write_bytes as i64,
                    };
                    match db::insert_disk_io_sample(&conn, &sample) {
                        Ok(()) => written += 1,
                        Err(e) => eprintln!("failed to write disk io sample: {e}"),
                    }
                }
                if written > 0 {
                    println!("disk io: {written} process(es) with activity");
                }
            }
            _ = idle_interval.tick() => {
                if let Some(event) = idle_watcher.poll().await {
                    record_idle_event(&conn, event);
                }
            }
            Some(signal) = lock_events.next() => {
                if let Ok(args) = signal.args() {
                    let event = if *args.new_value() { IdleEvent::Locked } else { IdleEvent::Unlocked };
                    record_idle_event(&conn, event);
                }
            }
            Some(signal) = sleep_events.next() => {
                if let Ok(args) = signal.args() {
                    let event = if *args.start() { IdleEvent::Suspended } else { IdleEvent::Resumed };
                    record_idle_event(&conn, event);
                }
            }
            // When app_focus_events is None (extension not enabled), this
            // awaits a future that never resolves, so the branch simply
            // never fires rather than needing a separate code path.
            maybe_signal = async {
                match app_focus_events.as_mut() {
                    Some(stream) => stream.next().await,
                    None => std::future::pending().await,
                }
            } => {
                if let Some(signal) = maybe_signal {
                    if let Ok(args) = signal.args() {
                        let app_id = args.app_id().to_string();
                        let now = unix_now();
                        if let Some(session_id) = current_app_session.take() {
                            if let Err(e) = db::end_app_session(&conn, session_id, now) {
                                eprintln!("failed to close app session: {e}");
                            }
                        }
                        if !app_id.is_empty() {
                            match db::start_app_session(&conn, &app_id, now) {
                                Ok(id) => {
                                    current_app_session = Some(id);
                                    println!("app focus: {app_id}");
                                }
                                Err(e) => eprintln!("failed to start app session: {e}"),
                            }
                        }
                    }
                }
            }
            _ = tokio::signal::ctrl_c() => {
                if let Some(session_id) = current_app_session.take() {
                    if let Err(e) = db::end_app_session(&conn, session_id, unix_now()) {
                        eprintln!("failed to close app session: {e}");
                    }
                }
                println!("shutting down");
                break;
            }
        }
    }
}
