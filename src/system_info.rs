//! Live system information helpers (no GTK dependency).

use gtk4::glib::DateTime;

pub fn read_uptime() -> Option<String> {
    let content = std::fs::read_to_string("/proc/uptime").ok()?;
    let seconds: f64 = content.split_whitespace().next()?.parse().ok()?;
    let total_seconds = seconds as u64;
    let days = total_seconds / 86400;
    let hours = (total_seconds % 86400) / 3600;
    let minutes = (total_seconds % 3600) / 60;
    if days > 0 {
        Some(format!("{}d {}h {}m", days, hours, minutes))
    } else if hours > 0 {
        Some(format!("{}h {}m", hours, minutes))
    } else {
        Some(format!("{}m", minutes))
    }
}

pub fn read_memory_usage() -> Option<String> {
    let content = std::fs::read_to_string("/proc/meminfo").ok()?;
    let mut total: Option<u64> = None;
    let mut available: Option<u64> = None;
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("MemTotal:") {
            total = rest
                .split_whitespace()
                .next()
                .and_then(|s| s.parse::<u64>().ok());
        } else if let Some(rest) = line.strip_prefix("MemAvailable:") {
            available = rest
                .split_whitespace()
                .next()
                .and_then(|s| s.parse::<u64>().ok());
        }
    }
    match (total, available) {
        (Some(t), Some(a)) if t > 0 => {
            let used = t.saturating_sub(a);
            let percent = (used as f64 / t as f64) * 100.0;
            Some(format!("{:.0}%", percent))
        }
        _ => None,
    }
}

pub fn read_loadavg() -> Option<String> {
    let content = std::fs::read_to_string("/proc/loadavg").ok()?;
    let first: f64 = content.split_whitespace().next()?.parse().ok()?;
    Some(format!("{:.2}", first))
}

pub fn current_time() -> (String, Option<String>) {
    let now = match DateTime::now_local() {
        Ok(dt) => dt,
        Err(_) => return (String::new(), None),
    };
    let time = now.format("%H:%M").unwrap_or_default().to_string();
    let tz = now.timezone_abbreviation();
    let tz = if tz.is_empty() {
        None
    } else {
        Some(tz.to_string())
    };
    (time, tz)
}
