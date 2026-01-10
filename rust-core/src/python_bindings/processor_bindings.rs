//! Python bindings for unified audio processor

use pyo3::prelude::*;
use numpy::PyArray1;
use crate::audio::AudioProcessor;
use super::filter_bindings::PyWindowType;

/// Unified audio processor exposed to Python
/// 
/// Eliminates Python/Rust boundary overhead - all processing happens in Rust thread
#[pyclass(name = "AudioProcessor", unsendable)]
pub struct PyAudioProcessor {
    processor: AudioProcessor,
}

#[pymethods]
impl PyAudioProcessor {
    /// Create new audio processor
    #[new]
    fn new() -> Self {
        Self {
            processor: AudioProcessor::new(),
        }
    }
    
    /// Start audio capture and processing
    /// 
    /// Returns:
    ///     Device name as string
    fn start(&mut self) -> PyResult<String> {
        self.processor.start()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))
    }
    
    /// Stop audio processing
    fn stop(&mut self) {
        self.processor.stop();
    }
    
    /// Design and apply new filter
    /// 
    /// Args:
    ///     omega_c1: Lower cutoff frequency (normalized, units of π)
    ///     omega_c2: Upper cutoff frequency (normalized, units of π)
    ///     delta_omega: Transition width (radians)
    ///     window_type: Window type
    /// 
    /// Returns:
    ///     Tuple of (filter_length, group_delay)
    fn design_filter(
        &mut self,
        omega_c1: f64,
        omega_c2: f64,
        delta_omega: f64,
        window_type: PyWindowType,
    ) -> PyResult<(usize, f64)> {
        self.processor.design_filter(omega_c1, omega_c2, delta_omega, window_type.into())
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))
    }
    
    /// Set filter bypass state
    fn set_bypass(&self, bypass: bool) {
        self.processor.set_bypass(bypass);
    }
    
    /// Enable audio monitoring (WARNING: Use headphones!)
    /// 
    /// Outputs filtered audio to speakers/headphones.
    /// Use headphones to avoid feedback loop!
    fn enable_monitoring(&mut self) -> PyResult<()> {
        self.processor.enable_monitoring()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))
    }
    
    /// Disable audio monitoring
    fn disable_monitoring(&mut self) {
        self.processor.disable_monitoring();
    }
    
    /// Check if monitoring is enabled
    fn is_monitoring(&self) -> bool {
        self.processor.is_monitoring()
    }
    
    /// Update FFT configuration
    fn update_fft_config(&self, fft_size: usize, window_type: PyWindowType) {
        self.processor.update_fft_config(fft_size, window_type.into());
    }
    
    /// Get latest processing results
    /// 
    /// Returns:
    ///     Dictionary with keys: 'input_waveform', 'filtered_waveform',
    ///     'spectrum_magnitude', 'spectrum_frequencies', 'sample_rate'
    ///     or None if no new data
    fn get_results<'py>(&self, py: Python<'py>) -> Option<PyObject> {
        self.processor.get_results().map(|results| {
            let dict = pyo3::types::PyDict::new(py);
            
            dict.set_item("input_waveform", 
                         PyArray1::from_vec(py, results.input_waveform)).ok();
            dict.set_item("filtered_waveform",
                         PyArray1::from_vec(py, results.filtered_waveform)).ok();
            dict.set_item("spectrum_magnitude",
                         PyArray1::from_vec(py, results.spectrum_magnitude)).ok();
            dict.set_item("spectrum_frequencies",
                         PyArray1::from_vec(py, results.spectrum_frequencies)).ok();
            dict.set_item("sample_rate", results.sample_rate).ok();
            
            dict.into()
        })
    }
    
    /// List available audio devices
    #[staticmethod]
    fn list_devices() -> PyResult<Vec<String>> {
        AudioProcessor::list_devices()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))
    }
}
