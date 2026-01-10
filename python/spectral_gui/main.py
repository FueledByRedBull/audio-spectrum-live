"""
Main application entry point
"""

import sys
from PyQt6.QtWidgets import QApplication
from .ui.main_window import MainWindow


def main():
    """Launch the Spectral Workbench application"""
    app = QApplication(sys.argv)
    app.setApplicationName("Spectral Workbench")
    app.setOrganizationName("DSP Lab")
    
    window = MainWindow()
    window.show()
    
    sys.exit(app.exec())


if __name__ == "__main__":
    main()
