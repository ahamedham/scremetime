use std::fs;
use std::path::{Path, PathBuf};

pub struct BatteryReading {
    pub percentage: i64,
    pub state: String,
    pub power_draw_watts: Option<f64>,
    pub time_to_empty_seconds: Option<i64>,
    pub time_to_full_seconds: Option<i64>,
}

/// Finds the battery under /sys/class/power_supply/. Linux names it BAT0
/// on most laptops but not all, so we look for the device whose "type"
/// file says "Battery" rather than hardcoding a name. Directory iteration
/// order here is kernel enumeration order, not alphabetical, so this
/// cannot assume the real battery is found first.
///
/// A "type" of "Battery" is not sufficient on its own: some USB charging
/// controllers (observed here with an Apple MFi fast-charge controller,
/// created when a phone is connected for charging) register a
/// power_supply node that also reports type "Battery" but exposes no
/// "capacity" file, since it tracks charge protocol state rather than an
/// actual battery. Requiring "capacity" to be present filters those out.
fn find_battery() -> Option<PathBuf> {
    let base = Path::new("/sys/class/power_supply");
    for entry in fs::read_dir(base).ok()?.flatten() {
        let path = entry.path();
        if read_string(&path, "type").as_deref() == Some("Battery")
            && path.join("capacity").is_file()
        {
            return Some(path);
        }
    }
    None
}

fn read_u64(dir: &Path, file: &str) -> Option<u64> {
    fs::read_to_string(dir.join(file)).ok()?.trim().parse().ok()
}

fn read_string(dir: &Path, file: &str) -> Option<String> {
    fs::read_to_string(dir.join(file))
        .ok()
        .map(|s| s.trim().to_string())
}

pub fn read_battery() -> Option<BatteryReading> {
    let bat = find_battery()?;

    let percentage = read_u64(&bat, "capacity")? as i64;
    let state = read_string(&bat, "status").unwrap_or_else(|| "Unknown".to_string());

    // Battery hardware reports itself in one of two styles. "Energy" style
    // (energy_now/energy_full/power_now, micro-watt-hours and micro-watts)
    // is more common on modern hardware. "Charge" style (charge_now/
    // charge_full/current_now, micro-amp-hours and micro-amps) is the
    // fallback. We keep "now", "full", and "rate" as a matched trio from
    // whichever style is present, because now/rate gives hours directly
    // only when they come from the same style - mixing them would need
    // the battery's voltage and introduce error.
    let energy_trio = (
        read_u64(&bat, "energy_now"),
        read_u64(&bat, "energy_full"),
        read_u64(&bat, "power_now"),
    );
    let charge_trio = (
        read_u64(&bat, "charge_now"),
        read_u64(&bat, "charge_full"),
        read_u64(&bat, "current_now"),
    );

    let (now, full, rate, power_draw_watts) = match energy_trio {
        (Some(n), Some(f), Some(r)) => (Some(n), Some(f), Some(r), Some(r as f64 / 1_000_000.0)),
        _ => match charge_trio {
            (Some(n), Some(f), Some(r)) => {
                let watts = read_u64(&bat, "voltage_now")
                    .map(|v| (r as f64 * v as f64) / 1_000_000_000_000.0);
                (Some(n), Some(f), Some(r), watts)
            }
            _ => (None, None, None, None),
        },
    };

    let mut time_to_empty_seconds = None;
    let mut time_to_full_seconds = None;

    if let (Some(now), Some(full), Some(rate)) = (now, full, rate) {
        if rate > 0 {
            let hours_remaining = now as f64 / rate as f64;
            let hours_to_full = full.saturating_sub(now) as f64 / rate as f64;
            match state.as_str() {
                "Discharging" => time_to_empty_seconds = Some((hours_remaining * 3600.0) as i64),
                "Charging" => time_to_full_seconds = Some((hours_to_full * 3600.0) as i64),
                _ => {}
            }
        }
    }

    Some(BatteryReading {
        percentage,
        state,
        power_draw_watts,
        time_to_empty_seconds,
        time_to_full_seconds,
    })
}
