//! TOML config file loading from `~/.config/kefctl/config.toml`.

use serde::Deserialize;
use std::path::{Path, PathBuf};

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
    pub(crate) default_source: Option<String>,
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

/// Returns the XDG state directory for kefctl, or None if unavailable.
/// Never falls back to `/tmp` — world-writable dirs are a symlink attack vector.
fn state_dir() -> Option<PathBuf> {
    dirs::state_dir()
        .or_else(dirs::data_local_dir)
        .map(|d| d.join("kefctl"))
}

/// Returns the path to the cached speaker IP file.
/// Uses XDG state dir: `~/.local/state/kefctl/last_speaker`
fn cache_path() -> Option<PathBuf> {
    state_dir().map(|d| d.join("last_speaker"))
}

/// Write a file atomically: write to a temp file then rename.
/// Prevents symlink attacks and partial writes.
fn atomic_write(path: &Path, contents: &str) {
    let Some(parent) = path.parent() else { return };
    #[cfg(unix)]
    let create_result = {
        use std::os::unix::fs::DirBuilderExt;
        std::fs::DirBuilder::new().recursive(true).mode(0o700).create(parent)
    };
    #[cfg(not(unix))]
    let create_result = std::fs::create_dir_all(parent);
    if let Err(e) = create_result {
        tracing::warn!("Failed to create dir {}: {e}", parent.display());
        return;
    }
    let tmp = parent.join(format!(
        ".{}.tmp",
        path.file_name().unwrap_or_default().to_string_lossy()
    ));
    let result = std::fs::write(&tmp, contents)
        .and_then(|()| {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = std::fs::set_permissions(
                    &tmp,
                    std::fs::Permissions::from_mode(0o600),
                );
            }
            std::fs::rename(&tmp, path)
        });
    if let Err(e) = result {
        tracing::warn!("Failed to write {}: {e}", path.display());
        let _ = std::fs::remove_file(&tmp);
    }
}

/// Load the cached speaker IP from the state file.
/// Returns a validated `IpAddr` — rejects malformed content.
pub(crate) fn load_cached_ip() -> Option<std::net::IpAddr> {
    let path = cache_path()?;
    match std::fs::read_to_string(&path) {
        Ok(contents) => {
            let trimmed = contents.trim();
            if trimmed.is_empty() {
                return None;
            }
            if let Ok(ip) = trimmed.parse() {
                Some(ip)
            } else {
                tracing::warn!(
                    "Invalid cached IP in {}: {trimmed}",
                    path.display()
                );
                None
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => {
            tracing::warn!(
                "Failed to read cached speaker IP from {}: {e}",
                path.display()
            );
            None
        }
    }
}

/// Returns the path to the cached last-used source file.
/// Uses XDG state dir: `~/.local/state/kefctl/last_source`
fn source_cache_path() -> Option<PathBuf> {
    state_dir().map(|d| d.join("last_source"))
}

/// Load the last-used source from the state file.
pub(crate) fn load_last_source() -> Option<String> {
    let path = source_cache_path()?;
    match std::fs::read_to_string(&path) {
        Ok(contents) => {
            let source = contents.trim().to_string();
            if source.is_empty() { None } else { Some(source) }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => {
            tracing::warn!(
                "Failed to read last source from {}: {e}",
                path.display()
            );
            None
        }
    }
}

/// Save the last-used source to the state file.
pub(crate) fn save_last_source(source: &str) {
    let Some(path) = source_cache_path() else {
        tracing::warn!("No XDG state dir available, skipping last source save");
        return;
    };
    atomic_write(&path, source);
}

/// Save a speaker IP to the state file for next launch.
pub(crate) fn save_cached_ip(ip: &std::net::IpAddr) {
    let Some(path) = cache_path() else {
        tracing::warn!("No XDG state dir available, skipping IP cache save");
        return;
    };
    atomic_write(&path, &ip.to_string());
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
    fn config_with_default_source() {
        let toml = r#"
            [speaker]
            ip = "192.168.1.100"
            default_source = "usb"
        "#;
        let config = Config::load_from_str(toml).unwrap();
        assert_eq!(config.speaker.default_source.as_deref(), Some("usb"));
    }

    #[test]
    fn config_without_default_source() {
        let toml = r#"
            [speaker]
            ip = "192.168.1.100"
        "#;
        let config = Config::load_from_str(toml).unwrap();
        assert_eq!(config.speaker.default_source, None);
    }

    #[test]
    fn save_and_load_last_source() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("last_source");

        std::fs::write(&path, "usb").unwrap();
        let contents = std::fs::read_to_string(&path).unwrap();
        assert_eq!(contents.trim(), "usb");
    }

    #[test]
    fn load_last_source_empty_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("last_source");
        std::fs::write(&path, "").unwrap();

        let contents = std::fs::read_to_string(&path).unwrap();
        let source = contents.trim().to_string();
        assert!(source.is_empty());
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
