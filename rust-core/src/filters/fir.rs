//! Real-time FIR filter with state management
//! 
//! Implements direct convolution with overlap handling for continuous audio streams
//! Optimized with ring buffer (no allocations) and SIMD-friendly layout

/// Real-time FIR filter processor with zero-allocation ring buffer
pub struct FirFilter {
    /// Filter coefficients h[n]
    coefficients: Vec<f64>,
    
    /// Ring buffer state (delay line) - fixed size, no allocations
    /// Stores previous M samples where M is filter length
    state_buffer: Vec<f64>,
    
    /// Current write position in ring buffer
    cursor: usize,
    
    /// Filter length
    length: usize,
}

impl FirFilter {
    /// Create a new FIR filter with given coefficients
    /// 
    /// # Arguments
    /// * `coefficients` - Filter coefficients h[n] for n = 0..M-1
    pub fn new(coefficients: Vec<f64>) -> Self {
        let length = coefficients.len();
        
        // Allocate ring buffer once, never reallocate
        let state_buffer = vec![0.0; length];
        
        Self {
            coefficients,
            state_buffer,
            cursor: 0,
            length,
        }
    }
    
    /// Update filter coefficients (for real-time parameter changes)
    /// 
    /// # Arguments
    /// * `coefficients` - New filter coefficients
    /// 
    /// # Note
    /// If new filter has different length, state buffer is resized and cleared
    pub fn update_coefficients(&mut self, coefficients: Vec<f64>) {
        let new_length = coefficients.len();
        
        if new_length != self.length {
            // Resize state buffer
            self.state_buffer = vec![0.0; new_length];
            self.cursor = 0;
            self.length = new_length;
        }
        
        self.coefficients = coefficients;
    }
    
    /// Process single sample (zero-allocation)
    /// 
    /// # Arguments
    /// * `input` - Input sample x[n]
    /// 
    /// # Returns
    /// Filtered output sample y[n]
    #[inline]
    pub fn process_sample(&mut self, input: f64) -> f64 {
        // Write new sample to current cursor position
        self.state_buffer[self.cursor] = input;
        
        // Compute convolution: y[n] = Σ h[k] * x[n-k]
        // Read from ring buffer using modulo arithmetic
        let mut output = 0.0;
        for (k, &coeff) in self.coefficients.iter().enumerate() {
            // Calculate index with wraparound: (cursor - k) mod length
            let idx = (self.cursor + self.length - k) % self.length;
            output += coeff * self.state_buffer[idx];
        }
        
        // Advance cursor (with wraparound)
        self.cursor = (self.cursor + 1) % self.length;
        
        output
    }
    
    /// Process a block of samples
    /// 
    /// # Arguments
    /// * `input` - Input samples
    /// 
    /// # Returns
    /// Filtered output samples (same length as input)
    pub fn process_block(&mut self, input: &[f64]) -> Vec<f64> {
        input.iter().map(|&x| self.process_sample(x)).collect()
    }
    
    /// Process a block in-place (more efficient, overwrites input buffer)
    /// 
    /// # Arguments
    /// * `buffer` - Input/output buffer (modified in-place)
    pub fn process_block_inplace(&mut self, buffer: &mut [f64]) {
        for sample in buffer.iter_mut() {
            *sample = self.process_sample(*sample);
        }
    }
    
    /// Reset filter state (clear delay line)
    pub fn reset(&mut self) {
        self.state_buffer.fill(0.0);
        self.cursor = 0;
    }
    
    /// Get filter coefficients
    pub fn coefficients(&self) -> &[f64] {
        &self.coefficients
    }
    
    /// Get filter length
    pub fn length(&self) -> usize {
        self.length
    }
    
    /// Get group delay (for linear phase Type I FIR)
    pub fn group_delay_samples(&self) -> f64 {
        (self.length - 1) as f64 / 2.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fir_filter_basic() {
        // Simple 3-tap moving average: [1/3, 1/3, 1/3]
        let coeffs = vec![1.0 / 3.0; 3];
        let mut filter = FirFilter::new(coeffs);
        
        // Feed impulse
        let output1 = filter.process_sample(3.0);
        let output2 = filter.process_sample(0.0);
        let output3 = filter.process_sample(0.0);
        
        // Should get [1, 1, 1] output
        assert!((output1 - 1.0).abs() < 1e-10);
        assert!((output2 - 1.0).abs() < 1e-10);
        assert!((output3 - 1.0).abs() < 1e-10);
    }
    
    #[test]
    fn test_fir_filter_block_processing() {
        let coeffs = vec![0.5, 0.5];
        let mut filter = FirFilter::new(coeffs);
        
        let input = vec![1.0, 2.0, 3.0, 4.0];
        let output = filter.process_block(&input);
        
        // Expected: [0.5, 1.5, 2.5, 3.5]
        assert_eq!(output.len(), 4);
        assert!((output[0] - 0.5).abs() < 1e-10);
        assert!((output[1] - 1.5).abs() < 1e-10);
        assert!((output[2] - 2.5).abs() < 1e-10);
        assert!((output[3] - 3.5).abs() < 1e-10);
    }
    
    #[test]
    fn test_fir_filter_reset() {
        let coeffs = vec![1.0, 1.0];
        let mut filter = FirFilter::new(coeffs);
        
        // Process some samples
        filter.process_sample(1.0);
        filter.process_sample(2.0);
        
        // Reset
        filter.reset();
        
        // Next output should be as if starting fresh
        let output = filter.process_sample(1.0);
        assert!((output - 1.0).abs() < 1e-10);
    }
    
    #[test]
    fn test_fir_filter_update_coefficients() {
        let coeffs = vec![1.0, 0.0];
        let mut filter = FirFilter::new(coeffs);
        
        let output1 = filter.process_sample(1.0);
        assert!((output1 - 1.0).abs() < 1e-10);
        
        // Change to different coefficients
        filter.update_coefficients(vec![0.0, 1.0]);
        
        let output2 = filter.process_sample(2.0);
        // Should use previous sample (1.0) with coefficient 1.0
        assert!((output2 - 1.0).abs() < 1e-10);
    }
    
    #[test]
    fn test_ring_buffer_wraparound() {
        // Test that ring buffer correctly wraps around
        let coeffs = vec![1.0, 0.0, 0.0, 1.0]; // Tap first and fourth samples
        let mut filter = FirFilter::new(coeffs);
        
        // Fill buffer
        filter.process_sample(1.0);
        filter.process_sample(2.0);
        filter.process_sample(3.0);
        filter.process_sample(4.0);
        
        // Next sample should wrap around
        let output = filter.process_sample(5.0);
        // Should get h[0]*5 + h[3]*2 = 1*5 + 1*2 = 7
        assert!((output - 7.0).abs() < 1e-10);
    }
}
