use eframe::egui;
use anyhow::Result;

mod config;
mod keyboard;
mod audio;
mod stt;
mod ui;
mod logging;
mod permissions;

use config::Config;

fn main() -> Result<()> {
    // Initialize logging
    logging::init_logging()?;
    
    // Load configuration
    let config = Config::load()?;

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
        Box::new(|cc| Box::new(ui::WhispoApp::new(cc, config))),
    )
    .map_err(|e| anyhow::anyhow!("Failed to run native app: {}", e))
}
