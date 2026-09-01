use std::collections::HashMap;
use std::fs;

pub struct DiskIoReading {
    pub pid: i32,
    pub process_name: String,
    pub read_bytes: u64,
    pub write_bytes: u64,
}

struct PrevCounters {
    read_bytes: u64,
    write_bytes: u64,
}

/// Tracks per process disk I/O as deltas between polls rather than the raw
/// cumulative counters the kernel reports. The kernel's read_bytes and
/// write_bytes in /proc/<pid>/io are totals since the process started,
/// which is not directly useful for "how much did this process read or
/// write recently" style analytics, so we keep the previous reading per
/// PID and compute the difference ourselves.
pub struct DiskIoCollector {
    previous: HashMap<i32, PrevCounters>,
}

impl DiskIoCollector {
    pub fn new() -> Self {
        Self {
            previous: HashMap::new(),
        }
    }

    /// Returns only processes with nonzero I/O activity since the last
    /// poll, so idle processes do not fill the database with rows of
    /// zeros every few seconds. A process seen for the first time this
    /// poll establishes a baseline but is not reported, since its
    /// lifetime-so-far totals are not "activity since last poll."
    pub fn read(&mut self) -> Vec<DiskIoReading> {
        let mut readings = Vec::new();
        let mut current = HashMap::new();

        let Ok(entries) = fs::read_dir("/proc") else {
            return readings;
        };

        for entry in entries.flatten() {
            let Some(pid_str) = entry.file_name().to_str().map(str::to_owned) else {
                continue;
            };
            let Ok(pid) = pid_str.parse::<i32>() else {
                continue;
            };
            let Some((read_bytes, write_bytes)) = read_io_counters(pid) else {
                continue;
            };

            if let Some(prev) = self.previous.get(&pid) {
                let read_delta = read_bytes.saturating_sub(prev.read_bytes);
                let write_delta = write_bytes.saturating_sub(prev.write_bytes);
                if read_delta > 0 || write_delta > 0 {
                    if let Some(name) = read_process_name(pid) {
                        readings.push(DiskIoReading {
                            pid,
                            process_name: name,
                            read_bytes: read_delta,
                            write_bytes: write_delta,
                        });
                    }
                }
            }

            current.insert(
                pid,
                PrevCounters {
                    read_bytes,
                    write_bytes,
                },
            );
        }

        // Rebuilding the map from scratch each poll drops PIDs that have
        // exited, so memory does not grow unbounded as processes churn.
        self.previous = current;
        readings
    }
}

fn read_io_counters(pid: i32) -> Option<(u64, u64)> {
    let contents = fs::read_to_string(format!("/proc/{pid}/io")).ok()?;
    let mut read_bytes = None;
    let mut write_bytes = None;
    for line in contents.lines() {
        if let Some(value) = line.strip_prefix("read_bytes:") {
            read_bytes = value.trim().parse().ok();
        } else if let Some(value) = line.strip_prefix("write_bytes:") {
            write_bytes = value.trim().parse().ok();
        }
    }
    Some((read_bytes?, write_bytes?))
}

fn read_process_name(pid: i32) -> Option<String> {
    fs::read_to_string(format!("/proc/{pid}/comm"))
        .ok()
        .map(|s| s.trim().to_string())
}
