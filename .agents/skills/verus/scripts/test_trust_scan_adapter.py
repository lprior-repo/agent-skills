from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

import trust_scan_adapter


def write_fixture(tmp_path: Path, text: str) -> Path:
    path = tmp_path / "proof.rs"
    path.write_text(text, encoding="utf-8")
    return path


def marker_kinds(path: Path) -> set[str]:
    return {marker["kind"] for marker in trust_scan_adapter.scan_paths([path])}


def test_detects_parenthesized_spaced_and_nested_cfg_attr(tmp_path: Path) -> None:
    path = write_fixture(
        tmp_path,
        """
#[verifier(external_body)]
fn parenthesized(bytes: &[u8]) -> u64 { 0 }

#[cfg_attr(any(verus_keep_ghost, feature = "proof"),
           verifier::external_body)]
fn nested(bytes: &[u8]) -> u64 { 0 }

#[ verus_verify ( external ) ]
mod helper {}
""".strip()
        + "\n",
    )

    kinds = marker_kinds(path)

    assert "verifier_external_body" in kinds
    assert "cfg_attr_verifier_external_body" in kinds
    assert "verus_verify_external" in kinds


def test_reports_start_line_for_multiline_attribute(tmp_path: Path) -> None:
    path = write_fixture(
        tmp_path,
        """
fn safe() {}
#[cfg_attr(
    any(verus_keep_ghost, feature = "proof"),
    verifier::external_body,
)]
fn digest(bytes: &[u8]) -> u64 { 0 }
""".strip()
        + "\n",
    )

    markers = trust_scan_adapter.scan_paths([path])
    cfg_markers = [m for m in markers if m["kind"] == "cfg_attr_verifier_external_body"]

    assert len(cfg_markers) == 1
    assert cfg_markers[0]["line"] == 2
    assert cfg_markers[0]["scan"] == "token-tree"


def test_cli_outputs_json_with_scope_labels(tmp_path: Path) -> None:
    changed = write_fixture(tmp_path, "assume(false);\n")
    closure = tmp_path / "closure.rs"
    closure.write_text("broadcast axiom lemma();\n", encoding="utf-8")

    proc = subprocess.run(
        [
            sys.executable,
            str(Path(trust_scan_adapter.__file__).resolve()),
            "--json",
            "--changed",
            str(changed),
            "--closure",
            str(closure),
        ],
        check=False,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )

    assert proc.returncode == 1
    payload = json.loads(proc.stdout)
    assert {hit["scope"] for hit in payload["markers"]} == {"changed", "closure"}
    assert {hit["kind"] for hit in payload["markers"]} == {"assume", "broadcast_axiom"}


def test_cli_accepts_repeated_scope_flags(tmp_path: Path) -> None:
    first = write_fixture(tmp_path, "assume(false);\n")
    second = tmp_path / "second.rs"
    second.write_text("admit();\n", encoding="utf-8")

    proc = subprocess.run(
        [
            sys.executable,
            str(Path(trust_scan_adapter.__file__).resolve()),
            "--json",
            "--changed",
            str(first),
            "--changed",
            str(second),
        ],
        check=False,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )

    assert proc.returncode == 1
    payload = json.loads(proc.stdout)
    assert {hit["kind"] for hit in payload["markers"]} == {"assume", "admit"}
    assert {hit["scope"] for hit in payload["markers"]} == {"changed"}
