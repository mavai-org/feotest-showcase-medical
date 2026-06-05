#!/usr/bin/env python3
"""Regenerate fixtures/scores.csv — the frozen evaluation set the feotest
contracts certify against.

Two modes:

  real (default)  Download the PatchCamelyon (PCam) evaluation split, run a
                  tumour-detection model over it, and record, per patch, the
                  model's tumour probability and its wall-clock inference
                  latency. This is the option-C path: real model outputs over
                  a real public benchmark, frozen as a small CSV. Raw images
                  are fetched by torchvision and never committed — only the
                  scores are.

  --placeholder   Synthesise a deterministic, seeded stand-in evaluation set
                  with no model or dataset download, so the Rust showcase
                  runs on a fresh clone. The committed fixture is produced
                  this way and is clearly marked as a placeholder; replace it
                  by running real mode with your evaluated model.

Usage:
  python scripts/regenerate_scores.py --placeholder
  python scripts/regenerate_scores.py --model path/to/model.pt --limit 1000
"""

from __future__ import annotations

import argparse
import csv
import random
import sys
from pathlib import Path

OUT = Path(__file__).resolve().parent.parent / "fixtures" / "scores.csv"

PLACEHOLDER_HEADER = [
    "# PLACEHOLDER evaluation set — synthetic, deterministic (seed=20260605).",
    "# NOT real model output. Regenerate with real mode + your evaluated model:",
    "#   python scripts/regenerate_scores.py --model <path> ",
    "# Columns: patch_id,label,score,inference_ms",
]

REAL_HEADER_TEMPLATE = [
    "# PatchCamelyon (PCam) evaluation split — real model output.",
    "# model: {model}",
    "# decision threshold applied downstream by the feotest contract.",
    "# Columns: patch_id,label,score,inference_ms",
]


def write_csv(rows: list[tuple[str, str, float, float]], header: list[str]) -> None:
    OUT.parent.mkdir(parents=True, exist_ok=True)
    with OUT.open("w", newline="") as fh:
        for line in header:
            fh.write(line + "\n")
        writer = csv.writer(fh)
        writer.writerow(["patch_id", "label", "score", "inference_ms"])
        for pid, label, score, ms in rows:
            writer.writerow([pid, label, f"{score:.4f}", f"{ms:.2f}"])
    print(f"wrote {len(rows)} rows → {OUT}")


def generate_placeholder(n_tumour: int, n_normal: int) -> list[tuple[str, str, float, float]]:
    """Deterministic stand-in: scores drawn so the model comfortably clears
    the sensitivity/specificity floors, with a realistic latency spread."""
    rng = random.Random(20260605)
    rows: list[tuple[str, str, float, float]] = []

    def latency() -> float:
        # ~12 ms typical, occasional tail — stays under a 40 ms p95 ceiling.
        return max(2.0, rng.gauss(12.0, 3.0))

    for i in range(n_tumour):
        # ~97% of tumour patches score above 0.5 (true positives).
        score = rng.betavariate(6, 1) if rng.random() < 0.97 else rng.betavariate(1, 6)
        rows.append((f"tum-{i:04d}", "tumour", score, latency()))

    for i in range(n_normal):
        # ~93% of normal patches score below 0.5 (true negatives).
        score = rng.betavariate(1, 6) if rng.random() < 0.93 else rng.betavariate(6, 1)
        rows.append((f"nrm-{i:04d}", "normal", score, latency()))

    rng.shuffle(rows)
    return rows


def generate_real(model_path: str, limit: int) -> list[tuple[str, str, float, float]]:
    """Run a tumour-detection model over the PCam evaluation split."""
    try:
        import time

        import torch
        from torchvision import transforms
        from torchvision.datasets import PCAM
    except ImportError:
        sys.exit(
            "real mode needs torch + torchvision:\n"
            "  pip install torch torchvision\n"
            "or use --placeholder for a model-free run."
        )

    model = torch.jit.load(model_path)
    model.eval()

    to_tensor = transforms.Compose([transforms.ToTensor()])
    dataset = PCAM(root="data", split="test", download=True, transform=to_tensor)

    rows: list[tuple[str, str, float, float]] = []
    count = min(limit, len(dataset))
    with torch.no_grad():
        for i in range(count):
            image, label = dataset[i]
            batch = image.unsqueeze(0)
            start = time.perf_counter()
            logits = model(batch)
            elapsed_ms = (time.perf_counter() - start) * 1000.0
            score = torch.sigmoid(logits).flatten()[0].item()
            rows.append(
                (
                    f"pcam-{i:05d}",
                    "tumour" if int(label) == 1 else "normal",
                    score,
                    elapsed_ms,
                )
            )
    return rows


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--placeholder", action="store_true", help="synthesise a model-free stand-in set")
    parser.add_argument("--model", help="path to a TorchScript tumour-detection model (real mode)")
    parser.add_argument("--limit", type=int, default=2000, help="max patches to score in real mode")
    parser.add_argument("--n-tumour", type=int, default=240, help="placeholder tumour-patch count")
    parser.add_argument("--n-normal", type=int, default=160, help="placeholder normal-patch count")
    args = parser.parse_args()

    if args.placeholder or not args.model:
        if not args.placeholder:
            print("no --model given; generating a placeholder set. Pass --model for real mode.\n")
        rows = generate_placeholder(args.n_tumour, args.n_normal)
        write_csv(rows, PLACEHOLDER_HEADER)
    else:
        rows = generate_real(args.model, args.limit)
        write_csv(rows, [line.format(model=args.model) for line in REAL_HEADER_TEMPLATE])


if __name__ == "__main__":
    main()
