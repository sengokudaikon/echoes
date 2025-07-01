use std::error::Error as StdError;
use std::fmt;

/// Main error type for Whispers application
#[derive(Debug)]
pub enum WhispersError {
    /// Audio recording and processing errors
    Audio(AudioError),
    /// Configuration errors
    Config(ConfigError),
    /// Keyboard/input handling errors
    Keyboard(KeyboardError),
    /// Speech-to-text service errors
    Stt(SttError),
    /// System permission errors
    Permission(PermissionError),
    /// Logging system errors
    Logging(LoggingError),
    /// UI/Application errors
    Ui(UiError),
    /// IO errors
    Io(std::io::Error),
    /// General errors that don't fit other categories
    Other(String),
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum AudioError {
    NoInputDevice,
    UnsupportedFormat(String),
    StreamCreationFailed(String),
    RecordingFailed(String),
    WavEncodingFailed(String),
    MutexPoisoned,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum ConfigError {
    LoadFailed(String),
    SaveFailed(String),
    ValidationFailed(String),
    InvalidShortcut(String),
    ParseError(String),
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum KeyboardError {
    ListenerStartFailed(String),
    ListenerStopFailed(String),
    EventChannelClosed,
    ShortcutConflict(String),
    MutexPoisoned,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum SttError {
    ApiKeyMissing(String),
    NetworkError(String),
    InvalidResponse(String),
    ServiceUnavailable(String),
    RateLimitExceeded,
    InvalidAudioFormat,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum PermissionError {
    AccessibilityDenied,
    MicrophoneDenied,
    SystemApiError(String),
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum LoggingError {
    FileCreationFailed(String),
    WriteFailed(String),
    MutexPoisoned,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum UiError {
    InitializationFailed(String),
    RenderingError(String),
}

// Result type alias for convenience
pub type Result<T> = std::result::Result<T, WhispersError>;

// Display implementations
impl fmt::Display for WhispersError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WhispersError::Audio(e) => write!(f, "Audio error: {e}"),
            WhispersError::Config(e) => write!(f, "Configuration error: {e}"),
            WhispersError::Keyboard(e) => write!(f, "Keyboard error: {e}"),
            WhispersError::Stt(e) => write!(f, "STT service error: {e}"),
            WhispersError::Permission(e) => write!(f, "Permission error: {e}"),
            WhispersError::Logging(e) => write!(f, "Logging error: {e}"),
            WhispersError::Ui(e) => write!(f, "UI error: {e}"),
            WhispersError::Io(e) => write!(f, "IO error: {e}"),
            WhispersError::Other(msg) => write!(f, "Error: {msg}"),
        }
    }
}

impl fmt::Display for AudioError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AudioError::NoInputDevice => write!(f, "No audio input device available"),
            AudioError::UnsupportedFormat(fmt) => write!(f, "Unsupported audio format: {fmt}"),
            AudioError::StreamCreationFailed(msg) => {
                write!(f, "Failed to create audio stream: {msg}")
            }
            AudioError::RecordingFailed(msg) => write!(f, "Recording failed: {msg}"),
            AudioError::WavEncodingFailed(msg) => write!(f, "WAV encoding failed: {msg}"),
            AudioError::MutexPoisoned => write!(f, "Audio mutex was poisoned"),
        }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::LoadFailed(msg) => write!(f, "Failed to load config: {msg}"),
            ConfigError::SaveFailed(msg) => write!(f, "Failed to save config: {msg}"),
            ConfigError::ValidationFailed(msg) => write!(f, "Config validation failed: {msg}"),
            ConfigError::InvalidShortcut(msg) => write!(f, "Invalid shortcut: {msg}"),
            ConfigError::ParseError(msg) => write!(f, "Config parse error: {msg}"),
        }
    }
}

impl fmt::Display for KeyboardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeyboardError::ListenerStartFailed(msg) => {
                write!(f, "Failed to start keyboard listener: {msg}")
            }
            KeyboardError::ListenerStopFailed(msg) => {
                write!(f, "Failed to stop keyboard listener: {msg}")
            }
            KeyboardError::EventChannelClosed => {
                write!(f, "Keyboard event channel closed unexpectedly")
            }
            KeyboardError::ShortcutConflict(msg) => write!(f, "Shortcut conflict: {msg}"),
            KeyboardError::MutexPoisoned => write!(f, "Keyboard mutex was poisoned"),
        }
    }
}

impl fmt::Display for SttError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SttError::ApiKeyMissing(provider) => write!(f, "API key missing for {provider}"),
            SttError::NetworkError(msg) => write!(f, "Network error: {msg}"),
            SttError::InvalidResponse(msg) => {
                write!(f, "Invalid response from STT service: {msg}")
            }
            SttError::ServiceUnavailable(msg) => write!(f, "STT service unavailable: {msg}"),
            SttError::RateLimitExceeded => write!(f, "Rate limit exceeded"),
            SttError::InvalidAudioFormat => write!(f, "Invalid audio format for STT service"),
        }
    }
}

impl fmt::Display for PermissionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PermissionError::AccessibilityDenied => {
                write!(
                    f,
                    "Accessibility permissions required. Please grant access in System Settings > Privacy & Security > Accessibility"
                )
            }
            PermissionError::MicrophoneDenied => {
                write!(
                    f,
                    "Microphone access denied. Please grant microphone permissions"
                )
            }
            PermissionError::SystemApiError(msg) => write!(f, "System API error: {msg}"),
        }
    }
}

impl fmt::Display for LoggingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoggingError::FileCreationFailed(msg) => {
                write!(f, "Failed to create log file: {msg}")
            }
            LoggingError::WriteFailed(msg) => write!(f, "Failed to write to log: {msg}"),
            LoggingError::MutexPoisoned => write!(f, "Logging mutex was poisoned"),
        }
    }
}

impl fmt::Display for UiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UiError::InitializationFailed(msg) => write!(f, "Failed to initialize UI: {msg}"),
            UiError::RenderingError(msg) => write!(f, "UI rendering error: {msg}"),
        }
    }
}

// StdError implementations
impl StdError for WhispersError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            WhispersError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl StdError for AudioError {}
impl StdError for ConfigError {}
impl StdError for KeyboardError {}
impl StdError for SttError {}
impl StdError for PermissionError {}
impl StdError for LoggingError {}
impl StdError for UiError {}

// From implementations for easy conversion
impl From<std::io::Error> for WhispersError {
    fn from(err: std::io::Error) -> Self {
        WhispersError::Io(err)
    }
}

impl From<AudioError> for WhispersError {
    fn from(err: AudioError) -> Self {
        WhispersError::Audio(err)
    }
}

impl From<ConfigError> for WhispersError {
    fn from(err: ConfigError) -> Self {
        WhispersError::Config(err)
    }
}

impl From<KeyboardError> for WhispersError {
    fn from(err: KeyboardError) -> Self {
        WhispersError::Keyboard(err)
    }
}

impl From<SttError> for WhispersError {
    fn from(err: SttError) -> Self {
        WhispersError::Stt(err)
    }
}

impl From<PermissionError> for WhispersError {
    fn from(err: PermissionError) -> Self {
        WhispersError::Permission(err)
    }
}

impl From<LoggingError> for WhispersError {
    fn from(err: LoggingError) -> Self {
        WhispersError::Logging(err)
    }
}

impl From<UiError> for WhispersError {
    fn from(err: UiError) -> Self {
        WhispersError::Ui(err)
    }
}

// Helper for converting mutex errors
impl<T> From<std::sync::PoisonError<T>> for AudioError {
    fn from(_: std::sync::PoisonError<T>) -> Self {
        AudioError::MutexPoisoned
    }
}

impl<T> From<std::sync::PoisonError<T>> for KeyboardError {
    fn from(_: std::sync::PoisonError<T>) -> Self {
        KeyboardError::MutexPoisoned
    }
}

impl<T> From<std::sync::PoisonError<T>> for LoggingError {
    fn from(_: std::sync::PoisonError<T>) -> Self {
        LoggingError::MutexPoisoned
    }
}
