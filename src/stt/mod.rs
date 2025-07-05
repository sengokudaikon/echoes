// STT module with multiple provider implementations
#![allow(dead_code)]

mod whisper;

use anyhow::Result;
#[allow(unused_imports)] pub use whisper::LocalWhisperStt;

pub trait SttProvider {
    fn transcribe(&self, audio_data: Vec<u8>) -> Result<String>;
}

pub struct OpenAiStt {
    api_key: String,
}

impl OpenAiStt {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
        }
    }
}

impl SttProvider for OpenAiStt {
    fn transcribe(&self, _audio_data: Vec<u8>) -> Result<String> {
        // Stub implementation
        Ok("STT not implemented yet".to_string())
    }
}
