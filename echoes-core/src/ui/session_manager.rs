use echoes_logging::debug;

/// Manages session state like recording status and logs
pub struct SessionManager {
    pub recording: bool,
    pub recording_shortcut: bool,
    pub logs: Vec<String>,
    pub error_message: Option<String>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            recording: false,
            recording_shortcut: false,
            logs: vec!["App started".into()],
            error_message: None,
        }
    }

    pub fn add_log(&mut self, msg: impl Into<String>) {
        let msg = msg.into();
        debug!("{}", msg);
        self.logs
            .push(format!("[{}] {}", chrono::Local::now().format("%H:%M:%S"), msg));
        // Keep only last 100 logs
        if self.logs.len() > 100 {
            self.logs.remove(0);
        }
    }

    pub fn set_error(&mut self, error: Option<String>) {
        self.error_message = error;
    }

    pub fn start_recording(&mut self) {
        self.recording = true;
    }

    pub fn stop_recording(&mut self) {
        self.recording = false;
    }

    pub fn start_shortcut_recording(&mut self) {
        self.recording_shortcut = true;
    }

    pub fn stop_shortcut_recording(&mut self) {
        self.recording_shortcut = false;
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}
