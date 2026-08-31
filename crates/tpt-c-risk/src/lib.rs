// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Schedule/cost risk analysis via Monte Carlo simulation.
//!
//! Self-contained (no external probability-distribution crate): provides the
//! common construction uncertainty distributions (uniform, triangular,
//! normal), a small deterministic PRNG, and a simulation harness that produces
//! percentile outcomes for cost or duration. Models weather risk and
//! productivity uncertainty as plain distributions.

use std::cmp::Ordering;

/// A probability distribution used as a risk input.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Dist {
    /// Uniform in `[lo, hi]`.
    Uniform {
        /// Lower bound.
        lo: f64,
        /// Upper bound.
        hi: f64,
    },
    /// Triangular with a most-likely `mode`.
    Triangular {
        /// Minimum.
        min: f64,
        /// Most likely.
        mode: f64,
        /// Maximum.
        max: f64,
    },
    /// Normal with mean and standard deviation.
    Normal {
        /// Mean.
        mean: f64,
        /// Standard deviation.
        sd: f64,
    },
}

/// A small, fast, deterministic PRNG (PCG-style) for reproducible simulations.
#[derive(Clone, Debug)]
pub struct Rng {
    state: u64,
}

impl Rng {
    /// Seed the generator.
    pub fn new(seed: u64) -> Self {
        Self { state: seed | 1 }
    }

    /// Next `u32` in the sequence.
    pub fn next_u32(&mut self) -> u32 {
        self.state = self.state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(144_269_504_088_896_340_7);
        let x = self.state;
        let rot = (x >> 59) as u32;
        let x = (x ^ (x >> 18)) >> 27;
        x.rotate_right(rot) as u32
    }

    /// Uniform float in `[0, 1)`.
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u32() as f64) / (u32::MAX as f64 + 1.0)
    }

    /// Standard normal sample via Box–Muller.
    pub fn normal(&mut self) -> f64 {
        let u1 = (self.next_f64().max(f64::MIN_POSITIVE)).min(1.0 - f64::MIN_POSITIVE);
        let u2 = self.next_f64();
        (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
    }
}

impl Dist {
    /// Draw a single sample using `rng`.
    pub fn sample(&self, rng: &mut Rng) -> f64 {
        match *self {
            Dist::Uniform { lo, hi } => lo + (hi - lo) * rng.next_f64(),
            Dist::Triangular { min, mode, max } => {
                let u = rng.next_f64();
                let fc = (mode - min) / (max - min);
                if u < fc {
                    min + (u * (max - min) * (mode - min)).sqrt()
                } else {
                    max - ((1.0 - u) * (max - min) * (max - mode)).sqrt()
                }
            }
            Dist::Normal { mean, sd } => mean + sd * rng.normal(),
        }
    }
}

/// Summary statistics of a simulated outcome distribution.
#[derive(Clone, Debug, PartialEq)]
pub struct Outcome {
    /// Mean outcome.
    pub mean: f64,
    /// Standard deviation.
    pub std_dev: f64,
    /// 10th percentile (pessimistic).
    pub p10: f64,
    /// 50th percentile (median).
    pub p50: f64,
    /// 90th percentile (optimistic).
    pub p90: f64,
    /// Minimum observed.
    pub min: f64,
    /// Maximum observed.
    pub max: f64,
}

impl Outcome {
    /// Compute statistics from a vector of samples (sorted internally).
    pub fn from_samples(mut samples: Vec<f64>) -> Self {
        let n = samples.len();
        let mean = if n == 0 {
            0.0
        } else {
            samples.iter().sum::<f64>() / n as f64
        };
        let variance = if n == 0 {
            0.0
        } else {
            samples.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n as f64
        };
        samples.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
        let at = |p: f64| -> f64 {
            if n == 0 {
                return 0.0;
            }
            let idx = ((p * (n as f64 - 1.0)).round() as usize).min(n - 1);
            samples[idx]
        };
        Self {
            mean,
            std_dev: variance.sqrt(),
            p10: at(0.10),
            p50: at(0.50),
            p90: at(0.90),
            min: samples.first().copied().unwrap_or(0.0),
            max: samples.last().copied().unwrap_or(0.0),
        }
    }
}

/// Run `iterations` Monte Carlo trials combining sampled `vars` via `combine`.
///
/// `combine` receives the sampled values (one per variable, in order) and
/// returns the scalar outcome for that trial.
pub fn simulate<F>(iterations: u32, seed: u64, vars: &[Dist], combine: F) -> Outcome
where
    F: Fn(&[f64]) -> f64,
{
    let mut rng = Rng::new(seed);
    let mut samples = Vec::with_capacity(iterations as usize);
    let mut buf = vec![0.0; vars.len()];
    for _ in 0..iterations {
        for (i, v) in vars.iter().enumerate() {
            buf[i] = v.sample(&mut rng);
        }
        samples.push(combine(&buf));
    }
    Outcome::from_samples(samples)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_and_centered() {
        let vars = [Dist::Uniform { lo: 90.0, hi: 110.0 }];
        let o = simulate(20_000, 42, &vars, |s| s[0]);
        assert!((o.mean - 100.0).abs() < 1.0);
        assert!(o.p10 < o.p50);
        assert!(o.p50 < o.p90);
    }

    #[test]
    fn additive_cost_risk() {
        // Material + labor + weather contingency.
        let vars = [
            Dist::Triangular { min: 80.0, mode: 100.0, max: 130.0 },
            Dist::Normal { mean: 50.0, sd: 8.0 },
        ];
        let o = simulate(10_000, 7, &vars, |s| s[0] + s[1]);
        assert!(o.mean > 120.0 && o.mean < 160.0);
    }
}
