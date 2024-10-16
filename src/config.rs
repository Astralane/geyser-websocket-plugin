use agave_geyser_plugin_interface::geyser_plugin_interface::{
    GeyserPluginError, Result as PluginResult,
};
use serde::Deserialize;
use std::fs::read_to_string;
use std::net::SocketAddr;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct ConfigLog {
    /// Log level.
    pub level: String,
}

impl Default for ConfigLog {
    fn default() -> Self {
        Self {
            level: "info".to_owned(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct ConfigWs {
    pub address: SocketAddr,
}

impl Default for ConfigWs {
    fn default() -> Self {
        Self {
            address: "127.0.0.1:10050".parse().unwrap(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub websocket: ConfigWs,
    pub log: ConfigLog,
}

impl Config {
    fn load_from_str(config: &str) -> PluginResult<Self> {
        serde_json::from_str(config).map_err(|error| GeyserPluginError::ConfigFileReadError {
            msg: error.to_string(),
        })
    }

    pub fn load_from_file<P: AsRef<Path>>(file: P) -> PluginResult<Self> {
        let config = read_to_string(file).map_err(GeyserPluginError::ConfigFileOpenError)?;
        Self::load_from_str(&config)
    }
}
