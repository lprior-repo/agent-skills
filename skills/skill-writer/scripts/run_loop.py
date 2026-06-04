#!/usr/bin/env python3
"""Optimize a portable skill description against trigger evals."""

from __future__ import annotations

import argparse
import os
import sys
from argparse import Namespace
from pathlib import Path
from typing import Any

from generate_report import write_report
from improve_description import propose_description
from run_eval import run_evals
from utils import (
    copy_skill_tree,
    load_eval_file,
    parse_skill_md,
    read_json,
    update_frontmatter_field,
    write_json,
)


def split_items(items: list[dict[str, Any]], train_ratio: float) -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    if len(items) < 2:
        return items, []
    split_at = max(1, min(len(items) - 1, int(round(len(items) * train_ratio))))
    return items[:split_at], items[split_at:]


def validate_trigger_items(items: list[dict[str, Any]]) -> None:
    missing: list[str] = []
    non_boolean: list[str] = []
    for item in items:
        item_id = str(item.get("id", item.get("name", "unknown")))
        if "should_trigger" not in item:
            missing.append(item_id)
        elif not isinstance(item.get("should_trigger"), bool):
            non_boolean.append(item_id)
    problems: list[str] = []
    if missing:
        problems.append("missing should_trigger: " + ", ".join(missing))
    if non_boolean:
        problems.append("non-boolean should_trigger: " + ", ".join(non_boolean))
    if problems:
        raise ValueError("trigger evals must provide boolean should_trigger labels; " + "; ".join(problems))


def score_records(records: list[dict[str, Any]]) -> dict[str, Any]:
    graded = [record for record in records if record.get("expected") is not None]
    total = len(graded)
    passed = sum(1 for record in graded if record.get("passed"))
    failed = total - passed
    return {
        "passed": passed,
        "failed": failed,
        "total": total,
        "accuracy": (passed / total) if total else None,
    }


def read_trigger_records(summary: dict[str, Any], items: list[dict[str, Any]]) -> list[dict[str, Any]]:
    by_id = {str(item.get("id")): item for item in items}
    records: list[dict[str, Any]] = []
    for run in summary.get("runs", []):
        run_dir = Path(run["run_dir"])
        grading_path = run_dir / "grading.json"
        grading = read_json(grading_path) if grading_path.exists() else {}
        trigger = grading.get("trigger", {}) if isinstance(grading, dict) else {}
        item = by_id.get(str(run.get("eval_id")), {})
        expected = trigger.get("expected")
        detected = trigger.get("detected")
        records.append(
            {
                "eval_id": run.get("eval_id"),
                "query": item.get("prompt", ""),
                "expected": expected,
                "detected": detected,
                "passed": expected == detected if expected is not None else None,
                "run_dir": run.get("run_dir"),
            }
        )
    return records


def evaluate_queries(
    *,
    skill_path: Path,
    items: list[dict[str, Any]],
    workspace: Path,
    command_template: str,
    trigger_regex: str,
    model: str,
    timeout: int,
) -> dict[str, Any]:
    eval_file = workspace / "trigger_queries.json"
    write_json(eval_file, {"queries": items})
    args = Namespace(
        skill_path=str(skill_path),
        evals=str(eval_file),
        workspace=str(workspace),
        iteration=1,
        mode="with_skill",
        configuration=None,
        runs=1,
        command_template=command_template,
        baseline_skill_path=None,
        model=model,
        timeout=timeout,
        trigger_regex=trigger_regex,
        manual=False,
    )
    summary = run_evals(args)
    records = read_trigger_records(summary, items)
    return {"records": records, "summary": score_records(records), "workspace": str(workspace)}


def default_workspace(skill: dict[str, Any]) -> Path:
    return Path(skill["dir"]).parent / f"{skill['name']}-description-workspace"


def optimize(args: argparse.Namespace) -> dict[str, Any]:
    command_template = args.command_template or os.environ.get("SKILL_WRITER_RUNNER")
    improver_command = args.improver_command_template or os.environ.get("SKILL_WRITER_MODEL_CMD")
    if not command_template:
        raise ValueError("no trigger runner configured; pass --command-template or set SKILL_WRITER_RUNNER")
    if not improver_command:
        raise ValueError(
            "no description improver configured; pass --improver-command-template or set SKILL_WRITER_MODEL_CMD"
        )
    if not args.trigger_regex:
        raise ValueError("--trigger-regex is required so the loop can score trigger decisions")

    skill = parse_skill_md(args.skill_path)
    _, items = load_eval_file(args.evals)
    validate_trigger_items(items)
    train_items, heldout_items = split_items(items, args.train_ratio)
    workspace = Path(args.workspace).expanduser().resolve() if args.workspace else default_workspace(skill)
    workspace.mkdir(parents=True, exist_ok=True)

    candidate_description = skill["description"]
    iterations: list[dict[str, Any]] = []
    best: dict[str, Any] | None = None

    for iteration in range(1, args.iterations + 1):
        iteration_dir = workspace / f"iteration-{iteration}"
        candidate_dir = iteration_dir / "skill-candidate"
        copy_skill_tree(args.skill_path, candidate_dir)
        update_frontmatter_field(candidate_dir, "description", candidate_description)

        evaluation = evaluate_queries(
            skill_path=candidate_dir,
            items=train_items,
            workspace=iteration_dir / "train",
            command_template=command_template,
            trigger_regex=args.trigger_regex,
            model=args.model or "",
            timeout=args.timeout,
        )
        record = {
            "iteration": iteration,
            "description": candidate_description,
            "summary": evaluation["summary"],
            "records": evaluation["records"],
        }
        iterations.append(record)

        accuracy = evaluation["summary"].get("accuracy")
        best_accuracy = best.get("summary", {}).get("accuracy") if best else None
        if best is None or (accuracy is not None and (best_accuracy is None or accuracy > best_accuracy)):
            best = record

        if evaluation["summary"].get("failed") == 0:
            break

        proposal = propose_description(
            candidate_dir,
            {"summary": evaluation["summary"], "records": evaluation["records"]},
            improver_command,
            max_length=args.max_length,
            model=args.model or "",
            timeout=args.timeout,
        )
        record["proposal"] = {
            "new_description": proposal["new_description"],
            "stderr": proposal.get("stderr", ""),
        }
        if proposal["new_description"] == candidate_description:
            break
        candidate_description = proposal["new_description"]

    if best is None:
        raise ValueError("no optimization iterations were run")

    heldout: dict[str, Any] = {"records": [], "summary": {"total": 0, "accuracy": None}}
    if heldout_items:
        heldout_dir = workspace / "heldout"
        candidate_dir = heldout_dir / "skill-candidate"
        copy_skill_tree(args.skill_path, candidate_dir)
        update_frontmatter_field(candidate_dir, "description", best["description"])
        heldout = evaluate_queries(
            skill_path=candidate_dir,
            items=heldout_items,
            workspace=heldout_dir / "test",
            command_template=command_template,
            trigger_regex=args.trigger_regex,
            model=args.model or "",
            timeout=args.timeout,
        )

    if args.apply:
        update_frontmatter_field(args.skill_path, "description", best["description"])

    result = {
        "skill_name": skill["name"],
        "workspace": str(workspace),
        "original_description": skill["description"],
        "final_description": best["description"],
        "applied": bool(args.apply),
        "iterations": iterations,
        "heldout": heldout,
    }
    result_path = workspace / "description_optimization.json"
    report_path = workspace / "description_optimization.html"
    write_json(result_path, result)
    write_report(result, report_path)
    result["result_path"] = str(result_path)
    result["report_path"] = str(report_path)
    return result


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("skill_path", help="Path to skill directory or SKILL.md")
    parser.add_argument("evals", help="Trigger eval JSON with queries/evals and should_trigger booleans")
    parser.add_argument("--workspace", help="Workspace for optimizer artifacts")
    parser.add_argument("--iterations", type=int, default=3)
    parser.add_argument("--train-ratio", type=float, default=0.7)
    parser.add_argument(
        "--command-template",
        help="Trigger runner command template. Also read from SKILL_WRITER_RUNNER.",
    )
    parser.add_argument(
        "--improver-command-template",
        help="Description improver command template. Also read from SKILL_WRITER_MODEL_CMD.",
    )
    parser.add_argument("--trigger-regex", help="Regex meaning the host selected the skill")
    parser.add_argument("--model", default="", help="Opaque model label passed to command templates")
    parser.add_argument("--timeout", type=int, default=600)
    parser.add_argument("--max-length", type=int, default=1024)
    parser.add_argument("--apply", action="store_true", help="Write the best description back to SKILL.md")
    args = parser.parse_args()

    try:
        result = optimize(args)
    except Exception as exc:
        print(f"ERROR: {exc}", file=sys.stderr)
        return 1

    print(f"Best description: {result['final_description']}")
    print(f"Wrote {result['result_path']}")
    print(f"Wrote {result['report_path']}")
    if not result["applied"]:
        print("Not applied. Re-run with --apply to update SKILL.md.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
