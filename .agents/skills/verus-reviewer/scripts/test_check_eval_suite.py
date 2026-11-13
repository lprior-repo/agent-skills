from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

import check_eval_suite


def write_suite(root: Path, evals: list[dict]) -> Path:
    fixture_dir = root / "evals" / "fixtures"
    fixture_dir.mkdir(parents=True)
    (fixture_dir / "case.md").write_text("# fixture\n\nline 3 evidence\n", encoding="utf-8")
    path = root / "evals" / "evals.json"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps({"skill_name": "verus-reviewer", "evals": evals}, indent=2) + "\n", encoding="utf-8")
    return path


def test_reject_eval_requires_exact_line_citation_assertion(tmp_path: Path) -> None:
    suite = write_suite(
        tmp_path,
        [
            {
                "id": "bad-reject",
                "prompt": "Review bad proof",
                "expected_output": "Reject this proof packet.",
                "files": ["evals/fixtures/case.md"],
                "assertions": [
                    "Status is REJECTED",
                    "Includes severity or BLOCKED classification",
                    "States an actionable required fix",
                    "Does not return APPROVED while unresolved findings remain",
                ],
            }
        ],
    )

    result = check_eval_suite.validate_suite(tmp_path, suite)

    assert not result.ok
    assert any("exact fixture file:line" in failure for failure in result.failures)


def test_positive_eval_requires_raw_evidence_grade_and_zero_findings(tmp_path: Path) -> None:
    suite = write_suite(
        tmp_path,
        [
            {
                "id": "bad-positive",
                "prompt": "Review clean proof",
                "expected_output": "Approve this proof packet.",
                "files": ["evals/fixtures/case.md"],
                "assertions": [
                    "Status is APPROVED",
                    "Cites exact fixture file:line or file:line-line evidence",
                ],
            }
        ],
    )

    result = check_eval_suite.validate_suite(tmp_path, suite)

    assert not result.ok
    joined = "\n".join(result.failures)
    assert "raw verifier output" in joined
    assert "claimed grade" in joined
    assert "zero findings" in joined


def test_cli_returns_nonzero_and_json_summary_for_invalid_suite(tmp_path: Path) -> None:
    suite = write_suite(
        tmp_path,
        [
            {
                "id": "bad-reject",
                "prompt": "Review bad proof",
                "expected_output": "Reject this proof packet.",
                "files": ["evals/fixtures/case.md"],
                "assertions": ["Status is REJECTED"],
            }
        ],
    )

    proc = subprocess.run(
        [sys.executable, str(Path(check_eval_suite.__file__).resolve()), str(suite), "--root", str(tmp_path)],
        check=False,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )

    assert proc.returncode == 1
    payload = json.loads(proc.stdout)
    assert payload["ok"] is False
    assert payload["eval_count"] == 1
    assert payload["failure_count"] >= 1
