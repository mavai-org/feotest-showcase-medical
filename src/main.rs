//! Runs the digital-pathology performance contracts against the frozen
//! evaluation set and reports a per-contract statistical verdict.
//!
//! ```text
//! cargo run            # certify against fixtures/scores.csv
//! ```
//!
//! Exit code is `0` only if every contract passes — so this binary drops
//! straight into a CI gate or a release-evidence pipeline.

mod fixtures;
mod pathology;

use std::process::ExitCode;
use std::time::Duration;

use feotest::ptest::ProbabilisticTest;
use feotest::service_contract::ServiceContract;

use crate::pathology::{DiagnosticContract, Patch};

/// The model's decision threshold: probability at or above which a patch is
/// called *tumour*. Frozen alongside the scores it is applied to.
const DECISION_THRESHOLD: f64 = 0.5;

/// Clinical floors the model must clear, with statistical confidence.
const SENSITIVITY_FLOOR: f64 = 0.95;
const SPECIFICITY_FLOOR: f64 = 0.90;

/// Per-patch inference-latency ceiling at the 95th percentile.
const P95_LATENCY: Duration = Duration::from_millis(40);

fn main() -> ExitCode {
    let patches = fixtures::load("fixtures/scores.csv");
    let (tumour, normal): (Vec<Patch>, Vec<Patch>) =
        patches.into_iter().partition(|p| p.is_tumour);

    assert!(
        !tumour.is_empty() && !normal.is_empty(),
        "evaluation set must contain both tumour and normal patches"
    );

    println!(
        "Digital-pathology tumour detection — certifying {} tumour / {} normal patches\n",
        tumour.len(),
        normal.len()
    );

    let sensitivity_ok = certify(
        DiagnosticContract::sensitivity(DECISION_THRESHOLD, SENSITIVITY_FLOOR, P95_LATENCY),
        &tumour,
    );
    let specificity_ok = certify(
        DiagnosticContract::specificity(DECISION_THRESHOLD, SPECIFICITY_FLOOR, P95_LATENCY),
        &normal,
    );

    if sensitivity_ok && specificity_ok {
        println!("\nVERDICT: PASS — every clinical contract is met on the evaluation set.");
        ExitCode::SUCCESS
    } else {
        println!("\nVERDICT: FAIL — at least one clinical contract is not met.");
        ExitCode::FAILURE
    }
}

/// Runs one contract over its population and prints its verdict. Returns
/// whether the contract passed (functional criteria *and* latency).
fn certify(contract: DiagnosticContract, population: &[Patch]) -> bool {
    let id = contract.id().to_owned();
    let result = ProbabilisticTest::for_contract(contract)
        .inputs(population)
        .samples(population.len() as u32)
        .run();

    let record = result.verdict_record();
    println!("[{id}]");

    for row in record.functional_assessment().criteria() {
        let passed = row.total() - row.fail();
        println!(
            "  {:<12} {:>4}/{:<4} passed   →  {:?}",
            row.name(),
            passed,
            row.total(),
            record.verdict()
        );
    }

    if let Some(latency) = record.latency() {
        println!(
            "  {:<12} p95 ≤ {:?}   →  {}",
            "latency",
            P95_LATENCY,
            if latency.passed() { "PASS" } else { "FAIL" }
        );
    }

    record.passed()
}
