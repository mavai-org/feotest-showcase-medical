//! The field sentinel: a minimal, harness-free self-diagnosis agent.
//!
//! ```text
//! cargo run --bin sentinel
//! ```
//!
//! A real sentinel ships as its own small binary — separate from any dev/test
//! tooling — carrying a pre-validated baseline. It runs **verification only** (a
//! field device does not re-run validation): it compares the device's calls on
//! onboard control material against the embedded baseline and emits a verdict.
//!
//! Two things are deliberately **not feotest's job**, and are left to the device
//! manufacturer:
//!
//! * **Scheduling.** feotest provides a self-check that *can* be run; whether,
//!   when, and how often it runs is the manufacturer's decision. feotest does
//!   not schedule anything.
//! * **The response.** feotest emits a verdict; what to do with a failing one
//!   (warn, flag for service, withhold results, …) is the manufacturer's
//!   risk-management decision.
//!
//! This demo simply *invokes* the self-check several times in a row, with the
//! instrument's measurement noise creeping up (modelling reagent ageing), so a
//! drift is caught. The escalate-on-consecutive-failures logic below is an
//! **example** manufacturer response policy — not something feotest provides.

use std::path::Path;
use std::process::ExitCode;

use feotest::model::ThresholdOrigin;
use feotest::ptest::ProbabilisticTest;
use feotest::ptest::builder::ThresholdApproach;
use feotest::spec::SpecResolver;

use feotest_showcase_medical::contract::DiagnosticContract;
use feotest_showcase_medical::device::MockAnalyzer;
use feotest_showcase_medical::panel::{self, Case};
use feotest_showcase_medical::scenarios::{PANEL, drifting};

/// The pre-validated baseline shipped with the sentinel (committed, version-
/// controlled). In a real device it is embedded in the firmware at build time;
/// the runtime never re-validates and never reaches back to a source tree.
const BASELINE_DIR: &str = "field-baseline";

/// Onboard control specimens per self-check — small, like real QC material.
const N_CONTROL: usize = 100;
const SEED_SENTINEL: u64 = 0x55;

fn main() -> ExitCode {
    let baseline = Path::new(BASELINE_DIR);
    if !baseline.is_dir() {
        eprintln!(
            "Missing embedded baseline `{BASELINE_DIR}/` — the sentinel ships with its validated baseline."
        );
        return ExitCode::FAILURE;
    }

    let (positives, _negatives) = panel::split(panel::load(PANEL));
    let controls = &positives[..N_CONTROL.min(positives.len())];

    println!("feotest sentinel — in-field self-diagnosis (verification only)\n");
    println!(
        "Baseline embedded; {} onboard control specimens per self-check. feotest neither\n\
         schedules this nor dictates the response: if and when to run it, and how to act on\n\
         a failing verdict, are the manufacturer's decisions.\n",
        controls.len()
    );

    // The operator invokes the self-check on several occasions as the instrument
    // ages and its measurement noise creeps up.
    let checks: [(&str, f64); 4] = [
        ("self-check 1", 0.10),
        ("self-check 2", 0.12),
        ("self-check 3", 0.20),
        ("self-check 4", 0.24),
    ];

    let mut consecutive = 0u32;
    for (i, (label, noise)) in checks.iter().enumerate() {
        let healthy_check =
            self_diagnose(label, controls, *noise, SEED_SENTINEL + i as u64, baseline);
        // Example manufacturer response policy — NOT part of feotest. A single
        // failing check may be sampling noise; escalate on consecutive failures.
        if healthy_check {
            consecutive = 0;
        } else {
            consecutive += 1;
            if consecutive == 1 {
                println!(
                    "    ⚠ self-check FAILED — under the example policy, keep monitoring (one check may be noise)"
                );
            } else {
                println!(
                    "    ⚑ ALERT (example policy) — {consecutive} consecutive failures; flag the instrument for service"
                );
            }
        }
    }

    println!("\nEach self-check emits a verdict record — evidence the manufacturer retains.");
    ExitCode::SUCCESS
}

/// One self-check: verify the device against the embedded baseline on the
/// onboard control material, and report the verdict.
fn self_diagnose(
    label: &str,
    controls: &[Case],
    noise: f64,
    seed: u64,
    baseline_dir: &Path,
) -> bool {
    println!("\n── {label} ──");
    let contract =
        DiagnosticContract::new(true, Box::new(MockAnalyzer::new(drifting(noise), seed)));
    let result = ProbabilisticTest::for_contract(contract)
        .inputs(controls)
        .approach(ThresholdApproach::SampleSizeFirst {
            samples: u32::try_from(controls.len()).expect("control count fits u32"),
            confidence: 0.95,
        })
        .spec_resolver(SpecResolver::with_dir(baseline_dir))
        .threshold_origin(ThresholdOrigin::Empirical)
        .run();
    let record = result.verdict_record();
    println!(">>> {label} → {:?}", record.verdict());
    record.passed()
}
