//! Main configuration structures and management

use std::path::PathBuf;

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::{shortcuts::RecordingShortcut, ConfigError, Result};

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub stt_provider: SttProvider,

    pub openai_api_key: Option<String>,
    pub groq_api_key: Option<String>,

    pub openai_base_url: Option<String>,
    pub groq_base_url: Option<String>,

    pub openai_stt_model: Option<String>,
    pub openai_stt_prompt: Option<String>,
    pub groq_stt_model: Option<String>,
    pub groq_stt_prompt: Option<String>,

    pub local_whisper: LocalWhisperConfig,

    pub recording_shortcut: RecordingShortcut,

    pub post_processing: PostProcessingConfig,
}

/// Available STT providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SttProvider {
    OpenAI,
    Groq,
    LocalWhisper,
}

/// Local Whisper configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalWhisperConfig {
    pub model: WhisperModel,
    pub model_path: Option<PathBuf>,
}

/// Available Whisper models
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

/// Post-processing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostProcessingConfig {
    pub enabled: bool,
    pub provider: LlmProvider,
    pub model: String,
    pub prompt: String,
}

/// Available LLM providers for post-processing
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
            openai_base_url: Some("https://api.openai.com/v1".into()),
            groq_base_url: Some("https://api.groq.com/openai/v1".into()),
            openai_stt_model: Some("whisper-1".into()),
            openai_stt_prompt: None,
            groq_stt_model: Some("whisper-large-v3".into()),
            groq_stt_prompt: None,
            local_whisper: LocalWhisperConfig {
                model: WhisperModel::Base,
                model_path: None,
            },
            recording_shortcut: RecordingShortcut::default(),
            post_processing: PostProcessingConfig {
                enabled: false,
                provider: LlmProvider::OpenAI,
                model: "gpt-4o-mini".into(),
                prompt: "Clean up the following transcript, fixing any errors and improving clarity while preserving \
                         the original meaning:\n\n{transcript}"
                    .into(),
            },
        }
    }
}

impl Config {
    /// Load configuration from file or create default
    ///
    /// # Errors
    ///
    /// Returns an error if the config file cannot be read, parsed, or if the
    /// default config cannot be saved.
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)
                .map_err(|e| ConfigError::LoadFailed(format!("Failed to read config file: {e}")))?;
            let config: Self =
                toml::from_str(&content).map_err(|e| ConfigError::ParseError(format!("Invalid config format: {e}")))?;
            Ok(config)
        } else {
            let config = Self::default();
            config.save()?;
            Ok(config)
        }
    }

    /// Save configuration to file
    ///
    /// # Errors
    ///
    /// Returns an error if the config directory cannot be created or the config
    /// file cannot be written.
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;

        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| ConfigError::SaveFailed(format!("Failed to create config directory: {e}")))?;
        }

        let content = toml::to_string_pretty(self)
            .map_err(|e| ConfigError::SaveFailed(format!("Failed to serialize config: {e}")))?;
        std::fs::write(&config_path, content)
            .map_err(|e| ConfigError::SaveFailed(format!("Failed to write config file: {e}")))?;

        Ok(())
    }

    /// Async version of save to avoid blocking the UI thread
    ///
    /// # Errors
    ///
    /// Returns an error if the config cannot be serialized, directory cannot be
    /// created, or file cannot be written.
    pub async fn save_async(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        let content = toml::to_string_pretty(self)
            .map_err(|e| ConfigError::SaveFailed(format!("Failed to serialize config: {e}")))?;

        let config_path = config_path.clone();
        let content = content.clone();

        tokio::task::spawn_blocking(move || {
            if let Some(parent) = config_path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| ConfigError::SaveFailed(format!("Failed to create config directory: {e}")))?;
            }

            std::fs::write(&config_path, content)
                .map_err(|e| ConfigError::SaveFailed(format!("Failed to write config file: {e}")))?;

            Ok::<(), ConfigError>(())
        })
        .await
        .map_err(|e| ConfigError::SaveFailed(format!("Task join error: {e}")))?
    }

    /// Get the configuration file path
    fn config_path() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("com", "echoes", "echoes")
            .ok_or_else(|| ConfigError::LoadFailed("Failed to determine config directory".into()))?;

        Ok(proj_dirs.config_dir().join("config.toml"))
    }

    /// Validate the entire configuration
    ///
    /// # Errors
    ///
    /// Returns an error if any configuration value is invalid, particularly
    /// shortcut validation.
    pub fn validate(&self) -> Result<()> {
        self.recording_shortcut
            .validate()
            .map_err(|e| ConfigError::ValidationError(e.to_string()))?;

        Ok(())
    }
}
