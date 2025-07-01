use eframe::egui;

mod audio;
mod config;
mod error;
mod keyboard;
mod logging;
mod permissions;
mod stt;
mod tracing_setup;
mod ui;

use config::Config;
use error::{Result, UiError, WhispersError};
use tracing_setup::{TracingConfig, init_tracing, setup_panic_handler};

fn main() -> Result<()> {
    // Set up panic handler
    setup_panic_handler();

    // Initialize tracing with default config
    let tracing_config = TracingConfig::default();
    init_tracing(tracing_config)?;

    // Load configuration
    let config =
        Config::load().map_err(|e| WhispersError::Other(format!("Failed to load config: {e}")))?;

    // Set up native options for the window
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([600.0, 400.0]),
        centered: true,
        ..Default::default()
    };

    // Run the app
    eframe::run_native(
        "Whispo",
        native_options,
        Box::new(|cc| Ok(Box::new(ui::WhispoApp::new(cc, config)))),
    )
    .map_err(|e| UiError::InitializationFailed(e.to_string()).into())
}
