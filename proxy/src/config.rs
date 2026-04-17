use std::{
    net::{IpAddr, SocketAddr},
    path::Path,
};

use anyhow::{Context, anyhow};
use common::utils::deserialize_listen_address;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PeeringInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv4: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv6: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link_local_ipv6: Option<String>,
    #[serde(
        deserialize_with = "deserialize_wg_pubkey",
        skip_serializing_if = "Option::is_none"
    )]
    pub wg_pubkey: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawConfig {
    bind_socket: String,
    #[serde(deserialize_with = "deserialize_listen_address")]
    listen: Vec<String>,
    #[serde(default)]
    allowed_ips: Vec<String>,
    shared_secret: Option<String>,
    traceroute_bin: Option<String>,
    #[serde(default, deserialize_with = "deserialize_traceroute_args")]
    traceroute_args: Vec<String>,
    peering: Option<PeeringInfo>,
    wireguard_command: Option<String>,
    ping_bin: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub bind_socket: String,
    pub listen: Vec<String>,
    pub allowed_nets: Vec<ipnet::IpNet>,
    pub shared_secret: Option<String>,
    pub traceroute_bin: Option<String>,
    pub traceroute_args: Vec<String>,
    pub peering: Option<PeeringInfo>,
    pub wireguard_command: Option<String>,
    pub ping_bin: Option<String>,
}

impl Config {
    pub fn new(path: &str) -> anyhow::Result<Self> {
        tracing::info!("Loading proxy config from {}", path);
        let raw = Self::read_and_parse(path)?;
        let cfg = raw
            .into_runtime()
            .with_context(|| format!("Failed to validate config '{}'", path))?;
        tracing::info!("Loaded proxy config from {}", path);
        Ok(cfg)
    }

    fn read_and_parse(path: &str) -> anyhow::Result<RawConfig> {
        let raw = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file '{}'", path))?;
        serde_json::from_str(&raw)
            .with_context(|| format!("Failed to parse config file '{}'", path))
    }
}

impl RawConfig {
    fn into_runtime(self) -> anyhow::Result<Config> {
        let RawConfig {
            bind_socket,
            listen,
            allowed_ips,
            shared_secret,
            traceroute_bin,
            traceroute_args,
            peering,
            wireguard_command,
            ping_bin,
        } = self;

        let mut errors = Vec::new();

        validate_endpoint("bind_socket", &bind_socket, &mut errors);
        validate_listen(&listen, &mut errors);
        let allowed_nets = normalize_allowed_ips(allowed_ips, &mut errors);
        validate_traceroute_bin(&traceroute_bin, &traceroute_args, &mut errors);
        validate_ping_bin(&ping_bin, &mut errors);

        if !errors.is_empty() {
            for err in &errors {
                tracing::error!("Config validation error: {}", err);
            }
            return Err(anyhow!(errors.join("; ")));
        }

        Ok(Config {
            bind_socket,
            listen,
            allowed_nets,
            shared_secret,
            traceroute_bin,
            traceroute_args,
            peering,
            wireguard_command,
            ping_bin,
        })
    }
}

fn validate_endpoint(name: &str, value: &str, errors: &mut Vec<String>) {
    if value.parse::<SocketAddr>().is_ok() {
        return;
    }

    if value.starts_with('/') {
        let path = Path::new(value);
        if value.trim().is_empty() {
            errors.push(format!("{} '{}' has empty unix socket path", name, value));
            return;
        }

        match path.parent() {
            Some(parent) if parent.exists() => {}
            Some(_) => errors.push(format!(
                "{} '{}' parent directory does not exist",
                name, value
            )),
            None => errors.push(format!(
                "{} '{}' is not a valid unix socket path",
                name, value
            )),
        }
        return;
    }

    errors.push(format!(
        "{} '{}' is not a valid socket address or unix socket",
        name, value
    ));
}

fn validate_listen(listen: &[String], errors: &mut Vec<String>) {
    for (idx, addr) in listen.iter().enumerate() {
        if let Err(error) = addr.parse::<SocketAddr>() {
            errors.push(format!(
                "listen[{}] '{}' is not a valid socket address: {}",
                idx, addr, error
            ));
        }
    }
}

fn normalize_allowed_ips(entries: Vec<String>, errors: &mut Vec<String>) -> Vec<ipnet::IpNet> {
    let mut allowed_nets = Vec::new();

    for entry in entries {
        let original = entry.clone();
        let parsed = if entry.contains('/') {
            entry
                .parse::<ipnet::IpNet>()
                .map_err(|error| format!("allowed_ip '{}' is invalid: {}", original, error))
        } else {
            match entry.parse::<IpAddr>() {
                Ok(IpAddr::V4(value)) => ipnet::Ipv4Net::new(value, 32)
                    .map(ipnet::IpNet::V4)
                    .map_err(|error| format!("allowed_ip '{}' is invalid: {}", original, error)),
                Ok(IpAddr::V6(value)) => ipnet::Ipv6Net::new(value, 128)
                    .map(ipnet::IpNet::V6)
                    .map_err(|error| format!("allowed_ip '{}' is invalid: {}", original, error)),
                Err(_) => Err(format!("allowed_ip '{}' has invalid IP", original)),
            }
        };

        match parsed {
            Ok(net) => allowed_nets.push(net),
            Err(error) => errors.push(error),
        }
    }

    allowed_nets
}

fn validate_traceroute_bin(
    traceroute_bin: &Option<String>,
    traceroute_args: &[String],
    errors: &mut Vec<String>,
) {
    if let Some(bin) = traceroute_bin {
        if bin.trim().is_empty() {
            errors.push("traceroute_bin must not be empty. you can set it to null to disable traceroute functionality".to_string());
            return;
        }

        let path = Path::new(bin);
        if !path.exists() {
            errors.push(format!("traceroute_bin '{}' does not exist", bin));
        } else if !path.is_file() {
            errors.push(format!("traceroute_bin '{}' is not a file", bin));
        }
    } else if !traceroute_args.is_empty() {
        errors.push("traceroute_args is set but traceroute_bin isn't".to_string());
    }
}

fn validate_ping_bin(ping_bin: &Option<String>, errors: &mut Vec<String>) {
    if let Some(bin) = ping_bin {
        if bin.trim().is_empty() {
            errors.push(
                "ping_bin must not be empty. you can set it to null to disable ping functionality"
                    .to_string(),
            );
            return;
        }

        let path = Path::new(bin);
        if !path.exists() {
            errors.push(format!("ping_bin '{}' does not exist", bin));
        } else if !path.is_file() {
            errors.push(format!("ping_bin '{}' is not a file", bin));
        }
    }
}

pub fn deserialize_wg_pubkey<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    use serde_json::Value;

    let value = Value::deserialize(deserializer)?;
    match value {
        Value::Null => Ok(None),
        Value::String(s) => {
            if s.starts_with('/') || s.starts_with("./") || s.starts_with("../") {
                std::fs::read_to_string(&s)
                    .map(|content| Some(content.trim().to_string()))
                    .map_err(|e| {
                        Error::custom(format!("Failed to read wg_pubkey from '{}': {}", s, e))
                    })
            } else {
                Ok(Some(s))
            }
        }
        _ => Err(Error::custom("wg_pubkey must be a string or null")),
    }
}

pub fn deserialize_traceroute_args<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    use serde_json::Value;

    let value = Value::deserialize(deserializer)?;
    match value {
        Value::Null => Ok(Vec::new()),
        Value::String(s) => {
            if s.trim().is_empty() {
                Ok(Vec::new())
            } else {
                Ok(s.split_whitespace()
                    .map(|value| value.to_string())
                    .collect())
            }
        }
        _ => Err(Error::custom("traceroute_args must be a string or null")),
    }
}

#[cfg(test)]
mod tests {
    use super::RawConfig;

    #[test]
    fn normalizes_plain_ip_allowlist_entries_to_nets() {
        let raw = RawConfig {
            bind_socket: "127.0.0.1:1790".to_string(),
            listen: vec!["127.0.0.1:3000".to_string()],
            allowed_ips: vec!["192.0.2.1".to_string(), "2001:db8::1".to_string()],
            shared_secret: None,
            traceroute_bin: None,
            traceroute_args: Vec::new(),
            peering: None,
            wireguard_command: None,
            ping_bin: None,
        };

        let config = raw.into_runtime().expect("runtime config should validate");
        assert_eq!(config.allowed_nets[0].to_string(), "192.0.2.1/32");
        assert_eq!(config.allowed_nets[1].to_string(), "2001:db8::1/128");
    }
}
