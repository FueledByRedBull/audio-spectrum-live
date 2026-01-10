//! Spectral Workbench - High-Performance Audio DSP Core
//! 
//! Real-time audio filtering and spectral analysis engine with Python bindings.

pub mod audio;
pub mod filters;
pub mod spectrum;
pub mod python_bindings;

pub use filters::{WindowType, FirFilter};
pub use spectrum::SpectrumAnalyzer;
