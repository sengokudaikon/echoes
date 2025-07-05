#![allow(dead_code)]

mod vad;

use crate::error::{AudioError, Result};
use cpal::SampleFormat;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::io::Cursor;
use std::sync::{Arc, Mutex};
use tracing::{debug, error};
use vad::VadProcessor;

pub struct AudioRecorder {
    samples: Arc<Mutex<Vec<f32>>>,
    stream: Option<cpal::Stream>,
    use_vad: bool,
    sample_rate: u32,
}

impl AudioRecorder {
    pub fn new() -> Self {
        Self {
            samples: Arc::new(Mutex::new(Vec::new())),
            stream: None,
            use_vad: true, // Enable VAD by default
            sample_rate: 16000, // Default, will be updated when recording starts
        }
    }

    /// Create a new recorder with VAD disabled
    pub fn new_without_vad() -> Self {
        Self {
            samples: Arc::new(Mutex::new(Vec::new())),
            stream: None,
            use_vad: false,
            sample_rate: 16000,
        }
    }

    /// Enable or disable VAD processing
    pub fn set_vad(&mut self, use_vad: bool) {
        self.use_vad = use_vad;
    }

    pub fn start_recording(&mut self) -> Result<()> {
        // Clear previous samples
        self.samples
            .lock()
            .map_err(|_| AudioError::MutexPoisoned)?
            .clear();

        // Get default input device
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or(AudioError::NoInputDevice)?;

        let device_name = device
            .name()
            .map_err(|e| AudioError::StreamCreationFailed(e.to_string()))?;
        debug!("Using input device: {}", device_name);

        // Get default config
        let config = device
            .default_input_config()
            .map_err(|e| AudioError::StreamCreationFailed(e.to_string()))?;
        debug!("Default input config: {:?}", config);
        
        // Store the actual sample rate
        self.sample_rate = config.sample_rate().0;

        let samples = Arc::clone(&self.samples);

        // Build the stream
        let stream = match config.sample_format() {
            SampleFormat::F32 => {
                self.build_input_stream::<f32>(&device, &config.into(), samples)?
            }
            SampleFormat::I16 => {
                self.build_input_stream::<i16>(&device, &config.into(), samples)?
            }
            SampleFormat::U16 => {
                self.build_input_stream::<u16>(&device, &config.into(), samples)?
            }
            sample_format => {
                return Err(AudioError::UnsupportedFormat(format!("{sample_format:?}")).into());
            }
        };

        stream
            .play()
            .map_err(|e| AudioError::StreamCreationFailed(e.to_string()))?;
        self.stream = Some(stream);

        Ok(())
    }

    pub fn stop_recording(&mut self) -> Result<Vec<u8>> {
        // Stop and drop the stream
        self.stream = None;

        // Get the samples
        let samples = self
            .samples
            .lock()
            .map_err(|_| AudioError::MutexPoisoned)?
            .clone();

        // Convert to WAV
        self.samples_to_wav(samples)
    }

    /// Stop recording and return both raw audio and VAD-processed segments
    pub fn stop_recording_with_vad(&mut self) -> Result<(Vec<u8>, Vec<Vec<u8>>)> {
        // Stop and drop the stream
        self.stream = None;

        // Get the samples
        let samples = self
            .samples
            .lock()
            .map_err(|_| AudioError::MutexPoisoned)?
            .clone();

        // First create the raw WAV
        let raw_wav = self.samples_to_wav(samples.clone())?;

        // Resample to 16kHz if needed for VAD
        let samples_16k = if self.sample_rate != 16000 {
            debug!("Resampling from {}Hz to 16000Hz", self.sample_rate);
            let original_len = samples.len();
            let resampled = self.resample_to_16khz(samples)?;
            debug!("Resampled from {} samples to {} samples", original_len, resampled.len());
            resampled
        } else {
            samples
        };

        // Process with VAD
        let mut vad = VadProcessor::new()?;
        let mut speech_segments = vad.process_audio(&samples_16k)?;
        
        // Check if there's a final segment
        if let Some(final_segment) = vad.finish() {
            speech_segments.push(final_segment);
        }

        // Convert each segment to WAV (at 16kHz)
        let mut wav_segments = Vec::new();
        let original_rate = self.sample_rate;
        self.sample_rate = 16000; // Temporarily set to 16kHz for WAV output
        
        for segment in speech_segments {
            let wav_data = self.samples_to_wav(segment)?;
            wav_segments.push(wav_data);
        }
        
        self.sample_rate = original_rate; // Restore original rate

        Ok((raw_wav, wav_segments))
    }

    /// Resample audio from current sample rate to 16kHz
    fn resample_to_16khz(&self, samples: Vec<f32>) -> Result<Vec<f32>> {
        use rubato::{Resampler, SincFixedIn, SincInterpolationType, SincInterpolationParameters, WindowFunction};
        
        let params = SincInterpolationParameters {
            sinc_len: 256,
            f_cutoff: 0.95,
            interpolation: SincInterpolationType::Linear,
            oversampling_factor: 256,
            window: WindowFunction::BlackmanHarris2,
        };
        
        // Create resampler with proper chunk size
        let chunk_size = 1024;
        let mut resampler = SincFixedIn::<f32>::new(
            16000_f64 / self.sample_rate as f64,
            2.0,
            params,
            chunk_size,
            1,
        ).map_err(|e| AudioError::StreamCreationFailed(format!("Failed to create resampler: {}", e)))?;
        
        // Process all samples in chunks
        let mut output = Vec::new();
        let mut position = 0;
        
        while position < samples.len() {
            let end = (position + chunk_size).min(samples.len());
            let chunk = &samples[position..end];
            
            if chunk.len() == chunk_size {
                // Process full chunk
                let waves_in = vec![chunk.to_vec()];
                let waves_out = resampler.process(&waves_in, None)
                    .map_err(|e| AudioError::StreamCreationFailed(format!("Resampling failed: {}", e)))?;
                if let Some(out_chunk) = waves_out.get(0) {
                    output.extend_from_slice(out_chunk);
                }
            } else if !chunk.is_empty() {
                // Process last partial chunk with padding
                let mut padded = chunk.to_vec();
                padded.resize(chunk_size, 0.0);
                let waves_in = vec![padded];
                let waves_out = resampler.process(&waves_in, None)
                    .map_err(|e| AudioError::StreamCreationFailed(format!("Resampling failed: {}", e)))?;
                if let Some(out_chunk) = waves_out.get(0) {
                    // Only take the proportional amount of output samples
                    let output_len = (chunk.len() as f64 * 16000.0 / self.sample_rate as f64) as usize;
                    output.extend_from_slice(&out_chunk[..output_len.min(out_chunk.len())]);
                }
            }
            
            position = end;
        }
        
        Ok(output)
    }

    fn build_input_stream<T>(
        &self,
        device: &cpal::Device,
        config: &cpal::StreamConfig,
        samples: Arc<Mutex<Vec<f32>>>,
    ) -> Result<cpal::Stream>
    where
        T: cpal::SizedSample + Send + 'static,
        f32: cpal::FromSample<T>,
    {
        let err_fn = |err| error!("An error occurred on the audio stream: {}", err);

        let stream = device
            .build_input_stream(
                config,
                move |data: &[T], _: &cpal::InputCallbackInfo| match samples.lock() {
                    Ok(mut samples) => {
                        for sample in data.iter() {
                            samples.push(sample.to_sample::<f32>());
                        }
                    }
                    Err(e) => {
                        error!("Failed to lock samples buffer: {}", e);
                    }
                },
                err_fn,
                None,
            )
            .map_err(|e| AudioError::StreamCreationFailed(e.to_string()))?;

        Ok(stream)
    }

    fn samples_to_wav(&self, samples: Vec<f32>) -> Result<Vec<u8>> {
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: self.sample_rate, // Use actual sample rate
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let mut cursor = Cursor::new(Vec::new());
        {
            let mut writer = hound::WavWriter::new(&mut cursor, spec)
                .map_err(|e| AudioError::WavEncodingFailed(e.to_string()))?;

            for sample in samples {
                let amplitude = (sample * i16::MAX as f32) as i16;
                writer
                    .write_sample(amplitude)
                    .map_err(|e| AudioError::WavEncodingFailed(e.to_string()))?;
            }

            writer
                .finalize()
                .map_err(|e| AudioError::WavEncodingFailed(e.to_string()))?;
        }

        Ok(cursor.into_inner())
    }

    /// Save samples directly to a WAV file
    pub fn save_samples_to_file(&self, samples: Vec<f32>, path: &std::path::Path) -> Result<()> {
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: self.sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let mut writer = hound::WavWriter::create(path, spec)
            .map_err(|e| AudioError::WavEncodingFailed(e.to_string()))?;

        for sample in samples {
            let amplitude = (sample * i16::MAX as f32) as i16;
            writer
                .write_sample(amplitude)
                .map_err(|e| AudioError::WavEncodingFailed(e.to_string()))?;
        }

        writer
            .finalize()
            .map_err(|e| AudioError::WavEncodingFailed(e.to_string()))?;

        Ok(())
    }
}
