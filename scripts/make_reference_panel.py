#!/usr/bin/env python3
"""Generate fixtures/reference-panel.csv — the characterised control material
the device is tested against.

Each row is one reference specimen: a stable id and a latent `severity` in
[0, 1]. Truth is `severity >= 0.5` (tumour-positive). The device measures this
latent value with noise at run time; the panel itself carries only the ground
truth, never any device output (that is generated stochastically by the mock,
or by a real instrument when one is dropped in).

The panel is illustrative. A real reference panel's truth comes from a
reference standard (adjudicated histopathology, orthogonal assay, …) — which in
deployment is the hard, expensive part this fixture stands in for.

    python scripts/make_reference_panel.py            # default 240 + 240
    python scripts/make_reference_panel.py --n 400
"""

from __future__ import annotations

import argparse
import csv
import random
from pathlib import Path

OUT = Path(__file__).resolve().parent.parent / "fixtures" / "reference-panel.csv"
HEADER = [
    "# Reference panel — illustrative control material (synthetic, seed=20260605).",
    "# Truth is severity >= 0.5 (tumour-positive). Device output is NOT here:",
    "# it is generated at run time by the mock (or a real instrument adapter).",
    "# Columns: case_id,severity",
]


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--n", type=int, default=240, help="specimens per class")
    args = parser.parse_args()
    rng = random.Random(20260605)

    rows: list[tuple[str, float]] = []
    # Positives in [0.5, 1.0], normals in [0.0, 0.5). Uniform spread means a
    # realistic share of near-boundary specimens — the ones a noisy device gets
    # wrong — without stacking the deck.
    for i in range(args.n):
        rows.append((f"pos-{i:04d}", rng.uniform(0.5, 1.0)))
    for i in range(args.n):
        rows.append((f"nrm-{i:04d}", rng.uniform(0.0, 0.5)))
    rng.shuffle(rows)

    OUT.parent.mkdir(parents=True, exist_ok=True)
    with OUT.open("w", newline="") as fh:
        for line in HEADER:
            fh.write(line + "\n")
        writer = csv.writer(fh)
        writer.writerow(["case_id", "severity"])
        for case_id, severity in rows:
            writer.writerow([case_id, f"{severity:.4f}"])
    print(f"wrote {len(rows)} specimens → {OUT}")


if __name__ == "__main__":
    main()
