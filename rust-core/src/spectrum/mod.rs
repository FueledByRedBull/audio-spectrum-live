//! Spectral analysis with FFT

pub mod fft;
pub mod windowing;
pub mod analysis;

pub use fft::FftEngine;
pub use windowing::apply_window;
pub use analysis::SpectrumAnalyzer;
