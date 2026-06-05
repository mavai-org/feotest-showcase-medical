//! The service-under-test and its clinical performance contracts.
//!
//! The "service" here is a tumour-detection model for digital-pathology
//! patches. Each invocation classifies one histopathology patch as *tumour*
//! or *normal*. The model itself is not run at test time: its scored outputs
//! are read from a frozen evaluation set (`fixtures/scores.csv`), so the
//! contract is fully reproducible and the verdict depends only on committed
//! data — exactly what a regulatory dossier wants to be able to re-run.
//!
//! Two contracts express the clinical commitment, because the two claims are
//! made over two different populations:
//!
//! * **Sensitivity** — over the tumour-positive patches, the fraction the
//!   model correctly flags must clear a normative floor (missing a tumour is
//!   the costly error).
//! * **Specificity** — over the normal patches, the fraction the model
//!   correctly clears must hold its own floor (controls the false-alarm
//!   burden on the pathologist).
//!
//! Both contracts also carry a per-patch latency commitment: the recorded
//! inference time of each patch is *replayed* on the invocation path, so the
//! latency dimension reflects the model's measured profile rather than the
//! cost of a fixture lookup.

use std::time::Duration;

use feotest::controls::Cost;
use feotest::criteria::{Criteria, Criterion};
use feotest::latency::{LatencyCriterion, Percentile};
use feotest::model::{ContractViolation, Defect};
use feotest::service_contract::ServiceContract;

/// One scored evaluation patch: the model's probability that the patch shows
/// tumour, the ground-truth label, and the recorded inference latency.
#[derive(Clone, Debug)]
pub struct Patch {
    pub is_tumour: bool,
    pub score: f64,
    pub inference_ms: f64,
}

/// A clinical performance contract over one population of patches.
///
/// One struct expresses both the sensitivity and the specificity contract:
/// the only thing that differs is which prediction is correct for the
/// population under test (`expected`) and the floor that prediction rate must
/// clear (`pass_rate`).
pub struct DiagnosticContract {
    id: &'static str,
    description: &'static str,
    /// Probability at or above which the model is taken to predict *tumour*.
    decision_threshold: f64,
    /// The prediction a correct model must produce for every patch in this
    /// population: `true` (tumour) for the sensitivity contract, `false`
    /// (normal) for the specificity contract.
    expected: bool,
    /// The normative pass-rate floor this population must clear, with
    /// statistical confidence, for the contract to pass.
    pass_rate: f64,
    /// Per-patch latency commitment: a percentile and its ceiling.
    latency_budget: (Percentile, Duration),
}

impl DiagnosticContract {
    /// The sensitivity contract: over tumour patches, correctly flag at least
    /// `floor` of them (with confidence). `p95_latency` bounds the 95th
    /// percentile of per-patch inference time.
    #[must_use]
    pub fn sensitivity(decision_threshold: f64, floor: f64, p95_latency: Duration) -> Self {
        Self {
            id: "pathology.tumour.sensitivity",
            description: "Tumour-detection sensitivity over the positive evaluation set",
            decision_threshold,
            expected: true,
            pass_rate: floor,
            latency_budget: (Percentile::P95, p95_latency),
        }
    }

    /// The specificity contract: over normal patches, correctly clear at least
    /// `floor` of them (with confidence).
    #[must_use]
    pub fn specificity(decision_threshold: f64, floor: f64, p95_latency: Duration) -> Self {
        Self {
            id: "pathology.tumour.specificity",
            description: "Tumour-detection specificity over the negative evaluation set",
            decision_threshold,
            expected: false,
            pass_rate: floor,
            latency_budget: (Percentile::P95, p95_latency),
        }
    }
}

impl ServiceContract for DiagnosticContract {
    type Input = Patch;
    type Output = bool;

    fn id(&self) -> &str {
        self.id
    }

    fn description(&self) -> &str {
        self.description
    }

    fn invoke(&self, patch: &Patch, cost: &mut Cost) -> Result<bool, Defect> {
        // Replay the recorded inference latency so the latency dimension
        // reflects the model's measured profile, not fixture-lookup time.
        std::thread::sleep(Duration::from_secs_f64(patch.inference_ms / 1000.0));
        cost.record_tokens(1);
        Ok(patch.score >= self.decision_threshold)
    }

    fn criteria(&self) -> Criteria<bool> {
        let expected = self.expected;
        let (check, violated, detail) = if expected {
            ("tumour detected", "missed", "tumour patch not flagged")
        } else {
            ("normal cleared", "false-alarm", "normal patch flagged as tumour")
        };
        Criteria::of([Criterion::meeting()
            .pass_rate(self.pass_rate)
            .name(if expected { "sensitivity" } else { "specificity" })
            .satisfies(check, move |predicted: &bool| {
                if *predicted == expected {
                    Ok(())
                } else {
                    Err(ContractViolation::new(violated, detail))
                }
            })
            .build()])
    }

    fn latency(&self) -> Option<LatencyCriterion> {
        let (percentile, ceiling) = self.latency_budget;
        Some(LatencyCriterion::meeting().at_most(percentile, ceiling))
    }
}
