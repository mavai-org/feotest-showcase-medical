# Reference panel

`reference-panel.csv` is the **characterised control material** the device is
tested against — the *known result* side of the contract. Each row is one
reference specimen:

| column     | meaning                                                          |
|------------|-----------------------------------------------------------------|
| `case_id`  | stable identifier (human reference; the contract ignores it)    |
| `severity` | latent quantity in `[0, 1]`; **truth is `severity >= 0.5`** (tumour-positive) |

Lines beginning with `#` are provenance comments.

## What is — and is not — here

The panel carries only **ground truth**. It contains **no device output**:
the instrument's calls are produced at run time, stochastically, by the mock
(or by a real instrument when its `Device` adapter is dropped in). Keeping the
two separate is the whole point — the device is a live stochastic service
sampled repeatedly, not a frozen table of pre-scored results.

```bash
python scripts/make_reference_panel.py        # default 240 positive + 240 normal
python scripts/make_reference_panel.py --n 400
```

## ⚠ Illustrative control material

The committed panel is **synthetic and deterministic** (seed `20260605`), so
the showcase runs on a fresh clone. In reality a reference panel's truth comes
from a **reference standard** — adjudicated histopathology, an orthogonal
assay, clinical follow-up — which is the hard, expensive part this fixture
stands in for. Replace it with a real characterised panel before drawing any
conclusion.
