//! PyO3 bindings for Python integration

use pyo3::prelude::*;

mod filter_bindings;
mod spectrum_bindings;
mod audio_bindings;
mod processor_bindings;

/// Python module definition
#[pymodule]
fn spectral_workbench(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<filter_bindings::PyFirFilter>()?;
    m.add_class::<spectrum_bindings::PySpectrumAnalyzer>()?;
    m.add_class::<audio_bindings::PyAudioEngine>()?;
    m.add_class::<processor_bindings::PyAudioProcessor>()?;
    
    // Add WindowType enum
    m.add_class::<filter_bindings::PyWindowType>()?;
    
    Ok(())
}
