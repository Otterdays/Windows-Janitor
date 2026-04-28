use thiserror::Error;

/// Core error type for Janitor engine operations.
#[derive(Debug, Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Path is on hard blacklist (safety boundary): {0}")]
    BlacklistViolation(String),

    #[error("Scanner error: {0}")]
    Scanner(String),

    #[error("Invalid rule: {0}")]
    InvalidRule(String),

    #[error("Scan context error: {0}")]
    ScanContext(String),

    #[error("Classification error: {0}")]
    Classification(String),

    #[error("Quarantine error: {0}")]
    Quarantine(String),

    #[error("Registry error: {0}")]
    Registry(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
