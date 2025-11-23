use std::net::IpAddr;

pub fn validate_target(target: &str) -> Result<(), &'static str> {
    let target = target.trim();

    if target.is_empty() {
        return Err("Target is required");
    }

    if target.parse::<IpAddr>().is_ok() {
        return Ok(());
    }

    if target.len() > 253 {
        return Err("Hostname is too long");
    }

    if target
        .bytes()
        .any(|b| !(b.is_ascii_alphanumeric() || b == b'-' || b == b'.'))
    {
        return Err("Hostname may only contain letters, digits, '-' or '.'");
    }

    if target.split('.').any(|label| {
        label.is_empty() || label.len() > 63 || label.starts_with('-') || label.ends_with('-')
    }) {
        return Err("Hostname labels must be 1-63 chars and cannot start or end with '-'");
    }

    Ok(())
}
