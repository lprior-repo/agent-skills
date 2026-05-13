#!/usr/bin/env python3
"""Propose a better portable skill trigger description from eval evidence."""

from __future__ import annotations

import argparse
import os
import re
import sys
from pathlib import Path
from typing import Any

from utils import parse_skill_md, read_json, run_command_template, write_json


DESCRIPTION_RE = re.compile(r"<new_description>\s*(.*?)\s*</new_description>", re.DOTALL)


def build_prompt(skill: dict[str, Any], results: dict[str, Any], max_length: int) -> str:
    return "\n".join(
        [
            "You are revising the trigger description for a portable agent skill.",
            "The description is the primary routing surface. It must say what the skill does, when to use it, and enough boundary language to avoid near-miss triggers.",
            "Do not mention a specific assistant product, fixed CLI, slash command, or host-only setup unless the skill itself is explicitly for that host.",
            f"Keep the result under {max_length} characters.",
            "Return exactly one block in this form:",
            "<new_description>description text</new_description>",
            "",
            "## Skill",
            f"Name: {skill['name']}",
            f"Current description: {skill['description']}",
            "",
            "## Eval Evidence",
            repr(results),
            "",
            "## Constraints",
            "- Prefer user intent and natural trigger phrases over implementation details.",
            "- Include hard near-miss boundaries only when evidence shows confusion.",
            "- Do not overfit to one failed query.",
            "- Do not invent benchmark results or claim a trigger rate.",
        ]
    )


def extract_description(output: str) -> str:
    match = DESCRIPTION_RE.search(output)
    if not match:
        raise ValueError("model output did not contain a new_description block")
    description = " ".join(match.group(1).split())
    if not description:
        raise ValueError("new description was empty")
    return description


def propose_description(
    skill_path: str | Path,
    results: dict[str, Any],
    command_template: str,
    *,
    max_length: int = 1024,
    model: str = "",
    timeout: int = 600,
) -> dict[str, Any]:
    skill = parse_skill_md(skill_path)
    prompt = build_prompt(skill, results, max_length)
    values = {
        "skill_path": skill["dir"],
        "skill_name": skill["name"],
        "description": skill["description"],
        "max_length": max_length,
        "model": model,
    }
    result = run_command_template(
        command_template,
        values,
        stdin=prompt,
        cwd=skill["dir"],
        timeout=timeout,
    )
    if result.get("exit_code") not in (0, None):
        raise ValueError(
            "description improver command failed: " + str(result.get("stderr", ""))
        )
    new_description = extract_description(str(result.get("stdout", "")))
    if len(new_description) > max_length:
        raise ValueError(
            f"new description is {len(new_description)} characters, over limit {max_length}"
        )
    return {
        "old_description": skill["description"],
        "new_description": new_description,
        "prompt": prompt,
        "raw_response": result.get("stdout", ""),
        "stderr": result.get("stderr", ""),
        "command": result.get("command", ""),
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("skill_path", help="Path to a skill directory or SKILL.md")
    parser.add_argument("--results", help="JSON file with eval or trigger evidence")
    parser.add_argument(
        "--command-template",
        help="Model command template. Also read from SKILL_WRITER_MODEL_CMD.",
    )
    parser.add_argument("--model", default="", help="Opaque model label passed to templates")
    parser.add_argument("--max-length", type=int, default=1024)
    parser.add_argument("--timeout", type=int, default=600)
    parser.add_argument("--output", help="Optional JSON output path")
    args = parser.parse_args()

    command_template = args.command_template or os.environ.get("SKILL_WRITER_MODEL_CMD")
    if not command_template:
        print(
            "ERROR: no improver command configured; pass --command-template or set SKILL_WRITER_MODEL_CMD",
            file=sys.stderr,
        )
        return 1

    results: dict[str, Any] = {}
    if args.results:
        loaded = read_json(args.results)
        if not isinstance(loaded, dict):
            print("ERROR: --results must point to a JSON object", file=sys.stderr)
            return 1
        results = loaded

    try:
        proposal = propose_description(
            args.skill_path,
            results,
            command_template,
            max_length=args.max_length,
            model=args.model,
            timeout=args.timeout,
        )
    except Exception as exc:
        print(f"ERROR: {exc}", file=sys.stderr)
        return 1

    if args.output:
        write_json(args.output, proposal)
    print(proposal["new_description"])
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
