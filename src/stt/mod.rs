// STT module stub for now
use anyhow::Result;

pub trait SttProvider {
    fn transcribe(&self, audio_data: Vec<u8>) -> Result<String>;
}

pub struct OpenAiStt {
    api_key: String,
}

impl OpenAiStt {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }
}

impl SttProvider for OpenAiStt {
    fn transcribe(&self, _audio_data: Vec<u8>) -> Result<String> {
        // Stub implementation
        Ok("STT not implemented yet".to_string())
    }
}