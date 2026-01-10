"""
Main application window
"""

from PyQt6.QtWidgets import (
    QMainWindow,
    QWidget,
    QVBoxLayout,
    QHBoxLayout,
    QSplitter,
    QStatusBar,
)
from PyQt6.QtCore import Qt, QTimer
from PyQt6.QtGui import QAction

from .filter_panel import FilterPanel
from .spectrum_plot import SpectrumPlot
from .waveform_plot import WaveformPlot
from ..controllers.dsp_controller import DSPController


class MainWindow(QMainWindow):
    """Main application window"""
    
    def __init__(self):
        super().__init__()
        
        self.setWindowTitle("Spectral Workbench - Real-time Audio Analysis")
        self.setGeometry(100, 100, 1400, 900)
        
        # Initialize DSP controller
        self.dsp_controller = DSPController()
        
        # Setup UI
        self._setup_ui()
        self._setup_menubar()
        self._setup_statusbar()
        
        # Setup update timer (60 Hz)
        self.update_timer = QTimer()
        self.update_timer.timeout.connect(self._update_plots)
        self.update_timer.setInterval(16)  # ~60 Hz
        
    def _setup_ui(self):
        """Setup the user interface"""
        central_widget = QWidget()
        self.setCentralWidget(central_widget)
        
        main_layout = QHBoxLayout(central_widget)
        
        # Left panel: Filter controls
        self.filter_panel = FilterPanel(self.dsp_controller)
        self.filter_panel.setMaximumWidth(350)
        
        # Right panel: Plots
        plots_widget = QWidget()
        plots_layout = QVBoxLayout(plots_widget)
        plots_layout.setContentsMargins(0, 0, 0, 0)
        
        # Create splitter for plots
        plots_splitter = QSplitter(Qt.Orientation.Vertical)
        
        # Waveform plot (time domain)
        self.waveform_plot = WaveformPlot()
        plots_splitter.addWidget(self.waveform_plot)
        
        # Spectrum plot (frequency domain)
        self.spectrum_plot = SpectrumPlot()
        plots_splitter.addWidget(self.spectrum_plot)
        
        plots_splitter.setStretchFactor(0, 1)
        plots_splitter.setStretchFactor(1, 2)
        
        plots_layout.addWidget(plots_splitter)
        
        # Add to main layout
        main_layout.addWidget(self.filter_panel)
        main_layout.addWidget(plots_widget, stretch=1)
        
    def _setup_menubar(self):
        """Setup menu bar"""
        menubar = self.menuBar()
        
        # File menu
        file_menu = menubar.addMenu("&File")
        
        start_action = QAction("&Start Audio", self)
        start_action.setShortcut("Ctrl+S")
        start_action.triggered.connect(self._start_audio)
        file_menu.addAction(start_action)
        
        stop_action = QAction("S&top Audio", self)
        stop_action.setShortcut("Ctrl+T")
        stop_action.triggered.connect(self._stop_audio)
        file_menu.addAction(stop_action)
        
        file_menu.addSeparator()
        
        exit_action = QAction("E&xit", self)
        exit_action.setShortcut("Ctrl+Q")
        exit_action.triggered.connect(self.close)
        file_menu.addAction(exit_action)
        
        # View menu
        view_menu = menubar.addMenu("&View")
        
        reset_view_action = QAction("&Reset View", self)
        reset_view_action.setShortcut("Ctrl+R")
        reset_view_action.triggered.connect(self._reset_view)
        view_menu.addAction(reset_view_action)
        
        # Help menu
        help_menu = menubar.addMenu("&Help")
        
        about_action = QAction("&About", self)
        about_action.triggered.connect(self._show_about)
        help_menu.addAction(about_action)
        
    def _setup_statusbar(self):
        """Setup status bar"""
        self.statusBar = QStatusBar()
        self.setStatusBar(self.statusBar)
        self.statusBar.showMessage("Ready")
        
    def _start_audio(self):
        """Start audio capture and processing"""
        try:
            device_info = self.dsp_controller.start_audio()
            self.statusBar.showMessage(
                f"Audio started: {device_info['name']} "
                f"({device_info['sample_rate']} Hz, {device_info['channels']} ch)"
            )
            self.update_timer.start()
        except Exception as e:
            self.statusBar.showMessage(f"Error starting audio: {e}")
            
    def _stop_audio(self):
        """Stop audio capture"""
        try:
            self.dsp_controller.stop_audio()
            self.update_timer.stop()
            self.statusBar.showMessage("Audio stopped")
        except Exception as e:
            self.statusBar.showMessage(f"Error stopping audio: {e}")
            
    def _update_plots(self):
        """Update plots with new data (called at 60 Hz)"""
        try:
            # Get processed audio data
            waveform_data = self.dsp_controller.get_waveform_data()
            spectrum_data = self.dsp_controller.get_spectrum_data()
            
            if waveform_data is not None:
                self.waveform_plot.update_plot(waveform_data)
                
            if spectrum_data is not None:
                self.spectrum_plot.update_plot(
                    spectrum_data['frequencies'],
                    spectrum_data['magnitude']
                )
                
        except Exception as e:
            self.statusBar.showMessage(f"Update error: {e}")
            
    def _reset_view(self):
        """Reset plot views"""
        self.waveform_plot.reset_view()
        self.spectrum_plot.reset_view()
        self.statusBar.showMessage("View reset")
        
    def _show_about(self):
        """Show about dialog"""
        from PyQt6.QtWidgets import QMessageBox
        
        QMessageBox.about(
            self,
            "About Spectral Workbench",
            "<h2>Spectral Workbench v0.1.0</h2>"
            "<p>Real-time audio filtering and spectral analysis tool</p>"
            "<p>Combining high-performance Rust DSP with PyQt6 GUI</p>"
            "<p>Based on FIR filter design and FFT analysis from DSP course</p>"
            "<p><b>Author:</b> Anastasios Chatzigiannakis</p>"
        )
        
    def closeEvent(self, event):
        """Handle window close"""
        self._stop_audio()
        event.accept()
