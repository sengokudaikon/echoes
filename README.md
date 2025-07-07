# echoes - Pure Rust Dictation App

A fast, native dictation application written entirely in Rust, supporting multiple STT providers including local Whisper.

## Architecture

### Core Components

1. **Keyboard Listener**
    - Uses `rdev` for cross-platform keyboard event capture
    - Detects Ctrl key hold/release for recording triggers

2. **Audio Recorder**
    - Uses `cpal` for cross-platform audio capture
    - Records to WAV format in memory
    - Configurable sample rate and channels

3. **STT Integration**
    - **OpenAI Whisper**: HTTP client using `reqwest`
    - TBD
    - Trait-based design for easy provider addition

4. **Text Output**
    - Uses `enigo` for typing text into active application
    - Cross-platform keyboard automation

5. **System Tray**
    - Uses `tray-icon` for cross-platform system tray
    - Simple menu for start/stop recording and settings

6. **Configuration**
    - TOML-based config using `serde` and `toml`
    - Stores API keys, shortcuts, and preferences
    - Platform-appropriate config locations

7. **UI (Optional Phase 2)**
    - Native UI using `egui` or `iced`
    - Settings panel and recording history

### Key Dependencies

```toml
[dependencies]
# Existing
rdev = "0.5"           # Keyboard events
enigo = "0.3"          # Keyboard automation
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# New dependencies
tokio = { version = "1", features = ["full"] }  # Async runtime
cpal = "0.15"          # Audio recording
hound = "3.5"          # WAV file handling
reqwest = { version = "0.11", features = ["json", "multipart"] }  # HTTP client
tray-icon = "0.11"     # System tray
tauri-winrt-notification = "0.1"  # Windows notifications
notify-rust = "4"      # Linux/Mac notifications
directories = "5"      # Platform config paths
toml = "0.8"          # Config files
anyhow = "1"          # Error handling
tracing = "0.1"       # Logging
tracing-subscriber = "0.3"
```

### Project Structure

```
echoes/
├── src/
│   ├── main.rs           # Entry point, CLI handling
│   ├── keyboard.rs       # Keyboard event handling
│   ├── audio.rs          # Audio recording
│   ├── stt/
│   │   ├── mod.rs        # STT trait definition
│   │   ├── openai.rs     # OpenAI Whisper implementation
│   │   ├── groq.rs       # Groq implementation
│   │   └── local.rs      # Local Whisper (whisper-rs)
│   ├── config.rs         # Configuration management
│   ├── tray.rs           # System tray integration
│   └── text_output.rs    # Text typing functionality
├── Cargo.toml
└── README.md
```

### Features

- [x] Cross-platform keyboard event capture
- [x] Text output to active application
- [ ] Audio recording
- [ ] OpenAI Whisper integration
- [ ] Local Whisper integration
- [ ] System tray
- [ ] Configuration management
- [ ] Notifications
- [ ] Recording history (SQLite)
- [ ] Native UI (Phase 2)

### Platform Support

- macOS: Full support
- Windows: Full support
- Linux: Full support
- Android: TBD
- iOS: TBD
- Web: TBD