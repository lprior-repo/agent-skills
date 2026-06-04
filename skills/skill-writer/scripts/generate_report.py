#!/usr/bin/env python3
"""Generate a neutral HTML report for description optimization runs."""

from __future__ import annotations

import argparse
import html
import sys
from pathlib import Path
from typing import Any

from utils import read_json, write_text


def esc(value: Any) -> str:
    return html.escape("" if value is None else str(value))


def pct(value: Any) -> str:
    if value is None:
        return "n/a"
    try:
        return f"{float(value) * 100:.1f}%"
    except Exception:
        return "n/a"


def render_report(data: dict[str, Any]) -> str:
    iterations = data.get("iterations", [])
    heldout = data.get("heldout", {})
    heldout_summary = heldout.get("summary", {}) if isinstance(heldout, dict) else {}
    rows: list[str] = []
    for item in iterations:
        rows.append(
            "<tr>"
            f"<td>{esc(item.get('iteration'))}</td>"
            f"<td>{pct(item.get('summary', {}).get('accuracy'))}</td>"
            f"<td><pre>{esc(item.get('description'))}</pre></td>"
            f"<td>{esc(item.get('summary', {}).get('total'))}</td>"
            f"<td>{esc(item.get('summary', {}).get('failed'))}</td>"
            "</tr>"
        )

    return f"""<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Skill Description Optimization</title>
  <style>
    body {{ font-family: system-ui, sans-serif; margin: 2rem; color: #17202a; background: #f8fafc; }}
    h1, h2 {{ margin-bottom: 0.4rem; }}
    .card {{ background: white; border: 1px solid #d9e2ec; border-radius: 8px; padding: 1rem; margin: 1rem 0; }}
    table {{ width: 100%; border-collapse: collapse; background: white; }}
    th, td {{ border: 1px solid #d9e2ec; padding: 0.65rem; vertical-align: top; }}
    th {{ background: #102a43; color: white; text-align: left; }}
    pre {{ white-space: pre-wrap; margin: 0; font-family: ui-monospace, SFMono-Regular, Menlo, monospace; }}
    .muted {{ color: #627d98; }}
  </style>
</head>
<body>
  <h1>Skill Description Optimization</h1>
  <p class="muted">Skill: {esc(data.get('skill_name'))}</p>
  <div class="card">
    <h2>Final Description</h2>
    <pre>{esc(data.get('final_description'))}</pre>
  </div>
  <div class="card">
    <h2>Held-out Score</h2>
    <p>{pct(heldout_summary.get('accuracy'))} over {esc(heldout_summary.get('total'))} cases</p>
  </div>
  <h2>Iterations</h2>
  <table>
    <thead><tr><th>Iteration</th><th>Accuracy</th><th>Description</th><th>Total</th><th>Failed</th></tr></thead>
    <tbody>{''.join(rows)}</tbody>
  </table>
</body>
</html>
"""


def write_report(data: dict[str, Any], output_path: str | Path) -> Path:
    output = Path(output_path)
    write_text(output, render_report(data))
    return output


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("results_json", help="Description optimization JSON")
    parser.add_argument("--output", help="HTML output path")
    args = parser.parse_args()

    try:
        data = read_json(args.results_json)
        output = Path(args.output) if args.output else Path(args.results_json).with_suffix(".html")
        write_report(data, output)
    except Exception as exc:
        print(f"ERROR: {exc}", file=sys.stderr)
        return 1

    print(f"Wrote {output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
