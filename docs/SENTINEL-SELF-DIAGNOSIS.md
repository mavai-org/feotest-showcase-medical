# Sentinel self-diagnosis — monitoring performance in the field

> **⚠ Early version — work in progress.** This document describes a *concept*
> and a demonstration of it (`cargo run --bin sentinel`). It is not a regulatory
> submission, a certification, or a clinical claim. See the disclaimer below.

A **sentinel** runs the *same* contract used to validate a device — against a
live deployment, without a test harness. Shipped *inside* the device it becomes
a built-in **self-diagnosis**: a probabilistic test that asks "does this
instrument still meet its validated performance?" and emits a verdict.

Two things are deliberately **not feotest's responsibility**, and belong to the
device manufacturer:

- **If and when it runs.** feotest provides a self-check that *can* be invoked;
  it does **not** schedule anything. How often the self-check runs — on power-up,
  before each batch, daily, on demand — is the manufacturer's decision.
- **How to respond to its verdict.** feotest *emits* a verdict; what to do with
  a failing one — warn, flag for service, withhold results, ignore as noise — is
  the manufacturer's risk-management decision.

feotest supplies the runnable self-check and the statistically-grounded verdict.
Scheduling and response are the device's, by design.

## The crux — what does it sample against?

A probabilistic test needs inputs whose correct answer is known, so it can judge
the device's calls. In the validation lab that is a characterised reference
panel. In the field, the device processes **live patient samples whose true
diagnosis is unknown** at the time — that is the whole reason it is running. You
cannot compute sensitivity/specificity on live cases.

So a field self-diagnosis samples against **onboard control material** —
specimens with established truth, carried by the device. That is not exotic: it
is how in-vitro-diagnostic devices already do **internal quality control** (QC
material, calibration verification, Levey-Jennings / Westgard monitoring). The
sentinel's contribution is to give that QC **statistical rigour and
continuity**: the same contract, the same verdict artefacts, and the same
covariate-scoped baseline as the design-validation run.

## What it monitors

The **verify** half of the measure → verify loop: *drift-from-baseline*, each
time the self-check is invoked, against the embedded baseline, scoped by the
field device's covariate profile (reagent lot, software/firmware version,
calibration state). A covariate change or a genuine degradation trips the
verdict. For an ML/AI component this is model-performance monitoring — what
post-market expectations for AI-enabled devices are moving toward.

## A separate, minimal binary

In this showcase the sentinel is its **own binary** (`cargo run --bin sentinel`,
`src/bin/sentinel.rs`) — as a real one would ship: separate from any dev/test
tooling, carrying a **pre-validated baseline** (committed under `field-baseline/`;
in a real device it is embedded in the firmware at build time and the runtime
never reaches back to a source tree). It runs **verification only** — a field
device does not re-run validation.

`cargo run --bin sentinel`:

1. Loads its embedded baseline and a small set of onboard control specimens
   (not the full validation panel — like real QC material).
2. Is *invoked* several times in a row (the demo stands in for the manufacturer
   choosing to run it); the instrument's measurement noise creeps up across the
   runs (modelling reagent ageing):
   - early checks **PASS** — the device still meets its validated floor;
   - a later check **FAILS** — emitted as a verdict;
   - the demo's **example response policy** treats one failure as possible
     sampling noise and escalates on a second consecutive failure to *flag the
     instrument for service*.

That escalation policy is an **illustration of a manufacturer response** — not
part of feotest. feotest's contribution stops at the per-check verdict.

## Two disciplines

- **Emit, don't act.** feotest *reports* a verdict; it never re-baselines itself
  or takes the device offline. Promotion of a new baseline, or any action on a
  failure, is the manufacturer's — a self-monitor that silently changed its own
  acceptance criteria, or downed a device on one noisy reading, would be a new
  hazard, not a feature.
- **Trend, not a single reading.** A small control run has limited statistical
  power, so one failing check may be noise. Sensible response policies look at a
  *trend* (e.g. consecutive failures) — but that logic, and its thresholds,
  belong to the manufacturer, layered on feotest's per-check verdict.

## The hard parts (the real gating work)

This is where "conceivable" meets "shippable":

- **It becomes safety-relevant software.** A dev tool on a laptop is one thing;
  firmware in a regulated device is another. The sentinel — and feotest itself —
  would be developed/qualified under the device's quality system and software
  safety class (IEC 62304), with the framework treated as software of unknown
  provenance. The open-source, independently-conformance-checked nature of
  feotest's statistical engine is a genuine asset to that argument (see
  [INFORMATION-FOR-AUDITORS.md](INFORMATION-FOR-AUDITORS.md)).
- **The response is the manufacturer's risk decision.** Because feotest does not
  act on a verdict, *what* a failing self-diagnosis triggers — and the sentinel's
  own false-positive / false-negative rate as a hazard — is risk management
  (ISO 14971) the manufacturer owns.
- **Embedded constraints.** "In the device" may mean limited, possibly
  safety-certified compute. Rust is a real asset here (no GC, deterministic);
  but a shipped sentinel would want a slimmed build of just the statistics and
  verdict core.

## Relationship to feotest's sentinel mechanism

feotest provides a harness-free sentinel runtime (a CLI with `measure` / `run` /
`check` subcommands) for exactly this deployment shape — but, again, *not* a
scheduler. This demo illustrates the self-diagnosis pattern using the showcase's
full, multi-criteria, covariate-scoped contract, so the field self-check is
demonstrably the same contract as the validation run.

## Disclaimer

This document and the accompanying showcase are provided for informational and
illustrative purposes only, on an "AS IS" basis without warranty of any kind, to
the fullest extent permitted by the [Apache License 2.0](../LICENSE). Nothing
here is legal, regulatory, or clinical advice. A built-in sentinel does not by
itself constitute or produce any accreditation, certification, conformity
assessment, or regulatory clearance (including under IEC 62304, ISO 14971,
ISO 13485, the EU IVDR/MDR, or FDA regulation); designing, verifying,
validating, scheduling, risk-assessing, and responding to such functionality —
and compliance with all applicable regulations and standards — rests entirely
with the device manufacturer. All data and devices in this showcase are
synthetic and illustrative.

## See also

- [INFORMATION-FOR-AUDITEES.md](INFORMATION-FOR-AUDITEES.md) — the evidence
  feotest produces, including continuous verification.
- [INFORMATION-FOR-AUDITORS.md](INFORMATION-FOR-AUDITORS.md) — the methodology
  and why the statistics are independently verifiable.
- **Statistical Companion:** <https://r.mavai.org/statistical-companion.pdf>
