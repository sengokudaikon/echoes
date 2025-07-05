//! Main configuration structures and management

use crate::shortcuts::RecordingShortcut;
use crate::{ConfigError, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main configuration structure
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

/// Available STT providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SttProvider {
    OpenAI,
    Groq,
    LocalWhisper,
    #[cfg(target_os = "macos")]
    LightningWhisper,
}

/// Lightning Whisper configuration (macOS only)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightningWhisperConfig {
    pub model: String,
    pub batch_size: u32,
    pub quantization: Option<String>, // "4bit", "8bit", or None
}

/// Local Whisper configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalWhisperConfig {
    pub model: WhisperModel,
    pub model_path: Option<PathBuf>, // Custom model path, if not using auto-download
}

/// Available Whisper models
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
            #[cfg(target_os = "macos")]
            lightning_whisper: LightningWhisperConfig {
                model: "distil-medium.en".into(),
                batch_size: 12,
                quantization: None,
            },
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
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)
                .map_err(|e| ConfigError::LoadFailed(format!("Failed to read config file: {e}")))?;
            let config: Config =
                toml::from_str(&content).map_err(|e| ConfigError::ParseError(format!("Invalid config format: {e}")))?;
            Ok(config)
        } else {
            // Create default config
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;

        // Create directory if it doesn't exist
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
    pub fn validate(&self) -> Result<()> {
        // Validate recording shortcut
        self.recording_shortcut
            .validate()
            .map_err(|e| ConfigError::ValidationError(e.to_string()))?;

        // Add more validation as needed
        Ok(())
    }
}
