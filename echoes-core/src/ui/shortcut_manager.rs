use echoes_config::RecordingShortcut;

/// Manages shortcut recording and editing state
pub struct ShortcutManager {
    pub recorded_shortcut: Option<RecordingShortcut>,
    pub show_visual_editor: bool,
}

impl ShortcutManager {
    pub const fn new() -> Self {
        Self {
            recorded_shortcut: None,
            show_visual_editor: false,
        }
    }

    pub fn record_shortcut(&mut self, shortcut: RecordingShortcut) {
        self.recorded_shortcut = Some(shortcut);
    }

    pub const fn take_recorded(&mut self) -> Option<RecordingShortcut> {
        self.recorded_shortcut.take()
    }

    pub fn clear_recorded(&mut self) {
        self.recorded_shortcut = None;
    }

    #[allow(dead_code)]
    pub const fn toggle_visual_editor(&mut self) {
        self.show_visual_editor = !self.show_visual_editor;
    }

    pub const fn set_visual_editor(&mut self, show: bool) {
        self.show_visual_editor = show;
    }
}

impl Default for ShortcutManager {
    fn default() -> Self {
        Self::new()
    }
}
