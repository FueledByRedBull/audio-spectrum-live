# Filter Presets for Testing

**Note:** With the new spinbox controls, you can enter values directly or use up/down arrows for 0.001 increments (24 Hz steps at 48 kHz).

**Frequency Conversion at 48 kHz:** `normalized_frequency = Hz / 24000` where 24000 is the Nyquist frequency.

## Recommended Settings for Each Window Type

### 1. **Voice Band (Telephony) - 300-3400 Hz**
Good for isolating speech/vocals
- **Filter Type:** Bandpass
- **Lower Cutoff (ωc1):** 0.0125 (300 Hz @ 48kHz)
- **Upper Cutoff (ωc2):** 0.1417 (3400 Hz @ 48kHz)
- **Transition Width:** 0.0021 (50 Hz)
- **Window:** Hamming (good stopband attenuation)

### 2. **Bass Filter - 20-250 Hz**
For low frequency analysis (bass, drums, rumble)
- **Filter Type:** Bandpass
- **Lower Cutoff (ωc1):** 0.0008 (20 Hz @ 48kHz)
- **Upper Cutoff (ωc2):** 0.0104 (250 Hz @ 48kHz)
- **Transition Width:** 0.0021 (50 Hz)
- **Window:** Blackman (best stopband, wider transition)

### 3. **Treble/High-Pass - 4-16 kHz**
For high frequency analysis (cymbals, sibilance)
- **Filter Type:** Bandpass
- **Lower Cutoff (ωc1):** 0.1667 (4000 Hz @ 48kHz)
- **Upper Cutoff (ωc2):** 0.6667 (16000 Hz @ 48kHz)
- **Transition Width:** 0.0208 (500 Hz)
- **Window:** Hann (good compromise)

### 4. **Narrow Notch - Remove 900-1100 Hz Tone**
For removing specific frequency range
- **Filter Type:** Bandpass
- **Lower Cutoff (ωc1):** 0.0375 (900 Hz)
- **Upper Cutoff (ωc2):** 0.0458 (1100 Hz)
- **Transition Width:** 0.0021 (50 Hz)
- **Window:** Rectangular (sharp, short filter)

### 5. **Wide Band Audio - 100 Hz to 10 kHz**
General purpose audio filtering
- **Filter Type:** Bandpass
- **Lower Cutoff (ωc1):** 0.0042 (100 Hz)
- **Upper Cutoff (ωc2):** 0.4167 (10000 Hz)
- **Transition Width:** 0.0083 (200 Hz)
- **Window:** Hamming (default, good overall)

### 6. **Part A Specification (Original DSP Assignment) - 9.6-14.4 kHz**
Exact specification from MATLAB project (ωc1=0.4π, ωc2=0.6π)
- **Filter Type:** Bandpass
- **Lower Cutoff (ωc1):** 0.400 (9600 Hz)
- **Upper Cutoff (ωc2):** 0.600 (14400 Hz)
- **Transition Width:** 0.050 (1200 Hz)
- **Window:** Any (try all to compare)

### 7. **Clear Voice (Ice) - 1.5-4 kHz**
Crystal clear, crispy voice with maximum intelligibility
- **Filter Type:** Bandpass
- **Lower Cutoff (ωc1):** 0.0625 (1500 Hz)
- **Upper Cutoff (ωc2):** 0.1667 (4000 Hz)
- **Transition Width:** 0.0083 (200 Hz)
- **Window:** Hamming (clean, minimal ringing)

### 8. **Bassy Clear Voice - 80-3000 Hz**
Bass voice with background noise/hiss removed
- **Filter Type:** Bandpass
- **Lower Cutoff (ωc1):** 0.0033 (80 Hz)
- **Upper Cutoff (ωc2):** 0.1250 (3000 Hz)
- **Transition Width:** 0.0083 (200 Hz)
- **Window:** Blackman (maximum noise rejection)

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
