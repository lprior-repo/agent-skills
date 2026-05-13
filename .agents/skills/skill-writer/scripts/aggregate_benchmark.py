#!/usr/bin/env python3
"""Aggregate grading artifacts into benchmark JSON and Markdown."""

from __future__ import annotations

import argparse
import math
import sys
from collections import defaultdict
from pathlib import Path
from typing import Any

from utils import read_json, write_json, write_text


PREFERRED_CONFIG_ORDER = ["with_skill", "new_skill", "without_skill", "old_skill", "baseline"]


def mean(values: list[float]) -> float | None:
    if not values:
        return None
    return sum(values) / len(values)


def stddev(values: list[float]) -> float | None:
    if len(values) < 2:
        return 0.0 if values else None
    m = sum(values) / len(values)
    return math.sqrt(sum((value - m) ** 2 for value in values) / len(values))


def stat(values: list[float]) -> dict[str, Any] | None:
    m = mean(values)
    if m is None:
        return None
    return {"mean": m, "stddev": stddev(values), "n": len(values)}


def find_eval_metadata(run_dir: Path, stop: Path) -> dict[str, Any]:
    for parent in [run_dir, *run_dir.parents]:
        if parent == stop.parent:
            break
        candidate = parent / "eval_metadata.json"
        if candidate.exists():
            try:
                return read_json(candidate)
            except Exception:
                return {}
    return {}


def detect_configuration(grading_path: Path) -> str:
    run_dir = grading_path.parent
    if run_dir.name.startswith("run-") and run_dir.parent.name:
        return run_dir.parent.name
    return run_dir.name


def collect_runs(workspace: Path) -> list[dict[str, Any]]:
    runs: list[dict[str, Any]] = []
    for grading_path in sorted(workspace.rglob("grading.json")):
        run_dir = grading_path.parent
        try:
            grading = read_json(grading_path)
        except Exception as exc:
            runs.append(
                {
                    "run_dir": str(run_dir),
                    "configuration": detect_configuration(grading_path),
                    "result": {"errors": 1, "error": str(exc)},
                }
            )
            continue

        metadata = find_eval_metadata(run_dir, workspace)
        timing_path = run_dir / "timing.json"
        runner_path = run_dir / "runner_result.json"
        timing = read_json(timing_path) if timing_path.exists() else {}
        runner = read_json(runner_path) if runner_path.exists() else {}
        summary = grading.get("summary", {}) if isinstance(grading, dict) else {}
        result = {
            "passed": summary.get("passed", 0),
            "failed": summary.get("failed", 0),
            "total": summary.get("total", 0),
            "pass_rate": summary.get("pass_rate"),
            "time_seconds": timing.get("duration_seconds"),
            "tokens": timing.get("tokens"),
            "errors": 1 if runner.get("exit_code") not in (0, None) else 0,
        }
        runs.append(
            {
                "eval_id": metadata.get("eval_id"),
                "eval_name": metadata.get("eval_name"),
                "configuration": detect_configuration(grading_path),
                "run_number": run_dir.name.replace("run-", "") if run_dir.name.startswith("run-") else None,
                "run_dir": str(run_dir),
                "result": result,
                "expectations": grading.get("expectations", []),
            }
        )
    return runs


def ordered_configs(configs: set[str]) -> list[str]:
    preferred = [config for config in PREFERRED_CONFIG_ORDER if config in configs]
    remaining = sorted(configs - set(preferred))
    return preferred + remaining


def aggregate(workspace: str | Path) -> dict[str, Any]:
    root = Path(workspace).expanduser().resolve()
    runs = collect_runs(root)
    configs = ordered_configs({run["configuration"] for run in runs})
    grouped: dict[str, list[dict[str, Any]]] = defaultdict(list)
    for run in runs:
        grouped[run["configuration"]].append(run)

    run_summary: dict[str, Any] = {}
    for config in configs:
        config_runs = grouped[config]
        pass_rates = [
            float(run["result"]["pass_rate"])
            for run in config_runs
            if run["result"].get("pass_rate") is not None
        ]
        times = [
            float(run["result"]["time_seconds"])
            for run in config_runs
            if run["result"].get("time_seconds") is not None
        ]
        tokens = [
            float(run["result"]["tokens"])
            for run in config_runs
            if run["result"].get("tokens") is not None
        ]
        run_summary[config] = {
            "runs": len(config_runs),
            "pass_rate": stat(pass_rates),
            "time_seconds": stat(times),
            "tokens": stat(tokens),
            "errors": sum(int(run["result"].get("errors", 0)) for run in config_runs),
        }

    if len(configs) >= 2:
        left, right = configs[0], configs[1]
        delta: dict[str, Any] = {}
        for key in ["pass_rate", "time_seconds", "tokens"]:
            left_stat = run_summary[left].get(key)
            right_stat = run_summary[right].get(key)
            if left_stat and right_stat:
                delta[key] = left_stat["mean"] - right_stat["mean"]
        run_summary["delta"] = delta

    return {
        "metadata": {"workspace": str(root), "configurations": configs},
        "run_summary": run_summary,
        "runs": runs,
        "notes": [],
    }


def fmt_stat(value: dict[str, Any] | None, percent: bool = False) -> str:
    if not value:
        return "n/a"
    multiplier = 100 if percent else 1
    suffix = "%" if percent else ""
    m = value["mean"] * multiplier
    s = (value.get("stddev") or 0) * multiplier
    return f"{m:.1f}{suffix} +/- {s:.1f}{suffix} (n={value['n']})"


def render_markdown(data: dict[str, Any]) -> str:
    lines = ["# Benchmark", ""]
    summary = data.get("run_summary", {})
    configs = data.get("metadata", {}).get("configurations", [])
    lines.append("| Configuration | Runs | Pass rate | Time | Tokens | Errors |")
    lines.append("| --- | ---: | ---: | ---: | ---: | ---: |")
    for config in configs:
        row = summary.get(config, {})
        lines.append(
            "| "
            + " | ".join(
                [
                    config,
                    str(row.get("runs", 0)),
                    fmt_stat(row.get("pass_rate"), percent=True),
                    fmt_stat(row.get("time_seconds")),
                    fmt_stat(row.get("tokens")),
                    str(row.get("errors", 0)),
                ]
            )
            + " |"
        )
    delta = summary.get("delta")
    if delta:
        lines.extend(["", "## Delta", ""])
        for key, value in delta.items():
            suffix = "" if key != "pass_rate" else " percentage points"
            rendered = value * 100 if key == "pass_rate" else value
            lines.append(f"- {key}: {rendered:.3f}{suffix}")
    lines.append("")
    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("workspace", help="Workspace containing grading.json artifacts")
    parser.add_argument("--output-json", help="Output benchmark.json path")
    parser.add_argument("--output-md", help="Output benchmark.md path")
    args = parser.parse_args()

    try:
        root = Path(args.workspace).expanduser().resolve()
        data = aggregate(root)
        json_path = Path(args.output_json) if args.output_json else root / "benchmark.json"
        md_path = Path(args.output_md) if args.output_md else root / "benchmark.md"
        write_json(json_path, data)
        write_text(md_path, render_markdown(data))
    except Exception as exc:
        print(f"ERROR: {exc}", file=sys.stderr)
        return 1

    print(f"Wrote {json_path}")
    print(f"Wrote {md_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
