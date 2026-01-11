//! Audio input capture using cpal
//! 
//! Real-time audio capture from microphone or line-in

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Stream, StreamConfig};
use super::buffer::AudioProducer;
use std::sync::{Arc, Mutex};
use thiserror::Error;

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
    
    #[error("Device does not support 48000 Hz (found: {0} Hz). Please change device sample rate to 48000 Hz in system settings.")]
    UnsupportedSampleRate(u32),
}

/// Audio input device information
#[derive(Debug, Clone)]
pub struct AudioDeviceInfo {
    pub name: String,
    pub sample_rate: u32,
    pub channels: u16,
}

/// Audio input stream
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
        
        // Wrap producer in Arc<Mutex> for thread-safe access
        let producer = Arc::new(Mutex::new(producer));
        
        // Build audio input stream
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
        
        Ok(Self {
            stream,
            device_info,
        })
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
