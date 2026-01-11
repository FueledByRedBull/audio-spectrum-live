//! Unified audio processor - keeps hot loop in Rust
//! 
//! Eliminates Python/Rust boundary overhead by processing audio entirely in Rust

use crate::filters::{FirFilter, FastFirFilter, FilterSpec, WindowType, design_bandpass_fir, design_lowpass_fir, design_highpass_fir};
use crate::spectrum::{SpectrumAnalyzer, analysis::AnalyzerConfig};
use crate::audio::{AudioInput, AudioOutput, AudioRingBuffer, input::list_input_devices};
use crate::audio::buffer::AudioProducer;
use crate::audio::gate::NoiseGate;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

/// Filter type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterType {
    Bandpass,
    Lowpass,
    Highpass,
}

/// Maximum buffer sizes for pre-allocated arrays
pub const MAX_WAVEFORM_SIZE: usize = 4096;
pub const MAX_SPECTRUM_SIZE: usize = 4097; // FFT_SIZE/2 + 1 for largest FFT (8192)

/// Results from audio processing (sent to Python)
/// Uses fixed-size arrays to eliminate hot-path allocations
#[derive(Clone)]
pub struct ProcessingResults {
    /// Input waveform samples (fixed-size buffer)
    pub input_waveform: Box<[f64; MAX_WAVEFORM_SIZE]>,

    /// Filtered waveform samples (fixed-size buffer)
    pub filtered_waveform: Box<[f64; MAX_WAVEFORM_SIZE]>,

    /// Spectrum magnitude in dB (fixed-size buffer)
    pub spectrum_magnitude: Box<[f64; MAX_SPECTRUM_SIZE]>,

    /// Frequency bins in Hz (fixed-size buffer)
    pub spectrum_frequencies: Box<[f64; MAX_SPECTRUM_SIZE]>,

    /// Actual length of waveform data
    pub waveform_len: usize,

    /// Actual length of spectrum data
    pub spectrum_len: usize,

    /// Sample rate
    pub sample_rate: f64,
}

impl Default for ProcessingResults {
    fn default() -> Self {
        Self {
            input_waveform: Box::new([0.0; MAX_WAVEFORM_SIZE]),
            filtered_waveform: Box::new([0.0; MAX_WAVEFORM_SIZE]),
            spectrum_magnitude: Box::new([0.0; MAX_SPECTRUM_SIZE]),
            spectrum_frequencies: Box::new([0.0; MAX_SPECTRUM_SIZE]),
            waveform_len: 0,
            spectrum_len: 0,
            sample_rate: 48000.0,
        }
    }
}

/// High-performance audio processor
/// 
/// Runs audio capture, filtering, and FFT analysis in Rust thread
/// Python only reads results (no per-sample boundary crossing)
pub struct AudioProcessor {
    /// Filter chain (applied in order: noise gate → user filter)
    filter_chain: Arc<Mutex<Vec<Option<Box<dyn FilterTrait + Send>>>>>,
    
    /// Noise gate enabled flag
    gate_enabled: Arc<AtomicBool>,
    
    /// Noise gate parameters (threshold_db, attack_ms, release_ms)
    gate_params: Arc<Mutex<(f64, f64, f64)>>,
    
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

/// Trait for polymorphic filter types with zero-allocation in-place processing
trait FilterTrait {
    /// Process block in-place (zero allocations)
    fn process_block_inplace(&mut self, buffer: &mut [f64]);
    #[allow(dead_code)]
    fn reset(&mut self);
}

impl FilterTrait for FirFilter {
    fn process_block_inplace(&mut self, buffer: &mut [f64]) {
        FirFilter::process_block_inplace(self, buffer)
    }

    fn reset(&mut self) {
        FirFilter::reset(self)
    }
}

impl FilterTrait for FastFirFilter {
    fn process_block_inplace(&mut self, buffer: &mut [f64]) {
        FastFirFilter::process_block_inplace(self, buffer)
    }

    fn reset(&mut self) {
        FastFirFilter::reset(self)
    }
}

impl FilterTrait for NoiseGate {
    fn process_block_inplace(&mut self, buffer: &mut [f64]) {
        NoiseGate::process_block_inplace(self, buffer)
    }

    fn reset(&mut self) {
        NoiseGate::reset(self)
    }
}

impl AudioProcessor {
    /// Create new audio processor
    pub fn new() -> Self {
        let sample_rate = 48000.0;
        
        let analyzer_config = AnalyzerConfig {
            fft_size: 4096,  // Larger FFT for better frequency resolution
            window_type: WindowType::Hamming,
            sample_rate,
            apply_correction: true,
        };
        
        Self {
            filter_chain: Arc::new(Mutex::new(vec![None, None])),  // [0] = gate, [1] = user filter
            gate_enabled: Arc::new(AtomicBool::new(false)),
            gate_params: Arc::new(Mutex::new((-40.0, 10.0, 100.0))),  // Default: -40dB, 10ms attack, 100ms release
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
        
        let filter_chain = Arc::clone(&self.filter_chain);

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
            let mut filtered_buffer = vec![0.0; MAX_WAVEFORM_SIZE];
            let mut padded_signal = vec![0.0; 8192]; // Max FFT size
            let mut consumer = consumer;

            // Pre-allocate results buffer (reused each frame - no hot-path allocations)
            let mut result_buffer = ProcessingResults::default();

            while running.load(Ordering::SeqCst) {
                // Read audio samples (blocks if not available)
                let n = consumer.read(&mut temp_buffer);

                if n > 0 {
                    let n = n.min(MAX_WAVEFORM_SIZE); // Clamp to max size
                    // Store input waveform
                    waveform_buffer[..n].copy_from_slice(&temp_buffer[..n]);

                    // Apply filter chain (if not bypassed) - ZERO ALLOCATIONS
                    // Copy input to filtered_buffer, then process in-place
                    filtered_buffer[..n].copy_from_slice(&waveform_buffer[..n]);

                    let filtered_len = if bypass.load(Ordering::SeqCst) {
                        n
                    } else {
                        // Process through filter chain IN-PLACE (no allocations)
                        if let Ok(mut chain_guard) = filter_chain.lock() {
                            for filter_opt in chain_guard.iter_mut() {
                                if let Some(filter) = filter_opt {
                                    filter.process_block_inplace(&mut filtered_buffer[..n]);
                                }
                            }
                        }
                        n
                    };

                    // Analyze spectrum (use fixed-size buffer for consistent output)
                    let spectrum_len = if let Ok(mut analyzer) = analyzer.lock() {
                        let fft_size = analyzer.config().fft_size;
                        let copy_len = filtered_len.min(fft_size);

                        // Clear and fill padded signal buffer
                        padded_signal[..fft_size].fill(0.0);
                        padded_signal[..copy_len].copy_from_slice(&filtered_buffer[..copy_len]);

                        let mag = analyzer.analyze_db(&padded_signal[..fft_size], 1.0);
                        let freq = analyzer.frequency_bins_hz();

                        let spec_len = mag.len().min(MAX_SPECTRUM_SIZE);
                        result_buffer.spectrum_magnitude[..spec_len].copy_from_slice(&mag[..spec_len]);
                        result_buffer.spectrum_frequencies[..spec_len].copy_from_slice(&freq[..spec_len]);
                        spec_len
                    } else {
                        0
                    };

                    // Copy waveform data to result buffer
                    result_buffer.input_waveform[..n].copy_from_slice(&waveform_buffer[..n]);
                    result_buffer.filtered_waveform[..filtered_len].copy_from_slice(&filtered_buffer[..filtered_len]);
                    result_buffer.waveform_len = n;
                    result_buffer.spectrum_len = spectrum_len;
                    result_buffer.sample_rate = sample_rate;

                    // Store results for Python to read (clone the pre-allocated buffer)
                    if let Ok(mut results_guard) = results.lock() {
                        *results_guard = Some(result_buffer.clone());
                    }

                    // Send filtered audio to output if monitoring is enabled
                    if monitoring.load(Ordering::SeqCst) {
                        if let Ok(mut producer_guard) = output_producer.lock() {
                            if let Some(producer) = producer_guard.as_mut() {
                                producer.write(&filtered_buffer[..filtered_len]);
                            }
                        }
                    }
                } else {
                    // No data available, sleep briefly to avoid busy-wait CPU burn
                    // 100µs is short enough to maintain low latency but prevents spin-wait
                    std::thread::sleep(std::time::Duration::from_micros(100));
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
        
        // Update filter chain: position 1 is user filter (after gate)
        if let Ok(mut chain_guard) = self.filter_chain.lock() {
            chain_guard[1] = Some(new_filter);
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
    

    
    /// Configure noise gate
    ///
    /// # Arguments
    /// * `enabled` - Enable or disable the noise gate
    /// * `threshold_db` - Threshold in dB (e.g., -40.0)
    /// * `attack_ms` - Attack time in milliseconds (e.g., 10.0)
    /// * `release_ms` - Release time in milliseconds (e.g., 100.0)
    pub fn configure_noise_gate(&mut self, enabled: bool, threshold_db: f64, attack_ms: f64, release_ms: f64) {
        self.gate_enabled.store(enabled, Ordering::SeqCst);
        
        // Store parameters
        if let Ok(mut params) = self.gate_params.lock() {
            *params = (threshold_db, attack_ms, release_ms);
        }
        
        if enabled {
            // Create and install noise gate at position 0
            let gate: Box<dyn FilterTrait + Send> = Box::new(NoiseGate::new(
                threshold_db,
                attack_ms,
                release_ms,
                self.sample_rate,
            ));
            
            if let Ok(mut chain_guard) = self.filter_chain.lock() {
                chain_guard[0] = Some(gate);
            }
        } else {
            // Remove gate (position 0)
            if let Ok(mut chain_guard) = self.filter_chain.lock() {
                chain_guard[0] = None;
            }
        }
    }
    
    /// Check if noise gate is enabled
    pub fn is_gate_enabled(&self) -> bool {
        self.gate_enabled.load(Ordering::SeqCst)
    }
}

impl Drop for AudioProcessor {
    fn drop(&mut self) {
        self.stop();
    }
}
