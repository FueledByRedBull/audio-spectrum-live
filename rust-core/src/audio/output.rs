//! Audio output playback using cpal
//! 
//! Real-time audio playback to speakers or line-out

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Stream, StreamConfig};
use super::buffer::AudioConsumer;
use std::sync::{Arc, Mutex};
use super::input::{AudioError, AudioDeviceInfo};

/// Audio output stream
pub struct AudioOutput {
    stream: Stream,
    device_info: AudioDeviceInfo,
}

impl AudioOutput {
    /// Create audio output from default device
    /// 
    /// # Arguments
    /// * `consumer` - Ring buffer consumer for audio playback
    pub fn from_default_device(consumer: AudioConsumer) -> Result<Self, AudioError> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or(AudioError::NoDevice)?;
        
        Self::from_device(device, consumer)
    }
    
    /// Create audio output from specific device
    pub fn from_device(device: Device, consumer: AudioConsumer) -> Result<Self, AudioError> {
        let name = device
            .name()
            .map_err(|e| AudioError::DeviceName(e.to_string()))?;
        
        let config = device
            .default_output_config()
            .map_err(|e| AudioError::DefaultConfig(e.to_string()))?;
        
        let sample_rate = config.sample_rate().0;
        
        // Require 48 kHz - refuse to start otherwise
        if sample_rate != 48000 {
            return Err(AudioError::UnsupportedSampleRate(sample_rate));
        }
        
        let channels = config.channels();
        
        let device_info = AudioDeviceInfo {
            name: name.clone(),
            sample_rate,
            channels,
        };
        
        let stream_config: StreamConfig = config.into();
        
        // Wrap consumer in Arc<Mutex> for thread-safe access
        let consumer = Arc::new(Mutex::new(consumer));
        
        // Build audio output stream
        let consumer_clone = Arc::clone(&consumer);
        let stream = device
            .build_output_stream(
                &stream_config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    // Read samples from ring buffer and convert to f32
                    let mut samples = vec![0.0; data.len()];
                    
                    if let Ok(mut cons) = consumer_clone.lock() {
                        let read = cons.read(&mut samples);
                        
                        // Convert f64 to f32 and write to output
                        for (i, &sample) in samples[..read].iter().enumerate() {
                            data[i] = sample as f32;
                        }
                        
                        // Zero remaining samples if not enough data
                        for sample in data[read..].iter_mut() {
                            *sample = 0.0;
                        }
                    }
                },
                move |err| {
                    eprintln!("Audio output error: {}", err);
                },
                None,
            )
            .map_err(|e| AudioError::BuildStream(e.to_string()))?;
        
        Ok(Self {
            stream,
            device_info,
        })
    }
    
    /// Start playing audio
    pub fn start(&self) -> Result<(), AudioError> {
        self.stream
            .play()
            .map_err(|e| AudioError::PlayStream(e.to_string()))
    }
    
    /// Pause audio playback
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

/// List available audio output devices
pub fn list_output_devices() -> Result<Vec<AudioDeviceInfo>, AudioError> {
    let host = cpal::default_host();
    let mut devices = Vec::new();
    
    let device_iter = host
        .output_devices()
        .map_err(|e| AudioError::DeviceName(e.to_string()))?;
    
    for device in device_iter {
        if let Ok(name) = device.name() {
            if let Ok(config) = device.default_output_config() {
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
