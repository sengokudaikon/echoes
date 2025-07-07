use std::{error::Error as StdError, fmt};

/// Main error type for Echoes application
#[derive(Debug)]
pub enum EchoesError {
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

pub type Result<T> = std::result::Result<T, EchoesError>;

// Display implementations
impl fmt::Display for EchoesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Audio(e) => write!(f, "Audio error: {e}"),
            Self::Config(e) => write!(f, "Configuration error: {e}"),
            Self::Keyboard(e) => write!(f, "Keyboard error: {e}"),
            Self::Stt(e) => write!(f, "STT service error: {e}"),
            Self::Permission(e) => write!(f, "Permission error: {e}"),
            Self::Logging(e) => write!(f, "Logging error: {e}"),
            Self::Ui(e) => write!(f, "UI error: {e}"),
            Self::Io(e) => write!(f, "IO error: {e}"),
            Self::Other(msg) => write!(f, "Error: {msg}"),
        }
    }
}

impl fmt::Display for AudioError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoInputDevice => write!(f, "No audio input device available"),
            Self::UnsupportedFormat(fmt) => write!(f, "Unsupported audio format: {fmt}"),
            Self::StreamCreationFailed(msg) => {
                write!(f, "Failed to create audio stream: {msg}")
            }
            Self::RecordingFailed(msg) => write!(f, "Recording failed: {msg}"),
            Self::WavEncodingFailed(msg) => write!(f, "WAV encoding failed: {msg}"),
            Self::MutexPoisoned => write!(f, "Audio mutex was poisoned"),
        }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LoadFailed(msg) => write!(f, "Failed to load config: {msg}"),
            Self::SaveFailed(msg) => write!(f, "Failed to save config: {msg}"),
            Self::ValidationFailed(msg) => write!(f, "Config validation failed: {msg}"),
            Self::InvalidShortcut(msg) => write!(f, "Invalid shortcut: {msg}"),
            Self::ParseError(msg) => write!(f, "Config parse error: {msg}"),
        }
    }
}

impl fmt::Display for KeyboardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ListenerStartFailed(msg) => {
                write!(f, "Failed to start keyboard listener: {msg}")
            }
            Self::ListenerStopFailed(msg) => {
                write!(f, "Failed to stop keyboard listener: {msg}")
            }
            Self::EventChannelClosed => {
                write!(f, "Keyboard event channel closed unexpectedly")
            }
            Self::ShortcutConflict(msg) => write!(f, "Shortcut conflict: {msg}"),
            Self::MutexPoisoned => write!(f, "Keyboard mutex was poisoned"),
        }
    }
}

impl fmt::Display for SttError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ApiKeyMissing(provider) => write!(f, "API key missing for {provider}"),
            Self::NetworkError(msg) => write!(f, "Network error: {msg}"),
            Self::InvalidResponse(msg) => {
                write!(f, "Invalid response from STT service: {msg}")
            }
            Self::ServiceUnavailable(msg) => write!(f, "STT service unavailable: {msg}"),
            Self::RateLimitExceeded => write!(f, "Rate limit exceeded"),
            Self::InvalidAudioFormat => write!(f, "Invalid audio format for STT service"),
        }
    }
}

impl fmt::Display for PermissionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AccessibilityDenied => {
                write!(
                    f,
                    "Accessibility permissions required. Please grant access in System Settings > Privacy & Security \
                     > Accessibility"
                )
            }
            Self::MicrophoneDenied => {
                write!(f, "Microphone access denied. Please grant microphone permissions")
            }
            Self::SystemApiError(msg) => write!(f, "System API error: {msg}"),
        }
    }
}

impl fmt::Display for LoggingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FileCreationFailed(msg) => {
                write!(f, "Failed to create log file: {msg}")
            }
            Self::WriteFailed(msg) => write!(f, "Failed to write to log: {msg}"),
            Self::MutexPoisoned => write!(f, "Logging mutex was poisoned"),
        }
    }
}

impl fmt::Display for UiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InitializationFailed(msg) => write!(f, "Failed to initialize UI: {msg}"),
            Self::RenderingError(msg) => write!(f, "UI rendering error: {msg}"),
        }
    }
}

impl StdError for EchoesError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Io(e) => Some(e),
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

impl From<std::io::Error> for EchoesError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<AudioError> for EchoesError {
    fn from(err: AudioError) -> Self {
        Self::Audio(err)
    }
}

impl From<ConfigError> for EchoesError {
    fn from(err: ConfigError) -> Self {
        Self::Config(err)
    }
}

impl From<KeyboardError> for EchoesError {
    fn from(err: KeyboardError) -> Self {
        Self::Keyboard(err)
    }
}

impl From<SttError> for EchoesError {
    fn from(err: SttError) -> Self {
        Self::Stt(err)
    }
}

impl From<PermissionError> for EchoesError {
    fn from(err: PermissionError) -> Self {
        Self::Permission(err)
    }
}

impl From<LoggingError> for EchoesError {
    fn from(err: LoggingError) -> Self {
        Self::Logging(err)
    }
}

impl From<echoes_logging::LoggingError> for EchoesError {
    fn from(err: echoes_logging::LoggingError) -> Self {
        Self::Other(err.to_string())
    }
}

impl From<UiError> for EchoesError {
    fn from(err: UiError) -> Self {
        Self::Ui(err)
    }
}

impl<T> From<std::sync::PoisonError<T>> for AudioError {
    fn from(_: std::sync::PoisonError<T>) -> Self {
        Self::MutexPoisoned
    }
}

impl<T> From<std::sync::PoisonError<T>> for KeyboardError {
    fn from(_: std::sync::PoisonError<T>) -> Self {
        Self::MutexPoisoned
    }
}

impl<T> From<std::sync::PoisonError<T>> for LoggingError {
    fn from(_: std::sync::PoisonError<T>) -> Self {
        Self::MutexPoisoned
    }
}
