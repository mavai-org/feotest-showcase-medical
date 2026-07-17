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

```bash
cargo run -- optimize
```

- **`optimize`** runs the *first* act of the lifecycle: **choosing** the
  configuration to validate. It sweeps a genuine instrument-configuration knob
  (the assay's **replicate count**), scores each candidate with a cost-aware
  scorer, and writes the canonical `mavai-optimize-1` artefact under
  `optimizations/`. This is **design/development work** — descriptive, not
  inferential — and the emission is the *"why this configuration"* record an
  auditor asks for. See [Choosing the assay protocol](#choosing-the-assay-protocol-the-first-act)
  below. Render it with the public `mavai` binary:
  `mavai optimize optimizations -o optimize-report.html`.

```bash
cargo run --bin sentinel
```

- **`sentinel`** — a **separate binary**, as a real field agent would ship —
  demonstrates **in-field self-diagnosis**: the *same* contract that validated
  the device runs, **verification only**, against **onboard control material**
  (not live patient samples, whose truth is unknown), comparing the device to a
  baseline embedded with the binary (`field-baseline/`). It is *invoked*
  repeatedly as the instrument drifts (reagent ageing) until the self-check
  fails. feotest **neither schedules the self-check nor dictates the response** —
  if and when to run it, and how to act on a failing verdict, are the
  manufacturer's decisions (the demo's consecutive-failure alert is an *example*
  policy, not part of feotest). See
  [`docs/SENTINEL-SELF-DIAGNOSIS.md`](docs/SENTINEL-SELF-DIAGNOSIS.md).

> **`field-baseline/` provenance.** The committed baseline is the *validated
> reference* the sentinel ships with — frozen and version-controlled, as an
> embedded firmware baseline would be, and deliberately distinct from the
> gitignored `baselines/` (transient runtime output). It is a generated
> artefact: regenerate it with `cargo run -- measure` and copy the resulting
> `diagnostics_tumour_sensitivity-*.yaml` into `field-baseline/`. If the
> contract changes it goes stale, which surfaces as a baseline-resolution
> warning rather than a silent mismatch.

## Three questions, one lifecycle

The showcase rests on a clean correspondence — three questions a regulated team
asks, three feotest tools, three phases of one device lifecycle:

| Question | feotest tool | Lifecycle |
|---|---|---|
| *Which configuration should the device ship with?* | **Optimize experiment** → iteration history + chosen optimum | Design / development |
| *How accurate is the device there?* | **Measure experiment** → empirical baseline | Validation |
| *Does it still meet its validated performance?* | **Probabilistic test** against that baseline | Verification |

Optimize is **descriptive development work**: the artefact records what was
tried and what the scorer preferred, and makes no inferential claim. The chosen
configuration is then *validated* by the measure experiment — optimize never
replaces validation. Measure and verify are the two phases of the continuous
**one loop** with a handoff: the experiment mints the baseline artefact, the
test consumes it. The verification answers
*drift-from-baseline*, not absolute accuracy re-derived — it is a
non-inferiority check, powered (via the sample size) for the degradation that
matters. The differentiator over a one-off study in a spreadsheet is that this
is **code**: run it on every firmware build, reagent lot, or software release
as an automated gate (lot-release, post-market surveillance under IVDR).

## Choosing the assay protocol: the first act

Before you can validate a configuration, you have to *choose* one. `cargo run
-- optimize` is that design-phase step. It tunes a single **genuine
instrument-configuration parameter** — the assay's **replicate count**: how many
replicate assays the instrument runs per specimen before averaging them into one
call. More replicates average the analytical noise down, so more calls agree
with the reference panel; but each replicate costs reagent and turnaround time.
The optimum is an honest interior trade — enough averaging to be reliable, not
so much that it is wasteful.

**The anti-pattern this chapter is built to avoid.** It would be tempting to
"optimize" the device's **calling threshold** instead, scored by agreement with
the panel. Don't. The panel defines truth as `severity >= 0.5`, so tuning the
threshold to maximise label-agreement just fits the decision boundary to the
answer key — it "discovers" the lot's calibration offset by overfitting to the
validation set, widening the goalposts until a goal is scored. **The optimized
knob must be a real configuration parameter that trades genuine qualities
(precision versus cost) — never the decision rule the validation later judges
against.** So the calling threshold stays *fixed* at the panel's truth boundary
throughout; accuracy is earned by precision, not by relabelling.

**Score ≠ pass rate, honestly.** The run is scored by a named, cost-aware custom
scorer, `accuracy-per-assay`: observed accuracy minus a penalty proportional to
the mean reagent cost per specimen. Cost travels through the artefact's *stated*
value — each replicate assay records one token, so stated tokens per sample =
replicate count — because a feotest `Scorer` sees only the execution result,
never the factor. Nothing statistical is added beyond this arithmetic over two
stated values. The consequence is the pedagogic punchline: **accuracy rises
monotonically with replicates while the score peaks at a modest count and then
falls**, so the highest-accuracy iterations sit near the *bottom* of the
leaderboard by score. The convergence line names `scorer: accuracy-per-assay`,
and because the scorer is not the pass rate, `mavai optimize --hide-scores`
would (correctly) draw a notice — the ranking is not a pass-rate ranking.

Running it over the reference panel sweeps replicate counts 1, 3, 5, … 15 and
writes one `mavai-optimize-1` document. The trajectory it records:

| Replicate count | Accuracy | `accuracy-per-assay` score |
|---|---|---|
| 1× | 29/40 | 0.701 |
| **3×** | **35/40** | **0.803 ← chosen** |
| 5× | 36/40 | 0.780 |
| 7× | 33/40 | 0.657 |
| 9× | 36/40 | 0.684 |
| 11× | 37/40 | 0.661 |
| 13× | 36/40 | 0.588 |
| 15× | 37/40 | 0.565 |

The chosen protocol is **3× replicate** — *not* the most accurate one (37/40 at
11× and 15×). The extra replicates buy real accuracy, but not enough to pay for
their reagent, and the cost-aware scorer says so. Each iteration's
`failureDistribution` also records *which way* the errors fall — `missed-tumour`
versus `false-positive` — so you can read the trade per protocol.

**Rendering the run.** The showcase renders nothing itself — comparison
rendering is the shared tool's job. Render the committed artefact with the
public `mavai` binaries (v0.2.0, [mavai-org/mavai releases](https://github.com/mavai-org/mavai/releases)):

```bash
mavai optimize optimizations -o optimize-report.html
```

The report gives you the ranked leaderboard, the rise-and-fall score trajectory,
the per-iteration criteria and failure matrix, and — the punchline again — that
the winning protocol is not the most accurate, because the named cost-aware
scorer ordered it that way.

> **`optimizations/` provenance.** The committed artefact under
> `optimizations/diagnostics.tumour.assay-protocol/` is generated output, like
> `field-baseline/`: regenerate it with `cargo run -- optimize` and commit the
> result, so a fresh clone has the whole story without running anything. The
> scores and counts are deterministic (fixed seed and panel); only the wall-clock
> latency figures and `generatedAt` timestamp vary between regenerations.

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
src/device.rs        the Device seam + the stochastic MockAnalyzer
src/contract.rs      the ServiceContract: criteria vector, covariates, latency
src/panel.rs         the reference panel (committed ground truth)
src/scenarios.rs     device configurations (healthy / regressed / new lot / drift)
src/optimize.rs      the optimize scenario: the assay-protocol knob, scorer, sweep
src/main.rs          the CLI: optimize / measure / verify / demo / report
src/bin/sentinel.rs  the standalone field self-diagnosis agent
field-baseline/      the pre-validated baseline the sentinel ships with
optimizations/       the committed optimize artefact (regenerate: cargo run -- optimize)
fixtures/            the reference panel + provenance (see its README)
scripts/             regenerate the reference panel
docs/                INFORMATION-FOR-AUDITORS.md / -AUDITEES.md / SENTINEL-SELF-DIAGNOSIS.md
```

## License

Licensed under the [Apache License, Version 2.0](LICENSE). Contributions are
accepted under the same license and the
[Developer Certificate of Origin](dco.txt) — see [CONTRIBUTING.md](CONTRIBUTING.md).
