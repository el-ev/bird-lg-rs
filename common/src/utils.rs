pub fn filter_protocol_details(raw: &str) -> String {
    const PROTOCOL_HEADERS: [&str; 6] = ["Name", "Proto", "Table", "State", "Since", "Info"];
    raw.lines()
        .filter(|line| PROTOCOL_HEADERS.iter().any(|header| !line.contains(header)))
        .collect::<Vec<_>>()
        .join("\n")
}

use std::net::IpAddr;

use serde::{Deserialize, Deserializer};

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

pub fn deserialize_listen_address<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    use serde_json::Value;

    let value = Value::deserialize(deserializer)?;
    match value {
        Value::String(s) => Ok(vec![s]),
        Value::Array(arr) => arr
            .into_iter()
            .map(|v| {
                v.as_str()
                    .ok_or_else(|| Error::custom("listen array must contain strings"))
                    .map(|s| s.to_string())
            })
            .collect(),
        _ => Err(Error::custom("listen must be a string or array of strings")),
    }
}
