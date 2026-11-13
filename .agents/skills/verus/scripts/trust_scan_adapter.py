#!/usr/bin/env python3
"""Portable Verus trust-marker scanner.

This is a lightweight token scanner for skill/eval use, not a Rust parser. It
handles Rust attributes as balanced token trees before falling back to same-line
text patterns. Multiline and nested cfg_attr markers are therefore visible to
reviewers instead of being hidden by grep-only scans.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Iterable

TEXT_PATTERNS: tuple[tuple[str, str], ...] = (
    ("assume", r"assume\s*\("),
    ("admit", r"admit\s*\("),
    ("assume_specification", r"assume_specification"),
    ("assume_termination", r"assume_termination"),
    ("external_type_specification", r"external_type_specification"),
    ("external_trait_specification", r"external_trait_specification"),
    ("external_trait_extension", r"external_trait_extension"),
    ("exec_spec_unverified", r"exec_spec_unverified\s*!"),
    ("external_fn_specification", r"external_fn_specification"),
    ("broadcast_axiom", r"broadcast\s+axiom"),
    ("axiom", r"\baxiom\b"),
    ("cfg_verus_only", r"cfg\s*\([^)]*verus_only"),
    ("code_parity_signal", r"when_used_as_spec|dual_spec|spec_exec|exec_spec"),
)

EXTERNAL_BODY_RE = re.compile(r"verifier\s*(?:::|\()\s*external_body\b", re.IGNORECASE)
EXTERNAL_RE = re.compile(r"verifier\s*(?:::|\()\s*external\b", re.IGNORECASE)
VERUS_VERIFY_BODY_RE = re.compile(r"verus_verify\s*\(\s*external_body\b", re.IGNORECASE)
VERUS_VERIFY_RE = re.compile(r"verus_verify\s*\(\s*external\b", re.IGNORECASE)
CFG_ATTR_RE = re.compile(r"cfg_attr\s*\(", re.IGNORECASE)
VERUS_ONLY_RE = re.compile(r"\bverus_only\b", re.IGNORECASE)
ATTR_START_RE = re.compile(r"#\s*\[")


def line_starts(text: str) -> list[int]:
    starts = [0]
    for index, char in enumerate(text):
        if char == "\n":
            starts.append(index + 1)
    return starts


def line_col(starts: list[int], offset: int) -> tuple[int, int]:
    line = 0
    for next_line, start in enumerate(starts):
        if start > offset:
            break
        line = next_line
    return line + 1, offset - starts[line] + 1


def close_attribute(text: str, bracket_start: int) -> int | None:
    depth = 0
    index = bracket_start
    in_string = False
    escape = False
    while index < len(text):
        char = text[index]
        if in_string:
            if escape:
                escape = False
            elif char == "\\":
                escape = True
            elif char == '"':
                in_string = False
        else:
            if char == '"':
                in_string = True
            elif char == "[":
                depth += 1
            elif char == "]":
                depth -= 1
                if depth == 0:
                    return index + 1
        index += 1
    return None


def iter_attributes(text: str) -> Iterable[tuple[int, int, str]]:
    for match in ATTR_START_RE.finditer(text):
        bracket_start = text.find("[", match.start())
        if bracket_start < 0:
            continue
        end = close_attribute(text, bracket_start)
        if end is not None:
            yield match.start(), end, text[match.start() : end]


def make_marker(path: Path, scope: str, starts: list[int], offset: int, kind: str, scan: str, text: str) -> dict[str, object]:
    line, column = line_col(starts, offset)
    return {
        "file": str(path),
        "line": line,
        "column": column,
        "scope": scope,
        "kind": kind,
        "scan": scan,
        "text": " ".join(text.strip().split()),
    }


def attribute_markers(path: Path, scope: str, text: str, starts: list[int]) -> list[dict[str, object]]:
    markers: list[dict[str, object]] = []
    for start, _end, attr in iter_attributes(text):
        if CFG_ATTR_RE.search(attr):
            if EXTERNAL_BODY_RE.search(attr) or VERUS_VERIFY_BODY_RE.search(attr):
                kind = "cfg_attr_verifier_external_body"
            elif EXTERNAL_RE.search(attr) or VERUS_VERIFY_RE.search(attr):
                kind = "cfg_attr_verifier_external"
            elif VERUS_ONLY_RE.search(attr):
                kind = "cfg_attr_verus_only"
            else:
                kind = "cfg_attr"
            markers.append(make_marker(path, scope, starts, start, kind, "token-tree", attr))
            continue
        if EXTERNAL_BODY_RE.search(attr):
            markers.append(make_marker(path, scope, starts, start, "verifier_external_body", "token-tree", attr))
        elif EXTERNAL_RE.search(attr):
            markers.append(make_marker(path, scope, starts, start, "verifier_external", "token-tree", attr))
        elif VERUS_VERIFY_BODY_RE.search(attr):
            markers.append(make_marker(path, scope, starts, start, "verus_verify_external_body", "token-tree", attr))
        elif VERUS_VERIFY_RE.search(attr):
            markers.append(make_marker(path, scope, starts, start, "verus_verify_external", "token-tree", attr))
    return markers


def text_markers(path: Path, scope: str, text: str, starts: list[int]) -> list[dict[str, object]]:
    markers: list[dict[str, object]] = []
    occupied_axiom_spans: list[range] = []
    for kind, pattern in TEXT_PATTERNS:
        for match in re.finditer(pattern, text, flags=re.IGNORECASE | re.MULTILINE):
            span = range(match.start(), match.end())
            if kind == "broadcast_axiom":
                occupied_axiom_spans.append(span)
            if kind == "axiom" and any(match.start() in used for used in occupied_axiom_spans):
                continue
            markers.append(make_marker(path, scope, starts, match.start(), kind, "text", match.group(0)))
    return markers


def scan_file(path: Path, scope: str = "scope") -> list[dict[str, object]]:
    text = path.read_text(encoding="utf-8")
    starts = line_starts(text)
    markers = attribute_markers(path, scope, text, starts)
    markers.extend(text_markers(path, scope, text, starts))
    return sorted(markers, key=lambda item: (str(item["file"]), int(item["line"]), int(item["column"]), str(item["kind"])))


def scan_paths(paths: Iterable[Path | str], scope: str = "scope") -> list[dict[str, object]]:
    markers: list[dict[str, object]] = []
    for raw_path in paths:
        path = Path(raw_path)
        if path.is_dir():
            for child in sorted(path.rglob("*.rs")):
                markers.extend(scan_file(child, scope))
        else:
            markers.extend(scan_file(path, scope))
    return sorted(markers, key=lambda item: (str(item["file"]), int(item["line"]), int(item["column"]), str(item["kind"])))


def flatten_path_groups(groups: Iterable[Iterable[Path]]) -> list[Path]:
    return [path for group in groups for path in group]


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Scan Rust/Verus files for trust markers.")
    parser.add_argument("paths", nargs="*", type=Path, help="Files or directories to scan when scopes are not split.")
    parser.add_argument("--changed", nargs="+", action="append", type=Path, default=[], help="Changed proof-relevant files.")
    parser.add_argument("--closure", nargs="+", action="append", type=Path, default=[], help="Verifier trust-closure files or directories.")
    parser.add_argument("--json", action="store_true", help="Emit JSON instead of text.")
    parser.add_argument("--no-fail", action="store_true", help="Exit 0 even when markers are found.")
    return parser


def run(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    changed_paths = flatten_path_groups(args.changed)
    closure_paths = flatten_path_groups(args.closure)
    markers: list[dict[str, object]] = []
    if changed_paths or closure_paths:
        markers.extend(scan_paths(changed_paths, "changed"))
        markers.extend(scan_paths(closure_paths, "closure"))
    markers.extend(scan_paths(args.paths, "scope"))
    payload = {"ok": not markers, "marker_count": len(markers), "markers": markers}
    if args.json:
        print(json.dumps(payload, indent=2, sort_keys=True))
    else:
        for marker in markers:
            print(f"{marker['file']}:{marker['line']}:{marker['column']}: {marker['scope']} {marker['kind']} [{marker['scan']}]: {marker['text']}")
        if not markers:
            print("No trust markers found")
    if markers and not args.no_fail:
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(run())
