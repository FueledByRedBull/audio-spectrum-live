# Audio Spectrum Live

High-performance real-time audio filtering and spectral analysis tool with Rust DSP core and PyQt6 GUI.

## Features

- **Real-Time Audio Processing**: <5ms latency with zero-allocation ring buffer architecture
- **FIR Bandpass Filtering**: Design filters with adjustable cutoff frequencies and transition width
- **Multiple Window Functions**: Hann, Hamming, Blackman, and Rectangular
- **FFT-Based Fast Convolution**: Automatic O(N log N) optimization for long filters (>128 taps)
- **Live Spectrum Analysis**: Real-time FFT magnitude spectrum display
- **Unified Audio Processor**: All DSP processing in Rust thread, minimal Python/Rust boundary crossings
- **Interactive GUI**: PyQt6 interface with real-time waveform and spectrum plots
- **Event-Driven Architecture**: No polling loops, immediate response to audio data

## Project Structure

```
audio-spectrum-live/
├── Cargo.toml              # Rust workspace configuration
├── pyproject.toml          # Python package configuration
├── build.ps1               # Windows build script
├── .gitignore              # Git ignore rules
│
├── rust-core/              # High-performance Rust DSP engine
│   ├── src/
│   │   ├── filters/        # FIR filter design (ring buffer, FFT convolution)
│   │   ├── spectrum/       # FFT and spectral analysis
│   │   ├── audio/          # Audio I/O with cpal + unified AudioProcessor
│   │   └── python_bindings/# PyO3 bindings for Python
│   └── Cargo.toml
│
└── python/                 # Python GUI application
    └── spectral_gui/
        ├── ui/             # PyQt6 interface components
        ├── controllers/    # DSP controller (AudioProcessor wrapper)
        └── utils/          # Utilities
```

## Installation

### Prerequisites

- **Rust**: Install from [rustup.rs](https://rustup.rs/)
- **Python 3.8+**: [python.org](https://www.python.org/)
- **Visual Studio Build Tools**: Required on Windows for C++ compilation

### Quick Start (Windows)

1. **Clone the repository**:
   ```powershell
   git clone https://github.com/FueledByRedBull/audio-spectrum-live.git
   cd audio-spectrum-live
   ```

2. **Build and install** (automated):
   ```powershell
   .\build.ps1
   ```

   This will:
   - Install maturin (Rust-Python build tool)
   - Compile Rust core in release mode
   - Build Python wheel
   - Install wheel to Python site-packages
   - Install PyQt6, pyqtgraph, numpy, scipy

## Usage

### Running the Application

```powershell
# Set Python path to find GUI code
$env:PYTHONPATH = "python"

# Launch application
python -m spectral_gui.main
```

### GUI Controls

**Filter Design**:
- **Lower Cutoff (ωc1)**: Passband lower edge (normalized by π)
- **Upper Cutoff (ωc2)**: Passband upper edge (normalized by π)
- **Transition Width**: Control sharpness of filter transition
- **Window Type**: Select window function (affects stopband attenuation)
- **Apply Filter**: Redesign and apply filter in real-time
- **Bypass**: Toggle filtering on/off

**Audio Controls**:
- **Start Audio**: Begin capturing from default microphone
- **Stop Audio**: Stop audio capture

**Display**:
- **Top Plot**: Time-domain waveforms (input in blue, filtered in red)
- **Bottom Plot**: Frequency-domain magnitude spectrum in dB

## Performance Optimizations

### Ring Buffer Architecture
- **Zero allocations** in audio path using fixed-size Vec with modulo indexing
- Replaced `VecDeque` with circular buffer for 48kHz sample rate processing
- SIMD-friendly memory layout

### FFT-Based Fast Convolution
- **O(N log N)** complexity for filters >128 taps (vs O(N*M) time-domain)
- Automatic selection between direct and FFT convolution
- Pre-computed frequency-domain filter coefficients

### Unified Audio Processor
- **All DSP in Rust thread**: capture → filter → FFT analysis
- Python GUI calls `get_results()` only once per frame (60 Hz)
- **800x reduction** in language boundary crossings (48kHz → 60Hz)

### Event-Driven Design
- No polling loops or fixed-time sleeps
- Ring buffer consumer yields when empty, wakes immediately on data
- cpal callbacks drive the processing thread

## Performance Metrics

- **Latency**: <5ms round-trip (WASAPI + zero-allocation path)
- **GUI Updates**: 60 Hz
- **CPU Usage**: <5% audio thread, <10% GUI thread
- **Memory**: Zero allocations in hot path after initialization

## Algorithm Details

### FIR Filter Design

Windowing method implementation:

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

### Spectrum Analysis

1. **Apply window** to input signal to reduce spectral leakage
2. **Compute FFT** using optimized real FFT (rustfft/realfft)
3. **Calculate magnitude** in dB: 20·log₁₀(|X[k]|)
4. **Generate frequency bins** for x-axis labeling

## Technical Stack

- **DSP Core**: Rust 2021 with `rustfft` 6.4, `realfft` 3.5, `cpal` 0.15, `ringbuf` 0.3
- **Python Bindings**: PyO3 0.20, numpy 0.20
- **GUI Framework**: PyQt6 6.6+
- **Plotting**: pyqtgraph 0.13+ (optimized for real-time)
- **Build Tool**: maturin for Rust-Python interop

## Development

### Rebuild After Changes

```powershell
.\build.ps1
```

### Run Rust Tests

```bash
cargo test --manifest-path rust-core/Cargo.toml
```

### Build Release Binary

```bash
cargo build --manifest-path rust-core/Cargo.toml --release
```

## License

MIT License - see LICENSE file for details

## Author

Anastasios Chatzigiannakis  
GitHub: [@FueledByRedBull](https://github.com/FueledByRedBull)
