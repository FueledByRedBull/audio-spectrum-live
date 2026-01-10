//! High-level spectrum analyzer
//! 
//! Combines FFT engine with windowing for real-time spectral analysis

use super::fft::FftEngine;
use super::windowing::{apply_window, window_correction_factor};
use crate::filters::windows::WindowType;

/// Spectrum analyzer configuration
#[derive(Debug, Clone)]
pub struct AnalyzerConfig {
    /// FFT size (number of samples, should be power of 2)
    pub fft_size: usize,
    
    /// Window type for spectral analysis
    pub window_type: WindowType,
    
    /// Sample rate in Hz
    pub sample_rate: f64,
    
    /// Apply amplitude correction for windowing
    pub apply_correction: bool,
}

impl Default for AnalyzerConfig {
    fn default() -> Self {
        Self {
            fft_size: 2048,
            window_type: WindowType::Hamming,
            sample_rate: 48000.0,
            apply_correction: true,
        }
    }
}

/// Real-time spectrum analyzer
pub struct SpectrumAnalyzer {
    config: AnalyzerConfig,
    fft_engine: FftEngine,
    correction_factor: f64,
}

impl SpectrumAnalyzer {
    /// Create new spectrum analyzer
    pub fn new(config: AnalyzerConfig) -> Self {
        let fft_engine = FftEngine::new(config.fft_size);
        let correction_factor = if config.apply_correction {
            window_correction_factor(config.window_type, config.fft_size)
        } else {
            1.0
        };
        
        Self {
            config,
            fft_engine,
            correction_factor,
        }
    }
    
    /// Analyze signal and return magnitude spectrum
    /// 
    /// # Arguments
    /// * `signal` - Input signal (will be windowed and zero-padded if needed)
    /// 
    /// # Returns
    /// Magnitude spectrum |X[k]| for positive frequencies
    pub fn analyze(&mut self, signal: &[f64]) -> Vec<f64> {
        // Apply window
        let windowed = apply_window(signal, self.config.window_type);
        
        // Compute FFT magnitude
        let mut spectrum = self.fft_engine.compute_magnitude(&windowed);
        
        // Apply correction factor
        if self.config.apply_correction {
            for s in spectrum.iter_mut() {
                *s *= self.correction_factor;
            }
        }
        
        spectrum
    }
    
    /// Analyze and return magnitude in dB
    /// 
    /// # Arguments
    /// * `signal` - Input signal
    /// * `reference` - Reference level for dB (default: 1.0)
    /// 
    /// # Returns
    /// Magnitude spectrum in dB
    pub fn analyze_db(&mut self, signal: &[f64], reference: f64) -> Vec<f64> {
        let spectrum = self.analyze(signal);
        spectrum
            .iter()
            .map(|&mag| {
                let mag_clamped = mag.max(1e-10);
                20.0 * (mag_clamped / reference).log10()
            })
            .collect()
    }
    
    /// Get frequency bins in Hz
    pub fn frequency_bins_hz(&self) -> Vec<f64> {
        self.fft_engine
            .frequency_axis()
            .iter()
            .map(|&f_norm| FftEngine::normalized_to_hz(f_norm, self.config.sample_rate))
            .collect()
    }
    
    /// Get frequency bins in normalized units (0 to 1, where 1 = Nyquist)
    pub fn frequency_bins_normalized(&self) -> Vec<f64> {
        self.fft_engine.frequency_axis()
    }
    
    /// Update configuration
    pub fn update_config(&mut self, config: AnalyzerConfig) {
        let needs_new_fft = config.fft_size != self.config.fft_size;
        
        if needs_new_fft {
            self.fft_engine = FftEngine::new(config.fft_size);
        }
        
        self.correction_factor = if config.apply_correction {
            window_correction_factor(config.window_type, config.fft_size)
        } else {
            1.0
        };
        
        self.config = config;
    }
    
    /// Get current configuration
    pub fn config(&self) -> &AnalyzerConfig {
        &self.config
    }
    
    /// Get number of frequency bins
    pub fn num_bins(&self) -> usize {
        self.fft_engine.num_bins()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;
    
    #[test]
    fn test_analyzer_basic() {
        let config = AnalyzerConfig {
            fft_size: 1024,
            window_type: WindowType::Hamming,
            sample_rate: 48000.0,
            apply_correction: true,
        };
        
        let mut analyzer = SpectrumAnalyzer::new(config);
        
        // Generate 1 kHz sine wave
        let freq_hz = 1000.0;
        let signal: Vec<f64> = (0..1024)
            .map(|n| (2.0 * PI * freq_hz * n as f64 / 48000.0).sin())
            .collect();
        
        let spectrum = analyzer.analyze(&signal);
        
        // Should have correct number of bins
        assert_eq!(spectrum.len(), 513);
        
        // Peak should be near 1 kHz
        let freqs = analyzer.frequency_bins_hz();
        let (peak_idx, _) = spectrum
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .unwrap();
        
        let peak_freq = freqs[peak_idx];
        assert!((peak_freq - freq_hz).abs() < 100.0);  // Within 100 Hz
    }
    
    #[test]
    fn test_analyzer_db() {
        let config = AnalyzerConfig::default();
        let mut analyzer = SpectrumAnalyzer::new(config);
        
        let signal = vec![1.0; 1024];
        let spectrum_db = analyzer.analyze_db(&signal, 1.0);
        
        // DC component should be high
        assert!(spectrum_db[0] > 50.0);
    }
}
