# Content series — spine

This showcase anchors a layered funnel. Each tier links down into the next,
ending at the runnable repo. LinkedIn targeting puts the top tier in front of
the demographic; the repo is the proof.

- **LinkedIn posts** — short, demographic-targeted (functional-safety /
  regulatory-affairs / digital-pathology R&D / embedded-Rust). One hook each,
  linking to an mavai article.
- **mavai.ch (regulatory register)** — positions the capability as compliance
  evidence: SaMD performance validation, EU AI Act high-risk obligations,
  post-market monitoring.
- **mavai.org (engineering register)** — the methodology: why a test set you
  "eyeball" is not evidence, what a confidence interval and a feasibility gate
  add, how the contract is authored.
- **This repo (README)** — clone, `cargo run`, read the verdict.

## Working titles (to be finalised once the framing is chosen)

The repo currently implements **framing 1** (pre-market floor). The article
spine should follow whichever framing leads.

1. *"95% sensitivity is not a result — it's a hypothesis."* (engineering) —
   point estimate vs. confidence bound vs. feasibility.
2. *"Your model passed validation. Can you prove it still passes?"*
   (regulatory) — post-market drift against a locked baseline.
3. *"Statistical evidence the EU AI Act will ask you for."* (regulatory,
   mavai.ch) — high-risk obligations, framed as the contract this repo runs.

> Drafts are not written yet — pending the framing decision (see the
> top-level README, "Two ways to frame the contract").
