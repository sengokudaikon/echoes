use std::{
    path::PathBuf,
    sync::{atomic::AtomicPtr, LazyLock},
};

use tracing::{Level, Subscriber};
use tracing_appender::{non_blocking, rolling};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

use crate::{error::LoggingError, Result};

/// Global atomic storage for the tracing guard to prevent memory leaks
static TRACING_GUARD: LazyLock<AtomicPtr<tracing_appender::non_blocking::WorkerGuard>> =
    LazyLock::new(|| AtomicPtr::new(std::ptr::null_mut()));

/// Configuration for the tracing system
pub struct TracingConfig {
    /// Directory for log files
    pub log_dir: PathBuf,
    /// Application name for log files
    pub app_name: String,
    /// Enable console output
    pub console_output: bool,
    /// Enable file output
    pub file_output: bool,
    /// Log level filter
    pub log_level: String,
    /// Enable ANSI colors in console output
    pub ansi_colors: bool,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            log_dir: directories::ProjectDirs::from("com", "echoes", "Echoes")
                .map_or_else(|| PathBuf::from("."), |dirs| dirs.data_dir().to_path_buf()),
            app_name: "echoes".to_string(),
            console_output: true,
            file_output: true,
            log_level: "echoes=debug,warn".to_string(),
            ansi_colors: true,
        }
    }
}

/// Initialize the tracing system with comprehensive error tracking
///
/// # Errors
///
/// Returns an error if:
/// - The log directory cannot be created
/// - The log filter configuration is invalid
/// - The tracing subscriber cannot be initialized
pub fn init_tracing(config: &TracingConfig) -> Result<()> {
    // Create log directory if it doesn't exist
    if config.file_output {
        std::fs::create_dir_all(&config.log_dir)
            .map_err(|e| LoggingError::FileCreationFailed(format!("Failed to create log directory: {e}")))?;
    }

    // Set up environment filter
    let env_filter = EnvFilter::try_new(&config.log_level)
        .map_err(|e| LoggingError::InvalidConfiguration(format!("Invalid log filter: {e}")))?;

    // Create the subscriber layers
    let mut layers = Vec::new();

    // Console layer
    if config.console_output {
        let console_layer = fmt::layer()
            .with_ansi(config.ansi_colors)
            .with_target(true)
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_file(true)
            .with_line_number(true)
            .boxed();
        layers.push(console_layer);
    }

    // File layer with rotation
    if config.file_output {
        let file_appender = rolling::daily(&config.log_dir, &config.app_name);
        let (non_blocking, guard) = non_blocking(file_appender);

        let file_layer = fmt::layer()
            .with_writer(non_blocking)
            .with_ansi(false)
            .with_target(true)
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_file(true)
            .with_line_number(true)
            .json() // Use JSON format for easier parsing
            .boxed();
        layers.push(file_layer);

        // Store the guard to keep the non-blocking writer alive
        // Use atomic storage to avoid mutex poisoning issues
        let guard_box = Box::new(guard);
        let guard_ptr = Box::into_raw(guard_box);

        // Store the pointer atomically
        let old_ptr = TRACING_GUARD.swap(guard_ptr, std::sync::atomic::Ordering::AcqRel);

        // Clean up any existing guard
        if !old_ptr.is_null() {
            unsafe {
                let old_guard = Box::from_raw(old_ptr);
                drop(old_guard);
            }
        }
    }

    // Error tracking layer
    let error_layer = ErrorTrackingLayer::new();

    // Build and initialize the subscriber
    tracing_subscriber::registry()
        .with(env_filter)
        .with(layers)
        .with(error_layer)
        .try_init()
        .map_err(|e| LoggingError::TracingInitFailed(format!("Failed to initialize tracing: {e}")))?;

    tracing::info!(
        app_name = config.app_name,
        log_dir = ?config.log_dir,
        "Tracing initialized"
    );

    Ok(())
}

/// Cleanup tracing resources on shutdown
#[allow(dead_code)]
pub fn cleanup_tracing() {
    // Atomically take the guard pointer
    let guard_ptr = TRACING_GUARD.swap(std::ptr::null_mut(), std::sync::atomic::Ordering::AcqRel);

    // Clean up the guard if it exists
    if !guard_ptr.is_null() {
        unsafe {
            let guard = Box::from_raw(guard_ptr);
            // Guard will be properly dropped here, flushing any remaining logs
            drop(guard);
        }
    }
}

/// Custom layer for tracking errors and sending them to an error reporting
/// service
struct ErrorTrackingLayer {
    error_count: std::sync::atomic::AtomicU64,
}

impl ErrorTrackingLayer {
    const fn new() -> Self {
        Self {
            error_count: std::sync::atomic::AtomicU64::new(0),
        }
    }
}

impl<S> Layer<S> for ErrorTrackingLayer
where
    S: Subscriber,
{
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        // Track error events
        if event.metadata().level() == &Level::ERROR {
            self.error_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

            // In a production app, you might send this to an error tracking service
            // For now, we'll just track locally
            let error_count = self.error_count.load(std::sync::atomic::Ordering::Relaxed);
            if error_count > 0 && error_count % 10 == 0 {
                tracing::warn!("Application has logged {} errors", error_count);
            }
        }
    }
}

/// Structured error reporting
#[derive(Debug)]
#[allow(dead_code)]
pub struct ErrorReport {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub error_type: String,
    pub message: String,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub thread: Option<String>,
    pub backtrace: Option<String>,
}

impl ErrorReport {
    #[allow(dead_code)]
    pub fn new(error: &dyn std::error::Error) -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            error_type: std::any::type_name_of_val(error).to_string(),
            message: error.to_string(),
            file: None,
            line: None,
            thread: std::thread::current().name().map(std::string::ToString::to_string),
            backtrace: std::env::var("RUST_BACKTRACE")
                .ok()
                .filter(|v| v == "1" || v == "full")
                .map(|_| std::backtrace::Backtrace::capture().to_string()),
        }
    }

    /// Convert to JSON for logging or reporting
    #[allow(dead_code)]
    #[must_use]
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "timestamp": self.timestamp.to_rfc3339(),
            "error_type": self.error_type,
            "message": self.message,
            "file": self.file,
            "line": self.line,
            "thread": self.thread,
            "backtrace": self.backtrace,
        })
    }
}

/// Macro for structured error logging
#[macro_export]
macro_rules! log_error_structured {
    ($error:expr) => {{
        let report = $crate::ErrorReport::new(&$error);
        tracing::error!(
            error_type = %report.error_type,
            error_message = %report.message,
            thread = ?report.thread,
            "Structured error occurred"
        );
        report
    }};
    ($error:expr, $($field:tt)*) => {{
        let report = $crate::ErrorReport::new(&$error);
        tracing::error!(
            error_type = %report.error_type,
            error_message = %report.message,
            thread = ?report.thread,
            $($field)*
        );
        report
    }};
}

/// Helper function to log panics
pub fn setup_panic_handler() {
    std::panic::set_hook(Box::new(|panic_info| {
        let location = panic_info.location().map_or_else(
            || "unknown location".to_string(),
            |l| format!("{}:{}:{}", l.file(), l.line(), l.column()),
        );

        let message = panic_info
            .payload()
            .downcast_ref::<&str>()
            .map(|s| (*s).to_string())
            .or_else(|| panic_info.payload().downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "Unknown panic payload".to_string());

        tracing::error!(
            panic.location = %location,
            panic.message = %message,
            panic.thread = ?std::thread::current().name(),
            "Application panicked"
        );
    }));
}

/// Clean up old log files
///
/// # Errors
///
/// Returns an error if the log directory cannot be read or if files cannot be
/// deleted.
#[allow(dead_code)]
pub fn cleanup_old_logs(log_dir: &PathBuf, days_to_keep: u32) -> Result<()> {
    let cutoff = chrono::Utc::now() - chrono::Duration::days(i64::from(days_to_keep));

    for entry in std::fs::read_dir(log_dir)
        .map_err(|e| LoggingError::FileCreationFailed(format!("Failed to read log directory: {e}")))?
    {
        let entry =
            entry.map_err(|e| LoggingError::FileCreationFailed(format!("Failed to read directory entry: {e}")))?;

        let metadata = entry
            .metadata()
            .map_err(|e| LoggingError::FileCreationFailed(format!("Failed to read file metadata: {e}")))?;

        if metadata.is_file() {
            if let Ok(modified) = metadata.modified() {
                let modified_time: chrono::DateTime<chrono::Utc> = modified.into();
                if modified_time < cutoff {
                    let _ = std::fs::remove_file(entry.path());
                }
            }
        }
    }

    Ok(())
}
