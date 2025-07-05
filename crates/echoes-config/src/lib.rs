//! Configuration management and shortcut conflict detection for Echoes
//!
//! This crate provides comprehensive configuration management including:
//! - Configuration data structures and validation
//! - Shortcut conflict detection system
//! - Platform-specific shortcut validation
//! - Configuration persistence

pub mod config;
pub mod conflict;
pub mod shortcuts;
pub mod validation;

// Re-export main types for convenience
pub use config::*;
pub use conflict::*;
pub use shortcuts::*;
pub use validation::*;

/// Result type for this crate
pub type Result<T> = std::result::Result<T, ConfigError>;

/// Main configuration error type
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Failed to load config: {0}")]
    LoadFailed(String),

    #[error("Failed to save config: {0}")]
    SaveFailed(String),

    #[error("Config parse error: {0}")]
    ParseError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),
}
