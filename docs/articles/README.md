# Content series — spine

This showcase anchors a layered funnel. Each tier links down into the next,
ending at the runnable repo. LinkedIn targeting puts the top tier in front of
the demographic; the repo is the proof.

- **LinkedIn posts** — short, demographic-targeted (IVD R&D / regulatory
  affairs / quality / V&V engineering / embedded-Rust). One hook each, linking
  to an mavai article.
- **mavai.ch (regulatory register)** — compliance evidence: IVDR performance
  evaluation, lot-release, post-market surveillance, the validation ↔
  verification distinction.
- **mavai.org (engineering register)** — the methodology: a device is a
  stochastic service; the measure → verify loop; covariates as baseline
  identity; why a number without a confidence floor and a feasibility check is
  not evidence.
- **This repo (README)** — clone, `cargo run`, watch a device get characterised
  and then caught drifting.

## The spine the showcase embodies

1. **"How accurate is your device? That's an experiment."** (engineering) —
   measure → baseline; point estimate vs confidence floor vs feasibility.
2. **"Does it still meet validated performance? That's a probabilistic test."**
   (engineering) — drift-from-baseline as a non-inferiority check; the silent
   regression caught in phase 3.
3. **"Re-baseline when the lot changes."** (engineering/regulatory) —
   covariates as baseline identity; the `COVARIATE_MISMATCH` guard; the silent
   undeclared-change failure mode.
4. **"The statistical evidence IVDR asks for — as code you re-run every
   release."** (regulatory, mavai.ch) — validation/verification, lot-release,
   post-market monitoring framed as the loop this repo runs.

> Drafts are not written yet — pending the operator's call on which tier leads
> the campaign.
