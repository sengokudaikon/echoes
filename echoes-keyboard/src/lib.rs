use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

use anyhow::Result;
use echoes_config::{KeyCode, RecordingShortcut, ShortcutMode};
use rdev::{listen, Event, EventType};

pub mod keys;
use keys::rdev_key_to_keycode;

/// Trait for handling keyboard listener errors
trait ErrorHandler {
    fn handle_error(&self, error: &str);
}

/// Default error handler that sends errors through the channel
struct ChannelErrorHandler {
    sender: mpsc::Sender<KeyboardEvent>,
}

impl ErrorHandler for ChannelErrorHandler {
    fn handle_error(&self, error: &str) {
        tracing::error!("Keyboard listener error: {}", error);
        let _ = self.sender.send(KeyboardEvent::ListenerError(error.to_string()));
    }
}

pub enum KeyboardEvent {
    RecordingKeyPressed,
    RecordingKeyReleased,
    OtherKeyPressed, // For cancelling
    ListenerError(String),
    // For shortcut recording
    ShortcutRecorded(RecordingShortcut),
    RecordingCancelled,
}

struct ListenerState {
    pressed_keys: Vec<KeyCode>,
    recording_active: bool,
    recording_shortcut: bool,    // True when recording a new shortcut
    recorded_keys: Vec<KeyCode>, // Keys pressed during recording
}

pub struct KeyboardListener {
    sender: mpsc::Sender<KeyboardEvent>,
    shortcut: Arc<Mutex<RecordingShortcut>>,
    state: Arc<Mutex<ListenerState>>,
}

impl KeyboardListener {
    #[must_use]
    pub fn new(sender: mpsc::Sender<KeyboardEvent>, shortcut: RecordingShortcut) -> Self {
        Self {
            sender,
            shortcut: Arc::new(Mutex::new(shortcut)),
            state: Arc::new(Mutex::new(ListenerState {
                pressed_keys: Vec::new(),
                recording_active: false,
                recording_shortcut: false,
                recorded_keys: Vec::new(),
            })),
        }
    }

    pub fn start_recording_shortcut(&self) {
        if let Ok(mut state) = self.state.lock() {
            state.recording_shortcut = true;
            state.recorded_keys.clear();
            tracing::debug!("Started recording shortcut");
        }
    }

    #[allow(dead_code)]
    pub fn stop_recording_shortcut(&self) {
        if let Ok(mut state) = self.state.lock() {
            state.recording_shortcut = false;
            state.recorded_keys.clear();
            tracing::debug!("Stopped recording shortcut");
        }
    }

    pub fn update_shortcut(&self, new_shortcut: RecordingShortcut) {
        if let Ok(mut shortcut) = self.shortcut.lock() {
            *shortcut = new_shortcut;
            tracing::debug!("Updated shortcut: {:?}", shortcut);
        }
    }

    /// Start listening for keyboard events in a background thread.
    ///
    /// # Errors
    ///
    /// Returns an error if the keyboard listener thread cannot be started or if
    /// platform permissions are insufficient.
    pub fn start_listening(&self) -> Result<()> {
        tracing::debug!("Starting keyboard listener thread");

        let sender = self.sender.clone();
        let shortcut = self.shortcut.clone();
        let state = self.state.clone();

        thread::spawn(move || {
            tracing::debug!("Keyboard listener thread started");

            let error_handler = ChannelErrorHandler { sender: sender.clone() };

            match listen(move |event| {
                handle_event(&event, &sender, &shortcut, &state);
            }) {
                Ok(()) => {
                    tracing::debug!("Keyboard listener exited normally");
                }
                Err(error) => {
                    error_handler.handle_error(&format!(
                        "Keyboard listener failed: {error:?}. This might be due to missing accessibility permissions."
                    ));
                }
            }
        });

        Ok(())
    }
}

fn handle_event(
    event: &Event, sender: &mpsc::Sender<KeyboardEvent>, shortcut: &Arc<Mutex<RecordingShortcut>>,
    state: &Arc<Mutex<ListenerState>>,
) {
    if let Ok(state_guard) = state.lock() {
        if state_guard.recording_shortcut {
            drop(state_guard); // Release the lock before processing
            handle_recording_event(event, sender, state);
            return;
        }
    }

    // Normal shortcut processing
    match event.event_type {
        EventType::KeyPress(key) => {
            if let Some(keycode) = rdev_key_to_keycode(key) {
                if let Ok(mut state) = state.lock() {
                    // Normal operation - track pressed keys
                    if !state.pressed_keys.contains(&keycode) {
                        state.pressed_keys.push(keycode);
                        tracing::debug!("Key pressed: {:?}", keycode);
                    }

                    // Check if shortcut is satisfied
                    if let Ok(shortcut) = shortcut.lock() {
                        if is_shortcut_active(&state.pressed_keys, &shortcut) {
                            match shortcut.mode {
                                ShortcutMode::Hold => {
                                    if !state.recording_active {
                                        state.recording_active = true;
                                        let _ = sender.send(KeyboardEvent::RecordingKeyPressed);
                                    }
                                }
                                ShortcutMode::Toggle => {
                                    if state.recording_active {
                                        state.recording_active = false;
                                        let _ = sender.send(KeyboardEvent::RecordingKeyReleased);
                                    } else {
                                        state.recording_active = true;
                                        let _ = sender.send(KeyboardEvent::RecordingKeyPressed);
                                    }
                                }
                            }
                        } else if state.recording_active && shortcut.mode == ShortcutMode::Hold {
                            // Any other key during hold mode cancels recording
                            state.recording_active = false;
                            let _ = sender.send(KeyboardEvent::OtherKeyPressed);
                        }
                    }
                }
            }
        }
        EventType::KeyRelease(key) => {
            if let Some(keycode) = rdev_key_to_keycode(key) {
                if let Ok(mut state) = state.lock() {
                    // Normal operation - remove from pressed keys
                    state.pressed_keys.retain(|&k| k != keycode);
                    tracing::debug!("Key released: {:?}", keycode);

                    // For hold mode, check if shortcut is no longer active
                    if let Ok(shortcut) = shortcut.lock() {
                        if shortcut.mode == ShortcutMode::Hold
                            && state.recording_active
                            && !is_shortcut_active(&state.pressed_keys, &shortcut)
                        {
                            state.recording_active = false;
                            let _ = sender.send(KeyboardEvent::RecordingKeyReleased);
                        }
                    }
                }
            }
        }
        _ => {}
    }
}

fn handle_recording_event(event: &Event, sender: &mpsc::Sender<KeyboardEvent>, state: &Arc<Mutex<ListenerState>>) {
    match event.event_type {
        EventType::KeyPress(key) => {
            if let Some(keycode) = rdev_key_to_keycode(key) {
                if let Ok(mut state) = state.lock() {
                    tracing::debug!("Recording mode - key pressed: {:?}", keycode);

                    // Cancel on Escape
                    if keycode == KeyCode::Escape {
                        tracing::debug!("Escape pressed, cancelling recording");
                        state.recording_shortcut = false;
                        state.recorded_keys.clear();
                        state.pressed_keys.clear();
                        let _ = sender.send(KeyboardEvent::RecordingCancelled);
                        return;
                    }

                    // Track pressed keys for release detection
                    if !state.pressed_keys.contains(&keycode) {
                        state.pressed_keys.push(keycode);
                    }

                    // Add key to recorded keys if not already there
                    if !state.recorded_keys.contains(&keycode) {
                        state.recorded_keys.push(keycode);
                        tracing::debug!("Recorded key: {:?}", keycode);
                    }
                }
            }
        }
        EventType::KeyRelease(key) => {
            if let Some(keycode) = rdev_key_to_keycode(key) {
                if let Ok(mut state) = state.lock() {
                    tracing::debug!("Recording mode - key released: {:?}", keycode);

                    // Remove from pressed keys
                    state.pressed_keys.retain(|&k| k != keycode);

                    // When all keys are released, finalize the recording
                    if !state.recorded_keys.is_empty() && state.pressed_keys.is_empty() {
                        tracing::debug!(
                            "All keys released, finalizing recording with keys: {:?}",
                            state.recorded_keys
                        );

                        // Create new shortcut from recorded keys
                        let (main_key, modifiers) = extract_shortcut_from_keys(&state.recorded_keys);
                        if let Some(main_key) = main_key {
                            let new_shortcut = RecordingShortcut {
                                mode: ShortcutMode::Hold, // Default to Hold mode
                                key: main_key,
                                modifiers,
                            };
                            tracing::debug!(
                                "Created new shortcut: key={:?}, modifiers={:?}",
                                main_key,
                                &new_shortcut.modifiers
                            );
                            state.recording_shortcut = false;
                            state.recorded_keys.clear();
                            let _ = sender.send(KeyboardEvent::ShortcutRecorded(new_shortcut));
                        } else {
                            tracing::debug!("No main key found in recorded keys");
                        }
                    }
                }
            }
        }
        _ => {}
    }
}

fn extract_shortcut_from_keys(keys: &[KeyCode]) -> (Option<KeyCode>, Vec<KeyCode>) {
    let modifier_keys = [
        KeyCode::ControlLeft,
        KeyCode::ControlRight,
        KeyCode::ShiftLeft,
        KeyCode::ShiftRight,
        KeyCode::Alt,
        KeyCode::AltGr,
        KeyCode::MetaLeft,
        KeyCode::MetaRight,
    ];

    let mut modifiers = Vec::new();
    let mut main_key = None;
    let mut potential_modifier_keys = Vec::new();

    // First pass: separate modifiers from regular keys
    for key in keys {
        if modifier_keys.contains(key) {
            potential_modifier_keys.push(*key);
        } else {
            // Use the last non-modifier key as the main key
            main_key = Some(*key);
        }
    }

    // If we only have modifier keys and no regular key, treat the first modifier as
    // the main key
    if main_key.is_none() && !potential_modifier_keys.is_empty() {
        // Use the first modifier key as the main key
        main_key = Some(potential_modifier_keys[0]);
        // The rest become modifiers
        for key in potential_modifier_keys.iter().skip(1) {
            let normalized = match key {
                KeyCode::ControlLeft | KeyCode::ControlRight => KeyCode::ControlLeft,
                KeyCode::ShiftLeft | KeyCode::ShiftRight => KeyCode::ShiftLeft,
                KeyCode::MetaLeft | KeyCode::MetaRight => KeyCode::MetaLeft,
                _ => *key,
            };
            if !modifiers.contains(&normalized) {
                modifiers.push(normalized);
            }
        }
    } else {
        // Normal case: all modifier keys become modifiers
        for key in potential_modifier_keys {
            let normalized = match key {
                KeyCode::ControlLeft | KeyCode::ControlRight => KeyCode::ControlLeft,
                KeyCode::ShiftLeft | KeyCode::ShiftRight => KeyCode::ShiftLeft,
                KeyCode::MetaLeft | KeyCode::MetaRight => KeyCode::MetaLeft,
                _ => key,
            };
            if !modifiers.contains(&normalized) {
                modifiers.push(normalized);
            }
        }
    }

    (main_key, modifiers)
}

fn is_shortcut_active(pressed_keys: &[KeyCode], shortcut: &RecordingShortcut) -> bool {
    // Check if main key is pressed
    if !pressed_keys.contains(&shortcut.key) {
        return false;
    }

    // Check if all modifiers are pressed
    for modifier in &shortcut.modifiers {
        if !pressed_keys.contains(modifier) {
            return false;
        }
    }

    // For shortcuts with modifiers, ensure no extra modifier keys are pressed
    // This prevents Ctrl+Shift+A from triggering when the shortcut is just Ctrl+A
    if !shortcut.modifiers.is_empty() {
        let modifier_keys = [
            KeyCode::ControlLeft,
            KeyCode::ControlRight,
            KeyCode::ShiftLeft,
            KeyCode::ShiftRight,
            KeyCode::Alt,
            KeyCode::AltGr,
            KeyCode::MetaLeft,
            KeyCode::MetaRight,
        ];

        for key in pressed_keys {
            if modifier_keys.contains(key) && !shortcut.modifiers.contains(key) && *key != shortcut.key {
                return false;
            }
        }
    }

    true
}

/// Type the given text using the system's text input mechanism.
///
/// # Errors
///
/// Returns an error if the text input system cannot be initialized or if text
/// cannot be typed.
#[allow(dead_code)]
pub fn type_text(text: &str) -> Result<()> {
    use enigo::{Enigo, Keyboard, Settings};

    let mut enigo =
        Enigo::new(&Settings::default()).map_err(|e| anyhow::anyhow!("Failed to create Enigo instance: {}", e))?;

    enigo
        .text(text)
        .map_err(|e| anyhow::anyhow!("Failed to type text: {}", e))?;

    Ok(())
}
