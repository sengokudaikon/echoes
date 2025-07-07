use echoes_config::{KeyCode, RecordingShortcut, ShortcutMode};
use eframe::egui;

use super::shortcut_editor::{ConflictDisplay, ShortcutBuilder, ShortcutEditor, ShortcutEditorAction};

/// Context for shortcut operations
#[allow(dead_code)]
pub struct ShortcutContext<'a> {
    pub config: &'a mut RecordingShortcut,
    pub is_recording: &'a mut bool,
    pub recorded: &'a mut Option<RecordingShortcut>,
}

/// Renders the shortcut presets UI
pub fn render_shortcut_presets(ui: &mut egui::Ui, mut on_apply: impl FnMut(RecordingShortcut)) {
    ui.label("Quick presets:");
    ui.horizontal(|ui| {
        if ui.button("Hold Ctrl").clicked() {
            on_apply(RecordingShortcut {
                mode: ShortcutMode::Hold,
                key: KeyCode::ControlLeft,
                modifiers: vec![],
            });
        }
        if ui.button("Ctrl+/").clicked() {
            on_apply(RecordingShortcut {
                mode: ShortcutMode::Toggle,
                key: KeyCode::Slash,
                modifiers: vec![KeyCode::ControlLeft],
            });
        }
        if ui.button("Cmd+Space").clicked() {
            on_apply(RecordingShortcut {
                mode: ShortcutMode::Toggle,
                key: KeyCode::Space,
                modifiers: vec![KeyCode::MetaLeft],
            });
        }
    });
}

/// Handles the shortcut editor UI and returns actions to take
#[allow(dead_code)]
pub fn handle_shortcut_editor(ui: &mut egui::Ui, ctx: &mut ShortcutContext<'_>) -> ShortcutEditorAction {
    // Shortcut editor
    let (_editor_response, editor_action) = ShortcutEditor::new(ctx.config)
        .recording(*ctx.is_recording)
        .with_recorded(ctx.recorded.clone())
        .show(ui);

    // Show conflicts for current shortcut
    let conflicts = ctx.config.check_conflicts();
    ConflictDisplay::new(&conflicts).show(ui);

    editor_action
}

/// Simplified shortcut editor handler for composition pattern
pub fn handle_shortcut_editor_simple(
    ui: &mut egui::Ui, config_shortcut: &mut RecordingShortcut, recording_shortcut: bool,
) -> ShortcutEditorAction {
    // Shortcut editor
    let (_editor_response, editor_action) = ShortcutEditor::new(config_shortcut)
        .recording(recording_shortcut)
        .show(ui);

    // Show conflicts for current shortcut
    let conflicts = config_shortcut.check_conflicts();
    ConflictDisplay::new(&conflicts).show(ui);

    editor_action
}

/// Renders the shortcut mode selection UI
pub fn render_shortcut_mode(ui: &mut egui::Ui, mode: &mut ShortcutMode, mut on_change: impl FnMut(&str)) -> bool {
    let mut changed = false;

    ui.horizontal(|ui| {
        ui.label("Mode:");
        if ui.radio_value(mode, ShortcutMode::Hold, "Hold").clicked() {
            on_change("Changed mode to Hold");
            changed = true;
        }
        if ui.radio_value(mode, ShortcutMode::Toggle, "Toggle").clicked() {
            on_change("Changed mode to Toggle");
            changed = true;
        }
    });

    changed
}

/// Renders the visual editor UI
pub fn render_visual_editor(
    ui: &mut egui::Ui, shortcut: &mut RecordingShortcut, show_visual_editor: &mut bool, mut on_change: impl FnMut(&str),
) -> bool {
    let mut changed = false;

    if ui
        .button(if *show_visual_editor {
            "Hide Visual Editor"
        } else {
            "Show Visual Editor"
        })
        .clicked()
    {
        *show_visual_editor = !*show_visual_editor;
    }

    if *show_visual_editor {
        ui.separator();
        let original_key = shortcut.key;
        let original_modifiers_len = shortcut.modifiers.len();

        let mut builder = ShortcutBuilder::new(shortcut);
        builder.show(ui);

        // Check if shortcut changed by comparing key fields
        let shortcut_changed = shortcut.key != original_key || shortcut.modifiers.len() != original_modifiers_len;

        if shortcut_changed {
            if let Err(err) = shortcut.validate() {
                ui.colored_label(egui::Color32::YELLOW, format!("⚠️ {err}"));
            } else {
                on_change("Updated shortcut from visual editor");
                changed = true;
            }
        }
    }

    changed
}

/// Formats a shortcut for display
pub fn format_shortcut(shortcut: &RecordingShortcut) -> String {
    shortcut.format_display()
}
