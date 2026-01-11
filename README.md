# Audio Spectrum Live

High-performance real-time audio filtering and spectral analysis tool with Rust DSP core and PyQt6 GUI.

## Features

- **Real-Time Audio Processing**: <5ms latency with zero-allocation ring buffer architecture
- **Advanced FIR Filtering**: Bandpass, Lowpass, and Highpass filters with precise control
- **High-Precision Filter Design**: 1000-step spinbox controls (24 Hz granularity at 48 kHz)
- **Multiple Window Functions**: Hann, Hamming, Blackman, and Rectangular with different stopband characteristics
- **FFT-Based Fast Convolution**: Automatic O(N log N) optimization for long filters (>128 taps)
- **Live Spectrum Analysis**: Real-time FFT magnitude spectrum display with consistent output size
- **Audio Monitoring**: Listen to filtered output in real-time (with feedback protection warning)
- **Filter Presets**: 8 built-in presets for common use cases (voice, bass, treble, etc.)
- **Unified Audio Processor**: All DSP processing in Rust thread, minimal Python/Rust boundary crossings
- **Interactive GUI**: PyQt6 interface with real-time waveform and spectrum plots
- **Event-Driven Architecture**: No polling loops, immediate response to audio data
- **48 kHz Sample Rate Enforcement**: Refuses to start with mismatched audio device sample rates

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
- **Preset Selector**: Load pre-configured filters (Voice, Bass, Treble, Clear Voice, etc.)
- **Filter Type**: Choose Bandpass, Lowpass, or Highpass operation
- **Lower Cutoff (ωc1)**: Passband lower edge with spinbox control (0.000-1.000, 0.001 step)
  - Shows both normalized (ω) and Hz values
  - Hidden for Lowpass filters
- **Upper Cutoff (ωc2)**: Passband upper edge with spinbox control
  - Shows both normalized (ω) and Hz values
  - Hidden for Highpass filters
- **Transition Width**: Control sharpness of filter transition (0.001-0.200)
  - Narrower = sharper cutoff but longer filter
- **Window Type**: Select window function (affects stopband attenuation)
  - Rectangular: 21 dB, shortest filter
  - Hann: 44 dB, medium length
  - Hamming: 53 dB, best compromise
  - Blackman: 74 dB, maximum noise rejection
- **Apply Filter**: Redesign and apply filter in real-time
- **Bypass**: Toggle filtering on/off
- **Reset to Part A**: Load original DSP assignment specifications

**Audio Controls**:
- **Start Audio**: Begin capturing from default microphone (requires 48 kHz device)
- **Stop Audio**: Stop audio capture
- **Monitor Output**: Listen to filtered audio through speakers/headphones
  - ⚠ Warning dialog to prevent feedback
  - Use headphones recommended

**FFT Analysis**:
- **FFT Size**: 512, 1024, 2048, 4096, or 8192 points
- **Analysis Window**: Window function for spectral analysis

**Display**:
- **Top Plot**: Time-domain waveforms (input in blue, filtered in red)
- **Bottom Tabs**: 
  - Magnitude Spectrum: Frequency-domain view in dB
  - Waterfall Spectrogram: Time-frequency visualization

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

### Fixed FFT Size for Spectrum
- Always uses full FFT buffer (2048 samples) for consistent 1025-bin output
- Prevents array shape errors from varying signal lengths
- Ensures smooth spectrum plotting without flickering

## Performance Metrics

- **Latency**: <5ms round-trip (WASAPI + zero-allocation path)
- **GUI Updates**: 60 Hz
- **CPU Usage**: <5% audio thread, <10% GUI thread
- **Memory**: Zero allocations in hot path after initialization
- **Filter Precision**: 24 Hz steps at 48 kHz sampling (0.001 normalized frequency)

## Algorithm Details

### FIR Filter Design

Windowing method implementation with support for Bandpass, Lowpass, and Highpass:

1. **Calculate filter length** based on window type and transition width:
   - Hann/Hamming: M = ⌈8π / Δω⌉
   - Blackman: M = ⌈12π / Δω⌉
   - Rectangular: M = ⌈4π / Δω⌉

2. **Generate ideal impulse response**:
   - **Bandpass**: `h[n] = (sin(ωc2·n) - sin(ωc1·n)) / (π·n)`
   - **Lowpass**: `h[n] = sin(ωc·n) / (π·n)`
   - **Highpass**: `h[n] = δ[n] - sin(ωc·n) / (π·n)`

3. **Apply window function** and shift to causal:
   ```
   h[n] = h_ideal[n - (M-1)/2] · w[n]
   ```

4. **Automatic convolution method selection**:
   - Direct time-domain: filters ≤128 taps
   - FFT-based fast convolution: filters >128 taps

### Spectrum Analysis

1. **Apply window** to input signal to reduce spectral leakage
2. **Pad to FFT size** (2048 samples) for consistent output dimensions
3. **Compute FFT** using optimized real FFT (rustfft/realfft)
4. **Calculate magnitude** in dB: 20·log₁₀(|X[k]|)
5. **Generate frequency bins** (1025 bins from 0 to Nyquist)

## Technical Stack

- **DSP Core**: Rust 2021 with `rustfft` 6.4, `realfft` 3.5, `cpal` 0.15 (WASAPI), `ringbuf` 0.3
- **Python Bindings**: PyO3 0.20, numpy 0.20
- **GUI Framework**: PyQt6 6.6+
- **Plotting**: pyqtgraph 0.13+ (optimized for real-time)
- **Build Tool**: maturin for Rust-Python interop

## Requirements

- **Windows 10/11** (WASAPI audio backend)
- **Audio Devices**: Both input and output must be configured to **48 kHz** sample rate
  - The application will refuse to start if devices are not at 48 kHz
  - Check: Settings → Sound → Device Properties → Advanced → Default Format
- **Headphones Recommended**: For audio monitoring to avoid feedback loops

## Filter Presets

The application includes 8 built-in filter presets (see [FILTER_PRESETS.md](FILTER_PRESETS.md)):

1. **Voice Band (300-3400 Hz)**: Telephony-quality speech isolation
2. **Bass Filter (20-250 Hz)**: Low-frequency analysis
3. **Treble (4-16 kHz)**: High-frequency analysis
4. **Narrow Notch**: Remove specific tones
5. **Wide Band (100-10 kHz)**: General audio filtering
6. **Part A Specification**: Original DSP assignment (9.6-14.4 kHz)
7. **Clear Voice (Ice)**: Crystal-clear speech (1.5-4 kHz intelligibility band)
8. **Bassy Clear Voice**: Bass voice with noise removal (80-3 kHz)

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
