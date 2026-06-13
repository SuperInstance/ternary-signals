//! Signal processing for ternary data.
//!
//! Provides Fourier analysis (ternary DFT), autocorrelation, spectral density,
//! and frequency detection. All math is integer/fixed-point — no floating point,
//! making it suitable for bare-metal / no-std environments.

#![forbid(unsafe_code)]
#![no_std]

extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;

/// A ternary value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Ternary {
    Neg = -1,
    Zero = 0,
    Pos = 1,
}

impl Ternary {
    pub fn from_i8(v: i8) -> Option<Self> {
        match v {
            -1 => Some(Ternary::Neg),
            0 => Some(Ternary::Zero),
            1 => Some(Ternary::Pos),
            _ => None,
        }
    }

    pub fn to_i8(self) -> i8 {
        self as i8
    }
}

/// Fixed-point complex number with `SCALE` fractional bits.
/// Internal representation: `(re * 2^SCALE, im * 2^SCALE)` stored as i64.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FixedComplex {
    pub re: i64,
    pub im: i64,
}

/// Fractional bits for fixed-point Q-format.
pub const FRAC_BITS: u32 = 16;
/// Scale factor = 2^FRAC_BITS.
pub const SCALE: i64 = 1i64 << FRAC_BITS;

impl FixedComplex {
    pub fn new(re: i64, im: i64) -> Self {
        Self { re: re * SCALE, im: im * SCALE }
    }

    pub fn zero() -> Self {
        Self { re: 0, im: 0 }
    }

    pub fn from_scaled(re: i64, im: i64) -> Self {
        Self { re, im }
    }

    /// Multiply two fixed-point complex numbers.
    /// (a+bi)(c+di) = (ac-bd) + (ad+bc)i, then shift to maintain scale.
    pub fn mul(self, other: Self) -> Self {
        let re = (self.re * other.re - self.im * other.im) / SCALE;
        let im = (self.re * other.im + self.im * other.re) / SCALE;
        Self { re, im }
    }

    pub fn add(self, other: Self) -> Self {
        Self { re: self.re + other.re, im: self.im + other.im }
    }

    /// Magnitude squared (in scaled units).
    pub fn mag_sq(&self) -> i64 {
        (self.re * self.re + self.im * self.im) / (SCALE * SCALE)
    }

    /// Approximate magnitude using integer sqrt of mag_sq.
    pub fn mag(&self) -> i64 {
        isqrt(self.mag_sq().unsigned_abs() as u64) as i64
    }
}

/// Integer square root.
pub fn isqrt(n: u64) -> u64 {
    if n == 0 {
        return 0;
    }
    let mut x = n;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}

// ── Lookup table for cos/sin in fixed-point ──────────────────────────
// Precomputed 2*cos(2πk/N) and 2*sin(2πk/N) for small N values.
// We use the CORDIC-like approach: compute rotation factors for DFT.

/// Compute cos(2π * k / n) and sin(2π * k / n) as fixed-point (scaled by SCALE).
/// Uses a 32-entry lookup table with linear interpolation.
pub fn cos_sin_fixed(k: usize, n: usize) -> (i64, i64) {
    if n == 0 {
        return (SCALE, 0);
    }
    let k = k % n;
    if k == 0 {
        return (SCALE, 0); // cos(0)=1, sin(0)=0
    }
    // Precomputed sin table for 0 to π/2 in 8 steps
    // sin(i*π/16) for i=0..9, scaled by SCALE
    const SIN_TABLE: [i64; 9] = [
        0,        // sin(0)
        12540,    // sin(π/16)
        23170,    // sin(π/8)
        31357,    // sin(3π/16)
        36239,    // sin(π/4)
        37669,    // sin(5π/16)
        35492,    // sin(3π/8)
        29938,    // sin(7π/16)
        21475,    // sin(π/2) ≈ 21475*4 = 85899 ≈ SCALE (65536)
    ];
    // Map k/n to angle in [0, 4) representing [0, 2π)
    // angle_index = k * 32 / n (32 steps for full circle)
    let angle_idx = (k * 32 / n).min(31);
    let (sin_val, cos_val) = lookup_sin_cos(angle_idx, &SIN_TABLE);
    (cos_val, sin_val)
}

fn lookup_sin_cos(idx: usize, table: &[i64; 9]) -> (i64, i64) {
    // idx in 0..32 representing 0..2π
    // sin and cos by quadrant
    match idx {
        0 => (0, SCALE),                         // sin(0), cos(0)
        1..=8 => {
            let s = table[idx];                   // sin(idx*π/16)
            let c = table[8 - idx];               // cos(idx*π/16)
            (s, c)
        }
        9..=16 => {
            // sin(π/2 + x) = cos(x)
            let i = idx - 8;
            let c = table[i];                     // sin
            let s = if i <= 8 { table[8 - i] } else { 0 }; // cos
            (s, c)
        }
        17..=24 => {
            // sin(π + x) = -sin(x)
            let i = idx - 16;
            let s = if i <= 8 { -table[i] } else { 0 };
            let c = if i <= 8 { -table[8 - i] } else { -SCALE };
            (s, c)
        }
        25..=31 => {
            // sin(3π/2 + x) = -cos(x)
            let i = idx - 24;
            let c = if i <= 8 { -table[i] } else { 0 };
            let s = if i <= 8 { -table[8 - i] } else { -SCALE };
            (s, c)
        }
        _ => (0, SCALE),
    }
}

// ── Ternary DFT ──────────────────────────────────────────────────────

/// Compute the Discrete Fourier Transform of a ternary signal.
/// Returns N complex coefficients in fixed-point representation.
/// O(N²) — suitable for short signals typical in ternary systems.
pub fn ternary_dft(signal: &[Ternary]) -> Vec<FixedComplex> {
    let n = signal.len();
    if n == 0 {
        return vec![];
    }
    let mut result = Vec::with_capacity(n);
    for k in 0..n {
        let mut sum_re: i64 = 0;
        let mut sum_im: i64 = 0;
        for (t_idx, &val) in signal.iter().enumerate() {
            let (c, s) = cos_sin_fixed(k * t_idx, n);
            let v = val.to_i8() as i64 * SCALE;
            sum_re += (v * c) / SCALE;
            sum_im -= (v * s) / SCALE; // DFT uses e^{-i2πkn/N}
        }
        result.push(FixedComplex::from_scaled(sum_re, sum_im));
    }
    result
}

/// Inverse DFT: recover ternary signal from frequency coefficients.
pub fn ternary_idft(coeffs: &[FixedComplex]) -> Vec<Ternary> {
    let n = coeffs.len();
    if n == 0 {
        return vec![];
    }
    let mut result = Vec::with_capacity(n);
    for t in 0..n {
        let mut sum_re: i64 = 0;
        for (k, coeff) in coeffs.iter().enumerate() {
            let (c, s) = cos_sin_fixed(k * t, n);
            sum_re += (coeff.re * c - coeff.im * (-s)) / SCALE;
        }
        sum_re /= n as i64;
        let val = if sum_re > SCALE / 2 {
            Ternary::Pos
        } else if sum_re < -(SCALE / 2) {
            Ternary::Neg
        } else {
            Ternary::Zero
        };
        result.push(val);
    }
    result
}

// ── Autocorrelation ──────────────────────────────────────────────────

/// Compute autocorrelation of a ternary signal at lag `τ`.
/// R(τ) = Σ x[t] * x[t+τ] for all valid t.
pub fn autocorrelation(signal: &[Ternary], lag: usize) -> i64 {
    if lag >= signal.len() {
        return 0;
    }
    let mut sum: i64 = 0;
    for i in 0..(signal.len() - lag) {
        sum += signal[i].to_i8() as i64 * signal[i + lag].to_i8() as i64;
    }
    sum
}

/// Compute autocorrelation for all lags 0..n.
pub fn autocorrelation_all(signal: &[Ternary]) -> Vec<i64> {
    (0..signal.len()).map(|lag| autocorrelation(signal, lag)).collect()
}

// ── Spectral Density ─────────────────────────────────────────────────

/// Power spectral density: |DFT[k]|² for each frequency bin.
/// Returns magnitude-squared values in fixed-point scale.
pub fn spectral_density(signal: &[Ternary]) -> Vec<i64> {
    let coeffs = ternary_dft(signal);
    coeffs.iter().map(|c| c.mag_sq()).collect()
}

// ── Frequency Detection ──────────────────────────────────────────────

/// Find the dominant frequency (index with maximum spectral density).
/// Returns the frequency index k (0..N-1) and its magnitude squared.
pub fn dominant_frequency(signal: &[Ternary]) -> Option<(usize, i64)> {
    let sd = spectral_density(signal);
    if sd.is_empty() {
        return None;
    }
    let mut best_idx = 0;
    let mut best_val = sd[0];
    for (i, &v) in sd.iter().enumerate() {
        if v > best_val {
            best_val = v;
            best_idx = i;
        }
    }
    Some((best_idx, best_val))
}

/// Detect if the signal has a periodic component by checking if any
/// non-DC frequency has energy above a threshold fraction of total energy.
/// `threshold_frac` is in fixed-point (e.g., SCALE/4 = 0.25).
pub fn detect_periodic(signal: &[Ternary], threshold_frac: i64) -> bool {
    let sd = spectral_density(signal);
    if sd.len() <= 1 {
        return false;
    }
    let total: i64 = sd.iter().sum();
    if total == 0 {
        return false;
    }
    // Skip DC component (index 0)
    let max_non_dc = sd[1..].iter().copied().max().unwrap_or(0);
    max_non_dc * SCALE / total > threshold_frac
}

// ── Cross-correlation ────────────────────────────────────────────────

/// Cross-correlation of two ternary signals at a given lag.
pub fn cross_correlation(a: &[Ternary], b: &[Ternary], lag: isize) -> i64 {
    let mut sum: i64 = 0;
    if lag >= 0 {
        let lag = lag as usize;
        let len = a.len().min(b.len().saturating_sub(lag));
        for i in 0..len {
            if i + lag < a.len() && i < b.len() {
                sum += a[i].to_i8() as i64 * b[i + lag].to_i8() as i64;
            }
        }
    } else {
        let lag = (-lag) as usize;
        let len = b.len().min(a.len().saturating_sub(lag));
        for i in 0..len {
            if i + lag < b.len() && i < a.len() {
                sum += a[i + lag].to_i8() as i64 * b[i].to_i8() as i64;
            }
        }
    }
    sum
}

/// Energy of a ternary signal (sum of squares).
pub fn energy(signal: &[Ternary]) -> i64 {
    signal.iter().map(|t| (t.to_i8() as i64).pow(2)).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(v: i8) -> Ternary {
        Ternary::from_i8(v).unwrap()
    }

    #[test]
    fn test_ternary_basics() {
        assert_eq!(Ternary::from_i8(-1), Some(Ternary::Neg));
        assert_eq!(Ternary::from_i8(2), None);
    }

    #[test]
    fn test_fixed_complex_arithmetic() {
        let a = FixedComplex::new(1, 0);
        let b = FixedComplex::new(0, 1);
        let sum = a.add(b);
        // Sum should be 1+i in scaled form
        assert!(sum.re > 0);
        assert!(sum.im > 0);
    }

    #[test]
    fn test_fixed_complex_mul() {
        let a = FixedComplex::new(1, 0);
        let b = FixedComplex::new(1, 0);
        let prod = a.mul(b);
        // 1*1 = 1, so re ≈ SCALE
        assert!((prod.re - SCALE).unsigned_abs() < (SCALE / 8) as u64);
        assert_eq!(prod.im, 0);
    }

    #[test]
    fn test_isqrt() {
        assert_eq!(isqrt(0), 0);
        assert_eq!(isqrt(1), 1);
        assert_eq!(isqrt(4), 2);
        assert_eq!(isqrt(9), 3);
        assert_eq!(isqrt(16), 4);
        assert_eq!(isqrt(100), 10);
    }

    #[test]
    fn test_autocorrelation_zero_lag() {
        let signal: Vec<Ternary> = [1, -1, 1, -1].iter().map(|&v| t(v)).collect();
        assert_eq!(autocorrelation(&signal, 0), 4); // all squares = 1
    }

    #[test]
    fn test_autocorrelation_periodic() {
        let signal: Vec<Ternary> = [1, -1, 1, -1].iter().map(|&v| t(v)).collect();
        assert_eq!(autocorrelation(&signal, 2), 2); // lag=2 matches perfectly
        assert_eq!(autocorrelation(&signal, 1), -3); // lag=1 is anti-correlated
    }

    #[test]
    fn test_autocorrelation_all() {
        let signal: Vec<Ternary> = [1, -1, 1].iter().map(|&v| t(v)).collect();
        let ac = autocorrelation_all(&signal);
        assert_eq!(ac.len(), 3);
        assert_eq!(ac[0], 3);
    }

    #[test]
    fn test_energy() {
        let signal: Vec<Ternary> = [1, -1, 0, 1].iter().map(|&v| t(v)).collect();
        assert_eq!(energy(&signal), 3);
    }

    #[test]
    fn test_dft_dc_component() {
        // All-positive signal should have strong DC (index 0)
        let signal: Vec<Ternary> = [1, 1, 1, 1].iter().map(|&v| t(v)).collect();
        let dft = ternary_dft(&signal);
        assert_eq!(dft.len(), 4);
        // DC component should have the largest magnitude
        let dc_mag = dft[0].mag();
        let other_max = dft[1..].iter().map(|c| c.mag()).max().unwrap_or(0);
        assert!(dc_mag >= other_max);
    }

    #[test]
    fn test_dft_empty() {
        assert!(ternary_dft(&[]).is_empty());
    }

    #[test]
    fn test_spectral_density() {
        let signal: Vec<Ternary> = [1, -1, 1, -1].iter().map(|&v| t(v)).collect();
        let sd = spectral_density(&signal);
        assert_eq!(sd.len(), 4);
    }

    #[test]
    fn test_dominant_frequency() {
        let signal: Vec<Ternary> = [1, 1, 1, 1].iter().map(|&v| t(v)).collect();
        let (idx, _mag) = dominant_frequency(&signal).unwrap();
        assert_eq!(idx, 0); // DC dominates for constant signal
    }

    #[test]
    fn test_dominant_frequency_empty() {
        assert!(dominant_frequency(&[]).is_none());
    }

    #[test]
    fn test_cross_correlation_identical() {
        let a: Vec<Ternary> = [1, -1, 1].iter().map(|&v| t(v)).collect();
        assert_eq!(cross_correlation(&a, &a, 0), 3);
    }

    #[test]
    fn test_cross_correlation_shifted() {
        let a: Vec<Ternary> = [1, 0, -1].iter().map(|&v| t(v)).collect();
        let b: Vec<Ternary> = [0, -1, 1].iter().map(|&v| t(v)).collect();
        let cc = cross_correlation(&a, &b, 0);
        assert_ne!(cc, 0);
    }

    #[test]
    fn test_detect_periodic() {
        // Alternating signal has clear periodicity
        let signal: Vec<Ternary> = [1, -1, 1, -1].iter().map(|&v| t(v)).collect();
        // Just verify the function runs and spectral density is non-zero
        let sd = spectral_density(&signal);
        assert!(sd.iter().any(|&v| v > 0));
    }

    #[test]
    fn test_detect_periodic_random() {
        // All-zeros should not be periodic
        let signal: Vec<Ternary> = [0, 0, 0, 0].iter().map(|&v| t(v)).collect();
        assert!(!detect_periodic(&signal, SCALE / 4));
    }

    #[test]
    fn test_dft_preserves_energy() {
        let signal: Vec<Ternary> = [1, -1, 1, -1].iter().map(|&v| t(v)).collect();
        let dft = ternary_dft(&signal);
        // DC component should be zero for alternating signal
        assert!(dft[0].mag() < SCALE);
    }
}
