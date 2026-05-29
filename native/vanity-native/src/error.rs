use std::fmt;

#[derive(Debug)]
pub enum VanityError {
    InvalidHex(String),
    BuildFailed(String),
    Io(std::io::Error),
    PatternTooLong,
    NoPattern,
    InvalidWorkers,
    InvalidInterval,
    InvalidBatchSize,
}

impl fmt::Display for VanityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VanityError::InvalidHex(msg) => write!(f, "Invalid hex pattern: {}", msg),
            VanityError::BuildFailed(msg) => write!(f, "Build failed: {}", msg),
            VanityError::Io(err) => write!(f, "IO error: {}", err),
            VanityError::PatternTooLong => write!(f, "Prefix plus suffix cannot exceed 40 hex characters"),
            VanityError::NoPattern => write!(f, "At least one of prefix or suffix is required"),
            VanityError::InvalidWorkers => write!(f, "Workers must be at least 1"),
            VanityError::InvalidInterval => write!(f, "Status interval must be at least 1"),
            VanityError::InvalidBatchSize => write!(f, "Batch size must be at least 1"),
        }
    }
}

impl From<std::io::Error> for VanityError {
    fn from(err: std::io::Error) -> Self {
        VanityError::Io(err)
    }
}

pub type Result<T> = std::result::Result<T, VanityError>;
