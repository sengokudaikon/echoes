use echoes_config::Config;
use eframe::egui;

mod shortcut_editor;
use shortcut_editor::ShortcutEditorAction;

mod app_state;
mod config;
mod config_manager;
mod keyboard_manager;
mod logs;
mod session_manager;
mod shortcut_manager;
mod shortcuts;
mod status;
mod system_manager;

use app_state::AppState;

pub struct WhispoApp {
    state: AppState,
}

impl WhispoApp {
    pub fn new(_cc: &eframe::CreationContext<'_>, config: Config) -> Self {
        Self {
            state: AppState::new(config),
        }
    }

    fn handle_shortcut_action(&mut self, action: ShortcutEditorAction) {
        match action {
            ShortcutEditorAction::StartRecording => {
                self.state.start_recording_shortcut();
            }
            ShortcutEditorAction::CancelRecording => {
                self.state.stop_recording_shortcut();
            }
            ShortcutEditorAction::Reset => {
                self.state.add_log("Shortcut reset to default (Ctrl)");
                self.state.config_manager.save_async(self.state.config.clone());
            }
            ShortcutEditorAction::None => {}
        }
    }

    fn process_recorded_shortcut(&mut self, ui: &mut egui::Ui) {
        if let Some(recorded) = self.state.recorded_shortcut() {
            match recorded.validate() {
                Ok(()) => {
                    self.state.config.recording_shortcut = recorded;
                    self.state.add_log("Applied new shortcut");
                    self.state.config_manager.save_async(self.state.config.clone());
                    self.state.update_shortcut_listener();
                }
                Err(err) => {
                    self.state.add_log(format!("Invalid shortcut: {err}"));
                    ui.colored_label(egui::Color32::RED, format!("⚠️ {err}"));
                }
            }
        }
    }
}

impl eframe::App for WhispoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle keyboard events
        let needs_keyboard_repaint = self.state.handle_keyboard_events();

        // Only request repaint when recording or there are pending events
        if self.state.recording() || self.state.recording_shortcut() || needs_keyboard_repaint {
            ctx.request_repaint_after(std::time::Duration::from_millis(50));
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Whispo - Minimal Dictation App");

            ui.separator();

            // Show error message if any
            let mut open_settings = false;
            let mut retry_permissions = false;

            status::render_error_section(
                ui,
                self.state.error_message(),
                self.state.permissions_granted(),
                || open_settings = true,
                || retry_permissions = true,
            );

            if open_settings {
                self.state.open_accessibility_settings();
            }
            if retry_permissions {
                self.state.init_keyboard_listener();
            }

            // Recording status
            status::render_status_section(ui, self.state.recording(), self.state.permissions_granted());

            ui.separator();

            // Configuration section
            ui.collapsing("Configuration", |ui| {
                self.render_configuration(ui);
            });

            ui.separator();

            // Logs section
            logs::render_logs(ui, self.state.logs());
        });
    }
}

// UI rendering methods
impl WhispoApp {
    fn render_configuration(&mut self, ui: &mut egui::Ui) {
        // STT Provider config
        let mut stt_message = None;
        if self::config::render_stt_provider_config(ui, &mut self.state.config, |msg| {
            stt_message = Some(msg.to_string());
        }) {
            if let Some(msg) = stt_message {
                self.state.add_log(msg);
            }
            self.state.config_manager.save_async(self.state.config.clone());
        }

        ui.add_space(10.0);

        // API Keys config
        let mut api_message = None;
        if self::config::render_api_keys_config(ui, &mut self.state.config, |msg| {
            api_message = Some(msg.to_string());
        }) {
            if let Some(msg) = api_message {
                self.state.add_log(msg);
            }
            self.state.config_manager.save_async(self.state.config.clone());
        }

        ui.add_space(10.0);

        // Recording shortcut
        ui.group(|ui| {
            ui.label("Recording Shortcut:");

            // Presets
            shortcuts::render_shortcut_presets(ui, |shortcut| {
                self.state.apply_shortcut(shortcut);
            });

            ui.separator();

            // Shortcut editor
            let recording_shortcut = self.state.recording_shortcut();
            let action = shortcuts::handle_shortcut_editor_simple(
                ui,
                &mut self.state.config.recording_shortcut,
                recording_shortcut,
            );
            self.handle_shortcut_action(action);

            // Process recorded shortcut
            self.process_recorded_shortcut(ui);

            ui.separator();

            // Shortcut mode
            let mut mode_message = None;
            if shortcuts::render_shortcut_mode(ui, &mut self.state.config.recording_shortcut.mode, |msg| {
                mode_message = Some(msg.to_string())
            }) {
                if let Some(msg) = mode_message {
                    self.state.add_log(msg);
                }
                self.state.config_manager.save_async(self.state.config.clone());
                self.state.update_shortcut_listener();
            }

            ui.separator();

            // Visual editor
            let mut editor_message = None;
            let mut show_editor = self.state.show_visual_editor();
            if shortcuts::render_visual_editor(ui, &mut self.state.config.recording_shortcut, &mut show_editor, |msg| {
                editor_message = Some(msg.to_string())
            }) {
                if let Some(msg) = editor_message {
                    self.state.add_log(msg);
                }
                self.state.config_manager.save_async(self.state.config.clone());
                self.state.update_shortcut_listener();
            }
            self.state.set_show_visual_editor(show_editor);
        });
    }
}
