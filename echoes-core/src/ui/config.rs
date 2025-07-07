use echoes_config::{Config, SttProvider};
use eframe::egui;

/// Configuration field types for form components
#[derive(Debug, Clone)]
struct FieldConfig<'a> {
    label: &'a str,
    description: &'a str,
    hint: Option<&'a str>,
    change_message: &'a str,
}

/// Renders the STT provider configuration UI
pub fn render_stt_provider_config(ui: &mut egui::Ui, config: &mut Config, mut on_change: impl FnMut(&str)) -> bool {
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
            if ui
                .radio(
                    matches!(config.stt_provider, SttProvider::LocalWhisper),
                    "Local Whisper",
                )
                .clicked()
            {
                config.stt_provider = SttProvider::LocalWhisper;
                on_change("Changed STT provider to Local Whisper");
                changed = true;
            }
        });
    });

    changed
}

/// Functional component for optional text field with change tracking
fn render_optional_text_field(
    ui: &mut egui::Ui, config: FieldConfig, value: &mut Option<String>, password: bool, mut on_change: impl FnMut(&str),
) -> bool {
    ui.vertical(|ui| {
        ui.label(config.label);
        ui.small(config.description);

        let mut temp_value = String::new();
        let value_to_edit = match value {
            Some(v) => v,
            None => &mut temp_value,
        };

        let mut text_edit = egui::TextEdit::singleline(value_to_edit);
        if password {
            text_edit = text_edit.password(true);
        }
        if let Some(hint) = config.hint {
            text_edit = text_edit.hint_text(hint);
        }

        let response = ui.add(text_edit);
        if response.changed() {
            if value_to_edit.is_empty() {
                *value = None;
            } else if value.is_none() {
                *value = Some(temp_value);
            }
            on_change(config.change_message);
            return true;
        }
        false
    })
    .inner
}

/// Functional component for optional multiline text field
fn render_optional_multiline_field(
    ui: &mut egui::Ui, config: FieldConfig, value: &mut Option<String>, rows: usize, mut on_change: impl FnMut(&str),
) -> bool {
    ui.vertical(|ui| {
        ui.label(config.label);
        ui.small(config.description);

        let mut temp_value = String::new();
        let value_to_edit = match value {
            Some(v) => v,
            None => &mut temp_value,
        };

        let mut text_edit = egui::TextEdit::multiline(value_to_edit).desired_rows(rows);
        if let Some(hint) = config.hint {
            text_edit = text_edit.hint_text(hint);
        }

        let response = ui.add(text_edit);
        if response.changed() {
            if value_to_edit.is_empty() {
                *value = None;
            } else if value.is_none() {
                *value = Some(temp_value);
            }
            on_change(config.change_message);
            return true;
        }
        false
    })
    .inner
}

/// Renders OpenAI STT provider configuration using functional components
fn render_openai_settings(ui: &mut egui::Ui, config: &mut Config, mut on_change: impl FnMut(&str)) -> bool {
    let mut changed = false;

    changed |= render_optional_text_field(
        ui,
        FieldConfig {
            label: "API Key:",
            description: "Your OpenAI API key",
            hint: None,
            change_message: "Updated OpenAI API key",
        },
        &mut config.openai_api_key,
        true,
        &mut on_change,
    );

    changed |= render_optional_text_field(
        ui,
        FieldConfig {
            label: "Base URL:",
            description: "Default: https://api.openai.com/v1 (leave empty for default)",
            hint: Some("https://api.openai.com/v1"),
            change_message: "Updated OpenAI base URL",
        },
        &mut config.openai_base_url,
        false,
        &mut on_change,
    );

    changed |= render_optional_text_field(
        ui,
        FieldConfig {
            label: "Model:",
            description: "Default: whisper-1 (available: whisper-1)",
            hint: Some("whisper-1"),
            change_message: "Updated OpenAI STT model",
        },
        &mut config.openai_stt_model,
        false,
        &mut on_change,
    );

    changed |= render_optional_multiline_field(
        ui,
        FieldConfig {
            label: "Prompt (optional):",
            description: "Helps guide transcription for specific context, terminology, or formatting",
            hint: Some("e.g., 'The following is a meeting transcript with technical terms...'"),
            change_message: "Updated OpenAI STT prompt",
        },
        &mut config.openai_stt_prompt,
        3,
        &mut on_change,
    );

    changed
}

/// Renders Groq STT provider configuration using functional components
fn render_groq_settings(ui: &mut egui::Ui, config: &mut Config, mut on_change: impl FnMut(&str)) -> bool {
    let mut changed = false;

    changed |= render_optional_text_field(
        ui,
        FieldConfig {
            label: "API Key:",
            description: "Your Groq API key",
            hint: None,
            change_message: "Updated Groq API key",
        },
        &mut config.groq_api_key,
        true,
        &mut on_change,
    );

    changed |= render_optional_text_field(
        ui,
        FieldConfig {
            label: "Base URL:",
            description: "Default: https://api.groq.com/openai/v1 (leave empty for default)",
            hint: Some("https://api.groq.com/openai/v1"),
            change_message: "Updated Groq base URL",
        },
        &mut config.groq_base_url,
        false,
        &mut on_change,
    );

    changed |= render_optional_text_field(
        ui,
        FieldConfig {
            label: "Model:",
            description: "Default: whisper-large-v3 (available: whisper-large-v3, distil-whisper-large-v3-en)",
            hint: Some("whisper-large-v3"),
            change_message: "Updated Groq STT model",
        },
        &mut config.groq_stt_model,
        false,
        &mut on_change,
    );

    changed |= render_optional_multiline_field(
        ui,
        FieldConfig {
            label: "Prompt (optional):",
            description: "Helps guide transcription for specific context, terminology, or formatting",
            hint: Some("e.g., 'The following is a meeting transcript with technical terms...'"),
            change_message: "Updated Groq STT prompt",
        },
        &mut config.groq_stt_prompt,
        3,
        &mut on_change,
    );

    changed
}

/// Renders Local Whisper STT provider configuration
fn render_local_whisper_settings(ui: &mut egui::Ui, config: &mut Config, mut on_change: impl FnMut(&str)) -> bool {
    let mut changed = false;

    ui.vertical(|ui| {
        ui.label("Model:");
        ui.small("Select the Whisper model to use (larger models are more accurate but slower)");

        let mut model_changed = false;
        egui::ComboBox::from_label("Whisper Model")
            .selected_text(format!("{:?}", config.local_whisper.model))
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut config.local_whisper.model,
                    echoes_config::WhisperModel::Tiny,
                    "Tiny",
                );
                ui.selectable_value(
                    &mut config.local_whisper.model,
                    echoes_config::WhisperModel::TinyEn,
                    "Tiny (English)",
                );
                ui.selectable_value(
                    &mut config.local_whisper.model,
                    echoes_config::WhisperModel::Base,
                    "Base",
                );
                ui.selectable_value(
                    &mut config.local_whisper.model,
                    echoes_config::WhisperModel::BaseEn,
                    "Base (English)",
                );
                ui.selectable_value(
                    &mut config.local_whisper.model,
                    echoes_config::WhisperModel::Small,
                    "Small",
                );
                ui.selectable_value(
                    &mut config.local_whisper.model,
                    echoes_config::WhisperModel::SmallEn,
                    "Small (English)",
                );
                ui.selectable_value(
                    &mut config.local_whisper.model,
                    echoes_config::WhisperModel::Medium,
                    "Medium",
                );
                ui.selectable_value(
                    &mut config.local_whisper.model,
                    echoes_config::WhisperModel::MediumEn,
                    "Medium (English)",
                );
                ui.selectable_value(
                    &mut config.local_whisper.model,
                    echoes_config::WhisperModel::LargeV1,
                    "Large V1",
                );
                ui.selectable_value(
                    &mut config.local_whisper.model,
                    echoes_config::WhisperModel::LargeV2,
                    "Large V2",
                );
                ui.selectable_value(
                    &mut config.local_whisper.model,
                    echoes_config::WhisperModel::LargeV3,
                    "Large V3",
                );
            });

        if model_changed {
            on_change("Updated Local Whisper model");
            changed = true;
        }
    });

    ui.vertical(|ui| {
        ui.label("Model Path (optional):");
        ui.small("Custom path to a local model file (leave empty to auto-download)");

        let mut temp_path = String::new();
        let path_to_edit = match &config.local_whisper.model_path {
            Some(path) => path.to_string_lossy().to_string(),
            None => temp_path.clone(),
        };

        let mut path_input = path_to_edit.clone();
        let response = ui.add(egui::TextEdit::singleline(&mut path_input).hint_text("/path/to/model.bin"));

        if response.changed() {
            if path_input.is_empty() {
                config.local_whisper.model_path = None;
            } else {
                config.local_whisper.model_path = Some(std::path::PathBuf::from(path_input));
            }
            on_change("Updated Local Whisper model path");
            changed = true;
        }
    });

    changed
}

/// Renders the STT provider-specific configuration UI
pub fn render_stt_provider_settings(ui: &mut egui::Ui, config: &mut Config, on_change: impl FnMut(&str)) -> bool {
    ui.group(|ui| {
        ui.label("STT Provider Settings:");

        match config.stt_provider {
            SttProvider::OpenAI => render_openai_settings(ui, config, on_change),
            SttProvider::Groq => render_groq_settings(ui, config, on_change),
            SttProvider::LocalWhisper => render_local_whisper_settings(ui, config, on_change),
        }
    })
    .inner
}
