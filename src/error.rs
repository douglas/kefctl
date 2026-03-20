use thiserror::Error;

#[derive(Debug, Error)]
pub enum KefError {
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("API error (status {status}): {message}")]
    Api { status: u16, message: String },

    #[error("discovery error: {0}")]
    Discovery(String),

    #[error("config error: {0}")]
    Config(#[from] toml::de::Error),
}
