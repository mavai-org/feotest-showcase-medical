# feotest showcase — a diagnostic device as a stochastic service

> **⚠ Early version — work in progress.** This is an early-stage showcase under
> active development. The device and reference panel are synthetic stand-ins
> (see below); APIs, structure, and content may change. It demonstrates a
> *methodology*, not a product, and is not a clinical result.

A worked, runnable example of using [feotest](https://github.com/mavai-org/feotest)
to make — and then *keep* — a statistically defensible claim about a medical
device's performance.

The "service under test" is a **physical diagnostic instrument behind an API**:
given a specimen it returns a call (tumour / normal) with analytical noise,
the occasional invalid/QC result, and a variable turnaround time. That is a
genuinely *stochastic* service — run the same specimen twice and the call can
differ — which is exactly what feotest is for. A frozen model scored once over
a frozen test set would have none of that variability; this does.

> **Illustrative, not a clinical result.** This demonstrates a *methodology*.
> The instrument is a stochastic mock; the reference panel is synthetic control
> material (see [`fixtures/README.md`](fixtures/README.md)). It makes no claim
> about any specific product or vendor.

## Run it — two entrypoints, one loop

The two operations are **explicit**, mirroring the real lifecycle:

```bash
cargo run -- measure   # experiment → baseline   ("how accurate is it?",   validation)
cargo run -- verify    # probabilistic test      ("does it still meet it?", verification)
```

- **`measure`** runs the measure experiment over the reference panel, derives
  the empirical baseline — sensitivity and specificity, each with a Wilson
  confidence floor, tagged with the device's covariate identity — and writes it
  to `baselines/`. You do this **once**, when you validate the device.
- **`verify`** runs the probabilistic test for the current device against that
  committed baseline and **exits non-zero on failure**, so it drops straight
  into a CI gate you re-run on every firmware build, reagent lot, or release. It
  refuses to run if no baseline exists — verification depends on validation
  having happened.

```bash
cargo run            # equivalently: cargo run -- demo
```

- **`demo`** (the default) runs the whole loop end-to-end in one process, so a
  fresh clone has the full story to look at, in four phases:
  1. **Characterise** — the measure experiment mints the baseline (validation).
  2. **Verify** a healthy device → **PASS** (verification).
  3. **Drift caught** — a *silently* degraded instrument (same declared config,
     more measurement noise) → **FAIL**, below the validated sensitivity floor:
     a regression the version number never advertised.
  4. **Covariate guard** — the same device with a **new reagent lot** → **PASS**
     with a `COVARIATE_MISMATCH` warning: the baseline was measured for a
     different lot, so it no longer applies as-is — re-measure before trusting it.

```bash
cargo run -- report
```

- **`report`** runs a measure → verify cycle and renders the resulting verdict
  as a standalone **HTML report** (`report.html`), using feotest's built-in
  report writer — the kind of durable artefact an auditee archives as a
  verification record. Requires `xsltproc` on `PATH` (feotest produces the
  report by XSLT over the verdict's XML interchange form).

## The two questions, the two tools

The showcase rests on a clean correspondence:

| Question | feotest tool | Lifecycle |
|---|---|---|
| *How accurate is the device?* | **Measure experiment** → empirical baseline | Validation |
| *Does it still meet its validated performance?* | **Probabilistic test** against that baseline | Verification |

They are two phases of **one loop** with a handoff: the experiment mints the
baseline artefact, the test consumes it. The verification answers
*drift-from-baseline*, not absolute accuracy re-derived — it is a
non-inferiority check, powered (via the sample size) for the degradation that
matters. The differentiator over a one-off study in a spreadsheet is that this
is **code**: run it on every firmware build, reagent lot, or software release
as an automated gate (lot-release, post-market surveillance under IVDR).

## What the contract asserts

A device spec is never one number, so neither is the contract. It is a
**covariate-scoped vector of criteria**, evaluated jointly on one sampling:

- **diagnostic** (sensitivity over the positive panel / specificity over the
  negative) — *empirical*: its floor is derived from the validated baseline, so
  it certifies conformance to validated performance, not a number plucked from
  the air;
- **valid-result** — *normative*: the device must return a usable call at least
  95% of the time (a fixed validity floor, not a drift metric);
- **latency** — a per-assay turnaround commitment at the 95th percentile.

A QC-fail is a *response*, judged by the criteria (it fails the diagnostic
criterion as a `no-result` and pulls down the validity rate); only a transport
failure — *no response at all* — is a defect that aborts the run. That is
feotest's `Result`/Outcome split, and it reads true to anyone who has
integrated an instrument.

## Covariates are baseline identity

`software_version` and `reagent_lot` are declared **covariates** — the versioned
identity a baseline is scoped to. A baseline measured under one profile is a
valid comparator only under the same profile; verify under a different reagent
lot and feotest raises `COVARIATE_MISMATCH` (phase 4). This is the guard
against the classic confound — *did the device degrade, or did the conditions
change?* — and it is why the verification half is honest rather than naive.

## The API seam — drop your instrument in

The contract drives the device through one trait:

```rust
pub trait Device {
    fn analyse(&self, case: &Case) -> Reading;   // a real adapter calls the instrument here
    fn config(&self) -> &DeviceConfig;
}
```

`MockAnalyzer` is a faithful stochastic stand-in. To certify a **real**
instrument, implement `Device` against its SDK / LIS / REST interface and drop
it in — the contract, the criteria, and the loop are unchanged.

## Honest caveats

- The device and panel are synthetic; the point is the *method*.
- A real panel's ground truth comes from a **reference standard**, which in
  deployment is the hard, expensive part this fixture stands in for.
- This **operationalises the statistics** a CLSI/IVDR performance study needs
  (the contract, the confidence floor, the feasibility check, the re-runnable
  gate). It does not replace the protocol or the reference standard.

## For auditors & auditees

Two companion documents in [`docs/`](docs/):

- [INFORMATION-FOR-AUDITORS.md](docs/INFORMATION-FOR-AUDITORS.md) — the
  methodology and the **verifiable statistical discipline** behind feotest: how
  its statistics are specified, independently implemented, and conformance-checked
  against the [Statistical Companion](https://r.mavai.org/statistical-companion.pdf)
  and the `mavai-R` oracle, so a verdict can be traced rather than taken on trust.
- [INFORMATION-FOR-AUDITEES.md](docs/INFORMATION-FOR-AUDITEES.md) — for the
  manufacturer being audited: the **evidence** feotest produces (baselines,
  verdicts, conformance, continuous verification) and which audit question each
  artefact helps answer.

Both carry an explicit disclaimer: feotest supplies evidence inputs, not
accreditation.

## Layout

```
src/device.rs    the Device seam + the stochastic MockAnalyzer
src/contract.rs  the ServiceContract: criteria vector, covariates, latency
src/panel.rs     the reference panel (committed ground truth)
src/main.rs      the CLI: `measure` / `verify` entrypoints + the `demo` loop
fixtures/        the reference panel + provenance (see its README)
scripts/         regenerate the reference panel
docs/            INFORMATION-FOR-AUDITORS.md / -AUDITEES.md — assurance & evidence
```

## License

Licensed under the [Apache License, Version 2.0](LICENSE). Contributions are
accepted under the same license and the
[Developer Certificate of Origin](dco.txt) — see [CONTRIBUTING.md](CONTRIBUTING.md).
