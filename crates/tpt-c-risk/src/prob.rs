// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Self-contained pseudo-random number generation and probability distributions.
//!
//! The upstream substrate `tpt-math-prob-dist` is not vendored in every
//! environment (see spec §4), so this module provides the small, dependency-free
//! toolkit the risk crate needs: a deterministic PCG64 generator plus the
//! Uniform, Triangular, Normal, and PERT/Beta distributions and a Poisson
//! sampler used for weather lost-day modelling.

/// A deterministic PCG32 pseudo-random generator (LCG64-XSH-rr).
///
/// The 64-bit state is advanced by a linear congruential generator and the
/// output is the well-tested XSH-RR permutation, yielding a uniformly
/// distributed `u32`. Wider values (`u64`, `f64`) are assembled from it.
#[derive(Clone, Debug)]
pub struct Rng {
    state: u64,
    inc: u64,
}

impl Rng {
    /// Seed the generator. The same seed always reproduces the same stream.
    pub fn new(seed: u64) -> Self {
        let mut s = Rng {
            state: 0,
            inc: (seed << 1) | 1,
        };
        s.next_u32();
        s.state = s.state.wrapping_add(seed);
        s.next_u32();
        s
    }

    /// Advance the underlying LCG state.
    fn step(&mut self) {
        self.state = self
            .state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(self.inc | 1);
    }

    /// The next raw 32-bit value (PCG XSH-RR output).
    pub fn next_u32(&mut self) -> u32 {
        let old = self.state;
        self.step();
        let xorshifted = (((old >> 18) ^ old) >> 27) as u32;
        let rot = (old >> 59) as u32;
        xorshifted.rotate_right(rot)
    }

    /// The next raw 64-bit value, built from two 32-bit outputs.
    pub fn next_u64(&mut self) -> u64 {
        (self.next_u32() as u64) << 32 | self.next_u32() as u64
    }

    /// A uniformly distributed `f64` in `[0, 1)`.
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Uniform sample in `[a, b)`.
    pub fn uniform(&mut self, a: f64, b: f64) -> f64 {
        a + (b - a) * self.next_f64()
    }

    /// A standard normal sample (mean 0, sd 1) via Box–Muller.
    pub fn standard_normal(&mut self) -> f64 {
        // Avoid log(0) by clamping the uniform away from 0.
        let u1 = (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64;
        let u1 = if u1 < 1e-12 { 1e-12 } else { u1 };
        let u2 = self.next_f64();
        let mag = (-2.0 * u1.ln()).sqrt();
        mag * (2.0 * std::f64::consts::PI * u2).cos()
    }

    /// A normal sample with the given mean and standard deviation.
    pub fn normal(&mut self, mean: f64, sd: f64) -> f64 {
        mean + sd * self.standard_normal()
    }

    /// Triangular sample with the given minimum, mode, and maximum.
    pub fn triangular(&mut self, min: f64, mode: f64, max: f64) -> f64 {
        let u = self.next_f64();
        let fc = (mode - min) / (max - min);
        if u < fc {
            min + (max - min).sqrt() * (u * (mode - min)).sqrt()
        } else {
            max - (max - min).sqrt() * ((1.0 - u) * (max - mode)).sqrt()
        }
    }

    /// PERT sample (Beta approximation) with the given minimum, mode, and
    /// maximum. The mode is weighted 4:1 versus the endpoints.
    pub fn pert(&mut self, min: f64, mode: f64, max: f64) -> f64 {
        let span = max - min;
        let alpha = 1.0 + 4.0 * (mode - min) / span;
        let beta = 1.0 + 4.0 * (max - mode) / span;
        let x = self.gamma(alpha);
        let y = self.gamma(beta);
        min + span * (x / (x + y))
    }

    /// Poisson sample with mean `lambda` (Knuth's algorithm; suitable for
    /// modest lambda such as expected weather lost days).
    pub fn poisson(&mut self, lambda: f64) -> f64 {
        if lambda <= 0.0 {
            return 0.0;
        }
        let l = (-lambda).exp();
        let mut k = 0.0;
        let mut p = 1.0;
        loop {
            k += 1.0;
            p *= self.next_f64();
            if p <= l {
                return k - 1.0;
            }
        }
    }

    /// Gamma sample with shape `k > 0` and unit scale (Marsaglia–Tsang).
    fn gamma(&mut self, k: f64) -> f64 {
        if k < 1.0 {
            // Boost small shapes using the scaling property Gamma(k) = Gamma(k+1) * U^(1/k).
            return self.gamma(k + 1.0) * self.next_f64().powf(1.0 / k);
        }
        let d = k - 1.0 / 3.0;
        let c = 1.0 / (9.0 * d).sqrt();
        loop {
            let x = self.standard_normal();
            let v = (1.0 + c * x).powi(3);
            let u = self.next_f64();
            if v > 0.0 && u < 1.0 - 0.0331 * x.powi(4) {
                return d * v;
            }
            if v > 0.0 && u.ln() < 0.5 * x * x + d * (1.0 - v + v.ln()) {
                return d * v;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_stream() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn ranges_and_moments() {
        let mut rng = Rng::new(7);
        for _ in 0..1000 {
            let u = rng.uniform(2.0, 5.0);
            assert!((2.0..5.0).contains(&u));
        }
        let mut sum = 0.0;
        let n = 20_000;
        let mut rng2 = Rng::new(11);
        for _ in 0..n {
            sum += rng2.normal(10.0, 2.0);
        }
        let mean = sum / n as f64;
        assert!((mean - 10.0).abs() < 0.1, "mean was {mean}");
    }

    #[test]
    fn pert_within_bounds() {
        let mut rng = Rng::new(99);
        for _ in 0..1000 {
            let v = rng.pert(1.0, 3.0, 9.0);
            assert!((1.0..=9.0).contains(&v));
        }
    }

    #[test]
    fn poisson_mean() {
        let mut rng = Rng::new(3);
        let n = 20_000;
        let mut sum = 0.0;
        for _ in 0..n {
            sum += rng.poisson(5.0);
        }
        let mean = sum / n as f64;
        assert!((mean - 5.0).abs() < 0.15, "mean {mean}");
    }

    #[test]
    fn uniform_quality() {
        let mut rng = Rng::new(123);
        let n = 100_000;
        let mut sum = 0.0;
        for _ in 0..n {
            sum += rng.next_f64();
        }
        let mean = sum / n as f64;
        assert!((mean - 0.5).abs() < 0.02, "mean {mean}");
    }
}
