//! The reference panel — the characterised control material the device is
//! tested against.
//!
//! This is the *known result* side of the contract: a set of specimens whose
//! ground-truth diagnosis is established (here, by a latent `severity`; a real
//! panel's truth comes from a reference standard, which in deployment is the
//! hard, expensive part). The panel is committed; the device's *responses* are
//! generated stochastically at run time — the two are deliberately separate.

use std::path::Path;

/// One reference specimen: a latent severity in `[0, 1]`. Truth is
/// `severity >= 0.5` (tumour-positive). The CSV carries a `case_id` column for
/// human reference; the contract judges only the severity.
#[derive(Clone, Debug)]
pub struct Case {
    pub severity: f64,
}

impl Case {
    #[must_use]
    pub fn is_positive(&self) -> bool {
        self.severity >= 0.5
    }
}

/// Loads the reference panel from a `case_id,severity` CSV (`#` lines and the
/// header are skipped).
///
/// # Panics
///
/// Panics if the file is missing or malformed — a broken panel is a defect in
/// the showcase, not a runtime condition.
#[must_use]
pub fn load(path: impl AsRef<Path>) -> Vec<Case> {
    let path = path.as_ref();
    let raw = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("cannot read panel {}: {e}", path.display()));

    raw.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#') && !l.starts_with("case_id"))
        .map(|line| {
            let (_id, severity) = line
                .split_once(',')
                .unwrap_or_else(|| panic!("expected `case_id,severity`: {line:?}"));
            Case {
                severity: severity
                    .trim()
                    .parse()
                    .unwrap_or_else(|_| panic!("bad severity: {line:?}")),
            }
        })
        .collect()
}

/// Splits the panel into the tumour-positive and normal populations. Sensitivity
/// is certified over the first; specificity over the second.
#[must_use]
pub fn split(cases: Vec<Case>) -> (Vec<Case>, Vec<Case>) {
    cases.into_iter().partition(Case::is_positive)
}
