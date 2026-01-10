//! Window functions for FIR filter design and spectral analysis
//! 
//! Ported from PartA_FIR_Filter_Design.m

use std::f64::consts::PI;

/// Window function types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowType {
    /// Hann window: w[n] = 0.5 - 0.5*cos(2πn/(M-1))
    /// Mainlobe width: 8π/M, Sidelobe attenuation: ~44 dB
    Hann,
    
    /// Hamming window: w[n] = 0.54 - 0.46*cos(2πn/(M-1))
    /// Mainlobe width: 8π/M, Sidelobe attenuation: ~53 dB
    Hamming,
    
    /// Blackman window: w[n] = 0.42 - 0.5*cos(2πn/(M-1)) + 0.08*cos(4πn/(M-1))
    /// Mainlobe width: 12π/M, Sidelobe attenuation: ~74 dB
    Blackman,
    
    /// Rectangular window (no windowing)
    Rectangular,
}

impl WindowType {
    /// Calculate required filter length M for given transition width
    /// Based on Table 7.1 from Oppenheim & Schafer
    /// 
    /// # Arguments
    /// * `delta_omega` - Transition width in radians
    /// 
    /// # Returns
    /// Filter length M (odd number for Type I linear phase)
    pub fn calculate_filter_length(&self, delta_omega: f64) -> usize {
        let a = match self {
            WindowType::Hann => 8.0,
            WindowType::Hamming => 8.0,
            WindowType::Blackman => 12.0,
            WindowType::Rectangular => 4.0,
        };
        
        let m = (a * PI / delta_omega).ceil() as usize;
        
        // Ensure odd length for Type I FIR (linear phase, symmetric)
        if m % 2 == 0 {
            m + 1
        } else {
            m
        }
    }
    
    /// Get mainlobe width factor
    pub fn mainlobe_width_factor(&self) -> f64 {
        match self {
            WindowType::Hann => 8.0,
            WindowType::Hamming => 8.0,
            WindowType::Blackman => 12.0,
            WindowType::Rectangular => 4.0,
        }
    }
    
    /// Get approximate stopband attenuation in dB
    pub fn stopband_attenuation_db(&self) -> f64 {
        match self {
            WindowType::Hann => -44.0,
            WindowType::Hamming => -53.0,
            WindowType::Blackman => -74.0,
            WindowType::Rectangular => -21.0,
        }
    }
}

/// Generate window coefficients
/// 
/// # Arguments
/// * `window_type` - Type of window function
/// * `length` - Number of samples (M)
/// 
/// # Returns
/// Vector of window coefficients w[n] for n = 0..M-1
pub fn generate_window(window_type: WindowType, length: usize) -> Vec<f64> {
    let m = length as f64;
    let mut window = Vec::with_capacity(length);
    
    match window_type {
        WindowType::Hann => {
            // w[n] = 0.5 - 0.5*cos(2πn/(M-1))
            // Also known as Hanning window
            for n in 0..length {
                let angle = 2.0 * PI * n as f64 / (m - 1.0);
                window.push(0.5 - 0.5 * angle.cos());
            }
        }
        
        WindowType::Hamming => {
            // w[n] = 0.54 - 0.46*cos(2πn/(M-1))
            // Optimized for sidelobe suppression
            for n in 0..length {
                let angle = 2.0 * PI * n as f64 / (m - 1.0);
                window.push(0.54 - 0.46 * angle.cos());
            }
        }
        
        WindowType::Blackman => {
            // w[n] = 0.42 - 0.5*cos(2πn/(M-1)) + 0.08*cos(4πn/(M-1))
            // Excellent sidelobe attenuation
            for n in 0..length {
                let angle1 = 2.0 * PI * n as f64 / (m - 1.0);
                let angle2 = 4.0 * PI * n as f64 / (m - 1.0);
                window.push(0.42 - 0.5 * angle1.cos() + 0.08 * angle2.cos());
            }
        }
        
        WindowType::Rectangular => {
            // w[n] = 1 for all n
            window.resize(length, 1.0);
        }
    }
    
    window
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_filter_length_calculation() {
        let delta_omega = 0.05 * PI; // From PartA specs
        
        let m_hann = WindowType::Hann.calculate_filter_length(delta_omega);
        let m_hamming = WindowType::Hamming.calculate_filter_length(delta_omega);
        let m_blackman = WindowType::Blackman.calculate_filter_length(delta_omega);
        
        // From MATLAB: M_hann = 161, M_hamming = 161, M_blackman = 241
        assert_eq!(m_hann, 161);
        assert_eq!(m_hamming, 161);
        assert_eq!(m_blackman, 241);
        
        // All should be odd
        assert_eq!(m_hann % 2, 1);
        assert_eq!(m_hamming % 2, 1);
        assert_eq!(m_blackman % 2, 1);
    }
    
    #[test]
    fn test_window_generation() {
        let length = 161;
        
        let hann = generate_window(WindowType::Hann, length);
        let hamming = generate_window(WindowType::Hamming, length);
        let blackman = generate_window(WindowType::Blackman, length);
        
        assert_eq!(hann.len(), length);
        assert_eq!(hamming.len(), length);
        assert_eq!(blackman.len(), length);
        
        // Check symmetry (Type I FIR)
        assert!((hann[0] - hann[length - 1]).abs() < 1e-10);
        assert!((hamming[0] - hamming[length - 1]).abs() < 1e-10);
        assert!((blackman[0] - blackman[length - 1]).abs() < 1e-10);
        
        // Check center values (should be 1.0 for symmetric windows)
        let center = length / 2;
        assert!((hann[center] - 1.0).abs() < 1e-10);
        assert!((hamming[center] - 1.0).abs() < 1e-10);
        assert!((blackman[center] - 1.0).abs() < 1e-10);
        
        // Hamming should have non-zero endpoints (0.08)
        assert!(hamming[0] > 0.07 && hamming[0] < 0.09);
    }
    
    #[test]
    fn test_rectangular_window() {
        let window = generate_window(WindowType::Rectangular, 100);
        assert_eq!(window.len(), 100);
        assert!(window.iter().all(|&w| w == 1.0));
    }
}
