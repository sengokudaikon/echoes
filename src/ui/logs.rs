use eframe::egui;

/// Renders the logs section UI
pub fn render_logs(ui: &mut egui::Ui, logs: &[String]) {
    ui.collapsing("Logs", |ui| {
        egui::ScrollArea::vertical()
            .max_height(300.0)
            .show(ui, |ui| {
                for log in logs.iter().rev() {
                    ui.label(log);
                }
            });
    });
}
