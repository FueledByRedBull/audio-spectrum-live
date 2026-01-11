//! Noise gate with RMS envelope detection
//!
//! Reduces gain when signal level falls below threshold, useful for
//! removing background noise during silent periods.

/// Noise gate processor
pub struct NoiseGate {
    /// Threshold in dB (e.g., -40.0)
    threshold_db: f64,
    
    /// Attack time constant (exponential smoothing coefficient)
    attack_coeff: f64,
    
    /// Release time constant (exponential smoothing coefficient)
    release_coeff: f64,
    
    /// Current envelope level (linear amplitude, not dB)
    envelope: f64,
    
    /// RMS window size in samples
    rms_window_size: usize,
    
    /// Circular buffer for RMS calculation
    rms_buffer: Vec<f64>,
    
    /// Current position in RMS buffer
    rms_cursor: usize,
    
    /// Sum of squares for efficient RMS calculation
    sum_of_squares: f64,
    
    /// Sample rate
    sample_rate: f64,
    
    /// Whether gate is currently open (for hysteresis)
    is_open: bool,
}

impl NoiseGate {
    /// Create a new noise gate
    ///
    /// # Arguments
    /// * `threshold_db` - Threshold in dB below which gate closes (e.g., -40.0)
    /// * `attack_ms` - Attack time in milliseconds (e.g., 10.0)
    /// * `release_ms` - Release time in milliseconds (e.g., 100.0)
    /// * `sample_rate` - Sample rate in Hz
    pub fn new(threshold_db: f64, attack_ms: f64, release_ms: f64, sample_rate: f64) -> Self {
        // Calculate time constants for exponential smoothing
        // tau = time_ms / 1000, coeff = exp(-1 / (tau * sample_rate))
        let attack_coeff = Self::time_constant_to_coeff(attack_ms, sample_rate);
        let release_coeff = Self::time_constant_to_coeff(release_ms, sample_rate);
        
        // RMS window: 50ms as specified
        let rms_window_size = (0.050 * sample_rate) as usize;
        
        Self {
            threshold_db,
            attack_coeff,
            release_coeff,
            envelope: 0.0,
            rms_window_size,
            rms_buffer: vec![0.0; rms_window_size],
            rms_cursor: 0,
            sum_of_squares: 0.0,
            sample_rate,
            is_open: false,
        }
    }
    
    /// Convert time constant in ms to exponential smoothing coefficient
    fn time_constant_to_coeff(time_ms: f64, sample_rate: f64) -> f64 {
        let tau = time_ms / 1000.0;  // Convert to seconds
        (-1.0 / (tau * sample_rate)).exp()
    }
    
    /// Update gate parameters
    pub fn set_threshold(&mut self, threshold_db: f64) {
        self.threshold_db = threshold_db;
    }
    
    /// Set attack time
    pub fn set_attack_time(&mut self, attack_ms: f64) {
        self.attack_coeff = Self::time_constant_to_coeff(attack_ms, self.sample_rate);
    }
    
    /// Set release time
    pub fn set_release_time(&mut self, release_ms: f64) {
        self.release_coeff = Self::time_constant_to_coeff(release_ms, self.sample_rate);
    }
    
    /// Process a single sample
    #[inline]
    fn process_sample(&mut self, input: f64) -> f64 {
        // Update RMS calculation (sliding window)
        let old_sample = self.rms_buffer[self.rms_cursor];
        self.rms_buffer[self.rms_cursor] = input;
        
        // Update sum of squares
        self.sum_of_squares -= old_sample * old_sample;
        self.sum_of_squares += input * input;
        
        // Advance cursor
        self.rms_cursor = (self.rms_cursor + 1) % self.rms_window_size;
        
        // Calculate RMS level
        let rms = (self.sum_of_squares / self.rms_window_size as f64).sqrt();
        
        // Convert to dB (with small epsilon to avoid log(0))
        let level_db = 20.0 * (rms + 1e-10).log10();
        
        // Determine if gate should be open
        // Use hysteresis: different thresholds for opening and closing
        let hysteresis_db = 3.0;  // 3 dB hysteresis
        
        if self.is_open {
            // Gate is open: close if level drops below (threshold - hysteresis)
            if level_db < self.threshold_db - hysteresis_db {
                self.is_open = false;
            }
        } else {
            // Gate is closed: open if level rises above threshold
            if level_db >= self.threshold_db {
                self.is_open = true;
            }
        }
        
        // Calculate target gain (0.0 when closed, 1.0 when open)
        let target_gain = if self.is_open { 1.0 } else { 0.0 };
        
        // Smooth gain transitions with attack/release
        let coeff = if target_gain > self.envelope {
            self.attack_coeff  // Opening: use attack time
        } else {
            self.release_coeff  // Closing: use release time
        };
        
        // Exponential smoothing: envelope = coeff * envelope + (1 - coeff) * target
        self.envelope = coeff * self.envelope + (1.0 - coeff) * target_gain;
        
        // Apply gain
        input * self.envelope
    }
    
    /// Process a block of samples
    pub fn process_block(&mut self, input: &[f64]) -> Vec<f64> {
        input.iter().map(|&x| self.process_sample(x)).collect()
    }
    
    /// Process a block in-place
    pub fn process_block_inplace(&mut self, buffer: &mut [f64]) {
        for sample in buffer.iter_mut() {
            *sample = self.process_sample(*sample);
        }
    }
    
    /// Reset gate state
    pub fn reset(&mut self) {
        self.envelope = 0.0;
        self.rms_buffer.fill(0.0);
        self.rms_cursor = 0;
        self.sum_of_squares = 0.0;
        self.is_open = false;
    }
    
    /// Get current envelope level (0.0 to 1.0)
    pub fn envelope(&self) -> f64 {
        self.envelope
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_noise_gate_opens_above_threshold() {
        let mut gate = NoiseGate::new(-40.0, 10.0, 100.0, 48000.0);
        
        // Feed strong signal (should open gate)
        let strong_signal = vec![0.1; 1000];  // About -20 dB
        let output = gate.process_block(&strong_signal);
        
        // After processing, envelope should be high
        assert!(gate.envelope() > 0.5);
    }
    
    #[test]
    fn test_noise_gate_closes_below_threshold() {
        let mut gate = NoiseGate::new(-40.0, 10.0, 100.0, 48000.0);
        
        // First open the gate with strong signal
        let strong_signal = vec![0.1; 1000];
        gate.process_block(&strong_signal);
        
        // Then feed weak signal (should close gate)
        let weak_signal = vec![0.0001; 10000];  // About -80 dB
        gate.process_block(&weak_signal);
        
        // Envelope should be low (but takes time due to release)
        assert!(gate.envelope() < 0.9);
    }
    
    #[test]
    fn test_noise_gate_hysteresis() {
        let mut gate = NoiseGate::new(-40.0, 1.0, 1.0, 48000.0);
        
        // Gate starts closed
        assert!(!gate.is_open);
        
        // Signal just above threshold should open it
        let signal_above = vec![0.01; 500];  // About -40 dB
        gate.process_block(&signal_above);
        assert!(gate.is_open);
        
        // Signal slightly below threshold shouldn't close it immediately (hysteresis)
        let signal_slightly_below = vec![0.007; 500];  // About -43 dB
        gate.process_block(&signal_slightly_below);
        assert!(gate.is_open);  // Still open due to hysteresis
    }
}
