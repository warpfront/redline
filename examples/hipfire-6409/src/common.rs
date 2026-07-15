// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

use serde::Serialize;

#[derive(Debug)]
pub struct Measurement {
    pub gpu_samples_us: Vec<f64>,
    pub output: Vec<u32>,
}

pub fn median(values: &[f64]) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    if sorted.len().is_multiple_of(2) {
        (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) * 0.5
    } else {
        sorted[sorted.len() / 2]
    }
}

pub fn percentile(values: &[f64], q: f64) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let p = q * (sorted.len() - 1) as f64;
    let lo = p.floor() as usize;
    let hi = p.ceil() as usize;
    sorted[lo] + (sorted[hi] - sorted[lo]) * (p - lo as f64)
}

#[derive(Clone, Debug, Serialize)]
pub struct Distribution {
    pub samples_us: Vec<f64>,
    pub median_us: f64,
    pub p05_us: f64,
    pub p95_us: f64,
    pub min_us: f64,
    pub max_us: f64,
}

impl Distribution {
    pub fn from_samples(samples_us: Vec<f64>) -> Self {
        Self {
            median_us: median(&samples_us),
            p05_us: percentile(&samples_us, 0.05),
            p95_us: percentile(&samples_us, 0.95),
            min_us: samples_us.iter().copied().reduce(f64::min).unwrap(),
            max_us: samples_us.iter().copied().reduce(f64::max).unwrap(),
            samples_us,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn median_handles_even_and_odd_samples() {
        assert_eq!(median(&[3.0, 1.0, 2.0]), 2.0);
        assert_eq!(median(&[4.0, 1.0, 3.0, 2.0]), 2.5);
    }
}
