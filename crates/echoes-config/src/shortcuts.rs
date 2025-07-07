//! Shortcut definitions and key handling

use serde::{Deserialize, Serialize};

/// Key codes for keyboard shortcuts
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum KeyCode {
    // Control keys
    ControlLeft,
    ControlRight,
    ShiftLeft,
    ShiftRight,
    Alt,
    AltGr,
    MetaLeft, // Cmd on Mac, Windows key on Windows
    MetaRight,

    // Special keys
    Space,
    Tab,
    Return,
    Escape,
    Backspace,
    Delete,
    Insert,
    Home,
    End,
    PageUp,
    PageDown,
    CapsLock,

    // Arrow keys
    UpArrow,
    DownArrow,
    LeftArrow,
    RightArrow,

    // Function keys
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,

    // Letters
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,

    // Numbers
    Num0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,

    // Symbols
    Slash,
    BackSlash,
    Equal,
    Minus,
    Comma,
    Dot,
    SemiColon,
    Quote,
    LeftBracket,
    RightBracket,
    BackQuote,
}

/// Shortcut mode for recording
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ShortcutMode {
    Hold,   // Hold key to record
    Toggle, // Press to start/stop
}

/// Recording shortcut configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct RecordingShortcut {
    pub mode: ShortcutMode,
    pub key: KeyCode,            // The main key
    pub modifiers: Vec<KeyCode>, // Additional modifier keys
}

impl RecordingShortcut {
    /// Create a new recording shortcut
    #[must_use]
    pub const fn new(mode: ShortcutMode, key: KeyCode, modifiers: Vec<KeyCode>) -> Self {
        Self { mode, key, modifiers }
    }

    /// Format shortcut for display
    pub fn format_display(&self) -> String {
        let mut parts = Vec::new();

        // Add modifiers in a consistent order
        let mut sorted_modifiers = self.modifiers.clone();
        sorted_modifiers.sort_by_key(modifier_sort_key);

        for modifier in &sorted_modifiers {
            parts.push(format_keycode(modifier));
        }

        // Add main key
        parts.push(format_keycode(&self.key));

        parts.join(" + ")
    }

    /// Validate the shortcut
    pub fn validate(&self) -> std::result::Result<(), crate::validation::ValidationError> {
        crate::validation::validate_shortcut(self)
    }

    /// Check for conflicts with system shortcuts
    #[must_use]
    pub fn check_conflicts(&self) -> Vec<crate::conflict::ConflictInfo> {
        crate::conflict::check_shortcut_conflicts(self)
    }
}

impl Default for RecordingShortcut {
    fn default() -> Self {
        Self {
            mode: ShortcutMode::Hold,
            key: KeyCode::ControlLeft,
            modifiers: vec![],
        }
    }
}

/// Check if a key is a modifier key
#[must_use]
pub const fn is_modifier_key(key: &KeyCode) -> bool {
    matches!(
        key,
        KeyCode::ControlLeft
            | KeyCode::ControlRight
            | KeyCode::ShiftLeft
            | KeyCode::ShiftRight
            | KeyCode::Alt
            | KeyCode::AltGr
            | KeyCode::MetaLeft
            | KeyCode::MetaRight
    )
}

/// Normalize modifier keys (left/right variants to canonical form)
#[must_use]
pub const fn normalize_modifier(key: &KeyCode) -> KeyCode {
    match key {
        KeyCode::ControlLeft | KeyCode::ControlRight => KeyCode::ControlLeft,
        KeyCode::ShiftLeft | KeyCode::ShiftRight => KeyCode::ShiftLeft,
        KeyCode::MetaLeft | KeyCode::MetaRight => KeyCode::MetaLeft,
        _ => *key,
    }
}

/// Get sort key for modifier ordering
const fn modifier_sort_key(key: &KeyCode) -> u8 {
    match normalize_modifier(key) {
        KeyCode::ControlLeft => 1,
        KeyCode::ShiftLeft => 2,
        KeyCode::Alt | KeyCode::AltGr => 3,
        KeyCode::MetaLeft => 4,
        _ => 5,
    }
}

/// Format a keycode for display
#[must_use]
pub fn format_keycode(key: &KeyCode) -> String {
    let result = match key {
        KeyCode::ControlLeft | KeyCode::ControlRight => "Ctrl",
        KeyCode::ShiftLeft | KeyCode::ShiftRight => "Shift",
        KeyCode::Alt => "Alt",
        KeyCode::AltGr => "AltGr",
        KeyCode::MetaLeft | KeyCode::MetaRight => {
            if cfg!(target_os = "macos") {
                "Cmd"
            } else {
                "Win"
            }
        }
        KeyCode::Space => "Space",
        KeyCode::Tab => "Tab",
        KeyCode::Return => "Enter",
        KeyCode::Escape => "Esc",
        KeyCode::Backspace => "Backspace",
        KeyCode::Slash => "/",
        KeyCode::BackSlash => "\\",
        KeyCode::Equal => "=",
        KeyCode::Minus => "-",
        KeyCode::Comma => ",",
        KeyCode::Dot => ".",
        KeyCode::SemiColon => ";",
        KeyCode::Quote => "'",
        KeyCode::LeftBracket => "[",
        KeyCode::RightBracket => "]",
        KeyCode::BackQuote => "`",
        KeyCode::Delete => "Delete",
        KeyCode::Insert => "Insert",
        KeyCode::Home => "Home",
        KeyCode::End => "End",
        KeyCode::PageUp => "PageUp",
        KeyCode::PageDown => "PageDown",
        KeyCode::CapsLock => "CapsLock",
        KeyCode::UpArrow => "↑",
        KeyCode::DownArrow => "↓",
        KeyCode::LeftArrow => "←",
        KeyCode::RightArrow => "→",
        KeyCode::F1 => "F1",
        KeyCode::F2 => "F2",
        KeyCode::F3 => "F3",
        KeyCode::F4 => "F4",
        KeyCode::F5 => "F5",
        KeyCode::F6 => "F6",
        KeyCode::F7 => "F7",
        KeyCode::F8 => "F8",
        KeyCode::F9 => "F9",
        KeyCode::F10 => "F10",
        KeyCode::F11 => "F11",
        KeyCode::F12 => "F12",
        KeyCode::A => "A",
        KeyCode::B => "B",
        KeyCode::C => "C",
        KeyCode::D => "D",
        KeyCode::E => "E",
        KeyCode::F => "F",
        KeyCode::G => "G",
        KeyCode::H => "H",
        KeyCode::I => "I",
        KeyCode::J => "J",
        KeyCode::K => "K",
        KeyCode::L => "L",
        KeyCode::M => "M",
        KeyCode::N => "N",
        KeyCode::O => "O",
        KeyCode::P => "P",
        KeyCode::Q => "Q",
        KeyCode::R => "R",
        KeyCode::S => "S",
        KeyCode::T => "T",
        KeyCode::U => "U",
        KeyCode::V => "V",
        KeyCode::W => "W",
        KeyCode::X => "X",
        KeyCode::Y => "Y",
        KeyCode::Z => "Z",
        KeyCode::Num0 => "0",
        KeyCode::Num1 => "1",
        KeyCode::Num2 => "2",
        KeyCode::Num3 => "3",
        KeyCode::Num4 => "4",
        KeyCode::Num5 => "5",
        KeyCode::Num6 => "6",
        KeyCode::Num7 => "7",
        KeyCode::Num8 => "8",
        KeyCode::Num9 => "9",
    };
    result.to_string()
}

/// Extract shortcut from recorded keys
#[must_use]
pub fn extract_shortcut_from_keys(keys: &[KeyCode]) -> (Option<KeyCode>, Vec<KeyCode>) {
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

    for key in keys {
        if modifier_keys.contains(key) {
            potential_modifier_keys.push(*key);
        } else {
            main_key = Some(*key);
        }
    }

    if main_key.is_none() && !potential_modifier_keys.is_empty() {
        main_key = Some(potential_modifier_keys[0]);
        for key in potential_modifier_keys.iter().skip(1) {
            let normalized = normalize_modifier(key);
            if !modifiers.contains(&normalized) {
                modifiers.push(normalized);
            }
        }
    } else {
        for key in potential_modifier_keys {
            let normalized = normalize_modifier(&key);
            if !modifiers.contains(&normalized) {
                modifiers.push(normalized);
            }
        }
    }

    (main_key, modifiers)
}
