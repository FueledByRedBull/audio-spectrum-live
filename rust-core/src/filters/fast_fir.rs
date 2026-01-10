//! FFT-based fast convolution for long FIR filters
//! 
//! Implements overlap-add method with frequency-domain multiplication
//! Complexity: O(N log N) vs O(N*M) for time-domain

use rustfft::{FftPlanner, num_complex::Complex};

/// FFT-based FIR filter for long impulse responses
/// Uses overlap-add with FFT convolution: O(N log N) instead of O(N*M)
pub struct FastFirFilter {
    /// Filter coefficients in frequency domain
    h_fft: Vec<Complex<f64>>,
    
    /// FFT size (must be power of 2, >= 2*max(N,M))
    fft_size: usize,
    
    /// Block size for input
    block_size: usize,
    
    /// Filter length
    filter_length: usize,
    
    /// Overlap buffer from previous block
    overlap: Vec<f64>,
    
    /// FFT planner (forward)
    fft: std::sync::Arc<dyn rustfft::Fft<f64>>,
    
    /// IFFT planner (inverse)
    ifft: std::sync::Arc<dyn rustfft::Fft<f64>>,
    
    /// Reusable buffers
    input_buffer: Vec<Complex<f64>>,
    output_buffer: Vec<Complex<f64>>,
}

impl FastFirFilter {
    /// Create new FFT-based filter
    /// 
    /// # Arguments
    /// * `coefficients` - Filter coefficients h[n]
    /// * `block_size` - Input block size (e.g., 1024)
    /// 
    /// # Note
    /// FFT size is chosen as next power of 2 >= (block_size + filter_length - 1)
    pub fn new(coefficients: Vec<f64>, block_size: usize) -> Self {
        let filter_length = coefficients.len();
        
        // FFT size must be at least block_size + filter_length - 1
        let min_fft_size = block_size + filter_length - 1;
        let fft_size = min_fft_size.next_power_of_two();
        
        // Create FFT planners
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(fft_size);
        let ifft = planner.plan_fft_inverse(fft_size);
        
        // Transform filter coefficients to frequency domain
        let mut h_time = vec![Complex::new(0.0, 0.0); fft_size];
        for (i, &coeff) in coefficients.iter().enumerate() {
            h_time[i] = Complex::new(coeff, 0.0);
        }
        
        let mut h_fft = h_time.clone();
        fft.process(&mut h_fft);
        
        // Allocate buffers
        let input_buffer = vec![Complex::new(0.0, 0.0); fft_size];
        let output_buffer = vec![Complex::new(0.0, 0.0); fft_size];
        let overlap = vec![0.0; filter_length - 1];
        
        Self {
            h_fft,
            fft_size,
            block_size,
            filter_length,
            overlap,
            fft,
            ifft,
            input_buffer,
            output_buffer,
        }
    }
    
    /// Process block using FFT-based overlap-add
    /// 
    /// # Arguments
    /// * `input` - Input block (length should be <= block_size)
    /// 
    /// # Returns
    /// Filtered output block (same length as input)
    pub fn process_block(&mut self, input: &[f64]) -> Vec<f64> {
        let n = input.len().min(self.block_size);
        
        // 1. Copy input to complex buffer and zero-pad
        for i in 0..n {
            self.input_buffer[i] = Complex::new(input[i], 0.0);
        }
        for i in n..self.fft_size {
            self.input_buffer[i] = Complex::new(0.0, 0.0);
        }
        
        // 2. Forward FFT of input
        self.fft.process(&mut self.input_buffer);
        
        // 3. Multiply in frequency domain (convolution in time domain)
        for i in 0..self.fft_size {
            self.output_buffer[i] = self.input_buffer[i] * self.h_fft[i];
        }
        
        // 4. Inverse FFT
        self.ifft.process(&mut self.output_buffer);
        
        // 5. Scale by 1/N (IFFT normalization)
        let scale = 1.0 / self.fft_size as f64;
        
        // 6. Overlap-add: combine with tail from previous block
        let mut output = vec![0.0; n];
        for i in 0..n {
            output[i] = self.output_buffer[i].re * scale;
            
            // Add overlap from previous block
            if i < self.overlap.len() {
                output[i] += self.overlap[i];
            }
        }
        
        // 7. Save tail for next block
        for i in 0..(self.filter_length - 1) {
            if n + i < self.fft_size {
                self.overlap[i] = self.output_buffer[n + i].re * scale;
            } else {
                self.overlap[i] = 0.0;
            }
        }
        
        output
    }
    
    /// Reset filter state
    pub fn reset(&mut self) {
        self.overlap.fill(0.0);
    }
    
    /// Get filter length
    pub fn filter_length(&self) -> usize {
        self.filter_length
    }
    
    /// Get block size
    pub fn block_size(&self) -> usize {
        self.block_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filters::design::*;
    use crate::filters::windows::WindowType;
    
    #[test]
    fn test_fast_fir_impulse() {
        // Simple 5-tap filter
        let h = vec![0.1, 0.2, 0.4, 0.2, 0.1];
        let mut filter = FastFirFilter::new(h.clone(), 32);
        
        // Impulse input
        let mut input = vec![0.0; 32];
        input[0] = 1.0;
        
        let output = filter.process_block(&input);
        
        // First 5 samples should match filter coefficients
        for (i, &expected) in h.iter().enumerate() {
            assert!((output[i] - expected).abs() < 1e-10, 
                    "Mismatch at {}: {} vs {}", i, output[i], expected);
        }
    }
    
    #[test]
    fn test_fast_fir_vs_direct() {
        // Design a real filter
        let spec = FilterSpec::from_part_a(WindowType::Hamming);
        let coeffs = design_bandpass_fir(&spec);
        
        let mut fast_filter = FastFirFilter::new(coeffs.clone(), 512);
        let mut direct_filter = super::FirFilter::new(coeffs);
        
        // Random input
        let input: Vec<f64> = (0..512).map(|i| (i as f64 * 0.01).sin()).collect();
        
        let fast_output = fast_filter.process_block(&input);
        let direct_output = direct_filter.process_block(&input);
        
        // Compare outputs (should be very close)
        for i in 0..input.len() {
            let diff = (fast_output[i] - direct_output[i]).abs();
            assert!(diff < 1e-6, "Mismatch at {}: diff = {}", i, diff);
        }
    }
}
