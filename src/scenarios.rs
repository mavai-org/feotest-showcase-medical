//! Device scenarios and shared constants used by the showcase binaries.

use crate::device::DeviceConfig;

/// The committed reference panel.
pub const PANEL: &str = "fixtures/reference-panel.csv";

/// The validated, healthy instrument.
#[must_use]
pub fn healthy() -> DeviceConfig {
    DeviceConfig {
        software_version: "fw-1.2.0".to_owned(),
        reagent_lot: "L42".to_owned(),
        noise_sd: 0.10,
        bias: 0.0,
        qc_fail_rate: 0.02,
        latency_ms_mean: 4.0,
        latency_ms_sd: 1.5,
    }
}

/// Same declared identity, but the instrument has silently degraded (more
/// measurement noise). The undeclared-change failure mode.
#[must_use]
pub fn regressed() -> DeviceConfig {
    DeviceConfig {
        noise_sd: 0.22,
        ..healthy()
    }
}

/// A new reagent lot — a *declared* change: the covariate value differs from
/// the baseline's, so the verdict carries a covariate-mismatch warning.
#[must_use]
pub fn new_lot() -> DeviceConfig {
    DeviceConfig {
        reagent_lot: "L77".to_owned(),
        bias: 0.04,
        ..healthy()
    }
}

/// The healthy instrument with a given measurement-noise level — used to model
/// gradual in-field drift across successive sentinel self-checks.
#[must_use]
pub fn drifting(noise_sd: f64) -> DeviceConfig {
    DeviceConfig {
        noise_sd,
        ..healthy()
    }
}
