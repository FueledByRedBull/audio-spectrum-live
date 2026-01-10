//! Python bindings for spectrum analysis

use pyo3::prelude::*;
use numpy::{PyArray1, PyReadonlyArray1};
use crate::spectrum::{SpectrumAnalyzer, analysis::AnalyzerConfig};
use super::filter_bindings::PyWindowType;

/// Spectrum analyzer exposed to Python
#[pyclass(name = "SpectrumAnalyzer")]
pub struct PySpectrumAnalyzer {
    analyzer: SpectrumAnalyzer,
}

#[pymethods]
impl PySpectrumAnalyzer {
    /// Create a new spectrum analyzer
    /// 
    /// Args:
    ///     fft_size: FFT size (number of samples, should be power of 2)
    ///     window_type: Window type for analysis
    ///     sample_rate: Sample rate in Hz
    ///     apply_correction: Whether to apply amplitude correction for windowing
    #[new]
    #[pyo3(signature = (fft_size=2048, window_type=PyWindowType::Hamming, sample_rate=48000.0, apply_correction=true))]
    fn new(
        fft_size: usize,
        window_type: PyWindowType,
        sample_rate: f64,
        apply_correction: bool,
    ) -> Self {
        let config = AnalyzerConfig {
            fft_size,
            window_type: window_type.into(),
            sample_rate,
            apply_correction,
        };
        
        Self {
            analyzer: SpectrumAnalyzer::new(config),
        }
    }
    
    /// Analyze signal and return magnitude spectrum
    /// 
    /// Args:
    ///     signal: Input signal as numpy array
    /// 
    /// Returns:
    ///     Magnitude spectrum as numpy array
    fn analyze<'py>(
        &mut self,
        py: Python<'py>,
        signal: PyReadonlyArray1<f64>,
    ) -> PyResult<&'py PyArray1<f64>> {
        let sig = signal.as_slice().unwrap();
        let spectrum = self.analyzer.analyze(sig);
        
        Ok(PyArray1::from_vec(py, spectrum))
    }
    
    /// Analyze signal and return magnitude in dB
    /// 
    /// Args:
    ///     signal: Input signal as numpy array
    ///     reference: Reference level for dB calculation (default: 1.0)
    /// 
    /// Returns:
    ///     Magnitude spectrum in dB as numpy array
    #[pyo3(signature = (signal, reference=1.0))]
    fn analyze_db<'py>(
        &mut self,
        py: Python<'py>,
        signal: PyReadonlyArray1<f64>,
        reference: f64,
    ) -> PyResult<&'py PyArray1<f64>> {
        let sig = signal.as_slice().unwrap();
        let spectrum = self.analyzer.analyze_db(sig, reference);
        
        Ok(PyArray1::from_vec(py, spectrum))
    }
    
    /// Get frequency bins in Hz
    /// 
    /// Returns:
    ///     Frequency bins as numpy array
    fn frequency_bins_hz<'py>(&self, py: Python<'py>) -> PyResult<&'py PyArray1<f64>> {
        let freqs = self.analyzer.frequency_bins_hz();
        Ok(PyArray1::from_vec(py, freqs))
    }
    
    /// Get frequency bins in normalized units (0 to 1, where 1 = Nyquist)
    /// 
    /// Returns:
    ///     Normalized frequency bins as numpy array
    fn frequency_bins_normalized<'py>(&self, py: Python<'py>) -> PyResult<&'py PyArray1<f64>> {
        let freqs = self.analyzer.frequency_bins_normalized();
        Ok(PyArray1::from_vec(py, freqs))
    }
    
    /// Get number of frequency bins
    fn num_bins(&self) -> usize {
        self.analyzer.num_bins()
    }
    
    /// Update configuration
    /// 
    /// Args:
    ///     fft_size: New FFT size
    ///     window_type: New window type
    ///     sample_rate: New sample rate
    ///     apply_correction: Whether to apply correction
    #[pyo3(signature = (fft_size=None, window_type=None, sample_rate=None, apply_correction=None))]
    fn update_config(
        &mut self,
        fft_size: Option<usize>,
        window_type: Option<PyWindowType>,
        sample_rate: Option<f64>,
        apply_correction: Option<bool>,
    ) {
        let mut config = self.analyzer.config().clone();
        
        if let Some(size) = fft_size {
            config.fft_size = size;
        }
        if let Some(win) = window_type {
            config.window_type = win.into();
        }
        if let Some(sr) = sample_rate {
            config.sample_rate = sr;
        }
        if let Some(corr) = apply_correction {
            config.apply_correction = corr;
        }
        
        self.analyzer.update_config(config);
    }
    
    /// Get current sample rate
    fn get_sample_rate(&self) -> f64 {
        self.analyzer.config().sample_rate
    }
    
    /// Get current FFT size
    fn get_fft_size(&self) -> usize {
        self.analyzer.config().fft_size
    }
}
