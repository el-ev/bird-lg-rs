use chrono::{DateTime, Duration, Utc};

pub fn humanize_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KiB", "MiB", "GiB", "TiB"];

    if bytes == 0 {
        return "0 B".to_string();
    }

    let k = 1024f64;
    let i = (bytes as f64).log(k).floor() as usize;
    let i = i.min(UNITS.len() - 1);

    let value = bytes as f64 / k.powi(i as i32);

    if i == 0 {
        format!("{} {}", bytes, UNITS[i])
    } else {
        format!("{:.2} {}", value, UNITS[i])
    }
}

pub fn humanize_duration(timestamp: i64) -> Option<String> {
    if timestamp == 0 {
        return None;
    }

    let handshake_time = DateTime::from_timestamp(timestamp, 0)?;
    let now = Utc::now();
    let elapsed = now.signed_duration_since(handshake_time);

    if elapsed < Duration::zero() {
        return Some("in the future".to_string());
    }

    let seconds = elapsed.num_seconds();

    if seconds < 60 {
        return Some(format!(
            "{} second{} ago",
            seconds,
            if seconds == 1 { "" } else { "s" }
        ));
    }

    let minutes = seconds / 60;
    if minutes < 60 {
        return Some(format!(
            "{} minute{} ago",
            minutes,
            if minutes == 1 { "" } else { "s" }
        ));
    }

    let hours = minutes / 60;
    if hours < 24 {
        return Some(format!(
            "{} hour{} ago",
            hours,
            if hours == 1 { "" } else { "s" }
        ));
    }

    let days = hours / 24;
    Some(format!(
        "{} day{} ago",
        days,
        if days == 1 { "" } else { "s" }
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_humanize_bytes() {
        assert_eq!(humanize_bytes(0), "0 B");
        assert_eq!(humanize_bytes(100), "100 B");
        assert_eq!(humanize_bytes(1024), "1.00 KiB");
        assert_eq!(humanize_bytes(1536), "1.50 KiB");
        assert_eq!(humanize_bytes(1048576), "1.00 MiB");
        assert_eq!(humanize_bytes(1073741824), "1.00 GiB");
    }
}
