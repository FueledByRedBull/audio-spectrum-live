//! FIR filter design using the windowing method
//! 
//! Ported from PartA_FIR_Filter_Design.m

use super::windows::{WindowType, generate_window};
use std::f64::consts::PI;

/// Filter specifications for bandpass FIR design
#[derive(Debug, Clone)]
pub struct FilterSpec {
    /// Lower passband edge (normalized frequency, units of π rad/sample)
    pub omega_p1: f64,
    
    /// Upper passband edge (normalized frequency, units of π rad/sample)
    pub omega_p2: f64,
    
    /// Lower stopband edge (normalized frequency, units of π rad/sample)
    pub omega_s1: f64,
    
    /// Upper stopband edge (normalized frequency, units of π rad/sample)
    pub omega_s2: f64,
    
    /// Transition width (radians)
    pub delta_omega: f64,
    
    /// Window type
    pub window_type: WindowType,
}

impl FilterSpec {
    /// Create filter spec matching PartA requirements
    /// Passband: [0.4π, 0.6π], Stopband: [0, 0.35π] ∪ [0.65π, π]
    pub fn from_part_a(window_type: WindowType) -> Self {
        Self {
            omega_p1: 0.4,  // units of π
            omega_p2: 0.6,
            omega_s1: 0.35,
            omega_s2: 0.65,
            delta_omega: 0.05 * PI,
            window_type,
        }
    }
    
    /// Create custom bandpass filter spec
    pub fn bandpass(
        omega_p1: f64,
        omega_p2: f64,
        delta_omega: f64,
        window_type: WindowType,
    ) -> Self {
        // Calculate stopband edges based on transition width
        let omega_s1 = omega_p1 - delta_omega / PI;
        let omega_s2 = omega_p2 + delta_omega / PI;
        
        Self {
            omega_p1,
            omega_p2,
            omega_s1,
            omega_s2,
            delta_omega,
            window_type,
        }
    }
    
    /// Calculate cutoff frequencies (midpoint of transition bands)
    pub fn cutoff_frequencies(&self) -> (f64, f64) {
        let wc1 = (self.omega_s1 + self.omega_p1) / 2.0;
        let wc2 = (self.omega_p2 + self.omega_s2) / 2.0;
        (wc1, wc2)
    }
}

/// Design a bandpass FIR filter using the windowing method
/// 
/// # Algorithm
/// 1. Calculate ideal impulse response using sinc functions
/// 2. Generate window of appropriate length
/// 3. Multiply ideal response by window to get filter coefficients
/// 
/// # Arguments
/// * `spec` - Filter specifications
/// 
/// # Returns
/// Vector of filter coefficients h[n] for n = 0..M-1
pub fn design_bandpass_fir(spec: &FilterSpec) -> Vec<f64> {
    // Calculate filter length based on window type and transition width
    let m = spec.window_type.calculate_filter_length(spec.delta_omega);
    
    // Get cutoff frequencies (normalized by π)
    let (wc1, wc2) = spec.cutoff_frequencies();
    
    // Convert to radians
    let wc1_rad = wc1 * PI;
    let wc2_rad = wc2 * PI;
    
    // Generate window
    let window = generate_window(spec.window_type, m);
    
    // Calculate ideal impulse response for bandpass filter
    // h_ideal[n] = (sin(wc2*n) - sin(wc1*n)) / (π*n)
    // Special case at n = (M-1)/2 (center): h_ideal = (wc2 - wc1) / π
    
    let center = (m - 1) as f64 / 2.0;
    let mut h = Vec::with_capacity(m);
    
    for n in 0..m {
        let n_shifted = n as f64 - center;  // Shift to center at 0
        
        let h_ideal = if n_shifted.abs() < 1e-10 {
            // At center point: limit as n -> 0
            (wc2_rad - wc1_rad) / PI
        } else {
            // General case: sinc function difference
            (wc2_rad * n_shifted).sin() / (PI * n_shifted)
                - (wc1_rad * n_shifted).sin() / (PI * n_shifted)
        };
        
        // Apply window
        h.push(h_ideal * window[n]);
    }
    
    h
}

/// Design a lowpass FIR filter
pub fn design_lowpass_fir(cutoff: f64, delta_omega: f64, window_type: WindowType) -> Vec<f64> {
    let m = window_type.calculate_filter_length(delta_omega);
    let window = generate_window(window_type, m);
    let wc_rad = cutoff * PI;
    
    let center = (m - 1) as f64 / 2.0;
    let mut h = Vec::with_capacity(m);
    
    for n in 0..m {
        let n_shifted = n as f64 - center;
        
        let h_ideal = if n_shifted.abs() < 1e-10 {
            wc_rad / PI
        } else {
            (wc_rad * n_shifted).sin() / (PI * n_shifted)
        };
        
        h.push(h_ideal * window[n]);
    }
    
    h
}

/// Design a highpass FIR filter
pub fn design_highpass_fir(cutoff: f64, delta_omega: f64, window_type: WindowType) -> Vec<f64> {
    let m = window_type.calculate_filter_length(delta_omega);
    let window = generate_window(window_type, m);
    let wc_rad = cutoff * PI;
    
    let center = (m - 1) as f64 / 2.0;
    let mut h = Vec::with_capacity(m);
    
    for n in 0..m {
        let n_shifted = n as f64 - center;
        
        // Highpass = impulse - lowpass
        // h_ideal[n] = δ[n] - sin(wc*n)/(π*n)
        let h_ideal = if n_shifted.abs() < 1e-10 {
            // At center: 1 - wc/π (already includes delta function)
            1.0 - wc_rad / PI
        } else {
            // Off-center: -sin(wc*n)/(π*n)
            -((wc_rad * n_shifted).sin() / (PI * n_shifted))
        };
        
        h.push(h_ideal * window[n]);
    }
    
    h
}

/// Calculate frequency response at given frequencies
/// 
/// # Arguments
/// * `h` - Filter coefficients
/// * `frequencies` - Normalized frequencies (units of π rad/sample)
/// 
/// # Returns
/// Complex frequency response H(e^jω)
pub fn frequency_response(h: &[f64], frequencies: &[f64]) -> Vec<num_complex::Complex64> {
    use num_complex::Complex64;
    
    let mut response = Vec::with_capacity(frequencies.len());
    
    for &omega in frequencies {
        let omega_rad = omega * PI;
        let mut sum = Complex64::new(0.0, 0.0);
        
        for (n, &h_n) in h.iter().enumerate() {
            let phase = -(omega_rad * n as f64);
            sum += h_n * Complex64::new(phase.cos(), phase.sin());
        }
        
        response.push(sum);
    }
    
    response
}

/// Calculate magnitude response in dB
pub fn magnitude_response_db(h: &[f64], frequencies: &[f64]) -> Vec<f64> {
    frequency_response(h, frequencies)
        .iter()
        .map(|c| 20.0 * c.norm().log10())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_bandpass_design_part_a() {
        // Test with Hamming window (Part A specs)
        let spec = FilterSpec::from_part_a(WindowType::Hamming);
        let h = design_bandpass_fir(&spec);
        
        // Should have length 161 (from Part A)
        assert_eq!(h.len(), 161);
        
        // Check symmetry (Type I FIR)
        for i in 0..h.len() / 2 {
            let diff = (h[i] - h[h.len() - 1 - i]).abs();
            assert!(diff < 1e-10, "Not symmetric at index {}: {} vs {}", 
                    i, h[i], h[h.len() - 1 - i]);
        }
        
        // Sum of coefficients should be small for bandpass (DC gain ~ 0)
        let sum: f64 = h.iter().sum();
        assert!(sum.abs() < 0.1, "DC gain too large: {}", sum);
    }
    
    #[test]
    fn test_cutoff_frequencies() {
        let spec = FilterSpec::from_part_a(WindowType::Hamming);
        let (wc1, wc2) = spec.cutoff_frequencies();
        
        // From MATLAB: wc1 = 0.375, wc2 = 0.625
        assert!((wc1 - 0.375).abs() < 1e-10);
        assert!((wc2 - 0.625).abs() < 1e-10);
    }
    
    #[test]
    fn test_lowpass_design() {
        let h = design_lowpass_fir(0.5, 0.05 * PI, WindowType::Hamming);
        
        // Check odd length
        assert_eq!(h.len() % 2, 1);
        
        // Check symmetry
        for i in 0..h.len() / 2 {
            assert!((h[i] - h[h.len() - 1 - i]).abs() < 1e-10);
        }
        
        // DC gain should be close to 1 for lowpass
        let sum: f64 = h.iter().sum();
        assert!((sum - 1.0).abs() < 0.1);
    }
}
