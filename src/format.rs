use std::time::Duration;

pub fn format_time(ms: u32) -> String {
    if ms == 0 {
        "<1ms".to_string()
    } else {
        format!("{}ms", ms)
    }
}

pub fn format_duration(duration: Duration) -> String {
    let total_ms = duration.as_millis() as u64;
    let hours = total_ms / 3_600_000;
    let minutes = (total_ms % 3_600_000) / 60_000;
    let seconds = (total_ms % 60_000) / 1_000;
    let millis = total_ms % 1_000;

    if hours > 0 {
        format!("{}h {}m {}.{:03}s", hours, minutes, seconds, millis)
    } else if minutes > 0 {
        format!("{}m {}.{:03}s", minutes, seconds, millis)
    } else {
        format!("{}.{:03}s", seconds, millis)
    }
}
