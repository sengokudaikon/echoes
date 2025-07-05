use super::SttProvider;
use crate::config::{LocalWhisperConfig, WhisperModel};
use anyhow::{Context, Result};
use std::path::PathBuf;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

pub struct LocalWhisperStt {
    context: WhisperContext,
}

impl LocalWhisperStt {
    pub fn new(config: &LocalWhisperConfig) -> Result<Self> {
        let model_path = if let Some(path) = &config.model_path {
            path.clone()
        } else {
            Self::get_model_path(config)?
        };

        let ctx_params = WhisperContextParameters::default();
        let context = WhisperContext::new_with_params(&model_path.to_string_lossy(), ctx_params)
            .context("Failed to create Whisper context")?;

        Ok(Self { context })
    }

    fn get_model_path(config: &LocalWhisperConfig) -> Result<PathBuf> {
        // Use a standard location for models
        let mut path = directories::ProjectDirs::from("com", "echoes", "echoes")
            .context("Failed to get project directories")?
            .data_dir()
            .to_path_buf();
        
        path.push("models");
        std::fs::create_dir_all(&path)?;
        
        let model_filename = match config.model {
            WhisperModel::Tiny => "ggml-tiny.bin",
            WhisperModel::TinyEn => "ggml-tiny.en.bin",
            WhisperModel::Base => "ggml-base.bin",
            WhisperModel::BaseEn => "ggml-base.en.bin",
            WhisperModel::Small => "ggml-small.bin",
            WhisperModel::SmallEn => "ggml-small.en.bin",
            WhisperModel::Medium => "ggml-medium.bin",
            WhisperModel::MediumEn => "ggml-medium.en.bin",
            WhisperModel::LargeV1 => "ggml-large-v1.bin",
            WhisperModel::LargeV2 => "ggml-large-v2.bin",
            WhisperModel::LargeV3 => "ggml-large-v3.bin",
        };
        
        path.push(model_filename);
        
        if !path.exists() {
            anyhow::bail!(
                "Whisper model not found at {:?}. Please download the model from https://huggingface.co/ggerganov/whisper.cpp/tree/main",
                path
            );
        }
        
        Ok(path)
    }
}

impl SttProvider for LocalWhisperStt {
    fn transcribe(&self, audio_data: Vec<u8>) -> Result<String> {
        // whisper-rs expects 16-bit PCM mono audio at 16kHz
        // The audio_data should already be in WAV format from our recording module
        
        // Parse WAV to get raw PCM data
        let mut reader = hound::WavReader::new(std::io::Cursor::new(audio_data))
            .context("Failed to parse WAV data")?;
        
        let spec = reader.spec();
        if spec.channels != 1 {
            anyhow::bail!("Audio must be mono, got {} channels", spec.channels);
        }
        if spec.sample_rate != 16000 {
            anyhow::bail!("Audio must be 16kHz, got {}Hz", spec.sample_rate);
        }
        
        // Convert to f32 samples as expected by whisper-rs
        let samples: Vec<f32> = reader
            .samples::<i16>()
            .map(|s| s.map(|sample| sample as f32 / i16::MAX as f32))
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to read audio samples")?;
        
        // Create parameters for this transcription
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        
        // Configure parameters for better accuracy
        params.set_language(Some("en"));
        params.set_translate(false);
        params.set_no_context(true);
        params.set_single_segment(false);
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);

        // Run inference
        let mut state = self.context.create_state()
            .context("Failed to create Whisper state")?;
        
        state.full(params, &samples)
            .context("Whisper inference failed")?;
        
        // Get the transcribed text
        let segment_count = state.full_n_segments()
            .context("Failed to get segment count")?;
        
        let mut transcript = String::new();
        for i in 0..segment_count {
            let text = state.full_get_segment_text(i)
                .context("Failed to get segment text")?;
            transcript.push_str(&text);
            transcript.push(' ');
        }
        
        Ok(transcript.trim().to_string())
    }
}