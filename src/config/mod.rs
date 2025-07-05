use crate::error::{ConfigError, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    // STT Provider settings
    pub stt_provider: SttProvider,

    // API Keys
    pub openai_api_key: Option<String>,
    pub groq_api_key: Option<String>,

    // API URLs (for custom deployments)
    pub openai_base_url: Option<String>,
    pub groq_base_url: Option<String>,

    // Lightning Whisper settings (Mac only)
    #[cfg(target_os = "macos")]
    pub lightning_whisper: LightningWhisperConfig,

    // Local Whisper settings (whisper-rs)
    pub local_whisper: LocalWhisperConfig,

    // Recording settings
    pub recording_shortcut: RecordingShortcut,

    // Post-processing
    pub post_processing: PostProcessingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SttProvider {
    OpenAI,
    Groq,
    LocalWhisper,
    #[cfg(target_os = "macos")]
    LightningWhisper,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightningWhisperConfig {
    pub model: String,
    pub batch_size: u32,
    pub quantization: Option<String>, // "4bit", "8bit", or None
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalWhisperConfig {
    pub model: WhisperModel,
    pub model_path: Option<PathBuf>, // Custom model path, if not using auto-download
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WhisperModel {
    Tiny,
    TinyEn,
    Base,
    BaseEn,
    Small,
    SmallEn,
    Medium,
    MediumEn,
    LargeV1,
    LargeV2,
    LargeV3,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RecordingShortcut {
    pub mode: ShortcutMode,
    pub key: KeyCode,            // The main key
    pub modifiers: Vec<KeyCode>, // Additional modifier keys
}

impl RecordingShortcut {
    pub fn validate(&self) -> std::result::Result<(), ValidationError> {
        // Allow single modifier keys as shortcuts (like Ctrl for recording)
        // Only reject if we have modifiers but the main key is also a modifier
        if is_modifier_key(&self.key) && !self.modifiers.is_empty() {
            return Err(ValidationError::ModifierOnly);
        }

        // Check for duplicate modifiers
        let mut seen_modifiers = std::collections::HashSet::new();
        for modifier in &self.modifiers {
            let normalized = normalize_modifier(modifier);
            if !seen_modifiers.insert(normalized) {
                return Err(ValidationError::DuplicateModifiers);
            }
        }

        // Check for system conflicts
        if let Some(conflict) = check_system_conflict(self) {
            return Err(ValidationError::SystemConflict(conflict));
        }

        Ok(())
    }

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

    pub fn check_conflicts(&self) -> Vec<ConflictInfo> {
        let mut conflicts = Vec::new();

        // Check system conflicts with error severity
        if let Some(system_conflict) = check_system_conflict(self) {
            conflicts.push(ConflictInfo {
                severity: ConflictSeverity::Error,
                description: system_conflict,
                suggestion: Some("Choose a different key combination".to_string()),
            });
        }

        // Check common application conflicts with warning severity
        if let Some(app_conflict) = check_application_conflict(self) {
            conflicts.push(app_conflict);
        }

        // Check accessibility concerns with info severity
        if let Some(accessibility_info) = check_accessibility_concerns(self) {
            conflicts.push(accessibility_info);
        }

        conflicts
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictSeverity {
    Error,   // System shortcut that cannot be overridden
    Warning, // Common application shortcut
    Info,    // Potential conflict with specific apps
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConflictInfo {
    pub severity: ConflictSeverity,
    pub description: String,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum ValidationError {
    EmptyShortcut,
    ModifierOnly,
    DuplicateModifiers,
    SystemConflict(String),
    ConflictsDetected(Vec<ConflictInfo>),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::EmptyShortcut => write!(f, "Shortcut cannot be empty"),
            ValidationError::ModifierOnly => write!(f, "Shortcut cannot be only a modifier key"),
            ValidationError::DuplicateModifiers => write!(f, "Duplicate modifier keys detected"),
            ValidationError::SystemConflict(desc) => {
                write!(f, "Conflicts with system shortcut: {desc}")
            }
            ValidationError::ConflictsDetected(conflicts) => {
                write!(f, "Conflicts detected: ")?;
                for (i, conflict) in conflicts.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", conflict.description)?;
                }
                Ok(())
            }
        }
    }
}

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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ShortcutMode {
    Hold,   // Hold key to record
    Toggle, // Press to start/stop
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostProcessingConfig {
    pub enabled: bool,
    pub provider: LlmProvider,
    pub model: String,
    pub prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LlmProvider {
    OpenAI,
    Groq,
    Gemini,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            stt_provider: SttProvider::OpenAI,
            openai_api_key: None,
            groq_api_key: None,
            openai_base_url: Some("https://api.openai.com/v1".to_string()),
            groq_base_url: Some("https://api.groq.com/openai/v1".to_string()),
            #[cfg(target_os = "macos")]
            lightning_whisper: LightningWhisperConfig {
                model: "distil-medium.en".to_string(),
                batch_size: 12,
                quantization: None,
            },
            local_whisper: LocalWhisperConfig {
                model: WhisperModel::Base,
                model_path: None,
            },
            recording_shortcut: RecordingShortcut {
                mode: ShortcutMode::Hold,
                key: KeyCode::ControlLeft,
                modifiers: vec![],
            },
            post_processing: PostProcessingConfig {
                enabled: false,
                provider: LlmProvider::OpenAI,
                model: "gpt-4o-mini".to_string(),
                prompt: "Clean up the following transcript, fixing any errors and improving clarity while preserving the original meaning:\n\n{transcript}".to_string(),
            },
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)
                .map_err(|e| ConfigError::LoadFailed(format!("Failed to read config file: {e}")))?;
            let config: Config = toml::from_str(&content)
                .map_err(|e| ConfigError::ParseError(format!("Invalid config format: {e}")))?;
            Ok(config)
        } else {
            // Create default config
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;

        // Create directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                ConfigError::SaveFailed(format!("Failed to create config directory: {e}"))
            })?;
        }

        let content = toml::to_string_pretty(self)
            .map_err(|e| ConfigError::SaveFailed(format!("Failed to serialize config: {e}")))?;
        std::fs::write(&config_path, content)
            .map_err(|e| ConfigError::SaveFailed(format!("Failed to write config file: {e}")))?;

        Ok(())
    }

    /// Async version of save to avoid blocking the UI thread
    pub async fn save_async(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        let content = toml::to_string_pretty(self)
            .map_err(|e| ConfigError::SaveFailed(format!("Failed to serialize config: {e}")))?;

        // Clone data to move into the blocking task
        let config_path = config_path.clone();
        let content = content.clone();

        tokio::task::spawn_blocking(move || {
            // Create directory if it doesn't exist
            if let Some(parent) = config_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    ConfigError::SaveFailed(format!("Failed to create config directory: {e}"))
                })?;
            }

            std::fs::write(&config_path, content).map_err(|e| {
                ConfigError::SaveFailed(format!("Failed to write config file: {e}"))
            })?;

            Ok::<(), crate::error::EchoesError>(())
        })
        .await
        .map_err(|e| ConfigError::SaveFailed(format!("Task join error: {e}")))?
    }

    fn config_path() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("com", "echoes", "echoes").ok_or_else(|| {
            ConfigError::LoadFailed("Failed to determine config directory".to_string())
        })?;

        Ok(proj_dirs.config_dir().join("config.toml"))
    }
}

// Helper functions for shortcut validation
fn is_modifier_key(key: &KeyCode) -> bool {
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

fn normalize_modifier(key: &KeyCode) -> KeyCode {
    match key {
        KeyCode::ControlLeft | KeyCode::ControlRight => KeyCode::ControlLeft,
        KeyCode::ShiftLeft | KeyCode::ShiftRight => KeyCode::ShiftLeft,
        KeyCode::MetaLeft | KeyCode::MetaRight => KeyCode::MetaLeft,
        _ => *key,
    }
}

fn modifier_sort_key(key: &KeyCode) -> u8 {
    match normalize_modifier(key) {
        KeyCode::ControlLeft => 1,
        KeyCode::ShiftLeft => 2,
        KeyCode::Alt | KeyCode::AltGr => 3,
        KeyCode::MetaLeft => 4,
        _ => 5,
    }
}

fn format_keycode(key: &KeyCode) -> String {
    match key {
        KeyCode::ControlLeft | KeyCode::ControlRight => "Ctrl".to_string(),
        KeyCode::ShiftLeft | KeyCode::ShiftRight => "Shift".to_string(),
        KeyCode::Alt => "Alt".to_string(),
        KeyCode::AltGr => "AltGr".to_string(),
        KeyCode::MetaLeft | KeyCode::MetaRight => {
            if cfg!(target_os = "macos") {
                "Cmd".to_string()
            } else {
                "Win".to_string()
            }
        }
        KeyCode::Space => "Space".to_string(),
        KeyCode::Tab => "Tab".to_string(),
        KeyCode::Return => "Enter".to_string(),
        KeyCode::Escape => "Esc".to_string(),
        KeyCode::Backspace => "Backspace".to_string(),
        KeyCode::Slash => "/".to_string(),
        KeyCode::BackSlash => "\\".to_string(),
        KeyCode::Equal => "=".to_string(),
        KeyCode::Minus => "-".to_string(),
        KeyCode::Comma => ",".to_string(),
        KeyCode::Dot => ".".to_string(),
        KeyCode::SemiColon => ";".to_string(),
        KeyCode::Quote => "'".to_string(),
        KeyCode::LeftBracket => "[".to_string(),
        KeyCode::RightBracket => "]".to_string(),
        KeyCode::BackQuote => "`".to_string(),
        KeyCode::Delete => "Delete".to_string(),
        KeyCode::Insert => "Insert".to_string(),
        KeyCode::Home => "Home".to_string(),
        KeyCode::End => "End".to_string(),
        KeyCode::PageUp => "PageUp".to_string(),
        KeyCode::PageDown => "PageDown".to_string(),
        KeyCode::CapsLock => "CapsLock".to_string(),
        KeyCode::UpArrow => "↑".to_string(),
        KeyCode::DownArrow => "↓".to_string(),
        KeyCode::LeftArrow => "←".to_string(),
        KeyCode::RightArrow => "→".to_string(),
        KeyCode::F1 => "F1".to_string(),
        KeyCode::F2 => "F2".to_string(),
        KeyCode::F3 => "F3".to_string(),
        KeyCode::F4 => "F4".to_string(),
        KeyCode::F5 => "F5".to_string(),
        KeyCode::F6 => "F6".to_string(),
        KeyCode::F7 => "F7".to_string(),
        KeyCode::F8 => "F8".to_string(),
        KeyCode::F9 => "F9".to_string(),
        KeyCode::F10 => "F10".to_string(),
        KeyCode::F11 => "F11".to_string(),
        KeyCode::F12 => "F12".to_string(),
        KeyCode::A => "A".to_string(),
        KeyCode::B => "B".to_string(),
        KeyCode::C => "C".to_string(),
        KeyCode::D => "D".to_string(),
        KeyCode::E => "E".to_string(),
        KeyCode::F => "F".to_string(),
        KeyCode::G => "G".to_string(),
        KeyCode::H => "H".to_string(),
        KeyCode::I => "I".to_string(),
        KeyCode::J => "J".to_string(),
        KeyCode::K => "K".to_string(),
        KeyCode::L => "L".to_string(),
        KeyCode::M => "M".to_string(),
        KeyCode::N => "N".to_string(),
        KeyCode::O => "O".to_string(),
        KeyCode::P => "P".to_string(),
        KeyCode::Q => "Q".to_string(),
        KeyCode::R => "R".to_string(),
        KeyCode::S => "S".to_string(),
        KeyCode::T => "T".to_string(),
        KeyCode::U => "U".to_string(),
        KeyCode::V => "V".to_string(),
        KeyCode::W => "W".to_string(),
        KeyCode::X => "X".to_string(),
        KeyCode::Y => "Y".to_string(),
        KeyCode::Z => "Z".to_string(),
        KeyCode::Num0 => "0".to_string(),
        KeyCode::Num1 => "1".to_string(),
        KeyCode::Num2 => "2".to_string(),
        KeyCode::Num3 => "3".to_string(),
        KeyCode::Num4 => "4".to_string(),
        KeyCode::Num5 => "5".to_string(),
        KeyCode::Num6 => "6".to_string(),
        KeyCode::Num7 => "7".to_string(),
        KeyCode::Num8 => "8".to_string(),
        KeyCode::Num9 => "9".to_string(),
    }
}

fn check_system_conflict(shortcut: &RecordingShortcut) -> Option<String> {
    // Platform-specific system shortcut checking
    #[cfg(target_os = "macos")]
    {
        // Check common macOS system shortcuts
        let has_cmd = shortcut
            .modifiers
            .iter()
            .any(|k| matches!(k, KeyCode::MetaLeft | KeyCode::MetaRight));

        if has_cmd {
            match &shortcut.key {
                KeyCode::Q => return Some("Cmd+Q quits applications".to_string()),
                KeyCode::W => return Some("Cmd+W closes windows".to_string()),
                KeyCode::H => return Some("Cmd+H hides applications".to_string()),
                KeyCode::M => return Some("Cmd+M minimizes windows".to_string()),
                KeyCode::Tab => return Some("Cmd+Tab switches applications".to_string()),
                KeyCode::Space => {
                    if shortcut.modifiers.len() == 1 {
                        return Some("Cmd+Space opens Spotlight search".to_string());
                    }
                }
                _ => {}
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Check common Windows system shortcuts
        let has_win = shortcut
            .modifiers
            .iter()
            .any(|k| matches!(k, KeyCode::MetaLeft | KeyCode::MetaRight));
        let has_alt = shortcut
            .modifiers
            .iter()
            .any(|k| matches!(k, KeyCode::Alt | KeyCode::AltGr));

        if has_win {
            match &shortcut.key {
                KeyCode::L => return Some("Win+L locks the computer".to_string()),
                KeyCode::D => return Some("Win+D shows desktop".to_string()),
                KeyCode::Tab => return Some("Win+Tab opens Task View".to_string()),
                _ => {}
            }
        }

        if has_alt && shortcut.key == KeyCode::Tab {
            return Some("Alt+Tab switches windows".to_string());
        }
    }

    // Cross-platform shortcuts
    let has_ctrl = shortcut
        .modifiers
        .iter()
        .any(|k| matches!(k, KeyCode::ControlLeft | KeyCode::ControlRight));
    let has_alt = shortcut
        .modifiers
        .iter()
        .any(|k| matches!(k, KeyCode::Alt | KeyCode::AltGr));

    if has_ctrl && has_alt && shortcut.key == KeyCode::Delete {
        return Some("Ctrl+Alt+Delete is a system shortcut".to_string());
    }

    None
}

fn check_application_conflict(shortcut: &RecordingShortcut) -> Option<ConflictInfo> {
    let has_ctrl = shortcut
        .modifiers
        .iter()
        .any(|k| matches!(k, KeyCode::ControlLeft | KeyCode::ControlRight));
    let has_cmd = shortcut
        .modifiers
        .iter()
        .any(|k| matches!(k, KeyCode::MetaLeft | KeyCode::MetaRight));
    let has_shift = shortcut
        .modifiers
        .iter()
        .any(|k| matches!(k, KeyCode::ShiftLeft | KeyCode::ShiftRight));

    // Common editor shortcuts
    if has_ctrl || has_cmd {
        match &shortcut.key {
            KeyCode::S => Some(ConflictInfo {
                severity: ConflictSeverity::Warning,
                description: "Conflicts with Save in most applications".to_string(),
                suggestion: Some(
                    "Consider adding another modifier or using a different key".to_string(),
                ),
            }),
            KeyCode::C => Some(ConflictInfo {
                severity: ConflictSeverity::Warning,
                description: "Conflicts with Copy in most applications".to_string(),
                suggestion: Some("This will prevent copying while recording".to_string()),
            }),
            KeyCode::V => Some(ConflictInfo {
                severity: ConflictSeverity::Warning,
                description: "Conflicts with Paste in most applications".to_string(),
                suggestion: Some("This will prevent pasting while recording".to_string()),
            }),
            KeyCode::X => Some(ConflictInfo {
                severity: ConflictSeverity::Warning,
                description: "Conflicts with Cut in most applications".to_string(),
                suggestion: Some("This will prevent cutting while recording".to_string()),
            }),
            KeyCode::Z => Some(ConflictInfo {
                severity: ConflictSeverity::Warning,
                description: "Conflicts with Undo in most applications".to_string(),
                suggestion: Some("This will prevent undoing while recording".to_string()),
            }),
            KeyCode::A => Some(ConflictInfo {
                severity: ConflictSeverity::Warning,
                description: "Conflicts with Select All in most applications".to_string(),
                suggestion: Some("This will prevent selecting all while recording".to_string()),
            }),
            KeyCode::F => Some(ConflictInfo {
                severity: ConflictSeverity::Warning,
                description: "Conflicts with Find in most applications".to_string(),
                suggestion: Some("This will prevent searching while recording".to_string()),
            }),
            KeyCode::N => Some(ConflictInfo {
                severity: ConflictSeverity::Warning,
                description: "Conflicts with New in most applications".to_string(),
                suggestion: Some(
                    "This will prevent creating new documents while recording".to_string(),
                ),
            }),
            KeyCode::O => Some(ConflictInfo {
                severity: ConflictSeverity::Warning,
                description: "Conflicts with Open in most applications".to_string(),
                suggestion: Some("This will prevent opening files while recording".to_string()),
            }),
            KeyCode::R => Some(ConflictInfo {
                severity: ConflictSeverity::Warning,
                description: "Conflicts with Refresh/Reload in most applications".to_string(),
                suggestion: Some("This will prevent refreshing while recording".to_string()),
            }),
            _ => None,
        }
    } else if has_shift {
        match &shortcut.key {
            KeyCode::Delete => Some(ConflictInfo {
                severity: ConflictSeverity::Info,
                description: "Conflicts with Shift+Delete (permanent delete)".to_string(),
                suggestion: Some("Consider using different modifier".to_string()),
            }),
            _ => None,
        }
    } else {
        None
    }
}

fn check_accessibility_concerns(shortcut: &RecordingShortcut) -> Option<ConflictInfo> {
    // Check if shortcut is difficult to press with one hand
    if !is_easily_accessible(shortcut) {
        return Some(ConflictInfo {
            severity: ConflictSeverity::Info,
            description: "This combination might be difficult to press with one hand".to_string(),
            suggestion: Some("Consider using keys closer together or fewer modifiers".to_string()),
        });
    }

    // Check for modifier-heavy shortcuts
    if shortcut.modifiers.len() >= 3 {
        return Some(ConflictInfo {
            severity: ConflictSeverity::Info,
            description: "Many modifier keys may be hard to press simultaneously".to_string(),
            suggestion: Some("Consider using fewer modifiers for easier access".to_string()),
        });
    }

    None
}

fn is_easily_accessible(shortcut: &RecordingShortcut) -> bool {
    // Check if the combination can be pressed comfortably with one hand
    let left_side_keys = [
        KeyCode::Q,
        KeyCode::W,
        KeyCode::E,
        KeyCode::R,
        KeyCode::T,
        KeyCode::A,
        KeyCode::S,
        KeyCode::D,
        KeyCode::F,
        KeyCode::G,
        KeyCode::Z,
        KeyCode::X,
        KeyCode::C,
        KeyCode::V,
        KeyCode::B,
        KeyCode::Tab,
        KeyCode::CapsLock,
        KeyCode::ShiftLeft,
        KeyCode::ControlLeft,
        KeyCode::Num1,
        KeyCode::Num2,
        KeyCode::Num3,
        KeyCode::Num4,
        KeyCode::Num5,
    ];

    let right_side_keys = [
        KeyCode::Y,
        KeyCode::U,
        KeyCode::I,
        KeyCode::O,
        KeyCode::P,
        KeyCode::H,
        KeyCode::J,
        KeyCode::K,
        KeyCode::L,
        KeyCode::N,
        KeyCode::M,
        KeyCode::ShiftRight,
        KeyCode::ControlRight,
        KeyCode::Num6,
        KeyCode::Num7,
        KeyCode::Num8,
        KeyCode::Num9,
        KeyCode::Num0,
    ];

    let main_key_left = left_side_keys.contains(&shortcut.key);
    let main_key_right = right_side_keys.contains(&shortcut.key);

    // If main key is on one side, check if all modifiers are also on the same side
    if main_key_left {
        shortcut
            .modifiers
            .iter()
            .all(|m| left_side_keys.contains(m) || is_universal_modifier(m))
    } else if main_key_right {
        shortcut
            .modifiers
            .iter()
            .all(|m| right_side_keys.contains(m) || is_universal_modifier(m))
    } else {
        // Main key is in the middle (like Space), generally accessible
        true
    }
}

fn is_universal_modifier(key: &KeyCode) -> bool {
    matches!(
        key,
        KeyCode::Alt | KeyCode::AltGr | KeyCode::MetaLeft | KeyCode::MetaRight
    )
}
