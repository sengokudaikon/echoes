use echoes_config::{Config, SttProvider};
use eframe::egui;

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
pub fn render_api_keys_config(ui: &mut egui::Ui, config: &mut Config, mut on_change: impl FnMut(&str)) -> bool {
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

/// Renders the STT provider-specific configuration UI
pub fn render_stt_provider_settings(ui: &mut egui::Ui, config: &mut Config, mut on_change: impl FnMut(&str)) -> bool {
    let mut changed = false;

    ui.group(|ui| {
        ui.label("STT Provider Settings:");

        match config.stt_provider {
            SttProvider::OpenAI => {
                ui.horizontal(|ui| {
                    ui.label("Base URL:");
                    let mut temp_url = String::new();
                    let url_to_edit = match &mut config.openai_base_url {
                        Some(url) => url,
                        None => &mut temp_url,
                    };

                    if ui.text_edit_singleline(url_to_edit).changed() {
                        if url_to_edit.is_empty() {
                            config.openai_base_url = None;
                        } else if config.openai_base_url.is_none() {
                            config.openai_base_url = Some(temp_url);
                        }
                        on_change("Updated OpenAI base URL");
                        changed = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Model:");
                    let mut temp_model = String::new();
                    let model_to_edit = match &mut config.openai_stt_model {
                        Some(model) => model,
                        None => &mut temp_model,
                    };

                    if ui.text_edit_singleline(model_to_edit).changed() {
                        if model_to_edit.is_empty() {
                            config.openai_stt_model = None;
                        } else if config.openai_stt_model.is_none() {
                            config.openai_stt_model = Some(temp_model);
                        }
                        on_change("Updated OpenAI STT model");
                        changed = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Prompt:");
                    let mut temp_prompt = String::new();
                    let prompt_to_edit = match &mut config.openai_stt_prompt {
                        Some(prompt) => prompt,
                        None => &mut temp_prompt,
                    };

                    if ui.text_edit_multiline(prompt_to_edit).changed() {
                        if prompt_to_edit.is_empty() {
                            config.openai_stt_prompt = None;
                        } else if config.openai_stt_prompt.is_none() {
                            config.openai_stt_prompt = Some(temp_prompt);
                        }
                        on_change("Updated OpenAI STT prompt");
                        changed = true;
                    }
                });
            }
            SttProvider::Groq => {
                ui.horizontal(|ui| {
                    ui.label("Base URL:");
                    let mut temp_url = String::new();
                    let url_to_edit = match &mut config.groq_base_url {
                        Some(url) => url,
                        None => &mut temp_url,
                    };

                    if ui.text_edit_singleline(url_to_edit).changed() {
                        if url_to_edit.is_empty() {
                            config.groq_base_url = None;
                        } else if config.groq_base_url.is_none() {
                            config.groq_base_url = Some(temp_url);
                        }
                        on_change("Updated Groq base URL");
                        changed = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Model:");
                    let mut temp_model = String::new();
                    let model_to_edit = match &mut config.groq_stt_model {
                        Some(model) => model,
                        None => &mut temp_model,
                    };

                    if ui.text_edit_singleline(model_to_edit).changed() {
                        if model_to_edit.is_empty() {
                            config.groq_stt_model = None;
                        } else if config.groq_stt_model.is_none() {
                            config.groq_stt_model = Some(temp_model);
                        }
                        on_change("Updated Groq STT model");
                        changed = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Prompt:");
                    let mut temp_prompt = String::new();
                    let prompt_to_edit = match &mut config.groq_stt_prompt {
                        Some(prompt) => prompt,
                        None => &mut temp_prompt,
                    };

                    if ui.text_edit_multiline(prompt_to_edit).changed() {
                        if prompt_to_edit.is_empty() {
                            config.groq_stt_prompt = None;
                        } else if config.groq_stt_prompt.is_none() {
                            config.groq_stt_prompt = Some(temp_prompt);
                        }
                        on_change("Updated Groq STT prompt");
                        changed = true;
                    }
                });
            }
            SttProvider::LocalWhisper => {
                ui.label("Local Whisper settings will be added here");
            }
            #[cfg(target_os = "macos")]
            SttProvider::LightningWhisper => {
                ui.label("Lightning Whisper settings will be added here");
            }
        }
    });

    changed
}
