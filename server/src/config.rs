use anyhow::Result;
use serde::Deserialize;
use std::fs;

pub use common::models::{NetworkInfo, PeeringInfo};
use common::utils::deserialize_listen_address;

#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    #[serde(deserialize_with = "deserialize_listen_address")]
    pub listen: Vec<String>,
    pub nodes: Vec<NodeConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<NetworkInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poll_idle_timeout: Option<u64>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct NodeConfig {
    pub name: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shared_secret: Option<String>,
}

impl Config {
    pub fn load(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config = serde_json::from_str(&content)?;
        Ok(config)
    }
}
