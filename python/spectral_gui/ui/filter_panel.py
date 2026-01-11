"""
Filter control panel with spinbox controls and filter type selection
"""

from PyQt6.QtWidgets import (
    QWidget,
    QVBoxLayout,
    QHBoxLayout,
    QLabel,
    QDoubleSpinBox,
    QComboBox,
    QPushButton,
    QGroupBox,
    QFormLayout,
    QMessageBox,
    QCheckBox,
)
from PyQt6.QtCore import Qt

# Filter presets: (omega_c1, omega_c2, delta_omega)
# Normalized frequency = Hz / 24000 (Nyquist at 48 kHz)
FILTER_PRESETS = {
    'Voice (300-3400 Hz)': (0.0125, 0.1417, 0.0021),  # 300/24k, 3400/24k, 50 Hz transition
    'Music Bass (20-250 Hz)': (0.0008, 0.0104, 0.0021),  # 20/24k, 250/24k, 50 Hz transition
    'Music Treble (4-16 kHz)': (0.1667, 0.6667, 0.0208),  # 4000/24k, 16000/24k, 500 Hz transition
    'Narrow Notch (900-1100 Hz)': (0.0375, 0.0458, 0.0021),  # 900/24k, 1100/24k, 50 Hz transition
    'Wide Band (100-10 kHz)': (0.0042, 0.4167, 0.0083),  # 100/24k, 10000/24k, 200 Hz transition
    'Part A Spec (0.4π-0.6π)': (0.400, 0.600, 0.050),  # 9.6-14.4 kHz (unchanged - already normalized)
    'Clear Voice Ice (1.5-4 kHz)': (0.0625, 0.1667, 0.0083),  # 1500/24k, 4000/24k, 200 Hz transition
    'Bassy Clear Voice (80-3 kHz)': (0.0033, 0.1250, 0.0083),  # 80/24k, 3000/24k, 200 Hz transition
}


class FilterPanel(QWidget):
    """Filter parameter control panel with spinbox controls"""
    
    def __init__(self, dsp_controller):
        super().__init__()
        self.dsp_controller = dsp_controller
        
        self._setup_ui()
        self._connect_signals()
        
    def _setup_ui(self):
        """Setup the UI components"""
        layout = QVBoxLayout(self)
        
        # Preset Selector (at top for discoverability)
        preset_group = QGroupBox("Filter Presets")
        preset_layout = QHBoxLayout()
        self.preset_combo = QComboBox()
        self.preset_combo.addItem("-- Select Preset --")
        for preset_name in FILTER_PRESETS.keys():
            self.preset_combo.addItem(preset_name)
        preset_layout.addWidget(QLabel("Load Preset:"))
        preset_layout.addWidget(self.preset_combo)
        preset_group.setLayout(preset_layout)
        layout.addWidget(preset_group)
        
        # Filter Design Group
        filter_group = QGroupBox("Filter Design")
        filter_layout = QFormLayout()
        
        # Filter type selection
        self.filter_type_combo = QComboBox()
        self.filter_type_combo.addItems(["Bandpass", "Lowpass", "Highpass"])
        self.filter_type_combo.setCurrentIndex(0)  # Default to Bandpass
        filter_layout.addRow("Filter Type:", self.filter_type_combo)
        
        # Window type selection
        self.window_combo = QComboBox()
        self.window_combo.addItems(["Hann", "Hamming", "Blackman", "Rectangular"])
        self.window_combo.setCurrentIndex(1)  # Default to Hamming
        filter_layout.addRow("Window Type:", self.window_combo)
        
        # Lower cutoff frequency (ωc1) - SpinBox
        self.cutoff1_label = QLabel("ω = 0.370π (8.88 kHz)")
        self.cutoff1_spinbox = QDoubleSpinBox()
        self.cutoff1_spinbox.setRange(0.000, 1.000)
        self.cutoff1_spinbox.setSingleStep(0.001)
        self.cutoff1_spinbox.setDecimals(3)
        self.cutoff1_spinbox.setValue(0.370)
        self.cutoff1_spinbox.setKeyboardTracking(False)
        self.cutoff1_spinbox.setMinimumWidth(100)
        
        cutoff1_layout = QVBoxLayout()
        cutoff1_layout.addWidget(self.cutoff1_label)
        cutoff1_layout.addWidget(self.cutoff1_spinbox)
        self.cutoff1_row_label = QLabel("Lower Cutoff (ωc1):")
        filter_layout.addRow(self.cutoff1_row_label, cutoff1_layout)
        
        # Upper cutoff frequency (ωc2) - SpinBox
        self.cutoff2_label = QLabel("ω = 0.620π (14.88 kHz)")
        self.cutoff2_spinbox = QDoubleSpinBox()
        self.cutoff2_spinbox.setRange(0.000, 1.000)
        self.cutoff2_spinbox.setSingleStep(0.001)
        self.cutoff2_spinbox.setDecimals(3)
        self.cutoff2_spinbox.setValue(0.620)
        self.cutoff2_spinbox.setKeyboardTracking(False)
        self.cutoff2_spinbox.setMinimumWidth(100)
        
        cutoff2_layout = QVBoxLayout()
        cutoff2_layout.addWidget(self.cutoff2_label)
        cutoff2_layout.addWidget(self.cutoff2_spinbox)
        self.cutoff2_row_label = QLabel("Upper Cutoff (ωc2):")
        filter_layout.addRow(self.cutoff2_row_label, cutoff2_layout)
        
        # Transition width - SpinBox
        self.transition_label = QLabel("ω = 0.050π (1.20 kHz)")
        self.transition_spinbox = QDoubleSpinBox()
        self.transition_spinbox.setRange(0.001, 0.200)
        self.transition_spinbox.setSingleStep(0.001)
        self.transition_spinbox.setDecimals(3)
        self.transition_spinbox.setValue(0.050)
        self.transition_spinbox.setKeyboardTracking(False)
        self.transition_spinbox.setMinimumWidth(100)
        
        transition_layout = QVBoxLayout()
        transition_layout.addWidget(self.transition_label)
        transition_layout.addWidget(self.transition_spinbox)
        filter_layout.addRow("Transition Width (Δω):", transition_layout)
        
        # Filter info display
        self.filter_length_label = QLabel("Length: 161")
        self.filter_delay_label = QLabel("Group Delay: 80 samples")
        filter_layout.addRow("Filter Info:", QLabel(""))
        filter_layout.addRow("", self.filter_length_label)
        filter_layout.addRow("", self.filter_delay_label)
        
        filter_group.setLayout(filter_layout)
        layout.addWidget(filter_group)
        
        # FFT Analysis Group
        fft_group = QGroupBox("FFT Analysis")
        fft_layout = QFormLayout()
        
        # FFT size
        self.fft_size_combo = QComboBox()
        self.fft_size_combo.addItems(["512", "1024", "2048", "4096", "8192"])
        self.fft_size_combo.setCurrentIndex(2)  # Default to 2048
        fft_layout.addRow("FFT Size:", self.fft_size_combo)
        
        # Window type for analysis
        self.analysis_window_combo = QComboBox()
        self.analysis_window_combo.addItems(["Hann", "Hamming", "Blackman", "Rectangular"])
        self.analysis_window_combo.setCurrentIndex(1)  # Default to Hamming
        fft_layout.addRow("Analysis Window:", self.analysis_window_combo)
        
        fft_group.setLayout(fft_layout)
        layout.addWidget(fft_group)
        
        # Control Buttons
        button_layout = QVBoxLayout()
        
        self.apply_button = QPushButton("Apply Filter")
        self.apply_button.setStyleSheet("QPushButton { background-color: #4CAF50; color: white; padding: 8px; font-weight: bold; }")
        button_layout.addWidget(self.apply_button)
        
        self.reset_button = QPushButton("Reset to Part A")
        button_layout.addWidget(self.reset_button)
        
        self.bypass_button = QPushButton("Bypass Filter")
        self.bypass_button.setCheckable(True)
        button_layout.addWidget(self.bypass_button)
        
        self.monitor_checkbox = QCheckBox("Monitor Output (⚠ Use Headphones!)")
        self.monitor_checkbox.setToolTip("Enable to hear filtered audio.\nWARNING: Use headphones to avoid feedback!")
        button_layout.addWidget(self.monitor_checkbox)
        
        layout.addLayout(button_layout)
        
        # Add stretch to push everything to top
        layout.addStretch()
        
    def _connect_signals(self):
        """Connect signals to slots"""
        # Preset selector
        self.preset_combo.currentTextChanged.connect(self._load_preset)
        
        # Filter type
        self.filter_type_combo.currentTextChanged.connect(self._on_filter_type_changed)
        
        # SpinBoxes
        self.cutoff1_spinbox.valueChanged.connect(self._update_cutoff1_label)
        self.cutoff2_spinbox.valueChanged.connect(self._update_cutoff2_label)
        self.transition_spinbox.valueChanged.connect(self._update_transition_label)
        
        # Buttons
        self.apply_button.clicked.connect(self._apply_filter)
        self.reset_button.clicked.connect(self._reset_to_part_a)
        self.bypass_button.toggled.connect(self._toggle_bypass)
        self.monitor_checkbox.toggled.connect(self._toggle_monitoring)
        
        # Combo boxes
        self.fft_size_combo.currentTextChanged.connect(self._update_fft_size)
        self.analysis_window_combo.currentTextChanged.connect(self._update_analysis_window)
        
    def _load_preset(self, preset_name):
        """Load filter parameters from preset"""
        if preset_name in FILTER_PRESETS:
            omega_c1, omega_c2, delta_omega = FILTER_PRESETS[preset_name]
            self.cutoff1_spinbox.setValue(omega_c1)
            self.cutoff2_spinbox.setValue(omega_c2)
            self.transition_spinbox.setValue(delta_omega)
            # Note: valueChanged signals will automatically update labels
            
    def _on_filter_type_changed(self, filter_type):
        """Handle filter type change - show/hide cutoff spinboxes"""
        if filter_type == "Lowpass":
            # Hide lower cutoff
            self.cutoff1_spinbox.setVisible(False)
            self.cutoff1_label.setVisible(False)
            self.cutoff1_row_label.setText("(Not used for Lowpass)")
            
            # Show upper cutoff
            self.cutoff2_spinbox.setVisible(True)
            self.cutoff2_label.setVisible(True)
            self.cutoff2_row_label.setText("Cutoff Frequency:")
            
        elif filter_type == "Highpass":
            # Show lower cutoff
            self.cutoff1_spinbox.setVisible(True)
            self.cutoff1_label.setVisible(True)
            self.cutoff1_row_label.setText("Cutoff Frequency:")
            
            # Hide upper cutoff
            self.cutoff2_spinbox.setVisible(False)
            self.cutoff2_label.setVisible(False)
            self.cutoff2_row_label.setText("(Not used for Highpass)")
            
        else:  # Bandpass
            # Show both cutoffs
            self.cutoff1_spinbox.setVisible(True)
            self.cutoff1_label.setVisible(True)
            self.cutoff1_row_label.setText("Lower Cutoff (ωc1):")
            
            self.cutoff2_spinbox.setVisible(True)
            self.cutoff2_label.setVisible(True)
            self.cutoff2_row_label.setText("Upper Cutoff (ωc2):")
        
    def _update_cutoff1_label(self, value):
        """Update cutoff1 label with frequency in Hz"""
        freq_hz = value * 24000.0  # value * (48000 / 2)
        self.cutoff1_label.setText(f"ω = {value:.3f}π ({freq_hz:.2f} Hz)")
        
    def _update_cutoff2_label(self, value):
        """Update cutoff2 label with frequency in Hz"""
        freq_hz = value * 24000.0  # value * (48000 / 2)
        self.cutoff2_label.setText(f"ω = {value:.3f}π ({freq_hz:.2f} Hz)")
        
    def _update_transition_label(self, value):
        """Update transition width label with frequency in Hz"""
        freq_hz = value * 24000.0  # value * (48000 / 2)
        self.transition_label.setText(f"ω = {value:.3f}π ({freq_hz:.2f} Hz)")
        
    def _apply_filter(self):
        """Apply filter with current parameters"""
        try:
            omega_c1 = self.cutoff1_spinbox.value()
            omega_c2 = self.cutoff2_spinbox.value()
            delta_omega = self.transition_spinbox.value() * 3.14159  # Convert to radians
            window_type = self.window_combo.currentText()
            filter_type = self.filter_type_combo.currentText()
            
            # Design and apply filter
            filter_info = self.dsp_controller.design_filter(
                omega_c1, omega_c2, delta_omega, window_type, filter_type
            )
            
            # Update info labels
            self.filter_length_label.setText(f"Length: {filter_info['length']}")
            self.filter_delay_label.setText(f"Group Delay: {filter_info['delay']:.1f} samples")
            
        except Exception as e:
            print(f"Error applying filter: {e}")
            QMessageBox.critical(self, "Filter Error", f"Failed to apply filter:\n{e}")
            
    def _reset_to_part_a(self):
        """Reset to Part A specifications"""
        self.filter_type_combo.setCurrentText("Bandpass")
        self.cutoff1_spinbox.setValue(0.400)  # 0.4π
        self.cutoff2_spinbox.setValue(0.600)  # 0.6π
        self.transition_spinbox.setValue(0.050)  # 0.05π
        self.window_combo.setCurrentIndex(1)  # Hamming
        self._apply_filter()
        
    def _toggle_bypass(self, checked):
        """Toggle filter bypass"""
        self.dsp_controller.set_bypass(checked)
        if checked:
            self.bypass_button.setText("Enable Filter")
        else:
            self.bypass_button.setText("Bypass Filter")
    
    def _toggle_monitoring(self, checked):
        """Toggle audio monitoring with feedback warning"""
        if checked:
            # Show warning dialog
            reply = QMessageBox.warning(
                self,
                "⚠ Feedback Warning",
                "Enabling monitoring will output audio to your speakers/headphones.\n\n"
                "⚠ WARNING: If you are using laptop speakers and a built-in microphone, "
                "this WILL create a loud feedback loop!\n\n"
                "✓ Use headphones to avoid feedback.\n\n"
                "Continue?",
                QMessageBox.StandardButton.Yes | QMessageBox.StandardButton.No,
                QMessageBox.StandardButton.No
            )
            
            if reply == QMessageBox.StandardButton.Yes:
                try:
                    self.dsp_controller.enable_monitoring()
                    print("✓ Audio monitoring enabled")
                except Exception as e:
                    print(f"✗ Failed to enable monitoring: {e}")
                    QMessageBox.critical(self, "Error", f"Failed to enable monitoring:\n{e}")
                    self.monitor_checkbox.setChecked(False)
            else:
                # User cancelled
                self.monitor_checkbox.setChecked(False)
        else:
            try:
                self.dsp_controller.disable_monitoring()
                print("Audio monitoring disabled")
            except Exception as e:
                print(f"Error disabling monitoring: {e}")
            
    def _update_fft_size(self, size_str):
        """Update FFT size"""
        try:
            size = int(size_str)
            self.dsp_controller.set_fft_size(size)
        except Exception as e:
            print(f"Error updating FFT size: {e}")
            
    def _update_analysis_window(self, window_name):
        """Update analysis window type"""
        try:
            self.dsp_controller.set_window_type(window_name)
        except Exception as e:
            print(f"Error updating analysis window: {e}")
