# Future Integration: ternary-signals

## Current State
Provides signal processing for ternary data using fixed-point arithmetic (Q16.16 format, no floating point): ternary DFT, autocorrelation, spectral density estimation, and frequency detection. `no_std` compatible — designed for bare-metal environments where float hardware is unavailable.

## Integration Opportunities

### With ternary-cell (Oscillation Detection)
A cell grid ticking at microsecond speed produces periodic ternary signals. ternary-signals' DFT reveals dominant oscillation frequencies in the grid — if cells synchronize at a specific frequency, it indicates emergent coherence. Autocorrelation detects whether a cell's state is self-repeating (high autocorrelation = stuck, low = exploratory). The spectral density profile of a healthy grid differs from a degrading one.

### With ternary-streaming (Real-Time Spectral Analysis)
`StreamWindow` from ternary-streaming buffers tick sequences. ternary-signals' DFT processes those buffers. Together they form a real-time spectrum analyzer: streaming fills the window, signals computes the DFT, and the resulting frequency profile feeds back into the cell's predict phase. Cells operating at similar frequencies can be grouped into coherent "tissue" regions.

### With ternary-scheduling (Frequency-Based Priority)
ternary-scheduling assigns priorities using ternary decisions. ternary-signals can drive those decisions from spectral data: high-frequency oscillation = prioritize (unstable, needs attention), low-frequency drift = defer (stable, can wait), flat spectrum = neutral. The spectral density at the dominant frequency becomes a scheduling urgency metric.

## Potential in Mature Systems
In room-as-codespace, each room has a spectral fingerprint — the DFT of its tick history. Rooms with similar fingerprints are likely performing related computations and can share tiles. Rooms with anomalous spectra (unexpected frequency peaks) trigger anomaly detection. PLATO maintains a spectral atlas: a library of healthy room spectra used as baselines for comparison. The fixed-point arithmetic runs on ESP32, so even bare-metal rooms contribute spectral data.

## Cross-Pollination Ideas
- **ternary-noise**: Noise injection changes the spectral profile. Use DFT to measure how noise affects the frequency content — the "noise floor" in ternary.
- **ternary-kalman**: Kalman filter state estimates in frequency domain — track dominant frequency over time rather than raw values.
- **ternary-music (flux-algebra)**: Musical intervals are frequency ratios. A ternary DFT peak at frequency ratio 3:2 is a perfect fifth — rooms can "harmonize" when their spectral peaks form consonant intervals.

## Dependencies for Next Steps
- Add `SpectralProfile` type to ternary-cell for room-level frequency analysis
- Benchmark DFT on ESP32 at various window sizes (16, 32, 64, 128 trits)
- Define spectral similarity metric for room clustering in PLATO
- Create `FixedComplex` serialization for ternary-protocol
