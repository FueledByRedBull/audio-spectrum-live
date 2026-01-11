//! Unified audio processor - keeps hot loop in Rust
//! 
//! Eliminates Python/Rust boundary overhead by processing audio entirely in Rust

use crate::filters::{FirFilter, FastFirFilter, FilterSpec, WindowType, design_bandpass_fir, design_lowpass_fir, design_highpass_fir};
use crate::spectrum::{SpectrumAnalyzer, analysis::AnalyzerConfig};
use crate::audio::{AudioInput, AudioOutput, AudioRingBuffer, input::list_input_devices};
use crate::audio::buffer::AudioProducer;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

/// Filter type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterType {
    Bandpass,
    Lowpass,
    Highpass,
}

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
    
    /// Audio output stream (for monitoring)
    audio_output: Option<AudioOutput>,
    
    /// Output ring buffer producer (for sending audio to output)
    output_producer: Arc<Mutex<Option<AudioProducer>>>,
    
    /// Processing thread handle
    process_thread: Option<std::thread::JoinHandle<()>>,
    
    /// Running flag
    running: Arc<AtomicBool>,
    
    /// Bypass flag
    bypass: Arc<AtomicBool>,
    
    /// Monitoring enabled flag
    monitoring: Arc<AtomicBool>,
    
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
            audio_output: None,
            output_producer: Arc::new(Mutex::new(None)),
            process_thread: None,
            running: Arc::new(AtomicBool::new(false)),
            bypass: Arc::new(AtomicBool::new(false)),
            monitoring: Arc::new(AtomicBool::new(false)),
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
        let monitoring = Arc::clone(&self.monitoring);
        let output_producer = Arc::clone(&self.output_producer);
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
                    
                    // Analyze spectrum (use fixed-size buffer for consistent output)
                    let (spectrum_mag, spectrum_freq) = if let Ok(mut analyzer) = analyzer.lock() {
                        // Always use full FFT size to get consistent spectrum length
                        let fft_size = analyzer.config().fft_size;
                        let mut padded_signal = vec![0.0; fft_size];
                        let copy_len = filtered.len().min(fft_size);
                        padded_signal[..copy_len].copy_from_slice(&filtered[..copy_len]);
                        
                        let mag = analyzer.analyze_db(&padded_signal, 1.0);
                        let freq = analyzer.frequency_bins_hz();
                        (mag, freq)
                    } else {
                        (vec![], vec![])
                    };
                    
                    // Store results for Python to read
                    if let Ok(mut results_guard) = results.lock() {
                        *results_guard = Some(ProcessingResults {
                            input_waveform: waveform_buffer[..n].to_vec(),
                            filtered_waveform: filtered.clone(),
                            spectrum_magnitude: spectrum_mag,
                            spectrum_frequencies: spectrum_freq,
                            sample_rate,
                        });
                    }
                    
                    // Send filtered audio to output if monitoring is enabled
                    if monitoring.load(Ordering::SeqCst) {
                        if let Ok(mut producer_guard) = output_producer.lock() {
                            if let Some(producer) = producer_guard.as_mut() {
                                producer.write(&filtered);
                            }
                        }
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
        filter_type: FilterType,
    ) -> Result<(usize, f64), String> {
        // Design filter coefficients based on type
        let coeffs = match filter_type {
            FilterType::Bandpass => {
                let spec = FilterSpec::bandpass(omega_c1, omega_c2, delta_omega, window_type);
                design_bandpass_fir(&spec)
            },
            FilterType::Lowpass => {
                design_lowpass_fir(omega_c2, delta_omega, window_type)
            },
            FilterType::Highpass => {
                design_highpass_fir(omega_c1, delta_omega, window_type)
            },
        };
        
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
    
    /// Enable audio monitoring (output filtered audio to speakers/headphones)
    /// 
    /// WARNING: Use headphones to avoid feedback loop!
    pub fn enable_monitoring(&mut self) -> Result<(), String> {
        if self.audio_output.is_some() {
            self.monitoring.store(true, Ordering::SeqCst);
            return Ok(());
        }
        
        // Create output ring buffer
        let rb = AudioRingBuffer::new(96000);
        let (producer, consumer) = rb.split();
        
        // Store producer for processing thread
        if let Ok(mut producer_guard) = self.output_producer.lock() {
            *producer_guard = Some(producer);
        }
        
        // Start audio output
        let output = AudioOutput::from_default_device(consumer)
            .map_err(|e| format!("Failed to start audio output: {}", e))?;
        
        output.start().map_err(|e| format!("Failed to start output stream: {}", e))?;
        
        self.audio_output = Some(output);
        self.monitoring.store(true, Ordering::SeqCst);
        
        Ok(())
    }
    
    /// Disable audio monitoring
    pub fn disable_monitoring(&mut self) {
        self.monitoring.store(false, Ordering::SeqCst);
        
        if let Some(output) = &self.audio_output {
            let _ = output.pause();
        }
        
        self.audio_output = None;
        
        // Clear output producer
        if let Ok(mut producer_guard) = self.output_producer.lock() {
            *producer_guard = None;
        }
    }
    
    /// Check if monitoring is enabled
    pub fn is_monitoring(&self) -> bool {
        self.monitoring.load(Ordering::SeqCst)
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
