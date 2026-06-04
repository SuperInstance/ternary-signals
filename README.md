# ternary-signals

Signal processing for ternary data — DFT, autocorrelation, spectral density, and frequency detection. No floating point.

## Why This Exists

Ternary signals (−1, 0, +1) show up in digital communications, ternary logic circuits, and quantized sensor data. But standard FFT libraries assume complex-valued inputs and output floating-point numbers. This crate provides a complete signal processing toolkit that works entirely in integer/fixed-point arithmetic — no `f64`, no `libm`. It's `no_std`-compatible and suitable for bare-metal DSP on ternary-valued data: Fourier analysis, autocorrelation, power spectral density, dominant frequency detection, and cross-correlation.

## Core Concepts

- **`Ternary`** — Three-valued signal element: `Neg` (−1), `Zero` (0), `Pos` (+1).
- **`FixedComplex`** — Fixed-point complex number using Q16 format (16 fractional bits). All DFT operations use this instead of `f64`/`Complex<f64>`.
- **`ternary_dft`** — Discrete Fourier Transform of a ternary signal. Returns N fixed-point complex coefficients.
- **`ternary_idft`** — Inverse DFT: recover a ternary signal from frequency coefficients by rounding.
- **Autocorrelation** — R(τ) = Σ x[t]·x[t+τ]. Pure integer arithmetic.
- **Spectral density** — |DFT[k]|² for each frequency bin, in fixed-point.

## Quick Start

```toml
# Cargo.toml
[dependencies]
ternary-signals = "0.1"
```

```rust
use ternary_signals::*;

fn main() {
    // Build a ternary signal
    let signal: Vec<Ternary> = [1, -1, 1, -1, 1, -1, 1, -1]
        .iter().map(|&v| Ternary::from_i8(v).unwrap()).collect();

    // Discrete Fourier Transform (fixed-point)
    let spectrum = ternary_dft(&signal);
    for (k, coeff) in spectrum.iter().enumerate() {
        println!("Bin {}: magnitude^2 = {}", k, coeff.mag_sq());
    }

    // Power spectral density
    let psd = spectral_density(&signal);
    println!("PSD: {:?}", psd);

    // Dominant frequency
    if let Some((freq, mag)) = dominant_frequency(&signal) {
        println!("Dominant frequency: bin {} (magnitude^2 = {})", freq, mag);
    }

    // Autocorrelation
    let ac = autocorrelation(&signal, 2);
    println!("Autocorrelation at lag 2: {}", ac);

    // Full autocorrelation
    let ac_all = autocorrelation_all(&signal);
    println!("All lags: {:?}", ac_all);

    // Energy
    println!("Signal energy: {}", energy(&signal));

    // Cross-correlation between two signals
    let other: Vec<Ternary> = [1, 1, -1, -1].iter()
        .map(|&v| Ternary::from_i8(v).unwrap()).collect();
    let cc = cross_correlation(&signal, &other, 0);
    println!("Cross-correlation: {}", cc);
}
```

## API Overview

### Fourier Analysis
- `ternary_dft(signal)` — Forward DFT. Returns `Vec<FixedComplex>` of length N. O(N²).
- `ternary_idft(coeffs)` — Inverse DFT. Rounds back to ternary values.
- `spectral_density(signal)` — |DFT[k]|² for each bin. Returns `Vec<i64>`.
- `dominant_frequency(signal)` — Index and magnitude of the strongest frequency component.

### Correlation
- `autocorrelation(signal, lag)` — Autocorrelation at a specific lag. Returns `i64`.
- `autocorrelation_all(signal)` — Autocorrelation for all lags 0..N−1.
- `cross_correlation(a, b, lag)` — Cross-correlation at a given lag (supports negative lags).

### Utilities
- `energy(signal)` — Sum of squares (Σ x[t]²).
- `detect_periodic(signal, threshold_frac)` — Check if any non-DC frequency exceeds a threshold fraction of total energy.

### Fixed-Point Arithmetic
- `FixedComplex::new(re, im)` — Create from integer real/imaginary parts
- `FixedComplex::zero()` — Zero complex number
- `a.mul(b)` / `a.add(b)` — Complex arithmetic in fixed-point
- `c.mag_sq()` / `c.mag()` — Magnitude (squared and approximate)
- `isqrt(n)` — Integer square root (Newton's method)

## How It Works

**Fixed-point arithmetic** uses Q16 format: all values are stored as `i64` with an implicit 2¹⁶ scale factor. Multiplication divides by the scale to prevent overflow. Trigonometric functions use a 32-entry lookup table with linear interpolation for cos/sin at angles 2πk/N.

**DFT** computes the standard formula X[k] = Σ x[t]·e^{−i2πkt/N} using the lookup table for twiddle factors. O(N²) complexity — appropriate for the short signal lengths typical in ternary systems.

**IDFT** computes x[t] = (1/N) Σ X[k]·e^{i2πkt/N} and rounds each value: if the result exceeds SCALE/2, it maps to Pos; below −SCALE/2, Neg; otherwise Zero.

**Autocorrelation** is pure integer arithmetic: R(τ) = Σ x[t]·x[t+τ] where x values are −1, 0, or +1. No fixed-point needed.

**Spectral density** runs the DFT and returns mag_sq() for each bin, which computes (re² + im²) / SCALE² in fixed-point.

## Use Cases

1. **Ternary communication receivers** — Demodulate ternary-encoded signals by detecting dominant frequency components using the fixed-point DFT on embedded hardware.
2. **Periodic pattern detection** — Check sensor data for periodic behavior using autocorrelation and spectral density without floating-point hardware.
3. **Signal similarity** — Use cross-correlation to find alignment between two ternary signal streams (e.g., matching a reference pattern in noisy data).
4. **Embedded DSP** — Process ternary sensor data on microcontrollers with no FPU (`no_std`, integer-only arithmetic).

## Ecosystem

- [`ternary-streaming`](https://github.com/user/ternary-streaming) — Streaming processing (windows, aggregators, pattern detection)
- [`ternary-regex`](https://github.com/user/ternary-regex) — Pattern matching on ternary sequences
- [`ternary-markov`](https://github.com/user/ternary-markov) — Markov chains on ternary state spaces

## License

MIT
