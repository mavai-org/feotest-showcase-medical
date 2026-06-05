//! The diagnostic performance contract over one population of the panel.
//!
//! One struct expresses both the sensitivity contract (over the tumour-positive
//! population) and the specificity contract (over the normal population) — the
//! only difference is which call is correct for the population under test.
//!
//! The contract is multi-criteria and covariate-scoped:
//! - a **diagnostic** criterion (sensitivity / specificity) — does the device's
//!   call agree with the reference truth?
//! - a **valid-result** criterion — did the device return a usable call at all,
//!   rather than a QC-fail?
//! - a **latency** commitment on per-assay turnaround time.
//!
//! Both pass-rate criteria are `empirical()`: their targets are derived from the
//! measured baseline, so the contract certifies *conformance to validated
//! performance*, not a number plucked from the air. The covariates
//! (`software_version`, `reagent_lot`) make the baseline scoped: a verdict run
//! under a different reagent lot resolves the baseline with a mismatch warning,
//! because performance is only comparable like-for-like.

use std::time::Duration;

use feotest::controls::Cost;
use feotest::criteria::{Criteria, Criterion};
use feotest::latency::{LatencyCriterion, Percentile};
use feotest::model::{ContractViolation, Defect};
use feotest::service_contract::{CovariateCategory, CovariateDeclaration, ServiceContract};
use feotest::spec::namer::CovariateProfile;

use crate::device::{Device, DeviceConfig, Reading};
use crate::panel::Case;

/// Per-assay turnaround-time ceiling enforced at the 95th percentile.
const P95_LATENCY: Duration = Duration::from_millis(25);

/// A clinical performance contract over one population, driving a device
/// through the API seam.
pub struct DiagnosticContract {
    expected_positive: bool,
    device: Box<dyn Device>,
}

impl DiagnosticContract {
    /// Builds the contract for a population. `expected_positive` is the call a
    /// correct device must produce for every specimen in scope: `true` over the
    /// tumour-positive panel (yielding sensitivity), `false` over the normal
    /// panel (yielding specificity).
    #[must_use]
    pub fn new(expected_positive: bool, device: Box<dyn Device>) -> Self {
        Self {
            expected_positive,
            device,
        }
    }

    /// The stable contract identity. A baseline is addressed by this id, so it
    /// must be identical across measurement and every verification.
    #[must_use]
    pub const fn id_for(expected_positive: bool) -> &'static str {
        if expected_positive {
            "diagnostics.tumour.sensitivity"
        } else {
            "diagnostics.tumour.specificity"
        }
    }
}

/// The covariate profile for a device configuration — its baseline identity.
#[must_use]
pub fn covariate_profile(config: &DeviceConfig) -> CovariateProfile {
    CovariateProfile::builder()
        .put("software_version", config.software_version.as_str())
        .put("reagent_lot", config.reagent_lot.as_str())
        .build()
}

/// The covariate key names, in declaration order.
#[must_use]
pub fn covariate_keys() -> Vec<String> {
    vec!["software_version".to_owned(), "reagent_lot".to_owned()]
}

impl ServiceContract for DiagnosticContract {
    type Input = Case;
    type Output = Reading;

    fn id(&self) -> &str {
        Self::id_for(self.expected_positive)
    }

    fn description(&self) -> &str {
        if self.expected_positive {
            "Tumour-detection sensitivity over the positive reference panel"
        } else {
            "Tumour-detection specificity over the normal reference panel"
        }
    }

    fn covariates(&self) -> Vec<CovariateDeclaration> {
        vec![
            CovariateDeclaration::new("software_version", CovariateCategory::ExternalDependency),
            CovariateDeclaration::new("reagent_lot", CovariateCategory::ExternalDependency),
        ]
    }

    fn resolve_covariates(&self) -> CovariateProfile {
        covariate_profile(self.device.config())
    }

    fn invoke(&self, case: &Case, cost: &mut Cost) -> Result<Reading, Defect> {
        // A real adapter would return Err(Defect) on a transport failure (no
        // response at all); a QC-fail is a *response*, judged by the criteria.
        cost.record_tokens(1);
        Ok(self.device.analyse(case))
    }

    fn criteria(&self) -> Criteria<Reading> {
        let expected = self.expected_positive;
        let diagnostic_name = if expected { "sensitivity" } else { "specificity" };

        Criteria::of([
            Criterion::empirical()
                .pass_rate()
                .name(diagnostic_name)
                .satisfies("call agrees with reference", move |r: &Reading| match r {
                    Reading::Call { positive, .. } if *positive == expected => Ok(()),
                    Reading::Call { measurement, .. } => Err(ContractViolation::new(
                        "misclassified",
                        format!("device call disagrees with reference (measurement {measurement:.3})"),
                    )),
                    Reading::QcFail { .. } => Err(ContractViolation::new(
                        "no-result",
                        "device returned no valid call",
                    )),
                })
                .build(),
            Criterion::meeting()
                .pass_rate(0.95)
                .name("valid-result")
                .satisfies("device returned a usable call", |r: &Reading| match r {
                    Reading::Call { .. } => Ok(()),
                    Reading::QcFail { reason } => {
                        Err(ContractViolation::new("qc-fail", reason.clone()))
                    }
                })
                .build(),
        ])
    }

    fn latency(&self) -> Option<LatencyCriterion> {
        Some(LatencyCriterion::meeting().at_most(Percentile::P95, P95_LATENCY))
    }
}
