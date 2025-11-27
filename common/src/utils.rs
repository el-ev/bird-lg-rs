use crate::models::{HopRange, TracerouteHop};
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

pub fn parse_traceroute_line(line: &str) -> Option<TracerouteHop> {
    let mut parts = line.split_whitespace();

    let hop = parts.next()?.parse::<u32>().ok()?;
    let next_part = parts.next()?;

    if next_part == "*" {
        return Some(TracerouteHop {
            hop: HopRange::Single(hop),
            address: None,
            hostname: None,
            rtts: None,
        });
    }

    let mut hostname = None;
    let address;

    if let Some(third_part) = parts.clone().next() {
        if third_part.starts_with('(') && third_part.ends_with(')') {
            hostname = Some(next_part.to_string());
            let addr_str = &third_part[1..third_part.len() - 1];
            address = Some(parse_ip(addr_str));
            parts.next();
        } else {
            address = Some(parse_ip(next_part));
        }
    } else {
        return None;
    }

    let mut rtts: Vec<f32> = Vec::new();
    let mut pending_value: Option<String> = None;

    for token in parts {
        if token == "*" {
            rtts.push(-1.0);
            pending_value = None;
            continue;
        }

        if token.eq_ignore_ascii_case("ms") {
            if let Some(value) = pending_value.take()
                && let Ok(parsed) = value.parse::<f32>()
            {
                rtts.push(parsed);
            }
            continue;
        }

        if token.ends_with("ms") {
            let numeric = token.trim_end_matches("ms");
            if let Ok(parsed) = numeric.trim().parse::<f32>() {
                rtts.push(parsed);
            }
            pending_value = None;
            continue;
        }

        pending_value = Some(token.to_string());
    }

    if let Some(value) = pending_value
        && let Ok(parsed) = value.parse::<f32>()
    {
        rtts.push(parsed);
    }

    Some(TracerouteHop {
        hop: HopRange::Single(hop),
        address,
        hostname,
        rtts: if rtts.is_empty() { None } else { Some(rtts) },
    })
}

fn parse_ip(value: &str) -> String {
    value
        .trim_matches(&['(', ')'][..])
        .parse::<IpAddr>()
        .map(|ip| ip.to_string())
        .unwrap_or_else(|_| value.to_string())
}

pub fn fold_timeouts(hops: &[TracerouteHop]) -> Vec<TracerouteHop> {
    if hops.is_empty() {
        return Vec::new();
    }

    let mut result = Vec::new();
    let mut pending_timeout: Option<(u32, u32)> = None;

    let flush_timeout = |(start, end): (u32, u32), res: &mut Vec<TracerouteHop>| {
        let hop_enum = if start == end {
            HopRange::Single(start)
        } else {
            HopRange::Range(start, end)
        };

        res.push(TracerouteHop {
            hop: hop_enum,
            address: None,
            hostname: None,
            rtts: None,
        });
    };

    for hop in hops {
        let is_timeout = hop.address.is_none() && hop.hostname.is_none();
        let hop_num = hop.hop.start();

        if is_timeout {
            match pending_timeout {
                Some((start, _)) => {
                    pending_timeout = Some((start, hop_num));
                }
                None => {
                    pending_timeout = Some((hop_num, hop_num));
                }
            }
        } else {
            if let Some(range) = pending_timeout.take() {
                flush_timeout(range, &mut result);
            }
            result.push(hop.clone());
        }
    }

    if let Some(range) = pending_timeout {
        flush_timeout(range, &mut result);
    }

    result
}
