//! Audio input capture using cpal with automatic resampling
//!
//! Real-time audio capture from microphone or line-in
//! Supports any device sample rate with automatic resampling to 48 kHz

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Stream, StreamConfig};
use super::buffer::AudioProducer;
use rubato::{FftFixedIn, Resampler};
use std::sync::{Arc, Mutex};
use thiserror::Error;

/// Target sample rate for internal processing
pub const TARGET_SAMPLE_RATE: u32 = 48000;

#[derive(Error, Debug)]
pub enum AudioError {
    #[error("No audio input device found")]
    NoDevice,

    #[error("Failed to get device name: {0}")]
    DeviceName(String),

    #[error("Failed to get default config: {0}")]
    DefaultConfig(String),

    #[error("Failed to build stream: {0}")]
    BuildStream(String),

    #[error("Failed to play stream: {0}")]
    PlayStream(String),

    #[error("Failed to create resampler: {0}")]
    ResamplerError(String),

    #[error("Output device does not support 48000 Hz (found: {0} Hz). Audio monitoring requires 48 kHz output device.")]
    UnsupportedSampleRate(u32),
}

/// Audio input device information
#[derive(Debug, Clone)]
pub struct AudioDeviceInfo {
    pub name: String,
    pub sample_rate: u32,
    pub channels: u16,
}

/// Audio input stream with optional resampling
pub struct AudioInput {
    stream: Stream,
    device_info: AudioDeviceInfo,
}

impl AudioInput {
    /// Create audio input from default device
    ///
    /// # Arguments
    /// * `producer` - Ring buffer producer for captured audio
    ///
    /// # Returns
    /// Audio input stream and device info
    pub fn from_default_device(producer: AudioProducer) -> Result<Self, AudioError> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or(AudioError::NoDevice)?;

        Self::from_device(device, producer)
    }

    /// Create audio input from specific device
    pub fn from_device(device: Device, producer: AudioProducer) -> Result<Self, AudioError> {
        let name = device
            .name()
            .map_err(|e| AudioError::DeviceName(e.to_string()))?;

        let config = device
            .default_input_config()
            .map_err(|e| AudioError::DefaultConfig(e.to_string()))?;

        let device_sample_rate = config.sample_rate().0;
        let channels = config.channels();

        // Report actual sample rate (always report target rate to caller)
        let device_info = AudioDeviceInfo {
            name: name.clone(),
            sample_rate: TARGET_SAMPLE_RATE, // Always report target rate
            channels,
        };

        let stream_config: StreamConfig = config.into();

        // Wrap producer in Arc<Mutex> for thread-safe access
        let producer = Arc::new(Mutex::new(producer));

        // Create resampler if device rate differs from target
        let needs_resampling = device_sample_rate != TARGET_SAMPLE_RATE;

        if needs_resampling {
            println!(
                "Device sample rate {} Hz differs from target {} Hz - enabling resampling",
                device_sample_rate, TARGET_SAMPLE_RATE
            );

            // Create FFT-based resampler (high quality, handles any ratio)
            // chunk_size is the input block size
            let chunk_size = 1024;
            let resampler = FftFixedIn::<f64>::new(
                device_sample_rate as usize,
                TARGET_SAMPLE_RATE as usize,
                chunk_size,
                2, // sub_chunks for interpolation quality
                1, // mono
            )
            .map_err(|e| AudioError::ResamplerError(e.to_string()))?;

            let resampler = Arc::new(Mutex::new(resampler));
            let input_buffer: Arc<Mutex<Vec<f64>>> = Arc::new(Mutex::new(Vec::with_capacity(chunk_size * 2)));

            // Build audio input stream with resampling
            let producer_clone = Arc::clone(&producer);
            let resampler_clone = Arc::clone(&resampler);
            let input_buffer_clone = Arc::clone(&input_buffer);

            let stream = device
                .build_input_stream(
                    &stream_config,
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        // Convert f32 to f64 and accumulate in input buffer
                        if let Ok(mut buffer) = input_buffer_clone.lock() {
                            buffer.extend(data.iter().map(|&s| s as f64));

                            // Process when we have enough samples
                            if let Ok(mut resampler) = resampler_clone.lock() {
                                let input_frames_needed = resampler.input_frames_next();

                                while buffer.len() >= input_frames_needed {
                                    // Extract input chunk
                                    let input_chunk: Vec<f64> = buffer.drain(..input_frames_needed).collect();

                                    // Resample (single channel)
                                    if let Ok(output) = resampler.process(&[input_chunk], None) {
                                        if !output.is_empty() && !output[0].is_empty() {
                                            if let Ok(mut prod) = producer_clone.lock() {
                                                prod.write(&output[0]);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    },
                    move |err| {
                        eprintln!("Audio input error: {}", err);
                    },
                    None,
                )
                .map_err(|e| AudioError::BuildStream(e.to_string()))?;

            Ok(Self { stream, device_info })
        } else {
            // No resampling needed - direct passthrough
            let producer_clone = Arc::clone(&producer);

            let stream = device
                .build_input_stream(
                    &stream_config,
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        // Convert f32 samples to f64 and write to ring buffer
                        let samples: Vec<f64> = data.iter().map(|&s| s as f64).collect();

                        if let Ok(mut prod) = producer_clone.lock() {
                            prod.write(&samples);
                        }
                    },
                    move |err| {
                        eprintln!("Audio input error: {}", err);
                    },
                    None,
                )
                .map_err(|e| AudioError::BuildStream(e.to_string()))?;

            Ok(Self { stream, device_info })
        }
    }

    /// Start capturing audio
    pub fn start(&self) -> Result<(), AudioError> {
        self.stream
            .play()
            .map_err(|e| AudioError::PlayStream(e.to_string()))
    }

    /// Pause audio capture
    pub fn pause(&self) -> Result<(), AudioError> {
        self.stream
            .pause()
            .map_err(|e| AudioError::PlayStream(e.to_string()))
    }

    /// Get device information
    pub fn device_info(&self) -> &AudioDeviceInfo {
        &self.device_info
    }
}

/// List available audio input devices
pub fn list_input_devices() -> Result<Vec<AudioDeviceInfo>, AudioError> {
    let host = cpal::default_host();
    let mut devices = Vec::new();

    let device_iter = host
        .input_devices()
        .map_err(|e| AudioError::DeviceName(e.to_string()))?;

    for device in device_iter {
        if let Ok(name) = device.name() {
            if let Ok(config) = device.default_input_config() {
                devices.push(AudioDeviceInfo {
                    name,
                    sample_rate: config.sample_rate().0,
                    channels: config.channels(),
                });
            }
        }
    }

    Ok(devices)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_devices() {
        // Just ensure it doesn't crash
        let _ = list_input_devices();
    }
}
