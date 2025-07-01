use serde::{Deserialize, Serialize};
use directories::ProjectDirs;
use std::path::PathBuf;
use anyhow::Result;

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
    
    // Recording settings
    pub recording_shortcut: RecordingShortcut,
    
    // Post-processing
    pub post_processing: PostProcessingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SttProvider {
    OpenAI,
    Groq,
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
pub struct RecordingShortcut {
    pub mode: ShortcutMode,
    pub key: KeyCode,  // The main key
    pub modifiers: Vec<KeyCode>, // Additional modifier keys
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum KeyCode {
    // Control keys
    ControlLeft,
    ControlRight,
    ShiftLeft,
    ShiftRight,
    Alt,
    AltGr,
    MetaLeft,  // Cmd on Mac, Windows key on Windows
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
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    
    // Letters
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    
    // Numbers
    Num0, Num1, Num2, Num3, Num4, Num5, Num6, Num7, Num8, Num9,
    
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
    Hold,    // Hold key to record
    Toggle,  // Press to start/stop
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
            let content = std::fs::read_to_string(&config_path)?;
            let config: Config = toml::from_str(&content)?;
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
            std::fs::create_dir_all(parent)?;
        }
        
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;
        
        Ok(())
    }
    
    fn config_path() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("com", "whispo", "whispo-rust")
            .ok_or_else(|| anyhow::anyhow!("Failed to determine config directory"))?;
        
        Ok(proj_dirs.config_dir().join("config.toml"))
    }
}