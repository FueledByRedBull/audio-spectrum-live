//! Windowing functions for spectral analysis
//! 
//! Applies windows to time-domain signals before FFT to reduce spectral leakage

use crate::filters::windows::{WindowType, generate_window};

/// Apply window to signal
/// 
/// # Arguments
/// * `signal` - Input signal
/// * `window_type` - Type of window to apply
/// 
/// # Returns
/// Windowed signal
pub fn apply_window(signal: &[f64], window_type: WindowType) -> Vec<f64> {
    let window = generate_window(window_type, signal.len());
    
    signal
        .iter()
        .zip(window.iter())
        .map(|(&s, &w)| s * w)
        .collect()
}

/// Apply window in-place
pub fn apply_window_inplace(signal: &mut [f64], window_type: WindowType) {
    let window = generate_window(window_type, signal.len());
    
    for (s, w) in signal.iter_mut().zip(window.iter()) {
        *s *= w;
    }
}

/// Extract windowed segment from longer signal
/// 
/// # Arguments
/// * `signal` - Long signal
/// * `center` - Center index of window
/// * `window_length` - Length of window (L)
/// * `window_type` - Type of window
/// 
/// # Returns
/// Windowed segment centered at `center` with length `window_length`
/// Range: [center - L/2, center + L/2 - 1]
pub fn extract_windowed_segment(
    signal: &[f64],
    center: usize,
    window_length: usize,
    window_type: WindowType,
) -> Vec<f64> {
    let half_len = window_length / 2;
    let start = center.saturating_sub(half_len);
    let end = (center + half_len).min(signal.len());
    
    let mut segment = vec![0.0; window_length];
    let actual_len = end - start;
    
    segment[..actual_len].copy_from_slice(&signal[start..end]);
    
    apply_window_inplace(&mut segment, window_type);
    
    segment
}

/// Calculate window correction factor
/// 
/// When applying windows, the signal amplitude is reduced. This factor
/// can be used to correct the FFT magnitude.
/// 
/// # Arguments
/// * `window_type` - Type of window
/// * `length` - Window length
/// 
/// # Returns
/// Correction factor (multiply FFT magnitude by this)
pub fn window_correction_factor(window_type: WindowType, length: usize) -> f64 {
    let window = generate_window(window_type, length);
    let sum: f64 = window.iter().sum();
    length as f64 / sum
}

/// Calculate window power correction factor (for power spectral density)
pub fn window_power_correction_factor(window_type: WindowType, length: usize) -> f64 {
    let window = generate_window(window_type, length);
    let sum_sq: f64 = window.iter().map(|&w| w * w).sum();
    length as f64 / sum_sq
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_apply_window() {
        let signal = vec![1.0; 100];
        let windowed = apply_window(&signal, WindowType::Hamming);
        
        assert_eq!(windowed.len(), 100);
        
        // Center should be close to 1.0
        assert!((windowed[50] - 1.0).abs() < 0.01);
        
        // Edges should be reduced (Hamming ~0.08)
        assert!(windowed[0] < 0.1);
        assert!(windowed[99] < 0.1);
    }
    
    #[test]
    fn test_correction_factor() {
        let factor_rect = window_correction_factor(WindowType::Rectangular, 100);
        let factor_hamming = window_correction_factor(WindowType::Hamming, 100);
        
        // Rectangular window has no correction needed
        assert!((factor_rect - 1.0).abs() < 0.01);
        
        // Hamming window reduces amplitude, so correction > 1
        assert!(factor_hamming > 1.5 && factor_hamming < 2.5);
    }
}
