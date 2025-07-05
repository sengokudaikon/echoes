use echoes_audio::AudioRecorder;
use echoes_config::{Config, RecordingShortcut, ShortcutMode};
use echoes_keyboard::KeyboardEvent;

use super::{
    config_manager::ConfigManager, keyboard_manager::KeyboardManager, session_manager::SessionManager,
    shortcut_manager::ShortcutManager, shortcuts, system_manager::SystemManager,
};

/// Command trait for handling keyboard events
trait KeyboardEventCommand {
    fn execute(&self, app_state: &mut AppState) -> bool;
}

/// Commands for handling specific keyboard events
struct RecordingKeyPressedCommand;
struct RecordingKeyReleasedCommand;
struct OtherKeyPressedCommand;
struct ListenerErrorCommand(String);
struct ShortcutRecordedCommand(RecordingShortcut);
struct RecordingCancelledCommand;

/// Core application state using composition pattern
pub struct AppState {
    pub config: Config,
    pub config_manager: ConfigManager,
    pub keyboard_manager: KeyboardManager,
    pub session_manager: SessionManager,
    pub shortcut_manager: ShortcutManager,
    #[allow(dead_code)]
    pub system_manager: SystemManager,
    pub audio_recorder: AudioRecorder,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        let mut state = Self {
            config,
            config_manager: ConfigManager::new(),
            keyboard_manager: KeyboardManager::new(),
            session_manager: SessionManager::new(),
            shortcut_manager: ShortcutManager::new(),
            system_manager: SystemManager::new(),
            audio_recorder: AudioRecorder::new(),
        };

        // Initialize keyboard listener
        state.init_keyboard_listener();
        state
    }

    pub fn init_keyboard_listener(&mut self) {
        match self.keyboard_manager.init(self.config.recording_shortcut.clone()) {
            Ok(()) => {
                self.session_manager.add_log("Keyboard listener started");
                self.session_manager.set_error(None);
            }
            Err(e) => {
                self.session_manager.add_log(format!("Keyboard init failed: {e}"));
                self.session_manager.set_error(Some(e));
            }
        }
    }

    pub fn open_accessibility_settings(&mut self) {
        match SystemManager::open_accessibility_settings() {
            Ok(()) => self.session_manager.add_log("Opened System Settings"),
            Err(e) => self.session_manager.add_log(format!("System settings error: {e}")),
        }
    }

    pub fn handle_keyboard_events(&mut self) -> bool {
        let events = self.keyboard_manager.try_recv_event();
        let mut needs_repaint = false;

        for event in events {
            needs_repaint = true;
            let command: Box<dyn KeyboardEventCommand> = match event {
                KeyboardEvent::RecordingKeyPressed => Box::new(RecordingKeyPressedCommand),
                KeyboardEvent::RecordingKeyReleased => Box::new(RecordingKeyReleasedCommand),
                KeyboardEvent::OtherKeyPressed => Box::new(OtherKeyPressedCommand),
                KeyboardEvent::ListenerError(msg) => Box::new(ListenerErrorCommand(msg)),
                KeyboardEvent::ShortcutRecorded(shortcut) => Box::new(ShortcutRecordedCommand(shortcut)),
                KeyboardEvent::RecordingCancelled => Box::new(RecordingCancelledCommand),
            };

            command.execute(self);
        }

        needs_repaint
    }

    pub fn apply_shortcut(&mut self, shortcut: RecordingShortcut) {
        let shortcut_str = shortcuts::format_shortcut(&shortcut);
        self.config.recording_shortcut = shortcut;
        self.session_manager
            .add_log(format!("Changed shortcut to {shortcut_str}"));
        self.config_manager.save_async(self.config.clone());
        self.keyboard_manager
            .update_shortcut(self.config.recording_shortcut.clone());
    }

    pub fn update_shortcut_listener(&self) {
        self.keyboard_manager
            .update_shortcut(self.config.recording_shortcut.clone());
    }

    pub fn start_recording_shortcut(&mut self) {
        self.session_manager.start_shortcut_recording();
        self.shortcut_manager.clear_recorded();
        self.session_manager.add_log("Started shortcut recording mode");

        self.keyboard_manager.start_recording_shortcut();
        if self.keyboard_manager.listener.is_some() {
            self.session_manager
                .add_log("Called start_recording_shortcut on listener");
        } else {
            self.session_manager.add_log("ERROR: No keyboard listener available!");
        }
    }

    pub fn stop_recording_shortcut(&mut self) {
        self.session_manager.stop_shortcut_recording();
        self.shortcut_manager.clear_recorded();
        self.session_manager.add_log("Shortcut recording cancelled");
        self.keyboard_manager.stop_recording_shortcut();
    }

    // Convenience accessors
    pub fn recording(&self) -> bool {
        self.session_manager.recording
    }

    pub fn recording_shortcut(&self) -> bool {
        self.session_manager.recording_shortcut
    }

    pub fn logs(&self) -> &[String] {
        &self.session_manager.logs
    }

    pub fn error_message(&self) -> &Option<String> {
        &self.session_manager.error_message
    }

    pub fn permissions_granted(&self) -> bool {
        self.keyboard_manager.permissions_granted
    }

    pub fn add_log(&mut self, msg: impl Into<String>) {
        self.session_manager.add_log(msg);
    }

    pub fn recorded_shortcut(&mut self) -> Option<RecordingShortcut> {
        self.shortcut_manager.take_recorded()
    }

    pub fn show_visual_editor(&self) -> bool {
        self.shortcut_manager.show_visual_editor
    }

    pub fn set_show_visual_editor(&mut self, show: bool) {
        self.shortcut_manager.set_visual_editor(show);
    }

    /// Helper method to get formatted shortcut string (cached to avoid repeated
    /// formatting)
    fn get_shortcut_str(&self) -> String {
        shortcuts::format_shortcut(&self.config.recording_shortcut)
    }

    /// Helper method to create recording state message
    fn create_recording_message(&self, action: &str) -> String {
        let shortcut_str = self.get_shortcut_str();
        match self.config.recording_shortcut.mode {
            ShortcutMode::Hold => {
                format!(
                    "{shortcut_str} {action} - Recording {}",
                    if action == "pressed" { "started" } else { "stopped" }
                )
            }
            ShortcutMode::Toggle => {
                format!(
                    "{shortcut_str} pressed - Recording {}",
                    if action == "pressed" { "started" } else { "stopped" }
                )
            }
        }
    }
}

/// Command implementations for keyboard events
impl KeyboardEventCommand for RecordingKeyPressedCommand {
    fn execute(&self, app_state: &mut AppState) -> bool {
        if !app_state.session_manager.recording {
            app_state.session_manager.start_recording();

            // Start audio recording
            if let Err(e) = app_state.audio_recorder.start_recording() {
                app_state
                    .session_manager
                    .add_log(format!("Failed to start audio recording: {e}"));
                app_state.session_manager.stop_recording();
            } else {
                let msg = app_state.create_recording_message("pressed");
                app_state.session_manager.add_log(msg);
            }
        }
        true
    }
}

impl KeyboardEventCommand for RecordingKeyReleasedCommand {
    fn execute(&self, app_state: &mut AppState) -> bool {
        if app_state.session_manager.recording {
            app_state.session_manager.stop_recording();

            // Stop audio recording and save files
            let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");

            // Process recording with VAD
            match app_state.audio_recorder.stop_recording_with_vad() {
                Ok((raw_audio, segments)) => {
                    // Save raw recording
                    let filename = format!("recording_{timestamp}_raw.wav");
                    match std::fs::write(&filename, &raw_audio) {
                        Ok(_) => {
                            app_state.session_manager.add_log(format!(
                                "Saved raw: {} ({} bytes)",
                                filename,
                                raw_audio.len()
                            ));
                        }
                        Err(e) => {
                            app_state
                                .session_manager
                                .add_log(format!("Failed to save raw recording: {e}"));
                        }
                    }

                    // Save VAD segments
                    app_state
                        .session_manager
                        .add_log(format!("Found {} speech segments", segments.len()));
                    for (i, segment_data) in segments.iter().enumerate() {
                        let filename = format!("recording_{timestamp}_segment_{i}.wav");
                        match std::fs::write(&filename, segment_data) {
                            Ok(_) => {
                                app_state.session_manager.add_log(format!(
                                    "Saved segment: {} ({} bytes)",
                                    filename,
                                    segment_data.len()
                                ));
                            }
                            Err(e) => {
                                app_state
                                    .session_manager
                                    .add_log(format!("Failed to save {filename}: {e}"));
                            }
                        }
                    }
                }
                Err(e) => {
                    app_state
                        .session_manager
                        .add_log(format!("Failed to process recording: {e}"));
                }
            }

            let msg = app_state.create_recording_message("released");
            app_state.session_manager.add_log(msg);
        }
        true
    }
}

impl KeyboardEventCommand for OtherKeyPressedCommand {
    fn execute(&self, app_state: &mut AppState) -> bool {
        if app_state.session_manager.recording {
            app_state.session_manager.stop_recording();
            // Stop recording without saving
            let _ = app_state.audio_recorder.stop_recording();
            app_state.session_manager.add_log("Recording cancelled");
        }
        true
    }
}

impl KeyboardEventCommand for ListenerErrorCommand {
    fn execute(&self, app_state: &mut AppState) -> bool {
        app_state.session_manager.set_error(Some(self.0.clone()));
        app_state
            .session_manager
            .add_log(format!("Keyboard listener error: {}", self.0));
        app_state.keyboard_manager.clear_receiver();
        true
    }
}

impl KeyboardEventCommand for ShortcutRecordedCommand {
    fn execute(&self, app_state: &mut AppState) -> bool {
        app_state.shortcut_manager.record_shortcut(self.0.clone());
        app_state.session_manager.stop_shortcut_recording();
        let shortcut_str = shortcuts::format_shortcut(&self.0);
        app_state
            .session_manager
            .add_log(format!("New shortcut recorded: {shortcut_str}"));
        true
    }
}

impl KeyboardEventCommand for RecordingCancelledCommand {
    fn execute(&self, app_state: &mut AppState) -> bool {
        app_state.session_manager.stop_shortcut_recording();
        app_state.shortcut_manager.clear_recorded();
        app_state.session_manager.add_log("Shortcut recording cancelled");
        true
    }
}
