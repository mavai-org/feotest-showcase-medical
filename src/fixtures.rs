//! Loads the frozen evaluation set from `fixtures/scores.csv`.
//!
//! The file is a plain CSV with a header row `patch_id,label,score,inference_ms`.
//! Lines beginning with `#` are provenance comments and are skipped. `label`
//! is `tumour` or `normal`. Parsing is intentionally dependency-free.

use std::path::Path;

use crate::pathology::Patch;

/// Reads and parses every patch in the evaluation set.
///
/// # Panics
///
/// Panics if the file is missing, unreadable, or malformed. A broken fixture
/// is a defect in the showcase, not a runtime condition to recover from.
#[must_use]
pub fn load(path: impl AsRef<Path>) -> Vec<Patch> {
    let path = path.as_ref();
    let raw = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("cannot read fixture {}: {e}", path.display()));

    raw.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .filter(|line| !line.starts_with("patch_id"))
        .map(parse_row)
        .collect()
}

/// Parses one `patch_id,label,score,inference_ms` row.
fn parse_row(line: &str) -> Patch {
    let fields: Vec<&str> = line.split(',').map(str::trim).collect();
    assert!(
        fields.len() == 4,
        "expected 4 fields (patch_id,label,score,inference_ms), got {}: {line:?}",
        fields.len()
    );
    let is_tumour = match fields[1] {
        "tumour" => true,
        "normal" => false,
        other => panic!("unknown label {other:?} (expected tumour|normal): {line:?}"),
    };
    Patch {
        is_tumour,
        score: fields[2]
            .parse()
            .unwrap_or_else(|_| panic!("bad score in row: {line:?}")),
        inference_ms: fields[3]
            .parse()
            .unwrap_or_else(|_| panic!("bad inference_ms in row: {line:?}")),
    }
}
