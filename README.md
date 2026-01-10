# Spectral Workbench

Real-time audio filtering and spectral analysis tool combining high-performance Rust DSP with PyQt6 GUI.

## Features

- **Real-time FIR Filtering**: Design and apply bandpass filters with adjustable parameters
- **Multiple Window Functions**: Hann, Hamming, Blackman, and Rectangular
- **Live Spectrum Analysis**: FFT-based magnitude spectrum and waterfall display
- **Interactive GUI**: Sliders for real-time parameter adjustment
- **High Performance**: Rust-powered DSP core with Python GUI

## Project Structure

```
Spectral Workbench/
├── Cargo.toml              # Rust workspace configuration
├── pyproject.toml          # Python package configuration
├── plan.md                 # Architecture and implementation plan
│
├── rust-core/              # High-performance Rust DSP engine
│   ├── src/
│   │   ├── filters/        # FIR filter design and processing
│   │   ├── spectrum/       # FFT and spectral analysis
│   │   ├── audio/          # Audio I/O with cpal
│   │   └── python_bindings/# PyO3 bindings for Python
│   └── Cargo.toml
│
├── python/                 # Python GUI application
│   └── spectral_workbench/
│       ├── ui/             # PyQt6 interface
│       ├── controllers/    # Business logic
│       └── utils/          # Utilities
│
└── docs/                   # Documentation and references
    ├── matlab-reference/   # Original MATLAB code
    └── images/             # Part A/B result images
```

## Installation

### Prerequisites

- **Rust**: Install from [rustup.rs](https://rustup.rs/)
- **Python 3.8+**: With pip
- **PyQt6**: Should be installed automatically

### Build Steps

1. **Clone or navigate to the project**:
   ```bash
   cd "C:\Users\anchatzigiannakis\Documents\Projects\Experiments\Spectral Workbench"
   ```

2. **Install Python dependencies and build Rust extension**:
   ```bash
   pip install maturin
   maturin develop --release
   ```

   This will:
   - Compile the Rust DSP core
   - Build the Python extension module
   - Install the package in development mode

3. **Install additional Python packages** (if not auto-installed):
   ```bash
   pip install PyQt6 pyqtgraph numpy scipy
   ```

## Usage

### Running the Application

```bash
python -m spectral_workbench.main
```

Or if installed as a package:
```bash
spectral-workbench
```

### GUI Controls

**Filter Design Panel**:
- **Window Type**: Select window function (Hann, Hamming, Blackman)
- **Lower Cutoff (ωc1)**: Adjust lower cutoff frequency
- **Upper Cutoff (ωc2)**: Adjust upper cutoff frequency  
- **Transition Width (Δω)**: Control transition band width
- **Apply Filter**: Redesign filter with current parameters
- **Reset to Part A**: Load original specifications from course project
- **Bypass Filter**: Toggle filtering on/off

**FFT Analysis Panel**:
- **FFT Size**: Select FFT size (512 to 8192 samples)
- **Analysis Window**: Window function for spectral analysis

**Menu Bar**:
- **File → Start Audio**: Begin capturing from microphone
- **File → Stop Audio**: Stop audio capture
- **View → Reset View**: Auto-scale plots

### Plots

- **Waveform Plot**: Time-domain display of input (blue) and filtered (red) signals
- **Magnitude Spectrum**: Frequency-domain magnitude in dB
- **Waterfall Spectrogram**: Time-frequency representation

## Algorithm Details

### FIR Filter Design (Part A)

Based on the windowing method:

1. **Calculate filter length** based on window type and transition width:
   - Hann/Hamming: M = ⌈8π / Δω⌉
   - Blackman: M = ⌈12π / Δω⌉

2. **Generate ideal impulse response** for bandpass:
   ```
   h_ideal[n] = (sin(ωc2·n) - sin(ωc1·n)) / (π·n)
   ```

3. **Apply window function** and shift to causal:
   ```
   h[n] = h_ideal[n - (M-1)/2] · w[n]
   ```

### Spectrum Analysis (Part B)

1. **Apply window** to input signal to reduce spectral leakage
2. **Zero-pad** to FFT size if needed
3. **Compute FFT** using optimized real FFT
4. **Calculate magnitude** in dB: 20·log₁₀(|X[k]|)

### Part A Specifications

- **Passband**: [0.4π, 0.6π] rad/sample
- **Stopband**: [0, 0.35π] ∪ [0.65π, π] rad/sample
- **Transition Width**: 0.05π rad
- **Default Window**: Hamming (53 dB stopband attenuation)

## Development

### Run Tests

```bash
# Rust tests
cd rust-core
cargo test

# Python tests (if added)
pytest
```

### Build for Release

```bash
maturin build --release
```

### Benchmarks

```bash
cd rust-core
cargo bench
```

## Technical Stack

- **DSP Core**: Rust with `rustfft`, `realfft`, `cpal`
- **Python Bindings**: PyO3, numpy
- **GUI Framework**: PyQt6
- **Plotting**: pyqtgraph (real-time performance)

## Performance

- **Audio Latency**: <20ms typical (depends on buffer size)
- **Filter Processing**: ~0.1ms for 161-tap filter on 1024 samples
- **FFT**: ~0.5ms for 2048-point real FFT
- **GUI Update Rate**: 60 Hz

## References

- **Course**: Digital Signal Processing (DSP), University of Thessaly
- **Original Implementation**: MATLAB scripts in `docs/matlab-reference/`
- **Textbook**: Oppenheim & Schafer, "Discrete-Time Signal Processing"

## License

Academic project - see course materials for usage restrictions.

## Author

Anastasios Chatzigiannakis (03839)  
Digital Signal Processing Course  
University of Thessaly, 2025-2026
