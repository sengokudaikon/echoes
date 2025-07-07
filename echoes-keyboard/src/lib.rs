use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

use anyhow::Result;
use echoes_config::{is_modifier_key, KeyCode, RecordingShortcut, ShortcutMode};
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
    OtherKeyPressed,
    ListenerError(String),
    ShortcutRecorded(RecordingShortcut),
    RecordingCancelled,
}

struct ListenerState {
    pressed_keys: Vec<KeyCode>,
    recording_active: bool,
    recording_shortcut: bool,
    recorded_keys: Vec<KeyCode>,
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
            drop(state_guard);
            handle_recording_event(event, sender, state);
            return;
        }
    }

    match event.event_type {
        EventType::KeyPress(key) => {
            if let Some(keycode) = rdev_key_to_keycode(key) {
                handle_key_press(keycode, sender, shortcut, state);
            }
        }
        EventType::KeyRelease(key) => {
            if let Some(keycode) = rdev_key_to_keycode(key) {
                handle_key_release(keycode, sender, shortcut, state);
            }
        }
        _ => {}
    }
}

fn handle_key_press(
    keycode: KeyCode, sender: &mpsc::Sender<KeyboardEvent>, shortcut: &Arc<Mutex<RecordingShortcut>>,
    state: &Arc<Mutex<ListenerState>>,
) {
    if let Ok(mut state) = state.lock() {
        if !state.pressed_keys.contains(&keycode) {
            state.pressed_keys.push(keycode);
            tracing::debug!("Key pressed: {:?}", keycode);
        }

        if let Ok(shortcut) = shortcut.lock() {
            if is_shortcut_active(&state.pressed_keys, &shortcut) {
                handle_shortcut_activation(&mut state, &shortcut, sender);
            } else if state.recording_active && shortcut.mode == ShortcutMode::Hold {
                // Any other key during hold mode cancels recording
                state.recording_active = false;
                let _ = sender.send(KeyboardEvent::OtherKeyPressed);
            }
        }
    }
}

fn handle_key_release(
    keycode: KeyCode, sender: &mpsc::Sender<KeyboardEvent>, shortcut: &Arc<Mutex<RecordingShortcut>>,
    state: &Arc<Mutex<ListenerState>>,
) {
    if let Ok(mut state) = state.lock() {
        state.pressed_keys.retain(|&k| k != keycode);
        tracing::debug!("Key released: {:?}", keycode);

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

fn handle_shortcut_activation(
    state: &mut ListenerState, shortcut: &RecordingShortcut, sender: &mpsc::Sender<KeyboardEvent>,
) {
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
}

fn handle_recording_event(event: &Event, sender: &mpsc::Sender<KeyboardEvent>, state: &Arc<Mutex<ListenerState>>) {
    match event.event_type {
        EventType::KeyPress(key) => {
            if let Some(keycode) = rdev_key_to_keycode(key) {
                handle_recording_key_press(keycode, sender, state);
            }
        }
        EventType::KeyRelease(key) => {
            if let Some(keycode) = rdev_key_to_keycode(key) {
                handle_recording_key_release(keycode, sender, state);
            }
        }
        _ => {}
    }
}

fn handle_recording_key_press(
    keycode: KeyCode, sender: &mpsc::Sender<KeyboardEvent>, state: &Arc<Mutex<ListenerState>>,
) {
    if let Ok(mut state) = state.lock() {
        tracing::debug!("Recording mode - key pressed: {:?}", keycode);

        if keycode == KeyCode::Escape {
            cancel_recording(&mut state, sender);
            return;
        }

        if !state.pressed_keys.contains(&keycode) {
            state.pressed_keys.push(keycode);
        }

        if !state.recorded_keys.contains(&keycode) {
            state.recorded_keys.push(keycode);
            tracing::debug!("Recorded key: {:?}", keycode);
        }
    }
}

fn handle_recording_key_release(
    keycode: KeyCode, sender: &mpsc::Sender<KeyboardEvent>, state: &Arc<Mutex<ListenerState>>,
) {
    if let Ok(mut state) = state.lock() {
        tracing::debug!("Recording mode - key released: {:?}", keycode);

        state.pressed_keys.retain(|&k| k != keycode);

        if !state.recorded_keys.is_empty() && state.pressed_keys.is_empty() {
            finalize_recording(&mut state, sender);
        }
    }
}

fn cancel_recording(state: &mut ListenerState, sender: &mpsc::Sender<KeyboardEvent>) {
    tracing::debug!("Escape pressed, cancelling recording");
    state.recording_shortcut = false;
    state.recorded_keys.clear();
    state.pressed_keys.clear();
    let _ = sender.send(KeyboardEvent::RecordingCancelled);
}

fn finalize_recording(state: &mut ListenerState, sender: &mpsc::Sender<KeyboardEvent>) {
    tracing::debug!(
        "All keys released, finalizing recording with keys: {:?}",
        state.recorded_keys
    );

    let (main_key, modifiers) = extract_shortcut_from_keys(&state.recorded_keys);
    if let Some(main_key) = main_key {
        let new_shortcut = RecordingShortcut {
            mode: ShortcutMode::Hold,
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

fn extract_shortcut_from_keys(keys: &[KeyCode]) -> (Option<KeyCode>, Vec<KeyCode>) {
    let (main_key, modifier_keys) = separate_main_and_modifier_keys(keys);
    let normalized_modifiers = normalize_modifier_keys(&modifier_keys);

    (main_key, normalized_modifiers)
}

fn separate_main_and_modifier_keys(keys: &[KeyCode]) -> (Option<KeyCode>, Vec<KeyCode>) {
    let mut main_key = None;
    let mut modifier_keys = Vec::new();

    for key in keys {
        if is_modifier_key(key) {
            modifier_keys.push(*key);
        } else {
            main_key = Some(*key);
        }
    }

    if main_key.is_none() && !modifier_keys.is_empty() {
        main_key = Some(modifier_keys[0]);
        modifier_keys.remove(0);
    }

    (main_key, modifier_keys)
}

fn normalize_modifier_keys(modifier_keys: &[KeyCode]) -> Vec<KeyCode> {
    let mut normalized = Vec::new();

    for key in modifier_keys {
        let normalized_key = normalize_modifier_key(*key);
        if !normalized.contains(&normalized_key) {
            normalized.push(normalized_key);
        }
    }

    normalized
}

const fn normalize_modifier_key(key: KeyCode) -> KeyCode {
    match key {
        KeyCode::ControlLeft | KeyCode::ControlRight => KeyCode::ControlLeft,
        KeyCode::ShiftLeft | KeyCode::ShiftRight => KeyCode::ShiftLeft,
        KeyCode::MetaLeft | KeyCode::MetaRight => KeyCode::MetaLeft,
        _ => key,
    }
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
