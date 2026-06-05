//! The device behind the API seam, and a stochastic mock standing in for it.
//!
//! `Device` is the seam: a real integration implements it against the
//! instrument's SDK / LIS / REST interface and drops that in. `MockAnalyzer`
//! is a faithful *stochastic* stand-in — it is not a fixture replay. Given the
//! same specimen twice it can return different calls, because it models what a
//! physical diagnostic instrument actually does: measure a latent quantity with
//! analytical noise, occasionally flag an invalid/QC result, and take a
//! variable amount of time. That run-to-run variability is the whole reason
//! feotest is the right tool — a frozen classifier on a frozen test set would
//! have none of it.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use crate::panel::Case;

/// A diagnostic instrument's response to one assay.
#[derive(Clone, Debug)]
pub enum Reading {
    /// The instrument made a call: `positive` (tumour) or not, plus the raw
    /// measurement it thresholded.
    Call { positive: bool, measurement: f64 },
    /// The instrument produced no valid result (QC flag, out-of-range, etc.).
    QcFail { reason: String },
}

/// The configuration a device runs under.
///
/// `software_version` and `reagent_lot` are the **covariates** — the declared,
/// versioned identity that a baseline is scoped to. The remaining fields are
/// the instrument's *hidden* analytical characteristics: a real device has
/// them physically; the mock models them explicitly so the showcase can drive
/// realistic baselines and realistic drift.
#[derive(Clone, Debug)]
pub struct DeviceConfig {
    pub software_version: String,
    pub reagent_lot: String,
    /// Measurement imprecision (sd of additive noise on the latent signal).
    pub noise_sd: f64,
    /// Systematic offset — e.g. a reagent-lot calibration shift.
    pub bias: f64,
    /// Probability of an invalid / QC-fail result on any given assay.
    pub qc_fail_rate: f64,
    /// Turnaround-time mean / sd, milliseconds.
    pub latency_ms_mean: f64,
    pub latency_ms_sd: f64,
}

/// The API seam. A real device adapter implements this against the instrument;
/// the contract neither knows nor cares which implementation it drives.
pub trait Device: Send + Sync {
    /// Run one assay on a specimen and return the instrument's reading.
    fn analyse(&self, case: &Case) -> Reading;

    /// The configuration this device is running under (its covariate identity).
    fn config(&self) -> &DeviceConfig;
}

/// A stochastic stand-in for a physical analyzer. Reproducible across a run
/// (seeded) but genuinely variable sample-to-sample.
pub struct MockAnalyzer {
    config: DeviceConfig,
    base_seed: u64,
    draw: AtomicU64,
}

impl MockAnalyzer {
    /// The decision threshold the instrument applies to its measurement. The
    /// reference panel defines truth as `severity >= THRESHOLD`, so a perfectly
    /// calibrated, noiseless device would be 100% accurate; noise and bias are
    /// what create the errors a real device has.
    const THRESHOLD: f64 = 0.5;

    #[must_use]
    pub const fn new(config: DeviceConfig, base_seed: u64) -> Self {
        Self {
            config,
            base_seed,
            draw: AtomicU64::new(0),
        }
    }
}

impl Device for MockAnalyzer {
    fn analyse(&self, case: &Case) -> Reading {
        let n = self.draw.fetch_add(1, Ordering::Relaxed);
        let seed = splitmix64(self.base_seed ^ n.wrapping_mul(0x2545_F491_4F6C_DD1D));

        // The instrument takes a variable amount of time. Sleeping here makes
        // feotest's latency dimension measure a real turnaround distribution.
        let latency = (self.config.latency_ms_mean + gaussian(seed ^ 0x0A) * self.config.latency_ms_sd)
            .max(0.3);
        std::thread::sleep(Duration::from_secs_f64(latency / 1000.0));

        if unit(seed ^ 0x0B) < self.config.qc_fail_rate {
            return Reading::QcFail {
                reason: "qc-flag".to_owned(),
            };
        }

        let measurement = case.severity + self.config.bias + gaussian(seed ^ 0x0C) * self.config.noise_sd;
        Reading::Call {
            positive: measurement >= Self::THRESHOLD,
            measurement,
        }
    }

    fn config(&self) -> &DeviceConfig {
        &self.config
    }
}

// --- tiny dependency-free PRNG: splitmix64 + Box-Muller -------------------

fn splitmix64(state: u64) -> u64 {
    let mut z = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Uniform in [0, 1).
fn unit(seed: u64) -> f64 {
    (splitmix64(seed) >> 11) as f64 / ((1u64 << 53) as f64)
}

/// Standard-normal draw via Box-Muller.
fn gaussian(seed: u64) -> f64 {
    let u1 = unit(seed).max(1e-12);
    let u2 = unit(seed ^ 0x9E37_79B9_7F4A_7C15);
    (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
}
