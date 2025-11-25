use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct NetworkInfo {
    pub name: String,
    pub asn: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct PeeringInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv4: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv6: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link_local_ipv6: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wg_pubkey: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    pub listen: String,
    pub nodes: Vec<NodeConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<NetworkInfo>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct NodeConfig {
    pub name: String,
    pub url: String,
}

impl Config {
    pub fn load(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config = serde_json::from_str(&content)?;
        Ok(config)
    }
}
