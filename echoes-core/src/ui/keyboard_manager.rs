use std::sync::mpsc;

use echoes_config::RecordingShortcut;
use echoes_keyboard::{KeyboardEvent, KeyboardListener};

/// Manages keyboard events and listener
pub struct KeyboardManager {
    pub listener: Option<std::sync::Arc<KeyboardListener>>,
    pub event_rx: Option<mpsc::Receiver<KeyboardEvent>>,
    pub permissions_granted: bool,
}

impl KeyboardManager {
    pub fn new() -> Self {
        Self {
            listener: None,
            event_rx: None,
            permissions_granted: false,
        }
    }

    pub fn init(&mut self, shortcut: RecordingShortcut) -> Result<(), String> {
        match echoes_platform::ensure_permissions() {
            Ok(true) => {
                self.permissions_granted = true;

                // Set up keyboard listener
                let (tx, rx) = mpsc::channel();
                let listener = KeyboardListener::new(tx.clone(), shortcut);
                let listener_arc = std::sync::Arc::new(listener);

                if let Err(e) = listener_arc.start_listening() {
                    return Err(format!("Failed to start keyboard listener: {e}"));
                }

                self.event_rx = Some(rx);
                self.listener = Some(listener_arc);
                Ok(())
            }
            Ok(false) => {
                self.permissions_granted = false;
                Err("Permissions not granted".into())
            }
            Err(e) => {
                self.permissions_granted = false;
                Err(e.to_string())
            }
        }
    }

    pub fn update_shortcut(&self, shortcut: RecordingShortcut) {
        if let Some(listener) = &self.listener {
            listener.update_shortcut(shortcut);
        }
    }

    pub fn start_recording_shortcut(&self) {
        if let Some(listener) = &self.listener {
            listener.start_recording_shortcut();
        }
    }

    pub fn stop_recording_shortcut(&self) {
        if let Some(listener) = &self.listener {
            listener.stop_recording_shortcut();
        }
    }

    pub fn try_recv_event(&self) -> Vec<KeyboardEvent> {
        let mut events = Vec::new();
        if let Some(rx) = &self.event_rx {
            while let Ok(event) = rx.try_recv() {
                events.push(event);
            }
        }
        events
    }

    pub fn clear_receiver(&mut self) {
        self.event_rx = None;
    }
}

impl Default for KeyboardManager {
    fn default() -> Self {
        Self::new()
    }
}
