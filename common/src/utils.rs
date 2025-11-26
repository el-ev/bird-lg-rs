use std::net::IpAddr;

pub fn filter_protocol_details(raw: &str) -> String {
    const PROTOCOL_HEADERS: [&str; 6] = ["Name", "Proto", "Table", "State", "Since", "Info"];
    raw.lines()
        .filter(|line| PROTOCOL_HEADERS.iter().any(|header| !line.contains(header)))
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn validate_target(target: &str) -> Result<(), String> {
    let target = target.trim();

    if target.is_empty() {
        return Err("Target is required".to_string());
    }

    if target.parse::<IpAddr>().is_ok() {
        return Ok(());
    }

    if target.len() > 253 {
        return Err("Hostname is too long".to_string());
    }

    if target
        .bytes()
        .any(|b| !(b.is_ascii_alphanumeric() || b == b'-' || b == b'.'))
    {
        return Err("Hostname may only contain letters, digits, '-' or '.'".to_string());
    }

    if target.split('.').any(|label| {
        label.is_empty() || label.len() > 63 || label.starts_with('-') || label.ends_with('-')
    }) {
        return Err(
            "Hostname labels must be 1-63 chars and cannot start or end with '-'".to_string(),
        );
    }

    Ok(())
}
