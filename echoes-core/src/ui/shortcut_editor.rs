use std::time::Instant;

use echoes_config::{ConflictInfo, ConflictSeverity, KeyCode, RecordingShortcut};
use egui::{Color32, FontId, Rect, Response, Sense, Stroke, Ui, Vec2};

pub struct ShortcutEditor<'a> {
    shortcut: &'a mut RecordingShortcut,
    is_recording: bool,
    recorded_shortcut: Option<RecordingShortcut>,
    recording_start_time: Option<Instant>,
    recording_timeout: f32,
}

#[allow(clippy::elidable_lifetime_names)]
impl<'a> ShortcutEditor<'a> {
    pub const fn new(shortcut: &'a mut RecordingShortcut) -> Self {
        Self {
            shortcut,
            is_recording: false,
            recorded_shortcut: None,
            recording_start_time: None,
            recording_timeout: 5.0,
        }
    }

    pub fn recording(mut self, is_recording: bool) -> Self {
        self.is_recording = is_recording;
        if is_recording && self.recording_start_time.is_none() {
            self.recording_start_time = Some(Instant::now());
        } else if !is_recording {
            self.recording_start_time = None;
        }
        self
    }

    #[allow(dead_code)]
    pub fn with_recorded(mut self, recorded: Option<RecordingShortcut>) -> Self {
        self.recorded_shortcut = recorded;
        self
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ShortcutEditorAction {
    None,
    StartRecording,
    CancelRecording,
    Reset,
}

#[allow(clippy::elidable_lifetime_names)]
impl<'a> ShortcutEditor<'a> {
    pub fn show(mut self, ui: &mut Ui) -> (Response, ShortcutEditorAction) {
        let desired_size = Vec2::new(ui.available_width(), 120.0);
        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click());

        if ui.is_rect_visible(rect) {
            if self.is_recording {
                ui.ctx().request_repaint_after(std::time::Duration::from_millis(16));
            }

            let painter = ui.painter();
            self.draw_background(painter, rect);
            self.draw_text_content(painter, rect);
            self.draw_border_and_effects(painter, rect);
        }

        if let Some(ref recorded) = self.recorded_shortcut {
            *self.shortcut = recorded.clone();
        }

        let action = self.handle_user_actions(&response);
        (response, action)
    }

    fn draw_background(&self, painter: &egui::Painter, rect: Rect) {
        let bg_color = if self.is_recording {
            Color32::from_rgb(40, 40, 60)
        } else {
            Color32::from_rgb(30, 30, 30)
        };
        painter.rect_filled(rect, 4.0, bg_color);
    }

    fn draw_border_and_effects(&self, painter: &egui::Painter, rect: Rect) {
        if self.is_recording {
            let elapsed = self
                .recording_start_time
                .map_or(0.0, |start| start.elapsed().as_secs_f32());
            let pulse = f32::midpoint((elapsed * 3.0).sin(), 1.0);
            let border_width = pulse.mul_add(2.0, 1.0);
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let border_color = Color32::from_rgb(
                100 + (pulse * 50.0).clamp(0.0, 155.0) as u8,
                100 + (pulse * 50.0).clamp(0.0, 155.0) as u8,
                200 + (pulse * 55.0).clamp(0.0, 55.0) as u8,
            );
            painter.rect_stroke(
                rect,
                4.0,
                Stroke::new(border_width, border_color),
                egui::epaint::StrokeKind::Middle,
            );

            self.draw_timeout_progress_bar(painter, rect, elapsed);
        } else {
            let border_color = Color32::from_rgb(60, 60, 60);
            painter.rect_stroke(
                rect,
                4.0,
                Stroke::new(1.0, border_color),
                egui::epaint::StrokeKind::Middle,
            );
        }
    }

    fn draw_timeout_progress_bar(&self, painter: &egui::Painter, rect: Rect, elapsed: f32) {
        if elapsed < self.recording_timeout {
            let progress = 1.0 - (elapsed / self.recording_timeout);
            let progress_rect = Rect::from_min_size(
                rect.min + Vec2::new(0.0, rect.height() - 4.0),
                Vec2::new(rect.width() * progress, 4.0),
            );
            painter.rect_filled(progress_rect, 0.0, Color32::from_rgb(100, 100, 200));
        }
    }

    fn draw_text_content(&self, painter: &egui::Painter, rect: Rect) {
        self.draw_title_text(painter, rect);
        self.draw_shortcut_text(painter, rect);
        self.draw_hint_text(painter, rect);
        self.draw_instruction_text(painter, rect);
    }

    fn draw_title_text(&self, painter: &egui::Painter, rect: Rect) {
        let title_pos = rect.min + Vec2::new(10.0, 10.0);
        let title_text = if self.is_recording {
            "Press your desired shortcut..."
        } else {
            "Current Shortcut"
        };
        painter.text(
            title_pos,
            egui::Align2::LEFT_TOP,
            title_text,
            FontId::proportional(14.0),
            Color32::from_rgb(200, 200, 200),
        );
    }

    fn draw_shortcut_text(&self, painter: &egui::Painter, rect: Rect) {
        let shortcut_text = self
            .recorded_shortcut
            .as_ref()
            .map_or_else(|| format_shortcut(self.shortcut), format_shortcut);

        let shortcut_pos = rect.center() - Vec2::new(0.0, 10.0);
        let text_color = if self.is_recording && self.recorded_shortcut.is_some() {
            Color32::from_rgb(150, 255, 150)
        } else {
            Color32::from_rgb(255, 255, 255)
        };

        painter.text(
            shortcut_pos,
            egui::Align2::CENTER_CENTER,
            &shortcut_text,
            FontId::proportional(24.0),
            text_color,
        );
    }

    fn draw_hint_text(&self, painter: &egui::Painter, rect: Rect) {
        if self.is_recording && self.recorded_shortcut.is_some() {
            let keys_hint_pos = rect.center() + Vec2::new(0.0, 20.0);
            painter.text(
                keys_hint_pos,
                egui::Align2::CENTER_CENTER,
                "Release all keys to set this shortcut",
                FontId::proportional(12.0),
                Color32::from_rgb(180, 180, 180),
            );
        }
    }

    fn draw_instruction_text(&self, painter: &egui::Painter, rect: Rect) {
        let instruction_pos = rect.max - Vec2::new(10.0, 30.0);
        let instruction_text = if self.is_recording {
            "Press ESC or right-click to cancel"
        } else {
            "Click to record new shortcut"
        };
        painter.text(
            instruction_pos,
            egui::Align2::RIGHT_BOTTOM,
            instruction_text,
            FontId::proportional(12.0),
            Color32::from_rgb(150, 150, 150),
        );

        let extra_instruction_pos = rect.max - Vec2::new(10.0, 10.0);
        let extra_text = if self.is_recording {
            "Release keys to confirm shortcut"
        } else {
            "Right-click to reset to Ctrl"
        };
        painter.text(
            extra_instruction_pos,
            egui::Align2::RIGHT_BOTTOM,
            extra_text,
            FontId::proportional(10.0),
            Color32::from_rgb(120, 120, 120),
        );
    }

    fn handle_user_actions(&mut self, response: &Response) -> ShortcutEditorAction {
        if response.clicked() && !self.is_recording {
            ShortcutEditorAction::StartRecording
        } else if response.secondary_clicked() {
            if self.is_recording {
                ShortcutEditorAction::CancelRecording
            } else {
                self.shortcut.key = KeyCode::ControlLeft;
                self.shortcut.modifiers.clear();
                ShortcutEditorAction::Reset
            }
        } else {
            ShortcutEditorAction::None
        }
    }
}

fn format_shortcut(shortcut: &RecordingShortcut) -> String {
    let mut parts = Vec::new();

    for modifier in &shortcut.modifiers {
        parts.push(format_key(*modifier));
    }

    parts.push(format_key(shortcut.key));

    parts.join(" + ")
}

fn format_key(key: KeyCode) -> String {
    match key {
        KeyCode::ControlLeft | KeyCode::ControlRight => "Ctrl".to_string(),
        KeyCode::ShiftLeft | KeyCode::ShiftRight => "Shift".to_string(),
        KeyCode::Alt => "Alt".to_string(),
        KeyCode::AltGr => "AltGr".to_string(),
        KeyCode::MetaLeft | KeyCode::MetaRight => {
            if cfg!(target_os = "macos") {
                "Cmd".to_string()
            } else {
                "Win".to_string()
            }
        }
        KeyCode::Space => "Space".to_string(),
        KeyCode::Tab => "Tab".to_string(),
        KeyCode::CapsLock => "CapsLock".to_string(),
        KeyCode::Escape => "Esc".to_string(),
        KeyCode::Backspace => "Backspace".to_string(),
        KeyCode::Return => "Enter".to_string(),
        KeyCode::Insert => "Insert".to_string(),
        KeyCode::Delete => "Delete".to_string(),
        KeyCode::Home => "Home".to_string(),
        KeyCode::End => "End".to_string(),
        KeyCode::PageUp => "PageUp".to_string(),
        KeyCode::PageDown => "PageDown".to_string(),
        KeyCode::BackQuote => "`".to_string(),
        KeyCode::Num1 => "1".to_string(),
        KeyCode::Num2 => "2".to_string(),
        KeyCode::Num3 => "3".to_string(),
        KeyCode::Num4 => "4".to_string(),
        KeyCode::Num5 => "5".to_string(),
        KeyCode::Num6 => "6".to_string(),
        KeyCode::Num7 => "7".to_string(),
        KeyCode::Num8 => "8".to_string(),
        KeyCode::Num9 => "9".to_string(),
        KeyCode::Num0 => "0".to_string(),
        KeyCode::Minus => "-".to_string(),
        KeyCode::Equal => "=".to_string(),
        KeyCode::Q => "Q".to_string(),
        KeyCode::W => "W".to_string(),
        KeyCode::E => "E".to_string(),
        KeyCode::R => "R".to_string(),
        KeyCode::T => "T".to_string(),
        KeyCode::Y => "Y".to_string(),
        KeyCode::U => "U".to_string(),
        KeyCode::I => "I".to_string(),
        KeyCode::O => "O".to_string(),
        KeyCode::P => "P".to_string(),
        KeyCode::LeftBracket => "[".to_string(),
        KeyCode::RightBracket => "]".to_string(),
        KeyCode::A => "A".to_string(),
        KeyCode::S => "S".to_string(),
        KeyCode::D => "D".to_string(),
        KeyCode::F => "F".to_string(),
        KeyCode::G => "G".to_string(),
        KeyCode::H => "H".to_string(),
        KeyCode::J => "J".to_string(),
        KeyCode::K => "K".to_string(),
        KeyCode::L => "L".to_string(),
        KeyCode::SemiColon => ";".to_string(),
        KeyCode::Quote => "'".to_string(),
        KeyCode::BackSlash => "\\".to_string(),
        KeyCode::Z => "Z".to_string(),
        KeyCode::X => "X".to_string(),
        KeyCode::C => "C".to_string(),
        KeyCode::V => "V".to_string(),
        KeyCode::B => "B".to_string(),
        KeyCode::N => "N".to_string(),
        KeyCode::M => "M".to_string(),
        KeyCode::Comma => ",".to_string(),
        KeyCode::Dot => ".".to_string(),
        KeyCode::Slash => "/".to_string(),
        KeyCode::F1 => "F1".to_string(),
        KeyCode::F2 => "F2".to_string(),
        KeyCode::F3 => "F3".to_string(),
        KeyCode::F4 => "F4".to_string(),
        KeyCode::F5 => "F5".to_string(),
        KeyCode::F6 => "F6".to_string(),
        KeyCode::F7 => "F7".to_string(),
        KeyCode::F8 => "F8".to_string(),
        KeyCode::F9 => "F9".to_string(),
        KeyCode::F10 => "F10".to_string(),
        KeyCode::F11 => "F11".to_string(),
        KeyCode::F12 => "F12".to_string(),
        KeyCode::LeftArrow => "←".to_string(),
        KeyCode::RightArrow => "→".to_string(),
        KeyCode::UpArrow => "↑".to_string(),
        KeyCode::DownArrow => "↓".to_string(),
    }
}

pub struct ShortcutBuilder<'a> {
    shortcut: &'a mut RecordingShortcut,
}

impl<'a> ShortcutBuilder<'a> {
    pub const fn new(shortcut: &'a mut RecordingShortcut) -> Self {
        Self { shortcut }
    }

    pub fn show(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.label("Build shortcut visually:");
            self.draw_modifier_checkboxes(ui);
            self.draw_main_key_selection(ui);
        });
    }

    fn draw_modifier_checkboxes(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Modifiers:");

            self.modifier_checkbox(
                ui,
                "Ctrl",
                KeyCode::ControlLeft,
                &[KeyCode::ControlLeft, KeyCode::ControlRight],
            );
            self.modifier_checkbox(
                ui,
                "Shift",
                KeyCode::ShiftLeft,
                &[KeyCode::ShiftLeft, KeyCode::ShiftRight],
            );
            self.modifier_checkbox(ui, "Alt", KeyCode::Alt, &[KeyCode::Alt, KeyCode::AltGr]);

            if cfg!(target_os = "macos") {
                self.modifier_checkbox(ui, "Cmd", KeyCode::MetaLeft, &[KeyCode::MetaLeft, KeyCode::MetaRight]);
            } else {
                self.modifier_checkbox(ui, "Win", KeyCode::MetaLeft, &[KeyCode::MetaLeft, KeyCode::MetaRight]);
            }
        });
    }

    fn modifier_checkbox(&mut self, ui: &mut Ui, label: &str, primary_key: KeyCode, all_variants: &[KeyCode]) {
        let mut has_modifier = self.shortcut.modifiers.contains(&primary_key);
        if ui.checkbox(&mut has_modifier, label).changed() {
            if has_modifier {
                if !self.shortcut.modifiers.contains(&primary_key) {
                    self.shortcut.modifiers.push(primary_key);
                }
            } else {
                self.shortcut.modifiers.retain(|k| !all_variants.contains(k));
            }
        }
    }

    fn draw_main_key_selection(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Main key:");

            let common_keys = vec![
                ("Space", KeyCode::Space),
                ("Enter", KeyCode::Return),
                ("Tab", KeyCode::Tab),
                ("Escape", KeyCode::Escape),
                ("/", KeyCode::Slash),
                (".", KeyCode::Dot),
                (",", KeyCode::Comma),
                ("A", KeyCode::A),
                ("S", KeyCode::S),
                ("D", KeyCode::D),
                ("F", KeyCode::F),
                ("R", KeyCode::R),
                ("X", KeyCode::X),
                ("C", KeyCode::C),
                ("V", KeyCode::V),
            ];

            let current_key_str = format_key(self.shortcut.key);
            egui::ComboBox::from_label("")
                .selected_text(&current_key_str)
                .show_ui(ui, |ui| {
                    for (label, key) in common_keys {
                        ui.selectable_value(&mut self.shortcut.key, key, label);
                    }
                });
        });
    }
}

pub struct ConflictDisplay<'a> {
    conflicts: &'a [ConflictInfo],
}

impl<'a> ConflictDisplay<'a> {
    pub const fn new(conflicts: &'a [ConflictInfo]) -> Self {
        Self { conflicts }
    }

    pub fn show(&self, ui: &mut Ui) {
        if self.conflicts.is_empty() {
            return;
        }

        ui.separator();
        ui.label("⚠️ Shortcut Conflicts:");

        for conflict in self.conflicts {
            let (icon, color) = match conflict.severity {
                ConflictSeverity::Error => ("🚫", Color32::from_rgb(255, 100, 100)),
                ConflictSeverity::Warning => ("⚠️", Color32::from_rgb(255, 200, 100)),
                ConflictSeverity::Info => ("ℹ️", Color32::from_rgb(100, 150, 255)),
            };

            ui.horizontal(|ui| {
                ui.label(icon);
                ui.colored_label(color, &conflict.description);
            });

            if let Some(suggestion) = &conflict.suggestion {
                ui.horizontal(|ui| {
                    ui.add_space(20.0);
                    ui.colored_label(Color32::from_rgb(180, 180, 180), format!("💡 {suggestion}"));
                });
            }

            ui.add_space(5.0);
        }
    }
}
