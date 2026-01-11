//! Spectral Workbench - High-Performance Audio DSP Core
//! 
//! Real-time audio filtering and spectral analysis engine with Python bindings.

// Suppress PyO3 non-local impl warnings (harmless macro-generated code)
#![allow(non_local_definitions)]

pub mod audio;
pub mod filters;
pub mod spectrum;
pub mod python_bindings;

pub use filters::{WindowType, FirFilter};
pub use spectrum::SpectrumAnalyzer;
