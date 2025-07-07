//! Logging and tracing utilities for Echoes
//!
//! This crate provides a unified logging system with:
//! - Structured logging with tracing
//! - File rotation and cleanup
//! - Error tracking and reporting
//! - Console and file output
//! - Panic handling

pub mod error;
pub mod tracing_setup;

pub use error::LoggingError;
/// Re-export tracing macros for convenience
pub use tracing::{debug, error, info, trace, warn};
pub use tracing_setup::{cleanup_tracing, init_tracing, setup_panic_handler, ErrorReport, TracingConfig};

/// Result type for logging operations
pub type Result<T> = std::result::Result<T, LoggingError>;
