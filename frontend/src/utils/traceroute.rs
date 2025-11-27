use std::net::IpAddr;

use common::models::{HopRange, TracerouteHop};

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
