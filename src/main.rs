//! A worked feotest showcase: a diagnostic device as a *stochastic service*,
//! certified through the measure → verify loop.
//!
//! ```text
//! cargo run
//! ```
//!
//! The run tells the whole story in four phases:
//!   1. Characterise the device — a measure experiment mints a baseline
//!      ("how accurate is it?", validation).
//!   2. Verify a healthy device still meets that baseline ("does it still meet
//!      validated performance?", verification).
//!   3. Catch a silent regression — same declared config, a degraded
//!      instrument — as a FAIL.
//!   4. Catch a declared change — a new reagent lot — as a covariate-mismatch
//!      warning against the baseline it no longer matches.

mod contract;
mod device;
mod panel;

use std::path::Path;

use feotest::experiment::MeasureExperiment;
use feotest::model::ThresholdOrigin;
use feotest::ptest::ProbabilisticTest;
use feotest::ptest::builder::ThresholdApproach;
use feotest::spec::SpecResolver;

use crate::contract::{DiagnosticContract, covariate_keys, covariate_profile};
use crate::device::{DeviceConfig, MockAnalyzer};
use crate::panel::Case;

// Distinct seeds per phase: the device is reproducible within a run, but
// measurement and verification are *independent draws* from the same process,
// so a passing verification is statistics, not identity.
const SEED_MEASURE: u64 = 0x11;
const SEED_VERIFY: u64 = 0x22;
const SEED_DRIFT: u64 = 0x33;
const SEED_LOT: u64 = 0x44;

fn main() {
    let dir = Path::new("baselines");
    let _ = std::fs::remove_dir_all(dir);
    std::fs::create_dir_all(dir).expect("create baselines dir");

    let (positives, negatives) = panel::split(panel::load("fixtures/reference-panel.csv"));
    assert!(
        !positives.is_empty() && !negatives.is_empty(),
        "reference panel must contain both positive and normal specimens"
    );
    println!(
        "Digital-pathology analyzer — {} positive / {} normal reference specimens\n",
        positives.len(),
        negatives.len()
    );

    println!("=== 1. Characterise — measure experiment → baseline (\"how accurate is it?\") ===");
    characterize(true, &positives, &healthy(), SEED_MEASURE, dir);
    characterize(false, &negatives, &healthy(), SEED_MEASURE, dir);

    println!(
        "\n=== 2. Verify — probabilistic test vs baseline (\"does it still meet validated performance?\") ==="
    );
    verify("healthy device, same config", true, &positives, &healthy(), SEED_VERIFY, dir);

    println!("\n=== 3. Drift caught — a silent regression (same declared config, degraded instrument) ===");
    verify("degraded device, undeclared", true, &positives, &regressed(), SEED_DRIFT, dir);

    println!("\n=== 4. Covariate guard — a declared change (new reagent lot) ===");
    verify("new reagent lot L77", true, &positives, &new_lot(), SEED_LOT, dir);

    println!(
        "\nThe device here is a stochastic mock. To certify a real instrument, implement\n\
         `Device` for its API/SDK and drop it in — the contract is unchanged."
    );
}

/// Phase 1: run a measure experiment to establish the device's baseline over a
/// population, and report the characterised performance.
fn characterize(expected_positive: bool, panel: &[Case], config: &DeviceConfig, seed: u64, dir: &Path) {
    let id = DiagnosticContract::id_for(expected_positive);
    let cfg = config.clone();
    let result = MeasureExperiment::builder()
        .service_contract_id(id)
        .service_contract(move || {
            DiagnosticContract::new(expected_positive, Box::new(MockAnalyzer::new(cfg.clone(), seed)))
        })
        .samples(u32::try_from(panel.len()).expect("panel size fits u32"))
        .inputs(panel)
        .baseline_dir(dir)
        .covariates(covariate_keys(), covariate_profile(config))
        .build()
        .run();

    let spec = result.spec();
    let diag = if expected_positive { "sensitivity" } else { "specificity" };
    let (passes, fails) = spec
        .statistics
        .per_criterion
        .as_ref()
        .and_then(|m| m.get(diag))
        .map_or(
            (spec.statistics.successes, spec.statistics.failures),
            |c| (c.successes, c.failures),
        );
    let total = (passes + fails).max(1);
    println!(
        "  {diag}: {:.3} ({passes}/{total} correct)  ·  empirical floor (Wilson lower @95%) = {:.3}  ·  n = {}",
        f64::from(passes) / f64::from(total),
        spec.requirements.min_pass_rate,
        spec.execution.samples_executed,
    );
    let baseline_file = result.spec_path().and_then(|p| p.file_name()).map_or_else(
        || "(in memory)".to_owned(),
        |f| f.to_string_lossy().into_owned(),
    );
    println!(
        "    covariates: software_version={}, reagent_lot={}  →  {baseline_file}",
        config.software_version, config.reagent_lot,
    );
}

/// Phases 2–4: verify a device against its committed baseline and render the
/// full feotest verdict, including any covariate-mismatch warning.
fn verify(label: &str, expected_positive: bool, panel: &[Case], config: &DeviceConfig, seed: u64, dir: &Path) {
    println!("\n── {label} ──");
    let contract = DiagnosticContract::new(expected_positive, Box::new(MockAnalyzer::new(config.clone(), seed)));
    // `run()` renders the full feotest verdict block (rate, threshold, Wilson
    // bound, latency, baseline provenance, warnings) to stdout.
    let result = ProbabilisticTest::for_contract(contract)
        .inputs(panel)
        .approach(ThresholdApproach::SampleSizeFirst {
            samples: u32::try_from(panel.len()).expect("panel size fits u32"),
            confidence: 0.95,
        })
        .spec_resolver(SpecResolver::with_dir(dir))
        .threshold_origin(ThresholdOrigin::Empirical)
        .run();

    let record = result.verdict_record();
    let note = if record.warnings().iter().any(|w| w.code() == "COVARIATE_MISMATCH") {
        "  ⚠ baseline was measured for a different reagent lot — re-measure before trusting it"
    } else {
        ""
    };
    println!(">>> {label} → {:?}{note}", record.verdict());
}

// --- device configurations -------------------------------------------------

/// The validated, healthy instrument.
fn healthy() -> DeviceConfig {
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
/// measurement noise). This is the undeclared-change failure mode.
fn regressed() -> DeviceConfig {
    DeviceConfig {
        noise_sd: 0.22,
        ..healthy()
    }
}

/// A new reagent lot — a *declared* change: the covariate value differs from
/// the baseline's, so the verdict carries a covariate-mismatch warning.
fn new_lot() -> DeviceConfig {
    DeviceConfig {
        reagent_lot: "L77".to_owned(),
        bias: 0.04,
        ..healthy()
    }
}
