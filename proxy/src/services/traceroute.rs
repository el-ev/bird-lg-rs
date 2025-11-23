use serde::Serialize;
use std::net::IpAddr;
use std::process::Stdio;
use tokio::process::Command;

use crate::config::Config;

#[derive(Debug)]
pub enum TracerouteError {
    InvalidTarget,
    ParseError,
}

impl std::fmt::Display for TracerouteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TracerouteError::InvalidTarget => write!(f, "Invalid target"),
            TracerouteError::ParseError => write!(f, "Parse error"),
        }
    }
}

impl std::error::Error for TracerouteError {}

pub enum IpVersion {
    V4,
    V6,
    Any,
}

// FIXME: Only works with -q1
#[derive(Serialize)]
pub struct TracerouteEntry {
    pub hop: u32,
    pub address: Option<IpAddr>,
    pub hostname: Option<String>,
    pub rtts: Option<Vec<f32>>,
}

impl TryFrom<&str> for TracerouteEntry {
    type Error = TracerouteError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut parts = value.split_whitespace();

        let hop_str = parts.next().ok_or(TracerouteError::ParseError)?;
        let hop = hop_str
            .parse::<u32>()
            .map_err(|_| TracerouteError::ParseError)?;

        let next_part = parts.next().ok_or(TracerouteError::ParseError)?;

        if next_part == "*" {
            return Ok(TracerouteEntry {
                hop,
                address: None,
                hostname: None,
                rtts: None,
            });
        }

        let hostname;
        let address;

        let third_part = parts.clone().next();

        if let Some(p) = third_part {
            if p.starts_with('(') && p.ends_with(')') {
                hostname = Some(next_part.to_string());
                let addr_str = &p[1..p.len() - 1];
                address = Some(
                    addr_str
                        .parse::<IpAddr>()
                        .map_err(|_| TracerouteError::ParseError)?,
                );
                parts.next();
            } else {
                hostname = None;
                address = Some(
                    next_part
                        .parse::<IpAddr>()
                        .map_err(|_| TracerouteError::ParseError)?,
                );
            }
        } else {
            return Err(TracerouteError::ParseError);
        }

        let mut rtts = Vec::new();
        for part in parts {
            if part == "ms" {
                continue;
            }
            if part == "*" {
                rtts.push(-1.0);
            } else if let Ok(rtt) = part.parse::<f32>() {
                rtts.push(rtt);
            }
        }

        Ok(TracerouteEntry {
            hop,
            address,
            hostname,
            rtts: Some(rtts),
        })
    }
}

pub fn validate_target(target: &str) -> Result<(), TracerouteError> {
    if target.is_empty() {
        return Err(TracerouteError::InvalidTarget);
    }

    if target.parse::<IpAddr>().is_ok() {
        return Ok(());
    }

    if target.len() > 253 {
        return Err(TracerouteError::InvalidTarget);
    }

    let allowed_chars = |c: u8| c.is_ascii_alphanumeric() || c == b'-' || c == b'.';
    if target.bytes().any(|b| !allowed_chars(b)) {
        return Err(TracerouteError::InvalidTarget);
    }

    if target.split('.').any(|label| {
        label.is_empty() || label.len() > 63 || label.starts_with('-') || label.ends_with('-')
    }) {
        return Err(TracerouteError::InvalidTarget);
    }

    Ok(())
}

pub fn build_traceroute_command(
    config: &Config,
    target: &str,
    version: IpVersion,
) -> Option<Command> {
    let bin = config.traceroute_bin.as_ref()?;
    let mut cmd = Command::new(bin);

    for arg in &config.tr_arglist {
        cmd.arg(arg);
    }

    match version {
        IpVersion::V4 => {
            cmd.arg("-4");
        }
        IpVersion::V6 => {
            cmd.arg("-6");
        }
        IpVersion::Any => {}
    }

    cmd.arg(target);
    cmd.stdout(Stdio::piped());

    Some(cmd)
}
