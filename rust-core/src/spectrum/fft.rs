//! FFT engine using realfft for real-valued signals
//! 
//! Optimized for real-time spectral analysis

use realfft::{RealFftPlanner, RealToComplex};
use std::sync::Arc;

/// FFT engine for real-valued signals
pub struct FftEngine {
    /// FFT size (number of samples)
    fft_size: usize,
    
    /// Real FFT processor
    r2c: Arc<dyn RealToComplex<f64>>,
    
    /// Reusable input buffer
    input_buffer: Vec<f64>,
    
    /// Reusable output buffer (complex spectrum)
    output_buffer: Vec<num_complex::Complex<f64>>,
}

impl FftEngine {
    /// Create new FFT engine
    /// 
    /// # Arguments
    /// * `fft_size` - FFT size (number of samples)
    pub fn new(fft_size: usize) -> Self {
        let mut planner = RealFftPlanner::<f64>::new();
        let r2c = planner.plan_fft_forward(fft_size);
        
        let input_buffer = vec![0.0; fft_size];
        let output_buffer = vec![num_complex::Complex::new(0.0, 0.0); fft_size / 2 + 1];
        
        Self {
            fft_size,
            r2c,
            input_buffer,
            output_buffer,
        }
    }
    
    /// Compute FFT and return magnitude spectrum
    /// 
    /// # Arguments
    /// * `signal` - Input signal (will be zero-padded if shorter than fft_size)
    /// 
    /// # Returns
    /// Magnitude spectrum |X[k]| for k = 0..fft_size/2 (positive frequencies only)
    pub fn compute_magnitude(&mut self, signal: &[f64]) -> Vec<f64> {
        // Copy signal to input buffer with zero-padding
        let copy_len = signal.len().min(self.fft_size);
        self.input_buffer[..copy_len].copy_from_slice(&signal[..copy_len]);
        if copy_len < self.fft_size {
            self.input_buffer[copy_len..].fill(0.0);
        }
        
        // Compute FFT
        self.r2c
            .process(&mut self.input_buffer, &mut self.output_buffer)
            .expect("FFT processing failed");
        
        // Calculate magnitude
        self.output_buffer.iter().map(|c| c.norm()).collect()
    }
    
    /// Compute FFT and return magnitude spectrum in dB
    /// 
    /// # Arguments
    /// * `signal` - Input signal
    /// * `reference` - Reference value for dB calculation (default: 1.0)
    /// 
    /// # Returns
    /// Magnitude spectrum in dB: 20*log10(|X[k]|/reference)
    pub fn compute_magnitude_db(&mut self, signal: &[f64], reference: f64) -> Vec<f64> {
        let magnitude = self.compute_magnitude(signal);
        magnitude
            .iter()
            .map(|&mag| {
                let mag_clamped = mag.max(1e-10);  // Avoid log(0)
                20.0 * (mag_clamped / reference).log10()
            })
            .collect()
    }
    
    /// Compute power spectrum (magnitude squared)
    pub fn compute_power(&mut self, signal: &[f64]) -> Vec<f64> {
        self.compute_magnitude(signal)
            .iter()
            .map(|&mag| mag * mag)
            .collect()
    }
    
    /// Compute power spectrum in dB
    pub fn compute_power_db(&mut self, signal: &[f64], reference: f64) -> Vec<f64> {
        let power = self.compute_power(signal);
        power
            .iter()
            .map(|&p| {
                let p_clamped = p.max(1e-20);
                10.0 * (p_clamped / (reference * reference)).log10()
            })
            .collect()
    }
    
    /// Get FFT size
    pub fn fft_size(&self) -> usize {
        self.fft_size
    }
    
    /// Get number of frequency bins (fft_size/2 + 1 for real FFT)
    pub fn num_bins(&self) -> usize {
        self.fft_size / 2 + 1
    }
    
    /// Convert bin index to normalized frequency (units of π rad/sample)
    pub fn bin_to_frequency(&self, bin: usize) -> f64 {
        2.0 * bin as f64 / self.fft_size as f64
    }
    
    /// Get frequency axis in normalized units (0 to 1, where 1 = π rad/sample)
    pub fn frequency_axis(&self) -> Vec<f64> {
        (0..self.num_bins())
            .map(|bin| self.bin_to_frequency(bin))
            .collect()
    }
    
    /// Convert normalized frequency to Hz
    /// 
    /// # Arguments
    /// * `normalized_freq` - Frequency in units of π rad/sample (0 to 1)
    /// * `sample_rate` - Sample rate in Hz
    pub fn normalized_to_hz(normalized_freq: f64, sample_rate: f64) -> f64 {
        normalized_freq * sample_rate / 2.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;
    
    #[test]
    fn test_fft_dc_signal() {
        let mut fft = FftEngine::new(1024);
        
        // DC signal (constant)
        let signal = vec![1.0; 100];
        let spectrum = fft.compute_magnitude(&signal);
        
        // DC bin (k=0) should have high magnitude
        assert!(spectrum[0] > 90.0);  // ~100 for 100 samples
        
        // Other bins should be near zero
        assert!(spectrum[10] < 1.0);
    }
    
    #[test]
    fn test_fft_sine_wave() {
        let mut fft = FftEngine::new(1024);
        
        // Generate sine wave at normalized frequency 0.1 (0.1π rad/sample)
        let freq = 0.1;
        let signal: Vec<f64> = (0..1024)
            .map(|n| (freq * PI * n as f64).sin())
            .collect();
        
        let spectrum = fft.compute_magnitude(&signal);
        
        // Find peak bin
        let (peak_bin, &peak_mag) = spectrum
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .unwrap();
        
        // Peak should be at bin corresponding to frequency 0.1
        let expected_bin = (freq * 1024.0 / 2.0).round() as usize;
        assert!((peak_bin as i32 - expected_bin as i32).abs() <= 1);
        
        // Peak magnitude should be roughly N/2 for sine wave
        assert!(peak_mag > 400.0 && peak_mag < 600.0);
    }
    
    #[test]
    fn test_frequency_axis() {
        let fft = FftEngine::new(1024);
        let freqs = fft.frequency_axis();
        
        assert_eq!(freqs.len(), 513);  // 1024/2 + 1
        assert_eq!(freqs[0], 0.0);     // DC
        assert!((freqs[512] - 1.0).abs() < 1e-10);  // Nyquist (π rad/sample)
    }
}
