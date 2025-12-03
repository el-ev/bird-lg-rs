use std::net::IpAddr;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum HopRange {
    Single(u32),
    Range(u32, u32),
}

impl HopRange {
    pub fn start(&self) -> u32 {
        match self {
            HopRange::Single(n) | HopRange::Range(n, _) => *n,
        }
    }

    pub fn end(&self) -> u32 {
        match self {
            HopRange::Single(n) | HopRange::Range(_, n) => *n,
        }
    }
}

impl std::fmt::Display for HopRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HopRange::Single(n) => write!(f, "{}", n),
            HopRange::Range(start, end) => write!(f, "{}-{}", start, end),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TracerouteHop {
    pub hop: HopRange,
    pub address: Option<String>,
    pub hostname: Option<String>,
    pub rtts: Option<Vec<f32>>,
}

#[derive(Deserialize)]
pub struct TracerouteParams {
    pub target: String,
    #[serde(default)]
    pub version: String,
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
