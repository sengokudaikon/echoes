use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Mutex, LazyLock};
use chrono::Local;
use tracing_subscriber::{fmt, EnvFilter};

static LOG_FILE: LazyLock<Mutex<Option<File>>> = LazyLock::new(|| Mutex::new(None));

pub fn init_logging() -> anyhow::Result<()> {
    // Set up file logging
    let log_path = PathBuf::from("whispo-rust.log");
    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)?;
    
    *LOG_FILE.lock().unwrap() = Some(log_file);
    
    // Set up tracing subscriber for console output
    fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("whispo_rust=debug".parse()?))
        .with_target(false)
        .init();
    
    log_to_file(&format!("=== Whispo Rust started at {} ===", Local::now().format("%Y-%m-%d %H:%M:%S")));
    
    Ok(())
}

pub fn log_to_file(message: &str) {
    if let Ok(mut guard) = LOG_FILE.lock() {
        if let Some(ref mut file) = *guard {
            let _ = writeln!(file, "[{}] {}", Local::now().format("%H:%M:%S%.3f"), message);
            let _ = file.flush();
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