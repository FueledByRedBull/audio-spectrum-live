"""
DSP Controller - Connects optimized Rust AudioProcessor to Python GUI
"""

import numpy as np
from typing import Dict, Optional

try:
    from spectral_workbench import AudioProcessor, WindowType
    RUST_AVAILABLE = True
except ImportError:
    RUST_AVAILABLE = False
    print("Warning: Rust core not available.")


class DSPController:
    """High-performance controller using unified Rust AudioProcessor"""
    
    def __init__(self):
        self.processor = None
        self.sample_rate = 48000.0
        
        if RUST_AVAILABLE:
            self._initialize_processor()
        else:
            print("ERROR: Rust core required but not available")
            
    def _initialize_processor(self):
        """Initialize unified AudioProcessor"""
        try:
            self.processor = AudioProcessor()
            print("✓ Unified AudioProcessor initialized")
        except Exception as e:
            print(f"✗ Error initializing AudioProcessor: {e}")
            
    def start_audio(self) -> Dict:
        """Start audio capture and processing"""
        if not self.processor:
            return {'name': 'ERROR', 'sample_rate': 0, 'channels': 0}
            
        try:
            device_name = self.processor.start()
            
            return {
                'name': device_name,
                'sample_rate': int(self.sample_rate),
                'channels': 1,
            }
        except Exception as e:
            print(f"Error starting audio: {e}")
            return {'name': 'ERROR', 'sample_rate': 0, 'channels': 0}
            
    def stop_audio(self):
        """Stop audio capture"""
        if self.processor:
            self.processor.stop()
            
    def design_filter(
        self,
        omega_c1: float,
        omega_c2: float,
        delta_omega: float,
        window_type: str,
        filter_type: str = "Bandpass"
    ) -> Dict:
        """
        Design a new FIR filter
        
        Args:
            omega_c1: Lower cutoff frequency (normalized, units of π)
            omega_c2: Upper cutoff frequency (normalized, units of π)
            delta_omega: Transition width (radians)
            window_type: Window type name
            filter_type: Filter type (Bandpass/Lowpass/Highpass)
            
        Returns:
            Dictionary with filter info
        """
        if not self.processor:
            return {'length': 0, 'delay': 0.0}
            
        try:
            # Map window type string to enum
            window_map = {
                'Hann': WindowType.Hann,
                'Hamming': WindowType.Hamming,
                'Blackman': WindowType.Blackman,
                'Rectangular': WindowType.Rectangular,
            }
            
            # Map filter type string to enum
            from spectral_workbench import FilterType
            filter_map = {
                'Bandpass': FilterType.Bandpass,
                'Lowpass': FilterType.Lowpass,
                'Highpass': FilterType.Highpass,
            }
            
            window_enum = window_map.get(window_type, WindowType.Hamming)
            filter_enum = filter_map.get(filter_type, FilterType.Bandpass)
            
            # Design and apply filter in Rust
            filter_length, group_delay = self.processor.design_filter(
                omega_c1, omega_c2, delta_omega, window_enum, filter_enum
            )
            
            return {
                'length': filter_length,
                'delay': group_delay,
            }
            
        except Exception as e:
            print(f"Error designing filter: {e}")
            return {'length': 0, 'delay': 0.0}
            
    def set_bypass(self, bypass: bool):
        """Set filter bypass state"""
        if self.processor:
            self.processor.set_bypass(bypass)
    
    def enable_monitoring(self):
        """Enable audio monitoring (output filtered audio)"""
        if self.processor:
            self.processor.enable_monitoring()
    
    def disable_monitoring(self):
        """Disable audio monitoring"""
        if self.processor:
            self.processor.disable_monitoring()
    
    def is_monitoring(self) -> bool:
        """Check if monitoring is enabled"""
        if self.processor:
            return self.processor.is_monitoring()
        return False
        
    def set_fft_size(self, size: int):
        """Update FFT size"""
        if self.processor:
            # Keep Hamming window by default
            self.processor.update_fft_config(size, WindowType.Hamming)
            
    def set_window_type(self, window_type: str):
        """Update FFT window type"""
        if not self.processor:
            return
            
        window_map = {
            'Hann': WindowType.Hann,
            'Hamming': WindowType.Hamming,
            'Blackman': WindowType.Blackman,
            'Rectangular': WindowType.Rectangular,
        }
        
        window_enum = window_map.get(window_type, WindowType.Hamming)
        
        # Update with current FFT size (default 4096)
        self.processor.update_fft_config(4096, window_enum)
        
    def get_waveform_data(self) -> Optional[Dict]:
        """
        Get latest waveform data
        
        Returns:
            Dictionary with 'time', 'input' and 'filtered' numpy arrays, or None
        """
        if not self.processor:
            return None
            
        try:
            results = self.processor.get_results()
            
            if results is None:
                return None
            
            # Create time axis
            input_waveform = np.array(results['input_waveform'], dtype=np.float64)
            filtered_waveform = np.array(results['filtered_waveform'], dtype=np.float64)
            sample_rate = float(results['sample_rate'])
            
            time = np.arange(len(input_waveform)) / sample_rate
                
            return {
                'time': time,
                'input': input_waveform,
                'filtered': filtered_waveform,
            }
            
        except Exception as e:
            print(f"Error getting waveform: {e}")
            return None
            
    def get_spectrum_data(self) -> Optional[Dict]:
        """
        Get latest spectrum data
        
        Returns:
            Dictionary with 'frequencies' and 'magnitude' arrays, or None
        """
        if not self.processor:
            return None
            
        try:
            results = self.processor.get_results()
            
            if results is None:
                return None
            
            # Ensure arrays are proper numpy arrays
            frequencies = np.array(results['spectrum_frequencies'], dtype=np.float64)
            magnitude = np.array(results['spectrum_magnitude'], dtype=np.float64)
                
            return {
                'frequencies': frequencies,
                'magnitude': magnitude,
            }
            
        except Exception as e:
            print(f"Error getting spectrum: {e}")
            return None
            
    def list_devices(self) -> list:
        """List available audio input devices"""
        if not RUST_AVAILABLE or not self.processor:
            return []
            
        try:
            return AudioProcessor.list_devices()
        except Exception as e:
            print(f"Error listing devices: {e}")
            return []
    

    
    def configure_noise_gate(self, enabled: bool, threshold_db: float, attack_ms: float, release_ms: float):
        """
        Configure noise gate
        
        Args:
            enabled: Enable or disable the noise gate
            threshold_db: Threshold in dB (e.g., -40.0)
            attack_ms: Attack time in milliseconds (e.g., 10.0)
            release_ms: Release time in milliseconds (e.g., 100.0)
        """
        if self.processor:
            self.processor.configure_noise_gate(enabled, threshold_db, attack_ms, release_ms)
