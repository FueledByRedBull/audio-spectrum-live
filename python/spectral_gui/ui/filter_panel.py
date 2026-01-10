"""
Filter control panel
"""

from PyQt6.QtWidgets import (
    QWidget,
    QVBoxLayout,
    QHBoxLayout,
    QLabel,
    QSlider,
    QComboBox,
    QPushButton,
    QGroupBox,
    QFormLayout,
    QMessageBox,
    QCheckBox,
)
from PyQt6.QtCore import Qt


class FilterPanel(QWidget):
    """Filter parameter control panel"""
    
    def __init__(self, dsp_controller):
        super().__init__()
        self.dsp_controller = dsp_controller
        
        self._setup_ui()
        self._connect_signals()
        
    def _setup_ui(self):
        """Setup the UI components"""
        layout = QVBoxLayout(self)
        
        # Filter Design Group
        filter_group = QGroupBox("Filter Design")
        filter_layout = QFormLayout()
        
        # Window type selection
        self.window_combo = QComboBox()
        self.window_combo.addItems(["Hann", "Hamming", "Blackman", "Rectangular"])
        self.window_combo.setCurrentIndex(1)  # Default to Hamming
        filter_layout.addRow("Window Type:", self.window_combo)
        
        # Lower cutoff frequency (ωc1)
        self.cutoff1_label = QLabel("0.375π")
        self.cutoff1_slider = QSlider(Qt.Orientation.Horizontal)
        self.cutoff1_slider.setMinimum(0)
        self.cutoff1_slider.setMaximum(100)
        self.cutoff1_slider.setValue(37)  # 0.375 * 100
        self.cutoff1_slider.setTickPosition(QSlider.TickPosition.TicksBelow)
        self.cutoff1_slider.setTickInterval(10)
        
        cutoff1_layout = QVBoxLayout()
        cutoff1_layout.addWidget(self.cutoff1_label)
        cutoff1_layout.addWidget(self.cutoff1_slider)
        filter_layout.addRow("Lower Cutoff (ωc1):", cutoff1_layout)
        
        # Upper cutoff frequency (ωc2)
        self.cutoff2_label = QLabel("0.625π")
        self.cutoff2_slider = QSlider(Qt.Orientation.Horizontal)
        self.cutoff2_slider.setMinimum(0)
        self.cutoff2_slider.setMaximum(100)
        self.cutoff2_slider.setValue(62)  # 0.625 * 100
        self.cutoff2_slider.setTickPosition(QSlider.TickPosition.TicksBelow)
        self.cutoff2_slider.setTickInterval(10)
        
        cutoff2_layout = QVBoxLayout()
        cutoff2_layout.addWidget(self.cutoff2_label)
        cutoff2_layout.addWidget(self.cutoff2_slider)
        filter_layout.addRow("Upper Cutoff (ωc2):", cutoff2_layout)
        
        # Transition width
        self.transition_label = QLabel("0.05π")
        self.transition_slider = QSlider(Qt.Orientation.Horizontal)
        self.transition_slider.setMinimum(1)
        self.transition_slider.setMaximum(20)
        self.transition_slider.setValue(5)  # 0.05 * 100
        self.transition_slider.setTickPosition(QSlider.TickPosition.TicksBelow)
        self.transition_slider.setTickInterval(5)
        
        transition_layout = QVBoxLayout()
        transition_layout.addWidget(self.transition_label)
        transition_layout.addWidget(self.transition_slider)
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
        # Sliders
        self.cutoff1_slider.valueChanged.connect(self._update_cutoff1_label)
        self.cutoff2_slider.valueChanged.connect(self._update_cutoff2_label)
        self.transition_slider.valueChanged.connect(self._update_transition_label)
        
        # Buttons
        self.apply_button.clicked.connect(self._apply_filter)
        self.reset_button.clicked.connect(self._reset_to_part_a)
        self.bypass_button.toggled.connect(self._toggle_bypass)
        self.monitor_checkbox.toggled.connect(self._toggle_monitoring)
        
        # Combo boxes
        self.fft_size_combo.currentTextChanged.connect(self._update_fft_size)
        self.analysis_window_combo.currentTextChanged.connect(self._update_analysis_window)
        
    def _update_cutoff1_label(self, value):
        """Update cutoff1 label"""
        freq = value / 100.0
        self.cutoff1_label.setText(f"{freq:.3f}π")
        
    def _update_cutoff2_label(self, value):
        """Update cutoff2 label"""
        freq = value / 100.0
        self.cutoff2_label.setText(f"{freq:.3f}π")
        
    def _update_transition_label(self, value):
        """Update transition width label"""
        width = value / 100.0
        self.transition_label.setText(f"{width:.3f}π")
        
    def _apply_filter(self):
        """Apply filter with current parameters"""
        try:
            omega_c1 = self.cutoff1_slider.value() / 100.0
            omega_c2 = self.cutoff2_slider.value() / 100.0
            delta_omega = self.transition_slider.value() / 100.0 * 3.14159
            window_type = self.window_combo.currentText()
            
            # Design and apply filter
            filter_info = self.dsp_controller.design_filter(
                omega_c1, omega_c2, delta_omega, window_type
            )
            
            # Update info labels
            self.filter_length_label.setText(f"Length: {filter_info['length']}")
            self.filter_delay_label.setText(f"Group Delay: {filter_info['delay']:.1f} samples")
            
        except Exception as e:
            print(f"Error applying filter: {e}")
            
    def _reset_to_part_a(self):
        """Reset to Part A specifications"""
        self.cutoff1_slider.setValue(37)  # 0.375
        self.cutoff2_slider.setValue(62)  # 0.625
        self.transition_slider.setValue(5)  # 0.05
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
            self.dsp_controller.set_analysis_window(window_name)
        except Exception as e:
            print(f"Error updating analysis window: {e}")
