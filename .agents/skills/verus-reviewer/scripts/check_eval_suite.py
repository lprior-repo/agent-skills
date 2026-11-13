#!/usr/bin/env python3
"""Deterministic structural checks for verus-reviewer eval suites."""

from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any

REJECT_STATUS_RE = re.compile(r"Status is (REJECTED|REJECTED or BLOCKED|REJECTED or BLOCKED, not APPROVED|REJECTED or BLOCKED classification)", re.IGNORECASE)
APPROVED_STATUS_RE = re.compile(r"Status is APPROVED", re.IGNORECASE)
LINE_ASSERTION_RE = re.compile(r"exact fixture file:line|file:line-line", re.IGNORECASE)
SEVERITY_RE = re.compile(r"severity|BLOCKED classification", re.IGNORECASE)
ACTIONABLE_RE = re.compile(r"actionable|required fix|requires? ", re.IGNORECASE)
NO_APPROVED_RE = re.compile(r"Does not return APPROVED while unresolved findings remain", re.IGNORECASE)
RAW_VERIFIER_RE = re.compile(r"raw verifier output|verifier/test evidence", re.IGNORECASE)
TRUST_SCAN_RE = re.compile(r"trust scan", re.IGNORECASE)
PARSER_DELTA_RE = re.compile(r"parser|token|delta", re.IGNORECASE)
BEHAVIOR_RE = re.compile(r"behavior-test|behavior test|test output", re.IGNORECASE)
CARGO_VERUS_RE = re.compile(r"Cargo-Verus|cargo verus", re.IGNORECASE)
GRADE_RE = re.compile(r"grade|ACCEPTABLE|PRODUCTION_USABLE|UPSTREAM_GRADE|PASS", re.IGNORECASE)
ZERO_FINDINGS_RE = re.compile(r"zero findings|Findings section is empty|no severity findings", re.IGNORECASE)


@dataclass(frozen=True)
class ValidationResult:
    ok: bool
    eval_count: int
    failure_count: int
    failures: list[str]


def as_text_list(value: Any) -> list[str]:
    if not isinstance(value, list):
        return []
    return [item for item in value if isinstance(item, str)]


def joined_assertions(entry: dict[str, Any]) -> str:
    return "\n".join(as_text_list(entry.get("assertions")))


def is_positive_eval(entry: dict[str, Any]) -> bool:
    assertions = joined_assertions(entry)
    expected = str(entry.get("expected_output", ""))
    return bool(APPROVED_STATUS_RE.search(assertions)) or expected.strip().lower().startswith("approve")


def is_reject_eval(entry: dict[str, Any]) -> bool:
    if is_positive_eval(entry):
        return False
    text = f"{entry.get('expected_output', '')}\n{joined_assertions(entry)}"
    return bool(re.search(r"reject|block|not approved", text, re.IGNORECASE))


def require(condition: bool, failures: list[str], eval_id: str, message: str) -> None:
    if not condition:
        failures.append(f"{eval_id}: {message}")


def validate_common(root: Path, entry: dict[str, Any], seen: set[str], failures: list[str]) -> None:
    eval_id = str(entry.get("id", "<missing-id>"))
    require(isinstance(entry.get("id"), str) and bool(entry.get("id")), failures, eval_id, "missing string id")
    require(eval_id not in seen, failures, eval_id, "duplicate id")
    seen.add(eval_id)
    require(isinstance(entry.get("prompt"), str) and bool(entry.get("prompt", "").strip()), failures, eval_id, "missing prompt")
    require(isinstance(entry.get("expected_output"), str) and bool(entry.get("expected_output", "").strip()), failures, eval_id, "missing expected_output")
    files = entry.get("files")
    require(isinstance(files, list) and bool(files), failures, eval_id, "files must be a non-empty list")
    if isinstance(files, list):
        for rel in files:
            if not isinstance(rel, str):
                failures.append(f"{eval_id}: file entry is not a string")
                continue
            path = root / rel
            if not path.exists():
                failures.append(f"{eval_id}: fixture missing: {rel}")
            elif path.is_file() and not path.read_text(encoding="utf-8").strip():
                failures.append(f"{eval_id}: fixture is empty: {rel}")
    assertions = as_text_list(entry.get("assertions"))
    require(bool(assertions), failures, eval_id, "assertions must be a non-empty list of strings")
    require(len(assertions) == len(entry.get("assertions", [])) if isinstance(entry.get("assertions"), list) else False, failures, eval_id, "all assertions must be strings")


def validate_reject_eval(entry: dict[str, Any], failures: list[str]) -> None:
    eval_id = str(entry.get("id", "<missing-id>"))
    assertions = joined_assertions(entry)
    require(bool(REJECT_STATUS_RE.search(assertions)), failures, eval_id, "reject/block eval must assert rejected or blocked status")
    require(bool(SEVERITY_RE.search(assertions)), failures, eval_id, "reject/block eval must require severity or BLOCKED classification")
    require(bool(ACTIONABLE_RE.search(assertions)), failures, eval_id, "reject/block eval must require an actionable required fix")
    require(bool(NO_APPROVED_RE.search(assertions)), failures, eval_id, "reject/block eval must forbid APPROVED while unresolved findings remain")
    require(bool(LINE_ASSERTION_RE.search(assertions)), failures, eval_id, "reject/block eval must require exact fixture file:line or file:line-line evidence")


def validate_positive_eval(entry: dict[str, Any], failures: list[str]) -> None:
    eval_id = str(entry.get("id", "<missing-id>"))
    assertions = joined_assertions(entry)
    require(bool(APPROVED_STATUS_RE.search(assertions)), failures, eval_id, "positive eval must assert approved status")
    require(bool(GRADE_RE.search(assertions)), failures, eval_id, "positive eval must cite claimed grade")
    require(bool(RAW_VERIFIER_RE.search(assertions)), failures, eval_id, "positive eval must require raw verifier output")
    require(bool(TRUST_SCAN_RE.search(assertions)), failures, eval_id, "positive eval must require trust scan evidence")
    require(bool(PARSER_DELTA_RE.search(assertions)), failures, eval_id, "positive eval must require parser/token delta evidence")
    require(bool(BEHAVIOR_RE.search(assertions)), failures, eval_id, "positive eval must require behavior-test evidence")
    require(bool(CARGO_VERUS_RE.search(assertions)), failures, eval_id, "positive eval must require Cargo-Verus opt-in evidence")
    require(bool(ZERO_FINDINGS_RE.search(assertions)), failures, eval_id, "positive eval must require zero findings")
    require(bool(LINE_ASSERTION_RE.search(assertions)), failures, eval_id, "positive eval must require exact fixture file:line or file:line-line evidence")


def validate_suite(root: Path, eval_file: Path) -> ValidationResult:
    data = json.loads(eval_file.read_text(encoding="utf-8"))
    failures: list[str] = []
    require(isinstance(data.get("skill_name"), str) and data.get("skill_name") == "verus-reviewer", failures, "<suite>", "skill_name must be verus-reviewer")
    evals = data.get("evals")
    require(isinstance(evals, list), failures, "<suite>", "evals must be a list")
    if not isinstance(evals, list):
        return ValidationResult(False, 0, len(failures), failures)
    seen: set[str] = set()
    for entry in evals:
        if not isinstance(entry, dict):
            failures.append("<suite>: eval entry is not an object")
            continue
        validate_common(root, entry, seen, failures)
        if is_positive_eval(entry):
            validate_positive_eval(entry, failures)
        elif is_reject_eval(entry):
            validate_reject_eval(entry, failures)
    return ValidationResult(not failures, len(evals), len(failures), failures)


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Validate verus-reviewer eval suite structure.")
    parser.add_argument("eval_file", type=Path, help="Path to evals.json")
    parser.add_argument("--root", type=Path, default=None, help="Skill root containing eval fixture paths")
    return parser


def run(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    root = args.root if args.root is not None else args.eval_file.parent.parent
    result = validate_suite(root, args.eval_file)
    print(json.dumps({"ok": result.ok, "eval_count": result.eval_count, "failure_count": result.failure_count, "failures": result.failures}, indent=2))
    return 0 if result.ok else 1


if __name__ == "__main__":
    raise SystemExit(run())
