use echoes_config::Config;
use eframe::egui;
use tracing::info;

pub mod error;
pub mod ui;

use echoes_logging::{TracingConfig, init_tracing, setup_panic_handler};
use error::{EchoesError, Result, UiError};

/// Runs the main application loop
///
/// # Errors
///
/// Returns an error if:
/// - Configuration cannot be loaded
/// - UI initialization fails
/// - eframe native window creation fails
pub async fn run() -> Result<()> {
    setup_panic_handler();

    let tracing_config = TracingConfig::default();
    init_tracing(&tracing_config)?;

    let config = Config::load().map_err(|e| EchoesError::Other(format!("Failed to load config: {e}")))?;

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([600.0, 400.0]),
        centered: true,
        ..Default::default()
    };

    info!("About to initialize eframe");

    eframe::run_native(
        "Whispo",
        native_options,
        Box::new(|cc| {
            info!("Creating WhispoApp");
            Ok(Box::new(ui::WhispoApp::new(cc, config)))
        }),
    )
    .map_err(|e| UiError::InitializationFailed(e.to_string()).into())
}
