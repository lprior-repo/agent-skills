#!/usr/bin/env python3
"""Run portable skill eval prompts through a configurable host command."""

from __future__ import annotations

import argparse
import os
import re
import sys
from pathlib import Path
from typing import Any

from utils import (
    load_eval_file,
    parse_skill_md,
    run_command_template,
    safe_slug,
    transcript_from_result,
    write_json,
    write_text,
)


DEFAULT_CONFIGS = ["with_skill", "without_skill"]


def configuration_list(args: argparse.Namespace) -> list[str]:
    if args.configuration:
        return args.configuration
    if args.mode == "both":
        return DEFAULT_CONFIGS
    return [args.mode]


def default_workspace(skill_name: str) -> Path:
    return Path.cwd() / f"{skill_name}-workspace"


def active_skill_path(config: str, skill_path: str, baseline_skill_path: str | None) -> str:
    if config in {"with_skill", "new_skill", "skill", "current"}:
        return skill_path
    if config in {"old_skill", "baseline"} and baseline_skill_path:
        return baseline_skill_path
    return ""


def should_trigger_value(item: dict[str, Any]) -> str:
    if "should_trigger" not in item:
        return ""
    return "true" if expected_trigger(item) else "false"


def expected_trigger(item: dict[str, Any]) -> bool:
    value = item.get("should_trigger")
    if not isinstance(value, bool):
        item_id = item.get("id", "unknown")
        raise ValueError(f"eval item {item_id} should_trigger must be a JSON boolean")
    return value


def write_trigger_grading(
    run_dir: Path,
    item: dict[str, Any],
    result: dict[str, Any],
    trigger_regex: str | None,
) -> None:
    if trigger_regex is None or "should_trigger" not in item:
        return
    output = str(result.get("stdout", "")) + "\n" + str(result.get("stderr", ""))
    detected = re.search(trigger_regex, output, flags=re.IGNORECASE) is not None
    expected = expected_trigger(item)
    passed = detected == expected
    grading = {
        "expectations": [
            {
                "text": "Trigger decision matches expected should_trigger value",
                "passed": passed,
                "evidence": (
                    f"expected={expected}; detected={detected}; regex={trigger_regex}"
                ),
            }
        ],
        "summary": {
            "passed": 1 if passed else 0,
            "failed": 0 if passed else 1,
            "total": 1,
            "pass_rate": 1.0 if passed else 0.0,
        },
        "claims": [],
        "trigger": {
            "expected": expected,
            "detected": detected,
            "regex": trigger_regex,
        },
    }
    write_json(run_dir / "grading.json", grading)


def run_one(
    *,
    item: dict[str, Any],
    skill: dict[str, Any],
    config: str,
    run_number: int,
    run_dir: Path,
    command_template: str | None,
    baseline_skill_path: str | None,
    model: str,
    timeout: int,
    manual: bool,
    trigger_regex: str | None,
) -> dict[str, Any]:
    run_dir.mkdir(parents=True, exist_ok=True)
    outputs_dir = run_dir / "outputs"
    outputs_dir.mkdir(exist_ok=True)
    prompt_file = run_dir / "prompt.txt"
    write_text(prompt_file, item["prompt"] + "\n")

    values = {
        "query": item["prompt"],
        "prompt": item["prompt"],
        "prompt_file": str(prompt_file),
        "skill_path": skill["dir"],
        "active_skill_path": active_skill_path(
            config, skill["dir"], baseline_skill_path
        ),
        "baseline_skill_path": baseline_skill_path or "",
        "skill_name": skill["name"],
        "description": skill["description"],
        "mode": config,
        "configuration": config,
        "output_dir": str(outputs_dir),
        "run_dir": str(run_dir),
        "eval_id": item.get("id", ""),
        "eval_name": item.get("name", ""),
        "model": model,
        "should_trigger": should_trigger_value(item),
        "expected_trigger": should_trigger_value(item),
    }

    if manual:
        result = {
            "command": "manual",
            "exit_code": None,
            "stdout": "Manual mode selected. Execute the prompt with this configuration and place outputs in outputs/.",
            "stderr": "",
            "duration_seconds": None,
            "timed_out": False,
        }
    elif command_template:
        result = run_command_template(
            command_template,
            values,
            stdin=item["prompt"],
            cwd=run_dir,
            timeout=timeout,
        )
    else:
        raise ValueError(
            "no runner command configured; pass --command-template, set SKILL_WRITER_RUNNER, or use --manual"
        )

    write_text(run_dir / "transcript.md", transcript_from_result(result))
    write_json(run_dir / "runner_result.json", result)
    write_json(
        run_dir / "timing.json",
        {
            "duration_seconds": result.get("duration_seconds"),
            "tokens": None,
            "source": "runner_wall_clock" if result.get("duration_seconds") is not None else "unavailable",
        },
    )
    write_trigger_grading(run_dir, item, result, trigger_regex)
    return result


def run_evals(args: argparse.Namespace) -> dict[str, Any]:
    skill = parse_skill_md(args.skill_path)
    eval_path = Path(args.evals)
    if not eval_path.is_absolute() and not eval_path.exists():
        candidate = Path(skill["dir"]) / eval_path
        if candidate.exists():
            eval_path = candidate
    if not eval_path.exists():
        raise FileNotFoundError(f"eval file not found: {eval_path}")
    eval_data, items = load_eval_file(eval_path)
    workspace = Path(args.workspace).expanduser().resolve() if args.workspace else default_workspace(skill["name"])
    iteration_dir = workspace / f"iteration-{args.iteration}"
    configs = configuration_list(args)
    command_template = args.command_template or os.environ.get("SKILL_WRITER_RUNNER")
    results: list[dict[str, Any]] = []

    for item in items:
        eval_name = str(item.get("name") or safe_slug(item["prompt"], "eval"))
        eval_dir = iteration_dir / f"eval-{item.get('id')}-{safe_slug(eval_name, 'eval')}"
        eval_dir.mkdir(parents=True, exist_ok=True)
        write_json(
            eval_dir / "eval_metadata.json",
            {
                "eval_id": item.get("id"),
                "eval_name": eval_name,
                "prompt": item["prompt"],
                "expectations": item.get("expectations", []),
                "should_trigger": item.get("should_trigger"),
                "source": str(eval_path.resolve()),
            },
        )
        for config in configs:
            for run_number in range(1, args.runs + 1):
                run_dir = eval_dir / config / f"run-{run_number}"
                result = run_one(
                    item=item,
                    skill=skill,
                    config=config,
                    run_number=run_number,
                    run_dir=run_dir,
                    command_template=command_template,
                    baseline_skill_path=args.baseline_skill_path,
                    model=args.model or "",
                    timeout=args.timeout,
                    manual=args.manual,
                    trigger_regex=args.trigger_regex,
                )
                results.append(
                    {
                        "eval_id": item.get("id"),
                        "eval_name": eval_name,
                        "configuration": config,
                        "run_number": run_number,
                        "run_dir": str(run_dir),
                        "exit_code": result.get("exit_code"),
                        "duration_seconds": result.get("duration_seconds"),
                    }
                )

    summary = {
        "skill_name": skill["name"],
        "workspace": str(workspace),
        "iteration": args.iteration,
        "eval_file": str(eval_path.resolve()),
        "configurations": configs,
        "runs": results,
        "eval_metadata": eval_data,
    }
    write_json(iteration_dir / "run_eval_summary.json", summary)
    return summary


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("skill_path", help="Path to the skill directory or SKILL.md")
    parser.add_argument(
        "--evals",
        default="evals/evals.json",
        help="JSON file with evals or trigger queries",
    )
    parser.add_argument("--workspace", help="Workspace root for generated artifacts")
    parser.add_argument("--iteration", type=int, default=1, help="Iteration number")
    parser.add_argument(
        "--mode",
        choices=["both", "with_skill", "without_skill", "old_skill", "new_skill"],
        default="both",
        help="Default configuration set to run",
    )
    parser.add_argument(
        "--configuration",
        action="append",
        help="Custom configuration name; can be repeated and overrides --mode",
    )
    parser.add_argument("--runs", type=int, default=1, help="Runs per eval/configuration")
    parser.add_argument(
        "--command-template",
        help="Host runner command template. Also read from SKILL_WRITER_RUNNER.",
    )
    parser.add_argument("--baseline-skill-path", help="Optional old/baseline skill path")
    parser.add_argument("--model", default="", help="Opaque model label passed to templates")
    parser.add_argument("--timeout", type=int, default=600, help="Command timeout in seconds")
    parser.add_argument(
        "--trigger-regex",
        help="Regex whose presence in runner output means the skill triggered",
    )
    parser.add_argument(
        "--manual",
        action="store_true",
        help="Create artifact folders without invoking a host command",
    )
    args = parser.parse_args()

    try:
        summary = run_evals(args)
    except Exception as exc:
        print(f"ERROR: {exc}", file=sys.stderr)
        return 1

    print(f"Wrote eval artifacts to {summary['workspace']}/iteration-{summary['iteration']}")
    print(f"Runs: {len(summary['runs'])}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
