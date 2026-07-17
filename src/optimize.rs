//! Choosing the assay protocol — the *first* act of the device lifecycle.
//!
//! Before a configuration can be validated it has to be *chosen*. This module
//! is the design/development step: an optimize experiment that tunes one
//! genuine instrument-configuration parameter and records what it tried and
//! what a cost-aware scorer preferred. It is **descriptive development work** —
//! it makes no inferential claim. The configuration it selects is then handed
//! to the measure experiment for *validation*; optimize never replaces it.
//!
//! ## The knob is a real configuration parameter — never the decision rule
//!
//! The tuned parameter is the **replicate count**: how many replicate assays
//! the instrument runs per specimen before averaging them into a single call.
//! More replicates average the analytical noise down, so more calls agree with
//! the reference panel — but each replicate costs reagent and turnaround time.
//! The optimum is therefore an honest interior trade: enough averaging to be
//! reliable, not so much that it is wasteful.
//!
//! The **calling threshold stays fixed** at the panel's truth boundary
//! throughout ([`CALLING_THRESHOLD`]). Accuracy is earned by *precision*, not
//! by relabelling. Tuning the threshold instead would be the anti-pattern:
//! truth is `severity >= 0.5`, so moving the calling line to maximise
//! agreement with the answer key just overfits the decision boundary to the
//! validation set — widening the goalposts until a goal is scored. The knob
//! must trade genuine qualities (precision versus cost); it must never be the
//! decision rule the validation later judges against.

use feotest::controls::Cost;
use feotest::criteria::{Criteria, Criterion};
use feotest::experiment::{
    ContractExecutionResult, FactorMutator, IterationRecord, OptimizeExperiment, Scorer,
};
use feotest::model::{ContractViolation, Defect};
use feotest::service_contract::ServiceContract;
use serde::Serialize;

use crate::device::{Device, DeviceConfig, MockAnalyzer, Reading};
use crate::panel::Case;

/// The stable identity of the optimization's service contract. The emitted
/// artefact lands under `optimizations/<this id>/`.
pub const CONTRACT_ID: &str = "diagnostics.tumour.assay-protocol";

/// The experiment identifier — the YAML filename stem of the emission.
pub const EXPERIMENT_ID: &str = "assay-protocol-tuning";

/// The instrument's fixed calling threshold: the panel's truth boundary,
/// never tuned. Truth is `severity >= 0.5`; the device calls positive when its
/// (replicate-averaged) measurement clears the same line. Holding this fixed is
/// what keeps the optimization honest — see the module docs.
pub const CALLING_THRESHOLD: f64 = 0.5;

/// The per-assay reagent penalty applied by the scorer, in units of pass-rate.
/// Each additional replicate must earn about this much accuracy to be worth its
/// reagent; set so the score peaks at a modest replicate count rather than at
/// the most accurate (and most expensive) protocol.
pub const COST_PER_ASSAY: f64 = 0.024;

/// The reproducible seed for the lot under evaluation. The optimization is
/// deterministic end to end: same seed, same panel, same trajectory on every
/// run, so the committed emission is reproducible.
const LOT_SEED: u64 = 42;

/// Samples drawn per iteration.
const SAMPLES_PER_ITERATION: u32 = 40;

/// How many replicate-count settings the sweep evaluates.
const MAX_ITERATIONS: u32 = 8;

/// The replicate count at the first iteration.
const INITIAL_REPLICATES: u32 = 1;

/// The step the sweep adds to the replicate count each iteration.
const REPLICATE_STEP: u32 = 2;

/// The candidate lot being characterised. Its analytical noise is what
/// replicate averaging buys down; the calling threshold is fixed and honest, so
/// accuracy is earned by precision, not by relabelling.
#[must_use]
pub fn lot_under_evaluation() -> DeviceConfig {
    DeviceConfig {
        software_version: "fw-1.2.0".to_owned(),
        reagent_lot: "L43-candidate".to_owned(),
        noise_sd: 0.28,
        bias: 0.0,
        qc_fail_rate: 0.02,
        latency_ms_mean: 4.0,
        latency_ms_sd: 1.5,
    }
}

/// The optimized knob: how many replicate assays the instrument averages per
/// specimen. A named, single-field configuration parameter — it serialises as
/// `factors: { replicateCount: N }`, a real device knob, never an opaque scalar
/// under the `factor` key.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssayProtocol {
    /// Replicate assays averaged per specimen before the call.
    pub replicate_count: u32,
}

impl std::fmt::Display for AssayProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}× replicate", self.replicate_count)
    }
}

/// The device run under a replicate protocol: it assays each specimen
/// `replicate_count` times, averages the measurements, and calls against the
/// fixed [`CALLING_THRESHOLD`]. Averaging shrinks the analytical noise, so more
/// calls agree with the panel as the replicate count rises.
pub struct ReplicatedCalling {
    analyzer: MockAnalyzer,
    replicate_count: u32,
}

impl ServiceContract for ReplicatedCalling {
    type Input = Case;
    type Output = String;

    fn id(&self) -> &'static str {
        CONTRACT_ID
    }

    fn description(&self) -> &str {
        "Assay-protocol operating point: replicate count vs reagent cost, fixed calling threshold"
    }

    fn invoke(&self, case: &Case, cost: &mut Cost) -> Result<String, Defect> {
        let mut sum = 0.0;
        let mut contributing = 0u32;
        for _ in 0..self.replicate_count {
            // Each replicate assay is one unit of reagent cost — recorded so the
            // artefact's stated cost carries the protocol's expense and the
            // cost-aware scorer can read it back deterministically.
            cost.record_tokens(1);
            if let Reading::Call { measurement, .. } = self.analyzer.analyse(case) {
                sum += measurement;
                contributing += 1;
            }
        }
        if contributing == 0 {
            return Ok("qc-invalid".to_owned());
        }
        let averaged = sum / f64::from(contributing);
        let call_positive = averaged >= CALLING_THRESHOLD;
        Ok(if call_positive == case.is_positive() {
            "correct".to_owned()
        } else if case.is_positive() {
            "missed-tumour".to_owned()
        } else {
            "false-positive".to_owned()
        })
    }

    fn criteria(&self) -> Criteria<String> {
        // One diagnostic criterion — does the call agree with the reference
        // panel? The *direction* of every disagreement travels into the
        // artefact's failure distribution as `missed-tumour` vs
        // `false-positive`, so each iteration records which way its errors fall
        // as the replicate count moves.
        Criteria::of([Criterion::meeting()
            .pass_rate(0.5)
            .name("correct-call")
            .satisfies(
                "call agrees with the reference panel",
                |outcome: &String| {
                    if outcome == "correct" {
                        Ok(())
                    } else {
                        Err(ContractViolation::new(
                            outcome.clone(),
                            "call disagrees with the reference panel",
                        ))
                    }
                },
            )
            .build()])
    }
}

/// Steps the replicate count up by [`REPLICATE_STEP`] each iteration: 1, 3, 5, …
pub struct ReplicateSweep;

impl FactorMutator<AssayProtocol> for ReplicateSweep {
    fn mutate(
        &self,
        current: &AssayProtocol,
        _history: &[IterationRecord<AssayProtocol>],
    ) -> AssayProtocol {
        AssayProtocol {
            replicate_count: current.replicate_count + REPLICATE_STEP,
        }
    }
}

/// A cost-aware scorer: observed accuracy minus a penalty proportional to the
/// mean reagent cost per specimen (assays run = tokens recorded). This is the
/// author's scoring function — a composition of two values the artefact already
/// *states*, the observed pass rate and the stated average cost — not a
/// framework statistic.
///
/// A [`Scorer`] sees only the [`ContractExecutionResult`], never the factor, so
/// cost-by-configuration must travel through *stated* cost (tokens here). That
/// is what keeps the run descriptive and deterministic.
///
/// Because the scorer is cost-aware the **score is not the pass rate**:
/// accuracy keeps rising with replicates while the score peaks where the
/// marginal accuracy no longer pays for the extra assay, then falls. The chosen
/// best is therefore demonstrably *not* the most accurate iteration.
pub struct AccuracyPerAssay {
    /// Pass-rate penalty charged per replicate assay per specimen.
    pub cost_per_assay: f64,
}

impl Scorer for AccuracyPerAssay {
    fn score(&self, result: &ContractExecutionResult) -> f64 {
        let accuracy = result.summary().observed_pass_rate();
        let assays_per_specimen = result.summary().cost().avg_tokens_per_sample();
        #[allow(
            clippy::cast_precision_loss,
            reason = "assays-per-specimen is a small count; f64 is exact well past the range here"
        )]
        let cost = self.cost_per_assay * assays_per_specimen as f64;
        accuracy - cost
    }

    fn name(&self) -> Option<&str> {
        Some("accuracy-per-assay")
    }
}

/// Runs the assay-protocol optimization over the reference panel and returns the
/// result. Deterministic: the lot seed, the panel, and the sweep are all fixed,
/// so the score trajectory is identical on every run.
#[must_use]
pub fn run(cases: &[Case]) -> feotest::experiment::OptimizeResult<AssayProtocol> {
    OptimizeExperiment::builder()
        .service_contract_id(CONTRACT_ID)
        .initial_factor(AssayProtocol {
            replicate_count: INITIAL_REPLICATES,
        })
        .service_contract(|protocol: &AssayProtocol| ReplicatedCalling {
            analyzer: MockAnalyzer::new(lot_under_evaluation(), LOT_SEED),
            replicate_count: protocol.replicate_count,
        })
        .scorer(AccuracyPerAssay {
            cost_per_assay: COST_PER_ASSAY,
        })
        .mutator(ReplicateSweep)
        .samples_per_iteration(SAMPLES_PER_ITERATION)
        .inputs(cases)
        .max_iterations(MAX_ITERATIONS)
        .no_improvement_window(MAX_ITERATIONS)
        .experiment_id(EXPERIMENT_ID)
        .build()
        .run()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::panel;
    use crate::scenarios::PANEL;

    /// The load-bearing pedagogic invariant: the cost-aware scorer's chosen
    /// best is *not* the most accurate protocol. Accuracy keeps rising with
    /// replicates, but the score peaks at a modest count because each replicate
    /// costs reagent — so a later, higher-accuracy iteration sits below the best
    /// by score. If this ever fails, the chapter's story no longer holds.
    #[test]
    fn best_by_score_is_not_the_most_accurate_protocol() {
        let cases = panel::load(PANEL);
        let result = run(&cases);

        let best = result.best_iteration().expect("a best iteration exists");
        let best_record = &result.history()[best as usize];
        let best_accuracy = best_record.successes();

        // Some iteration is strictly more accurate than the chosen best, yet
        // scores lower — the cost-aware scorer deliberately declined it.
        let more_accurate_but_lower_scoring = result
            .history()
            .iter()
            .any(|r| r.successes() > best_accuracy && r.score() < best_record.score());
        assert!(
            more_accurate_but_lower_scoring,
            "expected a higher-accuracy, lower-scoring protocol than the chosen best"
        );
    }

    /// The optimize run is deterministic in the quantities the story rests on:
    /// scores and counts are identical across runs (only wall-clock latency
    /// varies), so the committed emission's trajectory is reproducible.
    #[test]
    fn score_trajectory_is_deterministic() {
        let cases = panel::load(PANEL);
        let a = run(&cases);
        let b = run(&cases);

        assert_eq!(a.best_iteration(), b.best_iteration());
        let scores_a: Vec<f64> = a.history().iter().map(IterationRecord::score).collect();
        let scores_b: Vec<f64> = b.history().iter().map(IterationRecord::score).collect();
        assert_eq!(scores_a, scores_b);
    }
}
