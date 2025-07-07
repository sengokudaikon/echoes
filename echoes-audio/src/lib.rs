pub mod error;
pub mod vad;

use std::io::Cursor;

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    SampleFormat,
};
pub use error::{AudioError, Result};
use rtrb::{Consumer, Producer, RingBuffer};
use tracing::{debug, error};
use vad::VadProcessor;

pub struct AudioRecorder {
    ring_buffer_producer: Option<Producer<f32>>,
    ring_buffer_consumer: Option<Consumer<f32>>,
    stream: Option<cpal::Stream>,
    use_vad: bool,
    sample_rate: u32,
    /// Maximum recording duration in seconds (default: 300 seconds = 5 minutes)
    max_duration_seconds: u32,
    /// Ring buffer capacity in samples
    ring_buffer_capacity: usize,
}

impl Default for AudioRecorder {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioRecorder {
    #[must_use]
    pub fn new() -> Self {
        let ring_buffer_capacity = 300 * 16000;
        let (producer, consumer) = RingBuffer::new(ring_buffer_capacity);

        Self {
            ring_buffer_producer: Some(producer),
            ring_buffer_consumer: Some(consumer),
            stream: None,
            use_vad: true,
            sample_rate: 16000,
            max_duration_seconds: 300,
            ring_buffer_capacity,
        }
    }

    /// Create a new recorder with VAD disabled
    #[must_use]
    pub fn new_without_vad() -> Self {
        let ring_buffer_capacity = 300 * 16000;
        let (producer, consumer) = RingBuffer::new(ring_buffer_capacity);

        Self {
            ring_buffer_producer: Some(producer),
            ring_buffer_consumer: Some(consumer),
            stream: None,
            use_vad: false,
            sample_rate: 16000,
            max_duration_seconds: 300,
            ring_buffer_capacity,
        }
    }

    /// Enable or disable VAD processing
    pub const fn set_vad(&mut self, use_vad: bool) {
        self.use_vad = use_vad;
    }

    /// Set maximum recording duration in seconds
    pub fn set_max_duration(&mut self, seconds: u32) {
        self.max_duration_seconds = seconds;
        let ring_buffer_capacity = (seconds as usize) * (self.sample_rate as usize);
        let (producer, consumer) = RingBuffer::new(ring_buffer_capacity);
        self.ring_buffer_producer = Some(producer);
        self.ring_buffer_consumer = Some(consumer);
        self.ring_buffer_capacity = ring_buffer_capacity;
    }

    /// Clear the audio buffer by consuming all available samples
    ///
    /// # Errors
    ///
    /// Returns an error if the ring buffer operations fail
    pub fn clear_buffer(&mut self) -> Result<()> {
        if let Some(ref mut consumer) = self.ring_buffer_consumer {
            while let Ok(chunk) = consumer.read_chunk(consumer.slots()) {
                if chunk.is_empty() {
                    break;
                }
                chunk.commit_all();
            }
        }
        Ok(())
    }

    /// Stop the audio stream and collect all samples from the ring buffer
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Stream pause fails
    /// - Ring buffer consumer is not available
    fn stop_and_collect_samples(&mut self) -> Result<Vec<f32>> {
        // Explicitly pause the stream before dropping it
        if let Some(stream) = &self.stream {
            stream
                .pause()
                .map_err(|e| AudioError::StreamCreationFailed(format!("Failed to pause stream: {e}")))?;
        }

        // Stop and drop the stream
        self.stream = None;

        // Collect all samples from the ring buffer
        let mut samples = Vec::new();
        if let Some(ref mut consumer) = self.ring_buffer_consumer {
            while let Ok(chunk) = consumer.read_chunk(consumer.slots()) {
                if chunk.is_empty() {
                    break;
                }
                // Copy data from the chunk to our samples Vec
                let (first_slice, second_slice) = chunk.as_slices();
                samples.extend_from_slice(first_slice);
                samples.extend_from_slice(second_slice);
                chunk.commit_all();
            }
        }

        Ok(samples)
    }

    /// Start audio recording from the default input device
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No input device is available
    /// - Audio stream creation fails
    /// - Ring buffer is not available
    pub fn start_recording(&mut self) -> Result<()> {
        // Clear any existing samples
        self.clear_buffer()?;

        let host = cpal::default_host();
        let device = host.default_input_device().ok_or(AudioError::NoInputDevice)?;

        let device_name = device
            .name()
            .map_err(|e| AudioError::StreamCreationFailed(e.to_string()))?;
        debug!("Using input device: {}", device_name);

        let config = device
            .default_input_config()
            .map_err(|e| AudioError::StreamCreationFailed(e.to_string()))?;
        debug!("Default input config: {:?}", config);

        self.sample_rate = config.sample_rate().0;

        // Take the producer from the option (we'll need to recreate it if this fails)
        let producer = self
            .ring_buffer_producer
            .take()
            .ok_or_else(|| AudioError::Other("Ring buffer producer not available".into()))?;

        debug!("Ring buffer capacity: {} samples", self.ring_buffer_capacity);

        let stream = match config.sample_format() {
            SampleFormat::F32 => Self::build_input_stream::<f32>(&device, &config.into(), producer)?,
            SampleFormat::I16 => Self::build_input_stream::<i16>(&device, &config.into(), producer)?,
            SampleFormat::U16 => Self::build_input_stream::<u16>(&device, &config.into(), producer)?,
            sample_format => {
                return Err(AudioError::UnsupportedFormat(format!("{sample_format:?}")));
            }
        };

        stream
            .play()
            .map_err(|e| AudioError::StreamCreationFailed(e.to_string()))?;
        self.stream = Some(stream);

        Ok(())
    }

    /// Stop audio recording and return results based on VAD setting
    ///
    /// Returns a tuple containing:
    /// - Raw WAV data of the entire recording
    /// - Vector of WAV data for each detected speech segment (empty if VAD is
    ///   disabled)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Ring buffer consumer is not available
    /// - WAV encoding fails
    /// - VAD processing fails (if VAD is enabled)
    /// - Audio resampling fails (if VAD is enabled)
    /// - Stream stop fails
    pub fn stop_recording(&mut self) -> Result<(Vec<u8>, Vec<Vec<u8>>)> {
        let samples = self.stop_and_collect_samples()?;

        // Always create the raw WAV
        let raw_wav = self.samples_to_wav(&samples)?;

        if self.use_vad {
            let vad_segments = self.process_samples_with_vad(samples)?;
            Ok((raw_wav, vad_segments))
        } else {
            Ok((raw_wav, Vec::new())) // Empty segments when VAD is disabled
        }
    }

    /// Process samples with VAD and return speech segments as WAV data
    ///
    /// # Errors
    ///
    /// Returns an error if VAD processing or WAV encoding fails
    fn process_samples_with_vad(&mut self, samples: Vec<f32>) -> Result<Vec<Vec<u8>>> {
        // Resample to 16kHz if needed for VAD
        let samples_16k = if self.sample_rate == 16000 {
            samples
        } else {
            debug!("Resampling from {}Hz to 16000Hz", self.sample_rate);
            let original_len = samples.len();
            let resampled = self.resample_to_16khz(&samples)?;
            debug!("Resampled from {} samples to {} samples", original_len, resampled.len());
            resampled
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
            let wav_data = self.samples_to_wav(&segment)?;
            wav_segments.push(wav_data);
        }

        self.sample_rate = original_rate; // Restore original rate

        Ok(wav_segments)
    }

    /// Resample audio from current sample rate to 16kHz
    fn resample_to_16khz(&self, samples: &[f32]) -> Result<Vec<f32>> {
        use rubato::{Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction};

        let params = SincInterpolationParameters {
            sinc_len: 256,
            f_cutoff: 0.95,
            interpolation: SincInterpolationType::Linear,
            oversampling_factor: 256,
            window: WindowFunction::BlackmanHarris2,
        };

        // Create resampler with proper chunk size
        let chunk_size = 1024;
        let mut resampler =
            SincFixedIn::<f32>::new(16000_f64 / f64::from(self.sample_rate), 2.0, params, chunk_size, 1)
                .map_err(|e| AudioError::StreamCreationFailed(format!("Failed to create resampler: {e}")))?;

        // Process all samples in chunks
        let mut output = Vec::new();
        let mut position = 0;

        while position < samples.len() {
            let end = (position + chunk_size).min(samples.len());
            let chunk = &samples[position..end];

            if chunk.len() == chunk_size {
                // Process full chunk
                let waves_in = vec![chunk.to_vec()];
                let waves_out = resampler
                    .process(&waves_in, None)
                    .map_err(|e| AudioError::StreamCreationFailed(format!("Resampling failed: {e}")))?;
                if let Some(out_chunk) = waves_out.first() {
                    output.extend_from_slice(out_chunk);
                }
            } else if !chunk.is_empty() {
                // Process last partial chunk with padding
                let mut padded = chunk.to_vec();
                padded.resize(chunk_size, 0.0);
                let waves_in = vec![padded];
                let waves_out = resampler
                    .process(&waves_in, None)
                    .map_err(|e| AudioError::StreamCreationFailed(format!("Resampling failed: {e}")))?;
                if let Some(out_chunk) = waves_out.first() {
                    // Only take the proportional amount of output samples
                    // Safe: chunk.len() is audio chunk size (typically small), calculation result
                    // is bounded by resampling ratio
                    #[allow(
                        clippy::cast_precision_loss,
                        clippy::cast_possible_truncation,
                        clippy::cast_sign_loss
                    )]
                    let output_len = (chunk.len() as f64 * 16000.0 / f64::from(self.sample_rate)) as usize;
                    output.extend_from_slice(&out_chunk[..output_len.min(out_chunk.len())]);
                }
            }

            position = end;
        }

        Ok(output)
    }

    fn build_input_stream<T>(
        device: &cpal::Device, config: &cpal::StreamConfig, mut producer: Producer<f32>,
    ) -> Result<cpal::Stream>
    where
        T: cpal::SizedSample + Send + 'static,
        f32: cpal::FromSample<T>,
    {
        let err_fn = |err| error!("An error occurred on the audio stream: {}", err);

        let stream = device
            .build_input_stream(
                config,
                move |data: &[T], _: &cpal::InputCallbackInfo| {
                    let samples: Vec<f32> = data.iter().map(|sample| sample.to_sample::<f32>()).collect();

                    if let Ok(mut chunk) = producer.write_chunk_uninit(samples.len()) {
                        let mut write_pos = 0;
                        let (first_slice, second_slice) = chunk.as_mut_slices();

                        let first_len = first_slice.len().min(samples.len() - write_pos);
                        for i in 0..first_len {
                            first_slice[i].write(samples[write_pos + i]);
                        }
                        write_pos += first_len;

                        if write_pos < samples.len() {
                            let second_len = second_slice.len().min(samples.len() - write_pos);
                            for i in 0..second_len {
                                second_slice[i].write(samples[write_pos + i]);
                            }
                        }

                        // Safety: We've initialized all elements
                        unsafe {
                            chunk.commit_all();
                        }
                    } else {
                        debug!("Ring buffer full, dropping audio samples");
                    }
                },
                err_fn,
                None,
            )
            .map_err(|e| AudioError::StreamCreationFailed(e.to_string()))?;

        Ok(stream)
    }

    fn samples_to_wav(&self, samples: &[f32]) -> Result<Vec<u8>> {
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: self.sample_rate, // Use actual sample rate
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let mut cursor = Cursor::new(Vec::new());
        {
            let mut writer =
                hound::WavWriter::new(&mut cursor, spec).map_err(|e| AudioError::WavEncodingFailed(e.to_string()))?;

            for sample in samples {
                // Proper conversion from f32 audio sample [-1.0, 1.0] to int16 with clamping
                #[allow(clippy::cast_possible_truncation)]
                let amplitude = (sample.clamp(-1.0, 1.0) * 32767.0).round().clamp(-32768.0, 32767.0) as i16;
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
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - File creation fails
    /// - WAV encoding fails
    /// - File writing fails
    pub fn save_samples_to_file(&self, samples: &[f32], path: &std::path::Path) -> Result<()> {
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: self.sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let mut writer =
            hound::WavWriter::create(path, spec).map_err(|e| AudioError::WavEncodingFailed(e.to_string()))?;

        for sample in samples {
            // Proper conversion from f32 audio sample [-1.0, 1.0] to int16 with clamping
            #[allow(clippy::cast_possible_truncation)]
            let amplitude = (sample.clamp(-1.0, 1.0) * 32767.0).round().clamp(-32768.0, 32767.0) as i16;
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
