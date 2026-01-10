//! Unified audio processor - keeps hot loop in Rust
//! 
//! Eliminates Python/Rust boundary overhead by processing audio entirely in Rust

use crate::filters::{FirFilter, FastFirFilter, FilterSpec, WindowType, design_bandpass_fir};
use crate::spectrum::{SpectrumAnalyzer, analysis::AnalyzerConfig};
use crate::audio::{AudioInput, AudioRingBuffer, input::list_input_devices};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

/// Results from audio processing (sent to Python)
#[derive(Clone)]
pub struct ProcessingResults {
    /// Input waveform samples
    pub input_waveform: Vec<f64>,
    
    /// Filtered waveform samples
    pub filtered_waveform: Vec<f64>,
    
    /// Spectrum magnitude in dB
    pub spectrum_magnitude: Vec<f64>,
    
    /// Frequency bins in Hz
    pub spectrum_frequencies: Vec<f64>,
    
    /// Sample rate
    pub sample_rate: f64,
}

/// High-performance audio processor
/// 
/// Runs audio capture, filtering, and FFT analysis in Rust thread
/// Python only reads results (no per-sample boundary crossing)
pub struct AudioProcessor {
    /// Filter (use fast version for long filters)
    filter: Arc<Mutex<Option<Box<dyn FilterTrait + Send>>>>,
    
    /// Spectrum analyzer
    analyzer: Arc<Mutex<SpectrumAnalyzer>>,
    
    /// Latest processing results
    results: Arc<Mutex<Option<ProcessingResults>>>,
    
    /// Audio input stream
    audio_input: Option<AudioInput>,
    
    /// Processing thread handle
    process_thread: Option<std::thread::JoinHandle<()>>,
    
    /// Running flag
    running: Arc<AtomicBool>,
    
    /// Bypass flag
    bypass: Arc<AtomicBool>,
    
    /// Sample rate
    sample_rate: f64,
}

/// Trait for polymorphic filter types
trait FilterTrait {
    fn process_block(&mut self, input: &[f64]) -> Vec<f64>;
    fn reset(&mut self);
}

impl FilterTrait for FirFilter {
    fn process_block(&mut self, input: &[f64]) -> Vec<f64> {
        FirFilter::process_block(self, input)
    }
    
    fn reset(&mut self) {
        FirFilter::reset(self)
    }
}

impl FilterTrait for FastFirFilter {
    fn process_block(&mut self, input: &[f64]) -> Vec<f64> {
        FastFirFilter::process_block(self, input)
    }
    
    fn reset(&mut self) {
        FastFirFilter::reset(self)
    }
}

impl AudioProcessor {
    /// Create new audio processor
    pub fn new() -> Self {
        let sample_rate = 48000.0;
        
        let analyzer_config = AnalyzerConfig {
            fft_size: 2048,
            window_type: WindowType::Hamming,
            sample_rate,
            apply_correction: true,
        };
        
        Self {
            filter: Arc::new(Mutex::new(None)),
            analyzer: Arc::new(Mutex::new(SpectrumAnalyzer::new(analyzer_config))),
            results: Arc::new(Mutex::new(None)),
            audio_input: None,
            process_thread: None,
            running: Arc::new(AtomicBool::new(false)),
            bypass: Arc::new(AtomicBool::new(false)),
            sample_rate,
        }
    }
    
    /// Start audio capture and processing
    pub fn start(&mut self) -> Result<String, String> {
        // Create ring buffer
        let rb = AudioRingBuffer::new(96000);
        let (producer, consumer) = rb.split();
        
        // Start audio input
        let input = AudioInput::from_default_device(producer)
            .map_err(|e| format!("Failed to start audio: {}", e))?;
        
        let device_name = input.device_info().name.clone();
        self.sample_rate = input.device_info().sample_rate as f64;
        
        // Update analyzer sample rate
        if let Ok(mut analyzer) = self.analyzer.lock() {
            let old_config = analyzer.config().clone();
            analyzer.update_config(AnalyzerConfig {
                fft_size: old_config.fft_size,
                window_type: old_config.window_type,
                sample_rate: self.sample_rate,
                apply_correction: old_config.apply_correction,
            });
        }
        
        input.start().map_err(|e| format!("Failed to start stream: {}", e))?;
        
        self.audio_input = Some(input);
        
        // Start processing thread (hot loop stays in Rust!)
        self.running.store(true, Ordering::SeqCst);
        
        let filter = Arc::clone(&self.filter);
        let analyzer = Arc::clone(&self.analyzer);
        let results = Arc::clone(&self.results);
        let running = Arc::clone(&self.running);
        let bypass = Arc::clone(&self.bypass);
        let sample_rate = self.sample_rate;
        
        let handle = std::thread::spawn(move || {
            let mut temp_buffer = vec![0.0; 2048];
            let mut waveform_buffer = vec![0.0; 2048];
            let mut consumer = consumer;
            
            while running.load(Ordering::SeqCst) {
                // Read audio samples (blocks if not available)
                let n = consumer.read(&mut temp_buffer);
                
                if n > 0 {
                    // Store input waveform
                    waveform_buffer[..n].copy_from_slice(&temp_buffer[..n]);
                    
                    // Apply filter (if not bypassed)
                    let filtered = if bypass.load(Ordering::SeqCst) {
                        waveform_buffer[..n].to_vec()
                    } else {
                        if let Ok(mut filter_guard) = filter.lock() {
                            if let Some(filt) = filter_guard.as_mut() {
                                filt.process_block(&waveform_buffer[..n])
                            } else {
                                waveform_buffer[..n].to_vec()
                            }
                        } else {
                            waveform_buffer[..n].to_vec()
                        }
                    };
                    
                    // Analyze spectrum
                    let (spectrum_mag, spectrum_freq) = if let Ok(mut analyzer) = analyzer.lock() {
                        let mag = analyzer.analyze_db(&filtered, 1.0);
                        let freq = analyzer.frequency_bins_hz();
                        (mag, freq)
                    } else {
                        (vec![], vec![])
                    };
                    
                    // Store results for Python to read
                    if let Ok(mut results_guard) = results.lock() {
                        *results_guard = Some(ProcessingResults {
                            input_waveform: waveform_buffer[..n].to_vec(),
                            filtered_waveform: filtered,
                            spectrum_magnitude: spectrum_mag,
                            spectrum_frequencies: spectrum_freq,
                            sample_rate,
                        });
                    }
                } else {
                    // No data available, yield to avoid busy-wait
                    std::thread::yield_now();
                }
            }
        });
        
        self.process_thread = Some(handle);
        
        Ok(device_name)
    }
    
    /// Stop audio capture
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        
        if let Some(handle) = self.process_thread.take() {
            let _ = handle.join();
        }
        
        if let Some(input) = &self.audio_input {
            let _ = input.pause();
        }
        
        self.audio_input = None;
    }
    
    /// Design and update filter
    pub fn design_filter(
        &mut self,
        omega_c1: f64,
        omega_c2: f64,
        delta_omega: f64,
        window_type: WindowType,
    ) -> Result<(usize, f64), String> {
        let spec = FilterSpec::bandpass(omega_c1, omega_c2, delta_omega, window_type);
        let coeffs = design_bandpass_fir(&spec);
        
        let filter_length = coeffs.len();
        let group_delay = (filter_length - 1) as f64 / 2.0;
        
        // Choose implementation based on filter length
        let new_filter: Box<dyn FilterTrait + Send> = if filter_length > 128 {
            // Use FFT-based convolution for long filters
            Box::new(FastFirFilter::new(coeffs, 2048))
        } else {
            // Use direct convolution for short filters
            Box::new(FirFilter::new(coeffs))
        };
        
        if let Ok(mut filter_guard) = self.filter.lock() {
            *filter_guard = Some(new_filter);
        }
        
        Ok((filter_length, group_delay))
    }
    
    /// Set bypass state
    pub fn set_bypass(&self, bypass: bool) {
        self.bypass.store(bypass, Ordering::SeqCst);
    }
    
    /// Update FFT configuration
    pub fn update_fft_config(&self, fft_size: usize, window_type: WindowType) {
        if let Ok(mut analyzer) = self.analyzer.lock() {
            analyzer.update_config(AnalyzerConfig {
                fft_size,
                window_type,
                sample_rate: self.sample_rate,
                apply_correction: true,
            });
        }
    }
    
    /// Get latest processing results (called from Python at 60 Hz)
    pub fn get_results(&self) -> Option<ProcessingResults> {
        if let Ok(mut results_guard) = self.results.lock() {
            results_guard.take()
        } else {
            None
        }
    }
    
    /// List available audio devices
    pub fn list_devices() -> Result<Vec<String>, String> {
        list_input_devices()
            .map(|devices| devices.iter().map(|d| d.name.clone()).collect())
            .map_err(|e| format!("Failed to list devices: {}", e))
    }
}

impl Drop for AudioProcessor {
    fn drop(&mut self) {
        self.stop();
    }
}
