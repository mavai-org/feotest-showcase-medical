# Information for medical-device auditors

> **⚠ Early version — work in progress.** This document accompanies an
> early-stage showcase. It explains a *methodology* and the assurance model
> behind the [feotest](https://github.com/mavai-org/feotest) framework; it is
> not a regulatory submission, a certification, or a clinical claim.

## Who this is for

Auditors, assessors, and verification-and-validation (V&V) reviewers examining a
device whose software uses **feotest** to state and re-check a performance
claim. It explains, in audit terms, what the framework does, and — the part
that matters most to an auditor — *why its statistical results can be
independently verified rather than taken on the vendor's word*.

## What feotest does, and does not, claim

feotest **operationalises the statistics** of a performance claim: it expresses
the claim as executable contract code, computes the inference, and emits a
reproducible verdict with full provenance. It does **not**:

- replace the study protocol (e.g. the applicable CLSI procedure),
- supply the **reference standard** (the source of ground truth — in
  deployment, the hard and decisive part), or
- constitute clinical validation or regulatory clearance.

Read every claim below within that boundary. feotest is evidence-machinery, not
a substitute for the regulated process around it.

## The methodology, in audit terms

The framework separates the two questions a regulated team must answer, and maps
them onto two distinct, traceable operations:

| Question | Operation | Lifecycle phase |
|---|---|---|
| *How accurate is the device?* | a **measure experiment** that derives an empirical **baseline** | validation |
| *Does it still meet its validated performance?* | a **probabilistic test** against that baseline | verification |

Four properties make the result audit-grade:

- **Confidence, not point estimates.** A performance figure is reported with a
  Wilson score confidence bound; the verification verdict turns on that bound,
  not on an unqualified observed rate.
- **Feasibility is stated, not assumed.** Before sampling, the framework checks
  whether the configured sample size can support the claim at the required
  confidence. An under-powered test is flagged, not silently passed.
- **Covariate scoping prevents confounding.** The baseline is tagged with the
  conditions it was measured under (e.g. software version, reagent lot). A
  verification run under different conditions raises an explicit mismatch
  signal — so "did the device degrade?" is never confused with "did the
  conditions change?".
- **The contract is multi-dimensional.** Performance is asserted as a vector of
  named criteria (e.g. diagnostic sensitivity, specificity, valid-result rate)
  plus a latency commitment, evaluated jointly on one sampling — the device's
  performance specification expressed as code.

## Why the statistics are verifiable, not vendor-asserted

This is the core assurance argument. feotest's statistical correctness is not a
claim the framework makes about itself. It is established by a **closed loop**
with an independent oracle:

1. **A canonical specification.** Every formula the framework uses — Wilson
   interval construction, threshold derivation, feasibility checking,
   latency-percentile bounds, verdict evaluation — is defined and *justified* in
   a single, language-agnostic methodology document, the **Statistical
   Companion** (<https://r.mavai.org/statistical-companion.pdf>).
   The Companion is the authority: where an implementation and the Companion
   disagree, the Companion wins.
2. **An independent implementation (the oracle).** Those formulae are computed a
   *second time*, in **R**, against established, peer-reviewed statistical
   packages — the `mavai-R` oracle. This is a deliberately separate
   implementation in a different language: a common bug would have to occur
   identically in both to go undetected.
3. **Vendored reference fixtures.** The oracle's results — per-topic
   `(inputs, expected)` cases at floating-point precision — are vendored into
   the **open-source feotest repository** (`tests/conformance/`), pinned per
   release, so they are publicly inspectable and the check runs offline. (The
   oracle itself, `mavai-R`, is not an open-source project; it is the published
   methodology and these vendored reference values, not the oracle's source,
   that an auditor inspects.)
4. **Automated conformance.** feotest carries a conformance test
   (`tests/conformance.rs`) that loads those fixtures and asserts agreement to a
   stated tolerance (typically 1e-6). **A green conformance test means the
   framework agrees, numerically, with the independent oracle.** A red one means
   either the framework has drifted or the oracle has surfaced a defect — both
   are first-class, investigated outcomes, not silent failures.

For an auditor, the consequence is direct: the statistics are *specified
independently, implemented independently, and cross-checked automatically*. You
do not have to trust the framework's arithmetic — you can trace it.

## What you can independently verify

Concrete checks an auditor can perform without privileged access:

- **Read the methodology.** The Statistical Companion states every formula and
  the rationale for each choice (e.g. why a one-sided Wilson *lower* bound is
  used for degradation tests).
- **Read the source.** feotest is an open-source project (Apache-2.0),
  deliberately architected so its statistical engine is easy to read and audit:
  the statistics live in a single, isolated module — kept transparent and
  reviewable rather than buried inside a third-party black box — and implement
  standard, well-established methods (Wilson score intervals, non-parametric
  percentile bounds) on a widely-used numerical library, the same methods the R
  oracle computes independently.
- **Inspect the reference fixtures.** The oracle's `(inputs, expected)` cases
  are vendored — human-readable and pinned — in the open-source feotest
  repository (`tests/conformance/`); no privileged access is required.
- **Run the conformance test.** `cargo test --test conformance` in feotest
  re-checks the implementation against the oracle on your own machine.
- **Inspect a baseline artefact.** A committed baseline records its provenance:
  sample count, confidence level, the derived minimum pass rate (Wilson lower
  bound), the covariate profile it was measured under, and a content fingerprint
  for tamper detection.
- **Re-run the verification.** Against a committed baseline, the verdict is
  reproducible, and it carries enough metadata (threshold origin, baseline
  reference, covariate profile, sample count, confidence, observed rate, Wilson
  lower bound) to reconstruct how it was reached.

More than permitting this scrutiny, the mavai project actively **invites** it.
feotest is open source precisely so that qualified parties can read its
statistical engine — kept small, isolated, and built on standard, well-
established methods — and challenge any aspect: the model, the methodology, the
implementation, or the conformance fixtures. A finding that surfaces a genuine
defect in the framework *or* in the oracle is welcomed as a first-class outcome
of the method, not a failure to be hidden.

Every statistical behaviour is traceable end to end:

```
implementation code  →  requirements catalog  →  Statistical Companion  →  oracle fixtures
   (feotest)              (what must hold)          (the formula + why)       (independent expected values)
```

A catalog requirement references the Companion section it realises; the
implementation is derived from the catalog; the conformance test validates the
implementation against the oracle. No statistical claim originates in the code
without a path back to the specification and an independent check.

## Limitations and honest scope

- The accuracy of any verdict is bounded by the **reference standard** supplying
  ground truth. feotest assumes you have a characterised reference; establishing
  it is the operator's responsibility and, in practice, the expensive part.
- A verification test that *passes* is a non-inferiority result: it should be
  powered for the smallest degradation that matters. "Did not detect drift" is
  not "proven equivalent" unless the test was adequately sized — which the
  feasibility check is there to make explicit.
- This showcase ships **synthetic** stand-ins (a mock instrument and a synthetic
  reference panel) to demonstrate the method. No figure in it is a real
  performance result.

## Disclaimer

This document and the accompanying showcase are provided for informational and
illustrative purposes only, on an "AS IS" basis without warranty of any kind, to
the fullest extent permitted by the [Apache License 2.0](../LICENSE) under which
this project is distributed. Nothing here is legal, regulatory, or clinical
advice.

feotest and this showcase do not constitute, and do not by themselves produce,
any accreditation, certification, conformity assessment, or regulatory clearance
(including under IEC 62304, ISO 13485, the EU IVDR/MDR, or FDA regulation).
Responsibility for the validity of any performance claim, for the reference
standard behind it, and for compliance with all applicable regulations and
standards rests entirely with the device manufacturer or operator. Use of
feotest does not guarantee acceptance of any result by an auditor, notified
body, or regulator.

All data, devices, and figures in this showcase are synthetic and illustrative
and must not be cited as real performance results.

## References

- **Statistical Companion** — the canonical methodology:
  <https://r.mavai.org/statistical-companion.pdf>
- **mavai-R** — the statistical oracle (home: <https://r.mavai.org>). Not an
  open-source project; its reference fixtures are vendored, and publicly
  inspectable, in the feotest repository under `tests/conformance/`.
- **feotest** — the open-source framework, its statistical engine, and its
  conformance test: <https://github.com/mavai-org/feotest>
