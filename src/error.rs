use thiserror::Error;

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("HTTP error: {0}")]
    ReqwestError(#[from] reqwest::Error),

    #[error("Key is unavailable or invalid")]
    KeyInvalid,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    ConfigError(#[from] figment::Error),

    #[error("TOML serialization error: {0}")]
    TomlSer(#[from] toml::ser::Error),

    #[error("TOML deserialization error: {0}")]
    TomlDe(#[from] toml::de::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),

    #[error("Key validation failed: {0}")]
    Validation(String),
}

pub type Result<T> = std::result::Result<T, ValidationError>;
