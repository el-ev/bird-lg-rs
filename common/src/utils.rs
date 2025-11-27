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

pub fn fold_consecutive_timeouts(hops: Vec<TracerouteHop>) -> Vec<TracerouteHop> {
    if hops.is_empty() {
        return hops;
    }

    let mut result = Vec::new();
    let mut timeout_start: Option<u32> = None;
    let mut timeout_end: Option<u32> = None;

    for hop in hops {
        let is_timeout = hop.address.is_none() && hop.hostname.is_none();

        if is_timeout {
            let hop_num = hop.hop.start();
            match timeout_start {
                None => {
                    // Start of a new timeout sequence
                    timeout_start = Some(hop_num);
                    timeout_end = Some(hop_num);
                }
                Some(_) => {
                    // Continue the timeout sequence
                    timeout_end = Some(hop_num);
                }
            }
        } else {
            // Not a timeout - flush any pending timeout range
            if let (Some(start), Some(end)) = (timeout_start, timeout_end) {
                if start == end {
                    result.push(TracerouteHop {
                        hop: HopRange::Single(start),
                        address: None,
                        hostname: None,
                        rtts: None,
                    });
                } else {
                    result.push(TracerouteHop {
                        hop: HopRange::Range(start, end),
                        address: None,
                        hostname: None,
                        rtts: None,
                    });
                }
                timeout_start = None;
                timeout_end = None;
            }

            // Add the current non-timeout hop
            result.push(hop);
        }
    }

    // Flush any remaining timeout range at the end
    if let (Some(start), Some(end)) = (timeout_start, timeout_end) {
        if start == end {
            result.push(TracerouteHop {
                hop: HopRange::Single(start),
                address: None,
                hostname: None,
                rtts: None,
            });
        } else {
            result.push(TracerouteHop {
                hop: HopRange::Range(start, end),
                address: None,
                hostname: None,
                rtts: None,
            });
        }
    }

    result
}

fn parse_ip(value: &str) -> String {
    value
        .trim_matches(&['(', ')'][..])
        .parse::<IpAddr>()
        .map(|ip| ip.to_string())
        .unwrap_or_else(|_| value.to_string())
}
