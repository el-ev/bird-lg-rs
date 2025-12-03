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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub bind_socket: String,
    #[serde(deserialize_with = "deserialize_listen_address")]
    pub listen: Vec<String>,
    allowed_ips: Vec<String>,
    #[serde(skip)]
    pub allowed_nets: Vec<ipnet::IpNet>,
    pub shared_secret: Option<String>,
    pub traceroute_bin: Option<String>,
    #[serde(default, deserialize_with = "deserialize_traceroute_args")]
    pub traceroute_args: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peering: Option<PeeringInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wireguard_command: Option<String>,
}

impl Config {
    pub fn new(path: &str) -> anyhow::Result<Self> {
        tracing::info!("Loading proxy config from {}", path);
        let cfg = Self::read_and_parse(path)?
            .validated()
            .with_context(|| format!("Failed to validate config '{}'", path))?;
        tracing::info!("Loaded proxy config from {}", path);
        Ok(cfg)
    }

    fn read_and_parse(path: &str) -> anyhow::Result<Self> {
        let raw = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file '{}'", path))?;
        let cfg = serde_json::from_str(&raw)
            .with_context(|| format!("Failed to parse config file '{}'", path))?;
        Ok(cfg)
    }

    pub fn validated(mut self) -> anyhow::Result<Self> {
        let mut errors: Vec<String> = Vec::new();

        self.validate_endpoint("bind_socket", &mut errors);
        self.validate_listen(&mut errors);
        self.validate_allowed_ips(&mut errors);
        self.validate_traceroute_bin(&mut errors);

        if errors.is_empty() {
            Ok(self)
        } else {
            for err in &errors {
                tracing::error!("Config validation error: {}", err);
            }
            Err(anyhow!(errors.join("; ")))
        }
    }

    fn validate_endpoint(&self, name: &str, errors: &mut Vec<String>) {
        let val = self.bind_socket.as_str();
        if val.parse::<SocketAddr>().is_ok() {
            return;
        }

        if val.starts_with('/') {
            let path = Path::new(val);
            if val.trim().is_empty() {
                errors.push(format!("{} '{}' has empty unix socket path", name, val));
                return;
            }
            match path.parent() {
                Some(parent) if parent.exists() => {}
                Some(_) => errors.push(format!(
                    "{} '{}' parent directory does not exist",
                    name, val
                )),
                None => errors.push(format!(
                    "{} '{}' is not a valid unix socket path",
                    name, val
                )),
            }
            return;
        }

        errors.push(format!(
            "{} '{}' is not a valid socket address or unix socket",
            name, val
        ));
    }

    fn validate_listen(&self, errors: &mut Vec<String>) {
        for (idx, addr) in self.listen.iter().enumerate() {
            if let Err(e) = addr.parse::<SocketAddr>() {
                errors.push(format!(
                    "listen[{}] '{}' is not a valid socket address: {}",
                    idx, addr, e
                ));
            }
        }
    }

    fn validate_allowed_ips(&mut self, errors: &mut Vec<String>) {
        self.allowed_nets.clear();
        for entry in &mut self.allowed_ips {
            let original = entry.clone();

            let net_res: Result<ipnet::IpNet, String> = if entry.contains('/') {
                entry
                    .parse::<ipnet::IpNet>()
                    .map_err(|e| format!("allowed_ip '{}' is invalid: {}", original, e))
            } else {
                match entry.parse::<IpAddr>() {
                    Ok(IpAddr::V4(a)) => ipnet::Ipv4Net::new(a, 32)
                        .map(ipnet::IpNet::V4)
                        .map_err(|e| format!("allowed_ip '{}' is invalid: {}", original, e)),
                    Ok(IpAddr::V6(a)) => ipnet::Ipv6Net::new(a, 128)
                        .map(ipnet::IpNet::V6)
                        .map_err(|e| format!("allowed_ip '{}' is invalid: {}", original, e)),
                    Err(_) => Err(format!("allowed_ip '{}' has invalid IP", original)),
                }
            };

            match net_res {
                Ok(net) => {
                    *entry = net.to_string();
                    self.allowed_nets.push(net);
                }
                Err(e) => errors.push(e),
            }
        }
    }

    fn validate_traceroute_bin(&mut self, errors: &mut Vec<String>) {
        if let Some(ref bin) = self.traceroute_bin {
            if bin.trim().is_empty() {
                errors.push("traceroute_bin must not be empty. you can set it to null to disable traceroute functionality".to_string());
                return;
            }

            let p = Path::new(bin);
            if !p.exists() {
                errors.push(format!("traceroute_bin '{}' does not exist", bin));
            } else if !p.is_file() {
                errors.push(format!("traceroute_bin '{}' is not a file", bin));
            }
        } else if !self.traceroute_args.is_empty() {
            errors.push("traceroute_args is set but traceroute_bin isn't".to_string());
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

// FIXME maybe called split something
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
                Ok(s.split_whitespace().map(|s| s.to_string()).collect())
            }
        }
        _ => Err(Error::custom("traceroute_args must be a string or null")),
    }
}
