use tracing::debug;
use voice_activity_detector::VoiceActivityDetector;

use crate::error::{AudioError, Result};

/// Voice Activity Detector wrapper for audio processing
pub struct VadProcessor {
    detector: VoiceActivityDetector,
    /// Number of consecutive frames to wait before switching states
    hangover_frames: usize,
    /// Counter for hangover mechanism
    silence_counter: usize,
    /// Current VAD state
    is_speaking: bool,
    /// Minimum speech duration in samples (to avoid very short segments)
    min_speech_samples: usize,
    /// Speech segment buffer
    current_segment: Vec<f32>,
}

impl VadProcessor {
    pub fn new() -> Result<Self> {
        // Initialize with default settings optimized for speech
        let detector = VoiceActivityDetector::builder()
            .sample_rate(16000) // Match our recording sample rate
            .chunk_size(512usize) // Process in 32ms chunks (512 samples at 16kHz)
            .build()
            .map_err(|e| AudioError::StreamCreationFailed(format!("Failed to build VAD detector: {e}")))?;

        Ok(Self {
            detector,
            hangover_frames: 10, // ~320ms of silence before cutting
            silence_counter: 0,
            is_speaking: false,
            min_speech_samples: 4800, // 300ms minimum speech duration
            current_segment: Vec::new(),
        })
    }

    /// Process audio samples and extract speech segments
    pub fn process_audio(&mut self, samples: &[f32]) -> Result<Vec<Vec<f32>>> {
        let mut speech_segments = Vec::new();
        debug!("Processing {} samples with VAD", samples.len());

        // Calculate overall RMS to check audio level
        let rms = (samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32).sqrt();
        debug!("Audio RMS level: {:.6}", rms);

        // Process in chunks that match VAD requirements
        for (chunk_idx, chunk) in samples.chunks(512).enumerate() {
            // Pad the last chunk if needed
            let mut chunk_vec = chunk.to_vec();
            if chunk_vec.len() < 512 {
                chunk_vec.resize(512, 0.0);
            }

            // Run VAD on this chunk
            let probability = self.detector.predict(chunk_vec.clone());

            let is_speech = probability > 0.5;

            if chunk_idx % 10 == 0 {
                // Log every 10th chunk to avoid spam
                debug!(
                    "Chunk {}: probability = {:.3}, is_speech = {}",
                    chunk_idx, probability, is_speech
                );
            }

            // State machine for speech detection with hangover
            match (self.is_speaking, is_speech) {
                (false, true) => {
                    // Start of speech detected
                    self.is_speaking = true;
                    self.silence_counter = 0;
                    self.current_segment.extend_from_slice(chunk);
                }
                (true, true) => {
                    // Continuing speech
                    self.silence_counter = 0;
                    self.current_segment.extend_from_slice(chunk);
                }
                (true, false) => {
                    // Possible end of speech, but wait for hangover
                    self.silence_counter += 1;
                    self.current_segment.extend_from_slice(chunk);

                    if self.silence_counter >= self.hangover_frames {
                        // End of speech confirmed
                        self.is_speaking = false;

                        // Only keep segments longer than minimum duration
                        if self.current_segment.len() >= self.min_speech_samples {
                            // Trim trailing silence
                            let segment = Self::trim_silence_static(&self.current_segment);
                            if !segment.is_empty() {
                                speech_segments.push(segment);
                            }
                        }

                        self.current_segment.clear();
                        self.silence_counter = 0;
                    }
                }
                (false, false) => {
                    // Continuing silence
                    self.silence_counter = 0;
                }
            }
        }

        debug!(
            "VAD processing complete: found {} speech segments",
            speech_segments.len()
        );
        Ok(speech_segments)
    }

    /// Get any remaining speech segment (call when recording stops)
    #[must_use]
    pub fn finish(self) -> Option<Vec<f32>> {
        if self.is_speaking && self.current_segment.len() >= self.min_speech_samples {
            let segment = self.current_segment;
            Some(Self::trim_silence_static(&segment))
        } else {
            None
        }
    }

    /// Trim silence from the beginning and end of a segment (static version)
    fn trim_silence_static(segment: &[f32]) -> Vec<f32> {
        const SILENCE_THRESHOLD: f32 = 0.01;

        // Trim from beginning
        let start = segment.iter().position(|&s| s.abs() > SILENCE_THRESHOLD).unwrap_or(0);

        // Trim from end
        let end = segment
            .iter()
            .rposition(|&s| s.abs() > SILENCE_THRESHOLD)
            .map_or(segment.len(), |pos| pos + 1);

        if start < end {
            segment[start..end].to_vec()
        } else {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vad_initialization() -> Result<()> {
        let vad = VadProcessor::new()?;
        assert_eq!(vad.hangover_frames, 10);
        assert_eq!(vad.min_speech_samples, 4800);
        Ok(())
    }

    #[test]
    fn test_silence_detection() -> Result<()> {
        let mut vad = VadProcessor::new()?;
        let silence = vec![0.0f32; 16000]; // 1 second of silence

        let segments = vad.process_audio(&silence)?;
        assert!(segments.is_empty(), "Should not detect speech in silence");
        Ok(())
    }
}
