use anyhow::Result;
use reqwest::multipart::{Form, Part};
use tracing::{debug, error};

use super::SttProvider;

pub struct OpenAiStt {
    api_key: String,
    base_url: String,
    model: String,
    prompt: Option<String>,
    client: reqwest::Client,
}

impl OpenAiStt {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: "https://api.openai.com/v1".to_string(),
            model: "whisper-1".to_string(),
            prompt: None,
            client: reqwest::Client::new(),
        }
    }

    #[must_use]
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    #[must_use]
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    #[must_use]
    pub fn with_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.prompt = Some(prompt.into());
        self
    }
}

impl SttProvider for OpenAiStt {
    async fn transcribe(&self, audio_data: Vec<u8>) -> Result<String> {
        debug!("Starting OpenAI transcription with model: {}", self.model);
        let audio_part = Part::bytes(audio_data).file_name("audio.wav").mime_str("audio/wav")?;

        let mut form = Form::new()
            .part("file", audio_part)
            .text("model", self.model.clone())
            .text("response_format", "json");

        if let Some(ref prompt) = self.prompt {
            form = form.text("prompt", prompt.clone());
        }

        let url = format!("{}/audio/transcriptions", self.base_url);
        debug!("Making request to: {}", url);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .multipart(form)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            let error_message = format!("OpenAI API error: {status} - {error_text}");
            error!("{}", error_message);
            #[allow(clippy::wildcard_imports)]
            return Err(anyhow::anyhow!(error_message));
        }

        let response_text = response.text().await?;
        debug!("Raw response: {}", response_text);

        let response_json: serde_json::Value = serde_json::from_str(&response_text)?;

        #[allow(clippy::wildcard_imports)]
        let text = response_json["text"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'text' field in response"))?
            .to_string();

        debug!("Transcription result: {}", text);
        Ok(text)
    }
}
