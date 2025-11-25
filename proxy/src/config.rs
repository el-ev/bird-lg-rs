use anyhow::{Context, anyhow};
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, SocketAddr};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PeeringInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv4: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv6: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link_local_ipv6: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wg_pubkey_path: Option<String>,
    #[serde(skip)]
    pub wg_pubkey: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub bind_socket: String,
    pub listen: String,
    allowed_ips: Vec<String>,
    #[serde(skip)]
    pub allowed_nets: Vec<ipnet::IpNet>,
    pub shared_secret: Option<String>,
    pub traceroute_bin: Option<String>,
    traceroute_args: Option<String>,
    #[serde(skip)]
    pub tr_arglist: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peering: Option<PeeringInfo>,
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
        self.load_peering_pubkey(&mut errors);

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
        if let Err(e) = self.listen.parse::<SocketAddr>() {
            errors.push(format!(
                "listen '{}' is not a valid socket address: {}",
                self.listen, e
            ));
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
        } else if let Some(args) = &self.traceroute_args
            && !args.trim().is_empty()
        {
            errors.push("traceroute_args is set but traceroute_bin isn't".to_string());
        }
        self.tr_arglist = self
            .traceroute_args
            .as_ref()
            .map(|s| s.split_whitespace().map(|s| s.to_string()).collect())
            .unwrap_or_default();
    }

    fn load_peering_pubkey(&mut self, errors: &mut Vec<String>) {
        if let Some(ref mut peering) = self.peering {
            if let Some(ref path) = peering.wg_pubkey_path {
                match std::fs::read_to_string(path) {
                    Ok(content) => {
                        peering.wg_pubkey = Some(content.trim().to_string());
                    }
                    Err(e) => {
                        errors.push(format!("Failed to read wg_pubkey_path '{}': {}", path, e));
                    }
                }
            }
        }
    }
}
