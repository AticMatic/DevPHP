use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error, Serialize, Clone)]
pub enum DevPhpError {
    #[error("Process error: {0}")]
    ProcessError(String),

    #[error("Download error: {0}")]
    DownloadError(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Port {0} is already in use. Tried ports {1}-{2}, all occupied.")]
    PortRangeExhausted(u16, u16, u16),

    #[error("PHP not found. {0}")]
    PhpNotFound(String),

    #[error("Service '{0}' not found in registry")]
    ServiceNotFound(String),
}

impl From<std::io::Error> for DevPhpError {
    fn from(e: std::io::Error) -> Self {
        DevPhpError::IoError(e.to_string())
    }
}

impl From<serde_json::Error> for DevPhpError {
    fn from(e: serde_json::Error) -> Self {
        DevPhpError::ConfigError(e.to_string())
    }
}

impl From<reqwest::Error> for DevPhpError {
    fn from(e: reqwest::Error) -> Self {
        DevPhpError::DownloadError(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, DevPhpError>;
