use thiserror::Error;

pub type Result<T> = std::result::Result<T, AudioError>;

#[derive(Error, Debug)]
pub enum AudioError {
    #[error("No input device available")]
    NoInputDevice,

    #[error("Stream creation failed: {0}")]
    StreamCreationFailed(String),

    #[error("Unsupported sample format: {0}")]
    UnsupportedFormat(String),

    #[error("WAV encoding failed: {0}")]
    WavEncodingFailed(String),

    #[error("Mutex poisoned")]
    MutexPoisoned,

    #[error("VAD processing failed: {0}")]
    VadProcessingFailed(String),

    #[error("Other error: {0}")]
    Other(String),
}
