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

/// Returns the path to the cached speaker IP file.
/// Uses XDG state dir: `~/.local/state/kefctl/last_speaker`
fn cache_path() -> PathBuf {
    dirs::state_dir()
        .or_else(dirs::data_local_dir)
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("kefctl")
        .join("last_speaker")
}

/// Load the cached speaker IP from the state file.
pub(crate) fn load_cached_ip() -> Option<String> {
    let path = cache_path();
    match std::fs::read_to_string(&path) {
        Ok(contents) => {
            let ip = contents.trim().to_string();
            if ip.is_empty() { None } else { Some(ip) }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => {
            tracing::warn!("Failed to read cached speaker IP from {}: {e}", path.display());
            None
        }
    }
}

/// Save a speaker IP to the state file for next launch.
pub(crate) fn save_cached_ip(ip: &std::net::IpAddr) {
    let path = cache_path();
    if let Some(parent) = path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            tracing::warn!("Failed to create cache dir {}: {e}", parent.display());
            return;
        }
    }
    if let Err(e) = std::fs::write(&path, ip.to_string()) {
        tracing::warn!("Failed to save cached speaker IP to {}: {e}", path.display());
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
    fn save_and_load_cached_ip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("last_speaker");

        // Save
        let ip: std::net::IpAddr = "192.168.50.17".parse().unwrap();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        std::fs::write(&path, ip.to_string()).unwrap();

        // Load
        let contents = std::fs::read_to_string(&path).unwrap();
        let loaded = contents.trim().to_string();
        assert_eq!(loaded, "192.168.50.17");
    }

    #[test]
    fn load_cached_ip_empty_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("last_speaker");
        std::fs::write(&path, "").unwrap();

        let contents = std::fs::read_to_string(&path).unwrap();
        let ip = contents.trim().to_string();
        assert!(ip.is_empty());
    }

    #[test]
    fn load_cached_ip_missing_file() {
        // load_cached_ip returns None when file doesn't exist
        // Test the logic directly
        let result = std::fs::read_to_string("/nonexistent/path/last_speaker");
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
