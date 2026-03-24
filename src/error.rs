//! `KefError` enum — network, API, type mismatch, discovery, config.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum KefError {
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("API error (status {status}): {message}")]
    Api { status: u16, message: String },

    #[error("type mismatch: expected {expected}, got {got}")]
    TypeMismatch {
        expected: &'static str,
        got: String,
    },

    #[error("discovery error: {0}")]
    Discovery(String),

    #[error("config error: {0}")]
    Config(#[from] toml::de::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_error_display() {
        let err = KefError::Api {
            status: 404,
            message: "not found".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("404"), "should contain status: {msg}");
        assert!(msg.contains("not found"), "should contain message: {msg}");
    }

    #[test]
    fn type_mismatch_display() {
        let err = KefError::TypeMismatch {
            expected: "i32",
            got: "String".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("i32"), "should contain expected: {msg}");
        assert!(msg.contains("String"), "should contain got: {msg}");
    }

    #[test]
    fn discovery_error_display() {
        let err = KefError::Discovery("timeout".to_string());
        let msg = err.to_string();
        assert!(
            msg.contains("timeout"),
            "should contain inner string: {msg}"
        );
    }

    #[test]
    fn config_error_display() {
        let toml_err = toml::from_str::<toml::Value>("not [valid").unwrap_err();
        let err = KefError::Config(toml_err);
        let msg = err.to_string();
        assert!(
            msg.contains("config error"),
            "should contain prefix: {msg}"
        );
    }
}
