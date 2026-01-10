//! Python bindings for FIR filter operations

use pyo3::prelude::*;
use numpy::{PyArray1, PyReadonlyArray1};
use crate::filters::{WindowType, FilterSpec, design_bandpass_fir, FirFilter};

/// Window type enum exposed to Python
#[pyclass(name = "WindowType")]
#[derive(Clone)]
pub enum PyWindowType {
    Hann,
    Hamming,
    Blackman,
    Rectangular,
}

impl From<PyWindowType> for WindowType {
    fn from(py_win: PyWindowType) -> Self {
        match py_win {
            PyWindowType::Hann => WindowType::Hann,
            PyWindowType::Hamming => WindowType::Hamming,
            PyWindowType::Blackman => WindowType::Blackman,
            PyWindowType::Rectangular => WindowType::Rectangular,
        }
    }
}

/// FIR filter exposed to Python
#[pyclass(name = "FirFilter")]
pub struct PyFirFilter {
    filter: FirFilter,
}

#[pymethods]
impl PyFirFilter {
    /// Create a new FIR filter
    /// 
    /// Args:
    ///     coefficients: Filter coefficients as numpy array
    #[new]
    fn new(coefficients: PyReadonlyArray1<f64>) -> Self {
        let coeffs = coefficients.as_slice().unwrap().to_vec();
        Self {
            filter: FirFilter::new(coeffs),
        }
    }
    
    /// Design a bandpass FIR filter
    /// 
    /// Args:
    ///     omega_p1: Lower passband edge (normalized, units of π)
    ///     omega_p2: Upper passband edge (normalized, units of π)
    ///     delta_omega: Transition width (radians)
    ///     window_type: Window type
    /// 
    /// Returns:
    ///     New FIR filter instance
    #[staticmethod]
    fn design_bandpass(
        omega_p1: f64,
        omega_p2: f64,
        delta_omega: f64,
        window_type: PyWindowType,
    ) -> PyResult<Self> {
        let spec = FilterSpec::bandpass(omega_p1, omega_p2, delta_omega, window_type.into());
        let coeffs = design_bandpass_fir(&spec);
        
        Ok(Self {
            filter: FirFilter::new(coeffs),
        })
    }
    
    /// Design bandpass filter with Part A specifications
    /// 
    /// Args:
    ///     window_type: Window type
    /// 
    /// Returns:
    ///     New FIR filter instance with Part A specs
    #[staticmethod]
    fn design_part_a(window_type: PyWindowType) -> PyResult<Self> {
        let spec = FilterSpec::from_part_a(window_type.into());
        let coeffs = design_bandpass_fir(&spec);
        
        Ok(Self {
            filter: FirFilter::new(coeffs),
        })
    }
    
    /// Process a block of samples
    /// 
    /// Args:
    ///     input_signal: Input samples as numpy array
    /// 
    /// Returns:
    ///     Filtered output as numpy array
    fn process_block<'py>(
        &mut self,
        py: Python<'py>,
        input_signal: PyReadonlyArray1<f64>,
    ) -> PyResult<&'py PyArray1<f64>> {
        let input = input_signal.as_slice().unwrap();
        let output = self.filter.process_block(input);
        
        Ok(PyArray1::from_vec(py, output))
    }
    
    /// Reset filter state
    fn reset(&mut self) {
        self.filter.reset();
    }
    
    /// Get filter coefficients
    fn get_coefficients<'py>(&self, py: Python<'py>) -> PyResult<&'py PyArray1<f64>> {
        let coeffs = self.filter.coefficients().to_vec();
        Ok(PyArray1::from_vec(py, coeffs))
    }
    
    /// Get filter length
    fn length(&self) -> usize {
        self.filter.length()
    }
    
    /// Get group delay in samples
    fn group_delay(&self) -> f64 {
        self.filter.group_delay_samples()
    }
    
    /// Update filter coefficients
    /// 
    /// Args:
    ///     coefficients: New filter coefficients
    fn update_coefficients(&mut self, coefficients: PyReadonlyArray1<f64>) {
        let coeffs = coefficients.as_slice().unwrap().to_vec();
        self.filter.update_coefficients(coeffs);
    }
}
