# feotest showcase — digital-pathology tumour detection

A worked, runnable example of using [feotest](https://github.com/mavai-org/feotest)
to put **statistical rigour** behind a claim a regulated medical-device team
actually has to make and defend:

> *Over its evaluation set, the tumour-detection model meets its clinical
> sensitivity and specificity floors — and the evaluation was large enough for
> that to mean something.*

The "service under test" is a tumour/normal classifier for histopathology
image patches — the kind of stochastic ML component that increasingly sits
inside in-vitro-diagnostic and digital-pathology devices. feotest does not
treat it as a black box to eyeball on a test set; it treats it as a
**stochastic service under contract** and renders a reproducible pass/fail
verdict with the supporting statistics.

> **Illustrative, not a clinical result.** This repository demonstrates a
> *methodology*. It ships a placeholder evaluation set (see
> [`fixtures/README.md`](fixtures/README.md)) and makes no claim about any
> specific product or vendor. Point it at a real evaluated model to draw real
> conclusions.

## Run it

```bash
cargo run
```

This loads the frozen evaluation set (`fixtures/scores.csv`), splits it into
the tumour-positive and normal populations, and certifies two contracts:

- **Sensitivity** — over tumour patches, the fraction correctly flagged must
  clear the clinical floor (missing a tumour is the costly error).
- **Specificity** — over normal patches, the fraction correctly cleared must
  hold its floor (controls the pathologist's false-alarm burden).

Each contract also carries a **per-patch latency** commitment: the recorded
inference time of every patch is replayed on the invocation path, so feotest's
latency dimension reflects the model's measured profile rather than a fixture
lookup. Exit code is `0` only if every contract holds — so this drops straight
into a CI gate or a release-evidence pipeline.

## What feotest actually checks

A clinical floor is a **normative** target — a number asserted from a
specification, not learned from data. feotest evaluates it as follows: the
contract **passes when the observed pass rate clears the floor**, and it
reports, alongside the verdict, the *implied confidence*, the Wilson score
interval for the true rate, and a **feasibility** check on whether the
evaluation set was large enough to detect a clinically meaningful degradation.
The rigour is twofold — a transparent, reproducible verdict, and an explicit
statement of whether your sample size earns the claim — rather than an
unqualified "we ran it on a test set and it looked fine."

## Two ways to frame the contract

The same machinery supports two distinct regulatory narratives. **This repo
currently implements the first.**

1. **Pre-market certification against a fixed clinical floor** *(normative)* —
   "does the model meet the acceptance threshold on an adequately sized
   evaluation set?" Maps to IEC 62304 verification and SaMD performance
   validation.
2. **Post-market drift detection against a validated baseline** *(empirical,
   confidence-gated)* — "has the model degraded from its locked, validated
   baseline beyond what sampling noise explains, at a stated confidence?" Here
   the threshold is the Wilson lower bound derived from the baseline, so the
   *confidence bound itself* gates the verdict. Maps to EU AI Act post-market
   monitoring and continuous validation.

Which framing leads the showcase (and its companion articles) is a deliberate
choice — see the content spine in [`docs/articles/`](docs/articles/).

## Layout

```
src/pathology.rs   the ServiceContract: populations, criteria, latency
src/fixtures.rs    loads the frozen evaluation set
src/main.rs        runs both contracts, prints verdicts, sets exit code
fixtures/          the committed scores + provenance (see its README)
scripts/           regenerate the scores from a real model + PCam
docs/articles/     the content series this showcase anchors
```

## License

Apache-2.0.
