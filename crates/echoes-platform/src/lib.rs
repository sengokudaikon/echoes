//! Platform-specific functionality for echoes dictation application
//!
//! This crate provides platform-specific implementations for permissions,
//! notifications, and other system integration features.

// Re-export platform modules
pub mod notifications;
pub mod permissions;

// Re-export common types
pub use notifications::*;
pub use permissions::*;

/// Error type for platform-specific operations
#[derive(thiserror::Error, Debug)]
pub enum PlatformError {
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Platform not supported: {0}")]
    PlatformNotSupported(String),
    #[error("System error: {0}")]
    SystemError(String),
}

/// Result type for platform operations
pub type Result<T> = std::result::Result<T, PlatformError>;
