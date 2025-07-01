use eframe::egui;

/// Renders the status section showing recording state and permissions
pub fn render_status_section(ui: &mut egui::Ui, recording: bool, permissions_granted: bool) {
    ui.horizontal(|ui| {
        ui.label("Status:");
        if recording {
            ui.colored_label(egui::Color32::RED, "● RECORDING");
        } else if permissions_granted {
            ui.colored_label(egui::Color32::GREEN, "● Ready");
        } else {
            ui.colored_label(egui::Color32::YELLOW, "● Permissions Required");
        }
    });
}

/// Renders error messages and permission-related UI
pub fn render_error_section(
    ui: &mut egui::Ui,
    error_message: &Option<String>,
    permissions_granted: bool,
    mut on_open_settings: impl FnMut(),
    mut on_retry_permissions: impl FnMut(),
) {
    if let Some(error) = error_message {
        ui.colored_label(egui::Color32::RED, format!("⚠️ {error}"));

        // Add buttons for permissions
        if !permissions_granted {
            ui.horizontal(|ui| {
                if ui.button("Open System Settings").clicked() {
                    on_open_settings();
                }

                if ui.button("Retry Permissions Check").clicked() {
                    on_retry_permissions();
                }
            });
        }
        ui.separator();
    }
}
