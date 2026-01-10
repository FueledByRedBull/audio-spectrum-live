"""
Waveform plot (time domain)
"""

import pyqtgraph as pg
from PyQt6.QtWidgets import QWidget, QVBoxLayout
import numpy as np


class WaveformPlot(QWidget):
    """Time-domain waveform display"""
    
    def __init__(self):
        super().__init__()
        
        self._setup_ui()
        
    def _setup_ui(self):
        """Setup the plot widget"""
        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        
        # Create plot widget
        self.plot_widget = pg.PlotWidget()
        self.plot_widget.setBackground('w')
        self.plot_widget.setLabel('left', 'Amplitude')
        self.plot_widget.setLabel('bottom', 'Time', units='s')
        self.plot_widget.setTitle('Audio Waveform (Time Domain)')
        self.plot_widget.showGrid(x=True, y=True, alpha=0.3)
        
        # Create plot curves
        self.input_curve = self.plot_widget.plot(
            pen=pg.mkPen(color='b', width=1),
            name='Input'
        )
        self.filtered_curve = self.plot_widget.plot(
            pen=pg.mkPen(color='r', width=1),
            name='Filtered'
        )
        
        # Add legend
        self.plot_widget.addLegend()
        
        layout.addWidget(self.plot_widget)
        
    def update_plot(self, data):
        """
        Update waveform display
        
        Args:
            data: Dictionary with 'time', 'input', 'filtered' keys
        """
        if 'time' in data and 'input' in data:
            self.input_curve.setData(data['time'], data['input'])
            
        if 'time' in data and 'filtered' in data:
            self.filtered_curve.setData(data['time'], data['filtered'])
            
    def reset_view(self):
        """Reset view to auto-range"""
        self.plot_widget.enableAutoRange()
