use std::sync::mpsc;

use echoes_config::Config;
use echoes_logging::error;
use tokio::task;

use crate::error::Result;

/// Manages config operations without blocking the UI thread
pub struct ConfigManager {
    save_tx: mpsc::Sender<Config>,
}

impl ConfigManager {
    pub fn new() -> Self {
        let (save_tx, save_rx) = mpsc::channel::<Config>();

        // Spawn a background task to handle config saves
        task::spawn(async move {
            while let Ok(config) = save_rx.recv() {
                if let Err(e) = config.save_async().await {
                    error!("Failed to save config: {e}");
                }
            }
        });

        Self { save_tx }
    }

    /// Queue a config save operation (non-blocking)
    pub fn save_async(&self, config: Config) {
        if let Err(e) = self.save_tx.send(config) {
            error!("Failed to queue config save: {e}");
        }
    }

    /// Synchronous save for critical operations
    #[allow(dead_code)]
    pub fn save_sync(config: &Config) -> Result<()> {
        config
            .save()
            .map_err(|e| crate::error::EchoesError::Other(e.to_string()))
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}
