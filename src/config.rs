use serde::Deserialize;
use std::path::PathBuf;

use crate::error::KefError;

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    pub speaker: SpeakerConfig,
    pub ui: UiConfig,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct SpeakerConfig {
    pub ip: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct UiConfig {
    pub refresh_ms: u64,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self { refresh_ms: 1000 }
    }
}

impl Config {
    pub fn load() -> Result<Self, KefError> {
        let path = Self::config_path();
        match std::fs::read_to_string(&path) {
            Ok(contents) => Ok(toml::from_str(&contents)?),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Config::default()),
            Err(_) => Ok(Config::default()),
        }
    }

    fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("kefctl")
            .join("config.toml")
    }
}
