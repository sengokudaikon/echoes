use crate::config::{Config, SttProvider};
use eframe::egui;

/// Renders the STT provider configuration UI
pub fn render_stt_provider_config(
    ui: &mut egui::Ui,
    config: &mut Config,
    mut on_change: impl FnMut(&str),
) -> bool {
    let mut changed = false;

    ui.group(|ui| {
        ui.label("STT Provider:");
        ui.horizontal(|ui| {
            if ui
                .radio(matches!(config.stt_provider, SttProvider::OpenAI), "OpenAI")
                .clicked()
            {
                config.stt_provider = SttProvider::OpenAI;
                on_change("Changed STT provider to OpenAI");
                changed = true;
            }
            if ui
                .radio(matches!(config.stt_provider, SttProvider::Groq), "Groq")
                .clicked()
            {
                config.stt_provider = SttProvider::Groq;
                on_change("Changed STT provider to Groq");
                changed = true;
            }
            #[cfg(target_os = "macos")]
            if ui
                .radio(
                    matches!(config.stt_provider, SttProvider::LightningWhisper),
                    "Lightning Whisper",
                )
                .clicked()
            {
                config.stt_provider = SttProvider::LightningWhisper;
                on_change("Changed STT provider to Lightning Whisper");
                changed = true;
            }
        });
    });

    changed
}

/// Renders the API keys configuration UI
pub fn render_api_keys_config(
    ui: &mut egui::Ui,
    config: &mut Config,
    mut on_change: impl FnMut(&str),
) -> bool {
    let mut changed = false;

    ui.group(|ui| {
        ui.label("API Keys:");

        ui.horizontal(|ui| {
            ui.label("OpenAI:");
            let mut temp_key = String::new();
            let key_to_edit = match &mut config.openai_api_key {
                Some(key) => key,
                None => &mut temp_key,
            };

            if ui.text_edit_singleline(key_to_edit).changed() {
                if key_to_edit.is_empty() {
                    config.openai_api_key = None;
                } else if config.openai_api_key.is_none() {
                    config.openai_api_key = Some(temp_key);
                }
                on_change("Updated OpenAI API key");
                changed = true;
            }
        });

        ui.horizontal(|ui| {
            ui.label("Groq:");
            let mut temp_key = String::new();
            let key_to_edit = match &mut config.groq_api_key {
                Some(key) => key,
                None => &mut temp_key,
            };

            if ui.text_edit_singleline(key_to_edit).changed() {
                if key_to_edit.is_empty() {
                    config.groq_api_key = None;
                } else if config.groq_api_key.is_none() {
                    config.groq_api_key = Some(temp_key);
                }
                on_change("Updated Groq API key");
                changed = true;
            }
        });
    });

    changed
}
