# Information for auditees

> **⚠ Early version — work in progress.** This document accompanies an
> early-stage showcase. It catalogues the evidence [feotest](https://github.com/mavai-org/feotest)
> can produce; it is not a regulatory submission, a certification, or a clinical
> claim. It is the companion to
> [INFORMATION-FOR-AUDITORS.md](INFORMATION-FOR-AUDITORS.md).

## Who this is for

The **auditee** — the manufacturer or operator of a device whose software uses
feotest, preparing for accreditation or audit. It catalogues the concrete data
feotest produces that you can present as **evidence inputs** to your technical
documentation and quality records, and indicates which audit questions each
artefact helps answer.

## What this is, and is not

feotest emits durable, machine-readable artefacts that can serve as **evidence
inputs**. It does **not** produce accreditation, and it does not relieve you of
the study protocol, the **reference standard** (your source of ground truth), or
clinical evaluation. The artefacts below strengthen and make *reproducible* the
evidence you must already be generating — they do not replace the regulated
process around it.

## The artefacts feotest produces

### 1. The baseline — a validation record

A *measure experiment* derives an empirical **baseline**, written as a
version-controllable file. It captures, for a stated device configuration:

- observed performance per criterion (e.g. sensitivity, specificity) and the
  raw pass/fail counts;
- the **derived minimum pass rate** (the Wilson lower confidence bound) — the
  floor later verification is held to;
- the **covariate profile** it was measured under (e.g. software version,
  reagent lot) — the conditions the claim is scoped to;
- the latency distribution (percentiles and the underlying order statistics);
- sample size, generation timestamp, and an explicit validity window;
- an **integrity fingerprint** (tamper-evident) and an identity footprint.

*Evidences:* "the device was characterised at this performance, with this
confidence, under these conditions, on this date, over this many samples" — a
self-describing, reproducible validation record.

### 2. The verdict — a verification record

A *probabilistic test* against a baseline produces a **verdict** carrying:

- the decision (pass / fail / inconclusive);
- the statistical evidence behind it (observed rate, Wilson lower bound, sample
  count, confidence level, the threshold and where it came from);
- the covariate profile of the run, and any **warnings** (e.g. a covariate
  mismatch flagging that the baseline no longer applies as-is);
- the latency assessment.

The verdict is **serialisable to a cross-language, schema-validated XML
interchange format** (and renderable as a human-readable report), so it can be
archived as a durable record and re-read by other tools. Against a committed
baseline the verdict is **reproducible** and carries enough provenance to
reconstruct exactly how it was reached.

*Evidences:* objective, dated, reproducible verification that the device still
meets its validated performance — the record you re-generate on each release,
reagent lot, or firmware change.

### 3. Conformance — tool-validation evidence

feotest's statistical engine is continuously cross-checked against an
independent statistical oracle (`mavai-R`) via a conformance test that asserts
numerical agreement to a stated tolerance. A green conformance result is
evidence that the *tool's* computations are validated against an independent
reference — material to a tool-qualification or software-of-unknown-provenance
argument. (See INFORMATION-FOR-AUDITORS.md for how this loop works.)

### 4. Continuous verification — post-market evidence

The same contract can be run against a *live* system on a schedule, without a
test harness, emitting verdicts to configured sinks (e.g. a log or webhook).
This produces an ongoing stream of verification records suitable as
post-market-surveillance / performance-monitoring evidence. Taken to its
conclusion — a sentinel *inside* the device running periodic self-diagnosis on
onboard control material — this is in-field continuous verification; see
[SENTINEL-SELF-DIAGNOSIS.md](SENTINEL-SELF-DIAGNOSIS.md).

### 5. The contract as code — a controlled specification

The acceptance criteria are expressed as an executable, version-controlled
contract: named criteria (sensitivity, specificity, valid-result rate, …) plus
a latency commitment. This is the device's performance specification under
change control — every change to "what must hold" is diffable and traceable.

### 6. Failure taxonomy — investigation inputs

When a criterion fails, the verdict records *why*, as a named failure
distribution (which check failed, and how often). This feeds investigation /
corrective-action records rather than leaving a bare pass/fail.

## How the artefacts map to common audit needs

> Indicative only — feotest *supports* these activities with evidence; it does
> not by itself *satisfy* any regulatory requirement.

| Audit need | Artefact(s) that help |
|---|---|
| Analytical / clinical performance with confidence | Baseline (1), Verdict (2) |
| Ongoing verification on change (lot, firmware, release) | Verdict (2) re-run; Continuous verification (4) |
| Post-market surveillance / performance monitoring | Continuous verification (4) |
| Software verification records (lifecycle) | Verdict (2), the re-runnable gate |
| Tool qualification / SOUP confidence | Conformance (3) |
| Specification & change control | Contract as code (5) |
| Traceability of a result to its derivation | Provenance carried by Baseline (1) & Verdict (2) |
| Corrective action / investigation | Failure taxonomy (6) |

## Reproducibility and integrity

- A committed baseline plus a deterministic verification run is a **re-runnable
  gate** — you can demonstrate the verification on demand, not just present a
  past result.
- The baseline's integrity fingerprint makes tampering detectable on load.
- Each verdict carries threshold origin, baseline reference, covariate profile,
  sample count, confidence, observed rate, and Wilson bound — enough to
  reconstruct the derivation independently.

## What this showcase demonstrates

Running `cargo run -- measure` produces baseline artefacts; `cargo run -- verify`
produces verdicts. The XML/report emission, continuous (sentinel) operation, and
archival of these artefacts as controlled records are feotest capabilities a
real deployment would wire in; this early showcase demonstrates the underlying
measure → verify evidence, not a full records system.

## Limitations and honest scope

- Evidence inputs are only as sound as the **reference standard** behind them;
  that remains your responsibility.
- A passing verification is a non-inferiority result and must be adequately
  powered for the degradation that matters; the feasibility check makes the
  sizing explicit, but the judgement of "what matters" is yours.
- This showcase ships **synthetic** stand-ins; no figure in it is a real
  performance result.

## Disclaimer

This document and the accompanying showcase are provided for informational and
illustrative purposes only, on an "AS IS" basis without warranty of any kind, to
the fullest extent permitted by the [Apache License 2.0](../LICENSE) under which
this project is distributed. Nothing here is legal, regulatory, or clinical
advice.

feotest and this showcase do not constitute, and do not by themselves produce,
any accreditation, certification, conformity assessment, or regulatory clearance
(including under IEC 62304, ISO 13485, the EU IVDR/MDR, or FDA regulation). The
artefacts described are potential **evidence inputs** only; responsibility for
the validity of any performance claim, for the reference standard behind it, and
for compliance with all applicable regulations and standards rests entirely with
the device manufacturer or operator. Use of feotest does not guarantee
acceptance of any result by an auditor, notified body, or regulator.

All data, devices, and figures in this showcase are synthetic and illustrative
and must not be cited as real performance results.

## See also

- [INFORMATION-FOR-AUDITORS.md](INFORMATION-FOR-AUDITORS.md) — the complementary
  document: the methodology and why the statistics are independently verifiable.
- **Statistical Companion:**
  <https://r.mavai.org/statistical-companion.pdf>
- **feotest:** <https://github.com/mavai-org/feotest>
