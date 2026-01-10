"""
Spectrum plot (frequency domain)
"""

import pyqtgraph as pg
from PyQt6.QtWidgets import QWidget, QVBoxLayout, QTabWidget
import numpy as np


class SpectrumPlot(QWidget):
    """Frequency-domain spectrum display with waterfall"""
    
    def __init__(self):
        super().__init__()
        
        self.waterfall_history = []
        self.max_waterfall_lines = 200
        
        self._setup_ui()
        
    def _setup_ui(self):
        """Setup the plot widgets"""
        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        
        # Create tab widget for different views
        self.tabs = QTabWidget()
        
        # Magnitude spectrum plot
        self.magnitude_widget = pg.PlotWidget()
        self.magnitude_widget.setBackground('w')
        self.magnitude_widget.setLabel('left', 'Magnitude', units='dB')
        self.magnitude_widget.setLabel('bottom', 'Frequency', units='Hz')
        self.magnitude_widget.setTitle('Magnitude Spectrum')
        self.magnitude_widget.showGrid(x=True, y=True, alpha=0.3)
        
        self.magnitude_curve = self.magnitude_widget.plot(
            pen=pg.mkPen(color='b', width=2)
        )
        
        self.tabs.addTab(self.magnitude_widget, "Magnitude Spectrum")
        
        # Waterfall spectrogram
        self.waterfall_widget = pg.PlotWidget()
        self.waterfall_widget.setBackground('k')
        self.waterfall_widget.setLabel('left', 'Time')
        self.waterfall_widget.setLabel('bottom', 'Frequency', units='Hz')
        self.waterfall_widget.setTitle('Spectrogram (Waterfall)')
        
        # Create image item for waterfall
        self.waterfall_image = pg.ImageItem()
        self.waterfall_widget.addItem(self.waterfall_image)
        
        # Setup colormap (hot)
        colors = [
            (0, 0, 0),      # black
            (128, 0, 128),  # purple
            (255, 0, 0),    # red
            (255, 255, 0),  # yellow
            (255, 255, 255) # white
        ]
        cmap = pg.ColorMap(pos=np.linspace(0, 1, len(colors)), color=colors)
        self.waterfall_image.setLookupTable(cmap.getLookupTable())
        
        self.tabs.addTab(self.waterfall_widget, "Waterfall (Spectrogram)")
        
        layout.addWidget(self.tabs)
        
    def update_plot(self, frequencies, magnitude):
        """
        Update spectrum display
        
        Args:
            frequencies: Frequency bins in Hz
            magnitude: Magnitude spectrum in dB
        """
        # Update magnitude plot
        self.magnitude_curve.setData(frequencies, magnitude)
        
        # Update waterfall
        self.waterfall_history.append(magnitude.copy())
        
        # Keep only recent history
        if len(self.waterfall_history) > self.max_waterfall_lines:
            self.waterfall_history.pop(0)
            
        # Create waterfall image
        if len(self.waterfall_history) > 1:
            waterfall_data = np.array(self.waterfall_history)
            
            # Normalize for display
            vmin = np.percentile(waterfall_data, 5)
            vmax = np.percentile(waterfall_data, 95)
            waterfall_data = np.clip((waterfall_data - vmin) / (vmax - vmin + 1e-10), 0, 1)
            
            self.waterfall_image.setImage(
                waterfall_data.T,
                autoLevels=False,
                levels=[0, 1]
            )
            
            # Set correct scaling
            self.waterfall_image.setRect(
                0, 0,
                frequencies[-1], len(self.waterfall_history)
            )
            
    def reset_view(self):
        """Reset view to auto-range"""
        self.magnitude_widget.enableAutoRange()
        self.waterfall_widget.enableAutoRange()
        
    def clear_waterfall(self):
        """Clear waterfall history"""
        self.waterfall_history.clear()
