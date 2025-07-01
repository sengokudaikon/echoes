#![allow(dead_code)]

use crate::error::{AudioError, Result};
use cpal::SampleFormat;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::io::Cursor;
use std::sync::{Arc, Mutex};
use tracing::{debug, error};

pub struct AudioRecorder {
    samples: Arc<Mutex<Vec<f32>>>,
    stream: Option<cpal::Stream>,
}

impl AudioRecorder {
    pub fn new() -> Self {
        Self {
            samples: Arc::new(Mutex::new(Vec::new())),
            stream: None,
        }
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
            sample_rate: 16000, // Standard for speech recognition
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
}
