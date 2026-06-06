# ternary-signals

**Fixed-point signal processing for ternary data: DFT, autocorrelation, spectral density, frequency detection, and cross-correlation — no floating point, no_std compatible.**

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

## Background

Digital signal processing (DSP) for ternary-valued signals is an unexplored frontier. Classical DSP assumes real or complex-valued signals and relies on floating-point arithmetic. But ternary signals — sequences of {−1, 0, +1} values — arise in ternary logic circuits, balanced-neural network activations, three-state sensor readings, and ternary communication channels.

The challenge: how do you perform Fourier analysis, autocorrelation, and frequency detection on ternary data without floating point? The answer lies in **fixed-point arithmetic**: represent complex numbers as scaled integers (Q16.16 format) and compute trigonometric values via lookup tables. This makes the entire signal processing pipeline compatible with `#![no_std]` environments — microcontrollers, FPGAs, and embedded systems where floating-point units may not exist.

`ternary-signals` provides:
- A fixed-point complex number type (`FixedComplex`) with Q16.16 scaling.
- A lookup-table-based `cos_sin_fixed()` function for trigonometric computation.
- Ternary DFT (O(N²), suitable for short signals).
- Autocorrelation and cross-correlation.
- Power spectral density and dominant frequency detection.
- Periodicity detection.
- All using only integer arithmetic.

## How It Works

### Fixed-Point Complex Numbers

```rust
pub struct FixedComplex {
    pub re: i64,  // Real part × 2^16
    pub im: i64,  // Imaginary part × 2^16
}
```

Operations:
- `add` — Component-wise addition.
- `mul` — Complex multiplication with scale correction: `(ac − bd)/SCALE + (ad + bc)/SCALE`.
- `mag_sq` — Squared magnitude: `(re² + im²) / SCALE²`.
- `mag` — Approximate magnitude via integer square root (`isqrt`).

The `SCALE` constant is `2^16 = 65536`, giving ~4 decimal digits of precision.

### Trigonometric Lookup

`cos_sin_fixed(k, n)` computes cos(2πk/n) and sin(2πk/n) using a 9-entry lookup table for sin(0) through sin(π/2) with 8 intermediate steps, then maps arbitrary angles to the correct quadrant. This avoids any floating-point transcendental functions.

### Ternary DFT

```rust
let signal: Vec<Ternary> = /* ... */;
let spectrum: Vec<FixedComplex> = ternary_dft(&signal);
```

Computes X[k] = Σ x[t] × e^(−i2πkt/N) for k = 0..N−1. O(N²) complexity — appropriate for the short signal lengths typical in ternary systems (N < 1000).

### Spectral Analysis

- `spectral_density(signal)` → Power spectral density: |X[k]|² for each frequency bin.
- `dominant_frequency(signal)` → Index and magnitude of the strongest frequency component.
- `detect_periodic(signal, threshold)` → Boolean: does any non-DC frequency have energy above the threshold fraction of total energy?

### Correlation

- `autocorrelation(signal, lag)` → Sum of x[t] × x[t + lag] over all valid t.
- `autocorrelation_all(signal)` → Autocorrelation at all lags 0..N−1.
- `cross_correlation(a, b, lag)` → Cross-correlation at a given lag (positive or negative).
- `energy(signal)` → Sum of squares.

## Experimental Results

The test suite verifies:
- **Fixed-point arithmetic**: `FixedComplex::new(1,0) × new(1,0) ≈ new(1,0)` (within 1/8 SCALE).
- **Integer square root**: `isqrt(4) = 2`, `isqrt(100) = 10`.
- **Autocorrelation of alternating signal**: `[+1, −1, +1, −1]` has autocorrelation 4 at lag 0, −3 at lag 1 (anti-correlated), and 2 at lag 2 (periodic).
- **Energy conservation**: `energy([+1, −1, 0, +1]) = 3`.
- **DFT DC component**: All-positive signal has strongest DC component.
- **Spectral density**: Non-empty, correct length.
- **Dominant frequency**: Constant signal has DC (index 0) as dominant.
- **Cross-correlation**: Identical signals at lag 0 give energy-matching correlation.
- **Periodicity detection**: All-zeros signal is not periodic.

## Impact

This crate proves that meaningful signal processing is possible on ternary data using only integer arithmetic. The no_std, no-float design makes it deployable on any platform Rust targets — from ARM Cortex-M microcontrollers to RISC-V embedded processors to WASM modules in the browser. The fixed-point approach trades some precision for universal compatibility.

## Use Cases

1. **Ternary Communication Analysis** — Analyze ternary-encoded signals in communication channels. Compute spectral density to identify channel characteristics and dominant frequency to detect carrier signals.
2. **Sensor Fusion** — Three-state sensor outputs (low/normal/high) can be analyzed for periodicity, autocorrelation (persistence), and cross-correlation between sensors.
3. **Embedded Ternary DSP** — On microcontrollers without FPU, process ternary sensor data in real time. The O(N²) DFT is practical for N < 100.
4. **Neural Network Activation Analysis** — Analyze the spectral content of ternary neural network activations across layers to understand information flow.
5. **Anomaly Detection** — Compare the spectral density of a running ternary signal against a baseline. Deviations indicate anomalies.

## Open Questions

1. **FFT for ternary signals** — The O(N²) DFT limits practical signal length. Would a radix-2 or radix-3 FFT adapted for fixed-point arithmetic provide meaningful speedup?
2. **Precision analysis** — What is the maximum signal length N for which Q16.16 precision gives acceptable results? At what N does accumulated rounding error dominate?
3. **Ternary-specific transforms** — Is there a natural transform (analogous to the Walsh-Hadamard transform for binary) that exploits the three-valued structure more directly than the DFT?

## Connection to Oxide Stack

`ternary-signals` is the analysis layer of the ternary fleet. It consumes ternary data from `ternary-walk` (walk trajectory analysis), `ternary-irradiate` (damage signal analysis), `ternary-morph` (spatial frequency of morphological features), and `ternary-pid` (control signal analysis). The `#![no_std]` guarantee aligns with `ternary-core`'s philosophy: the entire ternary stack, from arithmetic to signal processing, should run anywhere Rust runs.
