# Filter Presets for Testing

**Note:** With the new spinbox controls, you can enter values directly or use up/down arrows for 0.001 increments (24 Hz steps at 48 kHz).

## Recommended Settings for Each Window Type

### 1. **Voice Band (Telephony) - 300-3400 Hz**
Good for isolating speech/vocals
- **Filter Type:** Bandpass
- **Lower Cutoff (ωc1):** 0.040 (960 Hz @ 48kHz)
- **Upper Cutoff (ωc2):** 0.440 (10.56 kHz @ 48kHz)
- **Transition Width:** 0.050 (1.2 kHz)
- **Window:** Hamming (good stopband attenuation)

### 2. **Bass Filter - 20-250 Hz**
For low frequency analysis (bass, drums, rumble)
- **Filter Type:** Lowpass
- **Cutoff:** 0.030 (720 Hz @ 48kHz)
- **Transition Width:** 0.020 (480 Hz)
- **Window:** Blackman (best stopband, wider transition)

### 3. **Treble/High-Pass - 4-16 kHz**
For high frequency analysis (cymbals, sibilance)
- **Filter Type:** Highpass
- **Cutoff:** 0.520 (12.48 kHz @ 48kHz)
- **Transition Width:** 0.100 (2.4 kHz)
- **Window:** Hann (good compromise)

### 4. **Narrow Notch - Remove 1 kHz Hum**
For removing specific frequency (like AC hum at 1 kHz)
- **Filter Type:** Bandpass
- **Lower Cutoff (ωc1):** 0.130 (3.12 kHz)
- **Upper Cutoff (ωc2):** 0.140 (3.36 kHz)
- **Transition Width:** 0.010 (240 Hz)
- **Window:** Rectangular (sharp, short filter)

### 5. **Wide Band Audio - 100 Hz to 10 kHz**
General purpose audio filtering
- **Filter Type:** Bandpass
- **Lower Cutoff (ωc1):** 0.010 (240 Hz)
- **Upper Cutoff (ωc2):** 0.830 (19.92 kHz)
- **Transition Width:** 0.050 (1.2 kHz)
- **Window:** Hamming (default, good overall)

### 6. **Part A Specification (Original DSP Assignment)**
Exact specification from MATLAB project
- **Filter Type:** Bandpass
- **Lower Cutoff (ωc1):** 0.400 (9.6 kHz)
- **Upper Cutoff (ωc2):** 0.600 (14.4 kHz)
- **Transition Width:** 0.050 (1.2 kHz)
- **Window:** Any (try all to compare)

---

## Window Type Characteristics

| Window | Main Lobe Width | Stopband Attenuation | Use Case |
|--------|----------------|---------------------|----------|
| **Rectangular** | Narrowest | ~21 dB (worst) | Sharp transitions, shortest filter |
| **Hann** | Medium | ~44 dB | General purpose, smooth transitions |
| **Hamming** | Medium | ~53 dB | Best compromise for most audio |
| **Blackman** | Widest | ~74 dB (best) | Maximum stopband rejection |

---

## Quick Test Procedure

1. **Set both mic and headphones to 48 kHz** in Windows sound settings
2. **Start Audio** - verify no errors about sample rate
3. **Select preset** from dropdown at top of panel
4. **Adjust values** using spinbox (type directly or use arrows)
5. **Apply Filter** - check filter length in status bar
6. **Enable Monitor** (with headphones!) - verify no crackling
7. **Test different filter types** - observe spectrum changes
8. **Switch window types** - observe transition sharpness changes

---

## Frequency Conversion Reference

At 48 kHz sample rate:
- **ω = 0.001π** → 24 Hz (minimum spinbox step)
- **ω = 0.010π** → 240 Hz
- **ω = 0.100π** → 2.4 kHz
- **ω = 0.400π** → 9.6 kHz
- **ω = 0.600π** → 14.4 kHz
- **ω = 0.800π** → 19.2 kHz
- **ω = 1.000π** → 24 kHz (Nyquist)

Formula: `f_Hz = (ω/π) × (sample_rate/2) = ω × 24000` at 48 kHz
