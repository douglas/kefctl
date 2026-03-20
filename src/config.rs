//! TOML config file loading from `~/.config/kefctl/config.toml`.

use serde::Deserialize;
use std::path::PathBuf;

use crate::error::KefError;

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub(crate) struct Config {
    pub(crate) speaker: SpeakerConfig,
    pub(crate) ui: UiConfig,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub(crate) struct SpeakerConfig {
    pub(crate) ip: Option<String>,
    pub(crate) name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub(crate) struct UiConfig {
    pub(crate) refresh_ms: u64,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self { refresh_ms: 1000 }
    }
}

impl Config {
    pub(crate) fn load() -> Result<Self, KefError> {
        let path = Self::config_path();
        match std::fs::read_to_string(&path) {
            Ok(contents) => Self::load_from_str(&contents),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Config::default()),
            Err(_) => Ok(Config::default()),
        }
    }

    pub(crate) fn load_from_str(s: &str) -> Result<Self, KefError> {
        Ok(toml::from_str(s)?)
    }

    fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("/etc"))
            .join("kefctl")
            .join("config.toml")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_full_config() {
        let toml = r#"
            [speaker]
            ip = "192.168.1.100"
            name = "Living Room"

            [ui]
            refresh_ms = 500
        "#;
        let config = Config::load_from_str(toml).unwrap();
        assert_eq!(config.speaker.ip.as_deref(), Some("192.168.1.100"));
        assert_eq!(config.speaker.name.as_deref(), Some("Living Room"));
        assert_eq!(config.ui.refresh_ms, 500);
    }

    #[test]
    fn valid_partial_config() {
        let toml = r#"
            [speaker]
            ip = "10.0.0.5"
        "#;
        let config = Config::load_from_str(toml).unwrap();
        assert_eq!(config.speaker.ip.as_deref(), Some("10.0.0.5"));
        assert_eq!(config.speaker.name, None);
        assert_eq!(config.ui.refresh_ms, 1000);
    }

    #[test]
    fn empty_config() {
        let config = Config::load_from_str("").unwrap();
        assert_eq!(config.speaker.ip, None);
        assert_eq!(config.speaker.name, None);
        assert_eq!(config.ui.refresh_ms, 1000);
    }

    #[test]
    fn invalid_toml() {
        let result = Config::load_from_str("not [valid toml");
        assert!(result.is_err());
    }

    #[test]
    fn unknown_fields_ignored() {
        let toml = r#"
            [speaker]
            ip = "1.2.3.4"
            unknown_field = "whatever"

            [mystery_section]
            foo = "bar"
        "#;
        let config = Config::load_from_str(toml).unwrap();
        assert_eq!(config.speaker.ip.as_deref(), Some("1.2.3.4"));
    }

    #[test]
    fn default_refresh_ms() {
        let config = Config::default();
        assert_eq!(config.ui.refresh_ms, 1000);
    }

    #[test]
    fn wrong_type_for_refresh_ms() {
        let toml = r#"
            [ui]
            refresh_ms = "not_a_number"
        "#;
        let result = Config::load_from_str(toml);
        assert!(result.is_err());
    }

    #[test]
    fn wrong_type_for_ip() {
        // ip expects a string; giving an integer should fail
        let toml = r"
            [speaker]
            ip = 12345
        ";
        let result = Config::load_from_str(toml);
        assert!(result.is_err());
    }

    #[test]
    fn nested_invalid_structure() {
        // speaker.ip is a flat string, not a table
        let toml = r#"
            [speaker]
            [speaker.ip]
            addr = "1.2.3.4"
        "#;
        let result = Config::load_from_str(toml);
        assert!(result.is_err());
    }
}
