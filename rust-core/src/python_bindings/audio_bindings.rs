//! Python bindings for audio I/O

use pyo3::prelude::*;
use numpy::PyArray1;
use crate::audio::{AudioInput, AudioRingBuffer};
use crate::audio::input::list_input_devices;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

/// Audio device information exposed to Python
#[pyclass(name = "AudioDeviceInfo")]
#[derive(Clone)]
pub struct PyAudioDeviceInfo {
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub sample_rate: u32,
    #[pyo3(get)]
    pub channels: u16,
}

/// Audio engine for real-time processing
#[pyclass(name = "AudioEngine", unsendable)]
pub struct PyAudioEngine {
    input: Option<AudioInput>,
    buffer_consumer: Arc<Mutex<VecDeque<f64>>>,
    buffer_capacity: usize,
}

#[pymethods]
impl PyAudioEngine {
    /// Create a new audio engine
    /// 
    /// Args:
    ///     buffer_capacity: Ring buffer capacity in samples (default: 48000)
    #[new]
    #[pyo3(signature = (buffer_capacity=48000))]
    fn new(buffer_capacity: usize) -> Self {
        Self {
            input: None,
            buffer_consumer: Arc::new(Mutex::new(VecDeque::new())),
            buffer_capacity,
        }
    }
    
    /// List available input devices
    /// 
    /// Returns:
    ///     List of audio device information
    #[staticmethod]
    fn list_input_devices() -> PyResult<Vec<PyAudioDeviceInfo>> {
        match list_input_devices() {
            Ok(devices) => {
                let device_list: Vec<PyAudioDeviceInfo> = devices
                    .into_iter()
                    .map(|d| PyAudioDeviceInfo {
                        name: d.name,
                        sample_rate: d.sample_rate,
                        channels: d.channels,
                    })
                    .collect();
                Ok(device_list)
            }
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                format!("Failed to list devices: {}", e),
            )),
        }
    }
    
    /// Start audio capture from default device
    fn start_capture(&mut self) -> PyResult<PyAudioDeviceInfo> {
        // Create ring buffer
        let rb = AudioRingBuffer::new(self.buffer_capacity);
        let (producer, mut consumer) = rb.split();
        
        // Start audio input
        let input = AudioInput::from_default_device(producer)
            .map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                    format!("Failed to create audio input: {}", e),
                )
            })?;
        
        let device_info = PyAudioDeviceInfo {
            name: input.device_info().name.clone(),
            sample_rate: input.device_info().sample_rate,
            channels: input.device_info().channels,
        };
        
        input.start().map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                format!("Failed to start audio: {}", e),
            )
        })?;
        
        // Move data from ring buffer to our queue (simplified for Python access)
        let buffer_clone = Arc::clone(&self.buffer_consumer);
        std::thread::spawn(move || {
            let mut temp_buffer = vec![0.0; 1024];
            loop {
                let n = consumer.read(&mut temp_buffer);
                if n > 0 {
                    if let Ok(mut queue) = buffer_clone.lock() {
                        queue.extend(&temp_buffer[..n]);
                        // Keep queue size reasonable
                        while queue.len() > 96000 {
                            queue.pop_front();
                        }
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        });
        
        self.input = Some(input);
        Ok(device_info)
    }
    
    /// Stop audio capture
    fn stop_capture(&mut self) -> PyResult<()> {
        if let Some(input) = &self.input {
            input.pause().map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                    format!("Failed to stop audio: {}", e),
                )
            })?;
        }
        self.input = None;
        Ok(())
    }
    
    /// Read available audio samples
    /// 
    /// Args:
    ///     max_samples: Maximum number of samples to read
    /// 
    /// Returns:
    ///     Audio samples as numpy array
    fn read_audio<'py>(
        &mut self,
        py: Python<'py>,
        max_samples: usize,
    ) -> PyResult<&'py PyArray1<f64>> {
        let samples = if let Ok(mut queue) = self.buffer_consumer.lock() {
            let n = max_samples.min(queue.len());
            let mut samples = Vec::with_capacity(n);
            for _ in 0..n {
                if let Some(sample) = queue.pop_front() {
                    samples.push(sample);
                }
            }
            samples
        } else {
            Vec::new()
        };
        
        Ok(PyArray1::from_vec(py, samples))
    }
    
    /// Get number of available samples in buffer
    fn available_samples(&self) -> usize {
        if let Ok(queue) = self.buffer_consumer.lock() {
            queue.len()
        } else {
            0
        }
    }
    
    /// Clear audio buffer
    fn clear_buffer(&mut self) {
        if let Ok(mut queue) = self.buffer_consumer.lock() {
            queue.clear();
        }
    }
}
