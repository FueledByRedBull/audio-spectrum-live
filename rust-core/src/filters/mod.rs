//! FIR filter design and real-time filtering

pub mod windows;
pub mod design;
pub mod fir;
pub mod fast_fir;

pub use windows::{WindowType, generate_window};
pub use design::{FilterSpec, design_bandpass_fir, design_lowpass_fir, design_highpass_fir};
pub use fir::FirFilter;
pub use fast_fir::FastFirFilter;
