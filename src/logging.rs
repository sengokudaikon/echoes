use crate::error::{LoggingError, Result};
use chrono::Local;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{LazyLock, Mutex};
use tracing_subscriber::{EnvFilter, fmt};

static LOG_FILE: LazyLock<Mutex<Option<File>>> = LazyLock::new(|| Mutex::new(None));

#[allow(dead_code)]
pub fn init_logging() -> Result<()> {
    // Set up file logging
    let log_path = PathBuf::from("whispers.log");
    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .map_err(|e| LoggingError::FileCreationFailed(e.to_string()))?;

    *LOG_FILE.lock().map_err(|_| LoggingError::MutexPoisoned)? = Some(log_file);

    // Set up tracing subscriber for console output
    fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(
            "whispers=debug".parse().map_err(|e| {
                LoggingError::FileCreationFailed(format!("Invalid log filter: {e}"))
            })?,
        ))
        .with_target(false)
        .init();

    log_to_file(&format!(
        "=== Whispers started at {} ===",
        Local::now().format("%Y-%m-%d %H:%M:%S")
    ));

    Ok(())
}

pub fn log_to_file(message: &str) {
    match LOG_FILE.lock() {
        Ok(mut guard) => {
            if let Some(ref mut file) = *guard {
                let _ = writeln!(
                    file,
                    "[{}] {}",
                    Local::now().format("%H:%M:%S%.3f"),
                    message
                );
                let _ = file.flush();
            }
        }
        Err(e) => {
            // If mutex is poisoned, attempt to recover by creating a new file
            eprintln!("Log mutex poisoned: {e}. Attempting recovery...");
            // Don't try to fix here to avoid infinite recursion
        }
    }
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        {
            let msg = format!($($arg)*);
            tracing::debug!("{}", &msg);
            $crate::logging::log_to_file(&msg);
        }
    };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        {
            let msg = format!($($arg)*);
            tracing::error!("{}", &msg);
            $crate::logging::log_to_file(&format!("ERROR: {}", &msg));
        }
    };
}
