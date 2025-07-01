use rdev::{listen, Event, EventType};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use anyhow::Result;
use crate::{log_debug, log_error};
use crate::config::{RecordingShortcut, ShortcutMode, KeyCode};

mod keys;
use keys::rdev_key_to_keycode;

pub enum KeyboardEvent {
    RecordingKeyPressed,
    RecordingKeyReleased,
    OtherKeyPressed, // For cancelling
    ListenerError(String),
}

struct ListenerState {
    pressed_keys: Vec<KeyCode>,
    recording_active: bool,
}

pub struct KeyboardListener {
    sender: mpsc::Sender<KeyboardEvent>,
    shortcut: RecordingShortcut,
    state: Arc<Mutex<ListenerState>>,
}

impl KeyboardListener {
    pub fn new(sender: mpsc::Sender<KeyboardEvent>, shortcut: RecordingShortcut) -> Self {
        Self { 
            sender,
            shortcut,
            state: Arc::new(Mutex::new(ListenerState {
                pressed_keys: Vec::new(),
                recording_active: false,
            })),
        }
    }
    
    pub fn start_listening(self) -> Result<()> {
        log_debug!("Starting keyboard listener thread");
        
        let sender = self.sender.clone();
        thread::spawn(move || {
            log_debug!("Keyboard listener thread started");
            
            // Set up panic hook to catch details
            let orig_hook = std::panic::take_hook();
            std::panic::set_hook(Box::new(move |panic_info| {
                log_error!("Panic in keyboard thread: {:?}", panic_info);
                orig_hook(panic_info);
            }));
            
            // Wrap the listen call in a panic catch to handle the crash
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let sender = self.sender.clone();
                let shortcut = self.shortcut;
                let state = self.state.clone();
                
                listen(move |event| {
                    handle_event(event, &sender, &shortcut, &state);
                })
            }));
            
            match result {
                Ok(Ok(())) => {
                    log_debug!("Keyboard listener exited normally");
                }
                Ok(Err(error)) => {
                    log_error!("Keyboard listener error: {:?}", error);
                    let _ = sender.send(KeyboardEvent::ListenerError(
                        format!("Keyboard listener error: {:?}", error)
                    ));
                }
                Err(panic) => {
                    let msg = if let Some(s) = panic.downcast_ref::<&str>() {
                        s.to_string()
                    } else if let Some(s) = panic.downcast_ref::<String>() {
                        s.clone()
                    } else {
                        "Unknown panic in keyboard listener".to_string()
                    };
                    log_error!("Keyboard listener panicked: {}", msg);
                    let _ = sender.send(KeyboardEvent::ListenerError(
                        format!("Keyboard listener crashed. This might be due to macOS security restrictions. Try running from Terminal.app with accessibility permissions.")
                    ));
                }
            }
        });
        
        Ok(())
    }
    
}

fn handle_event(event: Event, sender: &mpsc::Sender<KeyboardEvent>, shortcut: &RecordingShortcut, state: &Arc<Mutex<ListenerState>>) {
    match event.event_type {
        EventType::KeyPress(key) => {
            if let Some(keycode) = rdev_key_to_keycode(key) {
                if let Ok(mut state) = state.lock() {
                    // Add key to pressed keys if not already there
                    if !state.pressed_keys.contains(&keycode) {
                        state.pressed_keys.push(keycode);
                        log_debug!("Key pressed: {:?}", keycode);
                    }
                    
                    // Check if shortcut is satisfied
                    if is_shortcut_active(&state.pressed_keys, shortcut) {
                        match shortcut.mode {
                            ShortcutMode::Hold => {
                                if !state.recording_active {
                                    state.recording_active = true;
                                    let _ = sender.send(KeyboardEvent::RecordingKeyPressed);
                                }
                            }
                            ShortcutMode::Toggle => {
                                if !state.recording_active {
                                    state.recording_active = true;
                                    let _ = sender.send(KeyboardEvent::RecordingKeyPressed);
                                } else {
                                    state.recording_active = false;
                                    let _ = sender.send(KeyboardEvent::RecordingKeyReleased);
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
        EventType::KeyRelease(key) => {
            if let Some(keycode) = rdev_key_to_keycode(key) {
                if let Ok(mut state) = state.lock() {
                    // Remove key from pressed keys
                    state.pressed_keys.retain(|&k| k != keycode);
                    log_debug!("Key released: {:?}", keycode);
                    
                    // For hold mode, check if shortcut is no longer active
                    if shortcut.mode == ShortcutMode::Hold && state.recording_active {
                        if !is_shortcut_active(&state.pressed_keys, shortcut) {
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
            KeyCode::ControlLeft, KeyCode::ControlRight,
            KeyCode::ShiftLeft, KeyCode::ShiftRight,
            KeyCode::Alt, KeyCode::AltGr,
            KeyCode::MetaLeft, KeyCode::MetaRight,
        ];
        
        for key in pressed_keys {
            if modifier_keys.contains(key) && !shortcut.modifiers.contains(key) && *key != shortcut.key {
                return false;
            }
        }
    }
    
    true
}

// Text output functionality from existing code
pub fn type_text(text: &str) -> Result<()> {
    use enigo::{Enigo, Keyboard, Settings};
    
    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| anyhow::anyhow!("Failed to create Enigo instance: {}", e))?;
    
    enigo.text(text)
        .map_err(|e| anyhow::anyhow!("Failed to type text: {}", e))?;
    
    Ok(())
}