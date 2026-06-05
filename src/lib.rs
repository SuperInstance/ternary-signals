//! Ternary signal processing: convolution, filtering, spectral analysis on {-1, 0, +1} signals.

/// Ternary signal — sequence of -1, 0, +1 values
#[derive(Clone, Debug)]
pub struct TernarySignal {
    pub samples: Vec<i8>,
    pub sample_rate: f64,
}

impl TernarySignal {
    pub fn new(samples: Vec<i8>) -> Self {
        assert!(samples.iter().all(|&v| v >= -1 && v <= 1));
        Self { samples, sample_rate: 1.0 }
    }

    pub fn with_rate(samples: Vec<i8>, rate: f64) -> Self {
        assert!(samples.iter().all(|&v| v >= -1 && v <= 1));
        Self { samples, sample_rate: rate }
    }

    pub fn len(&self) -> usize { self.samples.len() }
    pub fn is_empty(&self) -> bool { self.samples.is_empty() }

    /// Convolve with a kernel (integer arithmetic)
    pub fn convolve(&self, kernel: &[i8]) -> Vec<i64> {
        let n = self.samples.len();
        let k = kernel.len();
        let mut result = vec![0i64; n];
        for i in 0..n {
            for j in 0..k {
                let idx = if i + j >= n { i + j - n } else { i + j };
                result[i] += self.samples[idx] as i64 * kernel[j] as i64;
            }
        }
        result
    }

    /// Moving average (real-valued output)
    pub fn moving_average(&self, window: usize) -> Vec<f64> {
        let n = self.samples.len();
        let mut result = Vec::with_capacity(n);
        let mut sum = 0i64;
        for i in 0..n {
            sum += self.samples[i] as i64;
            if i >= window { sum -= self.samples[i - window] as i64; }
            let count = i.min(window - 1) + 1;
            result.push(sum as f64 / count as f64);
        }
        result
    }

    /// Discrete cosine transform (type-II) — returns amplitude spectrum
    pub fn dct(&self) -> Vec<f64> {
        let n = self.samples.len();
        let mut spectrum = Vec::with_capacity(n);
        for k in 0..n {
            let mut sum = 0.0;
            for i in 0..n {
                let angle = std::f64::consts::PI * (i as f64 + 0.5) * k as f64 / n as f64;
                sum += self.samples[i] as f64 * angle.cos();
            }
            spectrum.push(sum);
        }
        spectrum
    }

    /// Energy of the signal
    pub fn energy(&self) -> f64 {
        self.samples.iter().map(|&v| (v * v) as f64).sum()
    }

    /// Zero-crossing rate
    pub fn zero_crossing_rate(&self) -> f64 {
        if self.samples.len() < 2 { return 0.0; }
        let crossings = self.samples.windows(2)
            .filter(|w| w[0].signum() != w[1].signum())
            .count();
        crossings as f64 / (self.samples.len() - 1) as f64
    }

    /// Autocorrelation at given lag
    pub fn autocorrelation(&self, lag: usize) -> f64 {
        if lag >= self.samples.len() { return 0.0; }
        let n = self.samples.len() - lag;
        let mut sum = 0.0;
        for i in 0..n {
            sum += self.samples[i] as f64 * self.samples[i + lag] as f64;
        }
        sum / n as f64
    }

    /// Median filter (3-point)
    pub fn median_filter(&self) -> TernarySignal {
        let n = self.samples.len();
        let mut result = Vec::with_capacity(n);
        for i in 0..n {
            let mut window = vec![
                if i > 0 { self.samples[i-1] } else { self.samples[i] },
                self.samples[i],
                if i < n-1 { self.samples[i+1] } else { self.samples[i] },
            ];
            window.sort();
            result.push(window[1]);
        }
        TernarySignal { samples: result, sample_rate: self.sample_rate }
    }

    /// Threshold: quantize real signal to ternary
    pub fn quantize(samples: &[f64], low: f64, high: f64) -> TernarySignal {
        let ternary: Vec<i8> = samples.iter()
            .map(|&v| if v < low { -1 } else if v > high { 1 } else { 0 })
            .collect();
        TernarySignal::new(ternary)
    }

    /// Run-length encoding
    pub fn rle(&self) -> Vec<(i8, usize)> {
        let mut result = Vec::new();
        if self.samples.is_empty() { return result; }
        let mut current = self.samples[0];
        let mut count = 1;
        for &s in &self.samples[1..] {
            if s == current { count += 1; }
            else {
                result.push((current, count));
                current = s;
                count = 1;
            }
        }
        result.push((current, count));
        result
    }
}

/// Ternary filter bank
pub struct FilterBank {
    pub kernels: Vec<Vec<i8>>,
}

impl FilterBank {
    pub fn new() -> Self {
        Self { kernels: vec![
            vec![1, 1, 1],           // smoothing
            vec![-1, 0, 1],          // edge detect
            vec![1, -2, 1],          // second derivative
            vec![-1, -1, 2, 2, -1, -1], // bandpass
        ]}
    }

    pub fn apply(&self, signal: &TernarySignal) -> Vec<Vec<i64>> {
        self.kernels.iter().map(|k| signal.convolve(k)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convolution() {
        let sig = TernarySignal::new(vec![1, 1, -1, 0, 1]);
        let result = sig.convolve(&[1, 1]);
        assert_eq!(result.len(), 5);
        assert_eq!(result[0], 2); // 1*1 + 1*1
    }

    #[test]
    fn test_moving_average() {
        let sig = TernarySignal::new(vec![1, 1, -1, -1, 0]);
        let ma = sig.moving_average(3);
        assert!((ma[4] - (-2.0/3.0)).abs() < 1e-10);
    }

    #[test]
    fn test_dct() {
        let sig = TernarySignal::new(vec![1, 1, 1, 1, 1, 1, 1, 1]);
        let spectrum = sig.dct();
        assert!(!spectrum.is_empty());
        // All-ones signal: DC = 8, all other bins = 0
        assert!((spectrum[0] - 8.0).abs() < 1e-10);
        // k=1 bin should be 0 for constant signal
        assert!(spectrum[1].abs() < 1e-10);
    }

    #[test]
    fn test_energy() {
        let sig = TernarySignal::new(vec![1, -1, 0, 1]);
        assert!((sig.energy() - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_zero_crossing() {
        let sig = TernarySignal::new(vec![1, -1, 1, -1]);
        let rate = sig.zero_crossing_rate();
        assert!((rate - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_autocorrelation_periodic() {
        let sig = TernarySignal::new(vec![1, 0, -1, 0, 1, 0, -1, 0]);
        let ac0 = sig.autocorrelation(0);
        let ac4 = sig.autocorrelation(4);
        assert!(ac4 > 0.0); // Period 4 should show positive autocorrelation
    }

    #[test]
    fn test_median_filter_removes_impulse() {
        let mut samples = vec![0, 0, 0, 0, 0, 0, 0, 0, 0];
        samples[4] = 1; // impulse
        let sig = TernarySignal::new(samples);
        let filtered = sig.median_filter();
        assert_eq!(filtered.samples[4], 0); // impulse removed
    }

    #[test]
    fn test_quantize() {
        let data = vec![-2.0, -0.5, 0.0, 0.5, 2.0];
        let sig = TernarySignal::quantize(&data, -0.3, 0.3);
        assert_eq!(sig.samples, vec![-1, -1, 0, 1, 1]);
    }

    #[test]
    fn test_rle() {
        let sig = TernarySignal::new(vec![1, 1, 1, -1, -1, 0]);
        let rle = sig.rle();
        assert_eq!(rle, vec![(1, 3), (-1, 2), (0, 1)]);
    }

    #[test]
    fn test_filter_bank() {
        let fb = FilterBank::new();
        let sig = TernarySignal::new(vec![1, 0, -1, 1, 0, -1]);
        let results = fb.apply(&sig);
        assert_eq!(results.len(), 4);
        for r in &results {
            assert_eq!(r.len(), 6);
        }
    }
}
