//! Error types for logging operations

use thiserror::Error;

/// Errors that can occur during logging operations
#[derive(Error, Debug)]
pub enum LoggingError {
    #[error("Failed to create log file: {0}")]
    FileCreationFailed(String),

    #[error("Mutex poisoned")]
    MutexPoisoned,

    #[error("Invalid log configuration: {0}")]
    InvalidConfiguration(String),

    #[error("Tracing initialization failed: {0}")]
    TracingInitFailed(String),
}
