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
        window_type: str
    ) -> Dict:
        """
        Design a new FIR filter
        
        Args:
            omega_c1: Lower cutoff frequency (normalized, units of π)
            omega_c2: Upper cutoff frequency (normalized, units of π)
            delta_omega: Transition width (radians)
            window_type: Window type name
            
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
            
            window_enum = window_map.get(window_type, WindowType.Hamming)
            
            # Design and apply filter in Rust
            filter_length, group_delay = self.processor.design_filter(
                omega_c1, omega_c2, delta_omega, window_enum
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
        
        # Update with current FFT size (default 2048)
        self.processor.update_fft_config(2048, window_enum)
        
    def get_waveform_data(self) -> Optional[Dict]:
        """
        Get latest waveform data
        
        Returns:
            Dictionary with 'input' and 'filtered' numpy arrays, or None
        """
        if not self.processor:
            return None
            
        try:
            results = self.processor.get_results()
            
            if results is None:
                return None
                
            return {
                'input': results['input_waveform'],
                'filtered': results['filtered_waveform'],
                'sample_rate': results['sample_rate'],
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
                
            return {
                'frequencies': results['spectrum_frequencies'],
                'magnitude': results['spectrum_magnitude'],
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
