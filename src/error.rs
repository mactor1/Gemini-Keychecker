use thiserror::Error;

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("HTTP error: {0}")]
    HttpRequest(#[from] reqwest::Error),

    #[error("Key is unavailable or invalid")]
    KeyUnavailable,
    
    #[error("Key validation failed: {0}")]
    Invalid(String),
}

pub type Result<T> = std::result::Result<T, ValidationError>;
